// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Concat & DemoForge contributors

//! Screen capture, mouse cursor telemetry monitoring and automatic zoom generation.
//!
//! Ported and adapted from DemoForge desktop studio.

pub mod auto_zoom;
pub mod monitor;
pub mod telemetry;

pub use auto_zoom::{AutoZoomConfig, ZoomRegion, generate_auto_zoom_regions};
pub use monitor::{CursorMonitorHandle, start_cursor_monitor};
pub use telemetry::{CursorPoint, InteractionType, TelemetrySession};
