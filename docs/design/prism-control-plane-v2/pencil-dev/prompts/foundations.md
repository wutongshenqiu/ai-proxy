# Prism Foundations Prompt

Use the current Prism control-plane prototype as the visual and interaction baseline.

Create a reusable foundation layer for a data-dense AI gateway control plane.

Requirements:

- define variables for color, spacing, radius, stroke, elevation, typography, and status semantics
- preserve the current control-plane direction: dark shell, dense information, clear signal hierarchy
- optimize for operations workflows, not marketing layouts
- support English and Chinese shell copy without layout breakage
- prioritize shell primitives used across all workspaces

Reusable components to create:

- app shell sidebar
- global context bar
- workspace header
- KPI card
- segmented filter row
- dense table row
- inspector section
- status pill
- primary action button
- secondary action button

Constraints:

- requests, models, provider IDs, and technical identifiers should remain code-like and compact
- color cannot be the only state signal
- layouts should support future integrations like SLS or external observability evidence

Reference files:

- `docs/design/prism-control-plane-v2/prototype.html`
- `docs/design/prism-control-plane-v2/prototype.css`
- `docs/design/prism-control-plane-v2/README.md`
