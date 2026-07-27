//! Binary entry point that wires up storage and the route, starts the axum server on
//! `127.0.0.1:8080`, and opens the app in the browser.

mod router;

use std::sync::Arc;

use tokio::sync::broadcast;
use api_types::Snapshot;
use route::Route;
use storage_sqlite::SqliteStorage;

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

/// Start the server. Open the database, build the router, bind the listener, open a browser,
/// and serve until shut down.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (sender, _) = broadcast::channel(16);
    let state = AppState {
        storage: Arc::new(SqliteStorage::open("rowing.db")?),
        route: Arc::new(Route::worlds_toughest_row()),
        sender,
    };

    let app = router::router(state);
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    println!("listening on http://{addr}");
    let _ = webbrowser::open(&format!("http://{addr}"));
    axum::serve(listener, app).await?;

    Ok(())
}
