# Technical Design: Dashboard Auth & Security Hardening

| Field     | Value          |
|-----------|----------------|
| Spec ID   | SPEC-046       |
| Title     | Dashboard Auth & Security Hardening |
| Author    | Claude          |
| Status    | Active         |
| Created   | 2026-03-13     |
| Updated   | 2026-03-13     |

## Config Schema

```yaml
dashboard:
  max-login-attempts: 5        # 0 = disabled
  login-lockout-secs: 300      # 5 minute lockout window
  localhost-only: false         # restrict to 127.0.0.1/::1
```

## Implementation

### LoginRateLimiter (`handler/dashboard/auth.rs`)
- In-memory `HashMap<String, Vec<Instant>>` tracking per-IP attempt timestamps
- Sliding window: prune attempts older than `login_lockout_secs`
- On failed login: record attempt, return 429 if threshold reached
- On successful login: clear attempt history

### Localhost-only (`middleware/dashboard_auth.rs` + login handler)
- Check `RequestContext.client_ip` against `127.0.0.1`, `::1`, `localhost`
- Applied in both the JWT auth middleware and the login handler
- Returns 403 Forbidden for non-local IPs

## Task Breakdown

- [x] Add config fields to DashboardConfig
- [x] Implement LoginRateLimiter
- [x] Apply rate limiting in login handler
- [x] Add localhost-only check
- [x] Add LoginRateLimiter to AppState
- [x] Tests
