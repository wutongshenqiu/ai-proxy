# Remaining Additions Roadmap for Prism Control Plane V2

This note answers a practical question:

What still needs to be added after the current prototype direction is accepted?

It is intentionally broader than a UI checklist.

For Prism, a serious control-plane redesign requires additions in five layers at once:

1. reusable design-system assets
2. workspace-level operator flows
3. cross-workspace product capabilities
4. backend and API capabilities
5. verification and delivery discipline

This roadmap should be read together with:

- [gateway-benchmark-analysis.md](/Users/qiufeng/work/proxy/prism/docs/design/prism-control-plane-v2/gateway-benchmark-analysis.md)
- [north-star-model.md](/Users/qiufeng/work/proxy/prism/docs/design/prism-control-plane-v2/north-star-model.md)
- [extensibility-model.md](/Users/qiufeng/work/proxy/prism/docs/design/prism-control-plane-v2/extensibility-model.md)
- [config-crud-model.md](/Users/qiufeng/work/proxy/prism/docs/design/prism-control-plane-v2/config-crud-model.md)
- [pencil-dev/ARCHITECTURE.md](/Users/qiufeng/work/proxy/prism/docs/design/prism-control-plane-v2/pencil-dev/ARCHITECTURE.md)

## Current Design Coverage

As of `2026-03-17`, the visual design pack now covers more than the original five workspaces.

The active Pencil document already includes:

- `Prism Foundations`
- `Platform Patterns`
- `Entity Editors`
- `Command Center`
- `Traffic Lab`
- `Provider Atlas`
- `Route Studio`
- `Change Studio`

That means a substantial part of the remaining work below is no longer "invent the UI".
The remaining items are mostly about:

- saving canonical `.pen` source files into the repo
- extracting stronger reusable foundations and shared patterns
- defining backend and API primitives that match the north-star model
- establishing implementation and verification discipline

## Definition of Done

Prism should not consider the redesign "complete" when it merely has five good-looking screens.

There are three different finish lines:

### A. Complete for reuse

This means the Pencil and prototype assets are stable enough to become the source of truth for later workspace reuse.

### B. Complete for frontend implementation

This means the React implementation can begin without inventing product rules on the fly.

### C. Complete as a north-star control plane

This means Prism can support not only current runtime truth, but also investigations, staged changes, and external evidence sources in a durable way.

## P0: Must Be Added Before Heavy Reuse

These are the items that should be completed before the team treats the current design direction as a reusable system.

### 1. Save formal `.pen` workspace files

Current gap:

- the active Pencil document contains the work, but the repo still lacks the canonical saved workspace files

Need to add:

- `prism-control-plane-foundations.pen`
- `prism-control-plane-command-center.pen`
- `prism-control-plane-traffic-lab.pen`
- `prism-control-plane-provider-atlas.pen`
- `prism-control-plane-route-studio.pen`
- `prism-control-plane-change-studio.pen`

### 2. Finish the foundations layer

Current gap:

- `Prism Foundations` exists visually, but foundations are not yet a full reusable token and contract source

Need to add:

- spacing scale board
- typography hierarchy board
- radius, border, elevation, and density rules
- semantic status system
- focus state system
- motion rules and reduced-motion rules
- compact vs standard density rules
- light/dark and i18n stress examples where relevant

### 3. Finish the shared workflow patterns

Current gap:

- shared shell primitives exist
- workflow patterns are still mostly implicit inside individual workspace screens

Need to add dedicated shared pattern boards for:

- `Inspect -> Explain -> Change -> Observe`
- `Detect -> Investigate -> Assign -> Resolve`
- `Create -> Validate -> Review -> Publish -> Watch -> Rollback`
- `Connect -> Verify -> Rotate -> Refresh -> Retire`
- `Replay -> Compare -> Save Lens -> Share`

### 4. Finish component state variants

Need to add state variants for:

- loading
- empty
- error
- stale or lagging
- disconnected
- compare mode
- draft mode
- degraded source mode

These should exist for:

