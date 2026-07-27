//! Data-transfer types shared between the Rust backend and the web UI.
//!
//! These structs are the single source of truth for the API's JSON shapes. `#[derive(TS)]` exports
//! matching TypeScript definitions into `web/src/bindings/` when `cargo test -p api-types` is run.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A geographic coordinate in decimal degrees.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LatLng {
    /// Latitude in degrees.
    pub lat: f64,
    /// Longitude in degrees.
    pub lng: f64,
}

/// A single logged rowing session.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Entry {
    /// Auto-incrementing database identifier.
    pub id: i32,
    /// Distance rowed in this entry, in meters.
    pub meters: i32,
    /// Timestamp the entry was recorded (SQLite `datetime('now')`, UTC).
    pub created_at: String,
}

/// A snapshot of overall progress along the route.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Progress {
    /// Distance covered along the route so far, in meters (clamped to route length).
    pub total_meters: i32,
    /// Distance still remaining to the finish, in meters.
    pub meters_remaining: i32,
    /// Fraction of the route completed, as a percentage (0–100).
    pub percent_complete: f64,
    /// Current interpolated position of the boat on the route.
    pub position: LatLng,
    /// The full route as an ordered list of waypoints.
    pub route: Vec<LatLng>,
    /// The path traveled so far, from the start to `position`.
    pub trail: Vec<LatLng>,
}

/// Request body for logging a row via `POST /api/entries`.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AdvanceRequest {
    /// Meters rowed. When omitted, the server defaults to 500 m.
    pub meters: Option<i32>,
}
