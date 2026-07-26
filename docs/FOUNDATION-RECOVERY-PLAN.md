# Broken Divinity Foundation Recovery Plan

**Status:** Complete chronological execution record; final acceptance gates
reopened by the Foundation Stabilization Plan

**Effective date:** 2026-07-24

**Product target:** Broken Divinity Foundation MVP

**Current phase:** Phase 9 complete as of 2026-07-24

**Next authorized work:** `FOUNDATION-STABILIZATION-PLAN.md` Phase 0

**Supersedes for execution:** `docs/ACTIVE-PLAN.md` and its Phase 0–11 completion claims

**Does not supersede:** `GDD.md`, `Kernel.md`, `Kernel-direction.md`, or locked decisions in `docs/DECISIONS-TO-LOCK.md`

## 1. Purpose

This plan is the authoritative path from the current partially integrated foundation to a trustworthy Broken Divinity Foundation MVP.

The project is not being restarted. Existing code, tests, content, and documentation are preserved by default. The work is an integration recovery focused on the two connected foundation loops:

```text
Shelter continuity
  → preparation and management
  → fixed dungeon risk
  → combat, loot, and extraction
  → persistent return to the shelter
```

The plan is complete only when code, content, tests, terminal behavior, and canonical documentation prove the same player journey.

### Post-acceptance regression status — 2026-07-24

A later clean-session discovery test disproved parts of the final acceptance
gate recorded below. The recovery implementation and evidence remain
preserved, but their unqualified MVP acceptance claim is no longer current.

Confirmed reopened areas:

- construction validates global colony resources while charging a player-owned
  pool;
- direct Foundation dungeon entry bypasses its declared colony cost;
- normal player defeat emits an invalid/despawned-entity Bevy command error;
- exit and shelter-gate hints mutate the log every idle frame;
- terminal drawing runs continuously while visible state is unchanged;
- the supported 80x24 layout clips controls and hides build options;
- inventory, daily-cycle, buffered-input, log-order, binding, and strict-lint
  gates are incomplete.

Current task authority is
[FOUNDATION-STABILIZATION-PLAN.md](FOUNDATION-STABILIZATION-PLAN.md). It reopens
only affected Foundation gates and does not authorize Product P2 or deferred
systems.

## 2. Authority and Reading Order

Agents and developers must use this order:

1. `GDD.md` — product and player-experience authority.
2. `docs/DECISIONS-TO-LOCK.md` — locked product decisions.
3. `Kernel.md` — technical architecture authority.
4. `Kernel-direction.md` — technical execution appendix, subordinate to `Kernel.md`.
5. `docs/MIGRATION-AND-DEPRECATION.md` — preservation and deprecation policy.
6. `docs/MVP-SCENARIO.md` — canonical Foundation MVP acceptance scenario.
7. This document — completed Foundation implementation sequence and exit gates.
8. Current code, tests, and content — evidence, not authority over the documents above.

If two documents conflict:

- product behavior follows `GDD.md`;
- locked scope follows `DECISIONS-TO-LOCK.md`;
- technical ownership follows `Kernel.md`;
- implementation stops until the conflict is recorded and resolved;
- no agent may silently choose whichever document is easiest to implement.

The root `GDD.md` is the only active GDD.
`broken-divinity/docs/archive/GDD-LEGACY.md` is a historical predecessor and
must not direct implementation.

## 3. Product Intent

Broken Divinity is a survival game about maintaining continuity inside the ruins of sacred legitimacy.

The Foundation MVP does not need to deliver the complete theological, faction, sanity, overworld, or narrative vision. It must deliver a stable structure that can later support those systems without replacement.

The foundation must already demonstrate:

- a physical shelter that persists;
- survivors, stations, assignments, resources, and production;
- preparation and visible resource consequences;
- one deterministic hand-authored dungeon;
- movement, exploration, combat, loot, defeat, and extraction;
- return to the shelter with results applied exactly once;
- practical skill improvement through actions;
- at least two representative virtue-expression hooks;
- two data-driven placeholder factions with behavior consistent with their data;
- deterministic and resumable state;
- a clear Ratatui player path without debug commands.

## 4. Scope

### 4.1 Required

- Bevy-Ratatui/Ratatui application shell.
- One fixed shelter.
- Three starter survivors.
- Five existing station types.
- Station placement and survivor assignment.
- One deterministic daily production/resource cycle.
- One fixed dungeon.
- One enemy archetype.
- One healing item.
- Tactical movement and combat.
- Explicit pickup, item use or storage, and extraction.
- Distinct extraction and defeat results.
- Colony storage for extracted loot.
- Melee, ranged, repair, and medicine progression types.
- At least one skill gain in the canonical scenario.
- At least two virtue-expression hooks in the canonical scenario.
- Exactly two stable, non-canonical placeholder faction records.
- Save/load for colony, dungeon, post-extraction, and defeat state.
- Deterministic continuation for the same saved state and action sequence.
- Content and configuration validation with readable failures.

### 4.2 Explicitly deferred

- Procedural dungeon generation in the foundation path.
- Full overworld travel.
- Raids.
- Colony events.
- Sanity.
- Theology-driven rules.
- Faction reputation and diplomacy.
- Final faction canon.
- Gabriel and deeper narrative integration.
- Multiple dungeon themes or floors.
- Full perk trees.
- Asset-heavy or graphical presentation.
- Standalone roguelike kernel proof.

Deferred code may remain compiled and tested in isolation, but it must not affect the Foundation runtime.

### 4.3 Prohibited scope expansion

Recovery work must not add:

- new enemy rosters;
- new dungeon themes;
- deeper crafting;
- final narrative content;
- reputation;
- procgen entry paths;
- raids or events;
- additional save slots;
- unrelated visual redesign.

Useful work outside the required list is recorded as Product P2/P3 and not implemented during recovery.

## 5. Current Evidence Baseline

The 2026-07-24 audit established:

- `cargo fmt --all -- --check` passes;
- `cargo check --workspace --quiet` passes;
- `cargo test --workspace --quiet` passes with 272 tests and 2 ignored tests;
- `cargo run -p bd_app -- --validate` passes;
- `git diff --check` passes;
- seven compiler warnings remain;
- the worktree contains a large uncommitted foundation implementation;
- a clean uninterrupted terminal run can enter the dungeon, fight, pick up/use loot, extract, and return;
- terminal building and survivor assignment can be initiated;
- save/load, defeat, full colony continuity, and the complete canonical scenario are not yet proven.

Passing unit tests do not override a failed acceptance gate.

## 6. Confirmed Recovery Defects

Each defect has a stable ID. Tests, commits, and status reports must reference these IDs.

| ID | Owner | Confirmed defect | Required outcome |
|---|---|---|---|
| FR-001 | Phase 0 | Root and repository-local GDD files compete for authority | One active root GDD; historical copy clearly archived/classified |
| FR-002 | Phase 0 | `ACTIVE-PLAN.md` and Phase 11 contract contradict each other | One honest recovery status |
| FR-003 | Phase 1 | Foundation round-trip test teleports the player and directly inserts loot | Canonical test uses production intents only |
| FR-004 | Phase 1 | Legacy `mvp.rs` tests use procgen and direct mutation | Legacy tests classified; foundation acceptance separated |
| FR-005 | Phase 2 | Built stations lack persistent/location scope | Stations survive dungeon round trips |
| FR-006 | Phase 2 | Shelter resource nodes lack persistent/location scope | Resource nodes survive and remain colony-only |
| FR-007 | Phase 2 | Colony entities can remain visible to unscoped tactical queries | Queries respect entity location scope |
| FR-008 | Phase 3 | Save snapshots omit survivors, tasks, stations, station types, resource nodes, and lifecycle markers | Complete MVP state round-trips |
| FR-009 | Phase 3 | `OutpostState` can retain stale Bevy `Entity` references after load | Restored state uses stable save IDs and rebuilt references |
| FR-010 | Phase 3 | Save filenames use a turn value that resets every day | Save selection cannot choose or overwrite the wrong state |
| FR-011 | Phase 3 | Combat RNG stream position is not restored | Saved deterministic continuation is repeatable |
| FR-012 | Phases 3–4 | Replay records only action IDs | Replay records complete typed action data |
| FR-013 | Phase 4 | Terminal attack uses `ability.attack` while melee progression metadata uses `ability.quick_attack` | Player attack and progression use one canonical action |
| FR-014 | Phase 5 | Player blueprint lacks Justice, Fortitude, and Kleos state | All canonical virtue state exists and persists |
| FR-015 | Phase 5 | Kleos is awarded for every defeated entity | Kleos requires an explicit notable achievement |
| FR-016 | Phase 5 | Canonical path does not prove two virtue hooks | Two hooks execute visibly through normal actions |
| FR-017 | Phase 5 | Faction records say neutral while combat treats every non-player as hostile | Typed faction disposition governs AI and targeting |
| FR-018 | Phase 4 | Pickup and station assignment bypass the shared `ActionIntent` contract | Foundation gameplay follows one validated action pipeline |
| FR-019 | Phase 6 | Daily survivor consequences poll `turn == 0` and can be frame-sensitive | One explicit day boundary drives each daily effect once |
| FR-020 | Phase 8 | Content validation does not prove reachability, walkability, collision safety, virtue IDs, disposition values, or runtime action links | Invalid foundation content fails before play with a readable report |
| FR-021 | Phase 8 | Invalid startup content panics | Application reports a controlled content/configuration error |
| FR-022 | Phase 7 | Configured keybindings do not drive actual TUI input | Input, help, footer, and action hints share one binding source |
| FR-023 | Phase 7 | Save/load controls and contextual actions are not fully discoverable | Mode-aware help exposes all supported controls |
| FR-024 | Phases 2–8 | Input, action, spatial, save, and screen modules mix excessive responsibilities | Recovery changes establish bounded owners without a rewrite |

