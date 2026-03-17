# North Star Model

This note intentionally ignores current backend limits and current dashboard page boundaries.

It answers a different question:

If Prism were designed today as a modern AI gateway control plane, what should the product model look like?

## Product Thesis

Prism should not be designed as:

- a config editor
- a logs page
- a provider CRUD console

Prism should be designed as:

- an operator control plane for AI traffic
- an investigation surface for runtime failures and regressions
- a change management surface for policy, routing, auth, and rollout decisions

That means the core product loop is:

`Detect -> Investigate -> Decide -> Change -> Observe -> Learn`

## Best-Practice Patterns To Borrow

These patterns come from official product/docs references, not from Prism's current implementation.

### 1. Investigation should be first-class

Grafana's Investigations model is useful because it treats an investigation as a curated set of signals rather than a single chart or single log query.

Useful ideas:

- mix metrics, logs, traces, and profiles in one analysis
- annotate why a signal matters
- compare time ranges
- turn an investigation into a reusable dashboard

Reference:

- https://grafana.com/docs/grafana/latest/explore/simplified-exploration/investigations/

Implication for Prism:

- an operator should be able to pin a request, route explain, provider health state, and change diff into one investigation workspace
- the product should support a persistent investigation object, not only temporary page state

### 2. Logs should support patterns, facets, and saved views

Datadog's Log Explorer emphasizes three things:

- facets and attributes for slicing
- grouping logs into higher-level entities such as fields, patterns, and transactions
- switching between list and time-based visualizations

Reference:

- https://docs.datadoghq.com/logs/explorer/

Implication for Prism:

- `Traffic Lab` should not stop at raw rows
- it should support saved lenses, anomaly grouping, and pattern-level views
- request list and time-series view should be two faces of the same investigation surface

### 3. Incident entities need ownership and action context

Sentry's issue detail model is useful because it combines:

- high-level impact summary
- user / frequency context
- actions like assign, resolve, share, archive
- suspect commits and ownership links

Reference:

- https://docs.sentry.io/product/issues/issue-details/

Implication for Prism:

- incidents and regressions should have owners, status, links, and change context
- a route incident should be more than a red cell in a table
- Prism should eventually support issue-like entities for recurring failures

### 4. Pipeline visibility should remain explicit

Apigee Trace remains a strong model for pipeline visibility:

- show the request journey step by step
- reveal where transformation or policy behavior changed the outcome

Reference:

- https://cloud.google.com/apigee/docs/api-platform/debug/trace

Implication for Prism:

- route explain alone is not enough
- the product should expose a full request pipeline story: ingress, transform, route selection, auth attachment, attempt chain, response

### 5. Control planes should stay runtime-first

Kong, Tyk, Gravitee, and similar systems consistently prioritize runtime posture, analytics, and drill-down over static settings pages.

References:

- https://developer.konghq.com/ai-gateway/
- https://konghq.com/products/kong-konnect/features/advanced-analytics
- https://tyk.io/docs/5.2/tyk-dashboard-analytics/
- https://documentation.gravitee.io/apim/analyze-and-monitor-apis/dashboards/overview-dashboard

Implication for Prism:

- the home of the product is not "Settings"
- the home of the product is runtime posture and active operator work

## The Objects Prism Should Be Built Around

Instead of building around current pages, Prism should build around these objects.

### 1. Request Session

The atomic runtime event.

Contains:

- request identity
- normalized request shape
- upstream transform
- route decision
- attempt chain
- response / stream preview
- related tenant, provider, auth, and change version

### 2. Investigation

A curated analysis thread, not just a filter state.

Contains:

- title
- owner
- status
- pinned evidence
- annotations
- time-range comparisons
- related changes
- external links

### 3. Runtime Entity

Providers, auth profiles, tenants, models, routes, protocol surfaces.

Each entity should have:

- current state
- historical health / usage clues
- dependencies
- linked investigations
- related pending changes

### 4. Change

A staged mutation with intent and evidence.

Contains:

- structured diff
- semantic diff
- risk
- rollout plan
- watch window
- rollback criteria
- linked investigation

### 5. Signal

An anomaly, threshold breach, drift event, or regression cluster.

Signals should be promotable into investigations.

## What This Means For The IA

The current five workspaces still make sense, but their job should be reframed:

- `Command Center`: signals, investigations, ownership, posture
- `Traffic Lab`: request sessions, patterns, comparisons, source switching
- `Provider Atlas`: entity graph for providers, auth, protocols, models
- `Route Studio`: decision lab and policy simulation
- `Change Studio`: staged mutation, rollout, evidence, watch windows

So the workspaces are not page categories.

They are task environments around the core objects above.

## New Capabilities Prism Should Eventually Want

These are not limited by today's backend.

### Investigation Journal

Operators can pin:

- a request session
- an SLS query
- a route explain result
- a provider health snapshot
- a config diff
- a note

### Evidence Graph

Show relationships between:

- request
- route
- provider
- auth profile
- change
- incident

### Time Comparison

Compare:

- current failing request vs successful baseline
- current 15m vs yesterday 15m
- pre-change vs post-change windows

### Pattern View

Group requests by:

- error pattern
- provider fallback path
- route rejection reason
- tenant spike signature

### Ownership Model

Every serious signal should be assignable, shareable, and linkable to external workflow.

## Anti-Patterns To Avoid

- Do not build only for CRUD pages.
- Do not treat an investigation as disposable local state.
- Do not separate incident triage from configuration evidence.
- Do not make external systems the only place where real history lives.
- Do not make the shell depend on the current backend response shape forever.

## Immediate Prototype Consequences

The next prototype layer should visibly introduce:

1. investigation as a first-class object
2. evidence graph / pinned evidence concept
3. time comparison as a built-in operator action
4. links between incidents and changes

## Decision Summary

If we drop historical baggage completely, Prism should evolve toward this model:

1. requests are not just log rows, they are sessions
2. incidents are not just alerts, they are investigations
3. config edits are not just saves, they are evidence-backed changes
4. workspaces exist to support these objects, not to mirror old pages
