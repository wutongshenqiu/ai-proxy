use super::{
    current_profile_response, ensure_managed_profile_shape, explicit_profile, internal_error,
    managed_auth_proxy_url, not_found, rebuild_router_from_state, validation_error,
};
use crate::AppState;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use chrono::{Duration, Utc};
use prism_core::auth_profile::{AuthMode, OAuthTokenState, validate_anthropic_subscription_token};
use serde::Deserialize;
use serde_json::json;
use std::path::Path as FsPath;

const OAUTH_SESSION_TTL_MINUTES: i64 = 10;
const DEVICE_SESSION_TTL_MINUTES: i64 = 15;

#[derive(Debug, Deserialize)]
pub struct StartCodexOauthRequest {
    pub provider: String,
    pub profile_id: String,
    pub redirect_uri: String,
}

#[derive(Debug, Deserialize)]
pub struct CompleteCodexOauthRequest {
    pub state: String,
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct StartCodexDeviceRequest {
    pub provider: String,
    pub profile_id: String,
}

#[derive(Debug, Deserialize)]
pub struct PollCodexDeviceRequest {
    pub state: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct ImportLocalAuthProfileRequest {
    #[serde(default)]
    pub path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ConnectAuthProfileRequest {
    pub secret: String,
}

/// POST /api/dashboard/auth-profiles/codex/oauth/start
pub async fn start_codex_oauth(
    State(state): State<AppState>,
    Json(body): Json<StartCodexOauthRequest>,
) -> impl IntoResponse {
    if body.provider.trim().is_empty()
        || body.profile_id.trim().is_empty()
        || body.redirect_uri.trim().is_empty()
    {
        return validation_error("provider, profile_id, and redirect_uri are required");
    }

    if let Err(response) = ensure_managed_profile_shape(
        &state,
        &body.provider,
        &body.profile_id,
        AuthMode::CodexOAuth,
    )
    .await
    {
        return response;
    }

    let state_key = uuid::Uuid::new_v4().to_string();
    let (code_verifier, challenge) = match crate::auth_runtime::AuthRuntimeManager::generate_pkce()
    {
        Ok(value) => value,
        Err(err) => return internal_error(err.to_string()),
    };

    state.oauth_sessions.insert(
        state_key.clone(),
        crate::auth_runtime::PendingCodexOauthSession {
            provider: body.provider.clone(),
            profile_id: body.profile_id.clone(),
            code_verifier,
            redirect_uri: body.redirect_uri.clone(),
            created_at: Utc::now(),
        },
    );

    let auth_url =
        state
            .auth_runtime
            .build_codex_auth_url(&state_key, &challenge, &body.redirect_uri);

    (
        StatusCode::OK,
        Json(json!({
            "state": state_key,
            "auth_url": auth_url,
            "provider": body.provider,
            "profile_id": body.profile_id,
            "expires_in": Duration::minutes(OAUTH_SESSION_TTL_MINUTES).num_seconds(),
        })),
    )
}

/// POST /api/dashboard/auth-profiles/codex/oauth/complete
pub async fn complete_codex_oauth(
    State(state): State<AppState>,
    Json(body): Json<CompleteCodexOauthRequest>,
) -> impl IntoResponse {
    if body.state.trim().is_empty() || body.code.trim().is_empty() {
        return validation_error("state and code are required");
    }

    let Some(session) = state
        .oauth_sessions
        .get(&body.state)
        .map(|entry| entry.clone())
    else {
        return not_found("OAuth session not found");
    };
    if session.created_at + Duration::minutes(OAUTH_SESSION_TTL_MINUTES) < Utc::now() {
        state.oauth_sessions.remove(&body.state);
        return (
            StatusCode::GONE,
            Json(json!({"error": "expired", "message": "OAuth session expired"})),
        );
    }

    let auth_proxy = managed_auth_proxy_url(&state);
    let tokens = match state
        .auth_runtime
        .exchange_codex_code(
            &state.http_client_pool,
            auth_proxy.as_deref(),
            &body.code,
            &session.redirect_uri,
            &session.code_verifier,
        )
        .await
    {
        Ok(tokens) => tokens,
        Err(message) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({"error": "oauth_exchange_failed", "message": message})),
            );
        }
    };

