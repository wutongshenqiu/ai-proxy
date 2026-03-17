# Shell Kit Prompt

Design the reusable shell kit for Prism as a control-plane system, not a page collection.

Goals:

- create reusable shell primitives for all major workspaces
- enforce a consistent runtime-first interaction model
- keep the system dense, structured, and expandable

Include:

- left navigation shell
- top context bar with environment, source mode, time window, and live state
- workspace header pattern
- KPI row pattern
- inspector section pattern
- action stack pattern
- dense table row pattern
- status pill family

Constraints:

- industrial technical tone
- one accent color only
- identifiers and metrics may use monospace
- explanations should use readable sans text
- avoid modal-first composition

Output should be reusable enough for:

- Traffic Lab
- Change Studio
- Route Studio
- Provider Atlas
- Command Center
