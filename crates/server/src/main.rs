//! Binary entry point that wires up storage and the route and starts the axum server on
//! `127.0.0.1:4800`.

mod router;

use std::sync::Arc;

use api_types::Snapshot;
use route::Route;
use storage_sqlite::SqliteStorage;
use tokio::sync::broadcast;

/// Shared application state passed to every request handler.
#[derive(Clone)]
pub struct AppState {
    /// Persistence backend for rowing entries.
    pub storage: Arc<dyn storage::Storage>,
    /// The fixed route progress is measured against.
    pub route: Arc<Route>,
    /// Broadcasts a fresh snapshot to all connected SSE clients on every change.
    pub sender: broadcast::Sender<Snapshot>,
}

/// Start the server. Open the database, build the router, bind the listener, and serve until shut
/// down.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (sender, _) = broadcast::channel(16);
    let state = AppState {
        storage: Arc::new(SqliteStorage::open("rowing.db")?),
        route: Arc::new(Route::worlds_toughest_row()),
        sender,
    };

    let app = router::router(state);
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 4800));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    println!("listening on http://{addr}");
    axum::serve(listener, app).await?;

    Ok(())
}
