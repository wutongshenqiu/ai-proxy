# Technical Design: Cross-Format Translation

| Field     | Value                    |
|-----------|--------------------------|
| Spec ID   | SPEC-002                 |
| Title     | Cross-Format Translation |
| Author    | AI Proxy Team            |
| Status    | Completed                |
| Created   | 2025-01-01               |
| Updated   | 2025-01-01               |

## Overview

The translator layer converts requests and responses between provider-specific API formats. It is organized as a registry of function-pointer pairs keyed by `(source_format, target_format)`. Translation has three paths: request translation, streaming response translation (stateful), and non-streaming response translation. See SPEC-002 PRD for requirements.

## Module Structure

```
crates/translator/src/
  lib.rs                # TranslatorRegistry, TranslateState, build_registry()
  openai_to_claude.rs   # OpenAI -> Claude request translation
  claude_to_openai.rs   # Claude -> OpenAI response translation (stream + non-stream)
  openai_to_gemini.rs   # OpenAI -> Gemini request translation
  gemini_to_openai.rs   # Gemini -> OpenAI response translation (stream + non-stream)
crates/core/src/types/
  openai.rs             # OpenAI type definitions
  claude.rs             # Claude type definitions
  gemini.rs             # Gemini type definitions
```

## Key Types

### TranslatorRegistry

```rust
pub struct TranslatorRegistry {
    requests: HashMap<(Format, Format), RequestTransformFn>,
    responses: HashMap<(Format, Format), ResponseTransform>,
}
```

The registry stores separate maps for request and response transformations, keyed by `(from_format, to_format)`.

Function pointer types:

```rust
pub type RequestTransformFn =
    fn(model: &str, raw_json: &[u8], stream: bool) -> Result<Vec<u8>, ProxyError>;

pub type StreamTransformFn = fn(
    model: &str,
    original_req: &[u8],
    event_type: Option<&str>,
    data: &[u8],
    state: &mut TranslateState,
) -> Result<Vec<String>, ProxyError>;

pub type NonStreamTransformFn =
    fn(model: &str, original_req: &[u8], data: &[u8]) -> Result<String, ProxyError>;

pub struct ResponseTransform {
    pub stream: StreamTransformFn,
    pub non_stream: NonStreamTransformFn,
}
```

### TranslateState

Accumulates state across streaming chunks for stateful translation:

```rust
pub struct TranslateState {
    pub response_id: String,             // Generated response ID (e.g., "chatcmpl-xxx")
    pub model: String,                   // Model name from upstream
    pub created: i64,                    // Unix timestamp
    pub current_tool_call_index: i32,    // Tracks tool_call index across chunks (-1 initial)
    pub current_content_index: i32,      // Tracks content block index (-1 initial)
    pub sent_role: bool,                 // Whether the initial role chunk has been sent
    pub input_tokens: u64,              // Input token count from message_start (Claude)
}
```

### Registry Methods

| Method | Behavior |
|--------|----------|
| `translate_request(from, to, model, json, stream)` | If `from == to`, replaces the `model` field in the JSON payload (for alias resolution) and returns. Otherwise looks up the `(from, to)` request transform function and calls it. |
| `translate_stream(from, to, model, orig_req, event_type, data, state)` | If `from == to`, passes through raw data. When `from != to` and `data == "[DONE]"`, passes through `[DONE]` as-is (individual translators also produce their own `[DONE]` on terminal events like `message_stop`). Otherwise calls the stream transform function. |
| `translate_non_stream(from, to, model, orig_req, data)` | If `from == to`, passes through. Otherwise calls the non-stream transform function. |
| `has_response_translator(from, to)` | Returns `true` if `from != to` and a response transform exists. |

### Registered Translation Pairs

Built by `build_registry()`:

| Direction | Request Transform | Response Transform |
|-----------|-------------------|--------------------|
| OpenAI -> Claude | `openai_to_claude::translate_request` | `claude_to_openai::translate_stream` / `translate_non_stream` |
| OpenAI -> Gemini | `openai_to_gemini::translate_request` | `gemini_to_openai::translate_stream` / `translate_non_stream` |

