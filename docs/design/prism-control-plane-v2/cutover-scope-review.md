# Cutover Scope Review for Prism Control Plane V2

This note answers one specific question:

When Prism cuts over from the legacy dashboard to the greenfield control plane, which capabilities must exist in V2, and which should explicitly *not* be added?

This is not a page-parity checklist.
It is a control-plane scope decision.

Read this together with:

- [gateway-benchmark-analysis.md](/Users/qiufeng/work/proxy/prism/docs/design/prism-control-plane-v2/gateway-benchmark-analysis.md)
- [frontend-implementation-plan.md](/Users/qiufeng/work/proxy/prism/docs/design/prism-control-plane-v2/frontend-implementation-plan.md)
- [backend-control-plane-model.md](/Users/qiufeng/work/proxy/prism/docs/design/prism-control-plane-v2/backend-control-plane-model.md)
- [remaining-additions-roadmap.md](/Users/qiufeng/work/proxy/prism/docs/design/prism-control-plane-v2/remaining-additions-roadmap.md)

## Decision Principle

The correct cutover bar is:

- full operator capability coverage
- one shared control-plane shell
- no revival of the old page tree

Prism should not ship V2 only when every legacy page has a one-to-one replacement.
It should ship when every *necessary operator job* is covered inside:

- `Command Center`
- `Traffic Lab`
- `Provider Atlas`
- `Route Studio`
- `Change Studio`

## What V2 Must Include

These are not optional if the greenfield shell is going to replace the old dashboard.

### 1. Full managed-auth lifecycle inside `Provider Atlas`

Current V2 status:

- create, refresh, import local, and delete are present

Still required:

- connect with secret
- browser OAuth start
- OAuth callback completion
- device flow start and polling
- edit or replace existing profiles with full runtime posture

Why this is required:

- managed auth is not a side feature in Prism
- it is part of provider identity and runtime correctness
- the old UI already supports these flows in:
  - [AuthProfiles.tsx](/Users/qiufeng/work/proxy/prism/web/src/pages/AuthProfiles.tsx)
  - [AuthProfileCallback.tsx](/Users/qiufeng/work/proxy/prism/web/src/pages/AuthProfileCallback.tsx)

Decision:

- add these flows into `Provider Atlas`
- do not recreate a standalone `Auth Profiles` page in V2

### 2. Full routing authoring inside `Route Studio`

Current V2 status:

- explain, simulate, and promote-to-change are present

Still required:

- default-profile switching
- rule CRUD
- advanced policy editing
- save and reset behavior
- validation feedback on write

Why this is required:

- the old routing page is not just a viewer
- it is a real mutation surface in:
  - [Routing.tsx](/Users/qiufeng/work/proxy/prism/web/src/pages/Routing.tsx)

Decision:

- `Route Studio` must absorb full routing authoring
- simulation alone is not enough for cutover

### 3. Full auth-key policy editing inside `Change Studio`

Current V2 status:

- create, reveal, delete, and tenant posture are present

Still required:

- edit existing keys
- allowed models
- allowed credentials
- rate limits
- budgets
- expiry
- copy or reveal ergonomics suitable for real operators

Why this is required:

- auth keys are Prism's primary gateway access boundary
- the old UI already exposes these controls in:
  - [AuthKeys.tsx](/Users/qiufeng/work/proxy/prism/web/src/pages/AuthKeys.tsx)

Decision:

- `Change Studio` must include a full access-policy editor
- a minimal create/delete sheet is not enough

### 4. Searchable protocol and model truth inside `Provider Atlas`

Current V2 status:

- protocol and model truth are visible

Still required:

- searchable protocol matrix
- searchable model inventory
- better provider-to-model and provider-to-surface drill-down

Why this is required:

- operators need to answer "which provider really supports this surface and model"
- the old UI has dedicated truth surfaces in:
  - [Protocols.tsx](/Users/qiufeng/work/proxy/prism/web/src/pages/Protocols.tsx)
  - [ModelsCapabilities.tsx](/Users/qiufeng/work/proxy/prism/web/src/pages/ModelsCapabilities.tsx)

Decision:

- keep the capability, not the old pages
- this belongs inside `Provider Atlas`

### 5. Shareable debug semantics inside `Traffic Lab`

