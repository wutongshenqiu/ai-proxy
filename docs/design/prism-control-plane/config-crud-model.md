# Config CRUD Model

This note focuses on one question:

Is Prism's configuration surface rich enough for a serious control plane?

Short answer:

Not yet, if "rich" means more than a YAML editor and a few edit forms.

The control plane should support full config lifecycle management.

## Core Principle

Configuration should be modeled as managed objects with lifecycle verbs, not as loose text blobs.

That means each important object family should support some combination of:

- query
- create
- edit
- clone
- disable or suspend
- delete or archive
- diff
- dependency inspection
- history
- rollback

## Object Families

Prism should eventually manage at least these families as first-class config objects.

### 1. Providers

Examples:

- upstream family
- base URL
- model mappings
- excluded models
- auth binding
- upstream presentation
- health policy

Required verbs:

- query
- create from template
- edit in structured form
- clone from existing
- disable
- guarded delete
- preview / validate
- dependency graph
- version history

### 2. Auth Profiles

Examples:

- API key profiles
- bearer token profiles
- OAuth-backed profiles
- device-flow backed profiles
- subscription-backed profiles

Required verbs:

- query
- create
- edit
- disable
- guarded delete
- connect
- import
- rotate
- refresh
- audit trail

### 3. Route Profiles and Rules

Examples:

- default profile
- rule matchers
- profile policy
- fallback order
- model resolution behavior

Required verbs:

- query
- create
- edit
- clone
- archive
- simulate
- explain
- compare against baseline
- rollback by version

### 4. Tenant Policies and Access Objects

Examples:

- tenant limits
- auth keys
- allowlists
- budget or quota policy
- temporary overrides

Required verbs:

- query
- create
- edit
- bulk edit
- suspend
- revoke
- expire
- audit history

### 5. Observability and Workflow Objects

Examples:

- SLS connectors
- OTLP connectors
- alert policies
- watch windows
- investigation templates

Required verbs:

- query
- create
- edit
- test connection
- disable
- retire
- audit

## CRUD Is Not Enough By Itself

For a real control plane, plain CRUD is too weak.

Each object should also support:

### Query Richness

- free-text search
- typed filters
- saved views
- dependency-aware search
- "show affected by current change"

### Create Richness

- start from template
- clone existing
- import from external system
- guided wizard for high-risk object types

### Edit Richness

- structured editor first
- raw config second
- semantic diff
- inline validation
- dependency impact summary

### Delete Richness

- hard delete only when safe
- prefer disable / archive / retire as first-class verbs
- always show dependencies and blast radius

### History Richness

- object version history
- who changed it
- why it changed
- diff from prior version
- rollback target

## Best-Practice Direction

The strongest operator products do not expose configuration as one flat admin list.

They expose:

- registries
- object detail surfaces
- diff and audit
- ownership
- change linkage
- safe destructive actions

Implication for Prism:

- `Change Studio` should become a registry plus workflow environment
- the user should be able to discover and manage config objects even before they open a publish flow

## Recommended Change Studio Layers

### 1. Registry Layer

Searchable inventory of object families and records.

Examples:

- Providers
- Auth Profiles
- Route Profiles
- Route Rules
- Tenant Policies
- Auth Keys
- Data Sources
- Alerts

### 2. Object Detail Layer

Structured editor plus dependency panel.

### 3. Change Layer

Diff, review, rollout, observe, rollback.

### 4. Audit Layer

History, actor, reason, links, and rollback references.

## Prototype Shape In V2

The current prototype should visibly demonstrate these layers, not just describe them.

That means `Change Studio` should show:

- a `registry` surface for family browsing, search, filtering, and saved views
- an `object detail` surface for structured fields, runtime posture, and dependency review
- a `history` surface for version trail, actor, and reason
- a `destructive action` surface that makes disable, archive, and hard delete visibly different
- a `change` surface that ties selected objects into diff, publish, observe, and rollback flow

If one of these layers is missing in the prototype, the final product will usually regress back toward a simple admin page.

## Family-Specific Editor Patterns

Not every object should share the same editor shape.

The prototype should prove this explicitly.

### Providers

Provider editing should combine:

- identity and protocol shape
- connectivity and model discovery
- auth binding
- presentation preview
- runtime validation

### Auth Profiles

Auth profile editing should combine:

- record creation
- mode selection
- runtime connection flow
- identity verification
- refresh / rotate lifecycle

### Route Profiles and Rules

Route editing should combine:

- profile selection
- rule and matcher editing
- simulation and explain
- dependency and blast-radius review
- publish and rollback linkage

If these three editors collapse into one generic admin form pattern, the redesign has failed to capture the real operator workflow.

## Guardrails

High-risk actions should never be silent.

Before destructive or high-impact changes, Prism should show:

- affected routes
- affected tenants
- affected auth identities
- active investigations
- pending watch windows
- rollback target

## Decision Summary

If Prism wants rich configuration management, the bar is:

1. YAML is an escape hatch, not the main surface
2. every important object family gets explicit lifecycle verbs
3. delete is guarded, not casual
4. history, dependency, and rollback are part of the object model
