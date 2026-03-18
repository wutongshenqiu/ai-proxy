# Technical Design: Prism Dashboard UX System & Prototype-First Redesign

| Field     | Value                                               |
|-----------|-----------------------------------------------------|
| Spec ID   | SPEC-071                                            |
| Title     | Prism Dashboard UX System & Prototype-First Redesign |
| Author    | Codex                                               |
| Status    | Completed                                           |
| Created   | 2026-03-16                                          |
| Updated   | 2026-03-18                                          |

## Overview

This work is now implemented as Prism's canonical production control plane. The UX system and prototype defined here were carried through into the shipped `web/` shell and matching dashboard workspace handlers.

The redesign is organized around one shell and five operator workspaces:

1. `Command Center` for global runtime posture and urgent signals
2. `Traffic Lab` for request streams, filters, and trace drill-down
3. `Provider Atlas` for providers, auth posture, models, and capability truth
4. `Route Studio` for route planning, explanation, and fallback reasoning
5. `Change Studio` for config diffs, rollout sequencing, and post-change observation

The prototype package lives under `docs/design/prism-control-plane/` and includes:

- a high-fidelity standalone HTML prototype
- CSS tokens and component styling
- a small JavaScript interaction layer
- a Figma handoff document
- frontend implementation guidance
- backend control-plane object guidance

Reference: [PRD](prd.md)

Related implementation notes:

- [../../design/prism-control-plane/frontend-implementation-plan.md](/Users/qiufeng/work/proxy/prism/docs/design/prism-control-plane/frontend-implementation-plan.md)
- [../../design/prism-control-plane/backend-control-plane-model.md](/Users/qiufeng/work/proxy/prism/docs/design/prism-control-plane/backend-control-plane-model.md)
- [../../design/prism-control-plane/rollout-strategy.md](/Users/qiufeng/work/proxy/prism/docs/design/prism-control-plane/rollout-strategy.md)

## API Design

The completed implementation added canonical workspace-level dashboard APIs while preserving runtime truth from SPEC-065 and SPEC-066.

Delivered guidance:

- keep backend truth sources from SPEC-065 and SPEC-066
- add frontend aggregation only where a workspace needs a composed control-plane view
- prefer canonical workspace-level queries over page-local ad hoc fetch chains
- treat external analytics and workflow systems as typed integrations rather than vendor-specific page forks

### Extensibility Model

The shell should support three information planes:

1. `Runtime truth` from Prism itself
2. `Analytics` from native or external stores
3. `Workflow` integrations for review, incident handling, and collaboration

The shell should stay stable while these planes evolve.

Recommended principle:

- runtime truth powers actions
- analytics powers correlation and long-range investigation
- workflow systems power coordination

This means future systems such as SLS, OTLP/Tempo/Jaeger, ClickHouse, Prometheus, or incident tools should appear as typed sources inside workspaces, not as new primary navigation silos.

### Proposed Frontend State Shapes

```ts
type WorkspaceId =
  | 'command-center'
  | 'traffic-lab'
  | 'provider-atlas'
  | 'route-studio'
  | 'change-studio';

interface ControlPlaneContext {
  environment: string;
  timeRange: '15m' | '1h' | '6h' | '24h';
  provider?: string;
  model?: string;
  tenant?: string;
  live: boolean;
}

interface InspectorState {
  entityType: 'request' | 'provider' | 'route' | 'change' | null;
  entityId: string | null;
}

interface WorkbenchState {
  mode: 'closed' | 'create' | 'edit' | 'review';
  workflow:
    | 'provider'
    | 'auth-profile'
    | 'route-rule'
    | 'config-publish'
    | null;
}

type DataSourceKind =
  | 'prism-runtime'
  | 'native-log-store'
  | 'external-logs'
  | 'external-metrics'
  | 'external-traces'
  | 'workflow';

interface DataSourceState {
  activeSource: string;
  mode: 'native' | 'hybrid' | 'external';
  availableSources: Array<{
    id: string;
    kind: DataSourceKind;
    label: string;
    vendor?: string;
    health: 'connected' | 'degraded' | 'disconnected';
    capabilities: string[];
  }>;
}

interface InvestigationState {
  id: string | null;
  title: string;
  status: 'open' | 'watching' | 'resolved';
  owner?: string;
  pinnedEvidence: Array<{
    kind: 'request' | 'route' | 'provider' | 'auth' | 'change' | 'external-link' | 'note';
    id: string;
    label: string;
  }>;
  comparisonMode?: 'baseline' | 'time-range' | 'pre-post-change';
}

interface ConfigRegistryState {
  activeFamily:
    | 'providers'
    | 'auth-profiles'
    | 'route-profiles'
    | 'route-rules'
    | 'auth-keys'
    | 'tenant-policies'
    | 'data-sources'
    | 'alerts';
  activeRecordId?: string;
  mode: 'browse' | 'create' | 'edit' | 'clone' | 'review-delete' | 'history';
}

interface ConfigRecordSummary {
  id: string;
  family: ConfigRegistryState['activeFamily'];
  status: 'active' | 'warning' | 'disabled' | 'draft' | 'archived';
  owner?: string;
  dependentCount: number;
  lastChangedAt: string;
}

interface ConfigImpactState {
  recordId: string;
  affectedRoutes: string[];
  affectedTenants: string[];
  affectedInvestigations: string[];
  deleteMode: 'soft-disable' | 'archive' | 'guarded-hard-delete';
}

interface EditorWorkflowState {
  family: 'provider' | 'auth-profile' | 'route-profile' | 'route-rule';
  step:
    | 'identity'
    | 'connectivity'
    | 'presentation'
    | 'validate'
    | 'mode'
    | 'connect'
    | 'verify'
    | 'matchers'
    | 'simulate'
    | 'impact';
  runtimeChecks: Array<'health' | 'model-fetch' | 'presentation-preview' | 'auth-status' | 'route-preview' | 'route-explain'>;
}
```