- context bars
- KPI and signal cards
- dense tables
- trace blocks
- inspector sections
- workbench panels

### 5. Freeze naming and taxonomy

Need to add and document one stable taxonomy for:

- object names
- workspace names
- source mode labels
- action labels
- status labels
- change stages
- signal severity
- ownership states

This avoids later drift between prototype copy, `.pen` components, and frontend implementation.

## P1: Must Be Added to Make the Product Operationally Complete

These are the additions that turn the design from a strong prototype into a usable control-plane specification.

## Command Center Additions

Current Command Center is directionally correct, but still needs:

- signal severity model: critical, degraded, watch, informational
- acknowledgment and assignment states
- linked investigation cards
- watch-window stack with freshness indicators
- source freshness and ingestion lag indicators
- ownership and runbook hooks
- environment comparison mode
- trend strip for change-related regressions
- system posture block that clearly separates runtime truth from external evidence

## Traffic Lab Additions

Traffic Lab is the highest-value workspace and still needs the deepest additions.

### Request debugger additions

- stage-by-stage request timeline with timestamps
- explicit transform diff panel
- attempt chain compare
- request baseline compare
- replay with current route draft
- replay with previous config version
- trace provenance and source provenance
- saved lens support

### Investigation additions

- pin evidence into an investigation
- compare current failure vs known-good baseline
- compare pre-change vs post-change windows
- group by pattern or rejection reason
- promote a session or pattern into a signal
- shareable investigation state

### Query and browsing additions

- richer search syntax
- saved filters
- reusable facets
- grouping modes
- request-to-route / provider / auth / tenant pivot actions

## Provider Atlas Additions

Provider Atlas still needs to become the runtime entity graph for providers and auth.

Need to add:

- capability matrix with provider-model-API surface truth
- credential and auth-profile relationship graph
- health history and instability trend
- circuit breaker state and failure windows
- provider weight and routing participation visibility
- rotation board for credentials and auth bindings
- region and policy overlay
- linked routes and linked changes
- provider disable / retire / suspend distinctions
- upstream console and external evidence links

## Route Studio Additions

Route Studio still needs:

- scenario library
- matcher builder variants
- historical hit/fallback overlay
- route version history
- compare current draft vs published route
- blast-radius by tenant, model, provider, and geography
- explain history
- publish gate tied into Change Studio
- rollback target visibility
- policy templates and starter profiles

## Change Studio Additions

Change Studio already covers a lot, but to be complete it still needs:

- object history and actor trail
- clone / import / export flows
- semantic diff and raw diff side by side
- staged rollout controls
- change approval model
- publish freeze state
- watch-window ownership
- rollback recommendation and rollback evidence
- grouped related changes
- object dependency explorer
- bulk actions for selected records
- stronger delete semantics: disable, archive, retire, hard delete

### Object-family additions

Need full editor patterns for:

- providers
- auth profiles
- route profiles
- route rules
- auth keys
- tenant policies
- data sources
- alert policies

## P1 Cross-Workspace Additions

These are product capabilities every mature control plane needs.

### 1. Global command surface

Need to add:

- command palette
- jump to entity
- jump to last viewed session
- replay action
- compare action
- publish action
- rollback action
- open external deep link

### 2. Global search

Need to search across:

- request ids
- providers
- auth profiles
- route profiles
- tenants
- changes
- investigations
- signals

### 3. Saved views and saved lenses

Need:

- saved Traffic Lab filters
- saved Change Studio registries
- saved Provider Atlas views
- shared team views

### 4. Ownership and collaboration

Need:

- assign owner
- acknowledge signal
- pin note
- attach runbook
- attach ticket or incident link
- share investigation URL

### 5. Auditability

Need:

- who changed what
- why it changed
- when it changed
- what was observed after publish
- who acknowledged or resolved a signal

### 6. i18n completion

Need:

- full message-key map
- pseudo-localization pass
- mixed-language stress layouts
- locale-aware time and number formatting
- untranslated technical entity policy