New confirmed defects must be added to this table before implementation. Suspicions remain in phase notes until reproduced by a failing test or direct code proof.

## 7. Locked Recovery Decisions

These decisions remove implementation ambiguity. Changing one requires an update to `DECISIONS-TO-LOCK.md` before code changes continue.

### R-01 — Recovery, not rewrite

Existing crate boundaries remain:

- `bd_app` owns application startup, paths, persistence orchestration, and terminal lifecycle;
- `bd_core` owns simulation, authoritative state, actions, transitions, results, and save data;
- `bd_data` owns content loading and validation;
- `bd_tui` owns semantic input binding, view models, and rendering;
- `bd_test_support` owns production-path test drivers and fixtures, not alternate gameplay behavior.

Modules may be split internally when required by SRP. Crates are not replaced.

### R-02 — One canonical foundation action pipeline

All player-visible foundation gameplay resolves through:

```text
ActionIntent
  → typed validation
  → cost resolution
  → effect emission
  → mutation resolver
  → typed result or denial
  → log/view-model projection
```

Pickup, item use, extraction, station building, station assignment, survivor task assignment, movement, combat, guard, repair, and waiting must follow this contract.

Specialized internal messages may exist after an accepted action, but they may not become alternate validation or mutation entry points.

### R-03 — One entity scope model

Foundation entities receive one authoritative scope classification:

- `RunPersistent` — player and state that survives location changes;
- `ColonyPersistent` — survivors, stations, shelter resource nodes, and colony fixtures;
- `DungeonTransient` — enemies, exit fixtures, and dungeon loot until extraction resolves.

Exact Rust naming may change during refactoring, but behavior may not.

Location-scoped queries must not infer scope from the absence of another marker. Cleanup must act on explicit scope.

### R-04 — Stable saved relationships

Runtime Bevy `Entity` values are never persisted as durable identity.

Snapshots use `SaveId` for:

- survivor-to-station assignment;
- containment;
- equipment;
- ownership;
- summons;
- outpost membership;
- any other cross-entity relationship.

Load validates all references before committing the restored state.

### R-05 — One manual MVP save slot

The Foundation MVP exposes one manual save slot:

- F5 atomically replaces the slot;
- F9 loads the slot;
- a temporary file is validated before replacement;
- no save is selected by within-day turn number;
- missing, incompatible, or corrupt saves produce readable results;
- multiple slots and autosaves are deferred.

The project has no released compatibility contract for current development saves. The save version may be bumped, and incompatible earlier development saves may be rejected with a readable message rather than silently migrated.

### R-06 — Deterministic random continuation

Determinism uses explicit run random state:

- run seed;
- named random stream or domain;
- persisted draw index/counter.

Loading the same snapshot and applying the same action sequence must produce the same combat results. Restarting the process must not change that result.

### R-07 — Complete typed replay records

Replay remains part of the Foundation deterministic proof. Each record includes:

- action ID;
- direction or position parameter;
- stable target/content identity where applicable;
- deterministic sequence number.

Pickup and colony actions are included. A list of action-name strings is not accepted as deterministic replay evidence.

### R-08 — Canonical combat action

`ability.quick_attack` is the canonical Foundation melee action because current content maps it to melee and Thumos.

The terminal attack command must resolve to this action. `ability.attack` is treated as a legacy compatibility path and must be deprecated, adapted, or removed from Foundation registration after test coverage is migrated.

### R-09 — Virtue state and Kleos

The player carries state for all six virtues:

- Temperance;
- Justice;
- Prudence;
- Fortitude;
- Thumos;
- Metis.

Kleos remains separate but persistable.

The Foundation scenario must exercise two representative virtue hooks. Full balancing remains deferred.

Kleos is never granted by a generic enemy-defeat message. It requires an explicit notable-achievement result.

### R-10 — Typed Foundation faction disposition

Foundation faction data uses a typed disposition toward the player:

- Hostile;
- Neutral.

`faction.placeholder_a` is hostile for the fixed enemy encounter. `faction.placeholder_b` is neutral and exists to prove that neutral data does not produce hostile behavior.

AI targeting and player hostility validation consult the same faction relationship service. Reputation remains deferred.

### R-11 — One explicit day boundary

Turn advancement emits one `DayAdvanced` result when the day changes.

The following consume that result exactly once:

- survivor food consumption;
- staffed-station production;
- gathering;
- starvation and mood consequences;
- daily summary logging.

Daily systems do not poll application frames or independently track day changes in unrelated local state.

### R-12 — Semantic input ownership

`bd_tui` owns:

- semantic input commands;
- validated command bindings;
- mode availability metadata;
- player-facing labels and key hints.

`bd_app` loads configuration and injects validated bindings. It does not define a second command catalog.

Actual input handling, help, footer text, and action-panel hints derive from the same command descriptors.

### R-13 — Tests prove behavior through production paths

Foundation acceptance tests may perform fixture setup, but they may not directly mutate gameplay state to manufacture success.

Forbidden in acceptance tests:

- teleporting the player;
- directly changing health;
- directly inserting inventory containment;
- directly changing progression or virtue values;
- directly changing colony resources;
- directly marking extraction;
- directly setting survivor assignment.

Unit tests may construct focused state to test a resolver. Such tests must be labeled and must not count as canonical acceptance evidence.

### R-14 — Documentation is preserved and classified

Superseded documentation is moved or marked Historical/Deprecated with a replacement link. It is not silently deleted.

Only one document may claim to be the active execution plan.

## 8. Implementation Protocol

Every implementation phase follows this order:

1. Re-read the relevant GDD section and locked decisions.
2. State the behavior contract and failure cases.
3. Add a failing test through the intended production path.
4. Confirm the test fails for the expected reason.
5. Implement the smallest behavior that passes.
6. Refactor while tests remain green.
7. Run phase-specific tests.
8. Run workspace validation.
9. Perform the phase GDD drift check.
10. Record evidence using the status template.
11. Begin the next phase only after the exit gate passes.

No phase is complete because code compiles.

## 9. Ambiguity and Stop Protocol

Implementation must stop when:

- a requested behavior conflicts with the GDD;
- two canonical documents disagree;
- a new product choice is required;
- preserving existing work would materially change scope;
- save compatibility with unknown external data becomes required;
- a phase cannot pass without activating a deferred system;
- the same state appears to have multiple authoritative owners;
- an action cannot be routed through the shared pipeline without changing product behavior.

When stopped:

1. record the ambiguity in the current phase evidence;
2. identify the exact conflicting sources or code paths;
3. propose the smallest mutually exclusive choices;
4. ask the project owner;
5. update the decision register after the answer;
6. resume only when the authority documents are consistent.

Agents may choose local implementation details only when all choices preserve the locked behavior, ownership, and exit gate.

## 10. Phase 0 — Documentation Authority and Baseline Control

### Purpose

Remove documentation ambiguity and protect the current implementation before recovery coding begins.

### Dependencies

None. Phase 0 is mandatory.

### Phase 0A — Protect the implementation baseline

1. Record:
   - branch and commit;
   - `git status --short`;
   - `git diff --stat`;
   - untracked foundation content;
   - current test and validation results.
2. Create an owner-approved Git checkpoint before reorganizing code or documents.
3. Create a SHA-256 manifest and timestamped archive copy of canonical/root documentation that lives outside the nested `broken-divinity` Git repository.
4. Do not mix documentation cleanup with gameplay implementation.
5. Record seven current compiler warnings as baseline issues rather than silently accepting new warnings.

### Phase 0B — Establish the active documentation set

The top-level active documentation set becomes:

- `GDD.md`;
- `Kernel.md`;
- `Kernel-direction.md`;
- `docs/README.md`;
- `docs/DECISIONS-TO-LOCK.md`;
- `docs/MIGRATION-AND-DEPRECATION.md`;
- `docs/MVP-SCENARIO.md`;
- `docs/DOCUMENT-INVENTORY.md`;
- this plan.

During recovery, `docs/README.md` names this document as the sole execution
plan. After the final gate, it names this document as the completed Foundation
evidence record and states that no later product plan is active.

### Phase 0C — Classify superseded planning records

Move or mark these as Historical:

