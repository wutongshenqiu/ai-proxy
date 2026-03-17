mod device_flow;
mod managed;
mod oauth;
mod response;
mod view;

pub use device_flow::{poll_codex_device, start_codex_device};
pub use managed::{connect_auth_profile, import_local_auth_profile, refresh_auth_profile};
pub use oauth::{complete_codex_oauth, start_codex_oauth};

use super::config_tx::{ConfigTxError, update_config_versioned};
use crate::AppState;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use prism_core::auth_profile::{AuthHeaderKind, AuthMode, AuthProfileEntry};
use prism_core::presentation::UpstreamPresentationConfig;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;

use self::response::{
    AuthProfileDeletedResponse, AuthProfileMutationEnvelope, AuthProfilesListResponse,
    AuthProfilesRuntimeResponse,
};
use self::view::{current_profile_response, hydrate_profile, summarize_profile};

#[derive(Debug, Deserialize)]
pub struct CreateAuthProfileRequest {
    pub provider: String,
    pub id: String,
    pub mode: AuthMode,
    #[serde(default)]
    pub header: AuthHeaderKind,
    #[serde(default)]
    pub secret: Option<String>,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub disabled: bool,
    #[serde(default = "default_weight")]
    pub weight: u32,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default)]
    pub prefix: Option<String>,
    #[serde(default)]
    pub upstream_presentation: UpstreamPresentationConfig,
}

#[derive(Debug, Deserialize)]
pub struct ReplaceAuthProfileRequest {
    pub mode: AuthMode,
    #[serde(default)]
    pub header: AuthHeaderKind,
    #[serde(default)]
    pub secret: Option<String>,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub disabled: bool,
    #[serde(default = "default_weight")]
    pub weight: u32,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default)]
    pub prefix: Option<String>,
    #[serde(default)]
    pub upstream_presentation: UpstreamPresentationConfig,
}

struct AuthProfileDraft {
    mode: AuthMode,
    header: AuthHeaderKind,
    secret: Option<String>,
    headers: HashMap<String, String>,
    disabled: bool,
    weight: u32,
    region: Option<String>,
    prefix: Option<String>,
    upstream_presentation: UpstreamPresentationConfig,
}

fn default_weight() -> u32 {
    1
}

pub(super) fn managed_auth_proxy_url(state: &AppState) -> Option<String> {
    state.config.load().managed_auth.proxy_url.clone()
}

fn migrate_legacy_provider_auth(entry: &mut prism_core::config::ProviderKeyEntry) {
    if !entry.auth_profiles.is_empty() || entry.api_key.trim().is_empty() {
        return;
    }

    entry.auth_profiles.push(AuthProfileEntry {
        id: entry.name.clone(),
        mode: AuthMode::ApiKey,
        header: AuthHeaderKind::Auto,
        secret: Some(entry.api_key.clone()),
        weight: entry.weight.max(1),
        ..Default::default()
    });
    entry.api_key.clear();
}

pub(super) fn explicit_profile<'a>(
    config: &'a prism_core::config::Config,
    provider: &str,
    profile_id: &str,
) -> Option<(
    &'a prism_core::config::ProviderKeyEntry,
    &'a AuthProfileEntry,
)> {
    let entry = config
        .providers
        .iter()
        .find(|entry| entry.name == provider)?;
    let profile = entry
        .auth_profiles
        .iter()
        .find(|profile| profile.id == profile_id)?;
    Some((entry, profile))
}

fn auth_profile_entry_from_create(
    request: &CreateAuthProfileRequest,
) -> Result<AuthProfileEntry, (StatusCode, Json<serde_json::Value>)> {
    auth_profile_entry(
        &request.id,
        AuthProfileDraft {
            mode: request.mode,
            header: request.header,
            secret: request.secret.clone(),
            headers: request.headers.clone(),
            disabled: request.disabled,
            weight: request.weight,
            region: request.region.clone(),
            prefix: request.prefix.clone(),
            upstream_presentation: request.upstream_presentation.clone(),
        },
    )
}

