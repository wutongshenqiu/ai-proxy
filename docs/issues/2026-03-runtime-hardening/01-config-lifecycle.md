## Task

@claude

Harden configuration loading so invalid config fails closed, and make every config mutation path rebuild and swap runtime state atomically.

## Context

**Priority:** P0-critical
**Crate(s):** `crates/core/`, `crates/server/`
**Related Spec:** `SPEC-042` follow-up

Current behavior is inconsistent:

- `crates/server/src/app.rs:251` falls back to `Config::default()` when initial load fails.
- `crates/server/src/auth.rs:14-16` skips auth when `auth_keys` is empty.
- watcher / SIGHUP reload updates router, rate limiter, and cost calculator, but dashboard reload and provider CRUD do not refresh the same derived state.

## Scope

- Introduce a validated config pipeline that separates raw parse, semantic validation, and runtime snapshot construction.
- Replace ad hoc reload logic with one shared runtime reloader.
- Ensure startup, file watcher reload, SIGHUP reload, dashboard reload, and dashboard config writes all use the same atomic apply path.
- Fail closed on invalid config or failed secret resolution.

## Acceptance Criteria

- [ ] Startup no longer falls back to `Config::default()` when config load fails.
- [ ] All config mutation paths rebuild the same derived runtime state before swapping.
- [ ] Secret resolution failures are surfaced as reload/startup errors instead of being silently ignored.
- [ ] Regression tests cover startup failure, SIGHUP reload, dashboard reload, and provider CRUD reload semantics.
- [ ] All tests pass (`cargo test --workspace`).
- [ ] No lint warnings (`cargo clippy --workspace --tests -- -D warnings`).

## Notes

- Keep the config format stable; this is a runtime correctness fix, not a product change.
- If the refactor grows large, land it as coordinator plumbing first, then migrate each entry point.
