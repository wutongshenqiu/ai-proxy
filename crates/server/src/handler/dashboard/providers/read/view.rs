use crate::AppState;
use prism_core::auth_profile::AuthProfileEntry;

use super::response::{AuthProfileSummary, ProviderDetailResponse, ProviderSummary};

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

fn summarize_auth_profiles(
    state: &AppState,
    entry: &prism_core::config::ProviderKeyEntry,
) -> Vec<AuthProfileSummary> {
    entry
        .expanded_auth_profiles()
        .into_iter()
        .map(|profile| {
            let hydrated = state
                .auth_runtime
                .apply_runtime_state(&entry.name, &profile)
                .unwrap_or(profile);
            summarize_auth_profile(&entry.name, &hydrated)
        })
        .collect()
}

pub(super) fn summarize_provider(
    state: &AppState,
    entry: &prism_core::config::ProviderKeyEntry,
) -> ProviderSummary {
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
        auth_profiles: summarize_auth_profiles(state, entry),
    }
}

pub(super) fn provider_detail_response(
    state: &AppState,
    entry: &prism_core::config::ProviderKeyEntry,
) -> ProviderDetailResponse {
    ProviderDetailResponse {
        name: entry.name.clone(),
        format: entry.format.as_str().to_string(),
        upstream: entry.upstream_kind().as_str().to_string(),
        api_key_masked: provider_api_key_masked(state, entry),
        base_url: entry.base_url.clone(),
        proxy_url: entry.proxy_url.clone(),
        prefix: entry.prefix.clone(),
        models: entry.models.clone(),
        excluded_models: entry.excluded_models.clone(),
        headers: entry.headers.clone(),
        disabled: entry.disabled,
        wire_api: entry.wire_api,
        weight: entry.weight,
        region: entry.region.clone(),
        upstream_presentation: entry.upstream_presentation.clone(),
        vertex: entry.vertex,
        vertex_project: entry.vertex_project.clone(),
        vertex_location: entry.vertex_location.clone(),
        auth_profiles: summarize_auth_profiles(state, entry),
    }
}
