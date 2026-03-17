# Prism Control Plane

`web/` is the canonical control-plane frontend for Prism.

It owns the runtime-first shell, workspace model, and operator workflows for traffic, providers, routing, and change management.

## Current Scope

This first slice establishes:

- a standalone Vite + React + TypeScript app
- latest verified frontend dependencies as of `2026-03-17`
- a new control-plane shell
- five workspace routes:
  - `Command Center`
  - `Traffic Lab`
  - `Provider Atlas`
  - `Route Studio`
  - `Change Studio`
- a right-side inspector model
- a small Zustand shell store
- a testable and buildable baseline

## Runtime Baseline

- `node`: `25.6.1`
- `npm`: `11.9.0`

See [.nvmrc](/Users/qiufeng/work/proxy/prism/web/.nvmrc).

## Verified Dependency Direction

The app was initialized and updated against the latest registry versions we checked during setup for:

- `react`
- `react-dom`
- `react-router-dom`
- `zustand`
- `axios`
- `lucide-react`
- `vite`
- `@vitejs/plugin-react`
- `typescript`
- `vitest`
- `@testing-library/*`
- `jsdom`
- `msw`

## Commands

```bash
cd web
npm install
npm run dev
npm run typecheck
npm run test
npm run build
npm run test:e2e
```
