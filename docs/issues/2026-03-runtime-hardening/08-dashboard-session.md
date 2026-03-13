## Task

@claude

Unify dashboard session state across Axios, Zustand, and WebSocket so token refresh and logout are consistent and realtime connections rotate credentials correctly.

## Context

**Priority:** P1-high
**Crate(s):** `web/`, `crates/server/`

Current behavior:

- Axios interceptors read and refresh tokens via `localStorage`.
- Zustand auth state is not the sole source of truth.
- the WebSocket singleton pins the initial token into the URL and does not rebuild on token changes.

## Scope

- Introduce one session manager abstraction for token load, refresh, persistence, and logout.
- Make WebSocket reconnect when auth state changes.
- Remove stale-token behavior between HTTP and WS.
- If practical, prepare the frontend for cookie or short-lived WS ticket auth.

## Acceptance Criteria

- [ ] HTTP and WebSocket always use the same current session state.
- [ ] Token refresh updates the frontend auth store and reconnects WS as needed.
- [ ] Logout tears down WS and clears all auth state consistently.
- [ ] Frontend tests cover refresh, token rotation, and logout behavior.
- [ ] Frontend type-check passes (`npx tsc --noEmit` in `web/`).
- [ ] Relevant tests pass.

## Notes

- Do not regress current protected-route behavior.
- Keep the change scoped to session consistency; broader auth model redesign can happen later.
