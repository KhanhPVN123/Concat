// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Concat & DemoForge contributors

//! Cursor telemetry data structures and recording storage.

use serde::{Deserialize, Serialize};

/// Type of mouse interaction recorded at a sample instant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InteractionType {
    /// Normal cursor movement without click.
    None,
    /// Left button press / click.
    Click,
    /// Rapid double-click.
    DoubleClick,
    /// Right button press.
    RightClick,
    /// Middle button press.
    MiddleClick,
    /// Cursor moving while left button held down.
    Drag,
}

impl Default for InteractionType {
    fn default() -> Self {
        Self::None
    }
}

/// A single sampled cursor position and state in time.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CursorPoint {
    /// Timestamp in milliseconds since recording began.
    pub time_ms: u64,
    /// Normalized horizontal coordinate (0.0 at left, 1.0 at right).
    pub cx: f32,
    /// Normalized vertical coordinate (0.0 at top, 1.0 at bottom).
    pub cy: f32,
    /// The interaction event at this instant.
    pub interaction: InteractionType,
    /// The active mouse cursor icon shape (e.g., "arrow", "pointer", "text").
    pub cursor_type: String,
}

impl CursorPoint {
    /// Creates a new cursor point, clamping coordinates to `[0.0, 1.0]`.
    pub fn new(time_ms: u64, cx: f32, cy: f32, interaction: InteractionType, cursor_type: impl Into<String>) -> Self {
        Self {
            time_ms,
            cx: cx.clamp(0.0, 1.0),
            cy: cy.clamp(0.0, 1.0),
            interaction,
            cursor_type: cursor_type.into(),
        }
    }
}

/// A complete recording session of cursor telemetry.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct TelemetrySession {
    /// The target screen width in pixels during recording.
    pub screen_width: u32,
    /// The target screen height in pixels during recording.
    pub screen_height: u32,
    /// Ordered list of sampled cursor points.
    pub points: Vec<CursorPoint>,
}

impl TelemetrySession {
    /// Creates an empty telemetry session for a given screen dimension.
    pub fn new(screen_width: u32, screen_height: u32) -> Self {
        Self {
            screen_width,
            screen_height,
            points: Vec::new(),
        }
    }

    /// Records a new point into the session.
    pub fn push(&mut self, point: CursorPoint) {
        self.points.push(point);
    }

    /// Total number of points recorded.
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Returns `true` if no points have been recorded.
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// Serializes the telemetry session to JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserializes a telemetry session from JSON string.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn telemetry_serializes_and_deserializes() {
        let mut session = TelemetrySession::new(1920, 1080);
        session.push(CursorPoint::new(0, 0.5, 0.5, InteractionType::None, "arrow"));
        session.push(CursorPoint::new(150, 0.6, 0.4, InteractionType::Click, "pointer"));

        let json = session.to_json().expect("serialize");
        let restored = TelemetrySession::from_json(&json).expect("deserialize");
        assert_eq!(session, restored);
    }
}
