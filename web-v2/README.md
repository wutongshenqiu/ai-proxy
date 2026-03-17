# Prism Web V2

`web-v2/` is the greenfield control-plane frontend for Prism.

It is intentionally separate from the legacy `web/` app so the new shell and workspace model can evolve without inheriting the old page boundaries or interaction patterns.

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

See [.nvmrc](/Users/qiufeng/work/proxy/prism/web-v2/.nvmrc).

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
cd web-v2
npm install
npm run dev
npm run typecheck
npm run test
npm run build
```

## Replacement Strategy

This directory is the place to complete the new control plane.

Recommended release flow:

1. continue building all required workspaces in `web-v2/`
2. validate the full shell end-to-end
3. keep legacy `web/` as the production UI until readiness
4. replace the production entry only after the V2 pack is complete
