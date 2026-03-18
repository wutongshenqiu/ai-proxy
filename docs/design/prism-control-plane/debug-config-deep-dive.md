# Debug & Config Deep Dive

This note ties together three things:

1. what Prism already supports in the current dashboard
2. what the standalone prototype should absorb and improve
3. which official gateway/control-plane patterns are worth borrowing

## 1. Prism Already Has Strong Debug and Config Primitives

The current dashboard already exposes more operational power than the surface layout suggests.

### Debugging primitives already in code

- `Request Logs` supports multi-field filtering, live mode, URL-linked selection, and detail drill-down.
- `LogDrawer` already shows:
  - request overview
  - provider / credential / tenant context
  - token usage
  - retry timeline
  - request body
  - upstream request body
  - response body or stream preview
- `Replay` already exposes route explanation with:
  - selected route
  - model resolution
  - candidate scoring
  - alternates
  - rejections
- `RoutePreview` already gives a lighter-weight preview inside the routing page.
- `Providers` already exposes:
  - health check
  - fetch models
  - presentation preview
- `Auth Profiles` already exposes managed auth operations:
  - token connect
  - browser OAuth
  - device flow
  - local auth import

Implication:

- Prism does not need a generic “logs page” and a separate “replay page” forever.
- It already has the ingredients for a proper debug workbench.

### Configuration primitives already in code

- `Config & Changes` already exposes:
  - runtime snapshot
  - raw YAML editor
  - validation
  - optimistic concurrency via config version hash
  - apply flow
- `System` already exposes config reload and provider health.
- `Providers` and `Auth Profiles` already behave like structured config editors for specific domains.

Implication:

- Prism does not need a single raw YAML page as the primary config surface.
- It already has enough domain-aware editing behavior to support a real publish workflow.

## 2. What The New Prototype Should Do With Those Capabilities

### A. Debug should become one linked workflow

Recommended flow:

`Signal -> Request Session -> Route Explain -> Provider/Auth Context -> Replay/Compare`

That means:

- `Traffic Lab` becomes the primary entry point for debugging
- request selection should open a rich session view, not just a drawer
- route explanation should be one click away from the selected request
- provider health and auth posture should be reachable from the same debugging context
- comparison with a successful baseline should be a first-class action

Prototype consequence:

- the new `Debug workbench` in `Traffic Lab` should absorb current `Request Logs`, `LogDrawer`, and `Replay`
- current `Replay` can survive as a deep-linkable advanced state, but not as an isolated top-level page forever

### B. Config should become a staged operator workflow

Recommended flow:

`Edit -> Validate -> Review Impact -> Publish -> Observe -> Rollback`

That means:

- structured editors should be the default path
- raw YAML remains an escape hatch, not the main doorway
- validation and conflict handling should stay visible in the same workspace
- publish needs staged rollout and watch-window concepts
- changes must link back to runtime signals after publication

Prototype consequence:

- `Change Studio` should absorb current `Config & Changes`, config reload semantics from `System`, and domain editing intent from `Providers` / `Auth Profiles`
- version hash, conflict detection, secret preservation, and blast radius all need visible UI affordances

## 3. Best Practices Worth Borrowing

The references below are official docs or official product pages, not random blog posts.

| Source | Pattern | Prism implication |
|--------|---------|-------------------|
| Kong AI Gateway + Konnect Advanced Analytics | traffic segmentation, analytics-first drill-down | make filterable analytics the main door into debugging, not a secondary page |
| Google Apigee Trace / Debug | step-by-step request pipeline visibility | show request transformation, routing, upstream call, retry, and failure chain in one timeline |
| Tyk Dashboard Analytics | strong slicing by API/key/time | preserve multi-dimensional filtering as a global shell concern |
| Gravitee overview dashboard | top-level operational posture with drill-down | keep the homepage focused on health, change, and anomaly signals |
| Figma variables + Dev Mode | design tokens and dev handoff discipline | keep the redesign systematic so debug/config flows do not drift visually as complexity grows |

## 4. Concrete UX Recommendations For Prism

### Debug Workbench

Minimum tabs:

- `Session`
- `Route Explain`
- `Upstream Transform`
- `Retry Tree`
- `Replay / Compare`

Important details:

- preserve filters and selected request in the URL
- keep request identity, tenant, provider, credential, and model pinned
- use a side inspector for the current object, not a new modal for every detail jump
- let operators compare failing and healthy traces side by side

### Change Studio

Minimum stages:

- `Edit`
- `Validate`
- `Review`
- `Canary`
- `Observe`
- `Rollback`

Important details:

- default to structured diffs and impact summaries
- keep raw YAML available in a secondary panel
- surface config version hash and conflict state
- link post-publish watch windows back to Command Center and Traffic Lab

### Internationalization

For a gateway control plane, do not translate technical identifiers:

- request ids
- provider ids
- model ids
- config keys
- raw error payloads

Do translate:

- navigation
- shell actions
- section titles
- empty states
- validation messages
- publish / rollback language

## 5. Reference URLs

- CLIProxyAPI official repo: `https://github.com/router-for-me/CLIProxyAPI`
- CLIProxyAPI Web UI docs: `https://help.router-for.me/management/webui.html`
- Kong AI Gateway: `https://developer.konghq.com/ai-gateway/`
- Kong Advanced Analytics: `https://konghq.com/products/kong-konnect/features/advanced-analytics`
- Tyk Dashboard Analytics: `https://tyk.io/docs/5.2/tyk-dashboard-analytics/`
- Gravitee overview dashboard: `https://documentation.gravitee.io/apim/analyze-and-monitor-apis/dashboards/overview-dashboard`
- Apigee Trace / Debug docs: `https://cloud.google.com/apigee/docs/api-platform/debug/trace`
- Apigee analytics overview: `https://cloud.google.com/apigee/docs/api-platform/analytics/analytics-dashboard`
- Figma variables: `https://help.figma.com/hc/en-us/articles/14506821864087-Overview-of-variables-collections-and-modes`
- Figma Dev Mode: `https://help.figma.com/hc/en-us/articles/15023124644247-Guide-to-Dev-Mode`

## 6. Recommended Next Prototype Iteration

If we keep deepening the prototype, the next useful layer is not more pages. It is deeper flows:

1. turn the `Traffic Lab` debug workbench into a real multi-state request session
2. add structured config editors for `Provider`, `Route`, and `Auth Profile` inside `Change Studio`
3. add explicit cross-links:
   - request -> route explain
   - request -> provider health
   - provider -> auth profile
   - change -> post-publish watch window
4. add one focused “compare before/after change” path for incident review
