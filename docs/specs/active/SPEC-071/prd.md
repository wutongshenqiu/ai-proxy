# PRD: Prism Dashboard UX System & Prototype-First Redesign

| Field     | Value                                               |
|-----------|-----------------------------------------------------|
| Spec ID   | SPEC-071                                            |
| Title     | Prism Dashboard UX System & Prototype-First Redesign |
| Author    | Codex                                               |
| Status    | Draft                                               |
| Created   | 2026-03-16                                          |
| Updated   | 2026-03-17                                          |
| Parent    | SPEC-065, SPEC-066                                  |

## Problem Statement

Prism's dashboard has become functionally rich, but the UX model is still page-list driven. Operators can inspect logs, manage providers, change routing, and manage auth, yet those workflows are split across separate pages with separate filters, separate local state, and separate modal patterns.

That creates four operator-facing problems:

1. The dashboard reflects backend truth better than before, but it still does not feel like one control plane.
2. Monitoring, configuration, identity, and rollout actions require too much page hopping and too little shared context.
3. Interaction patterns are inconsistent: some flows use tables, some use modals, some use drawers, and global context is weak.
4. Design decisions are being made directly in code instead of being stabilized in a prototype and a design system first.

The user explicitly wants a prototype before implementation and does not want compatibility constraints to limit the redesign. This work should therefore start with a prototype-first UX system, not with incremental styling on the current React pages.

## Goals

- Redesign Prism around operator workflows rather than around the current page inventory.
- Produce a high-fidelity prototype before any production dashboard implementation work.
- Establish a Figma-ready design system with tokens, component taxonomy, and handoff rules.
- Replace modal-heavy page interactions with a consistent shell based on global context, workspace canvas, and unified inspector/workbench patterns.
- Make it easier to manage UI interactions through a clear state model: global context, workspace state, inspector state, and workflow state.
- Promote debugging into a first-class request session workflow that connects logs, route explain, provider state, and auth posture.
- Promote configuration into a staged operator workflow that connects edit, validate, impact review, publish, and observe.
- Keep the shell extensible so future analytics and workflow systems such as SLS, OTLP traces, warehouses, and incident tooling can plug into workspaces without changing the core navigation model.
- Define a north-star product model around requests, investigations, runtime entities, signals, and changes, instead of mirroring the current backend page inventory.
- Make configuration management rich enough to cover object discovery and lifecycle verbs such as query, create, edit, clone, disable, delete, diff, history, and rollback.

## Non-Goals

- Preserving the current dashboard information architecture for compatibility reasons.
- Incrementally skinning the current pages without changing the interaction model.
- Implementing the production React dashboard in this spec phase.
- Finalizing every API contract required by the eventual implementation.

## User Stories

- As a gateway operator, I want to monitor traffic, provider health, auth coverage, and config drift from one consistent shell so that I can react faster.
- As a gateway operator, I want route explanation, replay, provider health, and auth management to feel connected instead of spread across unrelated pages.
- As a maintainer, I want a prototype and design system before implementation so that frontend work can be split safely without design drift.
- As a designer or engineer using Figma, I want clear variables, component variants, and ready-for-dev structure so that implementation handoff is deterministic.

## Success Metrics

- The prototype covers the top control-plane workflows: overview, live traffic, provider/runtime inventory, route reasoning, and config changes.
- At least 80% of high-frequency operator actions can be expressed without opening a generic blocking modal.
- A shared interaction model exists for global filters, detail inspection, and long-running workflows.
- The prototype package includes Figma handoff guidance precise enough for implementation without inventing a new design language in code.
- The prototype shows at least one end-to-end debug path: `request list -> request session -> route explain -> provider/auth context`.
- The prototype shows at least one end-to-end config path: `structured edit -> validate -> publish -> observe`.
- The prototype and design docs define a data-source model that can support `Prism runtime`, `native analytics`, and `external analytics` without inventing a separate shell for each integration.
- The prototype and docs describe at least one first-class `Investigation` flow that is not constrained by current backend routes.
- The prototype and docs describe a config registry model with explicit CRUD and non-CRUD lifecycle verbs for major object families.

## Constraints

- Compatibility with the current dashboard layout is explicitly not a constraint.
- The redesign must build on runtime-truth principles from SPEC-065 and SPEC-066.
- The UX must remain data-dense and operationally useful, not marketing-like.
- The design system must map cleanly into CSS variables, component variants, and inspectable states.
- Extensibility is required: the shell must not assume all observability and analytics stay inside Prism forever.
- The old dashboard may remain in production until the new control plane is fully ready.
- Production release should switch to the new control plane in one cutover, not workspace by workspace.

## Open Questions

- [ ] Should Providers, Protocols, and Models remain separate implementation routes but share one higher-level "Runtime Catalog" workspace in the UI?
- [ ] Should config edits move to a staged draft/review/publish flow, or remain immediate mutations with stronger confirmation UX?
- [ ] Should the eventual implementation adopt a typed router and query layer overhaul, or keep the current router and change only interaction primitives?

## Design Decisions

| Decision | Options Considered | Chosen | Rationale |
|----------|--------------------|--------|-----------|
| Delivery order | Implement first vs prototype first | Prototype first | Matches the user's request and reduces design churn |
| IA model | Page list vs workflow workspaces | Workflow workspaces | Better fits control-plane mental models |
| Interaction model | Mixed modal/drawer/page patterns vs unified shell | Unified shell | Makes state and navigation easier to reason about |
| Detail display | Per-page custom detail UI vs shared inspector | Shared inspector | Reduces repeated patterns and preserves context |
| Workflow editing | Blocking modals vs embedded workbench/sheets | Embedded workbench/sheets | Better for complex provider, routing, and config flows |
| Design handoff | Code-first styling vs Figma-native system | Figma-native system with code token mapping | Creates a cleaner bridge between prototype and implementation |
| Implementation posture | Incremental page migration vs greenfield control-plane shell | Greenfield shell with full cutover after readiness | Avoids legacy UX baggage while keeping one clean production switch |
