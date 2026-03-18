# Pencil Workspace Architecture

This file defines the north-star structure for Prism control-plane design work in Pencil Dev.

It intentionally ignores legacy dashboard page boundaries.

The objective is a system that stays coherent as the design evolves over many revisions.

## Design Goal

Prism should be designed as a control-plane shell with reusable primitives and workspace-specific workbenches.

Not as:

- a set of unrelated admin pages
- a collection of modal-heavy CRUD screens
- a one-file mockup with duplicated blocks

## Layer Model

Use a layered design model.

### Layer 1: Foundations

File:

- `prism-control-plane-foundations.pen`

Contains:

- color tokens
- typography tokens
- spacing scale
- radius and stroke rules
- status semantics
- motion rules
- light and dark modes if needed

This file should not contain full product screens.

### Layer 2: Shell Kit

Primary home:

- `prism-control-plane-foundations.pen`

Contains reusable primitives for the control-plane shell:

- left navigation
- top context bar
- workspace header
- KPI cards
- dense filter bars
- inspector sections
- evidence cards
- status pills
- primary and secondary action rows
- table headers and rows

This layer is where most reuse should happen.

### Layer 3: Workspace Files

Files:

- `prism-control-plane-traffic-lab.pen`
- `prism-control-plane-change-studio.pen`
- `prism-control-plane-route-studio.pen`
- future: `prism-control-plane-provider-atlas.pen`
- future: `prism-control-plane-command-center.pen`

Each file should contain one major workspace only.

That keeps diffs smaller and avoids cross-workspace merge noise.

### Layer 4: Explorations

File:

- `prism-control-plane-explorations.pen`

Use this only for materially different directions.

Examples:

- alternate shell density
- alternate inspector model
- alternate change-publish flow

Do not use this file as the default place for routine edits.

## Workspace Contract

Every major workspace should follow the same structural contract.

```text
Workspace Shell
├── Context Bar
├── Workspace Header
├── Signal / KPI Row
├── Main Workbench
│   ├── Primary Zone
│   ├── Secondary Zone
│   └── Embedded Action Strip
└── Inspector Rail
```

This contract is more important than any individual screen content.

It is what keeps the system scalable.

## Dominant Region Rules

Each workspace must have one dominant zone.

Recommended dominant zones:

- `Command Center`: signal grid / runtime posture
- `Traffic Lab`: session + trace investigation workbench
- `Provider Atlas`: provider capability and health matrix
- `Route Studio`: route draft and explain workbench
- `Change Studio`: staged change and watch window workbench

The inspector rail is never the dominant region.

## Reusable Component Families

Design components in families, not as isolated one-off blocks.

### Shell Family

- `Shell / Sidebar`
- `Shell / Context Bar`
- `Shell / Workspace Header`
- `Shell / Empty State`

### Signal Family

- `Signal / KPI Card`
- `Signal / Alert Strip`
- `Signal / Investigation Card`
- `Signal / Status Pill`

### Data Family

- `Data / Dense Table Header`
- `Data / Dense Row`
- `Data / Property List`
- `Data / Comparison Block`

### Workbench Family

- `Workbench / Filter Rail`
- `Workbench / Session List`
- `Workbench / Trace Block`
- `Workbench / Diff Block`
- `Workbench / Publish Step`

### Inspector Family

- `Inspector / Section`
- `Inspector / Linked Entity`
- `Inspector / Health Summary`
- `Inspector / Action Stack`

## Naming Model

Use human-readable semantic names inside `.pen` files.

Prefer:

- `Traffic Lab / Session List`
- `Traffic Lab / Trace Workbench`
- `Inspector / Linked Entities`

Avoid:

- `Group 12`
- `Frame Copy`
- `New Card`

Keep component names stable once they become reusable.

## Extensibility Rules

The design should be ready for more data sources and more operational domains.

Design for:

- runtime truth
- hybrid evidence
- external analytics such as SLS
- future OTLP traces
- future investigation ownership and collaboration

This means:

- source mode belongs in the shell contract
- evidence blocks should be composable
- inspector sections should support linked external systems
- workspaces should not assume a single backend source

## Density Rules

Prism is allowed to be dense, but density must be structured.

Recommended defaults:

- 12-column or equivalent rational grid
- compact data rows
- consistent 8px rhythm
- code-like mono for identifiers
- sans body text for explanations
- one accent color only

Do not mix soft consumer-style cards with hard-edge operational tables in the same workspace.

## State Rules

Every reusable workspace block should be designed with these states in mind:

- loading
- empty
- error
- stale or lagging
- live
- compare mode

If the base component cannot express state, it is not reusable enough.

## Recommended Build Order

1. Foundations
2. Shell kit
3. Traffic Lab
4. Change Studio
5. Route Studio
6. Provider Atlas
7. Command Center

This order keeps the highest-value operator loops first.

## Current Recommendation

Treat the current first Pencil screen as the seed of:

- `prism-control-plane-traffic-lab.pen`

Next practical steps:

1. save the current active Pencil document into the `workspaces/` directory
2. extract shell primitives into a dedicated foundations file
3. rebuild later workspaces against those primitives instead of copying the Traffic Lab frame