Note: The response transforms are registered under the same `(from, to)` key as the request. When a request goes OpenAI->Claude, the response comes back Claude->OpenAI using the same key pair.

## OpenAI -> Claude Request Translation

`openai_to_claude::translate_request(model, raw_json, stream)`

### Conversion Steps

1. **System messages**: Extracts all messages with `role: "system"` from the `messages` array, concatenates their text content with `\n\n`, and places the result in Claude's top-level `system` field.

2. **Message conversion**:
   - `role: "user"` -> `role: "user"` with content converted (string or multipart with text/image blocks)
   - `role: "assistant"` -> `role: "assistant"` with text -> `{"type": "text", "text": ...}` blocks; `tool_calls` -> `{"type": "tool_use", "id": ..., "name": ..., "input": ...}` blocks
   - `role: "tool"` -> `role: "user"` with `{"type": "tool_result", "tool_use_id": ..., "content": ...}` blocks. Multiple consecutive tool results merge into the same user message.

3. **Image handling**: `image_url` content parts are converted to Claude's `image` blocks. Base64 data URLs (`data:image/png;base64,...`) become `{"type": "base64", "media_type": ..., "data": ...}`. Regular URLs become `{"type": "url", "url": ...}`.

4. **Tools**: OpenAI `{"type": "function", "function": {"name", "description", "parameters"}}` -> Claude `{"name", "description", "input_schema"}`.

5. **Parameters**: `max_tokens` or `max_completion_tokens` -> `max_tokens` (default 8192). `temperature`, `top_p` forwarded. `stop` -> `stop_sequences` (string wrapped in array). `thinking` forwarded as-is (extended thinking support).

6. **Tool choice**: `"none"` -> `{"type": "none"}`, `"auto"` -> `{"type": "auto"}`, `"required"` -> `{"type": "any"}`, `{"function": {"name": X}}` -> `{"type": "tool", "name": X}`.

## Claude -> OpenAI Response Translation

### Non-Streaming (`claude_to_openai::translate_non_stream`)

Maps a Claude Messages response to an OpenAI ChatCompletion response:

| Claude Field | OpenAI Field |
|-------------|-------------|
| `id` | `id` (prefixed with `chatcmpl-`) |
| `model` | `model` |
| `content[].type == "text"` | `choices[0].message.content` (concatenated) |
| `content[].type == "tool_use"` | `choices[0].message.tool_calls[]` with `{"id", "type": "function", "function": {"name", "arguments"}}` |
| `stop_reason: "end_turn"` | `finish_reason: "stop"` |
| `stop_reason: "max_tokens"` | `finish_reason: "length"` |
| `stop_reason: "tool_use"` | `finish_reason: "tool_calls"` |
| `stop_reason: "stop_sequence"` | `finish_reason: "stop"` |
| `usage.input_tokens` | `usage.prompt_tokens` |
| `usage.output_tokens` | `usage.completion_tokens` |

### Streaming (`claude_to_openai::translate_stream`)

Translates Claude SSE events (dispatched by `event_type`) to OpenAI `chat.completion.chunk` objects:

| Claude Event | OpenAI Chunk |
|-------------|-------------|
| `message_start` | Initial chunk with `delta: {"role": "assistant", "content": ""}`. Extracts `id`, `model`, `usage.input_tokens` into `TranslateState`. |
| `content_block_start` (type `tool_use`) | Chunk with `delta.tool_calls[{index, id, type: "function", function: {name, arguments: ""}}]`. Increments `current_tool_call_index`. |
| `content_block_delta` (type `text_delta`) | Chunk with `delta: {"content": text}` |
| `content_block_delta` (type `input_json_delta`) | Chunk with `delta.tool_calls[{index, function: {arguments: partial_json}}]` |
| `message_delta` | Chunk with `finish_reason` mapped (same as non-stream) + `usage` if present |
| `message_stop` | Emits `[DONE]` sentinel |
| `ping`, `content_block_stop` | Skipped (no output) |