- `plan.md`;
- `docs/ACTIVE-PLAN.md`;
- `docs/PHASE-3-CONTENT-CONTRACT.md`;
- `docs/PHASE-5-PROGRESSION-CONTRACT.md`;
- `docs/PHASE-6-DUNGEON-VERTICAL-SLICE.md`;
- `docs/PHASE-7-COLONY-RETURN-CONTRACT.md`;
- `docs/PHASE-8-FACTION-CONTRACT.md`;
- `docs/PHASE-9-PERSISTENCE-CONTRACT.md`;
- `docs/PHASE-10-UX-CONTRACT.md`;
- `docs/PHASE-10-COMBAT-LOOP-REMEDIATION.md`;
- `docs/PHASE-11-MVP-AUDIT-CONTRACT.md`.

Required destination:

```text
docs/archive/foundation-plan-2026-07-24/
```

Each moved document receives or retains:

- original title;
- Historical status;
- date archived;
- replacement link to this plan;
- note that completion claims were not accepted as final MVP evidence.

### Phase 0D — Reconcile the two documentation trees

Create `docs/DOCUMENT-INVENTORY.md` listing every document under:

- root project files;
- `docs/`;
- `docs/design/`;
- `broken-divinity/docs/`.

Classify each as:

- Active;
- Reference;
- Reconcile;
- Historical;
- Deprecated.

Required classifications:

- `broken-divinity/docs/archive/GDD-LEGACY.md` → Historical;
- root `GDD.md` → Active product authority;
- `docs/design/` → non-authoritative design/lore reference unless promoted through the root GDD;
- `broken-divinity/docs/gameplay/`, `lore/`, and `ui/` → Reconcile against corresponding `docs/design/` copies;
- `broken-divinity/docs/decisions/` → historical technical decision records unless reaffirmed;
- current architecture guardrails and dependency records → Reference, subordinate to `Kernel.md`.

Identical duplicates may be consolidated only after hashes and inbound links are recorded. No unique content is deleted.

### Phase 0E — Correct status and navigation

1. Update the documentation hub reading order.
2. Update `MIGRATION-AND-DEPRECATION.md`.
3. Correct the canonical GDD displayed update date or record why it differs.
4. Add replacement links from superseded plans.
5. Update the repository README so controls, save location, and document authority are not false.
6. Verify Markdown links after moves.

### Tests and validation

No gameplay tests are added in Phase 0.

Run:

```text
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo run -p bd_app -- --validate
git diff --check
```

Documentation checks:

- exactly one active GDD;
- exactly one active execution plan;
- every archived document has a replacement link;
- no active document points to archived phase work as current authority;
- no unique document is deleted;
- document inventory covers both documentation trees.

### GDD drift check

Review GDD Sections 1–10 and D-01 through D-15. Phase 0 changes authority and classification only; it must not alter product scope.

### Exit gate

- The implementation baseline is recoverable.
- Documentation authority is unambiguous.
- This plan is linked as the only active execution plan.
- Superseded plans are preserved and classified.
- Both documentation trees have an inventory and owner.

Do not begin gameplay coding until this gate passes.

### Phase 0 completion evidence — 2026-07-24

Phase 0 passed its exit gate:

- nested-repository baseline checkpoint: `1f45ef1`;
- recoverable external-document baseline:
  `docs/archive/bd-phase0-docs-baseline-2026-07-24.tar.gz`;
- baseline hash and provenance record:
  `docs/archive/PHASE-0-BASELINE-SHA256.md`;
- superseded plans preserved under:
  `docs/archive/foundation-plan-2026-07-24/`;
- repository-local legacy GDD and development plan preserved under:
  `broken-divinity/docs/archive/`;
- complete classification and ownership map:
  `docs/DOCUMENT-INVENTORY.md`;
- active-document link check passed;
- authority search found one active product GDD and one active execution plan;
- `cargo fmt --all -- --check` passed;
- `cargo check --workspace --quiet` passed;
- `cargo test --workspace --quiet` passed with 272 tests passed and 2 ignored;
- `cargo run -p bd_app -- --validate` passed;
- `git diff --check` passed after documentation cleanup;
- seven pre-existing compiler warnings remain recorded as baseline debt;
- GDD Sections 1–10 and locked decisions D-01 through D-15 were reviewed;
  no product scope was changed during documentation cleanup.

This evidence authorizes Phase 1 preflight. It does not claim that the
Foundation MVP itself is complete.

## 11. Phase 1 — Trustworthy Foundation Acceptance Harness

### Purpose

Create tests that expose real integration failures instead of manufacturing success.

### Dependencies

Phase 0 complete.

### Tests first

Create the dedicated Foundation acceptance target:

```text
crates/bd_app/tests/foundation_scenario.rs
```

Add failing tests:

- `clean_launch_reaches_colony`;
- `foundation_app_does_not_register_deferred_systems`;
- `fixed_dungeon_loads_without_procgen`;
- `canonical_colony_setup_uses_actions`;
- `canonical_dungeon_run_uses_actions`;
- `canonical_extraction_applies_loot_once`;
- `canonical_defeat_awards_no_loot`;
- `colony_to_title_restart_is_legal`;
- `defeat_to_title_restart_is_legal`;
- `canonical_progression_improves_one_skill`;
- `canonical_progression_emits_two_virtues`;
- `canonical_colony_state_survives_round_trip`;
- `canonical_save_load_resumes_state`;
- `same_snapshot_and_actions_match`.

### Implementation approach

1. Build one test driver in `bd_test_support`.
2. Use the same Foundation content loader as the application.
3. Use `BdFoundationPlugin`.
4. Drive gameplay through `ActionIntent` and authoritative transition requests.
5. Advance Bevy schedules explicitly and deterministically.
6. Provide read-only state summaries for assertions.
7. Do not add alternate resolver behavior to test support.
8. Keep terminal rendering out of the headless scenario.

### Legacy test classification

Classify current `crates/bd_app/tests/mvp.rs` cases:

- valid unit/integration regression;
- procgen/deferred regression;
- invalid Foundation acceptance due to direct mutation.

Rename or relocate the target so it cannot be mistaken for the Foundation MVP acceptance suite.

### Internal validation

- The canonical scenario fails at the first real unsupported behavior.
- No acceptance helper teleports entities or inserts result components.
- Test output identifies the failed scenario step.
- Existing unit tests remain available as regression coverage.

### GDD drift check

Compare scenario steps against GDD Sections 3, 6, 7, and 8 and D-10 through D-15.

### Exit gate

The full canonical test exists, is honest, and provides the ordered failing queue for subsequent phases.

### Phase 1 completion evidence — 2026-07-24

```text
Phase: 1 — Trustworthy Foundation Acceptance Harness
Status: Complete
Scope: Production-path headless driver, canonical acceptance target, and legacy-test classification
Defect IDs addressed: FR-003, FR-004
Baseline commit: 47a2823
Tests added before implementation: 14 canonical Foundation acceptance tests
Expected red failure: bd_test_support::FoundationDriver did not exist
Implementation files:
  - crates/bd_app/tests/foundation_scenario.rs
  - crates/bd_test_support/src/lib.rs
  - crates/bd_app/tests/legacy_kernel_regressions.rs
  - crates/bd_app/Cargo.toml
  - Cargo.lock
Phase commit: 39ee550
Validation commands:
  - cargo fmt --all -- --check
  - cargo check --workspace
  - cargo test --workspace --lib
  - cargo test -p bd_app --bin bd --test content_loading --test diagnostic
    --test legacy_kernel_regressions --test stress
  - cargo test -p bd_app --test foundation_scenario -- --test-threads=1
  - cargo run -p bd_app -- --validate
  - git diff --check
Automated results:
  - formatting, workspace check, content validation, and diff checks passed
  - all pre-existing unit and classified regression targets passed
  - canonical target compiled and reported 7 passing and 7 failing tests
  - failures identify the unsupported production steps listed below
Manual evidence: None required; the phase owns the headless harness
GDD sections reviewed: 3, 6, 7, 8
Decision IDs reviewed: D-10, D-11, D-12, D-13, D-14, D-15
Drift found: None
Documentation updated: This evidence record and the authorization correction
Known limitations:
  - no shelter player actor for colony actions
  - shelter resource nodes are lost on dungeon return
  - pickup has no canonical action
  - extraction with collected loot cannot yet complete through actions
  - the second representative virtue result is absent
  - survivor/resource-node save restoration is incomplete
  - colony-to-title restart is rejected
Exit gate result: PASS — the honest ordered recovery queue exists
```

The canonical driver exposes no mutable world handle. It submits production
`ActionIntent` and `TransitionIntent` messages, advances the real Bevy
schedules, invokes the production save/load boundary, and returns read-only
summaries. The old direct-mutation and procgen cases are preserved under the
explicitly non-authoritative `legacy_kernel_regressions` target.

## 12. Phase 2 — Entity Scope and Location Continuity

### Purpose

Make the shelter and run state survive location transitions while dungeon state remains isolated.

### Dependencies

Phase 1 failing tests.

### Tests first

