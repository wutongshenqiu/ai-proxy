use super::{
    config_tx_error_response, is_valid_format, normalize_auth_profiles, parse_upstream_kind,
    seed_runtime_oauth_states, strip_runtime_oauth_data, validate_auth_shape,
    validate_provider_auth_profiles, validation_error,
};
use crate::AppState;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use prism_core::auth_profile::AuthProfileEntry;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct CreateProviderRequest {
    pub name: String,
    pub format: String,
    #[serde(default)]
    pub upstream: Option<String>,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub auth_profiles: Vec<AuthProfileEntry>,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub proxy_url: Option<String>,
    #[serde(default)]
    pub prefix: Option<String>,
    #[serde(default)]
    pub models: Vec<String>,
    #[serde(default)]
    pub excluded_models: Vec<String>,
    #[serde(default)]
    pub headers: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub disabled: bool,
    #[serde(default)]
    pub wire_api: Option<String>,
    #[serde(default = "default_weight")]
    pub weight: u32,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default)]
    pub upstream_presentation: Option<prism_core::presentation::UpstreamPresentationConfig>,
    #[serde(default)]
    pub vertex: bool,
    #[serde(default)]
    pub vertex_project: Option<String>,
    #[serde(default)]
    pub vertex_location: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProviderRequest {
    #[serde(default)]
    pub upstream: Option<Option<String>>,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub auth_profiles: Option<Vec<AuthProfileEntry>>,
    #[serde(default)]
    pub base_url: Option<Option<String>>,
    #[serde(default)]
    pub proxy_url: Option<Option<String>>,
    #[serde(default)]
    pub prefix: Option<Option<String>>,
    #[serde(default)]
    pub models: Option<Vec<String>>,
    #[serde(default)]
    pub excluded_models: Option<Vec<String>>,
    #[serde(default)]
    pub headers: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    pub disabled: Option<bool>,
    #[serde(default)]
    pub wire_api: Option<Option<String>>,
    #[serde(default)]
    pub weight: Option<u32>,
    #[serde(default)]
    pub region: Option<Option<String>>,
    #[serde(default)]
    pub upstream_presentation: Option<Option<prism_core::presentation::UpstreamPresentationConfig>>,
    #[serde(default)]
    pub vertex: Option<bool>,
    #[serde(default)]
    pub vertex_project: Option<Option<String>>,
    #[serde(default)]
    pub vertex_location: Option<Option<String>>,
}

fn default_weight() -> u32 {
    1
}

/// POST /api/dashboard/providers
pub async fn create_provider(
    State(state): State<AppState>,
    Json(body): Json<CreateProviderRequest>,
) -> impl IntoResponse {
    if body.name.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({"error": "validation_failed", "message": "name is required"})),
        );
    }
    if !is_valid_format(&body.format) {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(
                json!({"error": "validation_failed", "message": "Invalid format. Must be one of: openai, claude, gemini"}),
            ),
        );
    }
    let format: prism_core::provider::Format = body
        .format
        .parse()
        .unwrap_or(prism_core::provider::Format::OpenAI);
    let upstream = match parse_upstream_kind(format, body.upstream.as_deref()) {
        Ok(value) => value,
        Err(response) => return response,
    };

    let auth_profiles = match normalize_auth_profiles(&body.auth_profiles) {
        Ok(profiles) => profiles,
        Err(response) => return response,
    };
    if let Err(response) = validate_auth_shape(body.api_key.as_deref(), &auth_profiles) {
        return response;
    }
    if let Err(response) =
        validate_provider_auth_profiles(format, upstream, body.base_url.as_deref(), &auth_profiles)
    {
        return response;
    }

    {
        let config = state.config.load();
        if config.providers.iter().any(|e| e.name == body.name) {
            return (
                StatusCode::CONFLICT,
                Json(
                    json!({"error": "duplicate_name", "message": format!("Provider name '{}' already exists", body.name)}),
                ),
            );
        }
    }

    let models = body
        .models
        .into_iter()
        .map(|id| prism_core::config::ModelMapping { id, alias: None })
        .collect();

    let wire_api = if upstream == prism_core::provider::UpstreamKind::Codex {
        prism_core::provider::WireApi::Responses
    } else {
        match body.wire_api.as_deref() {
            Some("responses") => prism_core::provider::WireApi::Responses,
            _ => prism_core::provider::WireApi::Chat,
        }
    };

    let provider_name = body.name.clone();
    let api_key = body.api_key.unwrap_or_default();
    let (auth_profiles, runtime_oauth_states) = strip_runtime_oauth_data(auth_profiles);

    let new_entry = prism_core::config::ProviderKeyEntry {
        name: provider_name.clone(),
        format,
        upstream: Some(upstream),
        api_key,
        base_url: body.base_url,
        proxy_url: body.proxy_url,
        prefix: body.prefix,
        models,
        excluded_models: body.excluded_models,
        headers: body.headers,
        disabled: body.disabled,
        cloak: Default::default(),
        upstream_presentation: body.upstream_presentation.unwrap_or_default(),
        wire_api,
        weight: body.weight,
        region: body.region,
        credential_source: None,
        auth_profiles,
        vertex: body.vertex,
        vertex_project: body.vertex_project,
        vertex_location: body.vertex_location,
    };
    if let Err(message) = new_entry.validate_shape() {
        return validation_error(message);
    }

    match update_config_file(&state, |config| {
        config.providers.push(new_entry.clone());
    })
    .await
    {
        Ok(()) => {
            if let Err(err) =
                seed_runtime_oauth_states(&state, &provider_name, &runtime_oauth_states)
            {
                tracing::error!(
                    name = %provider_name,
                    error = %err,
                    "Provider created but runtime oauth seeding failed"
                );
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "runtime_auth_seed_failed", "message": err})),
                );
            }
            tracing::info!(
                name = %provider_name,
                format = %body.format,
                "Provider created via dashboard"
            );
            (
                StatusCode::CREATED,
                Json(json!({"message": "Provider created successfully"})),
            )
        }
        Err(error) => {
            tracing::error!(
                name = %provider_name,
                error = ?error,
                "Failed to create provider"
            );
            config_tx_error_response(error)
        }
    }
}

