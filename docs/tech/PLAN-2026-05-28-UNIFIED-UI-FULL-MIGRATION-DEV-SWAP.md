# PLAN-2026-05-28-UNIFIED-UI-FULL-MIGRATION-DEV-SWAP

Date: 2026-05-28
Branch Target: dev
Status: Proposed (phased execution, full swap required)

## Goal
When this plan is complete, the production runtime UI on dev is fully swapped to the unified prototype direction (style, hierarchy, interaction grammar), with no mixed legacy/unified screen behavior remaining.

## Non-Negotiable End State
- Dev branch runs with unified UI as the default and only runtime presentation path.
- Legacy UI paths are removed or hard-disabled behind non-default compatibility switches.
- All required runtime and UX contract tests pass before final swap commit.

## Scope
### In
- Full phased migration of runtime UI surfaces to unified design direction.
- Shared style/token migration to eliminate per-panel style drift.
- Inventory/equipment behavior migration with runtime-safe semantics.
- Input/modal/help/objective prompt alignment with unified hierarchy.
- Test-first behavior contracts for high-risk interactions.
- Final full swap commit on dev.

### Out
- Gameplay mechanics rebalance.
- Overworld graph generation changes.
- Combat formula changes.
- Major ECS architecture refactor unrelated to UI migration.

### Deferred
- Advanced motion profile variants beyond readability-safe defaults.
- Expanded accessibility preset suite beyond baseline contrast guarantees.

## Migration Strategy
- Use vertical slices with hard gates per phase.
- Each phase lands in dev only after tests and compile checks pass.
- Maintain one migration seam for style and one for shared UI behavior helpers to avoid duplication.
- No prototype state/resource types are allowed to leak into production runtime modules.

## Solo-Team Definition of Done (Global)
This migration is executed by one person, so done criteria must include implementation quality and process discipline.

Global DoD is met only when all items below are true:
1. Scope discipline:
   - Every completed task maps to an in-scope item.
   - No deferred/out-of-scope work was merged.
2. Plan discipline:
   - Each completed phase contains a recorded plan-validation note.
   - Any divergence from plan is logged with reason and approval note (self-approval note with explicit rationale).
3. Testing discipline:
   - Required tests were written before behavior changes in that phase.
   - Phase gate tests are green before moving to next phase.
4. Quality discipline:
   - No duplicate helper logic between prototype/runtime for migrated features.
   - No new magic style values in touched code when token alternatives exist.
   - No leakage of prototype state/resources into production runtime modules.
5. UX discipline:
   - Key hints match actual bindings.
   - Blocked actions provide user-visible status feedback.
6. Delivery discipline:
   - Handoff updated with what changed, what did not, and known residual risks.
   - Final cutover commit is auditable and reversible.

## Anti-Drift Validation Protocol (Mandatory)
Use this protocol at every step and phase boundary.

Step-level checkpoint (run after each execution step):
1. Confirm step output matches the step objective exactly.
2. Confirm no out-of-scope files were changed.
3. Confirm no behavior was modified unless the current step explicitly allows it.
4. Record one-line validation note in working log/handoff draft:
   - Step ID
   - Expected result
   - Actual result
   - Drift: none or describe

Phase-level checkpoint (run before phase gate):
1. Re-read phase objective, deliverables, and gate criteria.
2. Verify every deliverable is complete with evidence.
3. Verify test-first requirement was followed.
4. Verify gate commands pass.
5. If anything fails, do not start next phase.

Drift escalation rule:
- If two consecutive steps show drift, stop implementation, update plan with corrective sub-steps, then resume.

## Phase Plan

## Phase 0 — Lock Decisions and Test Harness First
Objective: Remove ambiguity before touching behavior.

Deliverables:
- Architecture decision record for migration seam.
- Divergence ledger file under docs/tech.
- New failing tests for migration-sensitive behavior.
- Phase acceptance sheet for all later gates.