- `built_station_survives_dungeon_round_trip`;
- `station_assignment_survives_dungeon_round_trip`;
- `resource_nodes_survive_dungeon_round_trip`;
- `colony_entities_do_not_block_dungeon_queries`;
- `dungeon_enemy_is_removed_on_extraction`;
- `uncollected_dungeon_loot_is_removed`;
- `player_survives_location_cleanup`;
- `carried_loot_reaches_extraction_resolver`;
- `cleanup_is_idempotent`.

### Implementation approach

1. Introduce the single scope model from R-03.
2. Assign scope at every production spawn boundary.
3. Make transition cleanup match explicit scope.
4. Add location filters to combat, movement, targeting, rendering, and colony queries.
5. Remove absence-based cleanup logic.
6. Keep legacy marker adapters only if required by deferred code.
7. Mark adapters Deprecated and prevent Foundation code from depending on them.

### Internal validation

- Colony state is unchanged by entering and leaving the dungeon.
- No tactical system reads colony-only entities.
- No colony system processes dungeon-only entities.
- Extraction transfers loot before dungeon cleanup.
- Cleanup warnings and double despawns do not occur.

### GDD drift check

Confirm the physical shelter persists and the fixed dungeon remains isolated. Do not introduce overworld or procgen behavior.

### Exit gate

The uninterrupted colony → dungeon → colony scenario preserves all colony entities and removes all dungeon-transient entities correctly.

### Phase 2 completion evidence — 2026-07-24

```text
Phase: 2 — Entity Scope and Location Continuity
Status: Complete
Scope: Explicit entity lifetime, location filtering, cleanup, and shelter player continuity
Defect IDs addressed: FR-005, FR-006, FR-007; FR-024 partially
Baseline commit: 39ee550
Tests added before implementation: 9 entity-scope acceptance tests
Expected red failure: EntityScope and scope-aware driver inspection did not exist
Implementation files:
  - crates/bd_core/src/spatial.rs
  - crates/bd_core/src/actions.rs
  - crates/bd_core/src/enemy_ai.rs
  - crates/bd_core/src/inventory.rs
  - crates/bd_core/src/colony/production.rs
  - crates/bd_core/src/colony/resources.rs
  - crates/bd_core/src/colony/survivors.rs
  - crates/bd_tui/src/lib.rs
  - crates/bd_tui/src/view_models.rs
  - crates/bd_test_support/src/lib.rs
  - crates/bd_app/tests/entity_scope.rs
Phase commit: 3730750
Validation commands:
  - cargo fmt --all -- --check
  - cargo check --workspace
  - cargo test --workspace --lib
  - cargo test -p bd_app --bin bd --test content_loading --test diagnostic
    --test entity_scope --test legacy_kernel_regressions --test stress
  - cargo test -p bd_app --test foundation_scenario -- --test-threads=1
  - cargo run -p bd_app -- --validate
  - git diff --check
Automated results:
  - all 9 entity-scope acceptance tests passed
  - all existing unit and classified regression tests passed
  - formatting, workspace check, content validation, and diff checks passed
  - canonical colony-state round trip changed from failing to passing
  - canonical queue remains honest at 7 passing and 7 failing tests
Manual evidence: Headless production-path round trip; terminal proof remains Phase 9
GDD sections reviewed: 3, 6, 8
Decision IDs reviewed: D-10, D-11, D-12, D-15
Drift found: None; no overworld or procgen path was introduced
Documentation updated: This evidence record
Known limitations:
  - EntityScope is not serialized until Phase 3
  - compatibility PersistentEntity/TransientEntity markers remain for legacy code
  - pickup and station assignment still use pre-Phase-4 specialized messages
Exit gate result: PASS
```

`EntityScope` is now the Foundation lifetime owner: `RunPersistent`,
`ColonyPersistent`, or `DungeonTransient`. Spawn boundaries assign it
explicitly; cleanup matches only `DungeonTransient`; action validation,
movement, targeting, enemy AI, inventory, colony processing, TUI input, and
view-model construction respect the active location. The compatibility marker
types remain documented as deprecated adapters and do not drive Foundation
cleanup.

## 13. Phase 3 — Complete Save, Load, and Determinism

### Purpose

Make persistence represent and resume the actual Foundation state.

### Dependencies

Phase 2 scope model stable.

### Tests first

- `save_load_colony_preserves_survivors_stations_and_assignments`;
- `save_load_dungeon_preserves_player_enemy_item_and_scope`;
- `save_load_post_extraction_does_not_reapply_loot`;
- `save_load_defeat_preserves_outcome`;
- `load_rebuilds_outpost_entity_references`;
- `load_rejects_missing_relationship_reference`;
- `manual_slot_replaces_atomically`;
- `latest_state_does_not_depend_on_turn_within_day`;
- `combat_rng_continues_after_load`;
- `same_snapshot_and_actions_match_after_process_restart`;
- `load_does_not_reapply_production_progression_or_virtues`.

### Implementation approach

1. Bump the save version.
2. Define snapshot records for all MVP components and resources.
3. Serialize entity scope.
4. Serialize survivor and station state.
5. Convert relationships to `SaveId`.
6. Validate the entire snapshot before clearing live state.
7. Build restored entities and relationship mappings in staging.
8. Commit restored state only after validation succeeds.
9. Reset or rebuild resources containing entity references.
10. Add explicit deterministic random stream state.
11. Replace turn-number filenames with the manual slot policy from R-05.
12. Use temporary-file write, validation, and atomic replacement.
13. Report readable missing, corrupt, version, and content mismatch errors.

### Internal validation

- Failed loads leave the current world unchanged.
- Loaded state can continue to extraction or defeat.
- The player, survivors, stations, items, and faction identities retain behavior, not just display data.
- No effect is applied twice.
- Save/load behavior is identical in headless and application paths.

### GDD drift check

Review the complete persistence boundary in `MVP-SCENARIO.md` and GDD Section 8. Persistence must not activate deferred systems.

### Exit gate

Colony, dungeon, post-extraction, and defeat snapshots all resume correctly and deterministically.

### Phase 3 completion evidence — 2026-07-24

```text
Phase: 3 — Complete Save, Load, and Determinism
Status: Complete
Scope: Complete Foundation snapshots, reference validation, deterministic RNG continuation, and one atomic manual slot
Defect IDs addressed: FR-008, FR-009, FR-010, FR-011; FR-012 remains owned by Phase 4
Baseline commit: 3730750
Tests added before implementation: 11 persistence acceptance tests
Expected red failure: FoundationDriver lacked Phase 3 persistence inspection and manual-slot operations
Implementation files:
  - crates/bd_app/tests/persistence.rs
  - crates/bd_core/src/save.rs
  - crates/bd_core/src/combat.rs
  - crates/bd_core/src/session.rs
  - crates/bd_core/src/actions.rs
  - crates/bd_core/src/inventory.rs
  - crates/bd_app/src/main.rs
  - crates/bd_test_support/src/lib.rs
Phase commit: 13ed27d
Validation commands:
  - cargo fmt --all -- --check
  - cargo check --workspace
  - cargo test --workspace --lib
  - cargo test -p bd_app --bin bd --test content_loading --test diagnostic
    --test entity_scope --test legacy_kernel_regressions --test persistence --test stress
  - cargo test -p bd_app --test foundation_scenario -- --test-threads=1
  - cargo run -p bd_app -- --validate
  - git diff --check
Automated results:
  - all 11 Phase 3 persistence acceptance tests passed
  - all 235 workspace library tests passed
  - all classified application/regression targets passed (one diagnostic and
    one real-terminal regression remain intentionally ignored)
  - formatting, workspace check, content validation, and diff checks passed
  - canonical queue advanced from 7 passing/7 failing to 9 passing/5 failing
  - canonical save/load, colony round trip, and process-restart determinism pass
Unexpected result resolved:
  - repeated outpost actions exposed a tactical enemy-phase lock leaking into
    colony play; the existing red day-boundary test now proves outpost turns
    advance without creating the tactical lock
Manual evidence: None required; terminal save/load proof remains Phase 9
GDD sections reviewed: 8
Decision IDs reviewed: D-15
Scenario sections reviewed: Persistence boundary and test ownership map
Drift found: None; persistence activates no deferred systems and applies no gameplay effect during load
Documentation updated: This evidence record and active-phase header
Known limitations:
  - complete typed replay records remain Phase 4 work under FR-012
  - compatibility PersistentEntity/TransientEntity markers are restored only
    as deprecated adapters while EntityScope remains authoritative
  - five canonical failures remain assigned to Phases 4 and 5
Exit gate result: PASS
```

Save version 4 now records Foundation entity scope, survivors and assignments,
stations, resource nodes, exit markers, stable relationship IDs, outpost party
references, and combat RNG state. The loader validates the complete snapshot
before clearing live entities, rebuilds runtime entity references, and restores
state without replaying effects. Application save/load uses one validated,
temporary-file-backed `manual-slot.ron` rather than turn-number file selection.

## 14. Phase 4 — Unified Foundation Actions

### Purpose

