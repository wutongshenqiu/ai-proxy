# PRD: SSE Streaming

| Field     | Value          |
|-----------|----------------|
| Spec ID   | SPEC-003       |
| Title     | SSE Streaming  |
| Author    | AI Proxy Team  |
| Status    | Completed      |
| Created   | 2025-01-01     |
| Updated   | 2025-01-01     |

## Problem Statement

AI model responses can take several seconds to generate. Without streaming, clients must wait for the entire response before seeing any output, resulting in poor user experience. The proxy must parse Server-Sent Events (SSE) from upstream providers, translate them if necessary, and forward them to clients in real time. Additionally, intermediate proxies and load balancers may terminate idle connections, so the proxy needs keepalive mechanisms for both streaming and non-streaming requests.

## Goals

- Parse SSE byte streams from upstream providers into structured events (`SseEvent`)
- Forward translated SSE events to clients with correct `text/event-stream` framing
- Send periodic keepalive comments during streaming to prevent intermediate proxy timeouts
- Provide bootstrap retry: retry the upstream connection if it fails before the first byte is received
- Support non-stream keepalive: send periodic whitespace during long-running non-streaming requests to keep the client connection alive

## Non-Goals

- WebSocket streaming support
- Client-side backpressure or flow control
- Multiplexing multiple upstream streams into a single client connection

## User Stories

- As a developer, I can stream responses from any provider and see tokens appear in real time.
- As an operator, I can configure keepalive intervals to prevent Nginx/Cloudflare from terminating idle SSE connections.
- As an operator, I can configure bootstrap retries so that transient upstream connection failures are retried before the client sees an error.
- As an operator, I can enable non-stream keepalive so that long-running non-streaming requests do not time out through intermediate proxies.

## Success Metrics

- SSE events from all providers (OpenAI, Claude, Gemini) are correctly parsed and forwarded
- Keepalive comments (`:` lines) are sent at the configured interval during streaming
- Bootstrap retries recover from transient connection failures without client awareness
- Non-stream keepalive prevents timeout for requests taking longer than typical proxy idle timeouts (60-120s)

## Constraints

- SSE parsing must handle both `\n\n` and `\r\n\r\n` event boundaries
- The parser must handle multi-line `data:` fields (concatenated with `\n`)
- Comment lines (starting with `:`) must be ignored during parsing
- The `[DONE]` sentinel must be passed through as-is

## Design Decisions

| Decision | Options Considered | Chosen | Rationale |
|----------|--------------------|--------|-----------|
| SSE parser | Third-party crate, custom parser | Custom parser using `futures::stream::unfold` | Full control over buffering and error handling; SSE is a simple protocol |
| Keepalive mechanism | Custom timer, Axum's `KeepAlive` | Axum's built-in `KeepAlive` on `Sse` response | Built-in, reliable, configurable interval |
| Non-stream keepalive | Background task, middleware | Config-driven interval sending whitespace | Simple, avoids middleware complexity |
