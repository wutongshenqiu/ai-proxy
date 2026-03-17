use crate::AppState;
use prism_core::auth_profile::{AuthMode, AuthProfileEntry};

use super::response::AuthProfileListItem;
use super::{internal_error, not_found};

fn mask_key(key: &str) -> String {
    if key.len() <= 8 {
        return "****".to_string();
    }
    format!("{}****{}", &key[..4], &key[key.len() - 4..])
}

fn mask_optional(value: Option<&str>) -> Option<String> {
    value.filter(|value| !value.is_empty()).map(mask_key)
}

fn profile_connected(profile: &AuthProfileEntry) -> bool {
    match profile.mode {
        AuthMode::ApiKey | AuthMode::BearerToken => profile
            .secret
            .as_deref()
            .is_some_and(|value| !value.is_empty()),
        AuthMode::CodexOAuth | AuthMode::AnthropicClaudeSubscription => {
            profile
                .refresh_token
                .as_deref()
                .is_some_and(|value| !value.is_empty())
                || profile
                    .access_token
                    .as_deref()
                    .is_some_and(|value| !value.is_empty())
        }
    }
}

pub(super) fn summarize_profile(
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
        connected: profile_connected(profile),
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

pub(super) fn hydrate_profile(
    state: &AppState,
    provider_name: &str,
    profile: &AuthProfileEntry,
) -> Result<AuthProfileEntry, (axum::http::StatusCode, axum::Json<serde_json::Value>)> {
    state
        .auth_runtime
        .apply_runtime_state(provider_name, profile)
        .map_err(internal_error)
}

pub(super) fn current_profile_response(
    state: &AppState,
    provider: &str,
    profile_id: &str,
) -> Result<AuthProfileListItem, (axum::http::StatusCode, axum::Json<serde_json::Value>)> {
    let config = state.config.load();
    let Some((entry, profile)) = super::explicit_profile(&config, provider, profile_id) else {
        return Err(not_found("Auth profile not found"));
    };
    let hydrated = hydrate_profile(state, &entry.name, profile)?;
    Ok(summarize_profile(&entry.name, entry.format, &hydrated))
}