/// PATCH /api/dashboard/providers/:name
pub async fn update_provider(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(body): Json<UpdateProviderRequest>,
) -> impl IntoResponse {
    let existing_entry = {
        let config = state.config.load();
        match config.providers.iter().find(|entry| entry.name == name) {
            Some(entry) => entry.clone(),
            None => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(json!({"error": "not_found", "message": "Provider not found"})),
                );
            }
        }
    };

    let name_for_log = name.clone();
    let auth_profiles = match body
        .auth_profiles
        .as_ref()
        .map(|profiles| normalize_auth_profiles(profiles))
        .transpose()
    {
        Ok(profiles) => profiles,
        Err(response) => return response,
    };
    if let Some(ref profiles) = auth_profiles
        && let Err(response) = validate_auth_shape(body.api_key.as_deref(), profiles)
    {
        return response;
    }
    let upstream = match body.upstream.as_ref() {
        Some(upstream) => match parse_upstream_kind(existing_entry.format, upstream.as_deref()) {
            Ok(value) => value,
            Err(response) => return response,
        },
        None => existing_entry.upstream_kind(),
    };
    let mut candidate_entry = existing_entry.clone();
    candidate_entry.upstream = Some(upstream);
    if let Some(ref key) = body.api_key {
        candidate_entry.api_key = key.clone();
    }
    if let Some(ref profiles) = auth_profiles {
        candidate_entry.auth_profiles = profiles.clone();
        if !profiles.is_empty() && body.api_key.is_none() {
            candidate_entry.api_key.clear();
        }
    }
    if let Some(ref url) = body.base_url {
        candidate_entry.base_url = url.clone();
    }
    if let Some(ref url) = body.proxy_url {
        candidate_entry.proxy_url = url.clone();
    }
    if let Some(ref prefix) = body.prefix {
        candidate_entry.prefix = prefix.clone();
    }
    if let Some(ref models) = body.models {
        candidate_entry.models = models
            .iter()
            .map(|id| prism_core::config::ModelMapping {
                id: id.clone(),
                alias: None,
            })
            .collect();
    }
    if let Some(ref excluded) = body.excluded_models {
        candidate_entry.excluded_models = excluded.clone();
    }
    if let Some(ref headers) = body.headers {
        candidate_entry.headers = headers.clone();
    }
    if let Some(disabled) = body.disabled {
        candidate_entry.disabled = disabled;
    }
    if upstream == prism_core::provider::UpstreamKind::Codex {
        candidate_entry.wire_api = prism_core::provider::WireApi::Responses;
    } else if let Some(ref wire_api_opt) = body.wire_api {
        candidate_entry.wire_api = match wire_api_opt.as_deref() {
            Some("responses") => prism_core::provider::WireApi::Responses,
            _ => prism_core::provider::WireApi::Chat,
        };
    }
    if let Some(weight) = body.weight {
        candidate_entry.weight = weight;
    }
    if let Some(ref region) = body.region {
        candidate_entry.region = region.clone();
    }
    if let Some(ref presentation_opt) = body.upstream_presentation {
        candidate_entry.upstream_presentation = presentation_opt.clone().unwrap_or_default();
    }
    if let Some(vertex) = body.vertex {
        candidate_entry.vertex = vertex;
    }
    if let Some(ref project) = body.vertex_project {
        candidate_entry.vertex_project = project.clone();
    }
    if let Some(ref location) = body.vertex_location {
        candidate_entry.vertex_location = location.clone();
    }

    if let Err(response) = validate_auth_shape(
        Some(candidate_entry.api_key.as_str()),
        &candidate_entry.auth_profiles,
    ) {
        return response;
    }
    if let Err(response) = validate_provider_auth_profiles(
        candidate_entry.format,
        candidate_entry.upstream_kind(),
        candidate_entry.base_url.as_deref(),
        &candidate_entry.auth_profiles,
    ) {
        return response;
    }
    if let Err(message) = candidate_entry.validate_shape() {
        return validation_error(message);
    }
    let runtime_oauth_states = auth_profiles.clone().map(strip_runtime_oauth_data);
    let auth_profiles_for_write = runtime_oauth_states
        .as_ref()
        .map(|(profiles, _)| profiles.clone());
    let runtime_oauth_states = runtime_oauth_states
        .map(|(_, states)| states)
        .unwrap_or_default();

    match update_config_file(&state, move |config| {
        if let Some(entry) = config.providers.iter_mut().find(|entry| entry.name == name) {
            if let Some(ref key) = body.api_key {
                entry.api_key = key.clone();
            }
            if let Some(ref upstream_opt) = body.upstream {
                entry.upstream = upstream_opt
                    .as_deref()
                    .filter(|value| !value.trim().is_empty())
                    .and_then(|value| value.parse().ok());
            }
            if let Some(ref profiles) = auth_profiles_for_write {
                entry.auth_profiles = profiles.clone();
                if !profiles.is_empty() && body.api_key.is_none() {
                    entry.api_key.clear();
                }
            }
            if let Some(ref url) = body.base_url {
                entry.base_url = url.clone();
            }
            if let Some(ref url) = body.proxy_url {
                entry.proxy_url = url.clone();
            }
            if let Some(ref prefix) = body.prefix {
                entry.prefix = prefix.clone();
            }
            if let Some(ref models) = body.models {
                entry.models = models
                    .iter()
                    .map(|id| prism_core::config::ModelMapping {
                        id: id.clone(),
                        alias: None,
                    })
                    .collect();
            }
            if let Some(ref excluded) = body.excluded_models {
                entry.excluded_models = excluded.clone();
            }
            if let Some(ref headers) = body.headers {
                entry.headers = headers.clone();
            }
            if let Some(disabled) = body.disabled {
                entry.disabled = disabled;
            }
            if entry.upstream_kind() == prism_core::provider::UpstreamKind::Codex {
                entry.wire_api = prism_core::provider::WireApi::Responses;
            } else if let Some(ref wire_api_opt) = body.wire_api {
                entry.wire_api = match wire_api_opt.as_deref() {
                    Some("responses") => prism_core::provider::WireApi::Responses,
                    _ => prism_core::provider::WireApi::Chat,
                };
            }
            if let Some(weight) = body.weight {
                entry.weight = weight;
            }
            if let Some(ref region) = body.region {
                entry.region = region.clone();
            }
            if let Some(ref presentation_opt) = body.upstream_presentation {
                entry.upstream_presentation = presentation_opt.clone().unwrap_or_default();
            }
            if let Some(vertex) = body.vertex {
                entry.vertex = vertex;
            }
            if let Some(ref project) = body.vertex_project {
                entry.vertex_project = project.clone();
            }
            if let Some(ref location) = body.vertex_location {
                entry.vertex_location = location.clone();
            }
        }
    })
    .await
    {
        Ok(()) => {
            if let Err(err) =
                seed_runtime_oauth_states(&state, &name_for_log, &runtime_oauth_states)
            {
                tracing::error!(
                    provider = %name_for_log,
                    error = %err,
                    "Provider updated but runtime oauth seeding failed"
                );
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "runtime_auth_seed_failed", "message": err})),
                );
            }
            tracing::info!(provider = %name_for_log, "Provider updated via dashboard");
            (
                StatusCode::OK,
                Json(json!({"message": "Provider updated successfully"})),
            )
        }
        Err(error) => {
            tracing::error!(
                provider = %name_for_log,
                error = ?error,
                "Failed to update provider"
            );
            config_tx_error_response(error)
        }
    }
}

/// DELETE /api/dashboard/providers/:name
pub async fn delete_provider(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    {
        let config = state.config.load();
        if !config.providers.iter().any(|entry| entry.name == name) {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "not_found", "message": "Provider not found"})),
            );
        }
    }

    let name_for_log = name.clone();
    match update_config_file(&state, move |config| {
        config.providers.retain(|entry| entry.name != name);
    })
    .await
    {
        Ok(()) => {
            tracing::info!(provider = %name_for_log, "Provider deleted via dashboard");
            (
                StatusCode::OK,
                Json(json!({"message": "Provider deleted successfully"})),
            )
        }
        Err(error) => {
            tracing::error!(
                provider = %name_for_log,
                error = ?error,
                "Failed to delete provider"
            );
            config_tx_error_response(error)
        }
    }
}

async fn update_config_file(
    state: &AppState,
    mutate: impl FnOnce(&mut prism_core::config::Config),
) -> Result<(), super::super::config_tx::ConfigTxError> {
    super::super::config_tx::update_config_versioned(state, None, mutate)
        .await
        .map(|_| ())
}
