# SPEC-012: Rate Limiting

## Problem

Without rate limiting, misconfigured scripts or excessive usage can exhaust upstream provider quotas, leading to account bans or degraded service.

## Goals

- G1: Per-API-key RPM limit (sliding window, in-memory)
- G2: Global RPM limit
- G3: Standard `x-ratelimit-*` response headers
- G4: HTTP 429 + `Retry-After` when limit exceeded
- G5: Hot-reload support

## Implementation

- `crates/core/src/rate_limit.rs` — `RateLimiter` with sliding window algorithm
- `crates/core/src/config.rs` — `RateLimitConfig` (enabled, global-rpm, per-key-rpm)
- `crates/core/src/error.rs` — `RateLimited` error variant
- `crates/server/src/middleware/rate_limit.rs` — rate limit middleware
- `AppState` gains `rate_limiter: Arc<RateLimiter>`
- Config hot-reload updates rate limiter config

## Status

Active — Implementation complete, pending review.