fn auth_profile_entry(
    id: &str,
    draft: AuthProfileDraft,
) -> Result<AuthProfileEntry, (StatusCode, Json<serde_json::Value>)> {
    let mut profile = AuthProfileEntry {
        id: id.trim().to_string(),
        mode: draft.mode,
        header: draft.header,
        secret: draft.secret.filter(|value| !value.trim().is_empty()),
        headers: draft.headers,
        disabled: draft.disabled,
        weight: draft.weight,
        region: draft.region,
        prefix: draft.prefix,
        upstream_presentation: draft.upstream_presentation,
        ..Default::default()
    };
    profile.normalize();

    if matches!(profile.mode, AuthMode::ApiKey | AuthMode::BearerToken)
        && !profile.disabled
        && profile.secret.as_deref().is_none_or(str::is_empty)
    {
        return Err(validation_error(
            "secret is required for api-key and bearer-token auth profiles",
        ));
    }
    if profile.mode.is_managed() && profile.secret.is_some() {
        return Err(validation_error(
            "secret must not be set for managed auth profiles",
        ));
    }
    profile
        .validate()
        .map_err(|message| validation_error(&message))?;
    Ok(profile)
}

pub(super) fn default_managed_header(mode: AuthMode) -> AuthHeaderKind {
    match mode {
        AuthMode::CodexOAuth => AuthHeaderKind::Bearer,
        AuthMode::AnthropicClaudeSubscription => AuthHeaderKind::XApiKey,
        AuthMode::ApiKey | AuthMode::BearerToken => AuthHeaderKind::Auto,
    }
}

pub(super) fn rebuild_router_from_state(state: &AppState) {
    let config = state.config.load();
    let _ = state.auth_runtime.sync_with_config(&config);
    state
        .router
        .set_oauth_states(state.auth_runtime.oauth_snapshot());
    state.router.update_from_config(&config);
    state
        .catalog
        .update_from_credentials(&state.router.credential_map());
}

pub(super) fn validation_error(message: &str) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(json!({"error": "validation_failed", "message": message})),
    )
}

pub(super) fn not_found(message: &str) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::NOT_FOUND,
        Json(json!({"error": "not_found", "message": message})),
    )
}

pub(super) fn internal_error(message: impl Into<String>) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"error": "internal_error", "message": message.into()})),
    )
}

pub(super) async fn ensure_managed_profile_shape(
    state: &AppState,
    provider: &str,
    profile_id: &str,
    mode: AuthMode,
) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    let config = state.config.load();
    let Some(entry) = config.providers.iter().find(|entry| entry.name == provider) else {
        return Err(not_found("Provider not found"));
    };
    AuthProfileEntry {
        id: profile_id.to_string(),
        mode,
        header: default_managed_header(mode),
        ..Default::default()
    }
    .validate_for_provider(
        entry.format,
        entry.upstream_kind(),
        entry.base_url.as_deref(),
    )
    .map_err(|message| validation_error(&message))?;
    drop(config);

    match update_config_versioned(state, None, move |config| {
        if let Some(entry) = config
            .providers
            .iter_mut()
            .find(|entry| entry.name == provider)
        {
            migrate_legacy_provider_auth(entry);
            if let Some(profile) = entry
                .auth_profiles
                .iter_mut()
                .find(|profile| profile.id == profile_id)
            {
                profile.mode = mode;
                profile.header = default_managed_header(mode);
                profile.secret = None;
                profile.access_token = None;
                profile.refresh_token = None;
                profile.id_token = None;
                profile.expires_at = None;
                profile.account_id = None;
                profile.email = None;
                profile.last_refresh = None;
                profile.disabled = false;
                return;
            }

            entry.auth_profiles.push(AuthProfileEntry {
                id: profile_id.to_string(),
                mode,
                header: default_managed_header(mode),
                disabled: false,
                ..Default::default()
            });
        }
    })
    .await
    {
        Ok(_) => Ok(()),
        Err(ConfigTxError::Conflict { current_version }) => Err((
            StatusCode::CONFLICT,
            Json(json!({"error": "config_conflict", "current_version": current_version})),
        )),
        Err(ConfigTxError::Validation(message)) => Err(validation_error(&message)),
        Err(ConfigTxError::Internal(message)) => Err(internal_error(message)),
    }
}

