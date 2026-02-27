# PRD: Cross-Format Translation

| Field     | Value                    |
|-----------|--------------------------|
| Spec ID   | SPEC-002                 |
| Title     | Cross-Format Translation |
| Author    | AI Proxy Team            |
| Status    | Completed                |
| Created   | 2025-01-01               |
| Updated   | 2025-01-01               |

## Problem Statement

AI providers use incompatible API formats. OpenAI uses `chat/completions` with `messages[].role` and `tool_calls`, Claude uses a Messages API with `content[]` blocks and `tool_use`, and Gemini uses `generateContent` with `parts[]` and `functionCall`. Clients should not need to implement all formats -- the proxy must translate requests and responses transparently so a client speaking one format can reach any provider.

## Goals

- Bidirectional OpenAI-to-Claude translation (request + streaming/non-streaming response)
- Bidirectional OpenAI-to-Gemini translation (request + streaming/non-streaming response)
- Stateful stream translation that tracks accumulation state across SSE chunks
- Preserve semantic fidelity: system messages, tool calls, tool results, images, stop reasons, usage statistics
- Handle the `[DONE]` sentinel correctly across all translation paths
- Passthrough mode for same-format requests (only replace model name for alias resolution)

## Non-Goals

- Direct Claude-to-Gemini translation (requests always go through OpenAI as the intermediary format)
- Translation for non-chat endpoints (e.g., embeddings, image generation)
- Lossless round-tripping of all provider-specific features (e.g., Gemini safety settings, Claude cache tokens)

## User Stories

- As a developer using an OpenAI SDK, I can send requests to Claude models and receive responses in OpenAI format without modifying my code.
- As a developer using an OpenAI SDK, I can send requests to Gemini models and receive responses in OpenAI format.
- As a developer, I can use tool calling (function calling) through the proxy regardless of which provider actually serves the request.
- As a developer, I can stream responses from any provider and receive them in my client's expected SSE format.

## Success Metrics

- All OpenAI Chat Completion fields (messages, tools, tool_choice, temperature, top_p, max_tokens, stop, stream) are correctly translated to Claude and Gemini equivalents
- Tool call round-trips work correctly: OpenAI tool_calls -> Claude tool_use -> Claude tool_result -> OpenAI tool message
- Streaming translation produces valid SSE chunks that OpenAI-compatible clients can parse
- Usage statistics (prompt_tokens, completion_tokens, total_tokens) are mapped from provider-specific fields

## Constraints

- Translation operates on raw JSON (`&[u8]`) using `serde_json::Value` for flexibility, not strongly-typed request structs
- The `TranslatorRegistry` uses function pointers (`fn`), not closures or trait objects, for zero-overhead dispatch

## Design Decisions

| Decision | Options Considered | Chosen | Rationale |
|----------|--------------------|--------|-----------|
| Translation dispatch | Trait objects, enum dispatch, function pointers | Function pointers keyed by `(Format, Format)` | Zero overhead, simple registration, no heap allocation |
| Intermediary format | OpenAI, Claude, custom IR | OpenAI | Most widely adopted; all client SDKs speak it; minimizes translation pairs |
| JSON handling | Typed structs, `serde_json::Value` | `serde_json::Value` | More resilient to upstream API changes; avoids maintaining exact type parity with every provider |
