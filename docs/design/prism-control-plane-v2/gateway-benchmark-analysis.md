# Gateway Benchmark Analysis for Prism Control Plane V2

As of `2026-03-17`, this note is the durable benchmark for Prism's control-plane redesign before the reusable shell and shared patterns are treated as implementation guidance.

The goal is not to decide whether Prism should "match" the current backend or copy another gateway's UI.

The goal is to answer three questions:

1. What runtime and config primitives Prism already has in code
2. What adjacent gateway and control-plane products consistently get right
3. What product model Prism should adopt even when the current backend does not fully support it yet

## Executive Summary

The strongest conclusion from current code and external benchmarks is:

- Prism should optimize around `sessions`, `signals`, `changes`, `providers`, `route drafts`, and `data sources`
- Prism should not preserve today's page boundaries as product boundaries
- Prism should keep one control-plane shell and let the backend evolve behind it
- Prism should treat native runtime truth, external analytics, and workflow systems as separate information planes

The current backend is already strong enough to support a much better operator experience than the current dashboard exposes.

What is missing is mostly not "more CRUD pages". What is missing is:

- first-class debugging workflows
- first-class staged change workflows
- first-class evidence grouping
- a typed integration model for external sources such as `SLS`, `OTLP`, `Tempo`, `ClickHouse`, or `Prometheus`

## Prism Current Capability Baseline

The current codebase already contains the primitives for a serious control plane.

### Request and route debugging already exist

- [RequestLogs.tsx](/Users/qiufeng/work/proxy/prism/web/src/pages/RequestLogs.tsx) already supports URL-backed filters, live mode, and deep-linkable request selection.
- [LogDrawer.tsx](/Users/qiufeng/work/proxy/prism/web/src/components/LogDrawer.tsx) already exposes request identity, provider, credential, tenant, token usage, retry attempts, request body, upstream request body, response body, and stream preview.
- [Replay.tsx](/Users/qiufeng/work/proxy/prism/web/src/pages/Replay.tsx) already exposes route explain, model resolution, scoring, alternates, and rejections.
- [routing.rs](/Users/qiufeng/work/proxy/prism/crates/server/src/handler/dashboard/routing.rs) already separates lightweight preview from full explain.

Interpretation:

- Prism does not lack debugging power.
- Prism lacks a first-class `request session debugger`.

### Config mutation already has truth, validation, and conflict semantics

- [Config.tsx](/Users/qiufeng/work/proxy/prism/web/src/pages/Config.tsx) already models runtime snapshot, raw YAML, validate, apply, and optimistic concurrency.
- [config_ops.rs](/Users/qiufeng/work/proxy/prism/crates/server/src/handler/dashboard/config_ops.rs) already exposes `validate`, `reload`, `apply`, `raw`, and `current`.
- Secret resolution is already validated for `env://` and `file://` references before apply.

Interpretation:

- Prism does not need to stay centered on a raw YAML editor.
- Prism can already support a staged change experience with better composition.

### Provider and auth runtime truth are richer than the current UI framing

- [Providers.tsx](/Users/qiufeng/work/proxy/prism/web/src/pages/Providers.tsx) already supports model fetch, health checks, and presentation preview.
- [AuthProfiles.tsx](/Users/qiufeng/work/proxy/prism/web/src/pages/AuthProfiles.tsx) already supports connect, local import, browser OAuth, device flow, and refresh.
- [providers.rs](/Users/qiufeng/work/proxy/prism/crates/server/src/handler/dashboard/providers.rs) already hydrates provider summaries with runtime auth state and probe results.
- [dashboard_tests.rs](/Users/qiufeng/work/proxy/prism/crates/server/tests/dashboard_tests.rs) already verifies that runtime-only auth state is preserved outside static config and that OAuth flows work.

Interpretation:

- Prism should stop presenting providers and auth as unrelated admin silos.
- Providers, identities, capabilities, and health should become one operator object family.

### Current product gaps are mostly compositional

The most important current gaps are:

- request detail is still a drawer instead of a persistent debugger workspace
- route explain is still separated from live request investigation
- config publishing is still framed as save/apply rather than a staged change lifecycle
- there is no first-class `change` object with watch window, owner, or rollback posture
- there is no first-class `signal` or `investigation` object
- there is no typed source registry for external observability systems

