use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LatLng {
    pub lat: f64,
    pub lng: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Entry {
    pub id: i32,
    pub meters: i32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Progress {
    pub total_meters: i32,
    pub meters_remaining: i32,
    pub percent_complete: f64,
    pub position: LatLng,
    pub route: Vec<LatLng>,
    pub trail: Vec<LatLng>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AdvanceRequest {
    pub meters: Option<i32>,
}
