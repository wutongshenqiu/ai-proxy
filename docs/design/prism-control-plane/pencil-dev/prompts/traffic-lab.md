# Traffic Lab Prompt

Design the `Traffic Lab` workspace for investigation-first request debugging in Prism.

Goals:

- make debugging the fastest operator workflow
- support runtime, hybrid, and external evidence modes
- keep filters, request session details, fallback reasoning, upstream transform inspection, and replay controls in one coherent workspace

Include:

- workspace header with global context
- saved lenses or filter presets
- request session list
- active request detail or trace chain
- fallback and retry reasoning
- upstream transform comparison
- replay actions
- right-side inspector for selected entity context

Design constraints:

- this is an operations console, not a notebook
- dense data is acceptable if hierarchy is strong
- prioritize cross-linking from request to route, provider, auth profile, and linked change
- preserve room for future SLS or OTLP evidence panes

Reference files:

- `docs/design/prism-control-plane/prototype.html?screen=traffic-lab`
- `docs/design/prism-control-plane/debug-config-deep-dive.md`
- `docs/design/prism-control-plane/extensibility-model.md`