## External Benchmark Findings

The products below are useful because they are solving adjacent operator problems, not because Prism should mimic their information architecture.

## OpenClaw

Official docs:

- `Control UI`: https://docs.openclaw.ai/web/control-ui
- `Dashboard`: https://docs.openclaw.ai/web/dashboard
- `Logging`: https://docs.openclaw.ai/logging
- `Doctor`: https://docs.openclaw.ai/gateway/doctor

Useful patterns:

- The Control UI is a small gateway-served SPA instead of a separate enterprise console product.
- The UI is explicitly an admin surface and its auth model is called out as such.
- The Logs tab tails the same rolling file log the CLI uses, instead of inventing a separate truth source.
- The UI localizes itself from browser locale, supports explicit language switching, and falls back cleanly.
- `openclaw doctor` is treated as a first-class repair flow, not as scattered troubleshooting pages.

What Prism should borrow:

- CLI, gateway, and UI should share the same runtime truth wherever possible.
- The UI should make repair and diagnosis explicit when runtime prerequisites are broken.
- i18n should be built into the shell early, not patched in later.

What Prism should not copy literally:

- OpenClaw is optimized for a self-hosted agent gateway plus chat/control experience.
- Prism is more squarely an AI/API traffic and routing control plane, so its primary object should still be `request session` rather than `chat session`.

## CLIProxyAPI and its Management Center

Official docs and repos:

- `Web UI`: https://help.router-for.me/management/webui.html
- `Management API`: https://help.router-for.me/cn/management/api.html
- `CLIProxyAPI repo`: https://github.com/router-for-me/CLIProxyAPI
- `Management Center repo`: https://github.com/router-for-me/Cli-Proxy-API-Management-Center

Useful patterns:

- The management panel is treated as a replaceable static asset rather than a permanently fused monolith.
- The project explicitly frames the management center as a tool for both config modification and runtime status monitoring.
- Remote panel updates are checksum-aware, which is a healthy integrity pattern for a self-hosted admin surface.
- The Management API is treated as a proper operational surface, not just a hidden helper for forms.
- The main project keeps runtime routing, account rotation, model fallback, and CLI-specific compatibility in one operator story.

What Prism should borrow:

- The UI shell should be able to evolve separately from gateway internals.
- Config mutation should stay behind one truthful management API contract.
- Runtime status and config changes should belong to one control plane.

What Prism should improve beyond CLIProxyAPI:

- Prism should avoid staying too close to a "config page plus status page" mental model.
- Prism should explicitly model `change`, `signal`, and `session`, not just config files and runtime tables.

## Apigee

Official docs:

- `Using Debug`: https://docs.cloud.google.com/apigee/docs/api-platform/debug/trace

Useful patterns:

- Debug is a named session with filters created up front.
- The operator can see which steps succeeded or failed and where time was spent.
- Sessions are shareable, downloadable, and time-bounded.
- The product explicitly connects analytics anomalies to request debugging.

What Prism should borrow:

- Treat debugging as a saved or shareable investigation state, not only as a transient drawer.
- Show a true step-by-step execution chain: ingress, transform, route decision, upstream attempt, retry, and failure cause.
- Allow targeted capture and saved evidence rather than only generic logs browsing.

## Kong

Official docs:

- `AI Gateway`: https://developer.konghq.com/ai-gateway/
- `Gateway tracing`: https://developer.konghq.com/gateway/tracing/
- `Debug Kong Gateway requests`: https://developer.konghq.com/gateway/debug-requests/
- `Advanced Analytics`: https://developer.konghq.com/index/advanced-analytics/
- `Konnect Debugger`: https://developer.konghq.com/konnect-platform/active-tracing/
- `OpenTelemetry plugin`: https://developer.konghq.com/plugins/opentelemetry/

Useful patterns:

- Every request gets a correlation identity that can be followed in headers and logs.
- There is an on-demand debug path for trace-level timing instead of forcing full deep traces on every request.
- The lifecycle map in Konnect Debugger makes request phases visually inspectable.
- OTLP is treated as a normal control-plane extension, not an afterthought.

