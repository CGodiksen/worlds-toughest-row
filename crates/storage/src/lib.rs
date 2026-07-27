//! The storage engine seam, including a [`Storage`] trait that persistence backends implement,
//! decoupling the server from any concrete database.

use api_types::Entry;

/// Trait implemented by storage backends (see the `storage-sqlite` crate) so the server depends on
/// this trait rather than a concrete database.
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
