# Provider Atlas Prompt

Design the `Provider Atlas` workspace for Prism as the provider, capability, auth, and health control surface.

Goals:

- make provider posture inspectable at a glance
- support comparison across providers, regions, model coverage, and auth status
- connect provider state back to routing decisions and live incidents

Include:

- provider roster or matrix
- capability and model coverage view
- auth posture and credential state
- health and latency posture
- links to impacted routes and active investigations

Constraints:

- treat providers as operational entities, not simple CRUD records
- allow future external telemetry inputs
- avoid burying critical health posture inside drawers or modals
