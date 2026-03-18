mod response;
mod view;

use crate::AppState;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;

use self::response::ProviderListResponse;
use self::view::{provider_detail_response, summarize_provider};

/// GET /api/dashboard/providers
pub async fn list_providers(State(state): State<AppState>) -> impl IntoResponse {
    let config = state.config.load();
    let providers = config
        .providers
        .iter()
        .map(|entry| summarize_provider(&state, entry))
        .collect::<Vec<_>>();

    (StatusCode::OK, Json(ProviderListResponse { providers }))
}

/// GET /api/dashboard/providers/:name
pub async fn get_provider(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let config = state.config.load();

    match config.providers.iter().find(|entry| entry.name == name) {
        Some(entry) => (
            StatusCode::OK,
            Json(provider_detail_response(&state, entry)),
        )
            .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "not_found", "message": "Provider not found"})),
        )
            .into_response(),
    }
}