### URL State Guidance

The eventual implementation should treat URL search state as part of the control-plane model:

```text
/?workspace=traffic-lab&provider=claude-sub&tenant=team-red&panel=request&id=req_3471
```

That allows copyable operator context, better browser navigation, and lower coupling between pages and local component state.

## Backend Implementation

The implementation added:

- workspace-oriented read models under `crates/server/src/handler/dashboard/control_plane_workspace/`
- provider/auth/runtime-truth seams for provider probes, managed auth, and control-plane aggregation
- frontend shell composition, workflow controllers, i18n foundations, and real browser/live-provider validation

Additional implementation detail now lives in:

- [../../design/prism-control-plane/frontend-implementation-plan.md](/Users/qiufeng/work/proxy/prism/docs/design/prism-control-plane/frontend-implementation-plan.md)
- [../../design/prism-control-plane/backend-control-plane-model.md](/Users/qiufeng/work/proxy/prism/docs/design/prism-control-plane/backend-control-plane-model.md)

### Proposed Frontend Module Structure

```text
web/src/
├── shell/
│   ├── AppShell.tsx
│   ├── GlobalContextBar.tsx
│   ├── WorkspaceHeader.tsx
│   ├── InspectorRail.tsx
│   └── CommandPalette.tsx
├── integrations/
│   ├── registry.ts
│   ├── sources.ts
│   └── deepLinks.ts
├── workspaces/
│   ├── command-center/
│   ├── traffic-lab/
│   ├── provider-atlas/
│   ├── route-studio/
│   └── change-studio/
├── stores/
│   ├── shellStore.ts
│   ├── contextStore.ts
│   ├── dataSourceStore.ts
│   ├── configRegistryStore.ts
│   ├── inspectorStore.ts
│   └── workflowStore.ts
├── queries/
│   ├── traffic.ts
│   ├── providers.ts
│   ├── routing.ts
│   ├── changes.ts
│   └── integrations.ts
└── components/
    ├── data-grid/
    ├── source-mode/
    ├── signals/
    ├── inspector/
    └── forms/
```

### Key Types

```ts
interface WorkspaceDefinition {
  id: WorkspaceId;
  label: string;
  summary: string;
  defaultInspector: string;
}

interface InspectorRecord {
  id: string;
  badge?: string;
  title: string;
  summary: string;
  facts: Array<{ label: string; value: string }>;
  notes: string[];
  actions: string[];
}

interface DeepLinkDefinition {
  id: string;
  sourceId: string;
  label: string;
  target: 'internal-panel' | 'new-tab';
  buildUrl(context: ControlPlaneContext & { requestId?: string }): string;
}
```

### Flow

1. Global context changes update one shared context store and the URL.
2. Active workspace reads shared context and fetches only the datasets it needs.
3. Selecting any row, card, or route decision updates the inspector store instead of spawning a workflow-specific modal.
4. Multi-step operations open an embedded workbench pattern, not a generic centered modal.
5. Notifications and live status are rendered by the shell, not reimplemented per page.
6. Data source selection changes what evidence is shown, but does not change the workspace mental model.
7. External integrations may enrich, compare, or deep-link from a workspace, but should not override Prism runtime truth for live operator actions.
8. Investigations should be shareable objects that can pin runtime evidence, external evidence, and change context in one place.

## Configuration Changes

No user-facing configuration changes are needed for the prototype phase.

## Provider Compatibility

| Provider | Supported | Notes |
|----------|-----------|-------|
| OpenAI   | Yes | Dashboard-only redesign; runtime behavior unchanged |
| Claude   | Yes | Dashboard-only redesign; runtime behavior unchanged |
| Gemini   | Yes | Dashboard-only redesign; runtime behavior unchanged |

## Alternative Approaches

| Approach | Pros | Cons | Verdict |
|----------|------|------|---------|
| Restyle current pages in place | Fastest visual refresh | Keeps fractured interaction model | Rejected |
| Figma-only concept without runnable prototype | Easy to discuss visually | Harder to validate density and interaction flow | Rejected |
| Full implementation before prototype | Produces working UI faster on paper | High risk of rework and design drift | Rejected |
| Standalone prototype plus design system, then implementation | Stable direction, low-risk handoff | Adds one upfront design step | Chosen |

## Task Breakdown

- [x] Capture research and UX decisions in SPEC-071 docs.
- [x] Produce a standalone prototype package under `docs/design/prism-control-plane/`.
- [x] Validate the prototype visually in a browser and capture review screenshots.
- [x] Convert the approved prototype into an implementation spec for `web/`.
- [x] Rebuild the production dashboard shell and migrate pages into workspace modules.

## Test Strategy

- **Rust verification:** `cargo fmt --check`, `cargo clippy --workspace --tests -- -D warnings`, `cargo test --workspace`
- **Frontend verification:** `npm run typecheck`, `npm run lint`, `npm run test -- --run`, `npm run build`
- **Browser verification:** real control-plane flow via `web/scripts/real-flow-check.mjs` with preserved screenshots and JSON reports under `artifacts/playwright/real-flow/`
- **Live provider verification:** real Codex and DashScope validation via `web/scripts/live-provider-matrix-check.mjs` with preserved screenshots and JSON reports under `artifacts/runtime/provider-live-check/`

## Rollout Plan

1. Approve the prototype direction and information architecture.
2. Recreate the prototype as the canonical production shell in `web/`.
3. Build all required workspaces and shared patterns directly in that shell until the control plane is ready for cutover.
4. Validate the new control plane end-to-end, including live provider verification.
5. Switch the production entry point to the new control plane in one release.
