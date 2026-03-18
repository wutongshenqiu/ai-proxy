use prism_core::presentation::{ActivationMode, ProfileKind};
use prism_core::provider::{Format, UpstreamKind, WireApi};
use prism_domain::capability::{ProviderCapabilities, UpstreamProtocol};
use prism_domain::operation::{ExecutionMode, IngressProtocol, Operation};
use serde::Serialize;

use crate::handler::dashboard::providers::ProbeStatus;

use super::specs::{EndpointScope, EndpointTransport, StreamTransport};

#[derive(Debug, Serialize)]
pub struct ProtocolMatrixResponse {
    pub endpoints: Vec<ProtocolEndpointEntry>,
    pub coverage: Vec<ProtocolCoverageEntry>,
}

#[derive(Debug, Serialize)]
pub struct ProtocolEndpointEntry {
    pub id: String,
    pub family: IngressProtocol,
    pub method: String,
    pub path: String,
    pub description: String,
    pub scope: EndpointScope,
    pub transport: EndpointTransport,
    pub operation: Operation,
    pub stream_transport: StreamTransport,
    pub state: CapabilityProbeState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ProtocolCoverageEntry {
    pub provider: String,
    pub format: Format,
    pub upstream: UpstreamKind,
    pub upstream_protocol: UpstreamProtocol,
    pub wire_api: WireApi,
    pub disabled: bool,
    pub surface_id: String,
    pub surface_label: String,
    pub ingress_protocol: IngressProtocol,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_mode: Option<ExecutionMode>,
    pub state: CapabilityProbeState,
}

#[derive(Debug, Serialize)]
pub struct ProviderCapabilitiesResponse {
    pub providers: Vec<ProviderCapabilityEntry>,
}

#[derive(Debug, Serialize)]
pub struct ProviderModelEntry {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ProviderCapabilityEntry {
    pub name: String,
    pub format: Format,
    pub upstream: UpstreamKind,
    pub upstream_protocol: UpstreamProtocol,
    pub wire_api: WireApi,
    pub presentation_profile: ProfileKind,
    pub presentation_mode: ActivationMode,
    pub models: Vec<ProviderModelEntry>,
    pub capabilities: ProviderCapabilities,
    pub probe_status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checked_at: Option<String>,
    pub probe: CapabilityProbeStates,
    pub disabled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CapabilityProbeStates {
    pub text: CapabilityProbeState,
    pub stream: CapabilityProbeState,
    pub tools: CapabilityProbeState,
    pub images: CapabilityProbeState,
    pub json_schema: CapabilityProbeState,
    pub reasoning: CapabilityProbeState,
    pub count_tokens: CapabilityProbeState,
}

#[derive(Debug, Clone, Serialize)]
pub struct CapabilityProbeState {
    pub status: ProbeStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}