    if let Err(response) = ensure_managed_profile_shape(
        &state,
        &session.provider,
        &session.profile_id,
        AuthMode::CodexOAuth,
    )
    .await
    {
        return response;
    }
    if let Err(err) =
        state
            .auth_runtime
            .store_codex_tokens(&session.provider, &session.profile_id, &tokens)
    {
        return internal_error(err);
    }
    state.oauth_sessions.remove(&body.state);
    rebuild_router_from_state(&state);

    match current_profile_response(&state, &session.provider, &session.profile_id) {
        Ok(profile) => (StatusCode::OK, Json(json!({ "profile": profile }))),
        Err(response) => response,
    }
}

/// POST /api/dashboard/auth-profiles/codex/device/start
pub async fn start_codex_device(
    State(state): State<AppState>,
    Json(body): Json<StartCodexDeviceRequest>,
) -> impl IntoResponse {
    if body.provider.trim().is_empty() || body.profile_id.trim().is_empty() {
        return validation_error("provider and profile_id are required");
    }

    if let Err(response) = ensure_managed_profile_shape(
        &state,
        &body.provider,
        &body.profile_id,
        AuthMode::CodexOAuth,
    )
    .await
    {
        return response;
    }

    let auth_proxy = managed_auth_proxy_url(&state);
    let start = match state
        .auth_runtime
        .start_codex_device_flow(&state.http_client_pool, auth_proxy.as_deref())
        .await
    {
        Ok(start) => start,
        Err(message) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({"error": "device_start_failed", "message": message})),
            );
        }
    };

    let state_key = uuid::Uuid::new_v4().to_string();
    state.device_sessions.insert(
        state_key.clone(),
        crate::auth_runtime::PendingCodexDeviceSession {
            provider: body.provider.clone(),
            profile_id: body.profile_id.clone(),
            device_auth_id: start.device_auth_id.clone(),
            user_code: start.user_code.clone(),
            interval_secs: start.interval_secs,
            created_at: Utc::now(),
        },
    );

    (
        StatusCode::OK,
        Json(json!({
            "state": state_key,
            "provider": body.provider,
            "profile_id": body.profile_id,
            "verification_url": start.verification_url,
            "user_code": start.user_code,
            "interval_secs": start.interval_secs,
            "expires_in": start.expires_in_secs,
        })),
    )
}

/// POST /api/dashboard/auth-profiles/codex/device/poll
pub async fn poll_codex_device(
    State(state): State<AppState>,
    Json(body): Json<PollCodexDeviceRequest>,
) -> impl IntoResponse {
    if body.state.trim().is_empty() {
        return validation_error("state is required");
    }

    let Some(session) = state
        .device_sessions
        .get(&body.state)
        .map(|entry| entry.clone())
    else {
        return not_found("Device session not found");
    };
    if session.created_at + Duration::minutes(DEVICE_SESSION_TTL_MINUTES) < Utc::now() {
        state.device_sessions.remove(&body.state);
        return (
            StatusCode::GONE,
            Json(json!({"error": "expired", "message": "Device session expired"})),
        );
    }

    let auth_proxy = managed_auth_proxy_url(&state);
    let result = match state
        .auth_runtime
        .poll_codex_device_flow(&state.http_client_pool, auth_proxy.as_deref(), &session)
        .await
    {
        Ok(result) => result,
        Err(message) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({"error": "device_poll_failed", "message": message})),
            );
        }
    };

    match result {
        crate::auth_runtime::CodexDevicePollResult::Pending => (
            StatusCode::OK,
            Json(json!({
                "status": "pending",
                "interval_secs": session.interval_secs,
            })),
        ),
        crate::auth_runtime::CodexDevicePollResult::Complete(tokens) => {
            if let Err(response) = ensure_managed_profile_shape(
                &state,
                &session.provider,
                &session.profile_id,
                AuthMode::CodexOAuth,
            )
            .await
            {
                return response;
            }
            if let Err(err) = state.auth_runtime.store_codex_tokens(
                &session.provider,
                &session.profile_id,
                &tokens,
            ) {
                return internal_error(err);
            }
            state.device_sessions.remove(&body.state);
            rebuild_router_from_state(&state);

            match current_profile_response(&state, &session.provider, &session.profile_id) {
                Ok(profile) => (
                    StatusCode::OK,
                    Json(json!({ "status": "completed", "profile": profile })),
                ),
                Err(response) => response,
            }
        }
    }
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
