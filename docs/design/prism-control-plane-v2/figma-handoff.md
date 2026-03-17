# Figma Handoff for Prism Control Plane V2

This prototype is intended to become a proper Figma file before implementation work begins.

## File Structure

Create one Figma file with these pages:

1. `00 Foundations`
2. `01 App Shell`
3. `02 Command Center`
4. `03 Traffic Lab`
5. `04 Provider Atlas`
6. `05 Route Studio`
7. `06 Change Studio`
8. `07 Components`
9. `08 Prototype Flows`

## Variables

Use native Figma variables, not loose local styles.

Recommended collections:

- `Color / Primitive`
- `Color / Semantic`
- `Spacing`
- `Radius`
- `Elevation`
- `Motion`
- `Density`

Recommended modes:

- `desktop-default`
- `desktop-critical`

The second mode is not a dark theme. It is for high-attention operational states, where surfaces, accents, and emphasis values tighten around warning or incident contexts.

## Component Sets

Build component variants for these primitives first:

- `Shell/Nav Item`
- `Shell/Context Pill`
- `Shell/Workspace Tab`
- `Signal/Metric Card`
- `Signal/Status Chip`
- `Signal/Timeline Row`
- `Data/Table Row`
- `Inspector/Section`
- `Action/Button`
- `Action/Command Item`

Recommended variant properties:

- `state = default | hover | active | selected | disabled`
- `tone = neutral | primary | success | warning | danger`
- `density = compact | comfortable`
- `emphasis = quiet | normal | strong`

## Prototype Variables

Use Figma variables in prototype flows for:

- active workspace
- active environment
- selected request
- selected provider
- live or paused state

That keeps the prototype aligned with the eventual URL and store model in code.

## Internationalization

Plan for bilingual layout from the beginning:

- create at least one `en` and one `zh-CN` prototype flow
- test navigation, topbar pills, buttons, table headers, and inspector titles under longer Chinese copy
- keep request ids, model ids, provider ids, and raw config snippets untranslated
- avoid fixed-width component assumptions for labels or stage names

## Dev Handoff

Use Dev Mode's ready-for-dev workflow instead of verbal handoff:

- Mark only approved frames and components as ready
- Keep annotations on layout behavior, truncation rules, and sticky regions
- Attach implementation notes where data density or responsive collapse matters

## Code Mapping

The implementation should map Figma variables to CSS variables directly.

Suggested naming:

- `color.bg.canvas` -> `--color-bg-canvas`
- `color.bg.panel` -> `--color-bg-panel`
- `color.text.primary` -> `--color-text-primary`
- `color.signal.success` -> `--color-success`
- `radius.panel.lg` -> `--radius-panel-lg`

Component names should also map cleanly into code:

- `Shell/Nav Item` -> `NavItem`
- `Signal/Metric Card` -> `MetricCard`
- `Inspector/Section` -> `InspectorSection`

## Workflow Recommendation

1. Recreate the prototype shell and screen layouts in Figma.
2. Convert repeated patterns into component sets.
3. Apply variables before creating additional screens.
4. Mark approved frames as ready for dev.
5. Only after that, start the React implementation in `web/`.