What Prism should borrow:

- Stable request identity should anchor the whole debugging experience.
- Native runtime truth and external tracing should coexist cleanly.
- The UI should represent timing and execution phases explicitly, not bury them in JSON.

## Gravitee

Official docs:

- `Overview dashboard`: https://documentation.gravitee.io/apim/analyze-and-monitor-apis/dashboards/overview-dashboard

Useful patterns:

- The overview stays environment-level and posture-first.
- The top-level dashboard is for visibility and drill-down, not exhaustive configuration.

What Prism should borrow:

- `Command Center` should stay focused on posture, urgent signals, and entry points into investigation.
- It should not regress into a wall of equal-weight metrics and cards.

## Grafana

Official docs:

- `Investigations`: https://grafana.com/docs/grafana/latest/explore/simplified-exploration/investigations/investigations/
- `Simplified exploration`: https://grafana.com/docs/grafana-cloud/visualizations/simplified-exploration/
- `Explore`: https://grafana.com/explore

Useful patterns:

- Logs, metrics, traces, and profiles can be collected into one investigation.
- Investigation items can be commented, compared across time ranges, and turned into a wider incident workflow.
- Queryless drilldown lowers the activation energy for operators who are triaging, not authoring dashboards.

What Prism should borrow:

- Prism should treat evidence as a set, not as a single page view.
- Time compare, pre/post-change compare, and linked notes are worth designing for now even if backend persistence comes later.

## Product Guidance Not Bound to Current Backend

Prism should not be organized around today's page inventory or handler modules.

It should be organized around first-class operator objects.

### 1. Request Session

The main debugging object should be a `request session`, not a log row.

Minimum shape:

- request identity
- tenant / API key / region / model
- route decision and model resolution
- upstream attempts and retry chain
- transform and payload deltas
- linked provider and auth identity
- linked change and external evidence

### 2. Signal

The main triage object should be a `signal`, not just a red metric.

Examples:

- fallback spike
- auth refresh failure cluster
- provider latency drift
- canary degradation
- config conflict storm

Signals should be promotable into investigations or changes.

### 3. Change

The main config mutation object should be a `change`, not only a YAML edit.

Minimum lifecycle:

`draft -> validate -> impact review -> publish -> watch -> rollback or complete`

### 4. Provider Identity

The main provider object should unify:

- endpoint configuration
- auth posture
- capabilities
- health and probes
- rotation and weighting
- coverage and route linkage

### 5. Route Draft

Routing should be edited as a draftable object with:

- matcher logic
- profile choice
- simulation
- blast radius
- publish linkage

### 6. Data Source

Prism should treat observability sources as typed objects:

- native runtime
- native log store
- external logs
- external traces
- external metrics
- workflow or incident systems

This is the key move that prevents future `SLS`, `OTLP`, or `ClickHouse` integrations from turning into navigation sprawl.

## Recommended Control-Plane Model

The current five-workspace model still holds:

- `Command Center`
- `Traffic Lab`
- `Provider Atlas`
- `Route Studio`
- `Change Studio`

But those workspaces should be understood as views over the objects above, not as isolated page silos.

### Shell contract

Every workspace should share:

- one global context bar
- one inspector rail
- one embedded workbench pattern
- one source-mode model: `runtime`, `hybrid`, `external`
- one shareable URL state model

### Traffic Lab contract

Primary job:

- explain and compare request sessions

Secondary jobs:

- slice traffic
- group failures into signals
- pivot into providers, auth, routes, and changes

### Change Studio contract

Primary job:

- safely mutate system behavior

Secondary jobs:

- browse config registry
- inspect dependencies
- publish gradually
- watch live evidence

### Provider Atlas contract

Primary job:

- maintain provider and identity posture

Secondary jobs:

- capability and coverage review
- auth lifecycle
- rotation and disablement
- linked route insight

## Recommended Backend-Agnostic Architecture Decisions

These are the design decisions worth locking in before React reuse.

### Decision 1: Keep runtime truth separate from analytics truth

Native runtime truth should power actions.

External analytics should power correlation and long-range investigation.

This avoids a common failure mode where the UI acts confidently on stale warehouse data.

### Decision 2: Preserve one truthful config mutation path

