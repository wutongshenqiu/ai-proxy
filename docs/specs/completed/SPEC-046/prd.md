# PRD: Dashboard Auth & Security Hardening

| Field     | Value          |
|-----------|----------------|
| Spec ID   | SPEC-046       |
| Title     | Dashboard Auth & Security Hardening |
| Author    | Claude          |
| Status    | Active         |
| Created   | 2026-03-13     |
| Updated   | 2026-03-13     |

## Problem Statement

Dashboard authentication needs hardening before safe remote exposure: no brute-force protection and no access restriction mechanism.

## Goals

- Add login rate limiting with configurable max attempts and lockout window
- Add localhost-only access mode for dashboard
- Clear rate limit state on successful login

## Non-Goals

- IP allowlist (deferred — localhost-only covers most use cases)
- Audit logging for dashboard actions (deferred)
