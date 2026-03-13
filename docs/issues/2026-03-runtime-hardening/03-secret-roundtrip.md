## Task

@claude

Preserve `env://` and `file://` secret references during dashboard-driven config writes instead of resolving and serializing them back as plaintext.

## Context

**Priority:** P0-critical
**Crate(s):** `crates/core/`, `crates/server/`

Current behavior:

- `Config::from_yaml()` sanitizes and resolves secrets.
- dashboard config writes read raw YAML into `Config`, mutate the resolved struct, and serialize it back.
- this turns secret references into plaintext in `config.yaml`.

## Scope

- Stop round-tripping dashboard writes through the fully resolved runtime config representation.
- Preserve raw YAML / AST values for secret-backed fields during CRUD updates.
- Surface secret resolution errors explicitly instead of silently ignoring them.
- Add regression coverage for provider, auth key, and dashboard secret fields.

## Acceptance Criteria

- [ ] Dashboard CRUD operations no longer replace secret references with plaintext.
- [ ] Existing `env://` and `file://` references remain unchanged after provider and auth-key edits.
- [ ] Secret resolution errors return actionable API errors.
- [ ] Tests cover provider API keys, auth keys, and dashboard JWT/password secret references.
- [ ] All tests pass (`cargo test --workspace`).
- [ ] No lint warnings (`cargo clippy --workspace --tests -- -D warnings`).

## Notes

- Treat the raw config file as the source of truth for persistence.
- Keep runtime config resolution unchanged for in-memory execution.
