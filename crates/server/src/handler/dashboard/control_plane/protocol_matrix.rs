use axum::{Json, extract::State};
use prism_core::config::ProviderKeyEntry;
use prism_core::provider::{Format, upstream_protocol_for_kind};
use prism_domain::operation::Operation;

use crate::{
    AppState,
    handler::dashboard::providers::{ProbeStatus, ProviderProbeResult, cached_probe_result},
};

use super::{
    specs::{ENDPOINT_SPECS, EndpointSpec, SURFACE_SPECS, SurfaceProbeKind, SurfaceSpec},
    types::{
        CapabilityProbeState, ProtocolCoverageEntry, ProtocolEndpointEntry, ProtocolMatrixResponse,
    },
};

/// GET /api/dashboard/protocols/matrix
/// Returns endpoint inventory and runtime provider coverage for the dashboard.
pub async fn protocol_matrix(State(state): State<AppState>) -> Json<ProtocolMatrixResponse> {
    let config = state.config.load();
    let coverage = build_protocol_coverage(&config.providers, &state);
    let active_provider_count = config
        .providers
        .iter()
        .filter(|provider| !provider.disabled)
        .count();
    let endpoints = ENDPOINT_SPECS
        .iter()
        .map(|spec| ProtocolEndpointEntry {
            id: spec.id.to_string(),
            family: spec.family,
            method: spec.method.to_string(),
            path: spec.path.to_string(),
            description: spec.description.to_string(),
            scope: spec.scope,
            transport: spec.transport,
            operation: spec.operation,
            stream_transport: spec.stream_transport,
            state: endpoint_state(spec, &coverage, active_provider_count),
            note: spec.note.map(str::to_string),
        })
        .collect();

    Json(ProtocolMatrixResponse {
        endpoints,
        coverage,
    })
}

fn build_protocol_coverage(
    providers: &[ProviderKeyEntry],
    state: &AppState,
) -> Vec<ProtocolCoverageEntry> {
    let mut coverage = Vec::new();

    for provider in providers {
        let upstream = provider.upstream_kind();
        let upstream_protocol = upstream_protocol_for_kind(upstream);
        let probe = cached_probe_result(state, &provider.name);

        for spec in SURFACE_SPECS {
            let state = surface_state(provider, probe.as_ref(), spec);
            let execution_mode = if state.status == ProbeStatus::Unsupported {
                None
            } else {
                Some(upstream_protocol.execution_mode_for(spec.ingress_protocol))
            };
            coverage.push(ProtocolCoverageEntry {
                provider: provider.name.clone(),
                format: provider.format,
                upstream,
                upstream_protocol,
                wire_api: provider.wire_api,
                disabled: provider.disabled,
                surface_id: spec.id.to_string(),
                surface_label: spec.label.to_string(),
                ingress_protocol: spec.ingress_protocol,
                execution_mode,
                state,
            });
        }
    }

    coverage
}

fn surface_state(
    provider: &ProviderKeyEntry,
    probe: Option<&ProviderProbeResult>,
    spec: &SurfaceSpec,
) -> CapabilityProbeState {
    if provider.disabled {
        return CapabilityProbeState {
            status: ProbeStatus::Unsupported,
            message: Some("provider is disabled".to_string()),
        };
    }

    if !spec.allowed_formats.contains(&provider.format) {
        let allowed = spec
            .allowed_formats
            .iter()
            .map(Format::as_str)
            .collect::<Vec<_>>()
            .join(", ");
        return CapabilityProbeState {
            status: ProbeStatus::Unsupported,
            message: Some(format!("surface requires provider format: {allowed}")),
        };
    }

    match spec.probe_kind {
        SurfaceProbeKind::Text => probe_state_ref(probe, "text"),
        SurfaceProbeKind::Stream => probe_state_ref(probe, "stream"),
        SurfaceProbeKind::CountTokens => probe_state_ref(probe, "count_tokens"),
    }
}

fn endpoint_state(
    endpoint: &EndpointSpec,
    coverage: &[ProtocolCoverageEntry],
    active_provider_count: usize,
) -> CapabilityProbeState {
    if endpoint.operation == Operation::ListModels {
        return if active_provider_count > 0 {
            CapabilityProbeState {
                status: ProbeStatus::Verified,
                message: Some("served from configured provider inventory".to_string()),
            }
        } else {
            CapabilityProbeState {
                status: ProbeStatus::Unsupported,
                message: Some("no active providers configured".to_string()),
            }
        };
    }

    let Some(surface_id) = endpoint.surface_id else {
        return CapabilityProbeState {
            status: ProbeStatus::Unknown,
            message: Some("no route state available".to_string()),
        };
    };

    let surface_entries = coverage
        .iter()
        .filter(|entry| !entry.disabled && entry.surface_id == surface_id)
        .collect::<Vec<_>>();
    if surface_entries.is_empty() {
        return CapabilityProbeState {
            status: ProbeStatus::Unsupported,
            message: Some("no active providers expose this surface".to_string()),
        };
    }

    if surface_entries
        .iter()
        .any(|entry| entry.state.status == ProbeStatus::Verified)
    {
        return CapabilityProbeState {
            status: ProbeStatus::Verified,
            message: Some("at least one active provider has verified runtime support".to_string()),
        };
    }

    if surface_entries
        .iter()
        .any(|entry| entry.state.status == ProbeStatus::Unknown)
    {
        return CapabilityProbeState {
            status: ProbeStatus::Unknown,
            message: Some(
                "surface is configured but no successful live probe has been recorded".to_string(),
            ),
        };
    }

    if surface_entries
        .iter()
        .any(|entry| entry.state.status == ProbeStatus::Failed)
    {
        return CapabilityProbeState {
            status: ProbeStatus::Failed,
            message: Some(
                "all active providers for this surface failed the live probe".to_string(),
            ),
        };
    }

    CapabilityProbeState {
        status: ProbeStatus::Unsupported,
        message: Some("surface is unsupported by all active providers".to_string()),
    }
}

fn probe_state_ref(probe: Option<&ProviderProbeResult>, capability: &str) -> CapabilityProbeState {
    let check = probe.and_then(|result| {
        result
            .checks
            .iter()
            .find(|check| check.capability == capability)
    });
    CapabilityProbeState {
        status: check
            .map(|value| value.status)
            .unwrap_or(ProbeStatus::Unknown),
        message: check.and_then(|value| value.message.clone()),
    }
}
