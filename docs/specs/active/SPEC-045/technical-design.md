# Technical Design: Model Rewrite Rules

| Field     | Value          |
|-----------|----------------|
| Spec ID   | SPEC-045       |
| Title     | Model Rewrite Rules |
| Author    | Claude          |
| Status    | Active         |
| Created   | 2026-03-13     |
| Updated   | 2026-03-13     |

## Overview

Add a `model_rewrites` list to the routing config section. Each rule has a glob `pattern` and a `target` model name. During dispatch, before provider resolution, the requested model is checked against rules in order; the first match rewrites the model name.

## Config Schema

```yaml
routing:
  model_rewrites:
    - pattern: "gpt-4"
      target: "gpt-4-turbo"
    - pattern: "claude-*"
      target: "claude-sonnet-4-20250514"
```

## Implementation

### Config (`crates/core/src/config.rs`)
- Add `ModelRewriteRule { pattern: String, target: String }` struct
- Add `model_rewrites: Vec<ModelRewriteRule>` to `RoutingConfig`
- Add `resolve_model_rewrite(&self, model: &str) -> Option<&str>` method

### Dispatch (`crates/server/src/dispatch.rs`)
- Apply rewrite after model ACL check, before cache lookup and provider resolution
- Record original and rewritten model in debug info

### Dashboard API
- Expose rewrite rules in routing GET/PATCH endpoints

## Task Breakdown

- [x] Add ModelRewriteRule to config
- [x] Add resolve_model_rewrite method
- [x] Apply rewrite in dispatch
- [x] Add debug header support
- [x] Dashboard API exposure
- [x] Tests
