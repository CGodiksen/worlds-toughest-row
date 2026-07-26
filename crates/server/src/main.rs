mod router;

use std::sync::Arc;
use route::Route;
use storage_sqlite::SqliteStorage;

#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<dyn route::Storage>,
    pub route: Arc<Route>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = AppState {
        storage: Arc::new(SqliteStorage::open("rowing.db")?),
        route: Arc::new(Route::worlds_toughest_row()),
    };

    let app = router::router(state);
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    println!("listening on http://{addr}");
    let _ = webbrowser::open(&format!("http://{addr}"));
    axum::serve(listener, app).await?;

    Ok(())
}
