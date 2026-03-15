use super::providers::{ConfigTxError, update_config_versioned};
use crate::AppState;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use chrono::{Duration, Utc};
use prism_core::auth_profile::{AuthHeaderKind, AuthMode, AuthProfileEntry, OAuthTokenState};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::{Arc, RwLock};

const OAUTH_SESSION_TTL_MINUTES: i64 = 10;

#[derive(Debug, Serialize)]
struct AuthProfileListItem {
    provider: String,
    format: String,
    id: String,
    qualified_name: String,
    mode: AuthMode,
    header: AuthHeaderKind,
    secret_masked: Option<String>,
    access_token_masked: Option<String>,
    refresh_token_present: bool,
    id_token_present: bool,
    expires_at: Option<String>,
    account_id: Option<String>,
    email: Option<String>,
    last_refresh: Option<String>,
    headers: std::collections::HashMap<String, String>,
    disabled: bool,
    weight: u32,
    region: Option<String>,
    prefix: Option<String>,
    upstream_presentation: prism_core::presentation::UpstreamPresentationConfig,
}

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

fn mask_key(key: &str) -> String {
    if key.len() <= 8 {
        return "****".to_string();
    }
    format!("{}****{}", &key[..4], &key[key.len() - 4..])
}

fn mask_optional(value: Option<&str>) -> Option<String> {
    value.filter(|value| !value.is_empty()).map(mask_key)
}

fn summarize_profile(
    provider_name: &str,
    format: prism_core::provider::Format,
    profile: &AuthProfileEntry,
) -> AuthProfileListItem {
    AuthProfileListItem {
        provider: provider_name.to_string(),
        format: format.as_str().to_string(),
        id: profile.id.clone(),
        qualified_name: format!("{provider_name}/{}", profile.id),
        mode: profile.mode,
        header: profile.header,
        secret_masked: mask_optional(profile.secret.as_deref()),
        access_token_masked: mask_optional(profile.access_token.as_deref()),
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

fn find_profile<'a>(
    config: &'a prism_core::config::Config,
    provider: &str,
    profile_id: &str,
) -> Option<(&'a prism_core::config::ProviderKeyEntry, AuthProfileEntry)> {
    let entry = config
        .providers
        .iter()
        .find(|entry| entry.name == provider)?;
    let profile = entry
        .expanded_auth_profiles()
        .into_iter()
        .find(|profile| profile.id == profile_id)?;
    Some((entry, profile))
}

fn current_profile_response(
    state: &AppState,
    provider: &str,
    profile_id: &str,
) -> Result<AuthProfileListItem, (StatusCode, Json<serde_json::Value>)> {
    let config = state.config.load();
    let Some((entry, profile)) = find_profile(&config, provider, profile_id) else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "not_found", "message": "Auth profile not found"})),
        ));
    };
    Ok(summarize_profile(&entry.name, entry.format, &profile))
}

async fn persist_codex_profile(
    state: &AppState,
    provider: &str,
    profile_id: &str,
    tokens: crate::auth_runtime::CodexOAuthTokens,
) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    let provider = provider.to_string();
    let profile_id = profile_id.to_string();
    let access_token = tokens.access_token;
    let refresh_token = tokens.refresh_token;
    let id_token = tokens.id_token;
    let expires_at = tokens.expires_at.map(|dt| dt.to_rfc3339());
    let account_id = tokens.account_id;
    let email = tokens.email;
    let last_refresh = Some(tokens.last_refresh.to_rfc3339());

    match update_config_versioned(state, None, move |config| {
        if let Some(entry) = config
            .providers
            .iter_mut()
            .find(|entry| entry.name == provider)
        {
            if let Some(profile) = entry
                .auth_profiles
                .iter_mut()
                .find(|profile| profile.id == profile_id)
            {
                profile.mode = AuthMode::OpenaiCodexOauth;
                profile.header = AuthHeaderKind::Bearer;
                profile.secret = None;
                profile.access_token = Some(access_token.clone());
                profile.refresh_token = Some(refresh_token.clone());
                profile.id_token = id_token.clone();
                profile.expires_at = expires_at.clone();
                profile.account_id = account_id.clone();
                profile.email = email.clone();
                profile.last_refresh = last_refresh.clone();
                profile.disabled = false;
                return;
            }

            entry.api_key.clear();
            entry.auth_profiles.push(AuthProfileEntry {
                id: profile_id.clone(),
                mode: AuthMode::OpenaiCodexOauth,
                header: AuthHeaderKind::Bearer,
                secret: None,
                access_token: Some(access_token.clone()),
                refresh_token: Some(refresh_token.clone()),
                id_token: id_token.clone(),
                expires_at: expires_at.clone(),
                account_id: account_id.clone(),
                email: email.clone(),
                last_refresh: last_refresh.clone(),
                ..Default::default()
            });
        }
    })
    .await
    {
        Ok(_) => Ok(()),
        Err(ConfigTxError::Conflict { current_version }) => Err((
            StatusCode::CONFLICT,
            Json(json!({
                "error": "config_conflict",
                "current_version": current_version,
            })),
        )),
        Err(ConfigTxError::Internal(message)) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "write_failed", "message": message})),
        )),
    }
}

