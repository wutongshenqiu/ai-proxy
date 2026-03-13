## Task

@claude

Stop rebuilding `reqwest::Client` per request. Reuse long-lived clients across provider traffic, compatibility endpoints, and dashboard upstream probes.

## Context

**Priority:** P1-high
**Crate(s):** `crates/core/`, `crates/provider/`, `crates/server/`
**Related Spec:** `SPEC-042` follow-up

Current behavior:

- provider requests construct a new `reqwest::Client` per request.
- compatibility endpoints build their own direct clients.
- dashboard `fetch-models` creates yet another independent client.

## Scope

- Introduce a shared client factory / pool keyed by transport-relevant settings.
- Put client reuse behind runtime snapshot / reload boundaries.
- Ensure connect/request timeout settings are applied consistently.
- Add focused tests or benchmarks to verify the refactor.

## Acceptance Criteria

- [ ] Data-plane provider requests reuse long-lived `reqwest::Client` instances.
- [ ] Compatibility endpoints and dashboard probes use the same transport creation path.
- [ ] Runtime config changes that affect transport rebuild the relevant client pool safely.
- [ ] Timeouts and proxy settings remain correct after the refactor.
- [ ] All tests pass (`cargo test --workspace`).
- [ ] No lint warnings (`cargo clippy --workspace --tests -- -D warnings`).

## Notes

- Keep the public API unchanged.
- If the full pool abstraction is too large for one PR, land an internal shared transport layer first.
