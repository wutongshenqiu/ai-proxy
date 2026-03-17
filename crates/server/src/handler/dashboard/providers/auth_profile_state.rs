use crate::AppState;
use axum::Json;
use axum::http::StatusCode;
use prism_core::auth_profile::{AuthProfileEntry, OAuthTokenState};

use super::helpers::validation_error;

pub(super) fn normalize_auth_profiles(
    profiles: &[AuthProfileEntry],
) -> Result<Vec<AuthProfileEntry>, (StatusCode, Json<serde_json::Value>)> {
    let mut normalized = profiles.to_vec();
    for profile in &mut normalized {
        profile.normalize();
        if let Err(err) = profile.validate() {
            return Err(validation_error(err));
        }
    }
    Ok(normalized)
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
