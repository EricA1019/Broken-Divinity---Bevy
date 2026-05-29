# UNIFIED-UI-DIVERGENCE-LEDGER

Date opened: 2026-05-28
Status: Active

Purpose:
Track every intentional runtime divergence from unified prototype direction during migration. Divergences must be explicit, constrained, and revisited.

## Entry format
- Divergence ID:
- Area:
- Runtime constraint:
- Prototype behavior:
- Runtime behavior kept/adapted:
- Why direct adoption is unsafe:
- Temporary or permanent:
- Revisit trigger:
- Owner:
- Date added:
- Date resolved:

## Active divergences

### DVG-001
- Area: Inventory accessory model
- Runtime constraint: Production equipment model currently exposes one accessory slot.
- Prototype behavior: Three accessory slots with routing priority and slot-specific targeting.
- Runtime behavior kept/adapted: Adapter path selected in Phase 4. Runtime keeps one accessory slot and routes accessory-like IDs through `inventory_rules` into that slot with swap-first semantics.
- Why direct adoption is unsafe: Direct copy would force uncontrolled data/save migration and risk runtime regressions.
- Temporary or permanent: Temporary until save/schema migration work is explicitly scoped.
- Revisit trigger: Phase 6 cutover review or any save-schema migration initiative that introduces multi-accessory runtime support.
- Owner: Solo implementation owner
- Date added: 2026-05-28
- Date resolved: 2026-05-28 (decision recorded; divergence remains active)

### DVG-002
- Area: Symbol/token strictness
- Runtime constraint: Existing runtime screens still contain mixed icon and token usage.
- Prototype behavior: Strict contract-driven symbol grammar.
- Runtime behavior kept/adapted: Staged token migration by phase.
- Why direct adoption is unsafe: One-shot replacement across all screens increases regression risk.
- Temporary or permanent: Temporary.
- Revisit trigger: Phase 5 consolidation gate.
- Owner: Solo implementation owner
- Date added: 2026-05-28
- Date resolved: 2026-05-28 (key-hint copy and binding tokens consolidated via `src/ui/input_hints.rs`)

### DVG-003
- Area: Runtime vs prototype launch surface
- Runtime constraint: Prototype binaries compile and link costs can interfere with runtime validation flow.
- Prototype behavior: Prototype binaries and modules are available in default build graph.
- Runtime behavior kept/adapted: Prototype surfaces are now opt-in behind Cargo feature `ux-prototypes`; default runtime path excludes them.
- Why direct adoption is unsafe: Default inclusion increases split-brain risk and expands build/test blast radius during runtime-phase gates.
- Temporary or permanent: Temporary until Phase 6 cutover/handoff decides long-term prototype retention policy.
- Revisit trigger: Phase 6 full swap and handoff review.
- Owner: Solo implementation owner
- Date added: 2026-05-28
- Date resolved:

### DVG-004
- Area: Dev cutover launcher
- Runtime constraint: Dev builds need a stable unified default while non-dev builds still need rollback continuity.
- Prototype behavior: Unified prototype becomes the default launcher in dev configurations.
- Runtime behavior kept/adapted: `src/main.rs` now selects the unified prototype launcher for `feature = "dev"` and keeps the legacy runtime path under `not(feature = "dev")`.
- Why direct adoption is unsafe: Removing the legacy path entirely before cutover validation would eliminate the fastest rollback boundary.
- Temporary or permanent: Temporary until Phase 6 final gate closes and a handoff decision is made on whether to retain the legacy branch.
- Revisit trigger: Phase 6 completion record and any post-cutover regression on dev.
- Owner: Solo implementation owner
- Date added: 2026-05-28
- Date resolved:
