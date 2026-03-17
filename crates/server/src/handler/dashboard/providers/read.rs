use crate::AppState;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use prism_core::auth_profile::AuthProfileEntry;
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
struct ProviderSummary {
    name: String,
    format: String,
    upstream: String,
    api_key_masked: String,
    base_url: Option<String>,
    models: Vec<prism_core::config::ModelMapping>,
    disabled: bool,
    wire_api: prism_core::provider::WireApi,
    upstream_presentation: prism_core::presentation::UpstreamPresentationConfig,
    auth_profiles: Vec<AuthProfileSummary>,
}

#[derive(Debug, Serialize)]
struct AuthProfileSummary {
    id: String,
    qualified_name: String,
    mode: prism_core::auth_profile::AuthMode,
    header: prism_core::auth_profile::AuthHeaderKind,
    secret_masked: Option<String>,
    access_token_masked: Option<String>,
    refresh_token_present: bool,
    id_token_present: bool,
    expires_at: Option<String>,
    account_id: Option<String>,
    email: Option<String>,
    last_refresh: Option<String>,
    headers: HashMap<String, String>,
    disabled: bool,
    weight: u32,
    region: Option<String>,
    prefix: Option<String>,
    upstream_presentation: prism_core::presentation::UpstreamPresentationConfig,
}

fn mask_key(key: &str) -> String {
    if key.len() <= 8 {
        return "****".to_string();
    }
    format!("{}****{}", &key[..4], &key[key.len() - 4..])
}

fn mask_optional_key(key: Option<&str>) -> Option<String> {
    key.filter(|value| !value.is_empty()).map(mask_key)
}

fn provider_api_key_masked(
    state: &AppState,
    entry: &prism_core::config::ProviderKeyEntry,
) -> String {
    if !entry.api_key.is_empty() {
        return mask_key(&entry.api_key);
    }

    entry
        .expanded_auth_profiles()
        .into_iter()
        .find_map(|profile| {
            let hydrated = state
                .auth_runtime
                .apply_runtime_state(&entry.name, &profile)
                .unwrap_or(profile);
            hydrated
                .secret
                .as_deref()
                .filter(|value| !value.is_empty())
                .or_else(|| {
                    hydrated
                        .access_token
                        .as_deref()
                        .filter(|value| !value.is_empty())
                })
                .map(mask_key)
        })
        .unwrap_or_default()
}

fn summarize_auth_profile(provider_name: &str, profile: &AuthProfileEntry) -> AuthProfileSummary {
    AuthProfileSummary {
        id: profile.id.clone(),
        qualified_name: format!("{provider_name}/{}", profile.id),
        mode: profile.mode,
        header: profile.header,
        secret_masked: mask_optional_key(profile.secret.as_deref()),
        access_token_masked: mask_optional_key(profile.access_token.as_deref()),
        refresh_token_present: profile
            .refresh_token
            .as_deref()
            .is_some_and(|value| !value.is_empty()),
        id_token_present: profile
            .id_token
            .as_deref()
            .is_some_and(|value| !value.is_empty()),
        expires_at: profile.expires_at.clone(),
        account_id: profile.account_id.clone(),
        email: profile.email.clone(),
        last_refresh: profile.last_refresh.clone(),
        headers: profile.headers.clone(),
        disabled: profile.disabled,
        weight: profile.weight.max(1),
        region: profile.region.clone(),
        prefix: profile.prefix.clone(),
        upstream_presentation: profile.upstream_presentation.clone(),
    }
}

fn summarize_provider(
    state: &AppState,
    entry: &prism_core::config::ProviderKeyEntry,
) -> ProviderSummary {
    let auth_profiles = entry
        .expanded_auth_profiles()
        .into_iter()
        .map(|profile| {
            let hydrated = state
                .auth_runtime
                .apply_runtime_state(&entry.name, &profile)
                .unwrap_or(profile);
            summarize_auth_profile(&entry.name, &hydrated)
        })
        .collect();

    ProviderSummary {
        name: entry.name.clone(),
        format: entry.format.as_str().to_string(),
        upstream: entry.upstream_kind().as_str().to_string(),
        api_key_masked: provider_api_key_masked(state, entry),
        base_url: entry.base_url.clone(),
        models: entry.models.clone(),
        disabled: entry.disabled,
        wire_api: entry.wire_api,
        upstream_presentation: entry.upstream_presentation.clone(),
        auth_profiles,
    }
}

/// GET /api/dashboard/providers
pub async fn list_providers(State(state): State<AppState>) -> impl IntoResponse {
    let config = state.config.load();
    let providers = config
        .providers
        .iter()
        .map(|entry| summarize_provider(&state, entry))
        .collect::<Vec<_>>();

    (StatusCode::OK, Json(json!({ "providers": providers })))
}

/// GET /api/dashboard/providers/:name
pub async fn get_provider(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let config = state.config.load();

    match config.providers.iter().find(|entry| entry.name == name) {
        Some(entry) => {
            let detail = json!({
                "name": entry.name,
                "format": entry.format.as_str(),
                "upstream": entry.upstream_kind().as_str(),
                "api_key_masked": provider_api_key_masked(&state, entry),
                "base_url": entry.base_url,
                "proxy_url": entry.proxy_url,
                "prefix": entry.prefix,
                "models": entry.models,
                "excluded_models": entry.excluded_models,
                "headers": entry.headers,
                "disabled": entry.disabled,
                "wire_api": entry.wire_api,
                "weight": entry.weight,
                "region": entry.region,
                "upstream_presentation": entry.upstream_presentation,
                "vertex": entry.vertex,
                "vertex_project": entry.vertex_project,
                "vertex_location": entry.vertex_location,
                "auth_profiles": entry
                    .expanded_auth_profiles()
                    .into_iter()
                    .map(|profile| {
                        let hydrated = state
                            .auth_runtime
                            .apply_runtime_state(&entry.name, &profile)
                            .unwrap_or(profile);
                        summarize_auth_profile(&entry.name, &hydrated)
                    })
                    .collect::<Vec<_>>(),
            });
            (StatusCode::OK, Json(detail))
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "not_found", "message": "Provider not found"})),
        ),
    }
}
