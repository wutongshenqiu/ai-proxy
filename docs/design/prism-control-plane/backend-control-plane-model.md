# Backend Control-Plane Model for Prism

This note defines the backend model that best fits the approved control-plane design.

It does not require Prism to throw away its current handlers.
It does require the implementation to stop treating the current page routes as the final product model.

Read this together with:

- [gateway-benchmark-analysis.md](/Users/qiufeng/work/proxy/prism/docs/design/prism-control-plane/gateway-benchmark-analysis.md)
- [north-star-model.md](/Users/qiufeng/work/proxy/prism/docs/design/prism-control-plane/north-star-model.md)
- [extensibility-model.md](/Users/qiufeng/work/proxy/prism/docs/design/prism-control-plane/extensibility-model.md)
- [frontend-implementation-plan.md](/Users/qiufeng/work/proxy/prism/docs/design/prism-control-plane/frontend-implementation-plan.md)

## Core Recommendation

Keep the current truthful primitive handlers.
Add a small set of higher-level control-plane objects on top of them.

This should support the canonical control-plane frontend.
The backend does not need to preserve earlier page boundaries just because older handlers existed.

The current backend is already good at:

- runtime truth
- configuration mutation
- routing introspection
- provider capability truth
- managed auth lifecycle
- key and tenant management

What it does not yet express cleanly is the north-star product model.

## Current Primitive Baseline

These primitives should be preserved and reused.

### Traffic and log truth

- log query, record fetch, stats, and filter options in `crates/server/src/handler/dashboard/logs.rs`

### Routing truth

- routing get and update
- route preview
- route explain

These live in `crates/server/src/handler/dashboard/routing.rs`.

### Configuration truth

- validate
- reload
- apply with optimistic version conflict
- raw YAML
- current sanitized config

These live in `crates/server/src/handler/dashboard/config_ops.rs`.

### Provider and protocol truth

- protocol matrix
- provider capabilities

These live in `crates/server/src/handler/dashboard/control_plane.rs`.

### Managed auth truth

- auth-profile list and runtime
- create, replace, delete
- browser OAuth start and complete
- device start and poll
- import local
- connect and refresh

These live in `crates/server/src/handler/dashboard/auth_profiles.rs`.

### Inventory CRUD primitives

- providers CRUD in `crates/server/src/handler/dashboard/providers.rs`
- auth-key CRUD in `crates/server/src/handler/dashboard/auth_keys.rs`
- tenant summaries and metrics in `crates/server/src/handler/dashboard/tenant.rs`

## North-Star Product Objects

The control plane should eventually speak in these objects:

1. `RequestSession`
2. `Signal`
3. `Investigation`
4. `Change`
5. `ConfigRecord`
6. `ProviderIdentity`
7. `RouteDraft`
8. `DataSource`
9. `WatchWindow`

These objects do not replace the current primitives.
They compose them and provide the read/write model the new frontend actually needs.

## Recommended Object Shapes

### Request session

```ts
interface RequestSession {
  id: string;
  startedAt: string;
  tenantId?: string;
  apiKeyId?: string;
  requestedModel: string;
  outcome: 'success' | 'retry' | 'fallback' | 'failed';
  routeDecisionId?: string;
  providerAttempts: ProviderAttempt[];
  source: 'prism-runtime' | 'hybrid' | 'external';
  linkedChangeIds: string[];
  linkedSignalIds: string[];
}
```

### Signal

```ts
interface Signal {
  id: string;
  kind: 'latency' | 'error-burst' | 'fallback-rate' | 'auth-failure' | 'source-lag';
  severity: 'critical' | 'degraded' | 'watch' | 'info';
  status: 'open' | 'acknowledged' | 'watching' | 'resolved';
  owner?: string;
  summary: string;
  evidence: string[];
  linkedInvestigationId?: string;
}
```

### Investigation

```ts
interface Investigation {
  id: string;
  title: string;
  status: 'open' | 'watching' | 'resolved';
  owner?: string;
  summary?: string;
  pinnedEvidence: InvestigationEvidence[];
  comparisonMode?: 'baseline' | 'time-range' | 'pre-post-change';
}
```

### Change

```ts
interface Change {
  id: string;
  family: 'provider' | 'auth-profile' | 'auth-key' | 'tenant-policy' | 'route' | 'data-source' | 'alert-policy';
  recordId: string;
  status: 'draft' | 'validated' | 'reviewed' | 'publishing' | 'watching' | 'rolled-back' | 'completed';
  configVersion?: string;
  actor: string;
  createdAt: string;
  watchWindow?: WatchWindow;
  linkedSignalIds: string[];
}
```

### Data source