Even if Prism later gains multiple structured editors, they should all converge on one config transaction model with:

- validation
- version conflict semantics
- auditability
- reload/apply outcome

### Decision 3: Allow deeper workflow objects than the backend currently stores

It is acceptable for the UI prototype and future React structure to model:

- changes
- investigations
- watch windows
- evidence groups

before the backend persists them directly.

The shell should not be held back by today's handler granularity.

### Decision 4: Treat external integrations as capability-bearing sources

An external integration should declare:

- kind
- health
- supported pivots
- deep-link formats
- evidence types

This is much better than hardcoding vendor tabs.

### Decision 5: Model high-density accessibility early

For a dense control plane:

- focus states must stay visible
- tab order must match visual order
- locale growth must be tolerated
- technical identifiers must remain untranslated
- tables need stable overflow behavior

## What Prism Can Build Immediately vs Later

### Feasible now with current backend

- a real `Traffic Lab` workspace composed from logs, request detail, route explain, provider health, and auth status
- a real `Change Studio` workspace composed from runtime snapshot, raw config, validate/apply, routing edits, provider edits, and auth profile flows
- a real `Provider Atlas` based on current provider and auth runtime handlers

### Needs additive backend work later

- persistent investigations
- first-class change history and watch windows
- signal detection and acknowledgment
- request-to-explain joins by request id without manual replay
- typed external source registry
- comparative pre/post-change evidence capture

### Worth planning for even if not phase one

- saved operator lenses
- blast-radius simulation against historical traffic
- rollback recommendations
- incident ownership and runbooks

## Anti-Patterns To Avoid

- Do not preserve `Logs`, `Replay`, `Routing`, and `Config` as permanently separate mental models.
- Do not let raw YAML remain the default path for common operational changes.
- Do not build new pages around current backend modules just because the handlers are separate today.
- Do not introduce vendor-specific primary navigation for every new observability or incident tool.
- Do not show false precision or false health when backend truth does not exist.
- Do not translate technical IDs, model names, provider IDs, config keys, or raw upstream payloads.

## Recommended Next Reuse Step

Before reusing the current Pencil work in React or `.pen` foundations, Prism should treat the following as fixed product guidance:

1. The reusable shell is valid.
2. The five workspace model is valid.
3. The control plane should optimize around `session`, `signal`, `change`, `provider identity`, `route draft`, and `data source`.
4. The current backend should be treated as a strong primitive layer, not a boundary on product ambition.

## References

- OpenClaw Control UI: https://docs.openclaw.ai/web/control-ui
- OpenClaw Dashboard: https://docs.openclaw.ai/web/dashboard
- OpenClaw Logging: https://docs.openclaw.ai/logging
- OpenClaw Doctor: https://docs.openclaw.ai/gateway/doctor
- CLIProxyAPI Web UI: https://help.router-for.me/management/webui.html
- CLIProxyAPI Management API: https://help.router-for.me/cn/management/api.html
- CLIProxyAPI: https://github.com/router-for-me/CLIProxyAPI
- CLIProxyAPI Management Center: https://github.com/router-for-me/Cli-Proxy-API-Management-Center
- Apigee Debug: https://docs.cloud.google.com/apigee/docs/api-platform/debug/trace
- Kong AI Gateway: https://developer.konghq.com/ai-gateway/
- Kong Gateway tracing: https://developer.konghq.com/gateway/tracing/
- Kong debug requests: https://developer.konghq.com/gateway/debug-requests/
- Kong Advanced Analytics: https://developer.konghq.com/index/advanced-analytics/
- Kong Konnect Debugger: https://developer.konghq.com/konnect-platform/active-tracing/
- Kong OpenTelemetry plugin: https://developer.konghq.com/plugins/opentelemetry/
- Gravitee overview dashboard: https://documentation.gravitee.io/apim/analyze-and-monitor-apis/dashboards/overview-dashboard
- Grafana Investigations: https://grafana.com/docs/grafana/latest/explore/simplified-exploration/investigations/investigations/
- Grafana simplified exploration: https://grafana.com/docs/grafana-cloud/visualizations/simplified-exploration/
- Grafana Explore: https://grafana.com/explore
