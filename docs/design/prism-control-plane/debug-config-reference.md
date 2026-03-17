# Debug & Config Reference

This note is the practical companion to the standalone prototype. It focuses on one question:

How should Prism present its existing debugging and configuration power without keeping the current page-by-page UI baggage?

## Why This Matters

For Prism, debugging and configuration are not secondary admin tasks.

- Debugging is how operators explain a failed or degraded request.
- Configuration is how operators change routing, providers, auth posture, and rollout behavior.
- The two are coupled: most incidents end with a config change, and most config changes need immediate runtime observation.

That means the UI should treat them as one operator loop:

`Signal -> Inspect -> Explain -> Change -> Publish -> Observe`

## Current Capability Map

Prism already has the backend and UI primitives needed for a strong control plane.

| Domain | Current UI Surface | Backend Truth | What already exists | Why the current UX still feels fragmented |
|--------|--------------------|---------------|---------------------|------------------------------------------|
| Request debugging | `Request Logs` + `LogDrawer` | `/api/dashboard/logs`, `/api/dashboard/logs/{id}` | filters, live mode, request detail, retry attempts, request/upstream/response bodies, stream preview | request detail is a drawer, so deep debugging still feels like an overlay rather than a workspace |
| Route explanation | `Replay` | `/api/dashboard/routing/preview`, `/api/dashboard/routing/explain` | selected route, scoring, alternates, rejections, model resolution | explain is isolated from live request inspection and provider context |
| Provider diagnostics | `Providers` | provider CRUD + `/presentation-preview` + `/health` | health probe, model discovery, presentation preview, auth-profile awareness | provider diagnostics live in CRUD-heavy modal flow instead of a runtime context |
| Managed auth diagnostics | `Auth Profiles` | runtime-backed connect / refresh / import / OAuth / device flow endpoints | connection state, refresh, browser OAuth, device flow, local import, runtime status | auth operations are operationally important but visually separated from provider and request debugging |
| Config editing | `Config & Changes` | `/config/current`, `/config/raw`, `/config/validate`, `/config/apply`, `/config/reload` | runtime snapshot, raw YAML, validate, optimistic concurrency, apply | config work is split between raw YAML and domain pages with no publish/observe loop |
| Routing config | `Routing` + `Replay` | `/api/dashboard/routing`, `/api/dashboard/routing/explain` | update config, validate references, preview decisions | routing is edited in one place, reasoned about in another, and observed in a third |

## Concrete Source Map

These are the files that matter most when mapping the redesign to real code.

### Frontend

- [RequestLogs.tsx](/Users/qiufeng/work/proxy/prism/web/src/pages/RequestLogs.tsx)
- [LogDrawer.tsx](/Users/qiufeng/work/proxy/prism/web/src/components/LogDrawer.tsx)
- [Replay.tsx](/Users/qiufeng/work/proxy/prism/web/src/pages/Replay.tsx)
- [Config.tsx](/Users/qiufeng/work/proxy/prism/web/src/pages/Config.tsx)
- [Providers.tsx](/Users/qiufeng/work/proxy/prism/web/src/pages/Providers.tsx)
- [AuthProfiles.tsx](/Users/qiufeng/work/proxy/prism/web/src/pages/AuthProfiles.tsx)
- [api.ts](/Users/qiufeng/work/proxy/prism/web/src/services/api.ts)

### Backend

- [routing.rs](/Users/qiufeng/work/proxy/prism/crates/server/src/handler/dashboard/routing.rs)
- [config_ops.rs](/Users/qiufeng/work/proxy/prism/crates/server/src/handler/dashboard/config_ops.rs)
- [providers.rs](/Users/qiufeng/work/proxy/prism/crates/server/src/handler/dashboard/providers.rs)
- [auth_profiles.rs](/Users/qiufeng/work/proxy/prism/crates/server/src/handler/dashboard/auth_profiles.rs)
- [dashboard_tests.rs](/Users/qiufeng/work/proxy/prism/crates/server/tests/dashboard_tests.rs)

## What V2 Should Absorb

### 1. Traffic Lab should absorb request debugging and route reasoning

The current product already has the pieces for a real request session debugger:

- request filters and live state
- request drill-down
- retry timeline
- upstream transform visibility
- route explanation
- provider / credential / tenant context

So the new debugging model should be:

