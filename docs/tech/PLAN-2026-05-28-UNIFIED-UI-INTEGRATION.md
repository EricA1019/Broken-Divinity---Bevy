# PLAN-2026-05-28-UNIFIED-UI-INTEGRATION

Date: 2026-05-28
Owner: UI/UX integration thread
Status: Proposed (implementation-ready)

## Goal
When this plan is complete, the production UI will use the unified prototype as its default design language across menu, dungeon, colony, overworld, dossier, and inventory/equipment, while preserving runtime gameplay/state contracts and existing test guarantees.

## Scope
**In**
- Align production style tokens and panel grammar to the unified prototype contract.
- Migrate runtime screens toward unified structure and copy hierarchy.
- Integrate inventory/equipment interaction model from unified prototype with runtime-safe behavior.
- Preserve and extend current runtime flow and menu/test contracts.
- Add or update tests for the newly integrated UI behavior where deterministic logic exists.

**Out**
- Deep gameplay mechanic changes (combat formulas, colony economy, overworld generation).
- Full replacement of runtime ECS data plumbing with prototype mock data structures.
- Art/audio/theme asset overhauls outside egui/text rendering surfaces.
- Refactors unrelated to UI integration scope.

**Deferred**
- Advanced motion profile rollout in production UI (Subtle/Drift/Scanline) beyond minimal readability-safe cues.
- Full unification of runtime app shell and main game shell architecture.
- Accessibility presets and colorblind variants beyond baseline contrast guardrails.

## Decision Policy (Divergence Handling)
- Default: unified prototype wins.
- Exception: keep runtime behavior when prototype-level behavior would violate state ownership, input capture order, save/load continuity, or a validated test contract.
- Any exception must be documented in code comment or plan addendum as: `Divergence: reason + constraint + revisit trigger`.

## Prerequisites
- Unified prototype reference exists: `src/ui/ux_unified_prototype.rs` ✓
- Shared style contract exists: `src/ui/ux_style_contract.rs` ✓
- Production UI surfaces are modularized under `src/ui/` ✓
- Runtime flow/menu baseline tests exist under `tests/` ✓
- Gap: contract mismatch between visual doc and style token implementation — Task 1 resolves this.

## Integration Map (Current -> Target)

### 1) Style Contract and Token Layer
Current:
- Production panels each define local colors/sizes.
Target:
- Shared token usage sourced from unified contract.
Files:
- `src/ui/ux_style_contract.rs`
- `docs/tech/UX-VISUAL-CONTRACT-2026-05-28.md`
- `src/ui/menu.rs`, `src/ui/hud.rs`, `src/ui/gamelog_panel.rs`, `src/ui/colony_panel.rs`, `src/ui/overworld_panel.rs`, `src/ui/stats_progression_panel.rs`, `src/ui/perk_choice_panel.rs`

### 2) Screen Grammar and Hierarchy
Current:
- Runtime UI uses multiple independent windows/panels with mixed hierarchy.
Target:
- Consistent header -> core content -> action row -> hint row grammar.
Files:
- `src/ui/menu.rs`
- `src/ui/hud.rs`
- `src/ui/gamelog_panel.rs`
- `src/ui/colony_panel.rs`
- `src/ui/overworld_panel.rs`

### 3) Inventory + Equipment
Current:
- Runtime inventory is simple list + use-item action.
Target:
- Unified inventory/equipment semantics: list readability, paper-doll layout, equip/unequip flows, safe replacement/swap policy.
Files:
- `src/ui/inventory_panel.rs`
- `src/ui/ux_inventory_equipment_prototype.rs` (reference)
- optional helper extraction module under `src/ui/`

### 4) UX Governance and Contracts
Current:
- Existing tests validate runtime flow/menu but not full UI integration semantics.
Target:
- Preserve current tests and add deterministic behavior checks for new integration logic.
Files:
- `tests/menu_runtime_contracts.rs`
- `tests/runtime_app_integration.rs`
- `tests/ux_baseline_red_harness.rs`
- new test file(s) for inventory/equipment deterministic helpers

## Tasks

