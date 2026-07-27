mod route;

pub use route::Route;

use api_types::{Entry, Progress};

pub trait Storage: Send + Sync + 'static {
    fn add_entry(&self, meters: i32) -> anyhow::Result<Entry>;
    fn list_entries(&self) -> anyhow::Result<Vec<Entry>>;
    fn total_meters(&self) -> anyhow::Result<i32>;
}

pub fn compute_progress(total_meters: i32, route: &Route) -> Progress {
    let d = (total_meters as f64).clamp(0.0, route.total_m);

    Progress {
        total_meters: d as i32,
        meters_remaining: (route.total_m - d).max(0.0) as i32,
        percent_complete: if route.total_m > 0.0 {
            d / route.total_m * 100.0
        } else {
            0.0
        },
        position: route.position_at(d),
        route: route.waypoints.clone(),
        trail: route.trail_to(d),
    }
}
