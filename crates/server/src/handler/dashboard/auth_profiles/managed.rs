use super::{
    current_profile_response, ensure_managed_profile_shape, explicit_profile, internal_error,
    managed_auth_proxy_url, not_found, rebuild_router_from_state, validation_error,
};
use crate::AppState;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use prism_core::auth_profile::{AuthMode, OAuthTokenState, validate_anthropic_subscription_token};
use serde::Deserialize;
use serde_json::json;
use std::path::Path as FsPath;

#[derive(Debug, Deserialize, Default)]
pub struct ImportLocalAuthProfileRequest {
    #[serde(default)]
    pub path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ConnectAuthProfileRequest {
    pub secret: String,
}

/// POST /api/dashboard/auth-profiles/{provider}/{profile}/import-local
pub async fn import_local_auth_profile(
    State(state): State<AppState>,
    Path((provider, profile_id)): Path<(String, String)>,
    Json(body): Json<ImportLocalAuthProfileRequest>,
) -> impl IntoResponse {
    if let Err(response) =
        ensure_managed_profile_shape(&state, &provider, &profile_id, AuthMode::CodexOAuth).await
    {
        return response;
    }

    let path_override = body
        .path
        .as_deref()
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .map(FsPath::new);
    let tokens = match state.auth_runtime.load_codex_cli_tokens(path_override) {
        Ok(tokens) => tokens,
        Err(message) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({"error": "local_import_failed", "message": message})),
            );
        }
    };
    if let Err(err) = state
        .auth_runtime
        .store_codex_tokens(&provider, &profile_id, &tokens)
    {
        return internal_error(err);
    }
    rebuild_router_from_state(&state);

    match current_profile_response(&state, &provider, &profile_id) {
        Ok(profile) => (StatusCode::OK, Json(json!({ "profile": profile }))),
        Err(response) => response,
    }
}

/// POST /api/dashboard/auth-profiles/{provider}/{profile}/connect
pub async fn connect_auth_profile(
    State(state): State<AppState>,
    Path((provider, profile_id)): Path<(String, String)>,
    Json(body): Json<ConnectAuthProfileRequest>,
) -> impl IntoResponse {
    if body.secret.trim().is_empty() {
        return validation_error("secret is required");
    }

    let config = state.config.load();
    let Some((entry, profile)) = explicit_profile(&config, &provider, &profile_id) else {
        return not_found("Auth profile not found");
    };
    if profile.mode != AuthMode::AnthropicClaudeSubscription {
        return validation_error(
            "connect is only supported for anthropic-claude-subscription profiles",
        );
    }
    if let Err(message) = profile.validate_for_provider(
        entry.format,
        entry.upstream_kind(),
        entry.base_url.as_deref(),
    ) {
        return validation_error(&message);
    }
    if let Err(message) = validate_anthropic_subscription_token(body.secret.trim()) {
        return validation_error(&message);
    }
    drop(config);

    if let Err(response) = ensure_managed_profile_shape(
        &state,
        &provider,
        &profile_id,
        AuthMode::AnthropicClaudeSubscription,
    )
    .await
    {
        return response;
    }

    if let Err(err) = state.auth_runtime.store_anthropic_subscription_token(
        &provider,
        &profile_id,
        body.secret.trim(),
    ) {
        return internal_error(err);
    }
    rebuild_router_from_state(&state);

    match current_profile_response(&state, &provider, &profile_id) {
        Ok(profile) => (StatusCode::OK, Json(json!({ "profile": profile }))),
        Err(response) => response,
    }
}

/// POST /api/dashboard/auth-profiles/{provider}/{profile}/refresh
pub async fn refresh_auth_profile(
    State(state): State<AppState>,
    Path((provider, profile_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let config = state.config.load();
    let Some((_, profile)) = explicit_profile(&config, &provider, &profile_id) else {
        return not_found("Auth profile not found");
    };
    if !profile.mode.supports_refresh() {
        return validation_error("refresh is only supported for refreshable managed auth profiles");
    }

    let shared_state = match state
        .auth_runtime
        .shared_state_for_profile(&provider, &profile_id)
    {
        Some(shared) => shared,
        None => match OAuthTokenState::from_profile(profile) {
            Some(runtime_state) => {
                match state
                    .auth_runtime
                    .ensure_shared_state(&provider, &profile_id, runtime_state)
                {
                    Ok(shared) => shared,
                    Err(message) => return internal_error(message),
                }
            }
            None => {
                return validation_error(
                    "auth profile is disconnected; reconnect it before refresh",
                );
            }
        },
    };
    drop(config);

    let auth_proxy = managed_auth_proxy_url(&state);
    match state
        .auth_runtime
        .refresh_codex_profile(
            &state.http_client_pool,
            auth_proxy.as_deref(),
            &provider,
            &profile_id,
            shared_state,
            true,
        )
        .await
    {
        Ok(()) => {}
        Err(err) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({"error": "oauth_refresh_failed", "message": err.to_string()})),
            );
        }
    }
    rebuild_router_from_state(&state);

    match current_profile_response(&state, &provider, &profile_id) {
        Ok(profile) => (StatusCode::OK, Json(json!({ "profile": profile }))),
        Err(response) => response,
    }
}