Remove alternate gameplay mutation paths and make results, denials, replay, and time advancement consistent.

### Dependencies

Phase 3 snapshot boundary stable.

### Tests first

- `terminal_attack_maps_to_quick_attack`;
- `pickup_resolves_through_action_pipeline`;
- `station_assignment_resolves_through_action_pipeline`;
- `survivor_task_assignment_resolves_through_action_pipeline`;
- `rejected_foundation_action_has_typed_reason`;
- `rejected_action_has_no_partial_mutation`;
- `accepted_action_advances_time_once`;
- `rejected_action_does_not_advance_time`;
- `replay_record_contains_action_parameters`;
- `replay_includes_pickup_and_colony_actions`.

### Implementation approach

1. Route the terminal attack command to `ability.quick_attack`.
2. Adapt or deprecate `ability.attack`.
3. Add canonical action definitions for pickup and station assignment.
4. Keep specialized inventory/assignment messages behind accepted action effects only.
5. Emit one typed `ActionResolved` or `ActionDenied`.
6. Record complete replay data after successful resolution.
7. Ensure progression observes only successful action results.
8. Ensure action failure cannot spend AP, move items, change tasks, or advance time.

### Internal validation

- One action has one validator and one mutation path.
- TUI and test drivers submit the same semantic action.
- Rejections are visible and typed.
- Enemy phase occurs at most once per accepted tactical action.

### GDD drift check

Compare against the required action semantics in `MVP-SCENARIO.md`. No direct gameplay mutation may be added to the TUI or test driver.

### Exit gate

All canonical player actions follow the shared pipeline and produce consistent results.

### Phase 4 completion evidence — 2026-07-24

```text
Phase: 4 — Unified Foundation Actions
Status: Complete
Scope: Canonical combat, pickup, colony assignment, typed denial, turn ownership, and parameterized replay
Defect IDs addressed: FR-012, FR-013, FR-018; FR-024 partially
Baseline commit: 13ed27d
Tests added before implementation: 10 Foundation action acceptance tests
Expected red failure: pickup/assignment action definitions, typed replay records,
  denial inspection, and terminal quick-attack mapping did not exist
Implementation files:
  - crates/bd_app/tests/foundation_actions.rs
  - crates/bd_core/src/actions.rs
  - crates/bd_core/src/colony/survivors.rs
  - crates/bd_core/src/inventory.rs
  - crates/bd_core/src/session.rs
  - crates/bd_core/src/lib.rs
  - crates/bd_tui/src/lib.rs
  - crates/bd_test_support/src/lib.rs
Phase commit: 7ee096f
Validation commands:
  - cargo fmt --all -- --check
  - cargo check --workspace
  - cargo test --workspace --lib
  - cargo test -p bd_core --test architecture
  - cargo test -p bd_app --bin bd --test content_loading --test diagnostic
    --test entity_scope --test foundation_actions
    --test legacy_kernel_regressions --test persistence --test stress
  - cargo test -p bd_app --test foundation_scenario -- --test-threads=1
  - cargo run -p bd_app -- --validate
  - git diff --check
Automated results:
  - all 10 Phase 4 action acceptance tests passed
  - all 235 workspace library tests and all 5 architecture tests passed
  - all classified application/regression targets passed
  - formatting, workspace check, content validation, and diff checks passed
  - canonical queue advanced from 9 passing/5 failing to 12 passing/2 failing
  - canonical colony setup, dungeon action loop, pickup, and extraction now pass
Unexpected results resolved:
  - the accepted pickup adapter initially rejected its own tactical phase lock;
    lock and time ownership now remain solely in the action pipeline
  - the acceptance driver selected the persistent shelter exit while tactical;
    read-only exit discovery now respects EntityScope
Manual evidence: None required; terminal interaction proof remains Phase 9
GDD sections reviewed: 7 and 8
Decision IDs reviewed: D-05, D-10, D-13, D-14, D-15
Scenario sections reviewed: Required action semantics, required outcomes, colony,
  dungeon, and persistence boundary
Drift found: None; no direct gameplay mutation was added to TUI or test support
Documentation updated: This evidence record and active-phase header
Known limitations:
  - representative Fortitude expression remains Phase 5
  - colony-to-title restart remains outside the Phase 4 action gate
  - Phase 7 will replace the temporary canonical key helper with configured bindings
Exit gate result: PASS
```

The terminal, headless driver, and gameplay systems now submit the same
`ActionIntent` semantics for quick attack, pickup, station assignment, and
survivor tasks. Pickup and station messages are mutation adapters emitted only
after shared validation. The action owner records actor, action ID, direction,
and target after success; rejected actions emit typed reasons and cannot spend,
mutate, enter an enemy phase, or advance time.

## 15. Phase 5 — Progression and Faction Semantics

### Purpose

Make the existing data-driven hooks affect normal play and ensure faction data governs behavior.

### Dependencies

Phase 4 canonical action IDs.

### Tests first

- `quick_attack_improves_melee`;
- `quick_attack_expresses_thumos`;
- `use_item_improves_medicine`;
- `use_item_expresses_temperance`;
- `rejected_action_grants_no_progression`;
- `player_has_all_six_virtues_and_kleos`;
- `generic_enemy_defeat_does_not_grant_kleos`;
- `progression_survives_save_load`;
- `two_foundation_factions_load`;
- `hostile_faction_drives_enemy_ai`;
- `neutral_faction_does_not_drive_enemy_ai`;
- `target_hostility_uses_faction_disposition`;
- `faction_identity_survives_save_load`.

### Implementation approach

1. Complete player virtue state.
2. Preserve content-driven skill/virtue metadata.
3. Make canonical player actions use that metadata.
4. Add visible skill and virtue results through view models/logs.
5. Remove generic Kleos-on-defeat behavior.
6. Add typed Foundation disposition.
7. Set placeholder A hostile and placeholder B neutral.
8. Make AI and action validation consume one faction relationship service.
9. Keep reputation and diplomacy unregistered.

### Internal validation

- The canonical scenario gains at least one skill.
- It emits at least two virtue hooks.
- Save/load does not duplicate gains.
- Neutral data produces neutral behavior.
- Adding a third faction record does not require a Rust match branch.

### GDD drift check

Review GDD Sections 5, 7, and 8 plus D-03, D-05, D-13, and D-14. Do not add final faction canon, reputation, or theological balance.

### Exit gate

Progression and faction identity are mechanically true in normal player behavior, not merely present in data.

### Phase 5 completion evidence — 2026-07-24

```text
Phase: 5 — Progression and Faction Semantics
Status: Complete
Scope: Complete virtue state, action-driven progression, representative virtue hooks, and typed faction behavior
Defect IDs addressed: FR-014, FR-015, FR-016, FR-017
Baseline commit: 7ee096f
Tests added before implementation: 11 Phase 5 integration tests plus focused
  neutral-AI and generic-Kleos unit regressions
Expected red evidence: canonical_progression_emits_two_virtues failed because
  the player lacked Fortitude state; both faction records were also neutral strings
Implementation files:
  - crates/bd_app/tests/progression_factions.rs
  - content/blueprints/foundation.ron
  - content/factions/foundation.ron
  - crates/bd_core/src/content.rs
  - crates/bd_core/src/factions.rs
  - crates/bd_core/src/actions.rs
  - crates/bd_core/src/enemy_ai.rs
  - crates/bd_core/src/progression.rs
  - crates/bd_core/src/virtues.rs
  - crates/bd_data/src/loader.rs
  - crates/bd_test_support/src/lib.rs
Phase commit: 1cc7b38
Validation commands:
  - cargo fmt --all -- --check
  - cargo check --workspace
  - cargo test --workspace --lib
  - cargo test -p bd_core --test architecture
  - cargo test -p bd_app --bin bd --test content_loading --test diagnostic
    --test entity_scope --test foundation_actions
    --test legacy_kernel_regressions --test persistence
    --test progression_factions --test stress
  - cargo test -p bd_app --test foundation_scenario -- --test-threads=1
  - cargo run -p bd_app -- --validate
  - git diff --check
Automated results:
  - all 11 Phase 5 integration tests passed
  - all 236 workspace library tests and all 5 architecture tests passed
  - all classified application/regression targets passed
  - formatting, workspace check, content validation, and diff checks passed
  - canonical queue advanced from 12 passing/2 failing to 13 passing/1 failing
  - canonical skill gain and two representative virtue hooks now pass
Manual evidence: None required; terminal presentation proof remains Phase 9
GDD sections reviewed: 5, 7, and 8
Decision IDs reviewed: D-03, D-05, D-13, D-14
Drift found: None; factions remain placeholders and reputation/diplomacy remain unregistered
Documentation updated: This evidence record and active-phase header
Known limitations:
  - Kleos has state but no generic-enemy award; notable achievements remain future content
  - richer faction effects, reputation, and final faction canon remain deferred
  - colony-to-title restart is the sole remaining canonical acceptance failure
Exit gate result: PASS
```

