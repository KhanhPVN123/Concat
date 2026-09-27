// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Concat & DemoForge contributors

//! Native cursor monitoring thread.
//!
//! Tracks high-resolution mouse position, clicks, and cursor shape
//! without installing WH_MOUSE_LL hooks that interfere with other apps.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use std::sync::Mutex;

use crate::telemetry::{CursorPoint, InteractionType, TelemetrySession};

#[cfg(windows)]
mod win32 {
    #[repr(C)]
    #[derive(Copy, Clone, Default, Debug)]
    pub struct POINT {
        pub x: i32,
        pub y: i32,
    }

    pub const SM_CXSCREEN: i32 = 0;
    pub const SM_CYSCREEN: i32 = 1;
    pub const VK_LBUTTON: i32 = 0x01;
    pub const VK_RBUTTON: i32 = 0x02;
    pub const _VK_MBUTTON: i32 = 0x04;

    #[link(name = "user32")]
    unsafe extern "system" {
        pub fn GetCursorPos(lpPoint: *mut POINT) -> i32;
        pub fn GetSystemMetrics(nIndex: i32) -> i32;
        pub fn GetAsyncKeyState(vKey: i32) -> i16;
    }
}

/// Active cursor monitoring session handle.
pub struct CursorMonitorHandle {
    running: Arc<AtomicBool>,
    session: Arc<Mutex<TelemetrySession>>,
}

impl CursorMonitorHandle {
    /// Stops the monitor thread and returns the collected telemetry session.
    pub fn stop(self) -> TelemetrySession {
        self.running.store(false, Ordering::SeqCst);
        let guard = self.session.lock().expect("session lock");
        guard.clone()
    }
}

/// Starts cursor tracking in a background worker thread.
pub fn start_cursor_monitor() -> CursorMonitorHandle {
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    #[cfg(windows)]
    let (screen_w, screen_h) = unsafe {
        let w = win32::GetSystemMetrics(win32::SM_CXSCREEN).max(1) as u32;
        let h = win32::GetSystemMetrics(win32::SM_CYSCREEN).max(1) as u32;
        (w, h)
    };

    #[cfg(not(windows))]
    let (screen_w, screen_h) = (1920, 1080);

    let session = Arc::new(Mutex::new(TelemetrySession::new(screen_w, screen_h)));
    let session_clone = session.clone();

    thread::spawn(move || {
        let start_time = Instant::now();
        let mut prev_left = false;
        let mut prev_right = false;

        while running_clone.load(Ordering::Relaxed) {
            let elapsed_ms = start_time.elapsed().as_millis() as u64;

            #[cfg(windows)]
            {
                let mut pt = win32::POINT::default();
                let has_pos = unsafe { win32::GetCursorPos(&mut pt) } != 0;
                let (cx, cy) = if has_pos {
                    (pt.x as f32 / screen_w as f32, pt.y as f32 / screen_h as f32)
                } else {
                    (0.5, 0.5)
                };

                let left_down = unsafe { (win32::GetAsyncKeyState(win32::VK_LBUTTON) as i32 & 0x8000) != 0 };
                let right_down = unsafe { (win32::GetAsyncKeyState(win32::VK_RBUTTON) as i32 & 0x8000) != 0 };

                let mut interaction = InteractionType::None;

                if left_down && !prev_left {
                    interaction = InteractionType::Click;
                } else if left_down && prev_left {
                    interaction = InteractionType::Drag;
                } else if right_down && !prev_right {
                    interaction = InteractionType::RightClick;
                }

                prev_left = left_down;
                prev_right = right_down;

                let point = CursorPoint::new(
                    elapsed_ms,
                    cx,
                    cy,
                    interaction,
                    if left_down { "pointer" } else { "arrow" },
                );

                if let Ok(mut s) = session_clone.lock() {
                    s.push(point);
                }
            }

            #[cfg(not(windows))]
            {
                // Fallback / simulation for non-Windows builds (e.g. CI)
                let point = CursorPoint::new(elapsed_ms, 0.5, 0.5, InteractionType::None, "arrow");
                if let Ok(mut s) = session_clone.lock() {
                    s.push(point);
                }
            }

            thread::sleep(Duration::from_millis(16)); // ~60 Hz
        }
    });

    CursorMonitorHandle { running, session }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_monitor_starts_and_stops() {
        let handle = start_cursor_monitor();
        let start = Instant::now();
        while start.elapsed() < Duration::from_millis(500) {
            if handle.session.lock().unwrap().len() >= 1 {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        let session = handle.stop();
        assert!(session.points.len() >= 1);
    }
}