Current V2 status:

- request selection, replay, explain, request detail, and local saved lens are present

Still required:

- URL-backed filter state
- request-id lookup and deep-link behavior
- compare mode
- explicit live connection state
- filter-respecting live updates

Why this is required:

- debugging quality is one of the main reasons for the redesign
- the legacy stack already has some of this behavior in:
  - [RequestLogs.tsx](/Users/qiufeng/work/proxy/prism/web/src/pages/RequestLogs.tsx)
  - [Replay.tsx](/Users/qiufeng/work/proxy/prism/web/src/pages/Replay.tsx)

Decision:

- `Traffic Lab` should be the full request-session debugger
- not just a prettier read-only request list

### 6. Diagnostic system posture inside `Command Center`

Current V2 status:

- top-level posture, signals, and system watch already exist

Still required:

- searchable application-log access
- clearer runtime repair actions
- connection freshness and lag state
- stronger operator affordances for runtime diagnosis

Why this is required:

- not every control-plane problem starts from a single request
- some start from gateway health, reload failure, or system drift
- the old UI surfaces these through:
  - [System.tsx](/Users/qiufeng/work/proxy/prism/web/src/pages/System.tsx)
  - [Logs.tsx](/Users/qiufeng/work/proxy/prism/web/src/pages/Logs.tsx)

Decision:

- keep system and log functionality
- integrate it into `Command Center`
- do not reintroduce separate `System` and `Logs` pages

### 7. Strong failure-path and session-path handling across the shell

Still required:

- expired dashboard session handling
- write conflict handling
- disconnected source state
- empty and degraded state coverage
- explicit long-running action feedback

Why this is required:

- the greenfield shell should not only succeed on the happy path
- operator confidence depends on failure semantics being clear

Decision:

- treat these as first-class product behavior, not polish

## What Should Not Be Added

These capabilities are not necessary as separate product surfaces, even though the old dashboard or the broader ecosystem may suggest them.

### 1. Do not rebuild the legacy page tree in V2

Do not reintroduce separate pages for:

- `RequestLogs`
- `Replay`
- `Providers`
- `AuthProfiles`
- `AuthKeys`
- `Tenants`
- `Protocols`
- `ModelsCapabilities`
- `System`
- `Logs`
- `Config`
- `Routing`

The capability should survive.
The page boundary should not.

### 2. Do not add a heavyweight standalone investigation product right now

Prism does need:

- saved lenses
- request compare
- signal context
- evidence grouping

Prism does **not** need, as a separate V2 product area:

- a full incident-management system
- threaded investigation comments
- a separate investigation home with its own object universe

Decision:

- keep investigation affordances lightweight and embedded in `Traffic Lab` and `Command Center`
- do not build a separate incident platform unless real operator workflow proves the need

### 3. Do not add vendor-specific external-observability pages

Prism should be able to model external sources such as `SLS`, `OTLP`, `Tempo`, `ClickHouse`, and `Prometheus`.

It does **not** need:

- one separate page per external vendor
- one separate navigation branch per data source

Decision:

- keep the shell source-aware
- keep integrations typed
- do not let integrations fracture the main workspace model

### 4. Do not add duplicate metric walls

The old style of:

- overview page
- metrics page
- health page
- logs page

all echoing the same runtime posture is exactly what the redesign should avoid.

Decision:

- keep one posture-first `Command Center`
- route drill-down into the relevant workspace

## Final Scope Decision

If Prism wants one clean cutover instead of endless compatibility drag, the rule should be:

- include every operator-critical capability listed in `What V2 Must Include`
- explicitly reject the separate surfaces listed in `What Should Not Be Added`

This means the correct target is not:

- "page parity"

The correct target is:

- "operator parity plus control-plane improvements, inside one shared shell"

## Concrete Build Implication

The next implementation passes should be judged against this checklist:

- can an operator fully manage provider identity and managed auth without leaving `Provider Atlas`
- can an operator fully author and validate routing without leaving `Route Studio`
- can an operator fully manage gateway access policy without leaving `Change Studio`
- can an operator fully debug a request session from `Traffic Lab`
- can an operator diagnose runtime posture without falling back to legacy `System` or `Logs`

If the answer to any of those is still "no", the cutover bar has not been met yet.
