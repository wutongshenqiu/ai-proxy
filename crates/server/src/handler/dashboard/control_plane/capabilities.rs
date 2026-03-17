use axum::{Json, extract::State};
use prism_core::provider::upstream_protocol_for_kind;
use prism_domain::capability::default_capabilities_for_protocol;

use crate::{AppState, handler::dashboard::providers::cached_probe_result};

use super::types::{
    CapabilityProbeStates, ProviderCapabilitiesResponse, ProviderCapabilityEntry,
    ProviderModelEntry,
};

/// GET /api/dashboard/providers/capabilities
/// Returns runtime capability truth for all providers and their models.
pub async fn provider_capabilities(
    State(state): State<AppState>,
) -> Json<ProviderCapabilitiesResponse> {
    let config = state.config.load();
    let mut providers = Vec::new();

    for provider in &config.providers {
        let upstream = provider.upstream_kind();
        let protocol = upstream_protocol_for_kind(upstream);
        let caps = default_capabilities_for_protocol(protocol);
        let probe = cached_probe_result(&state, &provider.name);
        let models = provider
            .models
            .iter()
            .map(|model| ProviderModelEntry {
                id: model.id.clone(),
                alias: model.alias.clone(),
            })
            .collect();

        providers.push(ProviderCapabilityEntry {
            name: provider.name.clone(),
            format: provider.format,
            upstream,
            upstream_protocol: protocol,
            wire_api: provider.wire_api,
            presentation_profile: provider.upstream_presentation.profile.clone(),
            presentation_mode: provider.upstream_presentation.mode.clone(),
            models,
            capabilities: caps,
            probe_status: probe
                .as_ref()
                .map(|value| value.status.clone())
                .unwrap_or_else(|| "unknown".to_string()),
            checked_at: probe.as_ref().map(|value| value.checked_at.clone()),
            probe: CapabilityProbeStates {
                text: probe_state(&probe, "text"),
                stream: probe_state(&probe, "stream"),
                tools: probe_state(&probe, "tools"),
                images: probe_state(&probe, "images"),
                json_schema: probe_state(&probe, "json_schema"),
                reasoning: probe_state(&probe, "reasoning"),
                count_tokens: probe_state(&probe, "count_tokens"),
            },
            disabled: provider.disabled,
        });
    }

    Json(ProviderCapabilitiesResponse { providers })
}

fn probe_state(
    probe: &Option<crate::handler::dashboard::providers::ProviderProbeResult>,
    capability: &str,
) -> super::types::CapabilityProbeState {
    let check = probe.as_ref().and_then(|result| {
        result
            .checks
            .iter()
            .find(|check| check.capability == capability)
    });
    super::types::CapabilityProbeState {
        status: check
            .map(|value| value.status)
            .unwrap_or(crate::handler::dashboard::providers::ProbeStatus::Unknown),
        message: check.and_then(|value| value.message.clone()),
    }
}
