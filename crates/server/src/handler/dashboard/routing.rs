use crate::AppState;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use prism_core::routing::config::{ModelResolution, RouteProfile, RouteRule};
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct UpdateRoutingRequest {
    pub default_profile: Option<String>,
    pub profiles: Option<HashMap<String, RouteProfile>>,
    pub rules: Option<Vec<RouteRule>>,
    pub model_resolution: Option<ModelResolution>,
}

/// GET /api/dashboard/routing
pub async fn get_routing(State(state): State<AppState>) -> impl IntoResponse {
    let config = state.config.load();
    (StatusCode::OK, Json(json!(&config.routing)))
}

/// PATCH /api/dashboard/routing
pub async fn update_routing(
    State(state): State<AppState>,
    Json(body): Json<UpdateRoutingRequest>,
) -> impl IntoResponse {
    match super::providers::update_config_file_public(&state, move |config| {
        if let Some(dp) = body.default_profile {
            config.routing.default_profile = dp;
        }
        if let Some(p) = body.profiles {
            config.routing.profiles = p;
        }
        if let Some(r) = body.rules {
            config.routing.rules = r;
        }
        if let Some(mr) = body.model_resolution {
            config.routing.model_resolution = mr;
        }
    })
    .await
    {
        Ok(()) => (
            StatusCode::OK,
            Json(json!({"message": "Routing configuration updated successfully"})),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "write_failed", "message": e})),
        ),
    }
}
