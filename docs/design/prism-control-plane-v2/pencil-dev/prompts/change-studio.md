# Change Studio Prompt

Design the `Change Studio` workspace for structured configuration management in Prism.

Goals:

- treat config work as governed change management, not raw form editing
- support query, create, edit, disable, delete review, publish, observe, and rollback
- keep impact visibility and guardrails visible during editing

Include:

- config registry
- object detail workbench
- dependency and blast-radius review
- version trail
- structured editor patterns for provider, auth profile, and route profile
- staged publish flow: preflight, review, canary, observe
- watch window and rollback criteria

Design constraints:

- config richness should not collapse into modal-heavy CRUD
- destructive actions require explicit context and safeguards
- raw configuration can exist as an advanced escape hatch, but should not dominate the default path
- leave room for evidence from Prism runtime and external analytics
- use explicit column and panel structure in dense regions; do not rely on floating badges, pills, or actions beside unconstrained text
- keep inspector content in key/value or stacked action structures rather than long prose
- keep contrast comfortably above the minimum for dark, data-dense UI
- the final screen must satisfy [QUALITY-GATES.md](/Users/qiufeng/work/proxy/prism/docs/design/prism-control-plane-v2/pencil-dev/QUALITY-GATES.md)

Reference files:

- `docs/design/prism-control-plane-v2/prototype.html?screen=change-studio`
- `docs/design/prism-control-plane-v2/config-crud-model.md`
- `docs/design/prism-control-plane-v2/north-star-model.md`
