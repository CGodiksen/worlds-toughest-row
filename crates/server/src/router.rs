//! HTTP routing. The JSON API handlers, an error wrapper that maps failures to `500` responses, and
//! a fallback that serves the embedded web UI.

use api_types::{AdvanceRequest, Entry, Progress};
use axum::{
    Json, Router,
    extract::State,
    http::{StatusCode, Uri, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use route::compute_progress;
use rust_embed::RustEmbed;

use crate::AppState;

/// Build the axum router including the `/api` routes plus a fallback serving the embedded UI.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/progress", get(get_progress))
        .route(
            "/api/entries",
            post(advance).get(list_entries).delete(reset),
        )
        .fallback(static_handler)
        .with_state(state)
}

/// `GET /api/progress` - return the current progress snapshot.
async fn get_progress(State(st): State<AppState>) -> Result<Json<Progress>, AppError> {
    let total = st.storage.total_meters()?;
    Ok(Json(compute_progress(total, &st.route)))
}

/// `POST /api/entries` - log a row (default 500 m) and return updated progress.
async fn advance(
    State(st): State<AppState>,
    Json(req): Json<AdvanceRequest>,
) -> Result<Json<Progress>, AppError> {
    let meters = req.meters.unwrap_or(500);
    st.storage.add_entry(meters)?;

    let total = st.storage.total_meters()?;
    Ok(Json(compute_progress(total, &st.route)))
}

/// `GET /api/entries` - return all logged entries, with the newest first.
async fn list_entries(State(st): State<AppState>) -> Result<Json<Vec<Entry>>, AppError> {
    Ok(Json(st.storage.list_entries()?))
}

/// `DELETE /api/entries` - clear all entries and return the reset progress.
async fn reset(State(st): State<AppState>) -> Result<Json<Progress>, AppError> {
    st.storage.reset()?;
    let total = st.storage.total_meters()?;

    Ok(Json(compute_progress(total, &st.route)))
}

/// Wraps any error so it can be returned as a `500 Internal Server Error`.
struct AppError(anyhow::Error);

impl From<anyhow::Error> for AppError {
    /// Convert any `anyhow::Error` into an `AppError`.
    fn from(e: anyhow::Error) -> Self {
        AppError(e)
    }
}
impl IntoResponse for AppError {
    /// Render the error as a `500` response with its message as the body.
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string()).into_response()
    }
}

/// The built web UI (`web/dist`), embedded into the binary at compile time.
#[derive(RustEmbed)]
#[folder = "../../web/dist"]
struct Assets;

/// Serve an embedded static asset, falling back to `index.html` (for SPA routing).
async fn static_handler(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    match Assets::get(path).or_else(|| Assets::get("index.html")) {
        Some(file) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], file.data).into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
