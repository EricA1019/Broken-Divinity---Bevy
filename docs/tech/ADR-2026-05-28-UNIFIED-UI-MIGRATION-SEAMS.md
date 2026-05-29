# ADR-2026-05-28-UNIFIED-UI-MIGRATION-SEAMS

Date: 2026-05-28
Status: Accepted
Decision owners: Solo implementation owner

## Context
The runtime UI and unified prototype currently diverge in style, hierarchy, and inventory/equipment interaction depth. Migration must converge these without destabilizing state flow, save/load behavior, or input/modal priority.

## Decision
Use two explicit seams for migration:

1. Style seam
- Shared style tokens and adapter helpers live in one runtime-facing module.
- Runtime screens consume style adapter APIs instead of local ad-hoc constants where migration touches them.

2. Behavior seam
- Inventory/equipment routing rules and slot-policy logic live in dedicated helper modules.
- Draw/process systems remain state-gated and continue to own ECS mutations.

## Hard constraints
- No prototype-only state/resource structs may be used as production runtime state.
- Unified prototype modules are design references, not runtime dependencies.
- Existing process action resources remain authoritative for gameplay mutations.

## Consequences
Positive:
- Reduces duplicated migration logic.
- Limits blast radius for high-risk behavior changes.
- Supports phased swaps with reversible checkpoints.

Negative:
- Requires extra adapter/helper layer before broad surface migration.
- Increases upfront planning and test scaffolding work.

## Rejected alternatives
1. Directly embedding unified prototype modules into runtime
- Rejected due to mock-state coupling and high regression risk.

2. Per-screen one-off rewrites without shared seams
- Rejected due to drift and long-term maintenance overhead.

## Validation
- Migration checks enforce seam usage and deny prototype-state leakage.
- Tests validate runtime behavior contracts remain intact during migration.