The Foundation player now owns all six virtues plus Kleos. Successful canonical
actions improve their declared skills and visibly express content-declared
virtues; rejected actions do neither. Generic enemies grant combat-survival
Fortitude but no Kleos. Placeholder A is typed Hostile, placeholder B is typed
Neutral, and the same content-backed disposition service controls both AI and
target validation without registering reputation.

## 16. Phase 6 — Exact-Once Colony Day Cycle

### Purpose

Make colony continuity observable through one deterministic resource and production cycle.

### Dependencies

Phase 2 scope and Phase 3 persistence.

### Tests first

- `day_advanced_emits_once`;
- `staffed_station_produces_once_per_day`;
- `unstaffed_station_does_not_produce`;
- `survivors_consume_food_once_per_day`;
- `starvation_consequence_applies_once_per_day`;
- `gathering_applies_once_per_day`;
- `daily_summary_matches_resource_delta`;
- `save_before_day_boundary_does_not_duplicate_cycle`;
- `save_after_day_boundary_does_not_duplicate_cycle`.

### Implementation approach

1. Emit one `DayAdvanced` result from authoritative time advancement.
2. Route production, gathering, consumption, and mood consequences from it.
3. Remove independent frame polling and unrelated `Local<u64>` day ownership.
4. Produce one typed daily summary.
5. Preserve current five station types.
6. Do not expand resource or crafting scope.

### Internal validation

- One day causes one production/consumption batch.
- Resource totals can be explained from visible assignments.
- Idle render frames never change colony state.
- The daily cycle is deterministic across save/load.

### GDD drift check

Review GDD Section 6 and D-12. Raids and events remain inactive.

### Exit gate

A player can build, assign, advance one day, and observe the exact expected colony resource result.

### Phase 6 completion evidence — 2026-07-24

```text
Phase: 6 — Exact-Once Colony Day Cycle
Status: Complete
Scope: Authoritative day boundary, deterministic colony transaction, typed daily summary, and save/load continuity
Defect IDs addressed: FR-019
Baseline commit: 1cc7b38
Tests added before implementation: 9 Phase 6 integration tests
Expected red evidence: Phase 6 tests failed to compile because DayAdvanced,
  DailySummary, resource observation, and day-cycle driver support did not exist
Implementation files:
  - crates/bd_app/tests/colony_day_cycle.rs
  - crates/bd_core/src/time.rs
  - crates/bd_core/src/colony/production.rs
  - crates/bd_core/src/colony/resources.rs
  - crates/bd_core/src/colony/survivors.rs
  - crates/bd_core/src/lib.rs
  - crates/bd_core/src/pools.rs
  - crates/bd_core/src/save.rs
  - crates/bd_test_support/src/lib.rs
Phase commit: 087bf8b
Validation commands:
  - cargo fmt --all -- --check
  - cargo check --workspace
  - cargo test --workspace --lib
  - cargo test -p bd_core --test architecture
  - cargo test -p bd_app --bin bd --test colony_day_cycle
    --test content_loading --test diagnostic --test entity_scope
    --test foundation_actions --test legacy_kernel_regressions
    --test persistence --test progression_factions --test stress
  - cargo test -p bd_app --test foundation_scenario -- --test-threads=1
  - cargo run -p bd_app -- --validate
  - git diff --check
Automated results:
  - all 9 Phase 6 integration tests passed
  - all workspace library tests and all 5 architecture tests passed
  - all classified application/regression targets passed
  - formatting, workspace check, content validation, and diff checks passed
  - canonical queue remains 13 passing/1 failing
Manual evidence: Headless build/assign/day-advance/resource observation;
  terminal presentation proof remains Phase 9
GDD sections reviewed: 6
Decision IDs reviewed: D-12
Drift found: None; five stations remain, while raids, events, crafting, and
  resource expansion remain deferred
Documentation updated: This evidence record and active-phase header
Known limitations:
  - the daily summary is persisted and logged but is not yet projected into
    dedicated TUI guidance; Phase 7 owns presentation
  - colony-to-title restart remains the sole canonical acceptance failure and
    is assigned to the final session-lifecycle audit
Exit gate result: PASS
```

One authoritative `DayAdvanced` message now starts the colony transaction.
Production, food consumption, gathering, and starvation consequences run once
from that boundary, and one typed summary records the resulting resource
deltas. The latest completed transaction survives save/load, while idle frames
cannot replay it.

## 17. Phase 7 — Input, Help, and Terminal Clarity

### Purpose

Make the working mechanics discoverable and ensure configuration matches actual input.

### Dependencies

Canonical actions and state stable.

### Tests first

- `configured_binding_emits_expected_command`;
- `help_uses_configured_binding`;
- `footer_uses_configured_binding`;
- `action_panel_uses_configured_binding`;
- `colony_help_lists_only_colony_actions`;
- `dungeon_help_lists_only_dungeon_actions`;
- `save_and_load_are_discoverable`;
- `no_target_attack_displays_denial`;
- `actions_panel_matches_kernel_availability`;
- `minimum_terminal_layout_preserves_required_controls`.

### Implementation approach

1. Define semantic commands and descriptors in `bd_tui`.
2. Inject validated bindings from `bd_app`.
3. Split global, colony, dungeon, build-mode, and game-over routing.
4. Derive help, footer, and action hints from descriptors.
5. Keep widgets dependent on view models only.
6. Diagnose and repair the existing `ActionListViewModel` pipeline.
7. Move quit and terminal teardown requests to the application boundary.
8. Define named minimum terminal dimensions and compact behavior.
9. Show save/load success and failure.
10. Make normal missing-config fallback informational.

### Internal validation

- Displayed controls always work.
- Rebinding a command changes all input/help projections together.
- No mode advertises an invalid action.
- Invalid actions never fail silently.
- Required controls fit the supported terminal size.

### GDD drift check

Confirm Ratatui remains the only player-facing runtime and no graphical/asset scope enters recovery.

### Exit gate

A first-time player can complete the canonical scenario using visible terminal guidance.

### Phase 7 completion evidence — 2026-07-24

```text
Phase: 7 — Input, Help, and Terminal Clarity
Status: Complete
Scope: Semantic commands, configured input, contextual guidance, truthful action availability, and safe terminal lifecycle
Defect IDs addressed: FR-022, FR-023
Baseline commit: 087bf8b
Tests added before implementation: 10 Phase 7 integration tests
Expected red evidence: Phase 7 tests failed to compile because the semantic
  command, binding, guidance, action-projection, and terminal-layout APIs did not exist
Implementation files:
  - crates/bd_tui/src/commands.rs
  - crates/bd_tui/src/lib.rs
  - crates/bd_tui/src/view_models.rs
  - crates/bd_tui/src/screens.rs
  - crates/bd_tui/tests/input_help.rs
  - crates/bd_app/src/config.rs
  - crates/bd_app/src/main.rs
  - crates/bd_app/tests/foundation_actions.rs
  - config/default.toml
Phase commit: 3c378cb
Validation commands:
  - cargo fmt --all -- --check
  - cargo check --workspace
  - cargo test --workspace --lib
  - cargo test -p bd_core --test architecture
  - cargo test -p bd_tui --test input_help
  - cargo test -p bd_app --bin bd --test colony_day_cycle
    --test content_loading --test diagnostic --test entity_scope
    --test foundation_actions --test legacy_kernel_regressions
    --test persistence --test progression_factions --test stress
  - cargo test -p bd_app --test foundation_scenario -- --test-threads=1
  - cargo run -p bd_app -- --validate
  - git diff --check
Automated results:
  - all 10 Phase 7 integration tests passed
  - all workspace library tests and all 5 architecture tests passed
  - all classified application/regression targets passed
  - shipped default config parses, validates, and rejects contextual key conflicts
  - formatting, workspace check, content validation, and diff checks passed
  - canonical queue remains 13 passing/1 failing
Manual evidence:
  - Ratatui launched at 80x24, title guidance rendered, and a first key entered the outpost
  - outpost action panel and contextual help exposed movement, wait, build,
    assignment, travel, save, load, help, and quit using the active bindings
  - application-boundary quit exited cleanly and restored the terminal
GDD sections reviewed: 3 and 6
Decision IDs reviewed: D-02 and D-06
Drift found: None; Ratatui remains the sole runtime and no asset or graphical scope entered
Documentation updated: This evidence record and active-phase header
Known limitations:
  - terminal play proof covered launch, outpost guidance, contextual help, and
    clean quit; the complete terminal journey remains the Phase 9 audit
  - colony-to-title restart remains the sole canonical acceptance failure and
    is assigned to the final session-lifecycle audit
Exit gate result: PASS
```

One semantic command catalog now drives configured key input, contextual help,
the footer, and action-panel hints. Colony and dungeon guidance no longer
advertise each other's controls, disabled actions explain why, save/load remain
discoverable, compact layouts reserve control space, and quit is requested from
the TUI but executed at the application boundary.

## 18. Phase 8 — Content and Failure-Path Hardening

### Purpose

Prevent valid-looking but unplayable content and configuration from entering the Foundation runtime.

### Dependencies

Stable action, faction, progression, and location schemas.

### Tests first

