## Task

@claude

Fix Request Logs URL state and failure handling so investigations are shareable, reproducible, and clear when data failed to load versus when there is simply no data.

## Context

**Priority:** P2-medium
**Crate(s):** `web/`

Current behavior:

- URL-based filters can trigger duplicate initial fetches.
- opening a log drawer overwrites all existing search params with only `id`.
- closing the drawer does not clean the URL.
- page/store failures mostly go to `console.error` with no visible error state.

## Scope

- Make filters, page, and selected log id share one URL-state model.
- Avoid duplicate initial requests when URL filters are present.
- Add visible page-level error and retry states.
- Preserve current live-log behavior.

## Acceptance Criteria

- [ ] Opening and closing the log drawer preserves current filters and paging state.
- [ ] URL-linked investigations can be shared and reopened without losing context.
- [ ] Initial load does not perform duplicate fetches for the same state.
- [ ] Error states are visible and distinguishable from empty-result states.
- [ ] Frontend type-check passes (`npx tsc --noEmit` in `web/`).
- [ ] Relevant tests pass.

## Notes

- Keep the solution simple; this is primarily a state-management cleanup and UX correctness fix.
