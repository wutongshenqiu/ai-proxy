## Task

@claude

Reduce default log exposure by switching request logging to metadata-only defaults and preventing full request / response bodies from being broadcast to dashboard clients by default.

## Context

**Priority:** P1-high
**Crate(s):** `crates/core/`, `crates/server/`, `web/`

Current behavior:

- `LogStoreConfig::default()` uses `detail_level = Full`.
- request, upstream-request, and response bodies are captured in normal execution paths.
- dashboard WebSocket pushes full `RequestRecord` payloads to connected clients.

## Scope

- Change the default log detail level to metadata-only.
- Make body capture explicit opt-in.
- Reduce dashboard WebSocket payloads to safe metadata by default.
- Add tests for default behavior, opt-in behavior, and redaction boundaries.

## Acceptance Criteria

- [ ] Fresh/default config no longer captures request or response bodies.
- [ ] Body capture still works when explicitly enabled.
- [ ] Dashboard live updates do not expose request/response bodies by default.
- [ ] Tests cover standard, full, and metadata-only behavior.
- [ ] All tests pass (`cargo test --workspace`).
- [ ] No lint warnings (`cargo clippy --workspace --tests -- -D warnings`).

## Notes

- Be careful not to break existing operators who intentionally opted into body capture.
- Document the new default in config examples if needed.
