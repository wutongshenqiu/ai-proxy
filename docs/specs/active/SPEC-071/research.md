# Research: Prism Dashboard UX System & Prototype-First Redesign

| Field     | Value                                               |
|-----------|-----------------------------------------------------|
| Spec ID   | SPEC-071                                            |
| Title     | Prism Dashboard UX System & Prototype-First Redesign |
| Author    | Codex                                               |
| Status    | Draft                                               |
| Created   | 2026-03-16                                          |
| Updated   | 2026-03-17                                          |

## Background

The user asked for a redesign of Prism's UI panel with reference to `cliproxy-manager`, mainstream gateway control planes, and a Figma-oriented workflow. They also asked that implementation should not start until a prototype is available.

## Research Questions

1. What patterns do adjacent AI/API gateway control planes emphasize?
2. What does the CLIProxyAPI management UI suggest about operator expectations in this space?
3. What Figma-native practices best support a prototype-first handoff?
4. How should the redesign stay extensible for future analytics and observability systems such as SLS?
5. If we ignore current backend limits, what higher-level product objects do mature observability/control planes converge on?

## Findings

Additional durable benchmark note:

- [gateway-benchmark-analysis.md](/Users/qiufeng/work/proxy/prism/docs/design/prism-control-plane-v2/gateway-benchmark-analysis.md)
- [remaining-additions-roadmap.md](/Users/qiufeng/work/proxy/prism/docs/design/prism-control-plane-v2/remaining-additions-roadmap.md)

### 1. Adjacent control planes center analytics, runtime truth, and drill-down

Official gateway products consistently prioritize:

- environment or control-plane scoping
- analytics and traffic drill-down
- runtime health visibility
- configuration and policy management from the same shell

This appears across Kong, Tyk, Gravitee, and Apigee documentation. The common pattern is not "settings pages first"; it is "runtime posture first, then drill into entities and policy."

Implication for Prism:

- Prism should lead with a runtime-first workspace shell instead of a flat page list.
- Global context such as environment, time range, provider, tenant, and live state should persist across workspaces.

### 2. CLIProxyAPI's management UI validates the need for one operator console

CLIProxyAPI's official management UI positions itself as a web-based management center, not just a config editor. The associated GitHub project also frames the UI as a way to simplify configuration changes and runtime monitoring.

Implication for Prism:

- Prism should not treat runtime monitoring and configuration as separate products.
- Auth, providers, traffic, routing, and rollout review should live inside one cohesive control plane.

### 3. Figma-native variables, modes, variants, and Dev Mode are enough for handoff

Recent Figma guidance is clear on four useful primitives:

- variables and collections for design tokens
- modes for theme or density variants
- component variants and properties for reusable UI primitives
- Dev Mode and ready-for-dev statuses for controlled handoff

Implication for Prism:

- The redesign should define tokens and component taxonomy before code.
- The eventual Figma file should mirror the prototype's shell, workspaces, and inspector patterns instead of being a loose visual concept.

### 4. Prism should design for multi-source investigation, not only built-in analytics

Prism's own completed specs already point toward a broader observability model:

- SPEC-041 explicitly left room for future log backends such as SLS and SQLite
- SPEC-040 explicitly called out an OpenTelemetry export path for correlation with external tracing systems

That means the redesign should avoid assuming all investigation stays inside one native in-memory store forever.

Implication for Prism:

- the shell should remain stable while sources evolve
- workspaces should support native, hybrid, and external evidence modes
- external systems should appear as typed data sources and deep links, not as random vendor-specific pages

### 5. Mature observability tools converge on investigations, grouped evidence, and ownership

Across Grafana, Datadog, Sentry, and Apigee, the stronger pattern is not just “more pages”.

The stronger pattern is:

- requests or events are treated as inspectable sessions
- anomalies can be grouped into issues, patterns, or investigations
- evidence can combine logs, metrics, traces, diffs, and notes
- operators can compare time windows and preserve findings
- serious runtime problems carry owner, status, and linked change context

Implication for Prism:

- Prism should eventually support first-class investigation objects
- `Traffic Lab` should evolve from a log table into an investigation surface
- `Command Center` should prioritize signals and active investigations, not only KPI cards
- `Change Studio` should link changes to investigations and observation evidence

## Recommendations

- Redesign Prism around a shared shell plus workflow workspaces: `Command Center`, `Traffic Lab`, `Provider Atlas`, `Route Studio`, and `Change Studio`.
- Use one interaction model everywhere: global context bar, main workspace canvas, unified inspector, and embedded workbench for long flows.
- Build the prototype outside the production app first, then convert it into React primitives.
- Treat the current backend as a strong primitive layer, not as the upper bound on the product model.
- Before reusing any shell or workspace patterns, review the code-linked benchmark in [gateway-benchmark-analysis.md](/Users/qiufeng/work/proxy/prism/docs/design/prism-control-plane-v2/gateway-benchmark-analysis.md).
- Use [remaining-additions-roadmap.md](/Users/qiufeng/work/proxy/prism/docs/design/prism-control-plane-v2/remaining-additions-roadmap.md) as the complete checklist for what still must be added before the redesign is implementation-ready.
- In Figma, use native variable collections, component variants, and ready-for-dev sections instead of ad hoc frame copies.
- Design workspace composition so future data sources like SLS can plug into Traffic Lab and Command Center without forcing a new navigation model.
- Model the product around requests, investigations, entities, signals, and changes rather than around today's page inventory.

## References

- CLIProxyAPI official repository: `https://github.com/router-for-me/CLIProxyAPI`
- CLIProxyAPI Web UI docs: `https://help.router-for.me/management/webui.html`
- CLIProxyAPI management center repository: `https://github.com/router-for-me/Cli-Proxy-API-Management-Center`
- Kong AI Gateway docs: `https://developer.konghq.com/ai-gateway/`
- Kong Advanced Analytics: `https://konghq.com/products/kong-konnect/features/advanced-analytics`
- Tyk dashboard analytics docs: `https://tyk.io/docs/5.2/tyk-dashboard-analytics/`
- Gravitee overview dashboard docs: `https://documentation.gravitee.io/apim/analyze-and-monitor-apis/dashboards/overview-dashboard`
- Apigee analytics overview: `https://docs.cloud.google.com/apigee/docs/api-platform/analytics/api-analytics-overview`
- Apigee Trace / Debug: `https://cloud.google.com/apigee/docs/api-platform/debug/trace`
- Grafana Investigations: `https://grafana.com/docs/grafana/latest/explore/simplified-exploration/investigations/`
- Datadog Log Explorer: `https://docs.datadoghq.com/logs/explorer/`
- Sentry Issue Details: `https://docs.sentry.io/product/issues/issue-details/`
- Figma variables overview: `https://help.figma.com/hc/en-us/articles/14506821864087-Overview-of-variables-collections-and-modes`
- Figma variable modes: `https://help.figma.com/hc/en-us/articles/15343816063383-Modes-for-variables`
- Figma variants: `https://help.figma.com/hc/en-us/articles/360056440594-Create-and-use-variants`
- Figma Dev Mode: `https://help.figma.com/hc/en-us/articles/15023124644247-Guide-to-Dev-Mode`

## Open Questions

- [ ] How much of Prism's eventual implementation should preserve the current routing structure versus moving to a typed workspace router?
- [ ] Should the first production implementation ship only the new shell plus one or two migrated workspaces, or switch the whole dashboard at once?
