# Pencil File Conventions

This note defines where Prism control-plane design files should live and how they should be named.

The goal is stability under repeated edits.

## Principles

- Keep source design files in `docs/`, because they are reviewable project artifacts.
- Keep generated PNG exports in `output/`, because they are binary review snapshots.
- Use stable canonical filenames for the current truth.
- Use separate exploration files only when a direction materially diverges.
- Let Git history carry normal iteration; do not encode every edit into the filename.

## Directory Layout

```text
docs/design/prism-control-plane/
├── README.md
├── prototype.html
├── prototype.css
├── prototype.js
└── pencil-dev/
    ├── README.md
    ├── CONVENTIONS.md
    ├── prompts/
    └── workspaces/
        ├── prism-control-plane-foundations.pen
        ├── prism-control-plane-traffic-lab.pen
        ├── prism-control-plane-change-studio.pen
        ├── prism-control-plane-route-studio.pen
        └── prism-control-plane-explorations.pen
```

Generated review assets belong here:

```text
output/pencil/prism-control-plane/
├── traffic-lab-overview--latest.png
├── traffic-lab-sessions-panel--latest.png
├── traffic-lab-trace-panel--latest.png
└── traffic-lab-inspector--latest.png
```

## Naming Rules

Use lowercase kebab-case.

Use `prism-control-plane-` as the prefix for durable `.pen` files.

Good canonical names:

- `prism-control-plane-foundations.pen`
- `prism-control-plane-traffic-lab.pen`
- `prism-control-plane-change-studio.pen`
- `prism-control-plane-route-studio.pen`

Good exploration names:

- `prism-control-plane-traffic-lab--explore-a.pen`
- `prism-control-plane-change-studio--explore-b.pen`
- `prism-control-plane-explorations.pen`

Good export names:

- `traffic-lab-overview--latest.png`
- `traffic-lab-trace-panel--latest.png`
- `change-studio-publish-flow--latest.png`

Avoid these patterns:

- `final.pen`
- `new.pen`
- `copy.pen`
- `v3.pen`
- raw canvas IDs such as `IZFA1.png`

## Review Gate

Stable naming is not enough.

Before updating any canonical `--latest` export, pass the workspace through:

- [QUALITY-GATES.md](/Users/qiufeng/work/proxy/prism/docs/design/prism-control-plane/pencil-dev/QUALITY-GATES.md)

That means:

- full-screen layout scan
- panel-level layout scans for dense or interactive zones
- focused PNG review for the same high-risk areas
- syncing any structural fix back into the reusable pattern, not only the current screen

## Revision Policy

For the current accepted design, update the canonical `.pen` file in place.

Reason:

- repeated edits are normal
- Git already records the revision history
- stable paths are easier to link from specs and docs

Create a separate exploration file only when:

- the layout direction changes materially
- two approaches need side-by-side review
- the experiment is intentionally not yet the new truth

Once an exploration wins:

1. merge it back into the canonical file
2. refresh the `--latest` PNG exports
3. remove or archive the exploration if it is no longer useful

## Screen Split Strategy

For Prism, use one `.pen` file per major workspace once the shell primitives are stable.

Why:

- smaller diffs
- fewer merge conflicts
- easier MCP targeting
- clearer ownership during repeated revisions

Recommended split:

- `prism-control-plane-foundations.pen`: tokens, shell primitives, cards, pills, tables, inspector blocks
- `prism-control-plane-traffic-lab.pen`: request debugging flows
- `prism-control-plane-change-studio.pen`: config CRUD and publish flows
- `prism-control-plane-route-studio.pen`: route editing and explain flows
- `prism-control-plane-explorations.pen`: rejected or temporary concepts

## Current Review Artifact Mapping

The current first Pencil pass should be treated as the seed for:

- `prism-control-plane-traffic-lab.pen`

Current exported snapshots:

- [traffic-lab-overview--latest.png](/Users/qiufeng/work/proxy/prism/output/pencil/prism-control-plane/traffic-lab-overview--latest.png)
- [traffic-lab-sessions-panel--latest.png](/Users/qiufeng/work/proxy/prism/output/pencil/prism-control-plane/traffic-lab-sessions-panel--latest.png)
- [traffic-lab-trace-panel--latest.png](/Users/qiufeng/work/proxy/prism/output/pencil/prism-control-plane/traffic-lab-trace-panel--latest.png)
- [traffic-lab-inspector--latest.png](/Users/qiufeng/work/proxy/prism/output/pencil/prism-control-plane/traffic-lab-inspector--latest.png)
