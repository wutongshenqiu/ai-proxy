# Extensibility Model

This note defines how the Prism control plane should stay extensible instead of freezing around today's built-in dashboard pages.

The short version:

- Prism runtime truth stays first-class.
- External systems such as SLS should plug in as typed data sources, not ad hoc buttons.
- The shell should support hybrid workflows where operators inspect in Prism, correlate in external analytics, and return without losing context.

## Why Extensibility Matters

The current redesign is intentionally not limited by the old dashboard layout.

That means it also should not be limited by the current data sources.

Prism already points in this direction:

- request logs were designed with future backends like SLS and SQLite in mind in [SPEC-041](/Users/qiufeng/work/proxy/prism/docs/specs/completed/SPEC-041/technical-design.md)
- tracing and OTel export were explicitly called out in [SPEC-040](/Users/qiufeng/work/proxy/prism/docs/specs/completed/SPEC-040/prd.md)
- the current dashboard already mixes live runtime state, persisted config, and derived analytics

If the UI is designed only around current in-memory metrics and request logs, it will become rigid quickly.

## The Three Data Planes

The UI should treat control-plane information as three planes, not one blob.

### 1. Runtime Truth Plane

Owned directly by Prism.

Examples:

- active provider/auth/runtime status
- route explain and route preview
- config snapshot and config version
- live request stream
- live websocket connection state
- publish / reload / conflict / validation actions

This plane is authoritative for operator actions.

### 2. Analytics Plane

Can be native or external.

Examples:

- in-memory request log stats
- file audit history
- SLS log queries
- ClickHouse / BigQuery / warehouse analytics
- Prometheus / long-range metrics
- OTLP / Tempo / Jaeger trace correlation

This plane is authoritative for history, aggregation, and correlation.

### 3. Workflow Plane

Connects operators to systems around Prism.

Examples:

- Slack / Feishu incident threads
- Jira / Linear change references
- GitOps / config repo review links
- PagerDuty / alert ownership
- internal runbooks

This plane is authoritative for coordination, not runtime decisions.

## Core Rule

The shell should never confuse these three planes.

Recommended behavior:

- runtime truth drives inline status and operator actions
- analytics sources enrich or extend investigation
- workflow tools handle notifications, ownership, and review

Do not let a stale external dashboard overwrite a fresh Prism runtime signal without making that distinction visible.

## Data Source Model

The eventual implementation should treat data sources as typed capabilities.

```ts
type DataSourceKind =
  | 'prism-runtime'
  | 'native-log-store'
  | 'external-logs'
  | 'external-metrics'
  | 'external-traces'
  | 'workflow';

type DataSourceScope =
  | 'command-center'
  | 'traffic-lab'
  | 'provider-atlas'
  | 'route-studio'
  | 'change-studio';

interface DataSourceDefinition {
  id: string;
  kind: DataSourceKind;
  label: string;
  vendor?: string;
  scopes: DataSourceScope[];
  capabilities: string[];
  mode: 'native' | 'hybrid' | 'external';
  health: 'connected' | 'degraded' | 'disconnected';
}

interface CrossLink {
  sourceId: string;
  label: string;
  target: 'internal-panel' | 'new-tab' | 'deep-link';
  template: string;
}
```

## How SLS Fits

SLS is a good example because it should not be modeled as “another page”.

It should be a data source with specific capabilities:

- long-range log analytics
- indexed search
- saved queries / dashboards
- correlation by request id, tenant id, provider, model, status

Recommended SLS UX patterns:

- `Traffic Lab` source switch: `Runtime`, `Hybrid`, `SLS`
- request row deep links into SLS with current filters preserved
- `Command Center` health card for SLS ingestion freshness
- `Change Studio` watch window that can compare “Prism live” vs “SLS 15m aggregate”

What not to do:

- do not rebuild all of SLS inside Prism
- do not make SLS the only way to inspect a request
- do not hide whether a chart comes from Prism runtime vs external SLS

## Workspace-Level Extensibility

### Command Center

Should support:

- source health cards
- ingestion lag indicators
- external analytics freshness
- drill-through links to external dashboards

Useful for:

- “Prism shows a spike, is SLS ingestion current?”
- “This incident needs long-range historical confirmation.”

### Traffic Lab

Should support:

- source mode switching: runtime / hybrid / external
- correlated request search by request id / tenant / provider / model
- external trace jump links
- compare local request detail against external trace/log context

This is the most important extensibility point.

### Provider Atlas

Should support:

- provider-specific observability hooks
- upstream vendor console links
- auth / health / metrics overlays
- integration readiness status

### Route Studio

Should support:

- route explain from Prism runtime
- optional analytics overlays showing historical match/fallback rate
- external evidence for why a route policy is failing in practice

### Change Studio

Should support:

- publish watch windows backed by Prism runtime plus optional external analytics
- rollout evidence cards
- workflow links to PRs, approvals, and incident threads

## UI Rules For Extensibility

### 1. Keep the shell stable

Do not add one nav item per integration.

Integrations belong inside workspaces and inspector surfaces, not in the primary navigation.

### 2. Make source provenance visible

Every important chart, table, or status block should make source provenance obvious:

- Prism Runtime
- Native Log Store
- SLS
- Prometheus
- OTLP Trace

### 3. Prefer hybrid investigation over full vendor embedding

Prism should be the operational frame.

External systems should be:

- query overlays
- compare views
- deep links
- embedded evidence cards

not giant cloned sub-products.

### 4. Preserve context when jumping out

Deep links should carry:

- time range
- environment
- request id
- tenant
- provider
- model

### 5. Do not block core actions on external dependencies

If SLS is disconnected, operators should still be able to:

- inspect live requests
- explain routes
- validate and publish config
- inspect provider/auth runtime posture

## Recommended Future Connectors

The shell should be designed to allow classes of connectors, not hand-coded one-offs.

### Observability

- SLS
- ClickHouse
- Elasticsearch / OpenSearch
- Loki
- Prometheus
- Grafana
- OTLP / Jaeger / Tempo

### Workflow

- GitHub / GitLab PR links
- Jira / Linear tickets
- Slack / Feishu / Teams incident threads
- PagerDuty / Opsgenie escalation links

### Storage / Audit

- local JSONL
- OSS / S3 archived audit
- SQLite or other persisted request stores

## Design Consequences For The Prototype

The prototype should make three extensibility ideas visible:

1. source mode can change without changing the workspace mental model
2. external analytics appear as part of investigation, not as a separate main product
3. source provenance is always visible

That is why the next prototype layer should show:

- an observability/data-source panel in `Command Center`
- a source selector in `Traffic Lab`
- publish watch windows that can reference both Prism live data and external analytics evidence

## Decision Summary

If Prism wants to stay usable after adding SLS and similar systems, the UI should follow these rules:

1. Prism runtime truth remains primary for actions.
2. External analytics are typed data sources, not random feature pages.
3. The shell stays stable while integrations appear inside workspaces and inspector flows.
