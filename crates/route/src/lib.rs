//! Core domain logic including the [`Storage`] persistence seam, the [`Route`] geometry, and
//! [`compute_progress`], which turns a rowed distance into a [`Progress`] snapshot.

mod route;

pub use route::Route;

use api_types::{Entry, Progress};

/// Persistence seam for rowing entries. Implemented by storage backends (see the `storage-sqlite`
/// crate), so the server depends on this trait rather than a concrete database.
pub trait Storage: Send + Sync + 'static {
    /// Record a row of `meters` and return the newly created entry.
    fn add_entry(&self, meters: i32) -> anyhow::Result<Entry>;

    /// List all entries, with the newest first.
    fn list_entries(&self) -> anyhow::Result<Vec<Entry>>;

    /// Sum of meters across all entries (0 when empty).
    fn total_meters(&self) -> anyhow::Result<i32>;

    /// Delete all entries, resetting progress to zero.
    fn reset(&self) -> anyhow::Result<()>;
}

/// Build a [`Progress`] snapshot from a total rowed distance and a route. The distance is clamped
/// to the route length, so `total_meters` means "distance along the route" and all stats cap at 100%.
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