- `dungeon_entrance_must_be_walkable`;
- `dungeon_extraction_must_be_walkable`;
- `dungeon_extraction_must_be_reachable`;
- `placement_must_be_on_walkable_tile`;
- `placements_must_not_overlap_illegally`;
- `virtue_reference_must_exist`;
- `faction_disposition_must_be_valid`;
- `content_action_must_be_registered`;
- `required_player_virtue_state_is_validated`;
- `invalid_content_returns_readable_application_error`;
- `invalid_config_returns_readable_application_error`.

### Implementation approach

1. Add focused validators under the existing validation facade.
2. Keep validation errors typed and file/record specific.
3. Validate runtime action linkage after content and action registration are available.
4. Replace startup panic with controlled application failure reporting.
5. Keep schemas limited to current Foundation needs.

### Internal validation

- Every required record is validated.
- Fixed dungeon validity is proven before entity spawn.
- Invalid content cannot partially initialize the game.
- Third-party/deferred content cannot silently alter Foundation scope.

### GDD drift check

Confirm validators enforce the fixed Foundation slice only and do not require procgen, reputation, sanity, raids, or final lore.

### Exit gate

Valid Foundation content always produces a playable scenario; invalid content fails clearly before play.

### Phase 8 completion evidence — 2026-07-24

```text
Phase: 8 — Content and Failure-Path Hardening
Status: Complete
Scope: Foundation content topology, runtime linkage, configuration, and controlled startup failure
Defect IDs addressed: FR-020, FR-021
Baseline commit: 3c378cb
Tests added before implementation: 11 Phase 8 unit/application tests
Expected red evidence: Phase 8 tests failed to compile because the focused
  Foundation validator and runtime action-linkage API did not exist
Implementation files:
  - crates/bd_data/src/loader.rs
  - crates/bd_core/src/lib.rs
  - crates/bd_app/src/config.rs
  - crates/bd_app/src/main.rs
  - crates/bd_app/tests/content_loading.rs
  - crates/bd_tui/src/theme.rs
  - crates/bd_tui/src/visual.rs
Phase commit: 7f7bee8
Validation commands:
  - cargo fmt --all -- --check
  - cargo check --workspace
  - cargo test --workspace --lib
  - cargo test -p bd_core --test architecture
  - cargo test -p bd_tui --test input_help
  - cargo test -p bd_app --bin bd --test colony_day_cycle
    --test content_loading --test diagnostic --test entity_scope
    --test foundation_actions --test legacy_kernel_regressions
    --test persistence --test progression_factions --test stress
  - cargo test -p bd_app --test foundation_scenario -- --test-threads=1
  - cargo run -p bd_app -- --validate
  - git diff --check
Automated results:
  - all 11 Phase 8 tests passed
  - all workspace library tests and all 5 architecture tests passed
  - all classified application/regression targets passed
  - fixed-dungeon walkability, reachability, placement, player virtue state,
    faction disposition, and content-to-runtime action linkage are validated
  - malformed content and configuration fail with readable errors before TUI startup
  - formatting, workspace check, content validation, and diff checks passed
  - canonical queue remains 13 passing/1 failing
Manual evidence: Startup sequencing is covered at the application boundary;
  complete terminal scenario proof remains Phase 9
GDD sections reviewed: 5, 6, 7, 8
Decision IDs reviewed: D-01, D-03, D-04, D-05, D-10, D-11, D-13, D-14, D-15
Drift found: None; validation is limited to the fixed Foundation slice and
  does not activate procgen, reputation, sanity, raids, events, or final lore
Documentation updated: This evidence record and active-phase header
Known limitations:
  - colony-to-title restart remains the sole canonical acceptance failure and
    is assigned to the Phase 9 session-lifecycle audit
Exit gate result: PASS
```

Foundation records are now rejected before play when their dungeon topology,
placements, virtue references, faction dispositions, player state, or runtime
action links are invalid. Configuration and content errors terminate startup
cleanly without partially initializing the terminal application.

## 19. Phase 9 — Final MVP Audit and Handoff

### Purpose

Prove the Foundation MVP and reconcile all final evidence.

### Dependencies

Phases 0–8 complete.

### Automated validation

Run:

```text
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo run -p bd_app -- --validate
cargo test -p bd_app --test foundation_scenario
cargo test -p bd_app --test content_loading
cargo test -p bd_app --test stress
git diff --check
```

No unexplained warning may remain in the Foundation path.

### Manual extraction scenario

From a clean launch:

```text
title
  → colony
  → inspect three survivors and resources
  → build one station
  → assign one survivor
  → advance one day
  → verify production
  → enter fixed dungeon
  → move and explore
  → defeat the hostile enemy
  → pick up the healing item
  → use or retain it
  → observe skill and two virtue results
  → reach exit
  → explicitly extract
  → verify loot once
  → verify station and assignment survived
  → save
  → restart/load
  → verify state
```

### Manual resume scenario

Save inside the dungeon before combat or extraction, restart, load, and complete the same run. Verify deterministic combat and exact-once extraction.

### Manual defeat scenario

Reach player defeat through normal combat:

- display a distinct defeat result;
- award no loot;
- preserve defeat through save/load;
- return to title/restart through visible controls.

### Final documentation work

- Update this plan with evidence.
- Update `MVP-SCENARIO.md` only if behavior remains within the GDD.
- Record known limitations and Product P2/P3 candidates.
- Update migration/deprecation status.
- Ensure no historical plan claims current authority.

### GDD alignment check

Review all GDD sections and D-01 through D-15. Record each requirement as:

- proven;
- explicitly deferred;
- failed.

No “partially complete” item may pass the MVP gate.

### Exit gate

The Foundation MVP is accepted only when:

- the headless canonical scenario passes;
- the terminal scenarios pass;
- persistence and determinism pass;
- deferred-system isolation passes;
- code, content, UI, tests, and canonical documents agree.

### Completion record — 2026-07-24

```text
Phase: 9 — Final MVP Audit and Handoff
Status: Complete
Scope: Final automated gate, full terminal extraction/resume/defeat audit,
  lifecycle defect repair, GDD reconciliation, and documentation handoff
Defect IDs addressed: final lifecycle audit; no new numbered recovery defect
Baseline commit: 7f7bee8
Phase commit: 1ac674f
Tests added before implementation:
  - colony-to-title transition assertion
  - title load routing and title footer truthfulness
  - restored-mode screen routing
  - Game Over persistence routing and footer truthfulness
  - persistence-after-result schedule ordering
  - exactly-one application player authority
  - restored dungeon continuation to defeat
  - Game Over splash/control consistency
Expected red failures:
  - canonical scenario reported 13/14 because Outpost → Title was rejected
  - title F9 started a new colony instead of loading
  - restored Tactical state retained the title screen
  - Game Over omitted and ignored Save/Load
  - application startup created two Player entities
  - Game Over splash advertised only restart/quit after persistence was enabled
Implementation files:
  - broken-divinity/crates/bd_core/src/session.rs
  - broken-divinity/crates/bd_app/src/main.rs
  - broken-divinity/crates/bd_app/tests/persistence.rs
  - broken-divinity/crates/bd_tui/src/commands.rs
  - broken-divinity/crates/bd_tui/src/lib.rs
  - broken-divinity/crates/bd_tui/src/screens.rs
  - broken-divinity/crates/bd_tui/tests/input_help.rs
Validation commands:
  - cargo fmt --all -- --check
  - cargo check --workspace
  - cargo test --workspace
  - cargo run -p bd_app -- --validate
  - cargo test -p bd_app --test foundation_scenario
  - cargo test -p bd_app --test content_loading
  - cargo test -p bd_app --test stress
  - git diff --check
Automated results:
  - formatting PASS
  - workspace check PASS with no warnings
  - workspace tests PASS; two documented terminal/diagnostic tests remain ignored
  - canonical Foundation scenario PASS 14/14
  - content-loading suite PASS 4/4
  - stress suite PASS 6/6
  - startup content validation PASS
Manual evidence:
  - clean title → shelter path showed three survivors and colony resources
  - built and staffed a Stove, advanced one day, and observed the exact daily
    production/consumption result
  - entered the fixed dungeon, saved before combat, restarted, loaded directly
    into the dungeon, defeated the Rat, picked up the healing item, and observed
    melee/Thumos plus medicine/Temperance feedback
  - repeated the saved branch while retaining the item and explicitly extracted
    exactly one item; the shelter, Stove, day, and extracted result survived save,
    restart, and load
  - reached defeat through normal enemy combat, received zero loot, saved the
    Game Over state, returned to title with the visible restart control, loaded
    from title, and restored the distinct Game Over screen
GDD sections reviewed: 1–10
Decision IDs reviewed: D-01 through D-15
Drift found:
  - duplicate application-level player spawning competed with the kernel owner
  - title and Game Over controls did not match persistence behavior
  - restored saves did not request the screen matching their session phase
Documentation updated:
  - FOUNDATION-RECOVERY-PLAN.md
  - MVP-SCENARIO.md
  - MIGRATION-AND-DEPRECATION.md
  - docs/README.md
  - broken-divinity/README.md
  - broken-divinity/KNOWN_ISSUES.md status
Known limitations:
  - one fixed dungeon, one hostile archetype, one healing item, and one manual
    save slot are intentional Foundation limits
  - only representative melee/medicine and Thumos/Temperance player-visible
    progression is required; full mappings and balance remain deferred
  - procgen, overworld travel, raids, events, sanity, theology mechanics,
    reputation, final factions, and deeper narrative remain isolated
Exit gate result: PASS
```

