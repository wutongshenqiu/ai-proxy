# Runtime Hardening Backlog

This directory contains GitHub issue bodies and Claude Code implementation plans derived from the 2026-03 repository audit.

Epic:

- `#168` `epic: address runtime hardening backlog from the 2026-03 audit`

GitHub issues:

- `#158` `fix: fail fast on invalid config and make runtime reload atomic`
- `#165` `fix: harden dashboard exposure boundaries and trusted IP handling`
- `#166` `fix: preserve env:// and file:// secret references during dashboard config writes`
- `#161` `fix: re-apply auth and transport invariants to compatibility endpoints`
- `#162` `fix: scope response cache keys by tenant, credential, and transform context`
- `#163` `fix: make request logging metadata-only by default and stop dashboard body fan-out`
- `#167` `perf: reuse long-lived reqwest::Client instances across runtime traffic`
- `#164` `fix: unify dashboard session state and WebSocket token rotation`
- `#160` `fix: finish config workspace apply semantics or relabel it as validator-only`
- `#159` `fix: preserve Request Logs URL state and visible error handling`

Local issue bodies:

- `01-config-lifecycle.md`
- `02-dashboard-boundary.md`
- `03-secret-roundtrip.md`
- `04-compat-endpoints.md`
- `05-cache-isolation.md`
- `06-log-exposure.md`
- `07-http-client-reuse.md`
- `08-dashboard-session.md`
- `09-config-workspace-followup.md`
- `10-request-logs-ux.md`
- `epic.md`

This backlog is separate from product-gap epic `#157`; `#157` tracks CLIProxyAPI product parity, while this directory tracks runtime hardening and implementation follow-ups discovered during the same audit.
