// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Concat & DemoForge contributors

//! Auto-zoom region generation and 3D tilt calculation based on cursor telemetry.

use serde::{Deserialize, Serialize};
use crate::telemetry::{CursorPoint, InteractionType, TelemetrySession};

/// A generated zoom region on the timeline.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ZoomRegion {
    /// Unique identifier for this zoom span.
    pub id: String,
    /// Timeline start in milliseconds.
    pub start_ms: u64,
    /// Timeline end in milliseconds.
    pub end_ms: u64,
    /// Normalized focus center X (0.0 to 1.0).
    pub focus_x: f32,
    /// Normalized focus center Y (0.0 to 1.0).
    pub focus_y: f32,
    /// Zoom level percentage (100 = 1x, 200 = 2x).
    pub zoom: f32,
    /// 3D pitch tilt angle in degrees.
    pub pitch: f32,
    /// 3D yaw tilt angle in degrees.
    pub yaw: f32,
}

/// Configuration settings for auto-zoom generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoZoomConfig {
    /// Pre-roll duration before a click in milliseconds (default: 350ms).
    pub lead_in_ms: u64,
    /// Hold duration after a click in milliseconds (default: 1200ms).
    pub hold_ms: u64,
    /// Maximum gap to bridge between consecutive clicks (default: 1500ms).
    pub max_gap_merge_ms: u64,
    /// Target zoom scale percentage (default: 180.0%).
    pub zoom_level: f32,
    /// Maximum 3D yaw angle tilt in degrees (default: 6.0°).
    pub max_yaw_deg: f32,
    /// Maximum 3D pitch angle tilt in degrees (default: 4.0°).
    pub max_pitch_deg: f32,
}

impl Default for AutoZoomConfig {
    fn default() -> Self {
        Self {
            lead_in_ms: 350,
            hold_ms: 1200,
            max_gap_merge_ms: 1500,
            zoom_level: 180.0,
            max_yaw_deg: 6.0,
            max_pitch_deg: 4.0,
        }
    }
}

/// Generates a list of `ZoomRegion`s from a recorded `TelemetrySession`.
pub fn generate_auto_zoom_regions(session: &TelemetrySession, config: &AutoZoomConfig) -> Vec<ZoomRegion> {
    if session.points.is_empty() {
        return Vec::new();
    }

    // 1. Identify click events
    let mut click_points: Vec<&CursorPoint> = session
        .points
        .iter()
        .filter(|pt| matches!(pt.interaction, InteractionType::Click | InteractionType::DoubleClick))
        .collect();

    if click_points.is_empty() {
        return Vec::new();
    }

    // Sort by timestamp
    click_points.sort_by_key(|pt| pt.time_ms);

    // 2. Cluster clicks that occur within max_gap_merge_ms
    let mut raw_regions: Vec<ZoomRegion> = Vec::new();

    for (idx, click) in click_points.into_iter().enumerate() {
        let start = click.time_ms.saturating_sub(config.lead_in_ms);
        let end = click.time_ms + config.hold_ms;

        // Calculate 3D tilt based on screen position
        // Center is (0.5, 0.5). If focus is on the left (cx < 0.5), yaw tilts right (+).
        let rel_x = click.cx - 0.5;
        let rel_y = click.cy - 0.5;

        let yaw = -rel_x * 2.0 * config.max_yaw_deg;
        let pitch = rel_y * 2.0 * config.max_pitch_deg;

        if let Some(last) = raw_regions.last_mut() {
            if start <= last.end_ms + config.max_gap_merge_ms {
                // Merge into the previous region
                last.end_ms = last.end_ms.max(end);
                // Weighted average for center focus
                last.focus_x = (last.focus_x + click.cx) * 0.5;
                last.focus_y = (last.focus_y + click.cy) * 0.5;
                last.yaw = (last.yaw + yaw) * 0.5;
                last.pitch = (last.pitch + pitch) * 0.5;
                continue;
            }
        }

        raw_regions.push(ZoomRegion {
            id: format!("zoom-{}", idx + 1),
            start_ms: start,
            end_ms: end,
            focus_x: click.cx,
            focus_y: click.cy,
            zoom: config.zoom_level,
            pitch,
            yaw,
        });
    }

    raw_regions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_regions_from_clicks() {
        let mut session = TelemetrySession::new(1920, 1080);
        session.push(CursorPoint::new(1000, 0.2, 0.3, InteractionType::Click, "pointer"));
        session.push(CursorPoint::new(5000, 0.8, 0.7, InteractionType::Click, "pointer"));

        let config = AutoZoomConfig::default();
        let regions = generate_auto_zoom_regions(&session, &config);

        assert_eq!(regions.len(), 2);
        assert_eq!(regions[0].start_ms, 650);
        assert_eq!(regions[0].end_ms, 2200);
        assert_eq!(regions[0].focus_x, 0.2);
        assert!(regions[0].yaw > 0.0); // Tilted facing towards the left focus

        assert_eq!(regions[1].start_ms, 4650);
        assert_eq!(regions[1].end_ms, 6200);
        assert_eq!(regions[1].focus_x, 0.8);
        assert!(regions[1].yaw < 0.0); // Tilted facing towards the right focus
    }

    #[test]
    fn merges_close_clicks_into_one_region() {
        let mut session = TelemetrySession::new(1920, 1080);
        session.push(CursorPoint::new(1000, 0.4, 0.4, InteractionType::Click, "pointer"));
        session.push(CursorPoint::new(1500, 0.42, 0.42, InteractionType::Click, "pointer"));

        let config = AutoZoomConfig::default();
        let regions = generate_auto_zoom_regions(&session, &config);

        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].start_ms, 650);
        assert_eq!(regions[0].end_ms, 2700);
    }
}