### Phase A — Contract Reconciliation (No behavior changes)
- [ ] 1. Reconcile symbol set and token naming between `docs/tech/UX-VISUAL-CONTRACT-2026-05-28.md` and `src/ui/ux_style_contract.rs`.
- [ ] 2. Define the production-safe subset of unified style tokens (palette, size tiers, spacing, text emphasis) in `src/ui/ux_style_contract.rs`.
- [ ] 3. Add a tiny style adapter API in `src/ui/ux_style_contract.rs` for runtime panels (no prototype-only assumptions).
- [ ] 4. Update `tech/ui-design.md` and `ui/README.md` only where they conflict with the chosen dossier vs inventory/equipment information architecture.

### Phase B — Low-Risk Surface Convergence
- [ ] 5. Apply shared style tokens to `src/ui/menu.rs` while preserving `MenuUiAction` and existing menu contract tests.
- [ ] 6. Apply shared style tokens to `src/ui/gamelog_panel.rs` and keep log color semantics intact.
- [ ] 7. Apply shared style tokens to `src/ui/perk_choice_panel.rs` with unchanged unlock behavior.
- [ ] 8. Align objective/CTA hint wording in `src/ui/colony_panel.rs` and `src/ui/overworld_panel.rs` to unified copy hierarchy.

### Phase C — Core Shell Integration
- [ ] 9. Refactor `src/ui/hud.rs` visual hierarchy toward unified shell grammar without changing data sources.
- [ ] 10. Align `src/ui/colony_panel.rs` panel structure to unified colony sectioning (resource strip + focused management blocks + clear action/hint row).
- [ ] 11. Align `src/ui/overworld_panel.rs` structure and emphasis to unified mission-board model while retaining runtime map/travel state.
- [ ] 12. Ensure help/modal/objective prompt interplay remains stable in `src/ui/help_panel.rs`, `src/ui/modal_priority.rs`, and `src/ui/objective_prompt.rs` after structural UI updates.

### Phase D — Inventory/Equipment Migration
- [ ] 13. Decide integration architecture: embed unified inventory/equipment renderer in production panel or extract shared renderer/helpers for both prototype/runtime.
- [ ] 14. Implement runtime-safe equip/unequip/drag/drop semantics in `src/ui/inventory_panel.rs` with explicit occupied-slot policy (swap or block+message).
- [ ] 15. Support accessory slot expansion policy in production data model/UI binding, or document a temporary adapter if core inventory model remains single accessory.
- [ ] 16. Preserve consumable-use turn flow (`PendingAction::UseItem`) while adding equipment interactions.
- [ ] 17. Add deterministic tests for inventory/equipment helper behavior (double-click equip, unequip routing, slot compatibility, occupied slot behavior).

### Phase E — Wiring, Validation, and Cleanup
- [ ] 18. Verify `src/main.rs` UI system ordering still respects modal priority and input expectations after integration.
- [ ] 19. Run targeted checks: menu/runtime contracts + new UI behavior tests + relevant `cargo check` bins.
- [ ] 20. Remove stale copy/hint strings that conflict with new unified behavior and ensure keybinding hints are exact.
- [ ] 21. Write follow-up handoff note capturing any accepted divergences and revisit triggers.

## Dependency Order
- Tasks 1-4 must complete before broad styling changes (5-12).
- Tasks 9-12 should complete before inventory migration finalization (13-17) to avoid duplicate hierarchy rewrites.
- Tasks 18-21 are finalization gates and require completion of all prior phases.

## Risk Register
- Risk: visual convergence accidentally changes gameplay input behavior.
  Mitigation: keep draw/process split unchanged; verify action resources and turn-phase transitions after each phase.
- Risk: inventory/equipment migration conflicts with current single-accessory runtime model.
  Mitigation: ship adapter or staged slot rollout with explicit temporary divergence note.
- Risk: token unification causes contrast regressions.
  Mitigation: preserve existing readability guardrails and re-check contrast-sensitive labels.

## Validation Matrix
- Contract validation:
  - `cargo test --test menu_runtime_contracts`
  - `cargo test --test runtime_app_integration`
  - `cargo test --test ux_baseline_red_harness`
- UI compile validation:
  - `cargo check`
  - `cargo check --bin ux_unified_prototype`
  - `cargo check --bin ux_inventory_equipment_prototype`
- Inventory behavior tests (new):
  - deterministic helper tests for equip/unequip/slot routing/occupied-slot policy

## Done When
- Production UI surfaces share a consistent tokenized visual grammar derived from unified prototype decisions.
- Runtime flow/menu contract tests remain green.
- Inventory/equipment behavior is integrated with explicit occupied-slot policy and deterministic tests.
- Any retained divergence from unified prototype is documented with reason and revisit trigger.
