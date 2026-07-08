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
- Runtime constraint: Prototype binaries and modules no longer belong in the runtime build graph.
- Prototype behavior: Old prototype launch path has been removed from the repository.
- Runtime behavior kept/adapted: Main runtime binary always launches runtime-authority UI and no longer exposes prototype selection.
- Why direct adoption is unsafe: Keeping a second launch surface in the repo reintroduces split-brain validation and makes the branch ambiguous.
- Temporary or permanent: Permanent for the main runtime binary; removed prototype path no longer participates in the build graph.
- Revisit trigger: Only if a future, separately scoped prototype revival is explicitly approved.
- Owner: Solo implementation owner
- Date added: 2026-05-28
- Date resolved: 2026-05-31

### DVG-004
- Area: Dev cutover launcher
- Runtime constraint: Dev builds need a stable runtime-authority default with no legacy prototype branch.
- Prototype behavior: Prototype launcher removed from the repository.
- Runtime behavior kept/adapted: `src/main.rs` always selects the runtime-authority path and the prototype launch graph no longer exists.
- Why direct adoption is unsafe: Any lingering launch branch would preserve split-brain behavior and confuse dev validation.
- Temporary or permanent: Permanent for the main runtime binary and the removed prototype path.
- Revisit trigger: Only if a new prototype branch is intentionally introduced in a separate scoped effort.
- Owner: Solo implementation owner
- Date added: 2026-05-28
- Date resolved: 2026-05-31
