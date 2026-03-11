use crate::AppState;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;
use serde_json::json;

fn parse_routing_strategy(s: &str) -> Option<prism_core::config::RoutingStrategy> {
    match s {
        "round-robin" | "RoundRobin" => Some(prism_core::config::RoutingStrategy::RoundRobin),
        "fill-first" | "FillFirst" => Some(prism_core::config::RoutingStrategy::FillFirst),
        "latency-aware" | "LatencyAware" => Some(prism_core::config::RoutingStrategy::LatencyAware),
        "geo-aware" | "GeoAware" => Some(prism_core::config::RoutingStrategy::GeoAware),
        _ => None,
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateRoutingRequest {
    pub strategy: Option<String>,
    pub request_retry: Option<u32>,
    pub max_retry_interval: Option<u64>,
    pub fallback_enabled: Option<bool>,
    pub model_strategies: Option<std::collections::HashMap<String, String>>,
    pub model_fallbacks: Option<std::collections::HashMap<String, Vec<String>>>,
}

/// GET /api/dashboard/routing
pub async fn get_routing(State(state): State<AppState>) -> impl IntoResponse {
    let config = state.config.load();
    (
        StatusCode::OK,
        Json(json!({
            "strategy": config.routing.strategy,
            "fallback_enabled": config.routing.fallback_enabled,
            "request_retry": config.request_retry,
            "max_retry_interval": config.max_retry_interval,
            "model_strategies": config.routing.model_strategies,
            "model_fallbacks": config.routing.model_fallbacks,
        })),
    )
}

/// PATCH /api/dashboard/routing
pub async fn update_routing(
    State(state): State<AppState>,
    Json(body): Json<UpdateRoutingRequest>,
) -> impl IntoResponse {
    let strategy = if let Some(ref s) = body.strategy {
        match parse_routing_strategy(s) {
            Some(s) => Some(s),
            None => {
                return (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(
                        json!({"error": "validation_failed", "message": "Invalid strategy. Must be 'round-robin', 'fill-first', 'latency-aware', or 'geo-aware'"}),
                    ),
                );
            }
        }
    } else {
        None
    };

    // Parse model_strategies
    let model_strategies = if let Some(ref ms) = body.model_strategies {
        let mut parsed = std::collections::HashMap::new();
        for (pattern, s) in ms {
            match parse_routing_strategy(s) {
                Some(strategy) => {
                    parsed.insert(pattern.clone(), strategy);
                }
                None => {
                    return (
                        StatusCode::UNPROCESSABLE_ENTITY,
                        Json(
                            json!({"error": "validation_failed", "message": format!("Invalid strategy '{}' for model pattern '{}'", s, pattern)}),
                        ),
                    );
                }
            }
        }
        Some(parsed)
    } else {
        None
    };

    let fallback_enabled = body.fallback_enabled;
    let request_retry = body.request_retry;
    let max_retry_interval = body.max_retry_interval;
    let model_fallbacks = body.model_fallbacks;

    match super::providers::update_config_file_public(&state, move |config| {
        if let Some(s) = strategy {
            config.routing.strategy = s;
        }
        if let Some(fb) = fallback_enabled {
            config.routing.fallback_enabled = fb;
        }
        if let Some(rr) = request_retry {
            config.request_retry = rr;
        }
        if let Some(mri) = max_retry_interval {
            config.max_retry_interval = mri;
        }
        if let Some(ms) = model_strategies {
            config.routing.model_strategies = ms;
        }
        if let Some(mf) = model_fallbacks {
            config.routing.model_fallbacks = mf;
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
