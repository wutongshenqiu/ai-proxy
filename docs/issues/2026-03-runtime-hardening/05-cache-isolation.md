## Task

@claude

Strengthen response-cache isolation so cache entries are scoped by tenant, credential, and request-transform context rather than just model plus a few body fields.

## Context

**Priority:** P1-high
**Crate(s):** `crates/core/`, `crates/server/`

Current behavior:

- cache keys are built from model name and a small subset of request fields.
- they do not include tenant, API key, credential, source format, or transformed request context.
- cloaking, payload mutation, and model rewrite can change upstream semantics without changing the cache key.

## Scope

- Expand cache key inputs to include tenant / auth / credential / source-format context and final routed model.
- Decide when cache should be disabled entirely for cloaked or otherwise transformed requests.
- Add regression tests for cross-tenant and cross-credential isolation.

## Acceptance Criteria

- [ ] Cache keys prevent cross-tenant and cross-credential response reuse.
- [ ] Requests with materially different transformed upstream payloads do not collide in cache.
- [ ] Cloaked or other unsafe-to-cache requests are either isolated correctly or skipped entirely.
- [ ] Tests prove isolation across tenant, API key, and credential selection.
- [ ] All tests pass (`cargo test --workspace`).
- [ ] No lint warnings (`cargo clippy --workspace --tests -- -D warnings`).

## Notes

- Favor correctness over cache hit rate.
- If canonicalization gets expensive, document the tradeoff and benchmark it.
