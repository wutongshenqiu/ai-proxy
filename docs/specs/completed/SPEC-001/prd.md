# PRD: Multi-Provider Routing & Credential Management

| Field     | Value                                         |
|-----------|-----------------------------------------------|
| Spec ID   | SPEC-001                                      |
| Title     | Multi-Provider Routing & Credential Management |
| Author    | AI Proxy Team                                 |
| Status    | Completed                                     |
| Created   | 2025-01-01                                    |
| Updated   | 2025-01-01                                    |

## Problem Statement

Applications that consume AI APIs are often locked into a single provider. Switching providers requires code changes, and managing multiple API keys across providers is error-prone. There is no unified gateway that can route requests to Claude, OpenAI, Gemini, and OpenAI-compatible providers while transparently managing credentials, handling key rotation, and supporting model aliasing.

## Goals

- Support four provider formats: OpenAI, Claude, Gemini, and OpenAI-compatible (e.g., DeepSeek, Groq)
- Route requests to the correct provider based on model name and credential configuration
- Support round-robin and fill-first credential routing strategies
- Enable credential cooldown to temporarily remove failing keys from rotation
- Allow model aliasing so users can reference models by short names (e.g., `sonnet` -> `claude-sonnet-4-20250514`)
- Support prefix-based routing to disambiguate models across providers (e.g., `anthropic/claude-sonnet-4-20250514`)
- Allow excluding specific models from a credential via glob patterns
- Provide a `/v1/models` endpoint listing all available models across providers

## Non-Goals

- Latency-based load balancing (selecting the fastest provider)
- Geographic routing (routing to the nearest provider region)
- Per-user or per-tenant credential isolation
- Rate limiting at the proxy level

## User Stories

- As a developer, I can send OpenAI-format requests and have them transparently routed to any configured provider (Claude, Gemini, etc.) without changing my client code.
- As an admin, I can configure multiple API keys per provider, each with different model lists, and have the proxy rotate between them.
- As an admin, I can set a model alias (e.g., `sonnet`) so developers do not need to know the full model ID.
- As an admin, I can prefix model names per credential (e.g., `team-a/gpt-4o`) to isolate key usage.
- As an admin, I can exclude specific models from a key using glob patterns (e.g., `*preview*`).
- As an operator, when an API key hits a rate limit, I want the proxy to cool down that key and try the next available one.

## Success Metrics

- Transparent provider switching: clients do not observe which upstream provider served their request
- Zero-downtime credential rotation: adding/removing keys via config reload does not interrupt traffic
- Cooldown state preservation: config reloads preserve existing cooldown timers on keys

## Constraints

- Config is YAML-based; credential entries live under `claude-api-key`, `openai-api-key`, `gemini-api-key`, and `openai-compatibility` arrays
- Hot-reload via file watcher (debounced, SHA256 dedup) atomically swaps config via `ArcSwap`

## Design Decisions

| Decision | Options Considered | Chosen | Rationale |
|----------|--------------------|--------|-----------|
| Routing strategy | Round-robin, fill-first, weighted, latency-based | Round-robin + fill-first | Simple, predictable; covers most use cases without complexity |
| Credential ID | API key hash, UUID | UUID (generated per reload) | Avoids exposing key material in logs; cheap to generate |
| Model matching | Exact match, regex, glob | Glob patterns | Familiar syntax, sufficient for model name patterns like `gpt-4*` |
