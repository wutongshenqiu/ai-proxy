## Task

@claude

Bring `/v1/responses` and `/v1/messages/count_tokens` back under the same auth, transport, and streaming invariants as the main dispatch path.

## Context

**Priority:** P0-critical
**Crate(s):** `crates/server/`, `crates/provider/`
**Related Spec:** `SPEC-044` follow-up
**Related Issue:** closed follow-up to `#153`

Current behavior:

- these handlers bypass dispatch-level model ACL checks.
- they build direct `reqwest` transport requests in handlers.
- `wire_api=responses` streaming is currently simulated from a completed non-stream response.

## Scope

- Reuse the same auth / model ACL enforcement used by main dispatch.
- Move Responses transport handling into the provider layer.
- Support real upstream Responses streaming instead of pseudo-streaming chunks.
- Add regression tests for ACL enforcement and real stream behavior.

## Acceptance Criteria

- [ ] `/v1/responses` enforces model ACLs the same way as `/v1/chat/completions`.
- [ ] `/v1/messages/count_tokens` enforces model ACLs before upstream execution.
- [ ] Handler-level transport duplication is removed or reduced to provider dispatch glue.
- [ ] `wire_api=responses` streaming no longer buffers the full response before emitting chunks.
- [ ] Tests cover ACL denial, successful passthrough, and streaming behavior.
- [ ] All tests pass (`cargo test --workspace`).
- [ ] No lint warnings (`cargo clippy --workspace --tests -- -D warnings`).

## Notes

- Preserve the public API surface introduced by `SPEC-044`.
- This task is allowed to refactor executor capabilities if needed.
