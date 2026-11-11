//! HTTP API routes.

pub mod library;
pub mod playback;

use crate::{API_VERSION, state::AppState};
use axum::http::header;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;

/// Creates versioned API routes and a compatibility alias without `/v1`.
pub fn router(state: AppState) -> Router {
    if let Ok(runtime) = tokio::runtime::Handle::try_current() {
        let worker_state = state.clone();
        runtime.spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                let _ = playback::pump(axum::extract::State(worker_state.clone())).await;
            }
        });
    }
    let web = Router::new()
        .route("/", get(web_index))
        .route("/assets/timbre.css", get(web_css))
        .route("/assets/timbre.js", get(web_js));
    let versioned = Router::new()
        .route("/api/v1/health", get(health))
        .merge(library::routes("/api/v1"))
        .merge(playback::routes("/api/v1"))
        .with_state(state.clone());
    let unversioned = Router::new()
        .route("/api/health", get(health))
        .merge(library::routes("/api"))
        .merge(playback::routes("/api"))
        .with_state(state);
    web.merge(versioned).merge(unversioned)
}

async fn web_index() -> ([(header::HeaderName, &'static str); 1], &'static str) {
    (
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        include_str!("../../web/index.html"),
    )
}

async fn web_css() -> ([(header::HeaderName, &'static str); 1], &'static str) {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        include_str!("../../web/timbre.css"),
    )
}

async fn web_js() -> ([(header::HeaderName, &'static str); 1], &'static str) {
    (
        [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
        include_str!("../../web/timbre.js"),
    )
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    api_version: &'static str,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        api_version: API_VERSION,
    })
}
