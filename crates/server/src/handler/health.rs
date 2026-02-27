use crate::AppState;
use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;

pub async fn health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

pub async fn metrics(State(state): State<AppState>) -> impl IntoResponse {
    Json(state.metrics.snapshot())
}