```ts
interface DataSource {
  id: string;
  kind: 'prism-runtime' | 'sls' | 'otlp' | 'tempo' | 'clickhouse' | 'prometheus' | 'warehouse';
  health: 'connected' | 'degraded' | 'disconnected';
  capabilities: Array<'logs' | 'metrics' | 'traces' | 'joins' | 'deep-links'>;
  freshness: {
    lagMs?: number;
    retentionHours?: number;
  };
}
```

## New Aggregated APIs Prism Should Add

These APIs are more important than adding more page-local routes.

### Traffic Lab

- `GET /api/dashboard/traffic/sessions`
- `GET /api/dashboard/traffic/sessions/{id}`
- `POST /api/dashboard/traffic/sessions/compare`
- `POST /api/dashboard/traffic/sessions/{id}/replay`

These should compose:

- request log query
- request log detail
- route explain
- provider capability context
- linked source metadata

### Investigations and signals

- `GET /api/dashboard/signals`
- `POST /api/dashboard/signals/{id}/ack`
- `POST /api/dashboard/signals/{id}/promote`
- `GET /api/dashboard/investigations`
- `POST /api/dashboard/investigations`
- `GET /api/dashboard/investigations/{id}`
- `PATCH /api/dashboard/investigations/{id}`

This is the biggest missing product layer today.

### Config registry and changes

- `GET /api/dashboard/config/registry?family=providers`
- `GET /api/dashboard/config/records/{family}/{id}`
- `GET /api/dashboard/config/records/{family}/{id}/history`
- `POST /api/dashboard/changes`
- `POST /api/dashboard/changes/{id}/validate`
- `POST /api/dashboard/changes/{id}/publish`
- `GET /api/dashboard/changes/{id}/watch`
- `POST /api/dashboard/changes/{id}/rollback`

The current `config_ops` endpoints remain the low-level transactional layer.
These new routes would provide the staged workflow model that the UI now expects.

### Sources and integrations

- `GET /api/dashboard/sources`
- `POST /api/dashboard/sources`
- `GET /api/dashboard/sources/{id}`
- `PATCH /api/dashboard/sources/{id}`
- `GET /api/dashboard/integrations`

This is how Prism should support SLS, OTLP, Tempo, ClickHouse, and other evidence systems without adding vendor-specific pages.

## What Should Stay as Primitives

Do not overload the new aggregate routes with low-level mutation semantics.

Keep these direct primitives:

- provider CRUD
- auth-profile CRUD and connect flows
- auth-key CRUD
- config validate and apply
- routing preview and explain
- tenant metrics
- protocol matrix and provider capabilities

These are already truthful and implementation-ready.

## Major Gaps Relative to the Design

The approved design package now assumes these backend capabilities exist eventually.
Today, they do not exist as first-class objects.

### 1. No first-class signal model

Current state:

- the UI can derive anomalous conditions, but the backend does not persist signal objects

Need:

- signal identity
- severity
- ownership
- acknowledgment
- linked evidence

### 2. No first-class investigation model

Current state:

- request logs and route explain are inspectable, but findings are not durable

Need:

- investigation persistence
- notes
- pinned evidence
- compare mode
- shareable URLs

### 3. No first-class change object

Current state:

- `config_ops` can validate and apply, but there is no staged `Change` object

Need:

- draft
- review
- publish
- watch window
- rollback evidence

### 4. No typed source registry

Current state:

- external observability systems are only a future direction

Need:

- source registration
- health and freshness
- capability declaration
- deep-link builders

### 5. No request-to-change correlation

Current state:

- a request can be inspected, and config can be changed, but causality across them is not modeled

Need:

- link a request session to candidate changes
- compare pre-change and post-change windows
- watch change-linked regressions

### 6. Tenant policies are implicit, not first-class

Current state:

- tenants are visible as usage summaries
- auth keys can be tenant-bound
- there is no durable tenant policy object

Need:

- quotas
- budgets
- allowed models
- route defaults
- ownership

## Delivery Guidance

The safest rollout is:

1. keep current primitive endpoints stable
2. add new aggregate read models first
3. let the new frontend shell consume those read models
4. add durable write-side objects for investigations and changes only after the read model stabilizes

In other words:

- do not start by rewriting providers, routing, or config mutations
- start by adding workspace-level read models and typed source metadata

This lets Prism complete the canonical frontend while keeping the production entry stable until cutover.

## Practical Recommendation

The production backend should evolve into two layers:

### Layer 1: truthful primitive handlers

These are the existing CRUD, validate, apply, explain, probe, and metrics endpoints.

### Layer 2: control-plane composition handlers

These provide:

- request sessions
- signals
- investigations
- changes
- sources
- watch windows

That structure matches the new design without throwing away the current backend investment.
