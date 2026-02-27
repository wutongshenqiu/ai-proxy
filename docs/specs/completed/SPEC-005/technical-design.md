# Technical Design: Request Retry & Resilience

| Field     | Value                       |
|-----------|-----------------------------|
| Spec ID   | SPEC-005                    |
| Title     | Request Retry & Resilience  |
| Author    | AI Proxy Team               |
| Status    | Completed                   |
| Created   | 2026-02-27                  |
| Updated   | 2026-02-27                  |

## Overview

The retry and resilience system ensures that failed upstream requests are automatically retried with exponential backoff, credential cooling, and cross-provider failover. The retry loop in `dispatch()` iterates across providers and credentials on each attempt, with failed credentials temporarily marked unavailable. See PRD (SPEC-005) for requirements.

## Backend Implementation

### Module Structure

```
crates/core/src/config.rs          -- RetryConfig definition and defaults
crates/server/src/dispatch.rs      -- dispatch() retry loop, handle_retry_error()
crates/provider/src/routing.rs     -- mark_unavailable(), pick() with cooldown filtering
```

### Key Types

#### RetryConfig

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct RetryConfig {
    pub max_retries: u32,           // default: 3
    pub max_backoff_secs: u64,      // default: 30
    pub cooldown_429_secs: u64,     // default: 60
    pub cooldown_5xx_secs: u64,     // default: 15
    pub cooldown_network_secs: u64, // default: 10
}
```

#### Cooldown Durations (defaults)

| Error Type | Config Field | Default |
|------------|-------------|---------|
| 429 Rate Limit | `cooldown_429_secs` | 60 seconds |
| 5xx Server Error | `cooldown_5xx_secs` | 15 seconds |
| Network Error | `cooldown_network_secs` | 10 seconds |

### Retry Loop in dispatch()

The `dispatch()` function implements a nested retry loop:

```
for attempt in 0..max_retries {
    for target_format in providers {       // try each provider format
        auth = router.pick(format, model, &tried)  // pick next available credential
        // translate request, apply payload rules, apply cloaking
        result = executor.execute(auth, request)
        if success -> return response
        if error -> mark_unavailable, add to tried list, continue
    }
    // exponential backoff between retry rounds
}
return last_error
```

**Key behaviors:**

1. **Provider iteration:** On each retry attempt, iterates all provider formats that support the requested model. This ensures quota exhaustion (429) on one provider automatically falls through to the next.

2. **Credential selection:** `router.pick(format, model, &tried)` returns the next available credential that:
   - Is not in cooldown (`is_available()`)
   - Supports the requested model (`supports_model()`)
   - Has not already been tried in this dispatch (`!tried.contains(&id)`)

3. **Tried list:** Each failed credential's ID is added to `tried`, preventing re-selection within the same dispatch call.

4. **Error handling:** On failure, `handle_retry_error()` is called to mark the credential unavailable and record metrics.

### Exponential Backoff

Applied between retry rounds (after iterating all providers):

```rust
if attempt + 1 < max_retries {
    let cap = std::cmp::min(1u64 << attempt, max_backoff_secs) as f64;
    let jittered = rand::random::<f64>() * cap;
    tokio::time::sleep(Duration::from_secs_f64(jittered)).await;
}
```

- **Formula:** `sleep(random() * min(2^attempt, max_backoff_secs))`
- **Full jitter:** Random value between 0 and cap, prevents thundering herd
- **Cap:** Bounded by `max_backoff_secs` (default 30s)
- **Example progression:** attempt 0 -> [0, 1s], attempt 1 -> [0, 2s], attempt 2 -> [0, 4s], ...

### handle_retry_error()

Routes error types to appropriate cooldown durations:

```rust
fn handle_retry_error(state: &AppState, auth_id: &str, error: &ProxyError, retry_cfg: &RetryConfig) {
    state.metrics.record_error();
    match error {
        ProxyError::Upstream { status, retry_after_secs, .. } => match *status {
            429 => {
                let secs = retry_after_secs.unwrap_or(retry_cfg.cooldown_429_secs);
                state.router.mark_unavailable(auth_id, Duration::from_secs(secs));
            }
            500..=599 => {
                let secs = retry_after_secs.unwrap_or(retry_cfg.cooldown_5xx_secs);
                state.router.mark_unavailable(auth_id, Duration::from_secs(secs));
            }
            _ => {}
        },
        ProxyError::Network(_) => {
            state.router.mark_unavailable(auth_id, Duration::from_secs(retry_cfg.cooldown_network_secs));
        }
        _ => {}
    }
}
```

**Retry-After header:** The `ProxyError::Upstream` variant carries an optional `retry_after_secs` field parsed from the upstream `Retry-After` header. When present, it overrides the config default cooldown duration.

### Credential Cooldown (mark_unavailable)

```rust
// In CredentialRouter (routing.rs)
pub fn mark_unavailable(&self, auth_id: &str, duration: Duration) {
    let until = Instant::now() + duration;
    // Set cooldown_until on the matching AuthRecord
    for entries in creds.values_mut() {
        for auth in entries.iter_mut() {
            if auth.id == auth_id {
                auth.cooldown_until = Some(until);
            }
        }
    }
}
```

- Cooldown is per-credential (identified by UUID `auth.id`)
- `AuthRecord::is_available()` checks `cooldown_until.map_or(true, |t| Instant::now() >= t)`
- Cooldown state is preserved across config hot-reloads: `update_from_config` matches new entries to old entries by `api_key` and copies `cooldown_until`

### Bootstrap Retries for Streaming

Streaming requests have a separate retry budget (`streaming.bootstrap_retries`, default: 1):

```rust
let bootstrap_limit = config.streaming.bootstrap_retries;
let mut bootstrap_attempts = 0u32;

// In streaming error handler:
bootstrap_attempts += 1;
if bootstrap_attempts > bootstrap_limit {
    tracing::warn!("Streaming bootstrap retry limit reached");
    return Err(e);
}
```

- Only retries before the first byte is sent to the client
- Once streaming begins (first chunk delivered), the response is committed and no retry is possible
- Default limit of 1 means: try once, retry once, then fail

### Provider Failover

The retry loop's inner iteration across `providers` (a `Vec<Format>`) provides automatic failover:

1. `resolve_providers(model)` returns all provider formats that have credentials supporting the model
2. If all credentials for provider A fail (e.g., all rate-limited), the loop moves to provider B
3. This enables scenarios like: Claude 429 -> fall through to OpenAI-compatible endpoint

## Configuration Changes

```yaml
retry:
  max-retries: 3
  max-backoff-secs: 30
  cooldown-429-secs: 60
  cooldown-5xx-secs: 15
  cooldown-network-secs: 10
streaming:
  bootstrap-retries: 1
```

## Provider Compatibility

| Provider | Supported | Notes |
|----------|-----------|-------|
| OpenAI   | Yes       | 429 and 5xx handling with Retry-After |
| Claude   | Yes       | 429 and 5xx handling with Retry-After |
| Gemini   | Yes       | 429 and 5xx handling with Retry-After |
| Compat   | Yes       | Same retry logic as all providers |

## Test Strategy

- **Unit tests:** `RetryConfig` default values verified in config tests
- **Integration tests:** Multi-credential failover with simulated 429/5xx responses
- **Manual verification:** Observe cooldown logs ("Rate limited (429), cooling down credential for ...") when upstream returns 429