### 7. accessibility completion

Need:

- keyboard path for all major flows
- visible focus system
- skip-link and landmark structure
- contrast validation for every dense dark panel
- state not conveyed by color alone

## P2: Backend and API Additions Needed for the Full Product

These are the additions needed if the redesign is implemented as a real product rather than a prototype shell.

## Runtime investigation capabilities

Need backend support for:

- request-session keyed joins across logs, route explain, provider, and auth context
- request correlation ids and trace ids exposed consistently
- replay and compare APIs
- baseline comparison APIs
- grouped failure pattern APIs

## Signal and investigation model

Need backend support for:

- signal detection
- signal acknowledgment and ownership
- investigation persistence
- pinned evidence persistence
- note and annotation persistence
- comparison session persistence

## Change model

Need backend support for:

- first-class change object ids
- staged publish metadata
- watch-window metadata
- rollout status
- rollback targets
- richer object history and semantic diff endpoints

## Config registry model

Need backend support for:

- object-family inventory endpoints
- dependency graph endpoints
- history endpoints
- clone/import/export helpers
- safer destructive action endpoints
- audit trail endpoints

## Provider and auth runtime detail

Need backend support for:

- historical health windows
- circuit-breaker visibility
- auth rotation posture
- provider capability inventory
- route participation insight

## Data source registry

Need backend support for:

- typed data source definitions
- health and freshness state
- capability declarations
- deep-link templates
- workspace scope mapping

## P2 Integrations That Should Be Designed for Now

These do not all need to ship first, but the product structure should leave room for them.

### Observability

- SLS
- OTLP traces
- Tempo / Jaeger
- Prometheus
- ClickHouse / warehouse analytics
- native file or audit backends

### Workflow

- Jira / Linear
- Slack / Feishu
- PagerDuty
- Git-based config review
- runbook registry

### Provider-side tools

- upstream vendor consoles
- auth/account consoles
- rate-limit or billing references

## P3: Delivery and Verification Additions

These are needed to make the redesign durable instead of a one-time visual exercise.

### 1. Repo-local asset completeness

Need to add:

- canonical `.pen` files
- export map
- naming registry
- shared prompts for each workspace
- handoff notes for frontend implementation

### 2. Prototype-to-React mapping

Need to add:

- component inventory map
- shell store model
- URL state contract
- query surface map
- source-mode model
- object model map

### 3. QA gates

Need to enforce:

- no overflow or clipped text
- no badge or button pressure on adjacent text
- no contrast regressions
- no unresolved source provenance
- no divergence between shell kit and workspace usage
- i18n expansion checks

### 4. Browser verification

Need to add:

- Playwright coverage for the new shell
- keyboard path coverage
- deep-link state coverage
- live / paused / stale / disconnected visual states

## Suggested Delivery Order

The cleanest order from here is:

### Phase 1: Formalize reuse

- save `.pen` files
- finish foundations
- finish shared workflow boards
- freeze naming and component taxonomy

### Phase 2: Make workspaces specification-complete

- deepen `Traffic Lab`
- deepen `Change Studio`
- finish `Provider Atlas`
- finish `Route Studio`
- tighten `Command Center`

### Phase 3: Add cross-workspace capabilities

- command palette
- global search
- saved lenses
- ownership and runbook model
- audit model

### Phase 4: Add backend north-star capabilities

- investigations
- change objects
- signal engine
- source registry
- compare and replay joins

### Phase 5: Implementation readiness

- React component map
- query and store model
- browser verification
- rollout strategy

## Minimum Shortlist If We Had To Prioritize Hard

If Prism had to choose only the highest-value remaining additions, they should be:

1. save the `.pen` workspaces and formal foundations
2. complete the `Traffic Lab` request debugger and compare flow
3. complete the `Change Studio` object-history and publish-watch loop
4. add global search, command actions, and saved lenses
5. design the typed data-source registry for hybrid and external evidence
6. design first-class signal and investigation objects

That set would unlock most of the real control-plane value without dragging legacy page models forward.