## OpenAI -> Gemini Request Translation

`openai_to_gemini::translate_request(model, raw_json, stream)`

### Conversion Steps

1. **System messages**: Extracted into Gemini's `systemInstruction` field as `{"parts": [{"text": ...}]}`.

2. **Message conversion**:
   - `role: "user"` -> `role: "user"` with `parts: [{"text": ...}]`
   - `role: "assistant"` -> `role: "model"` with text parts and `tool_calls` converted to `functionCall` parts
   - `role: "tool"` -> `role: "user"` with `functionResponse` parts. Content is parsed as JSON if possible, otherwise wrapped in `{"result": content}`.
   - Consecutive messages with the same Gemini role are merged (parts appended).

3. **Image handling**: Base64 data URLs become `inlineData` parts. Regular URLs are converted to text references since Gemini does not support direct URL image input.

4. **Tools**: OpenAI tools -> Gemini `[{"functionDeclarations": [...]}]` with `name`, `description`, and optional `parameters`.

5. **Generation config**: `temperature` -> `temperature`, `top_p` -> `topP`, `max_tokens`/`max_completion_tokens` -> `maxOutputTokens`, `stop` -> `stopSequences`.

6. **Model in body**: Gemini uses the model name in the URL path, not in the request body. The `model` parameter is unused in the body.

## Gemini -> OpenAI Response Translation

### Non-Streaming (`gemini_to_openai::translate_non_stream`)

| Gemini Field | OpenAI Field |
|-------------|-------------|
| `modelVersion` | `model` |
| `candidates[0].content.parts[].text` | `choices[0].message.content` (concatenated) |
| `candidates[0].content.parts[].functionCall` | `choices[0].message.tool_calls[]` (with generated `call_xxx` IDs) |
| `candidates[0].finishReason: "STOP"` | `finish_reason: "stop"` |
| `candidates[0].finishReason: "MAX_TOKENS"` | `finish_reason: "length"` |
| `candidates[0].finishReason: "SAFETY"/"RECITATION"` | `finish_reason: "content_filter"` |
| `usageMetadata.promptTokenCount` | `usage.prompt_tokens` |
| `usageMetadata.candidatesTokenCount` | `usage.completion_tokens` |
| `usageMetadata.totalTokenCount` | `usage.total_tokens` |

### Streaming (`gemini_to_openai::translate_stream`)

Gemini streams do not use named event types; each SSE data payload is a complete `GenerateContentResponse`. Translation:

1. On first chunk, initializes `TranslateState` with a generated `chatcmpl-xxx` ID and emits an initial role chunk.
2. For each chunk, extracts `candidates[0].content.parts[]`:
   - `text` parts -> `delta: {"content": text}` chunks
   - `functionCall` parts -> `delta.tool_calls[{index, id, type, function: {name, arguments}}]` chunks (full arguments in one chunk, unlike Claude's incremental JSON)
3. When `finishReason` is present, emits a final chunk with `finish_reason` + optional `usage`, followed by `[DONE]`.
4. Updates `state.model` from `modelVersion` if available in the response.

## Provider Compatibility

| Provider | Request Translation | Response Translation | Notes |
|----------|--------------------|-----------------------|-------|
| OpenAI   | Passthrough (model replacement only) | Passthrough | Native format |
| Claude   | OpenAI -> Claude | Claude -> OpenAI | Full bidirectional support |
| Gemini   | OpenAI -> Gemini | Gemini -> OpenAI | Full bidirectional support |
| OpenAI-compat | Passthrough | Passthrough | Uses OpenAI format natively |

## Test Strategy

- **Unit tests:** Request translation correctness (system message extraction, role mapping, tool conversion, image conversion), response translation (stop_reason mapping, usage mapping, tool_call serialization)
- **Stream tests:** Multi-event stream translation producing correct sequence of OpenAI chunks, state accumulation across events, `[DONE]` sentinel handling
