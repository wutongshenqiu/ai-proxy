use crate::AppState;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use prism_core::routing::config::{ModelResolution, RouteProfile, RouteRule, RoutingConfig};
use prism_core::routing::explain::explain;
use prism_core::routing::planner::RoutePlanner;
use prism_core::routing::types::RouteRequestFeatures;
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
    let current_routing = state.config.load().routing.clone();
    let effective_routing = materialize_routing_update(&body, &current_routing);
    if let Err(errors) = validate_effective_routing(&effective_routing) {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({"error": "validation_failed", "details": errors})),
        );
    }

    match super::config_tx::update_config_file_public(&state, move |config| {
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
        Ok(new_version) => {
            tracing::info!("Routing configuration updated via dashboard");
            (
                StatusCode::OK,
                Json(
                    json!({"message": "Routing configuration updated successfully", "config_version": new_version}),
                ),
            )
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to update routing configuration");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "write_failed", "message": e})),
            )
        }
    }
}

/// POST /api/dashboard/routing/preview — lightweight introspection (no scoring detail)
pub async fn preview_route(
    State(state): State<AppState>,
    Json(req): Json<RouteIntrospectionRequest>,
) -> impl IntoResponse {
    let features = req.to_features();
    let config = state.config.load();
    let inventory = state.catalog.snapshot();
    let health = state.health_manager.snapshot();
    let routing = match resolve_routing_override(req.routing_override, &config.routing) {
        Ok(routing) => routing,
        Err(errors) => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({"error": "validation_failed", "details": errors})),
            );
        }
    };

    let plan = RoutePlanner::plan(&features, &routing, &inventory, &health);
    let mut explanation = explain(&plan);
    // Preview omits detailed scoring
    explanation.scoring.clear();

    (StatusCode::OK, Json(json!(explanation)))
}

/// POST /api/dashboard/routing/explain — full introspection with scoring detail
pub async fn explain_route(
    State(state): State<AppState>,
    Json(req): Json<RouteIntrospectionRequest>,
) -> impl IntoResponse {
    let features = req.to_features();
    let config = state.config.load();
    let inventory = state.catalog.snapshot();
    let health = state.health_manager.snapshot();
    let routing = match resolve_routing_override(req.routing_override, &config.routing) {
        Ok(routing) => routing,
        Err(errors) => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({"error": "validation_failed", "details": errors})),
            );
        }
    };

    let plan = RoutePlanner::plan(&features, &routing, &inventory, &health);
    let explanation = explain(&plan);

    (StatusCode::OK, Json(json!(explanation)))
}

/// Canonical route introspection request shared by preview and explain endpoints.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RouteIntrospectionRequest {
    pub model: String,
    #[serde(default = "default_endpoint")]
    pub endpoint: String,
    #[serde(default = "default_source_format")]
    pub source_format: String,
    pub tenant_id: Option<String>,
    pub api_key_id: Option<String>,
    pub region: Option<String>,
    #[serde(default)]
    pub stream: bool,
    #[serde(default)]
    pub headers: std::collections::BTreeMap<String, String>,
    #[serde(default)]
    pub routing_override: Option<RoutingConfig>,
}

fn default_endpoint() -> String {
    "chat-completions".into()
}

fn default_source_format() -> String {
    "openai".to_string()
}

impl RouteIntrospectionRequest {
    pub fn to_features(&self) -> RouteRequestFeatures {
        use prism_core::provider::Format;
        use prism_core::routing::types::RouteEndpoint;

        let endpoint = match self.endpoint.as_str() {
            "messages" => RouteEndpoint::Messages,
            "responses" => RouteEndpoint::Responses,
            "generate-content" | "generate_content" => RouteEndpoint::GenerateContent,
            "stream-generate-content" => RouteEndpoint::StreamGenerateContent,
            "models" => RouteEndpoint::Models,
            _ => RouteEndpoint::ChatCompletions,
        };

        let source_format = match self.source_format.as_str() {
            "claude" => Format::Claude,
            "gemini" => Format::Gemini,
            _ => Format::OpenAI,
        };

        RouteRequestFeatures {
            requested_model: self.model.clone(),
            endpoint,
            source_format,
            tenant_id: self.tenant_id.clone(),
            api_key_id: self.api_key_id.clone(),
            region: self.region.clone(),
            stream: self.stream,
            headers: self.headers.clone(),
            allowed_credentials: Vec::new(),
            required_capabilities: None,
        }
    }
}

fn resolve_routing_override(
    routing_override: Option<RoutingConfig>,
    current: &RoutingConfig,
) -> Result<RoutingConfig, Vec<String>> {
    match routing_override {
        Some(routing) => {
            validate_effective_routing(&routing)?;
            Ok(routing)
        }
        None => Ok(current.clone()),
    }
}

fn materialize_routing_update(
    body: &UpdateRoutingRequest,
    current: &RoutingConfig,
) -> RoutingConfig {
    let mut next = current.clone();
    if let Some(default_profile) = &body.default_profile {
        next.default_profile = default_profile.clone();
    }
    if let Some(profiles) = &body.profiles {
        next.profiles = profiles.clone();
    }
    if let Some(rules) = &body.rules {
        next.rules = rules.clone();
    }
    if let Some(model_resolution) = &body.model_resolution {
        next.model_resolution = model_resolution.clone();
    }
    next
}

fn validate_effective_routing(routing: &RoutingConfig) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if routing.profiles.is_empty() {
        errors.push("profiles map must not be empty".to_string());
    }

    for (name, profile) in &routing.profiles {
        validate_profile(name, profile, &mut errors);
    }

    if !routing.profiles.contains_key(&routing.default_profile) {
        errors.push(format!(
            "default-profile '{}' does not exist in profiles",
            routing.default_profile
        ));
    }

    for rule in &routing.rules {
        if !routing.profiles.contains_key(&rule.use_profile) {
            errors.push(format!(
                "rule '{}' references non-existent profile '{}'",
                rule.name, rule.use_profile
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_profile(name: &str, profile: &RouteProfile, errors: &mut Vec<String>) {
    if let Err(error) = profile.validate() {
        errors.push(format!("profile '{}': {}", name, error));
    }
}
