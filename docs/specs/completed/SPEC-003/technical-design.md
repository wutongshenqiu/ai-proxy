# Technical Design: SSE Streaming

| Field     | Value          |
|-----------|----------------|
| Spec ID   | SPEC-003       |
| Title     | SSE Streaming  |
| Author    | AI Proxy Team  |
| Status    | Completed      |
| Created   | 2025-01-01     |
| Updated   | 2025-01-01     |

## Overview

The SSE streaming subsystem handles parsing upstream provider byte streams into structured events, translating them across formats, and writing them back as SSE to the client. It includes keepalive mechanisms to prevent timeout at intermediate proxies and bootstrap retry logic for transient connection failures. See SPEC-003 PRD for requirements.

## Module Structure

```
crates/provider/src/
  sse.rs            # SseEvent, parse_sse_stream(), SSE parser internals
crates/server/src/
  streaming.rs      # build_sse_response() -- Axum SSE response builder
crates/core/src/
  config.rs         # StreamingConfig (keepalive_seconds, bootstrap_retries),
                    # non_stream_keepalive_secs
```

## Key Types

### SseEvent

```rust
#[derive(Debug, Clone)]
pub struct SseEvent {
    pub event: Option<String>,   // SSE event type (from "event:" line)
    pub data: String,            // SSE data payload (from "data:" lines, joined with "\n")
}
```

### StreamingConfig

```rust
pub struct StreamingConfig {
    pub keepalive_seconds: u64,      // Default: 15
    pub bootstrap_retries: u32,      // Default: 1
}
```

### Config-Level Fields

```rust
// In Config struct:
pub streaming: StreamingConfig,
pub non_stream_keepalive_secs: u64,   // Default: 0 (disabled)
```

## SSE Parser: `parse_sse_stream`

```rust
pub fn parse_sse_stream(
    byte_stream: impl Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static,
) -> Pin<Box<dyn Stream<Item = Result<SseEvent, ProxyError>> + Send>>
```

Converts a raw byte stream (from `reqwest::Response::bytes_stream()`) into a stream of `SseEvent` values.

### Parser Algorithm

The parser uses `futures::stream::unfold` with an internal `SseState` that holds a pinned byte stream and a `String` buffer.

**Main loop:**

1. **Check buffer for complete event**: Search for a double-newline boundary (`\n\n` or `\r\n\r\n`) in the buffer using `find_event_boundary()`.
2. **If boundary found**: Extract the block before the boundary, advance the buffer past the boundary (2 or 4 bytes depending on line ending style), parse the block with `parse_event_block()`, and yield the event if non-empty.
3. **If no boundary**: Read the next chunk from the byte stream, decode as UTF-8, and append to the buffer. On stream end, process any remaining buffered data.

### `parse_event_block(block)` Parsing Rules

For each line in the block:

| Line Pattern | Action |
|-------------|--------|
| Starts with `:` | Comment -- skip |
| `event: <value>` | Set `event_type` to trimmed value |
| `data: <value>` | Append left-trimmed value (`trim_start()`) to `data_lines` |
| `id:` or `retry:` | Ignored |

If `data_lines` is empty after processing all lines, returns `None` (empty/comment-only event block). Otherwise returns `SseEvent { event: event_type, data: data_lines.join("\n") }`.

> Note: `[DONE]` sentinels are **not** filtered by `parse_event_block` — they are returned as a normal `SseEvent` with `data: "[DONE]"`. Filtering of `[DONE]` is handled downstream by the consumer (e.g., `build_sse_response`).

Multi-line data is supported: multiple `data:` lines within the same event block are joined with `\n`.

## Streaming Pipeline

The full streaming pipeline from upstream to client:

```
reqwest Response
  -> bytes_stream()       (Stream<Item = Result<Bytes, reqwest::Error>>)
  -> parse_sse_stream()   (Stream<Item = Result<SseEvent, ProxyError>>)
  -> translator            (stream mode: event_type + data -> Vec<String>)
  -> build_sse_response()  (Axum Sse<...> with keepalive)
  -> HTTP response to client
```

Each `SseEvent` from the parser is fed to the translator's `translate_stream()` method, which may produce zero or more output strings. These strings are then written as SSE events to the client by `build_sse_response()`.

## `build_sse_response`

```rust
pub fn build_sse_response(
    data_stream: impl Stream<Item = Result<String, ProxyError>> + Send + 'static,
    keepalive_seconds: u64,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>>
```

Converts a stream of translated data strings into an Axum `Sse` response.

### Processing Rules

The function first filters out empty strings, then flat-maps each string by splitting on `\n` and processing each line:

| Input Line | Output SSE Event |
|-----------|-----------------|
| Empty / whitespace | Skipped |
| `[DONE]` | `Event::default().data("[DONE]")` |
| `event: <type>` | `Event::default().event(<type>)` |
| `data: <payload>` | `Event::default().data(<payload>)` |
| Raw JSON | `Event::default().data(<line>)` |

Errors are serialized as `{"error": {"message": "..."}}` data events.

### Keepalive

The `Sse` response is configured with Axum's built-in `KeepAlive`:

```rust
Sse::new(stream).keep_alive(
    KeepAlive::new()
        .interval(Duration::from_secs(keepalive_seconds))
        .text(""),
)
```

This sends an empty SSE comment (`: \n\n`) at the configured interval when no data events are being sent. Default interval: 15 seconds.

## Bootstrap Retries

Configured via `streaming.bootstrap_retries` (default: 1). When a streaming request to an upstream provider fails before the first byte is received, the proxy retries the connection up to `bootstrap_retries` times. This handles transient TCP/TLS failures without exposing them to the client.

Bootstrap retries only apply before the first byte -- once streaming has started, failures are propagated to the client.

## Non-Stream Keepalive

Configured via `non_stream_keepalive_secs` (default: 0, disabled). For non-streaming requests that may take a long time (e.g., complex reasoning tasks), the proxy can send periodic whitespace bytes to the client to prevent intermediate proxies from terminating the connection due to idle timeout.

When enabled (`> 0`), a background task sends whitespace at the configured interval while waiting for the upstream response to complete.

## Configuration

```yaml
streaming:
  keepalive-seconds: 15       # SSE keepalive comment interval
  bootstrap-retries: 1        # Retry count before first byte

non-stream-keepalive-secs: 0  # 0 = disabled; set to e.g. 30 for long requests
```

## Provider Compatibility

| Provider | SSE Format | Event Types | Notes |
|----------|-----------|-------------|-------|
| OpenAI   | Standard SSE | No named events; `data:` only | `[DONE]` sentinel marks end |
| Claude   | Named SSE events | `message_start`, `content_block_start`, `content_block_delta`, `content_block_stop`, `message_delta`, `message_stop`, `ping` | Event type in `event:` line |
| Gemini   | Standard SSE | No named events; `data:` only | Each chunk is a complete `GenerateContentResponse` |
| OpenAI-compat | Standard SSE | Same as OpenAI | Depends on provider implementation |

## Test Strategy

- **Unit tests:** `parse_event_block` for basic data, event type extraction, `[DONE]` sentinel, multi-line data, comment lines
- **Integration tests:** Full `parse_sse_stream` with simulated byte streams, boundary handling (`\n\n` vs `\r\n\r\n`), stream termination with buffered data
- **Manual verification:** End-to-end streaming through the proxy with each provider, keepalive visible in network traces
