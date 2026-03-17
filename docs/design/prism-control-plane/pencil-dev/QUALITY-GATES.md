# Pencil Visual Quality Gates

This file turns the review lessons from the first Prism Pencil iterations into a repeatable acceptance bar.

Use it before calling any workspace "ready for review".

It exists to stop the same feedback loop from repeating:

- text running out of its frame
- floating badges or buttons visually sitting on top of copy
- dark-theme contrast drifting too low
- shell-kit patterns diverging from real workspace usage

## Core Principle

Prism can be dense.

It cannot be visually ambiguous.

Every dense block must still make these things obvious at a glance:

- what the dominant content is
- what is metadata
- what is status
- what is actionable

## Hard Fail Conditions

Do not export or present a screen as current if any of these are true.

### Layout

- Any visible text is clipped, cropped, or leaves its intended frame.
- Any badge, pill, button, or icon overlaps running text.
- Any right-aligned action group shares width with unconstrained prose.
- Dense rows rely on free-floating items instead of explicit columns.
- Inspector content uses long prose where a key/value structure is needed.

### Typography

- Mono is used for explanatory copy instead of identifiers, metrics, or IDs.
- 11px text is used for normal explanatory copy.
- Section headers are so long that they force layout compromises.
- Dense areas use too many font styles at once.

### Contrast

- Normal text falls below WCAG AA contrast for its background.
- Subtle labels are so dim that they only pass mathematically but feel weak in the dark UI.
- Status meaning depends only on color, without text or shape support.

### System Consistency

- The same pattern is built differently in the workspace and in `Shell Kit`.
- Table rows, pills, action rows, or inspector blocks drift away from their canonical structure.
- A one-off visual fix is made in a screen without feeding the same rule back into the reusable pattern.

## Dense Workspace Rules

These are the rules that came directly from the `Traffic Lab` review cycle.

### Session and Table Rows

- Use explicit column structure for dense operational rows.
- Do not combine long narrative text with floating result badges in the same freeform row.
- Keep headers short, usually one or two words.
- Reserve the far-right region for status and timing, not prose.
- If a row needs more explanation, use a secondary line that still respects the column grid.

### Trace and Timeline Blocks

- Timeline steps must show causal flow faster than they show decoration.
- Status pills must not steal width from the main reasoning line.
- Action buttons belong in a dedicated action row, not in the same line as running copy.
- Long technical explanations should be shortened or moved into a separate detail panel.

### Inspector Rails

- Prefer key/value rows for linked entities and operational context.
- Keep the order stable:
  1. current posture
  2. linked entities
  3. next actions
  4. external evidence
- Do not let the inspector become a second main content area.

### KPI and Signal Rows

- KPI rows must support the workspace, not compete with it.
- Avoid large decorative cards above a dense workbench.
- Keep signal wording compact and numerically legible.

## Typography Rules

- Use a sans family for headings, labels, and explanatory text.
- Use mono only for technical identifiers, timings, model names, route IDs, and request/session IDs.
- Avoid using 11px except for compact technical metadata.
- Raise line height before shrinking text size.
- If a label needs too much width, shorten the label before widening the whole system.

## Contrast Rules

Use WCAG AA as the floor, not the target.

For Prism's dark, data-dense UI:

- normal text should meet or exceed `4.5:1`
- weak labels should still remain comfortably readable in context
- primary actions should exceed the minimum by a healthy margin
- important state changes must use color plus text

Practical rule:

- never use gray-on-gray combinations that feel hesitant
- when in doubt, increase text brightness before increasing font weight

## Pre-Review Checklist

Before exporting any `--latest` PNG:

1. Run `snapshot_layout` on the full workspace.
2. Run `snapshot_layout` on the highest-risk panels:
   - dense rows or session lists
   - trace or timeline blocks
   - inspector rails
   - shell-kit variants
3. Export focused PNGs for those same risk panels.
4. Visually inspect the focused PNGs for:
   - clipped text
   - overlap
   - floating controls
   - accidental crowding
   - weak contrast
5. If a fix changes a reusable pattern, update `Shell Kit` too.

## Review Discipline

When feedback says "it still feels awkward", do not only shorten text.

Check the deeper cause first:

- wrong grid
- wrong content hierarchy
- wrong action placement
- wrong component syntax
- wrong density for the job

Structural fixes beat cosmetic fixes.

## Best-Practice Notes Behind These Gates

These gates align with the design direction already chosen for Prism:

- data-dense operational dashboard, not consumer UI
- dark theme with accessible contrast
- drill-down workbench as the dominant content
- minimal modal dependence
- clear separation between narrative copy, status semantics, and actions

They also match the current external guidance we referenced:

- dark, data-dense dashboards need stronger-than-minimum contrast
- enterprise tables benefit from short headers and explicit alignment
- operational UIs should separate status, content, and action zones

Use this file as the default quality bar for every new Prism Pencil workspace.
