# PRD: Request Retry & Resilience

| Field     | Value                       |
|-----------|-----------------------------|
| Spec ID   | SPEC-005                    |
| Title     | Request Retry & Resilience  |
| Author    | AI Proxy Team               |
| Status    | Completed                   |
| Created   | 2026-02-27                  |
| Updated   | 2026-02-27                  |

## Problem Statement

Upstream AI APIs (OpenAI, Claude, Gemini) can fail due to rate limits (429), server errors (5xx), and network issues. A production API gateway must automatically retry failed requests, cool down rate-limited credentials, and fail over to alternative providers/credentials to maximize request success rates.

## Goals

- Exponential backoff with full jitter between retry rounds
- Credential cooling on failure: temporarily mark credentials as unavailable with configurable cooldown durations per error type
- Distinct handling for 429 (rate limit), 5xx (server error), and network errors with separate cooldown intervals
- Retry-After header parsing: respect upstream-provided retry timing when available
- Auto failover across credentials within a provider and across providers
- Bootstrap retry limit for streaming: cap retries before first byte is sent to client
- Provider failover: if all credentials for one provider fail, try the next provider format

## Non-Goals

- Circuit breaker pattern (half-open/open/closed states)
- Persistent retry queues or async retry scheduling
- Client-facing retry indication headers

## User Stories

- As an operator, I want rate-limited credentials to automatically cool down so that the proxy stops hammering overloaded endpoints.
- As a user, I want my request to automatically try another API key or provider when one fails so that I get a response without manual intervention.
- As an operator, I want to configure cooldown durations per error type so that I can tune resilience to my traffic patterns.
- As a user, I want streaming requests to fail fast if the provider is down so that I do not wait indefinitely for a stream that will never start.

## Success Metrics

- Requests succeed on retry when at least one credential/provider is healthy
- 429-cooled credentials are not retried until their cooldown expires
- Upstream Retry-After headers are respected when present
- Streaming requests respect bootstrap retry limits

## Constraints

- Retry logic is in `crates/server/src/dispatch.rs`
- Credential cooldown state is managed by `CredentialRouter::mark_unavailable` in `crates/provider/src/routing.rs`
- Must not retry after first byte has been sent to client for streaming responses (only bootstrap retries are allowed)
- Cooldown state is preserved across config hot-reloads via `update_from_config`

## Open Questions

- [x] Should cooldown state survive config reloads? -- Yes, `update_from_config` preserves `cooldown_until` from existing credentials

## Design Decisions

| Decision | Options Considered | Chosen | Rationale |
|----------|--------------------|--------|-----------|
| Backoff strategy | Fixed, linear, exponential | Exponential with full jitter | Prevents thundering herd; standard practice for API rate limiting |
| Cooldown scope | Global, per-provider, per-credential | Per-credential | Allows healthy credentials to continue serving while bad ones cool down |
| Retry-After source | Config only, header only, header-with-fallback | Header with config fallback | Respects upstream guidance when available, uses config defaults otherwise |
| Streaming retry | Unlimited, none, capped | Capped (bootstrap_retries) | Prevents infinite retry loops while allowing recovery from transient failures |
