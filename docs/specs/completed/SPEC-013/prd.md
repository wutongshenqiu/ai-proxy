# SPEC-013: Model Fallback & Debug Mode

## Problem

1. Clients cannot specify fallback models — when a model is unavailable, the request fails.
2. Routing decisions are opaque, making debugging difficult.

## Goals

- G1: `models` array in request body for fallback chain
- G2: `x-debug: true` request header enables debug info
- G3: Debug response headers: `x-debug-provider`, `x-debug-model`, `x-debug-credential`, `x-debug-attempts`
- G4: `weight` field on `ProviderKeyEntry` for weighted round-robin

## Implementation

- `ParsedRequest` extended with `models` and `debug` fields
- `DispatchRequest` extended with `models` and `debug` fields
- `dispatch()` wraps retry loop in model fallback loop
- `DispatchDebug` struct collects routing info
- `AuthRecord` gains `credential_name` and `weight` fields
- `CredentialRouter::pick()` implements weighted round-robin
- `config.example.yaml` documents `weight` option

## Status

Active — Implementation complete, pending review.
