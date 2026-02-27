# PRD: Request Cloaking & Payload Rules

| Field     | Value                              |
|-----------|------------------------------------|
| Spec ID   | SPEC-007                           |
| Title     | Request Cloaking & Payload Rules   |
| Author    | AI Proxy Team                      |
| Status    | Completed                          |
| Created   | 2026-02-27                         |
| Updated   | 2026-02-27                         |

## Problem Statement

Different upstream AI providers have varying requirements for request payloads. The Claude API in particular requires specific request shaping when requests originate from third-party clients (non-Claude CLI). Additionally, operators need the ability to set default parameters, override fields, and filter out unsupported fields on a per-model basis.

## Goals

- Claude request cloaking: inject Claude Code system prompt, generate fake `user_id` in metadata, obfuscate sensitive words with zero-width spaces
- Cloaking mode control: Auto (skip for native Claude CLI/Code clients), Always, Never -- configurable per API key entry
- Strict mode: optionally replace the user's system prompt entirely with the cloaking prompt
- Sensitive word obfuscation: insert zero-width spaces to prevent pattern matching on configured words
- Payload rules with three types: `default` (set if missing), `override` (always set), `filter` (remove fields)
- Model-specific rules using glob pattern matching (e.g., `gemini-*`, `gpt-4*`)
- Protocol-specific rules: match by wire protocol (e.g., only apply to `openai` format)
- Dot-separated JSON path notation for nested field access (e.g., `generationConfig.thinkingConfig.thinkingBudget`)

## Non-Goals

- Request body encryption or signing
- Provider-specific payload transformation (handled by translator layer, see SPEC-003)
- Response body modification or cloaking

## User Stories

- As an operator, I want Claude API requests from non-Claude clients to be cloaked so that the API treats them as first-party Claude Code requests.
- As an operator, I want to automatically set a default thinking budget for Gemini models so that users get thinking enabled without having to specify it.
- As an operator, I want to force a specific `reasoning.effort` value for all OpenAI reasoning models so that I control cost.
- As an operator, I want to remove unsupported fields for specific models so that upstream APIs do not reject the request.
- As an operator, I want to obfuscate sensitive company names in messages sent to Claude so that they are not logged by the provider.

## Success Metrics

- Cloaked requests are accepted by Claude API as if from Claude Code
- Payload defaults are set only when the field is absent (do not overwrite user values)
- Payload overrides are always applied regardless of existing values
- Payload filters successfully remove nested fields without affecting sibling fields
- Glob matching correctly selects models (e.g., `gemini-*` matches `gemini-2.5-pro`)

## Constraints

- Cloaking logic is in `crates/core/src/cloak.rs`
- Payload rules logic is in `crates/core/src/payload.rs`
- CloakConfig is defined per `ProviderKeyEntry`, not globally
- Payload rules are applied BEFORE cloaking in the dispatch flow
- Cloaking is only applied when the target format is `Format::Claude`

## Open Questions

- [x] Should cloaking apply before or after payload rules? -- After: payload rules run first, then cloaking modifies the result

## Design Decisions

| Decision | Options Considered | Chosen | Rationale |
|----------|--------------------|--------|-----------|
| Cloak detection | Always cloak, header-based, User-Agent | User-Agent (Auto mode) | Native Claude CLI/Code clients already format requests correctly; only third-party clients need cloaking |
| User ID generation | Static, random, hash-based | Random with optional caching | Looks realistic; caching ensures consistent identity per API key |
| Sensitive word obfuscation | Asterisks, removal, zero-width spaces | Zero-width spaces | Preserves readability for the model while breaking exact string matching |
| Payload rule ordering | Configurable, fixed | Fixed (default -> override -> filter) | Predictable behavior; defaults set baselines, overrides enforce policy, filters clean up |
| Model matching | Exact, regex, glob | Glob | Simple, familiar pattern syntax; covers common use cases without regex complexity |
