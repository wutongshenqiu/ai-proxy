use prism_core::provider::Format;
use prism_domain::operation::{IngressProtocol, Operation};
use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EndpointScope {
    Public,
    ProviderScoped,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EndpointTransport {
    Http,
    WebSocket,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamTransport {
    None,
    Sse,
    WebSocketEvents,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum SurfaceProbeKind {
    Text,
    Stream,
    CountTokens,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct SurfaceSpec {
    pub(crate) id: &'static str,
    pub(crate) label: &'static str,
    pub(crate) ingress_protocol: IngressProtocol,
    pub(crate) allowed_formats: &'static [Format],
    pub(crate) probe_kind: SurfaceProbeKind,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct EndpointSpec {
    pub(crate) id: &'static str,
    pub(crate) family: IngressProtocol,
    pub(crate) method: &'static str,
    pub(crate) path: &'static str,
    pub(crate) description: &'static str,
    pub(crate) scope: EndpointScope,
    pub(crate) transport: EndpointTransport,
    pub(crate) operation: Operation,
    pub(crate) stream_transport: StreamTransport,
    pub(crate) surface_id: Option<&'static str>,
    pub(crate) note: Option<&'static str>,
}

const ANY_FORMAT: &[Format] = &[Format::OpenAI, Format::Claude, Format::Gemini];
const OPENAI_ONLY: &[Format] = &[Format::OpenAI];
const CLAUDE_ONLY: &[Format] = &[Format::Claude];

pub(crate) const SURFACE_SPECS: &[SurfaceSpec] = &[
    SurfaceSpec {
        id: "openai_chat",
        label: "OpenAI Chat",
        ingress_protocol: IngressProtocol::OpenAi,
        allowed_formats: ANY_FORMAT,
        probe_kind: SurfaceProbeKind::Text,
    },
    SurfaceSpec {
        id: "openai_responses",
        label: "OpenAI Responses",
        ingress_protocol: IngressProtocol::OpenAi,
        allowed_formats: OPENAI_ONLY,
        probe_kind: SurfaceProbeKind::Text,
    },
    SurfaceSpec {
        id: "openai_responses_ws",
        label: "OpenAI Responses WS",
        ingress_protocol: IngressProtocol::OpenAi,
        allowed_formats: OPENAI_ONLY,
        probe_kind: SurfaceProbeKind::Stream,
    },
    SurfaceSpec {
        id: "claude_messages",
        label: "Claude Messages",
        ingress_protocol: IngressProtocol::Claude,
        allowed_formats: ANY_FORMAT,
        probe_kind: SurfaceProbeKind::Text,
    },
    SurfaceSpec {
        id: "claude_count_tokens",
        label: "Claude Count Tokens",
        ingress_protocol: IngressProtocol::Claude,
        allowed_formats: CLAUDE_ONLY,
        probe_kind: SurfaceProbeKind::CountTokens,
    },
    SurfaceSpec {
        id: "gemini_generate",
        label: "Gemini Generate",
        ingress_protocol: IngressProtocol::Gemini,
        allowed_formats: ANY_FORMAT,
        probe_kind: SurfaceProbeKind::Text,
    },
    SurfaceSpec {
        id: "gemini_stream",
        label: "Gemini Stream",
        ingress_protocol: IngressProtocol::Gemini,
        allowed_formats: ANY_FORMAT,
        probe_kind: SurfaceProbeKind::Stream,
    },
];

pub(crate) const ENDPOINT_SPECS: &[EndpointSpec] = &[
    EndpointSpec {
        id: "openai_chat_completions",
        family: IngressProtocol::OpenAi,
        method: "POST",
        path: "/v1/chat/completions",
        description: "Unified OpenAI Chat Completions ingress.",
        scope: EndpointScope::Public,
        transport: EndpointTransport::Http,
        operation: Operation::Generate,
        stream_transport: StreamTransport::Sse,
        surface_id: Some("openai_chat"),
        note: None,
    },
    EndpointSpec {
        id: "openai_completions",
        family: IngressProtocol::OpenAi,
        method: "POST",
        path: "/v1/completions",
        description: "Legacy OpenAI Completions compatibility route.",
        scope: EndpointScope::Public,
        transport: EndpointTransport::Http,
        operation: Operation::Generate,
        stream_transport: StreamTransport::Sse,
        surface_id: Some("openai_chat"),
        note: None,
    },
    EndpointSpec {
        id: "openai_responses",
        family: IngressProtocol::OpenAi,
        method: "POST",
        path: "/v1/responses",
        description: "Native OpenAI Responses passthrough for OpenAI-format upstreams.",
        scope: EndpointScope::Public,
        transport: EndpointTransport::Http,
        operation: Operation::Generate,
        stream_transport: StreamTransport::Sse,
        surface_id: Some("openai_responses"),
        note: None,
    },
    EndpointSpec {
        id: "openai_responses_ws",
        family: IngressProtocol::OpenAi,
        method: "GET",
        path: "/v1/responses/ws",
        description: "WebSocket facade over Responses SSE with create/append semantics.",
        scope: EndpointScope::Public,
        transport: EndpointTransport::WebSocket,
        operation: Operation::Generate,
        stream_transport: StreamTransport::WebSocketEvents,
        surface_id: Some("openai_responses_ws"),
        note: Some("Terminal completion is signaled by response.completed, not [DONE]."),
    },
    EndpointSpec {
        id: "openai_models",
        family: IngressProtocol::OpenAi,
        method: "GET",
        path: "/v1/models",
        description: "Gateway-local model registry for OpenAI clients.",
        scope: EndpointScope::Public,
        transport: EndpointTransport::Http,
        operation: Operation::ListModels,
        stream_transport: StreamTransport::None,
        surface_id: None,
        note: Some("Served from configured provider inventory, not upstream model listing."),
    },
    EndpointSpec {
        id: "claude_messages",
        family: IngressProtocol::Claude,
        method: "POST",
        path: "/v1/messages",
        description: "Unified Claude Messages ingress.",
        scope: EndpointScope::Public,
        transport: EndpointTransport::Http,
        operation: Operation::Generate,
        stream_transport: StreamTransport::Sse,
        surface_id: Some("claude_messages"),
        note: None,
    },
    EndpointSpec {
        id: "claude_count_tokens",
        family: IngressProtocol::Claude,
        method: "POST",
        path: "/v1/messages/count_tokens",
        description: "Direct proxy to Anthropic count_tokens for Claude-format providers.",
        scope: EndpointScope::Public,
        transport: EndpointTransport::Http,
        operation: Operation::CountTokens,
        stream_transport: StreamTransport::None,
        surface_id: Some("claude_count_tokens"),
        note: None,
    },
    EndpointSpec {
        id: "gemini_models",
        family: IngressProtocol::Gemini,
        method: "GET",
        path: "/v1beta/models",
        description: "Gateway-local Gemini model registry.",
        scope: EndpointScope::Public,
        transport: EndpointTransport::Http,
        operation: Operation::ListModels,
        stream_transport: StreamTransport::None,
        surface_id: None,
        note: Some("Served from configured provider inventory, not upstream model listing."),
    },
    EndpointSpec {
        id: "gemini_generate",
        family: IngressProtocol::Gemini,
        method: "POST",
        path: "/v1beta/models/{model}:generateContent",
        description: "Unified Gemini generateContent ingress.",
        scope: EndpointScope::Public,
        transport: EndpointTransport::Http,
        operation: Operation::Generate,
        stream_transport: StreamTransport::None,
        surface_id: Some("gemini_generate"),
        note: None,
    },
    EndpointSpec {
        id: "gemini_stream",
        family: IngressProtocol::Gemini,
        method: "POST",
        path: "/v1beta/models/{model}:streamGenerateContent",
        description: "Unified Gemini streaming ingress.",
        scope: EndpointScope::Public,
        transport: EndpointTransport::Http,
        operation: Operation::Generate,
        stream_transport: StreamTransport::Sse,
        surface_id: Some("gemini_stream"),
        note: None,
    },
    EndpointSpec {
        id: "provider_openai_chat",
        family: IngressProtocol::OpenAi,
        method: "POST",
        path: "/api/provider/{provider}/v1/chat/completions",
        description: "Provider-pinned OpenAI Chat route for deterministic routing.",
        scope: EndpointScope::ProviderScoped,
        transport: EndpointTransport::Http,
        operation: Operation::Generate,
        stream_transport: StreamTransport::Sse,
        surface_id: Some("openai_chat"),
        note: Some("Bypasses provider selection and pins requests to the named provider."),
    },
    EndpointSpec {
        id: "provider_claude_messages",
        family: IngressProtocol::Claude,
        method: "POST",
        path: "/api/provider/{provider}/v1/messages",
        description: "Provider-pinned Claude Messages route.",
        scope: EndpointScope::ProviderScoped,
        transport: EndpointTransport::Http,
        operation: Operation::Generate,
        stream_transport: StreamTransport::Sse,
        surface_id: Some("claude_messages"),
        note: Some("Bypasses provider selection and pins requests to the named provider."),
    },
    EndpointSpec {
        id: "provider_openai_responses",
        family: IngressProtocol::OpenAi,
        method: "POST",
        path: "/api/provider/{provider}/v1/responses",
        description: "Provider-pinned OpenAI Responses passthrough.",
        scope: EndpointScope::ProviderScoped,
        transport: EndpointTransport::Http,
        operation: Operation::Generate,
        stream_transport: StreamTransport::Sse,
        surface_id: Some("openai_responses"),
        note: Some("Only available for OpenAI-format providers."),
    },
    EndpointSpec {
        id: "provider_openai_responses_ws",
        family: IngressProtocol::OpenAi,
        method: "GET",
        path: "/api/provider/{provider}/v1/responses/ws",
        description: "Provider-pinned WebSocket Responses facade.",
        scope: EndpointScope::ProviderScoped,
        transport: EndpointTransport::WebSocket,
        operation: Operation::Generate,
        stream_transport: StreamTransport::WebSocketEvents,
        surface_id: Some("openai_responses_ws"),
        note: Some("Preserves Codex previous_response_id when the pinned provider is Codex."),
    },
];
