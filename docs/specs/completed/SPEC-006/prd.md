# PRD: Security & Authentication

| Field     | Value                       |
|-----------|-----------------------------|
| Spec ID   | SPEC-006                    |
| Title     | Security & Authentication   |
| Author    | AI Proxy Team               |
| Status    | Completed                   |
| Created   | 2026-02-27                  |
| Updated   | 2026-02-27                  |

## Problem Statement

The AI Proxy Gateway needs to secure access from clients, protect communication with upstream providers, and enforce request limits. Without authentication, any client could consume upstream API quota. Without TLS, credentials are transmitted in plaintext. Without body size limits, the proxy is vulnerable to resource exhaustion.

## Goals

- Bearer token and x-api-key header authentication for client requests
- Optional authentication bypass when no API keys are configured (open proxy mode)
- TLS/HTTPS support via Rustls for encrypted communication
- CORS support for browser-based API clients
- Request body size limits to prevent resource exhaustion
- Admin endpoints accessible without authentication (read-only)
- Passthrough headers: forward specific upstream response headers to clients

## Non-Goals

- OAuth2 / OIDC / JWT token validation
- Per-key rate limiting or quota management on the proxy side
- IP-based access control or allowlisting
- Mutual TLS (mTLS) client certificate authentication

## User Stories

- As an operator, I want to restrict proxy access to authorized clients via API keys so that only permitted users consume upstream quota.
- As an operator, I want to run the proxy without authentication in development so that I can test quickly without managing keys.
- As an operator, I want to enable TLS so that API keys are not transmitted in plaintext.
- As a frontend developer, I want CORS to be permissive so that I can call the proxy from any browser origin.
- As an operator, I want body size limits so that a malicious client cannot crash the proxy by sending a huge request.
- As an operator, I want admin endpoints to be unauthenticated so that monitoring systems can check health without credentials.

## Success Metrics

- Unauthorized requests receive 401 with clear error message
- TLS connections use Rustls with no openssl dependency
- Body size limit rejects oversized requests before buffering the full body
- Admin endpoints remain accessible without authentication

## Constraints

- Authentication middleware is in `crates/server/src/auth.rs`
- TLS configuration is in `crates/core/src/config.rs` (TlsConfig)
- Router and middleware stack is in `crates/server/src/lib.rs`
- CORS uses `tower_http::cors::CorsLayer::permissive()` -- allows all origins
- Body limit uses `tower_http::limit::RequestBodyLimitLayer`

## Open Questions

- [x] Should admin endpoints require auth? -- No, they are read-only and used by monitoring

## Design Decisions

| Decision | Options Considered | Chosen | Rationale |
|----------|--------------------|--------|-----------|
| TLS library | OpenSSL, Rustls | Rustls | Pure Rust, no C dependency, easier cross-compilation |
| Auth scheme | Bearer only, x-api-key only, both | Both (Bearer + x-api-key) | Supports both OpenAI-style (Bearer) and Anthropic-style (x-api-key) clients |
| CORS policy | Restrictive, configurable, permissive | Permissive | API gateway use case; clients are trusted applications, not browsers with untrusted scripts |
| Auth bypass | Always require, optional | Optional (skip if api_keys empty) | Enables zero-config development setup |