### Final GDD and decision reconciliation

| Decision | Result | Foundation evidence |
|---|---|---|
| D-01 | Proven | Playable kernel shell, shelter, dungeon loop, and 14/14 canonical scenario |
| D-02 | Proven | Bevy-Ratatui/Ratatui is the launched and manually audited runtime |
| D-03 | Proven | Exactly two data-driven placeholder factions load; a third requires no Rust branch |
| D-04 | Explicitly deferred | Foundation plugin isolation proves sanity behavior is inactive |
| D-05 | Proven | Skills improve through actions; actions express representative virtues |
| D-06 | Explicitly deferred | Theology-driven mechanics remain outside the Foundation plugin and gate |
| D-07 | Proven | Root GDD and canonical root files own design; `docs/` owns general records |
| D-08 | Proven | This plan was the sole recovery authority and now records the completed gate |
| D-09 | Proven | Existing crates and useful procgen/deferred code were preserved and classified |
| D-10 | Proven | Enter, explore, fight, loot, explicitly extract, and return passed headless and terminal audits |
| D-11 | Proven | Foundation entry loads the hand-authored dungeon without invoking procgen |
| D-12 | Proven | Physical shelter, three survivors, five station types, assignment, resources, and production pass; raids/events are deferred |
| D-13 | Proven | Melee, ranged, repair, and medicine are typed and extensible; representative growth is exercised |
| D-14 | Proven | Progression interfaces and representative virtue hooks pass; full mapping is deferred |
| D-15 | Proven | Deterministic fixed content, factions, combat, loot, colony return, exact outcomes, and persistence pass |

No GDD requirement failed the Foundation gate. Deferred items are the
owner-approved exclusions named in the GDD and decision register, not partial
Foundation completions.

### Product candidates after Foundation

- Product P2: richer colony identity, faction response/trust, additional fixed
  dungeon themes, sanity, contradictory testimony, and stronger narrative
  meaning.
- Product P3: deeper colony management, living overworld travel, historical
  generation, procedural dungeon expansion, and broader faction complexity.
- Any next product must begin with a new plan that preserves the Foundation
  action, session, content, persistence, and presentation boundaries proven
  here.

## 20. Validation Matrix

| Requirement | Automated proof | Manual proof | GDD/decision |
|---|---|---|---|
| Ratatui shell | app construction and TUI tests | clean launch | GDD 8, D-02 |
| Physical shelter | colony state assertions | shelter inspection | GDD 6, D-12 |
| Station and assignment | action-driven scenario | build/assign | GDD 6/8, D-12 |
| Production | exact-once day test | one-day cycle | GDD 6/8, D-12 |
| Fixed dungeon | content/provider test | enter/explore | GDD 8, D-10/D-11 |
| Combat | action and defeat tests | fight/defeat | GDD 6/8, D-10 |
| Loot/extraction | exact-once scenario | pickup/extract | GDD 8, D-10 |
| Colony continuity | lifecycle and save tests | return inspection | GDD 3/6/8 |
| Skills | progression test | visible gain | GDD 7/8, D-05/D-13 |
| Virtues | two-hook test | visible hooks | GDD 7/8, D-05/D-14 |
| Factions | typed disposition tests | hostile encounter | GDD 5/8, D-03 |
| Persistence | state matrix | save/restart/load | GDD 8, D-15 |
| Determinism | snapshot continuation | resumed dungeon | D-15 |
| Deferred isolation | plugin/resource tests | no deferred behavior | D-04/D-06/D-15 |

## 21. Phase Evidence Template

Every phase status update must include:

```text
Phase:
Status: Pending | In progress | Blocked | Complete
Scope:
Defect IDs addressed:
Baseline commit:
Tests added before implementation:
Expected red failure:
Implementation files:
Validation commands:
Automated results:
Manual evidence:
GDD sections reviewed:
Decision IDs reviewed:
Drift found:
Documentation updated:
Known limitations:
Exit gate result:
```

At most one phase may be In progress.

## 22. Final Definition of Done

The project is in a proper working Foundation MVP state when:

- one canonical GDD and one active plan exist;
- the complete canonical scenario passes without direct mutation or debug commands;
- the shelter survives dungeon travel and loading;
- stations, survivors, assignments, resources, and production persist;
- dungeon entities are isolated and cleaned correctly;
- extraction and defeat are distinct and exact;
- save/load resumes colony, dungeon, extracted, and defeated state;
- deterministic continuation is proven;
- normal actions improve a skill and express at least two virtues;
- faction data controls hostile behavior;
- input configuration matches actual controls and help;
- invalid actions, saves, content, and configuration fail clearly;
- deferred systems remain inactive;
- all completion claims cite current evidence.

Every item above was recorded as passing on 2026-07-24. The post-acceptance
discovery audit has since reopened the affected gates listed near the beginning
of this record. Product P2 remains unauthorized.

## 23. Plan Validation Record

**Validated:** 2026-07-24

### Sources reviewed

- canonical root `GDD.md`;
- `Kernel.md`;
- `Kernel-direction.md`;
- `docs/DECISIONS-TO-LOCK.md`;
- `docs/MIGRATION-AND-DEPRECATION.md`;
- `docs/MVP-SCENARIO.md`;
- previous `docs/ACTIVE-PLAN.md`;
- Phase 3–11 implementation contracts;
- current workspace, content, tests, and terminal playtest evidence.

### Logic and sequencing review

- Documentation authority and baseline protection occur before implementation.
- A failing production-path acceptance harness precedes fixes.
- Entity scope is repaired before persistence serializes it.
- Persistence is repaired before later systems rely on resume behavior.
- Action convergence occurs before progression and faction semantics are accepted.
- Domain behavior is stable before input and presentation refactoring.
- Content validation is finalized after runtime schemas and action IDs stabilize.
- Manual acceptance occurs only after all automated phase gates.

### Protocol review

- **TDD:** Every implementation phase names failing tests before implementation.
- **DRY:** One action pipeline, one entity scope model, one day boundary, one faction relationship service, and one input descriptor source are required.
- **SRP:** Application, simulation, data, presentation, and test-support ownership are explicit.
- **Open/Closed:** Factions, actions, bindings, and content extend through typed data/registries rather than new mode-specific branches.
- **Data-driven:** Skill/virtue mappings, faction disposition, content identity, and command bindings are validated data.
- **Encapsulation:** TUI emits commands/intents and consumes view models; it does not mutate gameplay state or own persistence.
- **Magic numbers:** New timing, layout, cost, and threshold values require named constants or validated content fields.

### Risk review

Highest-risk work is isolated in:

1. entity scope migration;
2. save/load staging and relationship restoration;
3. deterministic random continuation;
4. action-pipeline convergence;
5. exact-once daily processing.

Each is separated into a phase with dedicated red tests. UX and content expansion cannot hide failures in these areas.

### Ambiguity review

No unresolved product ambiguity remains in this plan.

The following MVP-specific choices are intentionally locked to prevent agent invention:

- one manual save slot;
- one fixed dungeon;
- one hostile and one neutral placeholder faction;
- `ability.quick_attack` as the canonical melee action;
- all six virtue state lanes plus separate Kleos;
- exactly one day-boundary result;
- complete typed replay evidence;
- no compatibility guarantee for unreleased development saves.

Any request to change those choices invokes the stop protocol and requires a decision-register update.

### Validation result

The project owner authorized sequential execution of Phases 1–9 on
2026-07-24 after Phase 0 passed. All phases subsequently passed their internal
validation, GDD drift check, and exit gate. That authorization is exhausted;
Product P2 requires a new owner-approved plan.

## Post-recovery stabilization result — 2026-07-25

The later Foundation Stabilization Plan repaired every gate reopened by the
clean-session audit and passed its expanded final acceptance gate. The
authoritative evidence, terminal results, unexpected-regression fixes, and
GDD/D-01–D-16 reconciliation are recorded in
[FOUNDATION-STABILIZATION-PLAN.md](FOUNDATION-STABILIZATION-PLAN.md#21-phase-8-completion-evidence).

This recovery plan remains the chronological record of the earlier work. Its
2026-07-24 completion claim is not the final acceptance authority. Foundation
is accepted through the 2026-07-25 stabilization result. Product P2 remains
unauthorized.

A subsequent 2026-07-25 discovery run reopened that stabilization acceptance.
The draft response is
[FOUNDATION-MVP-CORRECTION-PLAN.md](FOUNDATION-MVP-CORRECTION-PLAN.md).
Neither this recovery record nor the draft authorizes implementation.