/// GET /api/dashboard/auth-profiles
pub async fn list_auth_profiles(State(state): State<AppState>) -> Response {
    let config = state.config.load();
    let mut profiles = Vec::new();

    for entry in &config.providers {
        for profile in &entry.auth_profiles {
            match hydrate_profile(&state, &entry.name, profile) {
                Ok(hydrated) => {
                    profiles.push(summarize_profile(&entry.name, entry.format, &hydrated))
                }
                Err(response) => return response.into_response(),
            }
        }
    }

    (StatusCode::OK, Json(AuthProfilesListResponse { profiles })).into_response()
}

/// GET /api/dashboard/auth-profiles/runtime
pub async fn auth_profiles_runtime(State(state): State<AppState>) -> Response {
    let storage_dir = match state.auth_runtime.storage_dir() {
        Ok(value) => value.map(|path| path.display().to_string()),
        Err(message) => return internal_error(message).into_response(),
    };
    let codex_auth_file = match state.auth_runtime.codex_auth_file_path() {
        Ok(value) => value.map(|path| path.display().to_string()),
        Err(message) => return internal_error(message).into_response(),
    };
    let proxy_url = managed_auth_proxy_url(&state);
    (
        StatusCode::OK,
        Json(AuthProfilesRuntimeResponse {
            storage_dir,
            codex_auth_file,
            proxy_url,
        }),
    )
        .into_response()
}

/// POST /api/dashboard/auth-profiles
pub async fn create_auth_profile(
    State(state): State<AppState>,
    Json(body): Json<CreateAuthProfileRequest>,
) -> Response {
    if body.provider.trim().is_empty() || body.id.trim().is_empty() {
        return validation_error("provider and id are required").into_response();
    }

    let profile = match auth_profile_entry_from_create(&body) {
        Ok(profile) => profile,
        Err(response) => return response.into_response(),
    };

    let config = state.config.load();
    let Some(entry) = config
        .providers
        .iter()
        .find(|entry| entry.name == body.provider)
    else {
        return not_found("Provider not found").into_response();
    };
    if let Err(message) = profile.validate_for_provider(
        entry.format,
        entry.upstream_kind(),
        entry.base_url.as_deref(),
    ) {
        return validation_error(&message).into_response();
    }
    let duplicate_after_migration = entry.api_key.trim().is_empty()
        && entry.auth_profiles.iter().any(|item| item.id == profile.id);
    let duplicate_legacy_profile = entry.auth_profiles.is_empty()
        && !entry.api_key.trim().is_empty()
        && entry.name == profile.id;
    if duplicate_after_migration || duplicate_legacy_profile {
        return (
            StatusCode::CONFLICT,
            Json(json!({
                "error": "duplicate_auth_profile",
                "message": "auth profile id already exists for provider"
            })),
        )
            .into_response();
    }
    drop(config);

    let provider = body.provider.clone();
    let profile_id = profile.id.clone();
    match update_config_versioned(&state, None, move |config| {
        if let Some(entry) = config
            .providers
            .iter_mut()
            .find(|entry| entry.name == provider)
        {
            migrate_legacy_provider_auth(entry);
            entry.auth_profiles.push(profile);
        }
    })
    .await
    {
        Ok(_) => {
            rebuild_router_from_state(&state);
            match current_profile_response(&state, &body.provider, &profile_id) {
                Ok(profile) => (
                    StatusCode::CREATED,
                    Json(AuthProfileMutationEnvelope { profile }),
                )
                    .into_response(),
                Err(response) => response.into_response(),
            }
        }
        Err(ConfigTxError::Conflict { current_version }) => (
            StatusCode::CONFLICT,
            Json(json!({"error": "config_conflict", "current_version": current_version})),
        )
            .into_response(),
        Err(ConfigTxError::Validation(message)) => validation_error(&message).into_response(),
        Err(ConfigTxError::Internal(message)) => internal_error(message).into_response(),
    }
}

