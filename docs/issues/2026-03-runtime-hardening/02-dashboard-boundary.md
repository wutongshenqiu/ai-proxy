## Task

@claude

Close the dashboard security boundary: trust client IP only from configured proxies, gate dashboard route registration by `dashboard.enabled`, and make dashboard auth fail closed everywhere including WebSocket.

## Context

**Priority:** P0-critical
**Crate(s):** `crates/server/`, `crates/core/`
**Related Spec:** `SPEC-046` follow-up
**Related Issue:** closed follow-up to `#156`

Current behavior:

- `crates/server/src/middleware/request_context.rs` trusts `x-forwarded-for` and `x-real-ip` unconditionally.
- `crates/server/src/lib.rs` always registers dashboard HTTP and WS routes.
- `crates/server/src/handler/dashboard/websocket.rs` skips auth entirely if JWT secret is missing and does not enforce `localhost_only`.
- dashboard routes do not have a body size limit.

## Scope

- Add trusted proxy / trusted forwarded headers configuration and default to socket peer address.
- Register dashboard HTTP and WS routes only when `dashboard.enabled = true`.
- Reuse one shared guard for login, JWT-protected routes, and `/ws/dashboard`.
- Fail closed when JWT secret is missing.
- Add request body limits for dashboard endpoints.

## Acceptance Criteria

- [ ] Forged `X-Forwarded-For` and `X-Real-IP` headers do not bypass `localhost_only` or login rate limiting by default.
- [ ] Dashboard routes are absent when disabled.
- [ ] WebSocket enforces the same enabled / localhost / JWT rules as HTTP routes.
- [ ] Dashboard write endpoints reject oversized bodies.
- [ ] Regression tests cover trusted IP extraction, disabled dashboard access, and WS fail-closed behavior.
- [ ] All tests pass (`cargo test --workspace`).
- [ ] No lint warnings (`cargo clippy --workspace --tests -- -D warnings`).

## Notes

- This is a hardening fix, not a UI change.
- Keep backward compatibility by making trusted proxy behavior explicit in config rather than inferring from any forwarded header.