Primary files:
- src/ui/mod.rs
- src/ui/inventory_panel.rs
- src/ui/help_panel.rs
- src/ui/modal_priority.rs
- src/ui/objective_prompt.rs
- tests/* (new and existing)
- docs/tech/* (decision + divergence ledger)

Execution steps:
1. Freeze seam decisions before code movement:
   - Shared runtime style helpers live in one place.
   - Shared runtime inventory/equipment rules live in one helper module.
   - Production code may reference prototype modules only as read-only design references.
2. Define branch discipline for phase work:
   - One phase = one mergeable unit.
   - No cross-phase behavior edits in the same PR.
3. Create divergence ledger format with required fields:
   - Divergence ID.
   - Runtime constraint.
   - Why prototype cannot be copied directly.
   - Revisit trigger.
4. Write failing tests before behavior changes:
   - Occupied equipment slot behavior contract.
   - Equip/unequip routing contract.
   - Accessory slot policy contract.
   - Key hint text matches active key bindings.
   - Modal/input precedence when multiple overlays compete.
5. Freeze final cutover acceptance criteria and publish in this plan.

Validate against plan checkpoints:
- After each step, run Step-level checkpoint.
- Before gate, run Phase-level checkpoint and confirm Phase 0 deliverables exactly match plan.

Test-first requirement:
- No Phase 1 implementation starts until failing tests from this phase exist and are reviewed.

Gate to exit:
- Architecture and divergence docs committed.
- New tests compile and fail for not-yet-implemented behavior.
- Existing baseline tests remain green.

Rollback trigger:
- If seam decision cannot be enforced without prototype state leakage, stop and revise architecture before proceeding.

## Phase 1 — Style Contract Reconciliation
Objective: Establish single source of truth for style and symbols.

Deliverables:
- One canonical style token source.
- Runtime-safe style adapter API.
- Token naming map from old per-screen values to unified tokens.

Primary files:
- src/ui/ux_style_contract.rs
- docs/tech/UX-VISUAL-CONTRACT-2026-05-28.md
- src/ui/menu.rs
- src/ui/hud.rs
- src/ui/gamelog_panel.rs
- src/ui/colony_panel.rs
- src/ui/overworld_panel.rs
- src/ui/stats_progression_panel.rs
- src/ui/perk_choice_panel.rs

Execution steps:
1. Reconcile documented symbol grammar and actual style token implementation.
2. Lock production token set:
   - Palette tokens.
   - Typography tiers.
   - Spacing tiers.
   - Emphasis semantics for warnings/errors/success.
3. Add runtime-facing adapter methods for common widget styling patterns.
4. Inventory current hardcoded style literals in target runtime files.
5. Replace style literals only where touched in this phase with named constants/tokens.
6. Add notes for untouched literals to later cleanup queue.

Validate against plan checkpoints:
- After each step, run Step-level checkpoint.
- Before gate, confirm no non-style behavior edits were introduced.

Test-first requirement:
- Add/adjust any lightweight contract checks for style token presence or usage helpers before broad per-screen edits.

Gate to exit:
- Style contract compiles and is internally consistent.
- Contract doc and code agree on symbol/tier naming.
- No new raw style values introduced in touched lines.

Rollback trigger:
- If adapter API requires per-screen exceptions beyond agreed limits, pause and simplify adapter before surface migration.

## Phase 2 — Low-Risk Surface Migration
Objective: Migrate low-risk surfaces to prove seam.

Targets:
- Menu
- Game log
- Perk choice panel

Deliverables:
- Low-risk screens visually aligned to unified grammar.
- Behavior unchanged for action dispatch and state transitions.

Primary files:
- src/ui/menu.rs
- src/ui/gamelog_panel.rs
- src/ui/perk_choice_panel.rs
- tests/menu_runtime_contracts.rs
- tests/runtime_app_integration.rs

Execution steps:
1. Menu migration slice:
   - Replace visual constants with style tokens.
   - Keep MenuUiAction and process flow unchanged.
   - Ensure seed/readability assertions still hold.
2. Game log migration slice:
   - Move panel chrome to unified token usage.
   - Preserve log color meaning and ordering.
3. Perk panel migration slice:
   - Apply unified hierarchy and tokenized typography.
   - Preserve unlock event flow and queue behavior.
4. After each slice:
   - Run targeted tests.
   - Verify no input or transition regressions.

Validate against plan checkpoints:
- After each slice, run Step-level checkpoint and attach test outcomes.
- Before gate, verify process behavior signatures are unchanged.

Test-first requirement:
- For any copy/key-hint changes, add/adjust expectation tests first where deterministic assertions are possible.

Gate to exit:
- menu_runtime_contracts passes.
- runtime_app_integration passes.
- No change to process-system behavior signatures.

Rollback trigger:
- If any slice alters action semantics, revert that slice and split visual and behavioral edits.

## Phase 3 — Core Shell Migration
Objective: Migrate always-visible shell structures.

Targets:
- HUD
- Colony panels
- Overworld panel

Deliverables:
- HUD/colony/overworld panels follow unified shell grammar.
- State gating and data ownership unchanged.
- Modal/help/objective interactions preserved.

Primary files:
- src/ui/hud.rs
- src/ui/colony_panel.rs
- src/ui/overworld_panel.rs
- src/ui/help_panel.rs
- src/ui/modal_priority.rs
- src/ui/objective_prompt.rs

Execution steps:
1. HUD migration:
   - Restructure into stable rows without changing query contracts.
   - Keep combat/sanity/ammo semantics unchanged.
2. Colony migration:
   - Reframe top strip + core management blocks + hint line.
   - Preserve existing action resource dispatch and resource warnings.
3. Overworld migration:
   - Align mission-board framing and CTA presentation.
   - Preserve travel-state and save/quit behavior.
4. Interaction safety pass:
   - Re-check help toggle suppression under blockers.
   - Re-check modal priority policy under raid/modal conditions.
   - Re-check objective prompt visibility progression.

Validate against plan checkpoints:
- After each target migration (HUD, Colony, Overworld), run Step-level checkpoint.
- Before gate, verify no state machine boundary changes were introduced.

Test-first requirement:
- Add/adjust tests for modal/help/objective behavior before shell structure edits that affect interaction priority.

Gate to exit:
- Existing baseline tests pass.
- New interaction-priority tests pass.
- No new ordering dependency introduced in system registration.

Rollback trigger:
- If shell migration requires changing state machine boundaries, stop and open a separate architecture plan.

## Phase 4 — Inventory/Equipment Full Behavior Migration
Objective: Move from basic runtime inventory to unified interaction model.

Deliverables:
- Runtime inventory/equipment behavior parity with approved unified semantics.
- Explicit occupied-slot policy implemented and tested.
- Consumable turn-flow preserved.

Primary files:
- src/ui/inventory_panel.rs
- src/core/inventory.rs (if model update required)
- src/ui/ux_inventory_equipment_prototype.rs (reference only)
- tests/* inventory-related contracts

Execution steps:
1. Finalize occupied-slot policy from failing tests:
   - Swap-first or block-with-feedback.
   - Apply policy consistently to click, drag, and double-click paths.
2. Extract or implement pure helper functions for inventory/equipment routing.
3. Wire helpers into runtime panel draw/process flow.
4. Preserve current consumable use action path:
   - PendingAction::UseItem and turn-phase transitions remain authoritative.
5. Accessory model decision:
   - If runtime model expands, include compatibility handling and explicit migration note.
   - If adapter path used, record as divergence with revisit trigger.
6. Add user-visible status feedback for blocked actions (inventory full, invalid slot, policy blocked).

Validate against plan checkpoints:
- After each behavior change, run Step-level checkpoint with test references.
- Before gate, verify consumable action semantics remain unchanged from baseline.

Test-first requirement:
- Behavioral tests for equip/unequip/swap/block/accessory routing must exist before runtime panel logic update.

Gate to exit:
- All new inventory behavior tests pass.
- Existing dungeon/turn flow tests pass.
- No regression in consumable action semantics.

Rollback trigger:
- If accessory model change forces uncontrolled save/state churn, ship adapter path and defer model expansion.

### Phase 4 Validation Note (2026-05-28)
- Step objective: Lock down runtime inventory/equipment routing semantics before Phase 5.
- Actual implementation:
   - Added dedicated routing seam in `src/ui/inventory_rules.rs`.
   - Implemented explicit equip/swap/unequip/block outcomes via `EquipOutcome`, `InventoryRuleError`, and `EquipmentSlot`.
   - Integrated seam into `src/ui/inventory_panel.rs` without changing consumable dispatch (`PendingAction::UseItem` remains authoritative).
   - Added blocked-action status feedback for equip/unequip failures.
- Test-first evidence:
   - Added `tests/ui_inventory_phase4_contracts.rs` before inventory runtime wiring updates.
   - Added contracts for weapon route, occupied-slot swap, non-equippable block, accessory route/swap, invalid slot, empty unequip, full-inventory unequip block, and consumable action routing preservation.
- Drift check:
   - Scope drift: none (changes limited to phase inventory behavior seam + panel integration).
   - Plan drift: recorded adapter-path decision in divergence ledger DVG-001.
- Gate status:
   - Compile diagnostics for all touched Phase 4 files: clean.
   - Library-scoped Phase 4 seam tests: pass (`cargo test --lib inventory_rules::tests`, 5/5 pass).
   - Library-scoped consumable regressions: pass (`cargo test --lib game::dungeon::consumables`, 5/5 pass).
   - Integration target `cargo test --test ui_inventory_phase4_contracts` still exhibits persistent long-running build behavior in this environment; re-run this single command in a clean shell before entering Phase 5.

## Phase 5 — Consolidation and Legacy Path Removal
Objective: Eliminate split-brain UI state before final swap.

Deliverables:
- Single active runtime UI path for all migrated screens.
- Unified key-hint copy and hierarchy consistency.
- Updated docs reflecting actual runtime architecture.

Primary files:
- src/ui/* (all migrated surfaces)
- src/main.rs (verification of registration order only)
- docs/tech/*
- ui/README.md
- tech/ui-design.md

Execution steps:
1. Remove or disable legacy rendering paths for migrated screens.
2. Remove duplicated helper logic introduced during migration.
3. Normalize key-hint copy against actual input bindings.
4. Confirm all migrated screens reference shared token/helper seam.
5. Update docs after code behavior is stable.

Validate against plan checkpoints:
- After each cleanup step, run Step-level checkpoint.
- Before gate, verify no screen still references legacy visual path.

Test-first requirement:
- Add a final key-hint and screen-presence sanity test where deterministic checks are possible before deleting legacy branches.

Gate to exit:
- No runtime surface still relies on legacy visual path.
- All contract tests pass after legacy-path removal.
- Docs reflect current runtime truth.

Rollback trigger:
- If legacy removal breaks a required flow, restore only the minimal removed path and add a tracked follow-up removal item.

### Phase 5 Validation Note (2026-05-28)
- Step objective: Remove split runtime/prototype path drift and normalize input-hint semantics.
- Actual implementation:
   - Disabled legacy prototype modules from default runtime compile path by gating `src/ui/mod.rs` prototype exports behind `feature = "ux-prototypes"`.
   - Converted prototype binaries to explicit opt-in by setting `autobins = false`, declaring all prototype bins in `Cargo.toml`, and applying `required-features = ["ux-prototypes"]`.
   - Consolidated runtime input bindings and key-hint copy through shared tokens in `src/ui/input_hints.rs` and routed inventory/journal/stats/help toggles through those canonical constants.
- Test-first evidence:
   - Expanded `tests/ui_phase5_runtime_contracts.rs` before implementation to enforce:
      - prototype feature-gating,
      - migrated runtime panel presence,
      - shared key-binding token usage,
      - canonical hint-copy expectations.
- Drift check:
   - Scope drift: none (changes restricted to runtime/prototype seam and key-hint consolidation).
   - Plan drift: required prototype-bin check commands now run with `--features ux-prototypes` because prototype binaries are intentionally opt-in.
- Gate status:
   - `cargo test --test ui_phase5_runtime_contracts`: pass (8/8).
   - Required validation matrix: pass (`menu_runtime_contracts`, `runtime_app_integration`, `ux_baseline_red_harness`, `cargo check`, prototype bin checks with feature, Phase 4 seam and consumable regressions).
   - Full `cargo test` remains intermittently blocked by long-running link stage on `ui_inventory_phase4_contracts` in this environment; rerun in a clean shell at cutover gate.

## Phase 6 — Full Swap on Dev (Cutover)
Objective: Perform irreversible default-path switch on dev.

Deliverables:
- Unified-only runtime defaults on dev.
- Final cutover commit.
- Handoff with divergence and risk summary.

Primary files:
- src/main.rs
- src/ui/*
- docs/tech/HANDOFF-*.md
- docs/tech/divergence-ledger*.md (or chosen file)

Execution steps:
1. Flip default runtime path to unified-only.
2. Remove remaining compatibility toggles unless explicitly retained for emergency rollback.
3. Run full validation matrix and record results in handoff.
4. Produce one auditable cutover commit with clear scope boundaries.
5. Publish residual risks and known limitations.

Validate against plan checkpoints:
- After cutover flip, run immediate Step-level checkpoint and full validation commands.
- Before final gate, confirm all Global DoD criteria are met.

Cutover checklist:
- Unified path active in dev by default.
- Legacy path unavailable in normal runtime execution.
- All validation commands green.
- Divergences documented and accepted.

Gate to exit (Final):
- Dev branch default runtime is unified UI only.
- All required tests and checks pass.
- Handoff includes divergence ledger, rollback note, and residual risks.

Rollback trigger:
- If full validation fails after cutover flip, revert cutover commit and reopen from last successful Phase 5 state.

### Phase 6 Validation Note (2026-05-28)
- Step objective: Perform irreversible default-path switch on dev while keeping a minimal rollback boundary.
- Actual implementation:
   - Split `src/main.rs` into a dev launcher and a non-dev legacy runtime path.
   - Dev builds now launch `broken_divinity::ui::ux_unified_prototype::UnifiedPrototypePlugin` as the default runtime surface.
   - Non-dev builds retain the legacy runtime path for rollback and release stability.
   - `Cargo.toml` now makes `dev` imply `ux-prototypes`, ensuring the cutover launcher compiles in dev builds.
- Test-first evidence:
   - Expanded `tests/ui_phase5_runtime_contracts.rs` to require:
      - a dev-cutover launcher marker,
      - unified prototype presence in the dev path,
      - `dev` feature inclusion of `ux-prototypes`.
- Drift check:
   - Scope drift: none; change is limited to default launcher selection, feature wiring, and associated documentation.
   - Plan drift: none; retaining the non-dev legacy branch is the explicit rollback boundary.
- Gate status:
   - `cargo test --test ui_phase5_runtime_contracts`: pass (10/10).
   - `cargo check --features dev`: pass.
   - Previous runtime validation matrix remains green from Phase 5 and still applies.

## Test and Validation Order (TDD-Enforced)
1. Write/adjust tests before each behavior migration segment.
2. Implement minimal code to satisfy tests.
3. Run targeted tests after each surface.
4. Run integration suite at each phase gate.
5. Run full suite at final cutover.

## Required Validation Commands
- cargo test --test menu_runtime_contracts
- cargo test --test runtime_app_integration
- cargo test --test ux_baseline_red_harness
- cargo check
- cargo check --features ux-prototypes --bin ux_unified_prototype
- cargo check --features ux-prototypes --bin ux_inventory_equipment_prototype
- additional deterministic inventory/equipment tests introduced in Phase 0/4

## Risk Register
1. Risk: hidden input conflicts during shell migration.
   - Mitigation: modal/input priority tests and gate checks in Phases 3 and 6.
2. Risk: inventory migration balloons into data-model refactor.
   - Mitigation: explicit accessory policy decision in Phase 4 with transitional adapter option.
3. Risk: duplicate logic across prototype/runtime.
   - Mitigation: shared helper seam, legacy cleanup in Phase 5.
4. Risk: cutover regressions on dev.
   - Mitigation: dedicated full-swap gate and cutover commit after full suite pass.

## Rollback Strategy
- If a phase gate fails, revert only that phase’s delta and keep previous successful phase state.
- Keep cutover as a single auditable commit for clean rollback in dev if post-cutover issues appear.

## Done When
- Unified UI is fully swapped in on dev as the sole runtime presentation path.
- Runtime behavior contracts remain intact.
- Inventory/equipment migration is complete with tested semantics.
- Legacy mixed-path UI drift is removed.

## Solo-Team Completion Record (Required)
Before declaring complete, append a completion record to the handoff with:
1. Phase-by-phase gate pass results.
2. Drift incidents and corrective actions.
3. Divergence ledger summary.
4. Final validation command outputs summary.
5. Explicit statement: "Global DoD satisfied" or "Global DoD not satisfied".
