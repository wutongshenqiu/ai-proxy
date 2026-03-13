## Task

@claude

Coordinate and land the runtime hardening backlog identified during the 2026-03 repository audit. This epic tracks correctness, security, performance, and implementation follow-ups that remain after the product-gap backlog in `#157`.

## Context

**Area:** runtime correctness, security hardening, dashboard implementation follow-ups, transport performance
**Related Epic:** `#157` covers product parity and dashboard feature gaps; this epic covers technical hardening and follow-up implementation work.

## Tracking

### P0

- [ ] #165 Harden dashboard exposure boundaries and trusted IP handling
- [ ] #158 Fail fast on invalid config and make runtime reload atomic
- [ ] #166 Preserve `env://` and `file://` secret references during dashboard config writes
- [ ] #161 Re-apply auth and transport invariants to compatibility endpoints

### P1

- [ ] #163 Make request logging metadata-only by default and stop dashboard body fan-out
- [ ] #167 Reuse long-lived `reqwest::Client` instances across runtime traffic
- [ ] #162 Scope response cache keys by tenant, credential, and transform context
- [ ] #164 Unify dashboard session state and WebSocket token rotation
- [ ] #160 Finish config workspace apply semantics or relabel it as validator-only

### P2

- [ ] #159 Preserve Request Logs URL state and visible error handling

## Suggested Order

1. `#165`
2. `#158`
3. `#166`
4. `#161`
5. `#163`
6. `#167`
7. `#162`
8. `#164`
9. `#160`
10. `#159`

## Notes

- Each child issue already contains a Claude Code-oriented implementation plan and acceptance criteria.
- Security and correctness issues should land before frontend polish tasks.