/// GET /api/dashboard/auth-profiles
pub async fn list_auth_profiles(State(state): State<AppState>) -> impl IntoResponse {
    let config = state.config.load();
    let profiles = config
        .providers
        .iter()
        .flat_map(|entry| {
            entry
                .expanded_auth_profiles()
                .into_iter()
                .map(move |profile| summarize_profile(&entry.name, entry.format, &profile))
        })
        .collect::<Vec<_>>();

    (StatusCode::OK, Json(json!({ "profiles": profiles })))
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
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "error": "validation_failed",
                "message": "provider, profile_id, and redirect_uri are required"
            })),
        );
    }

    let config = state.config.load();
    let Some(entry) = config
        .providers
        .iter()
        .find(|entry| entry.name == body.provider)
    else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "not_found", "message": "Provider not found"})),
        );
    };
    if entry.format != prism_core::provider::Format::OpenAI {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "error": "validation_failed",
                "message": "Codex OAuth is only supported for OpenAI-format providers"
            })),
        );
    }
    if let Some(existing) = entry
        .expanded_auth_profiles()
        .into_iter()
        .find(|profile| profile.id == body.profile_id)
        && existing.mode != AuthMode::OpenaiCodexOauth
    {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "error": "validation_failed",
                "message": "existing auth profile must use openai-codex-oauth mode"
            })),
        );
    }

    let state_key = uuid::Uuid::new_v4().to_string();
    let (code_verifier, challenge) = match crate::auth_runtime::AuthRuntimeManager::generate_pkce()
    {
        Ok(value) => value,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "oauth_error", "message": err.to_string()})),
            );
        }
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
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "error": "validation_failed",
                "message": "state and code are required"
            })),
        );
    }

    let Some(session) = state
        .oauth_sessions
        .get(&body.state)
        .map(|entry| entry.clone())
    else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "not_found", "message": "OAuth session not found"})),
        );
    };
    if session.created_at + Duration::minutes(OAUTH_SESSION_TTL_MINUTES) < Utc::now() {
        state.oauth_sessions.remove(&body.state);
        return (
            StatusCode::GONE,
            Json(json!({"error": "expired", "message": "OAuth session expired"})),
        );
    }

    let global_proxy = state.config.load().proxy_url.clone();
    let tokens = match state
        .auth_runtime
        .exchange_codex_code(
            &state.http_client_pool,
            global_proxy.as_deref(),
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

    if let Err(response) =
        persist_codex_profile(&state, &session.provider, &session.profile_id, tokens).await
    {
        return response;
    }
    state.oauth_sessions.remove(&body.state);

    match current_profile_response(&state, &session.provider, &session.profile_id) {
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
    let Some((_, profile)) = find_profile(&config, &provider, &profile_id) else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "not_found", "message": "Auth profile not found"})),
        );
    };
    if profile.mode != AuthMode::OpenaiCodexOauth {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "error": "validation_failed",
                "message": "refresh is only supported for openai-codex-oauth profiles"
            })),
        );
    }

    let Some(oauth_state) = OAuthTokenState::from_profile(&profile) else {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({"error": "validation_failed", "message": "invalid oauth profile state"})),
        );
    };

    let global_proxy = config.proxy_url.clone();
    drop(config);

    let tokens = match state
        .auth_runtime
        .refresh_codex_tokens(
            &state.http_client_pool,
            global_proxy.as_deref(),
            Arc::new(RwLock::new(oauth_state)),
        )
        .await
    {
        Ok(tokens) => tokens,
        Err(err) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({"error": "oauth_refresh_failed", "message": err.to_string()})),
            );
        }
    };

    if let Err(response) = persist_codex_profile(&state, &provider, &profile_id, tokens).await {
        return response;
    }

    match current_profile_response(&state, &provider, &profile_id) {
        Ok(profile) => (StatusCode::OK, Json(json!({ "profile": profile }))),
        Err(response) => response,
    }
}
