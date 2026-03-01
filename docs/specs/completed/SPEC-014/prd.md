# SPEC-014: Cost Tracking

## Problem

No visibility into per-request costs, making it impossible to analyze spending patterns.

## Goals

- G1: Built-in price table for major models (USD/1M tokens)
- G2: User-configurable price overrides via `model-prices` config
- G3: `x-cost` response header (future integration point)
- G4: `RequestLogEntry.cost` field
- G5: `Metrics.total_cost_usd` and `cost_by_model` in metrics snapshot

## Implementation

- `crates/core/src/cost.rs` — `CostCalculator` with built-in prices + user overrides
- `crates/core/src/config.rs` — `model_prices: HashMap<String, ModelPrice>`
- `crates/core/src/request_log.rs` — `cost: Option<f64>` field
- `crates/core/src/metrics.rs` — `total_cost_micro`, `model_costs`, `record_cost()`
- `AppState` gains `cost_calculator: Arc<CostCalculator>`
- Config hot-reload updates cost calculator prices

## Status

Active — Implementation complete, pending review.
