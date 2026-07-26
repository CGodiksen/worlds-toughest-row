use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../web/src/bindings/")]
pub struct LatLng {
    pub lat: f64,
    pub lng: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../web/src/bindings/")]
pub struct Entry {
    pub id: i64,
    pub meters: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../web/src/bindings/")]
pub struct Progress {
    pub total_meters: i64,
    pub meters_remaining: i64,
    pub percent_complete: f64,
    pub position: LatLng,
    pub route: Vec<LatLng>,
    pub trail: Vec<LatLng>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../web/src/bindings/")]
pub struct AdvanceRequest {
    pub meters: Option<i64>,
}
