# PRD: Model Rewrite Rules

| Field     | Value          |
|-----------|----------------|
| Spec ID   | SPEC-045       |
| Title     | Model Rewrite Rules |
| Author    | Claude          |
| Status    | Active         |
| Created   | 2026-03-13     |
| Updated   | 2026-03-13     |

## Problem Statement

Operators need to remap model names for compatibility with various coding agents and SDKs. For example, mapping `gpt-4` to a specific provider's model, or aliasing a custom model name to an upstream model.

## Goals

- Add configurable model rewrite rules in YAML config
- Support glob pattern matching for source model names
- Apply rewrites before routing so they're transparent to the dispatch pipeline
- Expose rewrite rules in dashboard for visibility
- Make rewrites inspectable via x-debug headers

## Non-Goals

- Complex conditional rewrites based on headers/client (deferred)
- Bidirectional rewrite of response model names (deferred)

## Design Decisions

| Decision | Options | Chosen | Rationale |
|----------|---------|--------|-----------|
| Pattern syntax | Regex, glob, exact | Glob | Consistent with existing model matching throughout codebase |
| Config location | Per-provider, global | Global (routing section) | Rewrites happen before provider selection |
| Evaluation order | First match, all matches | First match | Simple, predictable behavior |
