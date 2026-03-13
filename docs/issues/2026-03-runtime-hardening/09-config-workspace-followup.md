## Task

@claude

Finish the Config workspace so the page either supports a real edit/apply workflow or is explicitly relabeled as a validator-only tool with no misleading “editor” semantics.

## Context

**Priority:** P1-high
**Crate(s):** `web/`, `crates/server/`
**Related Spec:** `SPEC-043` follow-up
**Related Issue:** closed follow-up to `#149`

Current behavior:

- the page has editable YAML state and validation.
- `Reload Config` only reloads from disk; it does not persist editor content.
- the current UX can be mistaken for “validate and apply”.

## Scope

- Decide whether the page should support save/apply or become read-only plus validator.
- Align backend endpoints and frontend UX with that choice.
- Add dirty-state messaging, confirmation flow, and explicit result feedback.
- Update `SPEC-043` if implementation scope changes.

## Acceptance Criteria

- [ ] The page no longer suggests that editor changes were applied when they were not.
- [ ] If apply is supported, the flow is validate -> persist -> reload -> confirm result.
- [ ] If apply is not supported, the UI is clearly labeled and constrained accordingly.
- [ ] Tests cover the chosen flow and primary failure states.
- [ ] Frontend type-check passes (`npx tsc --noEmit` in `web/`).
- [ ] Relevant tests pass.

## Notes

- Prefer an honest product workflow over a half-editor.
- Coordinate with the secret-preservation work if raw YAML persistence is added.