`Traffic stream -> Request session -> Explain route -> Compare baseline -> Jump to provider/auth`

Recommended session layout:

- top pinned identity bar: request id, status, latency, tenant, provider, credential, model
- center stage timeline: ingress, transform, route selection, upstream attempts, final response
- left or top tabs: `Session`, `Explain`, `Transform`, `Retry`, `Replay`
- right inspector: provider health, auth profile, recent failures, route rule, config provenance

### 2. Change Studio should absorb raw config, domain editors, and publish semantics

The current code already supports validation, conflict detection, raw config access, and domain-aware editors.

So the config model should not stay as:

`Edit YAML -> Save`

It should become:

`Edit -> Validate -> Review Impact -> Publish -> Observe -> Rollback`

Recommended layers:

- structured domain editors first: Provider, Route, Auth Profile
- raw YAML second: always available, but not the default entry
- explicit conflict state using `config_version`
- preflight summary: validation errors, secret resolution, affected providers, affected routes
- post-publish watch window: recent request errors, provider health drift, rollback trigger

### 3. Provider and Auth should stop feeling like separate admin silos

Today they are operationally linked but visually split.

V2 should make these links explicit:

- request session -> provider health
- request session -> credential or auth profile
- provider detail -> managed auth posture
- auth profile -> affected providers
- change review -> impacted providers and auth identities

## Best-Practice Patterns Worth Borrowing

These are based on official product/docs references already gathered for this redesign.

| Source | Useful pattern | Prism implication |
|--------|----------------|-------------------|
| Apigee Trace / Debug | request pipeline visibility as a step-by-step trace | Prism should show ingress, transform, route decision, upstream attempt, retry, and failure cause in one timeline |
| Kong analytics / AI gateway positioning | analytics as the front door to investigation | `Command Center` and `Traffic Lab` should be the first entry for incident triage, not static config pages |
| Tyk analytics | strong slicing by key, API, time, geography | Prism should keep filters global and shareable instead of page-local |
| Gravitee overview dashboard | homepage focused on posture and drill-down | `Command Center` should emphasize health, anomalies, pending changes, and action queues |
| Figma variables + Dev Mode | token discipline and inspectable handoff | Prism should keep a stable shell, token set, and component taxonomy before React implementation |
| CLIProxyAPI web UI | runtime-oriented operational shell | Prism should remain operational and dense, not stylized into a generic SaaS admin panel |

## Proposed V3 Prototype Priorities

If the prototype is deepened again, the next step should not be more top-level pages. It should be richer operator flows.

### A. Request Session Debugger

Add a focused stateful request debugger with:

- stage timeline with timestamps
- failed attempt vs successful attempt compare
- upstream request diff
- explain result embedded in the same session
- “jump to provider” and “jump to auth profile” actions
- “replay with current route config” and “compare with baseline” actions

### B. Structured Change Workbench

Add structured config editing for:

- Provider
- Route profile / rule
- Auth profile

Each editor should show:

- raw diff
- semantic diff
- validation state
- impact summary
- publish scope

### C. Publish Watch Window

After publish, do not dump the operator back to a generic toast.

Show a watch window with:

- change id / config version
- canary scope
- live request error delta
- provider health delta
- rollback CTA

## Anti-Patterns To Avoid

- Do not keep debugging split between `Logs` and `Replay` forever.
- Do not make raw YAML the default surface for common changes.
- Do not hide conflict or reload semantics behind a simple success toast.
- Do not force operators into blocking modals for long-running flows.
- Do not translate technical ids, model names, config keys, or raw upstream payloads.
- Do not let every workspace invent its own filter bar and local context rules.

## Internationalization Rules

For this control plane, the right split is:

Translate:

- navigation
- shell actions
- stage labels
- validation summaries
- change workflow language

Do not translate:

- request ids
- provider names
- auth profile ids
- model ids
- route profile ids
- raw payload keys
- raw upstream error bodies

Layout rules:

- assume 30% to 50% label growth
- do not depend on fixed button widths
- use icon + text, not icon-only meaning
- keep technical entities in monospaced blocks

## Design Decision Summary

If we only keep three decisions from this round, they should be:

1. `Traffic Lab` becomes the main debugging home, not just a logs table.
2. `Change Studio` becomes a staged publish workspace, not just a YAML editor.
3. Provider, routing, auth, and config must cross-link as one control plane instead of isolated admin pages.