/// PUT /api/dashboard/auth-profiles/{provider}/{profile}
pub async fn replace_auth_profile(
    State(state): State<AppState>,
    Path((provider, profile_id)): Path<(String, String)>,
    Json(body): Json<ReplaceAuthProfileRequest>,
) -> Response {
    let config = state.config.load();
    let Some(entry) = config.providers.iter().find(|entry| entry.name == provider) else {
        return not_found("Auth profile not found").into_response();
    };
    let Some(existing_profile) = entry
        .auth_profiles
        .iter()
        .find(|profile| profile.id == profile_id)
        .cloned()
    else {
        return not_found("Auth profile not found").into_response();
    };

    let effective_secret = body.secret.clone().or_else(|| {
        (existing_profile.mode == body.mode
            && matches!(body.mode, AuthMode::ApiKey | AuthMode::BearerToken))
        .then(|| existing_profile.secret.clone())
        .flatten()
    });
    let replacement = match auth_profile_entry(
        &profile_id,
        AuthProfileDraft {
            mode: body.mode,
            header: body.header,
            secret: effective_secret,
            headers: body.headers.clone(),
            disabled: body.disabled,
            weight: body.weight,
            region: body.region.clone(),
            prefix: body.prefix.clone(),
            upstream_presentation: body.upstream_presentation.clone(),
        },
    ) {
        Ok(profile) => profile,
        Err(response) => return response.into_response(),
    };

    if let Err(message) = replacement.validate_for_provider(
        entry.format,
        entry.upstream_kind(),
        entry.base_url.as_deref(),
    ) {
        return validation_error(&message).into_response();
    }
    drop(config);

    let profile_was_managed = existing_profile.mode.is_managed();

    let provider_for_update = provider.clone();
    let profile_id_for_update = profile_id.clone();
    match update_config_versioned(&state, None, move |config| {
        if let Some(entry) = config
            .providers
            .iter_mut()
            .find(|entry| entry.name == provider_for_update)
            && let Some(profile) = entry
                .auth_profiles
                .iter_mut()
                .find(|profile| profile.id == profile_id_for_update)
        {
            *profile = replacement;
        }
    })
    .await
    {
        Ok(_) => {
            if !body.mode.is_managed()
                && profile_was_managed
                && let Err(err) = state
                    .auth_runtime
                    .clear_profile_state(&provider, &profile_id)
            {
                return internal_error(err).into_response();
            }
            rebuild_router_from_state(&state);
            match current_profile_response(&state, &provider, &profile_id) {
                Ok(profile) => (
                    StatusCode::OK,
                    Json(AuthProfileMutationEnvelope { profile }),
                )
                    .into_response(),
                Err(response) => response.into_response(),
            }
        }
        Err(ConfigTxError::Conflict { current_version }) => (
            StatusCode::CONFLICT,
            Json(json!({"error": "config_conflict", "current_version": current_version})),
        )
            .into_response(),
        Err(ConfigTxError::Validation(message)) => validation_error(&message).into_response(),
        Err(ConfigTxError::Internal(message)) => internal_error(message).into_response(),
    }
}

/// DELETE /api/dashboard/auth-profiles/{provider}/{profile}
pub async fn delete_auth_profile(
    State(state): State<AppState>,
    Path((provider, profile_id)): Path<(String, String)>,
) -> Response {
    let existed = explicit_profile(&state.config.load(), &provider, &profile_id).is_some();
    if !existed {
        return not_found("Auth profile not found").into_response();
    }

    let provider_for_delete = provider.clone();
    let profile_id_for_delete = profile_id.clone();
    match update_config_versioned(&state, None, move |config| {
        if let Some(entry) = config
            .providers
            .iter_mut()
            .find(|entry| entry.name == provider_for_delete)
        {
            entry
                .auth_profiles
                .retain(|profile| profile.id != profile_id_for_delete);
        }
    })
    .await
    {
        Ok(_) => {
            if let Err(err) = state
                .auth_runtime
                .clear_profile_state(&provider, &profile_id)
            {
                return internal_error(err).into_response();
            }
            rebuild_router_from_state(&state);
            (
                StatusCode::OK,
                Json(AuthProfileDeletedResponse { deleted: true }),
            )
                .into_response()
        }
        Err(ConfigTxError::Conflict { current_version }) => (
            StatusCode::CONFLICT,
            Json(json!({"error": "config_conflict", "current_version": current_version})),
        )
            .into_response(),
        Err(ConfigTxError::Validation(message)) => validation_error(&message).into_response(),
        Err(ConfigTxError::Internal(message)) => internal_error(message).into_response(),
    }
}
