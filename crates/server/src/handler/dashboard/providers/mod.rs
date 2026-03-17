mod mutation;
mod probe;

use crate::AppState;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use prism_core::auth_profile::{AuthProfileEntry, OAuthTokenState};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

pub use mutation::{create_provider, delete_provider, update_provider};
pub use probe::{cached_probe_result, fetch_models, health_check, presentation_preview};

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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProbeStatus {
    Verified,
    Failed,
    Unknown,
    Unsupported,
}

impl ProbeStatus {
    fn is_verified(self) -> bool {
        matches!(self, Self::Verified)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderProbeCheck {
    pub capability: String,
    pub status: ProbeStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderProbeResult {
    pub provider: String,
    pub upstream: String,
    pub status: String,
    pub checked_at: String,
    pub latency_ms: u64,
    pub checks: Vec<ProviderProbeCheck>,
}

impl ProviderProbeResult {
    pub fn capability_status(&self, capability: &str) -> ProbeStatus {
        self.checks
            .iter()
            .find(|check| check.capability == capability)
            .map(|check| check.status)
            .unwrap_or(ProbeStatus::Unknown)
    }
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

pub(super) fn normalize_auth_profiles(
    profiles: &[AuthProfileEntry],
) -> Result<Vec<AuthProfileEntry>, (StatusCode, Json<serde_json::Value>)> {
    let mut normalized = profiles.to_vec();
    for profile in &mut normalized {
        profile.normalize();
        if let Err(err) = profile.validate() {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({"error": "validation_failed", "message": err})),
            ));
        }
    }
    Ok(normalized)
}

pub(super) fn validation_error(
    message: impl Into<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(json!({"error": "validation_failed", "message": message.into()})),
    )
}

pub(super) fn strip_runtime_oauth_data(
    profiles: Vec<AuthProfileEntry>,
) -> (Vec<AuthProfileEntry>, Vec<(String, OAuthTokenState)>) {
    let mut stripped = Vec::with_capacity(profiles.len());
    let mut runtime_states = Vec::new();

    for mut profile in profiles {
        if profile.mode.is_managed()
            && let Some(state) = OAuthTokenState::from_profile(&profile)
        {
            let has_runtime_material = !state.access_token.is_empty()
                || !state.refresh_token.is_empty()
                || state.id_token.is_some()
                || state.account_id.is_some()
                || state.email.is_some()
                || state.expires_at.is_some()
                || state.last_refresh.is_some();
            if has_runtime_material {
                runtime_states.push((profile.id.clone(), state));
            }
            profile.access_token = None;
            profile.refresh_token = None;
            profile.id_token = None;
            profile.expires_at = None;
            profile.account_id = None;
            profile.email = None;
            profile.last_refresh = None;
        }
        stripped.push(profile);
    }

    (stripped, runtime_states)
}

pub(super) fn seed_runtime_oauth_states(
    state: &AppState,
    provider_name: &str,
    runtime_states: &[(String, OAuthTokenState)],
) -> Result<(), String> {
    if runtime_states.is_empty() {
        return Ok(());
    }

    for (profile_id, oauth_state) in runtime_states {
        state
            .auth_runtime
            .store_state(provider_name, profile_id, oauth_state.clone())?;
    }

    let config = state.config.load();
    state
        .router
        .set_oauth_states(state.auth_runtime.oauth_snapshot());
    state.router.update_from_config(&config);
    state
        .catalog
        .update_from_credentials(&state.router.credential_map());
    Ok(())
}

pub(super) fn validate_auth_shape(
    api_key: Option<&str>,
    auth_profiles: &[AuthProfileEntry],
) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    let has_api_key = api_key.is_some_and(|value| !value.trim().is_empty());
    let has_profiles = !auth_profiles.is_empty();
    if has_api_key && has_profiles {
        return Err(validation_error(
            "api_key and auth_profiles are mutually exclusive",
        ));
    }
    Ok(())
}

pub(super) fn validate_provider_auth_profiles(
    format: prism_core::provider::Format,
    upstream: prism_core::provider::UpstreamKind,
    base_url: Option<&str>,
    auth_profiles: &[AuthProfileEntry],
) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    for profile in auth_profiles {
        if let Err(message) = profile.validate_for_provider(format, upstream, base_url) {
            return Err(validation_error(message));
        }
    }
    Ok(())
}

pub(super) fn config_tx_error_response(
    error: super::config_tx::ConfigTxError,
) -> (StatusCode, Json<serde_json::Value>) {
    match error {
        super::config_tx::ConfigTxError::Conflict { current_version } => (
            StatusCode::CONFLICT,
            Json(json!({
                "error": "conflict",
                "message": "config version conflict",
                "current_version": current_version
            })),
        ),
        super::config_tx::ConfigTxError::Validation(message) => validation_error(message),
        super::config_tx::ConfigTxError::Internal(message) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "write_failed", "message": message})),
        ),
    }
}

pub(super) fn is_valid_format(format_str: &str) -> bool {
    matches!(format_str, "openai" | "claude" | "gemini")
}

pub(super) fn parse_upstream_kind(
    format: prism_core::provider::Format,
    upstream: Option<&str>,
) -> Result<prism_core::provider::UpstreamKind, (StatusCode, Json<serde_json::Value>)> {
    let Some(raw) = upstream.filter(|value| !value.trim().is_empty()) else {
        return Ok(format.into());
    };
    raw.parse().map_err(validation_error)
}

/// GET /api/dashboard/providers
pub async fn list_providers(State(state): State<AppState>) -> impl IntoResponse {
    let config = state.config.load();
    let mut providers = Vec::new();

    for entry in config.providers.iter() {
        providers.push(summarize_provider(&state, entry));
    }

    (StatusCode::OK, Json(json!({ "providers": providers })))
}

/// GET /api/dashboard/providers/:name
pub async fn get_provider(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let config = state.config.load();

    match config.providers.iter().find(|e| e.name == name) {
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
