# Broken Divinity Foundation Test and Colony UX Hardening Plan

**Status:** Owner-approved Foundation hardening plan; Phases 0–1 and the
Section 22 F0–F9 failure-remediation program are complete

**Created:** 2026-07-25

**Plan owner:** Project owner

**Execution target:** A smaller coding model working in short, validated batches

**Product boundary:** Foundation MVP test trust, shelter spatial safety, colony
legibility, and player-facing Ratatui behavior

**Coordination:** This plan owns behavior and implementation order.
[AUTHORITATIVE-TESTING-STANDARD-AND-MIGRATION-PLAN.md](AUTHORITATIVE-TESTING-STANDARD-AND-MIGRATION-PLAN.md)
owns test policy, contract registration, diagnostic quality, metrics, and
suite migration. Both gates apply.

**Supersedes:** No completed plan. This plan reopens only the affected test and
colony UX acceptance gates from the completed
[FOUNDATION-MVP-CORRECTION-PLAN.md](FOUNDATION-MVP-CORRECTION-PLAN.md).

**Does not authorize:** Product P2, procgen in the Foundation path, overworld
expansion, raids, colony events, sanity, theology-driven mechanics, faction
reputation, final factions, new dungeon content, broad balance changes, or a
new runtime technology.

---

## 1. Purpose

The Foundation currently has 462 active automated tests and several completed
acceptance records. A deep real-terminal playtest nevertheless proved critical
player-facing failures:

- legal station placements can permanently trap the player;
- the 40x30 shelter has no camera or viewport, so the player and survivors can
  disappear outside the fixed top-left render window;
- required resource nodes are not visibly discoverable at the supported
  60x20 profile and are barely discoverable at 80x24;
- survivor movement exists but is delayed, subtle, capable of overlapping its
  target, and disconnected from production;
- stations, survivors, and resource nodes use ambiguous raw glyphs and
  overlapping presentation categories;
- compact menus truncate decisive information;
- task/station assignment advances time despite the accepted correction
  contract saying confirmation does not advance time;
- several automated tests claim visibility, discoverability, schedule order,
  or stress coverage without asserting those outcomes.

This plan has two inseparable goals:

1. make the automated suite honest enough to guide future development; and
2. use those tests red-first to repair the confirmed Foundation colony UX
   failures without expanding product scope.

The plan is intentionally explicit. The executing model must not infer missing
requirements, combine phases, or replace a named test with a weaker
approximation.

---

## 2. Authority and Required Reading Order

Before changing any file, the implementation agent must read these documents
in order:

1. [README.md](README.md);
2. root [GDD.md](../GDD.md), especially Sections 3, 6, 8, and 9;
3. [DECISIONS-TO-LOCK.md](DECISIONS-TO-LOCK.md), especially D-02, D-09,
   D-12, D-16, D-17, and D-18;
4. root [Kernel.md](../Kernel.md), especially schedule discipline, semantic
   ASCII, view-model boundaries, data loading, and testing;
5. [MVP-SCENARIO.md](MVP-SCENARIO.md);
6. this plan in full;
7. [FOUNDATION-MVP-CORRECTION-PLAN.md](FOUNDATION-MVP-CORRECTION-PLAN.md) as
   completed evidence, not current instructions;
8. [MIGRATION-AND-DEPRECATION.md](MIGRATION-AND-DEPRECATION.md);
9. current code, tests, and content.

The GDD owns player experience. `Kernel.md` owns architecture. D-18 owns the
behavioral choices needed by this plan. This plan owns behavior task order.
D-19 and the authoritative testing-standard plan own evidence sufficiency,
test metrics, and suite migration.

If any lower-authority file conflicts with those sources, stop and report the
conflict. Do not select whichever behavior is easiest to implement.

---

## 3. Locked Behavior Contracts

These contracts are approved for this Foundation hardening pass. They are not
open implementation choices.

### THC-01 — Management is paused

- Opening, navigating, confirming, or cancelling survivor task/station
  management does not advance `GameTime`.
- Confirmation changes only the selected management relationship/task and its
  player-facing result state.
- Cancellation changes no gameplay state, emits no gameplay action, and
  advances no time.
- `c` opens task management.
- `e` opens station staffing or the station-assignment section directly.
- Both paths identify the named survivor and, where relevant, the named
  station before confirmation.

### THC-02 — Worker movement has explicit meaning

- `SurvivorTask` remains the durable player assignment.
- Player-facing activity is one of:
  - `Idle`;
  - `EnRoute`;
  - `Working`;
  - `Blocked`;
  - `Resting`;
  - `Defending` only when a Foundation effect exists.
- Idle survivors are not required to wander in Foundation.
- A newly assigned survivor does not move during the paused assignment.
- On each later accepted time-advancing Outpost turn, a non-idle survivor gets
  at most one deterministic movement step.
- Rest Until Next Day simulates the same number and order of survivor movement
  steps as the skipped individual Outpost turns.
- Survivors respect walls, stations, the player, resource nodes, and other
  survivors according to the occupancy contract below.

### THC-03 — Work requires a physical work position

- A station is blocking and the worker stops on a walkable cardinally adjacent
  tile.
- A resource node is a target fixture and the gatherer stops on a walkable
  cardinally adjacent tile.
- Survivors do not occupy station or resource-node tiles.
- Survivors do not stack on one another.
- Station production counts an assigned survivor only when that survivor is
  in `Working` range of the assigned station at the daily boundary.
- Gathering counts a survivor only when that survivor is in `Working` range of
  a matching non-depleted node at the daily boundary.
- A survivor with no valid route is `Blocked`, produces nothing, and exposes a
  player-facing reason.
- The fixed shelter and starting state must retain a tested zero-Supplies
  recovery path under these physical rules.

### THC-04 — The shelter is navigable and visible

- Every accepted station placement preserves a walkable path from the player
  to the shelter gate.
- A placement that removes the last path is rejected atomically before
  payment, turn advancement, or entity creation.
- The map panel is a viewport into the 40x30 shelter rather than a crop fixed
  at `(0,0)`.
- The viewport follows the player and clamps at every map edge.
- Terrain, player, survivors, stations, resource nodes, exits, enemies, items,
  and placement previews use one world-to-screen transform.
- The player remains visible at every walkable shelter position.
- An assigned off-screen target is discoverable through a directional edge
  indicator or equivalent compact affordance at both 80x24 and 60x20.

### THC-05 — The ASCII language is semantic and unambiguous

- Core/view-model code does not assign ad hoc presentation glyphs for
  survivors, stations, and resource nodes.
- Distinct simultaneous categories use distinct semantic visual tokens.
- Stations and resource nodes do not share `VisualToken::Item`.
- A glyph/style combination cannot ambiguously represent two categories that
  can be visible together.
- Content validation rejects ambiguous active symbol assignments.
- Help or an always-accessible legend explains every glyph category visible in
  the current mode.
- A staffed station and an unstaffed station are visibly distinguishable.
- A survivor cannot disappear merely because a station or resource layer is
  drawn later.

### THC-06 — Supported compact output is complete

- 80x24 remains the baseline profile.
- 60x20 remains a supported compact profile, not a best-effort profile.
- Required station cost, effect, availability, selection, and controls are
  readable in full through wrapping or a selected-item detail area.
- Required text must not end mid-word.
- Modal body controls and footer controls must agree.
- Long logs may be summarized, but the decisive action, target, result, and
  resource delta remain available.

---

## 4. Non-Negotiable Execution Rules for a Smaller Model

### 4.1 Work one numbered task at a time

The executing model must:

1. read the task;
2. inspect only the listed relevant files plus directly imported dependencies;
3. write the named test first;
4. run the named focused command;
5. record the expected red result;
6. implement only enough production behavior to satisfy that test and existing
   contracts;
7. rerun the focused command;
8. run the phase regression command;
9. perform the phase GDD check;
10. update this plan's execution record only after all gates pass.

Do not start the next task while the current task is red, flaky, or
unexplained.

### 4.2 Preserve the dirty worktree

The repository already contains owner work. Do not:

- reset, checkout, clean, stash, or delete unrelated changes;
- reformat unrelated files;
- replace whole modules to simplify a local edit;
- commit or push unless the owner separately requests it.

Before each phase, run `git status --short` and record only the files intended
for that phase. If a target file has overlapping unexplained changes, stop.

### 4.3 TDD is mandatory

For every behavior change:

- the new or strengthened test must fail for the intended reason before
  production code changes;
- compilation failure is not an acceptable red result unless the task is
  specifically introducing a new public type;
- a panic in test setup is not an acceptable red result;
- after implementation, the test must pass without weakening its assertions;
- no `#[ignore]`, conditional assertion, early return, broad tolerance, or
  fixture mutation may be used to turn red green.

Record the red failure message in the phase execution notes.

### 4.4 Test names must match proof

A test name may use words such as `visible`, `discoverable`, `atomic`,
`deterministic`, `complete`, or `reachable` only if its assertions directly
prove that property.

Forbidden patterns:

```rust
if let Some(stage) = stage {
    assert!(...);
}
```

for a required stage;

```rust
let _result = validate(...);
```

when validity is the claimed outcome;

```rust
assert!(output.contains("Workshop"));
```

as sole proof that the Workshop cost/effect/control presentation is readable.

### 4.5 Acceptance tests use player or production boundaries

- Domain tests may call pure domain functions.
- Schedule tests may inject typed production messages.
- Player-path tests must start from production key events and read production
  state/view models/render output.
- Player-path tests may not call `fixture_*`, directly mutate resources, or
  inject `ActionIntent`.
- Fixture mutation is allowed only for clearly named component/unit
  preconditions and may not manufacture the outcome being claimed.

### 4.6 Stable identity only

Do not use ECS query order, `first_survivor()`, `first_station()`,
`survivors()[1]`, or entity-bit ordering as identity in new acceptance tests.
Select by stable content ID or unique name and assert that uniqueness.

### 4.7 No new duplicate truth

- Economy formulas remain in `bd_core`.
- Station facts remain in the validated station catalog.
- Visual symbols remain in the symbol registry/content after Phase 7.
- The TUI projects state; it does not recalculate production, pathfinding,
  activity, or placement legality.
- Test helpers call production APIs; they do not implement alternate game
  rules.

### 4.8 Stop conditions

Stop and ask the owner if:

- a locked contract cannot be implemented without Product P2 scope;
- physical work breaks the zero-Supplies recovery invariant and no
  Foundation-scope shelter/content correction can restore it;
- a new stable survivor identity requires a product-level naming decision;
- Ratatui or Bevy scheduling behavior differs from the assumptions documented
  here;
- a test intended to reproduce a confirmed defect passes before implementation;
- a pre-existing test must be deleted or materially weakened;
- a public API must be broadened solely for tests;
- the implementation would require real-time animation rather than
  turn-driven presentation;
- canonical documents conflict.

---

## 5. Test Evidence Model

No single test may stand in for every layer. Each player-facing contract must
have the applicable evidence below.

| Layer | Proves | May use |
|---|---|---|
| Domain | Pure rule/invariant | Pure functions and value types |
| Schedule | Actual Bevy ordering and exact-once mutation | Production plugin and typed messages |
| State diff | Authorized and forbidden mutations | Normalized read-only state fingerprint |
| Projection | Semantic view-model and viewport output | Production view-model builders |
| Buffer | Exact supported-size terminal composition | Ratatui `TestBackend` |
| Player path | Controls, modal states, action path, visible result | Production key events only |
| PTY | Real process and terminal behavior | Built `bd` binary in isolated XDG roots |

The plan's final traceability matrix in Section 18 lists the minimum evidence
for every reopened gate.

### 5.1 Visual acceptance requires multiple representations

A text-only terminal snapshot is not sufficient visual evidence. It cannot
prove resolved foreground/background color, modifiers, semantic layer,
priority, or whether two categories remain distinguishable when their glyphs
match. Conversely, a style snapshot alone cannot prove complete wording,
wrapping, borders, or controls.

Every canonical visual scene must produce and test all applicable
representations:

1. **Semantic projection**
   - stable visual category and content ID;
   - world position;
   - resolved screen position or off-screen direction;
   - visual token and style token;
   - layer and priority;
   - stable label and player-facing state.
2. **Plain terminal canvas**
   - exact glyphs, whitespace, borders, wrapping, clipping, and panel
     composition from Ratatui `TestBackend`.
3. **Resolved style cells**
   - for every non-default/significant terminal cell: coordinate, symbol,
     normalized foreground, normalized background, and modifiers;
   - for every projected map cell: its semantic layer and priority before
     terminal-buffer resolution.
4. **Layout geometry**
   - terminal bounds, panel rectangles, viewport rectangle, modal rectangle,
     footer rows, and selected-detail rectangle.
5. **Transition evidence**
   - the before and after observations for state changes, including which
     semantic regions are allowed to change and which must remain stable.

Raster screenshots are optional review artifacts, not the automated source of
truth for this terminal application. Cell and semantic snapshots are more
precise and avoid font, renderer, antialiasing, and host-terminal differences.
Real PTY inspection remains mandatory for integration behavior that
`TestBackend` cannot prove.

### 5.2 Canonical visual fixture rules

Create canonical scenes through one deterministic fixture/gallery builder.
Each fixture must:

- use stable content IDs, survivor names, positions, and resource values;
- set every state explicitly rather than inheriting mutable defaults;
- use production projection and render functions;
- avoid ECS query order and raw entity IDs;
- render identically when constructed twice;
- state the expected player-visible purpose in one sentence;
- remain small enough that one failed snapshot identifies one behavior.

Do not build one giant scene that tries to prove the entire game. Do not create
test-only render paths or duplicate layout/style logic in the fixture builder.

### 5.3 Required visual-state matrix

The matrix below is the minimum visual gate. `S` means semantic projection,
`C` plain canvas, `Y` resolved style cells, `G` geometry, `T` transition
evidence, and `P` real PTY inspection.

| Canonical scene | Profiles | Evidence | Required proof |
|---|---|---|---|
| Title/new game | 80x24, 60x20 | C,Y,G,P | readable title, selectable action, no clipping |
| Clean shelter overview | 80x24, 60x20 | S,C,Y,G,P | player, gate, survivors, nodes, panels, controls |
| Help/context legend | 80x24, 60x20 | S,C,Y,G,P | every currently visible semantic category explained |
| Build selection | 80x24, 60x20 | S,C,Y,G,T,P | selection, complete cost/effect, controls, paused state |
| Valid build placement | 80x24, 60x20 | S,C,Y,G,T,P | visible valid preview and unchanged player visibility |
| Invalid build placement | 80x24, 60x20 | S,C,Y,G,T,P | distinct invalid preview and complete rejection reason |
| Survivor task management | 80x24, 60x20 | S,C,Y,G,T,P | selected survivor/task/detail and paused state |
| Station staffing | 80x24, 60x20 | S,C,Y,G,T,P | station, assignee, staffing state, paused state |
| Worker idle→en-route→working | 80x24, 60x20 | S,C,Y,T,P | visible movement/state progression without occlusion |
| Worker blocked | 80x24, 60x20 | S,C,Y,G,P | visible blocked state and specific reason |
| Off-screen assigned target | 80x24, 60x20 | S,C,Y,G,T,P | direction/target remains discoverable after movement |
| Adverse/zero-Supplies state | 80x24, 60x20 | S,C,Y,G,P | shortage, recovery action, and decisive values readable |
| Day summary | 80x24, 60x20 | S,C,Y,G,P | worker contributions and resource delta remain available |
| Save/load success and failure | 80x24, 60x20 | C,Y,G,T,P | result is explicit and prior screen remains coherent |
| Dungeon exploration/combat/loot | 80x24, 60x20 | S,C,Y,G,T,P | actor/target/loot/exits and current controls are legible |
| Extraction and game over | 80x24, 60x20 | C,Y,G,T,P | outcome and next action are complete and unambiguous |

The automated matrix must use the same supported profile dimensions as the
real application. A scene may share a fixture with another scene, but it must
have its own named snapshot and assertions.

### 5.4 Visual invariants that snapshots do not replace

Add direct assertions for:

- every visible entity has exactly one resolved on-screen cell unless its
  contract explicitly permits multiple cells;
- the player appears exactly once and is never outside the viewport;
- every required off-screen assigned target has one discoverability
  affordance;
- no panel or modal overlaps the footer or another exclusive rectangle;
- no stale non-default cell remains after modal close, viewport pan, or resize;
- required detail text contains complete words and all required control
  tokens;
- categories visible at the same time have distinct glyph/style pairs;
- foreground/background/modifier changes are detected even when glyphs do not
  change;
- player, survivor, station, node, preview, item, and terrain priorities
  resolve deterministically;
- rendering identical state twice yields byte-for-byte equivalent normalized
  observations;
- save/load of the same normalized state yields equivalent semantic, canvas,
  style, and geometry observations;
- viewport panning preserves relative world positions and changes only the
  expected projected region;
- resizing 60x20→80x24→60x20 reflows from current state without stale cells;
- ASCII-only/fallback presentation remains unambiguous without relying on
  color.

### 5.5 Snapshot review policy

- Snapshot files are reviewed changes, not generated truth.
- Never bulk-accept snapshots merely to make a failing run green.
- A changed snapshot must name the contract that intentionally changed.
- Unexpected changes outside the intended panel/region fail the task.
- Normalize unstable IDs and host paths, but never normalize gameplay text,
  glyphs, style, spacing, geometry, clipping, or state.
- Keep plain-canvas and style-cell snapshots separate so diffs remain
  readable.
- Do not update a snapshot until the corresponding semantic and structural
  assertions pass.
- Each phase completion record lists every accepted snapshot and why it
  changed.

---

## 6. Baseline Commands and Starting Evidence

Run from:

```text
/home/eric/projects/BD virtues/broken-divinity
```

### 6.1 Mandatory baseline

```bash
git status --short
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo test --workspace -- --list
cargo clippy --workspace --all-targets -- -D warnings
cargo run -p bd_app -- --validate
git diff --check
```

Expected starting facts:

- 462 active tests are listed;
- two tests are ignored;
- the root `src/` tree is not a workspace package and its tests are not part of
  `cargo test --workspace`;
- active legacy regression tests explicitly do not count as Foundation
  acceptance;
- `insta` is present only through `bd_test_support` and no active code uses
  snapshot macros;
- the existing workspace should pass before Phase 1 begins.

If the baseline differs, record the exact result and stop before changing code.

### 6.2 Baseline UX reproduction

Use isolated XDG roots and real terminal sizes. Record:

1. 80x24 clean shelter;
2. 60x20 clean shelter;
3. player walking beyond the visible map;
4. one gatherer moving beyond the visible map;
5. Stove east and Altar south of the shelter spawn trapping the player;
6. task assignment turn before/after;
7. Build, Help, task management, and station staffing screens.

Do not reuse an existing save.

---

## 7. Phase 0 — Authority, Audit Ledger, and Suite Classification

### Purpose

Make test evidence honest before adding more tests.

### Allowed files

- this plan;
- `docs/README.md`;
- `docs/DOCUMENT-INVENTORY.md`;
- `docs/MVP-SCENARIO.md`;
- `docs/DECISIONS-TO-LOCK.md`;
- a new repository file:
  `broken-divinity/testing/FOUNDATION-TEST-EVIDENCE.md`;
- a new repository file:
  `broken-divinity/testing/VISUAL-ACCEPTANCE-MATRIX.md`.

### Tasks

#### 0.1 Create the test evidence ledger

Create `testing/FOUNDATION-TEST-EVIDENCE.md` with one row per active
integration test target:

- target path;
- test count;
- classification:
  - Acceptance;
  - Contract;
  - Regression;
  - Legacy;
  - Deferred;
  - Diagnostic;
- production boundary used;
- fixture mutation used;
- player-path evidence: yes/no;
- GDD/decision contracts covered;
- known blind spots;
- whether it may be cited for Foundation acceptance.

At minimum classify:

- all `crates/bd_app/tests/*.rs`;
- `crates/bd_core/tests/architecture.rs`;
- `crates/bd_tui/tests/input_help.rs`;
- unit-test groups in `bd_core`, `bd_tui`, `bd_data`, and `bd_test_support`.

Do not count root `src/` tests as active.

#### 0.2 Identify misleading test names

Create a remediation table for at least:

- `zero_supply_colony_has_a_discoverable_recovery_path`;
- `every_buildable_station_has_a_visible_implemented_effect`;
- `system_order_matches_declared_schedule`;
- `trace_records_ordered_flow`;
- `hundred_turn_simulation_does_not_leak_entities`;
- `seed_batch_does_not_panic`;
- `event_queue_does_not_grow_unbounded`;
- current TUI functions named `snapshot`.

For each, choose exactly one:

- strengthen to prove the name;
- rename to its actual proof;
- reclassify as legacy/deferred;
- replace with a production-path test, retaining useful lower-level coverage.

Do not delete coverage during Phase 0.

#### 0.3 Reopen only affected MVP scenario gates

Update `MVP-SCENARIO.md` so that:

- simulation, persistence, dungeon, and economy evidence remain accepted;
- colony spatial safety, viewport visibility, semantic presentation, compact
  completeness, worker activity, and management-time behavior are marked
  reopened;
- this plan is named as the active closure authority.

#### 0.4 Create the executable visual acceptance ledger

Create `testing/VISUAL-ACCEPTANCE-MATRIX.md` by copying the canonical scene
matrix and visual invariants from Sections 5.3 and 5.4. Add columns for:

- fixture ID;
- test target and exact test name;
- semantic snapshot;
- canvas snapshot;
- style snapshot;
- geometry assertion;
- transition assertion;
- 80x24 result;
- 60x20 result;
- PTY result;
- GDD/decision contract;
- status and dated evidence.

All rows start `Not implemented`. A row becomes `Accepted` only when every
required evidence column has a passing result. The ledger may link to
snapshots; it must not embed hand-edited expected screen output.

### Validation

```bash
rg -n "testing-governance|behavior implementation|reopened" \
  docs/README.md docs/DOCUMENT-INVENTORY.md docs/MVP-SCENARIO.md
git diff --check
```

### Completion gate

- Both coordinated plans and their non-overlapping authority are named
  everywhere.
- Completed plans remain completed records.
- No deferred Product P2 feature is activated.
- The ledger distinguishes test count from acceptance evidence.
- Every required visual scene has an explicit unimplemented/accepted status;
  visual completeness is not inferred from general TUI test count.

### GDD check

Review GDD Sections 3, 6, and 8. Confirm that the reopened gates concern
clarity and physical Foundation behavior, not colony depth.

---

## 8. Phase 1 — Repair False-Positive and Non-Assertion Tests

### Purpose

Remove tests that pass without proving their stated requirement.

### Primary files

- `crates/bd_core/tests/architecture.rs`;
- `crates/bd_app/tests/stress.rs`;
- `crates/bd_app/tests/mvp_correction.rs`;
- `crates/bd_tui/src/lib.rs`;
- `crates/bd_tui/tests/input_help.rs`;
- `testing/FOUNDATION-TEST-EVIDENCE.md`.

### Tasks

#### 1.1 Make trace ordering unconditional

Replace the conditional stage-order assertion with required lookups:

```text
Validation exists exactly once
CostResolution exists exactly once
Mutation exists exactly once
Validation index < CostResolution index < Mutation index
```

Add a test-only schedule probe if the current trace cannot distinguish
duplicate/missing stages. Do not infer actual Bevy set order from enum
declaration order.

Required test names:

- `production_schedule_executes_required_sets_in_declared_order`;
- `accepted_move_emits_each_required_trace_stage_exactly_once`;
- `denied_move_never_reaches_cost_or_mutation`.

This is test hardening, not necessarily a behavior correction. The replacement
may initially pass if the production schedule already conforms. Prove that the
assertion itself is sensitive by testing its sequence validator/probe against a
deliberately missing, duplicated, and reordered fixture. Do not modify
production code merely to manufacture a red result.

#### 1.2 Give the event-queue stress test a postcondition

Use a test observer/count resource. Assert:

- exactly 1,000 messages were read;
- no message is processed twice;
- after the retention window, no old messages remain available to a newly
  registered reader according to Bevy's actual message semantics.

If queue length is not a supported public invariant, rename the test to the
observable consumption contract. Do not inspect private Bevy internals.

#### 1.3 Replace synthetic entity-leak coverage

Keep the hand-written ECS spawn/despawn exercise only if renamed as a Bevy
fixture test. Add a production-path test that:

- starts the Foundation app;
- performs repeated colony/dungeon/colony cycles through production actions;
- records normalized entity counts by scope after each completed cycle;
- proves DungeonTransient entities return to baseline;
- proves ColonyPersistent and RunPersistent counts remain stable;
- fails on monotonic growth.

Start with 10 cycles in the PR gate. Reserve a larger count for the slow
profile.

#### 1.4 Stop accepting invalid procgen as stress success

Procgen is deferred. Reclassify its tests as deferred regression. The test may
assert no panic, but it may not be cited as Foundation acceptance. If a test
name claims valid plans, every sampled plan must validate.

Do not modify the procgen algorithm in this plan.

#### 1.5 Correct misleading MVP test names

Until later phases add real UI proof:

- rename the current zero-Supplies test to
  `three_explicit_supplies_assignments_recover_the_action_threshold`;
- rename the current station test to
  `every_buildable_station_catalog_entry_has_an_implemented_effect`.

Later phases add separate `discoverable` and `visible` player-path tests.

#### 1.6 Rename substring checks

Functions that only use `output.contains(...)` must use names containing
`contains_required_tokens`, not `snapshot_is_readable`, `complete`, or
equivalent.

Do not weaken or remove the token checks. Real snapshots arrive in Phase 4.

### Focused validation

```bash
cargo test -p bd_core --test architecture -- --test-threads=1
cargo test -p bd_app --test stress -- --test-threads=1
cargo test -p bd_app --test mvp_correction -- --test-threads=1
cargo test -p bd_tui --lib -- --test-threads=1
cargo test -p bd_tui --test input_help -- --test-threads=1
```

### Completion gate

- No required assertion is conditional.
- No active test contains no meaningful postcondition.
- Deferred/legacy tests are not described as Foundation acceptance.
- Names match direct proof.
- Full workspace remains green.

---

## 9. Phase 2 — Test Harness Integrity and Normalized State Fingerprints

### Purpose

Prevent test helpers from hiding schedule defects, inaccessible actions, or
identity corruption.

### Primary files

- `crates/bd_test_support/src/lib.rs`;
- `crates/bd_app/tests/foundation_scenario.rs`;
- `crates/bd_app/tests/persistence.rs`;
- `crates/bd_app/tests/foundation_actions.rs`;
- new `crates/bd_app/tests/test_harness_contract.rs`.

### Tasks

#### 2.1 Split action submission from settling

Replace the helper behavior where `expect_action` silently performs multiple
updates with explicit operations:

- `submit_action(...)`;
- `advance_one_frame()`;
- `settle_until(predicate, max_frames)`.

Keep a convenience helper only if its name states the settle behavior and
tests assert how many frames were consumed.

Add:

- `action_submission_does_not_hide_next_frame_mutation`;
- `settle_reports_timeout_instead_of_advancing_unbounded`;
- `one_frame_and_settled_observations_are_distinguishable`.

#### 2.2 Add stable selectors

Add read-only lookup helpers:

- `survivor_by_name(&str)`;
- `station_by_content_id(&str)` or by unique station type when only one exists;
- `resource_nodes_by_kind(...)`;
- `entity_by_content_id(...)`.

New acceptance tests may not use first/index selection.

Do not expose mutable `World`.

#### 2.3 Introduce normalized Foundation fingerprint

Create an entity-ID-independent value containing:

- mode and session phase;
- day/turn;
- player stable identity, position, pools, inventory;
- each survivor stable identity/name, position, task target by stable ID,
  activity state, pools;
- each station content ID/type, position, staffing identity, active effect;
- each resource node kind, position, depletion state;
- colony resources and storage;
- active and completed run outcomes;
- dungeon identity and deterministic state;
- progression/virtue values;
- replay origin and RNG state where available;
- entity counts by scope.

Sort all collections by stable ID/name and position. Do not include raw entity
bits, filesystem paths, or transient log sequence numbers.

Add:

- `fingerprint_is_independent_of_query_iteration_order`;
- `fingerprint_changes_when_survivor_position_changes`;
- `fingerprint_changes_when_station_relationship_changes`;
- `fingerprint_ignores_entity_bit_reallocation_after_restore`.

#### 2.4 Strengthen persistence comparisons

Replace count-only save/load assertions with fingerprint or explicit detailed
comparisons. Preserve tests for atomic failed load and RNG continuation.

### Validation

```bash
cargo test -p bd_test_support -- --test-threads=1
cargo test -p bd_app --test test_harness_contract -- --test-threads=1
cargo test -p bd_app --test foundation_scenario -- --test-threads=1
cargo test -p bd_app --test persistence -- --test-threads=1
cargo test -p bd_app --test foundation_actions -- --test-threads=1
```

### Completion gate

- Test helpers do not silently settle frames.
- Acceptance identity is stable.
- Save/load can no longer pass on equal counts with corrupted relationships or
  positions.
- No production API is widened only for tests.

---

## 10. Phase 3 — Paused Management and Deterministic Schedule Order

### Purpose

Close the accepted-contract violation where management confirmation advances
time and worker movement sees stale task state.

### Primary files

- `crates/bd_app/tests/phase6_input.rs`;
- new `crates/bd_app/tests/colony_management_contract.rs`;
- `crates/bd_core/src/actions.rs`;
- `crates/bd_core/src/colony/survivors.rs`;
- `crates/bd_core/src/lib.rs`;
- `crates/bd_tui/src/lib.rs`;
- `crates/bd_tui/src/view_models.rs`.

### Task 3.1 — Red tests for paused management

Add player-path tests using key events only:

- `task_assignment_confirmation_changes_task_without_advancing_time`;
- `station_assignment_confirmation_changes_relationship_without_advancing_time`;
- `task_management_cancel_changes_nothing`;
- `station_management_cancel_changes_nothing`;
- `management_navigation_emits_no_action_replay_record`.

Each test captures a fingerprint before opening the modal and asserts an exact
allowed state diff after confirmation/cancel.

Expected red: assignment actions are currently classified as turn actions.

### Task 3.2 — Remove management from turn actions

Update action semantics so task and station assignment:

- still use validated typed actions;
- still produce one typed result;
- do not set `ShouldAdvanceTime`;
- do not spend AP;
- do not trigger enemy or survivor movement;
- remain replayable as non-time-advancing management records.

Do not bypass `ActionIntent`.

### Task 3.3 — Explicitly order later worker movement

Add a schedule contract test proving:

1. assignment mutation is committed;
2. no movement occurs in the paused assignment;
3. the next accepted Outpost turn reads the new task;
4. movement occurs once;
5. time advances once.

Use named sets or explicit `.after(...)` ordering plus deferred-command
application where required. Do not rely on two systems sharing `BdSet::Mutation`.

### Validation

```bash
cargo test -p bd_app --test colony_management_contract -- --test-threads=1
cargo test -p bd_app --test phase6_input -- --test-threads=1
cargo test -p bd_app --test foundation_actions -- --test-threads=1
cargo test -p bd_core colony::survivors -- --test-threads=1
```

### Manual gate

At 80x24 and 60x20:

1. note day/turn;
2. open `c`, assign Gather Supplies, confirm;
3. verify day/turn unchanged;
4. press Wait once;
5. verify one visible worker step and one turn;
6. repeat through `e` for a station assignment.

### GDD check

GDD Sections 3 and 6. Confirm this makes management deliberate and legible
without adding deeper survivor simulation.

---

## 11. Phase 4 — Semantic Presentation Test Infrastructure

### Purpose

Test what the player can perceive without making every assertion a brittle
full-screen string comparison.

### Primary files

- `crates/bd_tui/src/view_models.rs`;
- `crates/bd_tui/src/render_grid.rs`;
- `crates/bd_tui/src/screens.rs`;
- `crates/bd_tui/src/lib.rs`;
- new `crates/bd_tui/src/visual_contract_tests.rs` included with
  `#[cfg(test)]`;
- `crates/bd_test_support/src/lib.rs`;
- `crates/bd_test_support/Cargo.toml`;
- `crates/bd_tui/Cargo.toml`;
- new `crates/bd_tui/tests/presentation_contract.rs`;
- new `crates/bd_tui/tests/visual_scene_contract.rs`;
- new `crates/bd_tui/tests/visual_buffer_contract.rs`;
- new `crates/bd_tui/tests/visual_transition_contract.rs`;
- new snapshot files under `crates/bd_tui/tests/snapshots/`.

### Task 4.1 — Red tests for visual observation sensitivity

Before implementing new observation helpers, add tests that prove the current
text-only evidence misses each of these deliberate fixture differences:

- same glyph and layout, different foreground;
- same glyph and layout, different modifier;
- same resolved glyph/style, different semantic layer or priority;
- same required words, different panel overlap;
- same final words, one stale cell left by a closed modal.

The red condition is that the existing observation treats each pair as equal
or cannot express the distinction. The final tests must prove the replacement
observation distinguishes every pair.

Required test names:

- `style_observation_detects_foreground_only_change`;
- `style_observation_detects_modifier_only_change`;
- `semantic_observation_detects_layer_or_priority_change`;
- `geometry_observation_detects_panel_overlap`;
- `transition_observation_detects_stale_cell_after_modal_close`.

### Task 4.2 — Define semantic, cell, and geometry observations

Refactor only as far as needed so production rendering constructs one
crate-private immutable render plan before writing to the Ratatui buffer. The
production writer and crate-internal semantic tests inspect that same plan;
there must not be a second test-only projection implementation.

The crate-private render plan/read model contains:

- panel rectangles;
- viewport world bounds;
- each projected visual:
  - stable category;
  - world position;
  - screen position or off-screen direction;
  - visual token;
  - glyph after registry resolution;
  - style token;
  - layer/priority;
- modal title, selected item, complete detail text, and control tokens.

Add a normalized resolved-cell record containing:

- `x`, `y`;
- complete symbol/string, not only the first `char`;
- normalized foreground/background color names;
- normalized modifier names in a stable order.

Add a normalized visual observation containing:

- terminal dimensions;
- semantic projection records sorted by stable category/content ID and
  position;
- resolved significant cells sorted by `y`, then `x`;
- layout rectangles sorted by stable panel name;
- plain canvas rows;
- modal/footer state.

The semantic observation must be captured from that production render plan
before layer resolution so
layer/priority data is not lost. The cell observation must be captured from
the final Ratatui buffer so final style/composition is tested.

Keep semantic-plan tests in `visual_contract_tests.rs`, where crate-private
access is available. External integration targets test the public render/player
boundary and final `TestBackend` buffer. Do not make render internals public
solely for integration tests, do not expose mutable render state, and do not
use `#[cfg(test)]` to select a different production projection path.

### Task 4.3 — Build the deterministic scene gallery

Implement one deterministic world/state fixture gallery in `bd_test_support`
and thin render-profile selection in the TUI tests, following Section 5.2.
The support crate creates state only; it must not calculate layout, projection,
glyphs, styles, layering, or expected output. Add explicit fixture IDs for
every row in Section 5.3. In Phase 4, only the states already implemented must
receive passing snapshots; later-feature rows remain `Not implemented` in the
ledger and are closed by their owning phase.

Add:

- `same_fixture_renders_identically_when_built_twice`;
- `fixture_identity_does_not_depend_on_ecs_query_order`;
- `supported_profile_dimensions_match_runtime_contract`.

### Task 4.4 — Adopt real multi-representation golden snapshots

Use `insta` for a small approved matrix:

- clean Outpost 80x24;
- clean Outpost 60x20;
- build selection 80x24 and 60x20;
- build placement 80x24 and 60x20;
- task management 80x24 and 60x20;
- station staffing 80x24 and 60x20;
- Help/legend 80x24 and 60x20;
- one adverse economy state;
- one active worker state.

For each scene/profile pair, create separate snapshots named:

```text
<fixture_id>__<width>x<height>__canvas
<fixture_id>__<width>x<height>__styles
<fixture_id>__<width>x<height>__semantic
<fixture_id>__<width>x<height>__geometry
```

Add `insta.workspace = true` under `bd_tui` test/dev dependencies. Do not rely
on `bd_test_support`'s dev dependency being transitively available.

Snapshot normalization:

- use the Ratatui buffer rather than ANSI output for automated snapshots;
- replace process-specific paths;
- never include raw entity IDs;
- preserve whitespace, borders, clipping, and complete words;
- preserve glyphs, resolved color/style/modifiers, layer/priority, and panel
  geometry.

Do not snapshot every combinatorial state.

### Task 4.5 — Add structural assertions beside snapshots

Every snapshot test also asserts:

- no panel overlaps another;
- no content writes into footer rows;
- selected detail text is complete;
- required controls are semantically available;
- every projected on-screen entity falls inside the viewport rectangle.

Also add:

- `canonical_outpost_scene_matches_all_visual_representations`;
- `compact_outpost_scene_matches_all_visual_representations`;
- `modal_rectangle_never_overlaps_footer`;
- `player_cell_is_present_exactly_once`;
- `rendering_same_state_twice_produces_identical_visual_observation`;
- `glyph_unchanged_style_regression_fails_visual_contract`;
- `required_detail_text_never_ends_mid_word`.

### Task 4.6 — Add transition and stale-cell tests

Render before/after pairs through the same persistent `TestBackend` for:

- opening and closing Help;
- opening, navigating, confirming, and cancelling Build;
- opening, navigating, confirming, and cancelling survivor management;
- opening, navigating, confirming, and cancelling staffing;
- resizing 60x20→80x24→60x20.

Assert that:

- the modal region is cleared/recomposed after close;
- footer controls match the active mode on every frame;
- confirmation changes only authorized gameplay/UI state;
- cancellation restores the underlying scene without residue;
- no old border, glyph, style, or detail text remains;
- repeated rendering without state change produces no visual diff.

Required test names:

- `closing_modal_clears_every_modal_cell`;
- `cancel_restores_underlying_scene_without_visual_residue`;
- `confirmation_changes_only_authorized_visual_regions`;
- `resize_round_trip_leaves_no_stale_cells`;
- `unchanged_state_has_empty_normalized_visual_diff`.

### Validation

```bash
cargo test -p bd_tui --test presentation_contract -- --test-threads=1
cargo test -p bd_tui --test visual_scene_contract -- --test-threads=1
cargo test -p bd_tui --test visual_buffer_contract -- --test-threads=1
cargo test -p bd_tui --test visual_transition_contract -- --test-threads=1
cargo test -p bd_tui --lib -- --test-threads=1
cargo insta test -p bd_tui
cargo insta pending-snapshots
```

If `cargo insta` is unavailable, normal `cargo test` remains mandatory and
snapshot review must use the library macros/output. Do not install global tools
without owner approval.

### Completion gate

- Snapshot names correspond to actual supported states.
- Snapshot review has no unaccepted pending files.
- Semantic assertions catch overlap and clipping independently of snapshots.
- Canvas snapshots catch text/glyph/spacing regressions.
- Style snapshots catch color/background/modifier regressions with unchanged
  glyphs.
- Transition tests catch stale cells and unintended visual changes.
- `testing/VISUAL-ACCEPTANCE-MATRIX.md` records the result for each Phase 4
  scene and leaves later scenes explicitly open.

---

## 12. Phase 5 — Viewport and Resource Discoverability

### Purpose

Replace the fixed top-left crop with one authoritative player-following
viewport and prove required targets remain discoverable.

### Primary files

- new or existing viewport module under `crates/bd_tui/src/`;
- `crates/bd_tui/src/view_models.rs`;
- `crates/bd_tui/src/screens.rs`;
- `crates/bd_tui/tests/presentation_contract.rs`;
- new `crates/bd_tui/tests/viewport_contract.rs`;
- `crates/bd_app/tests/mvp_correction.rs`;
- `crates/bd_test_support/src/lib.rs`.

### Task 5.1 — Pure viewport red tests

Add table-driven tests:

- `viewport_centers_on_player_when_unclamped`;
- `viewport_clamps_at_left_top_edge`;
- `viewport_clamps_at_right_bottom_edge`;
- `viewport_handles_map_smaller_than_panel`;
- `world_to_screen_and_screen_to_world_round_trip_inside_viewport`;
- `world_to_screen_rejects_outside_positions`;
- `zero_sized_inner_area_is_safe`.

Test 80x24-derived and 60x20-derived map panel sizes.

### Task 5.2 — One transform for every layer

Add projection tests containing terrain, player, survivor, station, resource
node, exit, item, enemy, and build preview at known world coordinates. Assert
all use the same origin and produce expected screen positions.

Expected red: current rendering uses raw world coordinates and clips against
panel size.

### Task 5.3 — Implement viewport ownership

Compute viewport bounds in one place from:

- map dimensions;
- inner map-panel dimensions;
- player world position.

The renderer consumes projected coordinates or the shared transform. Do not
duplicate offsets in each entity loop.

### Task 5.4 — Off-screen target affordance

For an assigned worker target outside the viewport:

- project a directional edge indicator that does not overwrite the player,
  border, or another higher-priority indicator;
- expose target category/name and distance in semantic presentation/help;
- support multiple targets deterministically by priority and stable identity.

Foundation minimum may aggregate identical directions, for example
`> Water Source x2`, but must remain understandable at 60x20.

### Task 5.5 — Replace fake discoverability proof

Add player-path/projection tests:

- `every_required_resource_type_is_visible_or_has_an_offscreen_indicator`;
- `assigned_water_source_is_discoverable_at_80x24`;
- `assigned_water_source_is_discoverable_at_60x20`;
- `player_remains_visible_after_crossing_each_viewport_boundary`.

### Task 5.6 — Close viewport visual scenes

Update semantic, canvas, style, geometry, and transition snapshots at both
profiles for:

- clean shelter overview at center and every clamped edge;
- off-screen assigned target in each cardinal direction;
- viewport crossing;
- 60x20→80x24→60x20 resize.

Add:

- `viewport_pan_preserves_world_entity_relative_positions`;
- `viewport_pan_changes_only_expected_projected_cells`;
- `resize_reprojects_without_stale_cells`;
- `all_visible_entities_have_one_cell_or_one_offscreen_indicator`.

Mark only the corresponding rows in
`testing/VISUAL-ACCEPTANCE-MATRIX.md` accepted.

### Validation

```bash
cargo test -p bd_tui --test viewport_contract -- --test-threads=1
cargo test -p bd_tui --test presentation_contract -- --test-threads=1
cargo test -p bd_tui --test visual_buffer_contract -- --test-threads=1
cargo test -p bd_tui --test visual_transition_contract -- --test-threads=1
cargo test -p bd_app --test mvp_correction -- --test-threads=1
```

### Manual gate

At both profiles, walk to all four shelter extremes. The player remains
visible, viewport edges clamp cleanly, and assigned off-screen resource targets
remain discoverable.

---

## 13. Phase 6 — Construction Egress Safety

### Purpose

Make it impossible for accepted construction to trap the player.

### Primary files

- new `crates/bd_app/tests/colony_spatial_contract.rs`;
- `crates/bd_core/src/actions.rs`;
- `crates/bd_core/src/colony/stations.rs`;
- `crates/bd_core/src/pathfinding.rs`;
- `crates/bd_tui/src/view_models.rs`;
- `crates/bd_tui/src/screens.rs`.

### Task 6.1 — Red reproduction

Using production actions:

1. start at the named shelter spawn;
2. build a Stove east;
3. attempt an Altar south;
4. assert the second build is rejected because it removes the last gate path;
5. assert no second station, payment, time, replayed success, or partial state
   mutation.

Test name:

- `second_corner_station_is_rejected_before_it_traps_the_player`.

### Task 6.2 — Exhaustive accepted-placement invariant

For every walkable player position and cardinal candidate in the fixed shelter:

- clone or model the blocker set with the candidate station;
- if placement is accepted, prove a path remains from the player's resulting
  accessible region to the gate;
- if no path remains, prove the typed rejection reason is
  `WouldBlockShelterEgress` or equivalent typed reason.

Tests:

- `every_accepted_station_placement_preserves_gate_reachability`;
- `egress_rejection_is_atomic`;
- `egress_validation_counts_existing_stations_as_blockers`;
- `egress_validation_does_not_treat_survivor_motion_as_a_permanent_wall`.

### Task 6.3 — Implement one domain validator

Create one core placement validation function receiving:

- map;
- player position;
- gate position;
- current permanent blockers;
- candidate footprint.

Return a typed result. Use existing pathfinding infrastructure. Do not put
flood-fill logic in the TUI or test helper.

### Task 6.4 — Visible invalid preview

Projection tests assert:

- invalid preview uses semantic invalid-selection style;
- selected station name, cost, and rejection reason are visible;
- Enter on invalid preview cannot emit a successful build action.

### Task 6.5 — Close construction visual scenes

At both profiles, capture valid and invalid placement observations. Assert:

- valid and invalid previews differ in semantic token and resolved glyph/style;
- the invalid state remains distinguishable in ASCII-only fallback;
- player and gate remain visible;
- station name, complete cost, effect, and rejection reason remain readable;
- rejecting placement changes no underlying map cell or resource display;
- closing/cancelling placement leaves no preview cell or style residue.

Mark the valid and invalid placement rows in
`testing/VISUAL-ACCEPTANCE-MATRIX.md` accepted only after their transition and
PTY evidence passes.

### Validation

```bash
cargo test -p bd_app --test colony_spatial_contract -- --test-threads=1
cargo test -p bd_app --test foundation_stabilization -- --test-threads=1
cargo test -p bd_core pathfinding -- --test-threads=1
cargo test -p bd_tui --test presentation_contract -- --test-threads=1
cargo test -p bd_tui --test visual_scene_contract -- --test-threads=1
cargo test -p bd_tui --test visual_transition_contract -- --test-threads=1
```

### Manual gate

Repeat the exact Stove-east/Altar-south sequence. The second preview is visibly
invalid, confirmation explains the egress rule, Supplies remain unchanged, and
the player can still reach the gate.

---

## 14. Phase 7 — Survivor Pathing, Activity, and Physical Production

### Purpose

Make survivor motion deterministic, visible, collision-safe, and semantically
connected to work.

### Primary files

- new `crates/bd_app/tests/survivor_work_contract.rs`;
- `crates/bd_core/src/colony/survivors.rs`;
- `crates/bd_core/src/colony/resources.rs`;
- `crates/bd_core/src/colony/production.rs`;
- `crates/bd_core/src/pathfinding.rs`;
- `crates/bd_core/src/lib.rs`;
- `crates/bd_core/src/save.rs`;
- `crates/bd_tui/src/view_models.rs`;
- `crates/bd_test_support/src/lib.rs`.

### Task 7.1 — Define typed activity projection

Prefer a derived read model unless persistence is required:

```text
Idle
EnRoute { target stable ID/name, distance }
Working { target stable ID/name }
Blocked { target stable ID/name, reason }
Resting
Defending
```

Do not store duplicate assignment truth in the activity projection.

### Task 7.2 — Red movement tests

Add:

- `new_assignment_does_not_move_during_paused_confirmation`;
- `next_outpost_turn_moves_worker_exactly_one_cardinal_step`;
- `worker_uses_pathfinding_around_station_blockers`;
- `worker_never_enters_station_tile`;
- `gatherer_never_enters_resource_node_tile`;
- `survivors_do_not_stack`;
- `worker_stops_adjacent_and_becomes_working`;
- `unreachable_target_becomes_blocked`;
- `tactical_turns_do_not_move_colony_survivors`;
- `idle_render_frames_do_not_move_survivors`.

Use exact positions and fingerprints, not glyph observations.

### Task 7.3 — Deterministic occupancy

The movement resolver must:

- build an occupancy map once per movement step;
- process survivors in stable identity order;
- reserve each accepted destination before the next survivor;
- use existing pathfinding with permanent and reserved blockers;
- take at most one step per survivor per Outpost turn;
- emit a typed movement/activity result for presentation.

If stable survivor identity is missing, add the smallest data-driven stable ID
compatible with save/load. Stop if that requires naming/canon decisions.

### Task 7.4 — Rest equivalence

Add:

- `rest_and_individual_waits_produce_the_same_worker_positions`;
- `rest_and_individual_waits_produce_the_same_activity_states`;
- `rest_and_individual_waits_produce_the_same_daily_resources`;
- `rest_does_not_move_workers_after_the_daily_boundary_twice`.

Implement Rest movement as repeated logical Outpost turn steps, not one visual
step followed by a 24-turn time jump.

### Task 7.5 — Physical production

Red tests:

- `assigned_but_enroute_station_worker_produces_nothing`;
- `adjacent_station_worker_produces_once`;
- `assigned_but_enroute_gatherer_produces_nothing`;
- `adjacent_matching_gatherer_produces_once`;
- `gatherer_at_wrong_node_type_produces_nothing`;
- `blocked_worker_produces_nothing_and_reports_reason`;
- `zero_supply_recovery_remains_reachable_under_physical_work_rules`.

Production consumes the authoritative activity/range rule from core. It must
not recalculate a different meaning in forecast and execution.

### Task 7.6 — Forecast truth

Forecast must distinguish:

- currently working output;
- assigned but not yet working;
- blocked output;
- expected food consumption.

Do not promise future arrival unless a deterministic arrival projection is
implemented and tested.

### Task 7.7 — Persistence

Save/load tests must preserve:

- survivor positions;
- durable assignments by stable identity;
- enough state to derive the same activity after load;
- deterministic next movement step.

Loading must not immediately grant a movement or production step.

### Task 7.8 — Close worker-state visual scenes

At both profiles, capture the same named worker as:

- Idle;
- EnRoute at two distinct positions;
- Working adjacent to the target;
- Blocked with a specific reason;
- restored after save/load.

For each transition, assert:

- the survivor remains visible or has one explicit off-screen indicator;
- glyph/style and status text agree with the semantic activity;
- the target relation and distance/status update together;
- unrelated panels and entities remain visually unchanged;
- no survivor is hidden by a station, resource node, or another survivor;
- save/load reproduces the same semantic and resolved visual observation.

Add:

- `worker_state_transition_changes_expected_cell_and_status_only`;
- `worker_visual_state_matches_authoritative_activity`;
- `save_load_same_fingerprint_produces_same_visual_scene`;
- `survivor_collision_never_removes_a_visible_worker`.

Mark the worker idle→en-route→working and worker blocked rows in
`testing/VISUAL-ACCEPTANCE-MATRIX.md` accepted only after PTY evidence at both
profiles.

### Validation

```bash
cargo test -p bd_app --test survivor_work_contract -- --test-threads=1
cargo test -p bd_app --test colony_day_cycle -- --test-threads=1
cargo test -p bd_app --test mvp_correction -- --test-threads=1
cargo test -p bd_app --test persistence -- --test-threads=1
cargo test -p bd_core colony::survivors -- --test-threads=1
cargo test -p bd_tui --test visual_scene_contract -- --test-threads=1
cargo test -p bd_tui --test visual_transition_contract -- --test-threads=1
```

### Manual gate

At both profiles:

1. assign each survivor a different gathering target;
2. press Wait repeatedly;
3. observe distinct deterministic movement;
4. see `En route`, then `Working`, or a specific `Blocked` reason;
5. Rest and verify the day summary counts only workers who reached work range;
6. save/load and verify movement resumes consistently.

---

## 15. Phase 8 — Semantic ASCII, Layering, and Legend

### Purpose

Eliminate ambiguous glyphs and raw category logic.

### Primary files

- `crates/bd_tui/src/visual.rs`;
- `crates/bd_tui/src/view_models.rs`;
- `crates/bd_tui/src/screens.rs`;
- `crates/bd_tui/src/render_grid.rs`;
- `content/symbols/`;
- `crates/bd_data/src/loader.rs`;
- `crates/bd_data/src/validation.rs`;
- `crates/bd_tui/tests/presentation_contract.rs`;
- new `crates/bd_tui/tests/visual_language_contract.rs`.

### Task 8.1 — Red collision inventory

Construct one shelter projection containing:

- player;
- idle survivor;
- en-route gatherer;
- working station survivor;
- Stove;
- Altar;
- Workshop;
- Bed;
- Trees;
- Water Source;
- Wild Plants;
- gate;
- build preview.

Assert every simultaneously visible semantic category resolves to an
unambiguous glyph/style pair.

Expected current collisions:

- survivor `A` and Altar `A`;
- Workshop `W` and Water Source `W`;
- gatherer `G` and deferred Gabriel `G` if both active later;
- stations and nodes both using `VisualToken::Item`.

### Task 8.2 — Extend semantic tokens

Define tokens by gameplay category/state rather than specific content where
possible:

- survivor idle/en-route/working/blocked;
- station unstaffed/staffed;
- resource tree/water/plants or one resource token plus data subtype;
- off-screen target indicator;
- valid/invalid placement.

The symbol registry/content owns glyph and style resolution. View models emit
semantic tokens and stable labels, not raw glyphs.

### Task 8.3 — Content validation

Add:

- `active_simultaneous_symbols_reject_ambiguous_glyph_style_pairs`;
- `station_and_resource_categories_must_use_distinct_tokens`;
- `every_foundation_visual_token_has_a_symbol_definition`;
- `every_visible_symbol_has_a_help_legend_entry`;
- `fallback_symbols_are_unambiguous_in_ascii_only_mode`.

Validation errors identify both conflicting content IDs/tokens and the source
file.

### Task 8.4 — Layer safety

Add:

- `survivor_is_not_hidden_by_station_layer`;
- `survivor_is_not_hidden_by_resource_layer`;
- `player_remains_highest_gameplay_priority`;
- `invalid_build_preview_remains_visible_without_hiding_player`.

The preferred fix is preventing illegal overlap in core plus deterministic
presentation for legitimate co-location, not merely changing draw order.

### Task 8.5 — Style and fallback sensitivity

Add:

- `every_semantic_visual_has_expected_resolved_style`;
- `style_snapshot_detects_color_or_modifier_regression`;
- `glyph_style_pairs_are_unique_for_simultaneously_visible_categories`;
- `monochrome_fallback_remains_unambiguous`;
- `semantic_layer_snapshot_matches_resolved_top_cell`.

Tests must prove category distinction through glyph plus style and must also
prove the ASCII-only fallback does not depend on color alone.

### Task 8.6 — Contextual legend

Help/legend at both profiles must include only categories relevant to the
active mode and current Foundation content. It must explain worker states and
off-screen indicators without requiring source knowledge.

### Task 8.7 — Close semantic-language visual scenes

Regenerate only snapshots intentionally affected by semantic-token and style
changes. Review canvas and style diffs separately. At both profiles, assert
that the clean shelter, Help/legend, placement, staffing, task, worker, and
off-screen-target scenes use the same registry definitions.

Update affected ledger rows with the exact approved snapshot names and
style/glyph rationale. A color-only distinction is not sufficient.

### Validation

```bash
cargo test -p bd_tui --test visual_language_contract -- --test-threads=1
cargo test -p bd_tui --test presentation_contract -- --test-threads=1
cargo test -p bd_tui --test visual_scene_contract -- --test-threads=1
cargo test -p bd_tui --test visual_buffer_contract -- --test-threads=1
cargo test -p bd_data -- --test-threads=1
cargo run -p bd_app -- --validate
```

---

## 16. Phase 9 — Compact Management and Build UX

### Purpose

Make 60x20 genuinely supported and make management modes truthful.

### Primary files

- `crates/bd_tui/src/lib.rs`;
- `crates/bd_tui/src/screens.rs`;
- `crates/bd_tui/src/view_models.rs`;
- `crates/bd_tui/src/commands.rs`;
- `crates/bd_tui/tests/input_help.rs`;
- `crates/bd_tui/tests/presentation_contract.rs`;
- `crates/bd_app/tests/phase6_input.rs`;
- `crates/bd_app/tests/colony_management_contract.rs`.

### Task 9.1 — Distinct management modes

Red player-path tests:

- `c_opens_task_management_with_task_title`;
- `e_opens_station_staffing_with_station_title`;
- `station_staffing_initial_focus_is_a_station_assignment`;
- `task_and_staffing_cancel_controls_match_footer`;
- `each_mode_returns_to_normal_without_leaking_input`.

### Task 9.2 — Compact selected-detail layout

At 60x20, render:

- a short selectable station list;
- a wrapped detail region for the selected station containing:
  - full name;
  - complete cost;
  - complete effect;
  - staffing requirement/current worker;
  - complete unavailable reason;
  - controls.

Tests:

- `compact_build_detail_contains_no_mid_word_truncation`;
- `compact_build_detail_exposes_complete_selected_effect`;
- `compact_storage_reason_is_complete`;
- `compact_management_identifies_selected_survivor_and_target`;
- `modal_and_footer_controls_agree`.

Do not solve by shrinking meaningful text into unexplained abbreviations.

### Task 9.3 — Decisive feedback

After confirmation, expose one concise result containing:

- named survivor;
- task or station;
- resulting activity;
- no duplicate generic message.

At day boundary expose actual deltas and blocked/en-route workers that produced
nothing.

### Task 9.4 — Close compact visual scenes

Run the complete canonical scene matrix at 60x20, not only the management
screens changed in this phase. Add explicit visual diffs for:

- selection movement inside Build, task management, and staffing;
- opening and closing Help;
- confirmation and cancellation;
- day-summary appearance and dismissal;
- save/load success and failure feedback.

Assert no required text ends mid-word, no decisive feedback is hidden behind a
modal/footer, selected detail remains visible, and modal/footer controls agree.
Update every applicable 60x20 ledger result. Do not mark the compact profile
accepted while any applicable row remains open.

### Validation

```bash
cargo test -p bd_tui --test input_help -- --test-threads=1
cargo test -p bd_tui --test presentation_contract -- --test-threads=1
cargo test -p bd_tui --test visual_scene_contract -- --test-threads=1
cargo test -p bd_tui --test visual_buffer_contract -- --test-threads=1
cargo test -p bd_tui --test visual_transition_contract -- --test-threads=1
cargo test -p bd_app --test phase6_input -- --test-threads=1
cargo test -p bd_app --test colony_management_contract -- --test-threads=1
```

### Manual gate

Complete Build, task assignment, station staffing, Help, Wait, Rest, Save, and
Load at 60x20 without clipped decisive text or control contradictions.

---

## 17. Phase 10 — Property, Stress, Mutation, and Final Player Paths

### Purpose

Make future regressions harder to hide without making every PR slow or flaky.

### Task 10.1 — Deterministic property matrices

Add bounded deterministic matrices for:

- every shelter player position and cardinal station placement;
- every viewport edge and supported panel size;
- every Foundation visual category pair;
- every canonical visual fixture at 80x24 and 60x20;
- every survivor assignment/target availability state;
- resource values at min, zero, action threshold, near max, and max;
- save/load before and after assignment, movement, work arrival, and day
  boundary.

Use fixed seeds and print the failing case.

### Task 10.2 — Randomized invariant profile

If adding `proptest` or equivalent, first document and obtain dependency
approval. Otherwise use deterministic seeded sequences.

Generate valid and invalid action sequences and assert after every step:

- one player authority;
- pools remain bounded;
- relationships resolve;
- no two blockers occupy one tile;
- player retains gate reachability in Outpost;
- accepted action emits one result;
- rejected action is atomic;
- entity counts by scope remain bounded;
- save/load fingerprint is stable at checkpoints.

Keep a short PR profile and a longer ignored-by-default external command only
if the long profile has a documented CI/nightly owner. Do not add an ignored
test with no execution path.

### Task 10.3 — Mutation testing proposal

Do not add/install tooling silently. Prepare a documented command profile for
owner approval targeting:

- `bd_core::actions`;
- colony production/resources/survivors;
- save/restore;
- viewport calculations;
- placement validation.

The first mutation goal is not a global score. It is to prove that removing
critical conditions causes at least one named contract test to fail.

### Task 10.4 — Coverage reporting proposal

Coverage is diagnostic, not acceptance. If `cargo llvm-cov` is approved:

- report branch/line coverage by critical module;
- do not use one global percentage to hide untested critical branches;
- set initial thresholds only after capturing an honest baseline;
- never add meaningless assertions solely to increase coverage.

### Task 10.5 — Full player-path battery

Through production key events and rendered output:

1. clean launch;
2. inspect Help/legend;
3. traverse viewport boundaries;
4. attempt trapping construction and observe rejection;
5. build one valid productive station;
6. assign a named survivor;
7. observe EnRoute and Working;
8. verify physical production and forecast;
9. Rest;
10. save/load;
11. enter fixed dungeon;
12. explore, fight, loot, extract;
13. return and verify colony fingerprint;
14. repeat at 60x20 for compact-critical steps.

### Task 10.6 — Real PTY gate

Run the actual `bd` launcher/binary with isolated XDG roots at 80x24 and 60x20.
Verify:

- alternate screen restoration;
- cursor restoration;
- Press/Repeat/Release behavior;
- held/repeated movement policy;
- resize from 60x20 to 80x24 and back;
- no modal input leakage;
- save path isolation.

Automate only if the selected PTY library/tool is already available or owner
approved. Otherwise retain an exact manual script and capture results.

For each PTY profile, compare the observed screen and interaction to the
matching canonical scene row. Record terminal implementation, profile size,
fixture/scenario ID, result, and any intentional terminal-specific difference
in `testing/VISUAL-ACCEPTANCE-MATRIX.md`. ANSI/raster capture may support human
review, but it does not replace automated semantic/canvas/style/geometry
evidence.

### Task 10.7 — Close the complete visual matrix

Before final acceptance:

1. run every canonical scene at both supported profiles;
2. run every visual invariant in Section 5.4;
3. add or complete title, dungeon exploration/combat/loot, extraction, game
   over, day-summary, and save/load scenes not owned by Phases 5–9;
4. review every changed canvas, style, semantic, and geometry snapshot;
5. run all transition tests from a clean process and isolated save root;
6. complete the PTY evidence columns;
7. verify every applicable ledger row is `Accepted`;
8. verify no row is accepted through `output.contains(...)` evidence alone.

The final visual gate fails if:

- any snapshot is pending;
- any fixture renders nondeterministically;
- only one terminal profile passes;
- a semantic category lacks resolved-cell or legend evidence;
- any expected state transition lacks before/after evidence;
- a PTY observation contradicts its `TestBackend` scene;
- an unexplained visual diff is accepted.

---

## 18. Final Traceability Matrix

| Reopened contract | Domain | Schedule/state | Visual evidence | Player path | PTY | Authority |
|---|---|---|---|---|---|---|
| Management does not advance time | action classification | exact state diff | modal/result | `c`/`e` confirmation | physical keys | THC-01, GDD 3/6 |
| Worker motion is deterministic | path/occupancy | one step per turn | activity/position | Wait/Rest | terminal observation | THC-02 |
| Work requires arrival | range/work rule | daily transaction | forecast/activity | assign→work→Rest | terminal observation | THC-03, D-17 |
| Build cannot trap player | egress validator | atomic rejection | S/C/Y/G/T invalid preview | corner sequence | terminal sequence | THC-04, GDD 6 |
| Player always visible | viewport math | player position | S/C/Y/G/T at edges/resize | edge traversal | both sizes/resize | THC-04, D-02/D-16 |
| Nodes are discoverable | target/indicator rule | assignment target | S/C/Y/G/T edge indicator | assign and navigate | both sizes | THC-04, D-17 |
| Visual language unambiguous | symbol validation | overlap invariant | S/C/Y plus legend/fallback | Help inspection | both sizes | THC-05, Kernel |
| Compact UI complete | layout constraints | n/a | C/Y/G/T at 60x20 | Build/Manage/Help | 60x20 | THC-06, D-16 |
| Save/load preserves new state | fingerprint | restore/next step | equivalent S/C/Y/G | F5/F9 | isolated path | D-09/D-17 |

No row is complete if only one column passes.

---

## 19. Validation Profiles

### Focused task gate

Run the exact command listed in the task.

### Phase gate

Run all tests named in the phase plus:

```bash
cargo fmt --all -- --check
cargo check --workspace
git diff --check
```

### Full PR gate

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo test -p bd_app --test foundation_scenario -- --test-threads=1
cargo test -p bd_app --test foundation_stabilization -- --test-threads=1
cargo test -p bd_app --test mvp_correction -- --test-threads=1
cargo test -p bd_app --test colony_day_cycle -- --test-threads=1
cargo test -p bd_app --test persistence -- --test-threads=1
cargo test -p bd_app --test phase6_input -- --test-threads=1
cargo test -p bd_tui --test input_help -- --test-threads=1
cargo run -p bd_app -- --validate
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
```

Add every new integration target from this plan to the full gate as soon as it
exists.

The full gate must also verify:

```bash
cargo test -p bd_tui --test visual_scene_contract -- --test-threads=1
cargo test -p bd_tui --test visual_buffer_contract -- --test-threads=1
cargo test -p bd_tui --test visual_transition_contract -- --test-threads=1
cargo test -p bd_tui --test viewport_contract -- --test-threads=1
cargo test -p bd_tui --test visual_language_contract -- --test-threads=1
cargo insta pending-snapshots
```

If a named target does not exist because its owning phase has not begun, the
phase record must say `Not yet applicable`; it may not silently omit the
target after that phase is complete.

### Final acceptance gate

The full PR gate plus:

- all new semantic snapshots reviewed;
- all required canvas, style, semantic, and geometry snapshots reviewed;
- no pending or automatically accepted snapshot changes;
- all applicable rows in `testing/VISUAL-ACCEPTANCE-MATRIX.md` accepted;
- deterministic property matrix;
- production-path stress profile;
- complete 80x24 player path;
- compact 60x20 player path;
- real PTY terminal restoration and resize;
- GDD review;
- documentation status update.

---

## 20. Phase Completion Record Template

Append one record per completed phase. Do not rewrite earlier records.

```text
### Phase N completion — YYYY-MM-DD

Scope:

Red evidence:
- test:
- command:
- expected failure:

Implementation:
- files:
- behavior:

Focused validation:
- command:
- result:

Full regression:
- command:
- result:

Manual evidence:
- profile:
- scenario:
- observed result:

GDD review:
- sections:
- drift found:

Residual risk:

Next phase ready: yes/no
```

A phase is not complete if any field is omitted.

---

## 21. Final Completion Criteria

This plan is complete only when all of the following are true:

- the test evidence ledger distinguishes acceptance from legacy/deferred
  coverage;
- no active required test passes through conditional or missing assertions;
- misleading test names have been corrected or strengthened;
- production schedule ordering is directly tested;
- test helpers expose frame settling and stable identity explicitly;
- normalized fingerprints cover all authoritative Foundation state;
- assignment confirmation and cancellation do not advance time;
- worker movement is deterministic, collision-safe, and visible;
- physical arrival controls production and gathering;
- Rest and individual turns produce equivalent movement/day outcomes;
- every accepted build preserves gate reachability;
- the player remains visible throughout the 40x30 shelter;
- required resource targets are visible or explicitly indicated at both
  profiles;
- stations, survivors, and resource nodes have unambiguous semantic
  presentation;
- visual tests detect style-only, modifier-only, layering, overlap, stale-cell,
  viewport, resize, and fallback-mode regressions;
- every canonical scene has semantic, canvas, style, geometry, transition, and
  PTY evidence where Section 5.3 requires it;
- Help explains the active map language;
- 60x20 text is complete and controls agree;
- save/load preserves every new state and next-step behavior;
- full workspace, strict Clippy, validation, formatting, and diff checks pass;
- real 80x24 and 60x20 player paths pass;
- the final result is checked against GDD Sections 3, 6, 8, and 9;
- `MVP-SCENARIO.md`, the documentation hub, inventory, and this plan report the
  same acceptance status.

The plan must not be marked complete because test count increased. Completion
requires the player-facing and architectural contracts above to be proven at
the correct evidence layers.

---

### Phase 0 completion — 2026-07-25

Scope:

- authority alignment, active-suite classification, misleading-name audit,
  and canonical visual acceptance ledger.

Red evidence:

- test: n/a; documentation-only evidence phase.
- command: `cargo test --workspace -- --list`.
- expected failure: n/a; the ledger records known evidence gaps as Open rather
  than manufacturing a test failure.

Implementation:

- files: `testing/FOUNDATION-TEST-EVIDENCE.md`,
  `testing/VISUAL-ACCEPTANCE-MATRIX.md`, and previously aligned authority docs.
- behavior: no production behavior changed.

Focused validation:

- command: authority `rg`, ledger inspection, and `git diff --check`.
- result: one active plan is named; every visual row begins explicitly
  unaccepted; no whitespace errors.

Full regression:

- command: baseline formatting, check, workspace tests, strict Clippy, and
  `bd_app --validate`.
- result: 462 listed: 460 passed, 0 failed, 2 ignored; all other baseline gates
  passed.

Manual evidence:

- profile: n/a for this documentation-only phase.
- scenario: existing deep 80x24/60x20 audit is linked by D-18 and
  `MVP-SCENARIO.md`.
- observed result: affected UX gates remain reopened.

GDD review:

- sections: 3, 6, and 8.
- drift found: none; the ledgers activate no Product P2 feature.

Residual risk:

- all visual matrix rows remain unimplemented by design.

Next phase ready: yes

### Phase 1 completion — 2026-07-25

Scope:

- false-positive schedule, stress, naming, and substring-test repairs.

Red evidence:

- test: `production_schedule_executes_required_sets_in_declared_order` and
  `accepted_move_emits_each_required_trace_stage_exactly_once`.
- command:
  `cargo test -p bd_core --test architecture -- --test-threads=1`.
- expected failure: the first strengthened trace assertion found two
  legitimate signal records sharing the generic Mutation stage.

Implementation:

- files: `crates/bd_core/tests/architecture.rs`,
  `crates/bd_app/tests/stress.rs`,
  `crates/bd_app/tests/mvp_correction.rs`,
  `crates/bd_tui/src/lib.rs`, `crates/bd_tui/src/render_grid.rs`, and the test
  evidence ledger.
- behavior: production behavior and public APIs are unchanged; tests now
  distinguish schedule-set execution from multiple signals inside Mutation.

Focused validation:

- command: architecture, stress, MVP-correction, TUI library, and input-help
  targets listed in Phase 1.
- result: all focused targets passed.

Full regression:

- command: `cargo test --workspace`, formatting check, workspace check, strict
  Clippy, and `git diff --check`.
- result: 465 listed: 463 passed, 0 failed, 2 ignored; all other gates passed.

Manual evidence:

- profile: n/a; this phase changes test truthfulness only.
- scenario: no player-facing behavior changed.
- observed result: no manual UI claim was used to close a gate.

GDD review:

- sections: 3, 6, and 8.
- drift found: none; procgen remains deferred and visual/discoverability claims
  remain open for their owning phases.

Residual risk:

- normalized fingerprints and explicit frame-settling APIs remain Phase 2;
  real visual snapshots remain Phase 4.

Next phase ready: yes

---

## 22. Failure-Driven Remediation Program — 2026-07-26

### 22.1 Authority and effect on the earlier sequence

This section is the binding execution overlay for the 23 active Foundation
failures recorded on 2026-07-26. It does not create a second plan.

- Sections 1–6, the locked THC contracts, evidence rules, stop conditions, and
  final acceptance criteria remain unchanged.
- Completed Phase 0 and Phase 1 records remain historical evidence.
- For unresolved work in Sections 9–17, execute remediation phases F0–F9 below
  in their stated order.
- Where an older unresolved task and this section overlap, this section owns
  the immediate task order and the older section continues to own the broader
  acceptance requirement.
- A remediation phase may close its named red tests, but it does not by itself
  close the corresponding visual-matrix or Foundation acceptance row.
- D-18, D-19, the root GDD, and `Kernel.md` remain higher authority.

This overlay exists because fixing the 23 tests one at a time would preserve
the underlying architectural defects. The failures must be closed through
shared state, shared domain evaluators, and explicit schedule boundaries.

### 22.2 Recorded baseline

The latest complete run recorded:

```text
542 tests listed
517 passed
23 failed
2 ignored
```

Only these targets were red:

```text
bd_app --test survivor_work_contract    13 failed
bd_app --test phase6_input               8 failed
bd_tui --lib                             2 failed
```

Formatting, strict Clippy, content validation, contract-registry validation,
doctests, and `git diff --check` passed. The failure-discovery sweep changed
tests and testing governance only; it did not implement production behavior.

The number `542` is baseline evidence, not a permanent expected count. New
atomic tests required below may increase it. Final reporting must always state
listed, passed, failed, and ignored counts separately.

### 22.3 Confirmed failure inventory and ownership

| Owner | Failing test | Confirmed defect |
|---|---|---|
| F2 | `worker_uses_pathfinding_around_a_wall_blocker` | Movement takes a direct axis step and does not use the existing A* adapter |
| F2 | `unreachable_worker_stays_put_and_reports_a_specific_blocked_reason` | No typed Blocked activity or specific transition feedback exists |
| F2 | `station_worker_never_enters_the_station_tile` | The station tile, rather than an adjacent work tile, is used as the destination |
| F2 | `station_worker_stops_cardinally_adjacent_to_target` | Movement does not select and retain a valid work tile |
| F2 | `gatherer_never_enters_a_resource_node_tile` | Resource fixtures are treated as walkable destinations |
| F2 | `assigned_survivors_never_stack_on_one_tile` | There is no stable movement order or destination reservation |
| F2 | `rest_and_individual_waits_produce_the_same_worker_position` | Rest jumps time and does not replay equivalent Outpost worker steps |
| F3 | `assigned_but_enroute_station_worker_produces_nothing` | Station output counts assignment without physical work |
| F3 | `adjacent_station_worker_produces_once` | Movement pulls an already valid worker into the station and prevents correct work evaluation |
| F3 | `assigned_but_enroute_gatherer_produces_nothing` | Gathering counts assignment without physical work |
| F3 | `gatherer_at_wrong_node_type_produces_nothing` | Gathering searches for any matching node instead of evaluating the node beside the worker |
| F3 | `blocked_station_worker_produces_nothing` | Blocked assignment still counts as staffed production |
| F3 | `forecast_excludes_enroute_worker_output` | Forecast duplicates assignment-only production semantics |
| F4 | `station_staffing_lists_station_assignments_not_gathering_tasks` | Staffing and task modes share one combined choice list |
| F4 | `task_management_lists_survivor_tasks_not_station_staffing_choices` | Task management exposes station choices |
| F4 | `management_cancel_is_atomic_and_discards_modal_gameplay_input` | Same-batch routing does not predict or retain modal ownership |
| F5 | `entering_build_placement_starts_on_a_visible_adjacent_candidate` | The visible ghost starts on the player while confirmation uses another tile |
| F5 | `invalid_build_confirmation_keeps_preview_active_and_is_atomic` | Confirmation clears placement state before the core denial resolves |
| F5 | `build_placement_exposes_selected_station_name_cost_and_effect` | Placement projection loses the selected catalog detail |
| F6 | `altar_and_idle_survivor_remain_distinct_without_color` | Idle survivor and Altar both project as `A` |
| F6 | `workshop_and_water_source_remain_distinct_without_color` | Workshop and Water Source both project as `W` |
| F6 | `staffed_and_unstaffed_station_have_distinct_ascii_projection` | Staffing state is absent from station map projection |
| F7 | `rendered_outpost_help_contains_every_foundation_legend_at_supported_profiles` | The fixed Help layout clips the active legend at both supported profiles |

Confirmed green guard tests must remain green throughout the program:

- assignment does not move a worker immediately;
- idle survivors do not move on accepted Outpost turns;
- idle render frames do not move workers;
- one ordinary Outpost turn permits at most one cardinal worker step;
- Tactical turns do not move colony workers;
- adjacent matching gatherers produce once;
- Rest and individual waits currently agree on daily resources;
- load does not immediately move or produce;
- the next worker step remains deterministic across save/load;
- the zero-Supplies physical-gathering fixture remains recoverable.

A proposed implementation that makes the red tests green by disabling
movement, station output, gathering, Rest, or forecast is invalid because it
must fail one or more of these guards.

### 22.4 Locked implementation decisions

These decisions remove implementation ambiguity for the executing model. Do
not substitute a different architecture without stopping under Section 4.8.

#### 22.4.1 Assignment and activity are different concepts

`SurvivorTask` remains durable player intent. Introduce one typed
`WorkerActivity` read model with these Foundation states:

```text
Idle
EnRoute { target, distance }
Working { target }
Blocked { target, reason }
Resting
Defending
```

Rules:

- `target` is player-facing stable identity, never raw entity bits.
- `Blocked` has a typed reason. Foundation reasons must distinguish at least:
  `MissingTarget`, `TargetUnavailable`, `NoAdjacentWorkTile`, `NoRoute`, and
  `DestinationReserved`.
- Assignment remains the only durable task truth.
- Activity is derived from task, current position, target state, map, and
  occupancy. It is not a second assignment system.
- A cached ECS activity component is permitted only as a projection cache
  produced by the shared resolver.
- Activity is not independently persisted. Restore the assignment and
  position, then recompute activity without moving or producing.
- Production, forecast, logs, and view models consume the same activity/work
  evaluation. They must not each infer a different activity.
- Blocked feedback is emitted on transition into a new blocked state or when
  its reason/target changes, not every render/update frame.

#### 22.4.2 Time advancement has an explicit plan

Replace boolean-only worker scheduling with one typed accepted-action time
plan. The conceptual fields are:

```text
elapsed_turns
outpost_worker_steps
cause
```

Required semantics:

- a normal time-advancing Outpost action requests one elapsed turn and one
  Outpost worker step;
- a Tactical action may advance time but requests zero Outpost worker steps;
- Rest requests the exact number of elapsed turns until the next day and the
  same number of Outpost worker steps;
- management, Help, Build selection/navigation, and invalid/cancelled
  interactions request neither elapsed turns nor worker steps;
- each logical worker step completes before evaluating a day boundary crossed
  by that step;
- worker activity is refreshed after movement and before daily physical-work
  evaluation;
- result publication and view-model building occur only after authoritative
  mutation is complete.

Use named internal system sets chained within the existing `BdSet` pipeline.
Do not reorder or duplicate the top-level kernel sets. The intended internal
mutation order is:

```text
accepted action mutation
→ time-plan compilation
→ Outpost worker-step replay
→ activity refresh
→ GameTime/day-boundary mutation
→ physical daily transaction
→ result/summary emission
→ view-model build
```

If Bevy message visibility requires an explicit deferred-command barrier or a
one-frame handoff, encode that behavior in a named schedule test. Do not rely
on registration order or on multiple systems merely sharing
`BdSet::Mutation`.

`ShouldAdvanceTime` may be adapted temporarily at one boundary, but it may not
remain an independent second source for worker-step count.

#### 22.4.3 Stable identity and ordering

- Survivors are ordered by unique survivor name for Foundation movement and
  player-facing menus.
- Starter survivor names must be validated as unique.
- Stations are selected by a stable tuple containing station content ID/type
  and world position. Duplicate station types are therefore safe.
- Resource targets are selected by resource kind and world position.
- Raw Bevy entity bits may remain internal relationship storage where already
  required, but they may not determine menu order, movement order, forecast
  order, logs, fingerprints, or view-model identity.
- Any query whose result can change gameplay must sort explicitly before use.

#### 22.4.4 Movement and occupancy

Use the existing `bd_core::pathfinding::AStarPathfinder`; do not add a second
pathfinding implementation or crate.

For each logical Outpost worker step:

1. snapshot permanent blockers: walls, stations, resource nodes, and other
   non-walkable fixtures;
2. include the player as an occupied tile;
3. snapshot survivor positions;
4. process assigned survivors in stable name order;
5. derive all walkable cardinal tiles adjacent to the target;
6. find paths to those work tiles, never to the blocking target;
7. choose deterministically by shortest path, then `y`, then `x`;
8. reserve the accepted next destination immediately;
9. move at most one cardinal tile;
10. refresh activity after movement.

Existing survivor positions and destinations already reserved by an earlier
worker are unavailable. A survivor may remain on its own starting tile.
Workers never swap through one another in a single logical step.

For gathering, choose the nearest reachable matching non-depleted node by
shortest reachable work path, then node `y`, then node `x`. Manhattan distance
alone is not sufficient.

#### 22.4.5 Physical work has one evaluator

Create one core physical-work evaluator used by:

- station production;
- survivor gathering;
- colony forecast;
- day-summary contribution reporting;
- worker activity projection.

It returns a typed contribution or a typed no-work reason. It does not mutate
resources.

Station work is valid only when:

- the assigned station exists;
- the station is staffing-eligible under its catalog record;
- the worker is cardinally adjacent to that exact station;
- the worker is not EnRoute or Blocked.

Gathering work is valid only when:

- the worker is cardinally adjacent to a non-depleted node;
- that node's resource kind matches the durable gathering task;
- the worker is not EnRoute or Blocked.

A worker beside the wrong node does not receive credit for a remote matching
node. A station contributes at most once per day transaction according to its
catalog staffing capacity. A gatherer contributes at most once per day
transaction.

Forecast describes output valid at the current physical state. It does not
predict arrival before the next boundary in this Foundation pass.

#### 22.4.6 Management is one typed reducer with two modes

Use one pure interaction reducer that processes input events sequentially and
owns the predicted state for the entire input batch.

Task-management choices are exactly:

```text
Idle
Gather Supplies
Gather Materials
Gather Wild Plants
Rest
```

Do not expose station assignment choices in task mode.

Station-staffing choices contain only built, staffing-eligible stations, using
stable station selectors and catalog labels. If unassignment is needed for an
already staffed survivor, expose one explicitly named `Unstaff station`
choice. Do not expose gathering tasks in staffing mode.

The reducer emits typed UI transitions and at most one validated gameplay
action on confirmation. Choice records own their stable selector, label,
detail, enabled state, and denial reason; behavior must not reconstruct meaning
from a displayed string or hard-coded index.

Once an input batch enters management, management owns all remaining events in
that batch. Cancellation closes the modal at the end of routing and discards
all uncommitted modal input. A gameplay key buffered between open and Escape
must never execute in normal mode.

#### 22.4.7 Build is one transaction state

Replace the independent active/pending flags as behavioral authorities with
one transient build interaction:

```text
Inactive
Selecting { selected_station }
Placing { selected_station, cursor, validation }
AwaitingResolution { selected_station, cursor }
```

Legacy resources may be retained temporarily only as adapters during one
atomic migration. They may not remain independently writable afterward.

The shared cardinal candidate order is:

```text
East, South, West, North
```

Entering placement starts on the first in-bounds adjacent candidate in that
order. The visible cursor is exactly the target Enter submits.

Confirmation rules:

- an already-known invalid preview emits no build action and remains
  `Placing`;
- a valid preview submits once and becomes `AwaitingResolution`;
- duplicate confirmation is ignored while awaiting;
- matching success closes the interaction;
- matching denial returns to `Placing` at the same cursor with the typed reason
  visible;
- cancellation is atomic from Selecting, Placing, or AwaitingResolution where
  cancellation is still safe;
- player position never changes during the workflow.

The placement view model resolves selected name, cost, effect, staffing
requirement, available resources, and denial reason from the authoritative
station catalog. The TUI does not copy station formulas or costs.

#### 22.4.8 Foundation ASCII lock

The Foundation monochrome fallback is locked for this remediation:

| Category/state | Glyph |
|---|---|
| Player | `@` |
| Survivor Idle | `i` |
| Survivor EnRoute | `e` |
| Survivor Working | `*` |
| Survivor Blocked | `x` |
| Survivor Resting | `r` |
| Survivor Defending | `d` |
| Stove unstaffed/staffed | `f` / `F` |
| Altar unstaffed/staffed | `a` / `A` |
| Workshop unstaffed/staffed | `w` / `W` |
| Bed unstaffed/staffed | `b` / `B` |
| Storage unstaffed/staffed | `s` / `S` |
| Trees | `T` |
| Water Source | `~` |
| Wild Plants | `P` |
| Shelter gate | `>` |

These are data, not renderer conditionals:

- station content owns explicit unstaffed and staffed symbol references;
- resource content owns its semantic symbol reference;
- worker activity maps to activity-specific semantic symbols;
- the symbol registry resolves glyph, fallback glyph, style, layer, and
  priority;
- Help reads the same catalog/registry definitions.

Do not implement staffing by blindly uppercasing an arbitrary custom glyph.
Custom station data must provide both states. Content validation rejects a
missing symbol, missing fallback, or collision between categories that can be
visible together.

#### 22.4.9 Help is a dedicated responsive screen

Help uses the full inner terminal canvas instead of sharing the normal
stats/log panel layout. It contains:

- contextual controls derived from `CommandBindings`;
- a Foundation shelter legend derived from the symbol and station catalogs;
- concise worker-state explanations;
- off-screen indicator explanation when applicable;
- one consistent close control in body and footer.

At 80x24 and 60x20, use measured grouped columns or wrapped sections. Every
required entry must be present in the final Ratatui buffer, not merely in the
Help view model.

### 22.5 Common execution protocol for F0–F9

For every phase:

1. run `git status --short` and preserve unrelated owner changes;
2. run the phase's focused tests before editing;
3. record every existing red test and its actual assertion failure;
4. add the phase's missing atomic tests before production edits;
5. run those tests and record the intended red result;
6. implement one root-cause slice only;
7. rerun the smallest focused test after each meaningful change;
8. run the complete owning test target;
9. run the phase regression set;
10. run formatting, workspace check, strict Clippy, content validation, the
    contract-registry target, and `git diff --check`;
11. inspect the GDD and decisions named by that phase;
12. append a completion record using Section 20.

Do not:

- change an expected value merely to match current output;
- replace an exact assertion with `contains` or a count-only assertion;
- mark a required test ignored;
- add mutable `World` access to test support;
- expose production internals solely for tests;
- add a test-only movement, activity, forecast, render, or validation path;
- duplicate a rule in the TUI and core;
- update a registry row from `Red` before its primary and support tests pass;
- update `GreenUnreviewed` to `Accepted` without all required evidence layers;
- run broad formatting rewrites over unrelated dirty files.

If a phase exposes an unexpected failure in a previously green required
contract, stop that phase, add the failure to its root-cause inventory, and
revise this section before continuing.

### 22.6 Phase F0 — Baseline containment and dependency proof

#### Goal

Prove that the 23 failures and their ownership still match the recorded
baseline before production work begins.

#### Production changes

None.

#### Tasks

1. Run the three red targets independently with one test thread.
2. Save the exact failed-test list in the F0 completion record.
3. Confirm that all 23 names in Section 22.3 still exist and no additional
   test in those targets is red.
4. Run `cargo test --workspace -- --list` and report listed tests separately
   from executed results.
5. Run the contract-registry target and confirm:
   - all required records parse;
   - every red record has a known failure;
   - no required record is Deferred or Retired;
   - primary-test uniqueness still passes.
6. Run all green guard tests named in Section 22.3 with exact filters.
7. Inspect current schedule registration in `bd_core/src/lib.rs`,
   `actions.rs`, `time.rs`, and the three colony work modules. Record the
   actual current ordering; do not infer it from source-file order.
8. Confirm the existing A* adapter supports a dynamic blocker set and returns
   a path that includes start and destination.

#### Commands

```bash
cargo test -p bd_app --test survivor_work_contract -- --test-threads=1
cargo test -p bd_app --test phase6_input -- --test-threads=1
cargo test -p bd_tui --lib -- --test-threads=1
cargo test -p bd_test_support --test contract_registry -- --test-threads=1
cargo test --workspace -- --list
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo run -p bd_app -- --validate
git diff --check
```

#### Exit criteria

- The failure set is exactly understood and mapped.
- No production file changed.
- Previously green guards pass.
- No hidden schedule or pathfinder mismatch invalidates F1/F2.
- If the failure count differs, Section 22.3 is corrected before F1.

#### Authority check

GDD Sections 3, 6, and 8; D-18; Kernel mutation and schedule discipline.

### 22.7 Phase F1 — Activity authority and explicit time-step schedule

#### Goal

Create the shared activity and schedule boundaries required by movement and
physical work without yet implementing full path selection.

#### Primary production files

- `crates/bd_core/src/time.rs`;
- `crates/bd_core/src/lib.rs`;
- `crates/bd_core/src/actions.rs`;
- `crates/bd_core/src/colony/survivors.rs`;
- `crates/bd_core/src/save.rs` only if restore hooks require it;
- `crates/bd_tui/src/view_models.rs` only for typed projection plumbing.

#### Tests first

Add or strengthen discrete tests for:

- assignment plus a distant valid target derives `EnRoute`;
- cardinal adjacency to the exact target derives `Working`;
- missing target derives `Blocked(MissingTarget)`;
- no reachable work tile derives `Blocked(NoRoute)` or
  `Blocked(NoAdjacentWorkTile)` as appropriate;
- Idle and Resting are not inferred from display strings;
- activity resolution is independent of ECS query iteration order;
- restoring assignment and position recomputes the same activity without
  moving;
- entering Blocked emits one named transition message;
- an unchanged Blocked state does not spam another message;
- one accepted Outpost Wait produces one logical Outpost worker step;
- one Tactical turn produces zero Outpost worker steps;
- Rest from turns `0`, `1`, `22`, and `23` produces respectively `24`, `23`,
  `2`, and `1` logical Outpost worker steps;
- the final logical step is processed before work at a crossed day boundary;
- management and Build navigation produce zero time plans.

Each test should assert typed state or exact schedule counters. Do not inspect
logs as the sole proof of activity or ordering.

#### Implementation tasks

1. Add `WorkerActivity`, `WorkerTarget`, and typed `WorkerBlockedReason` in the
   colony domain.
2. Add one pure activity/work-position resolver that accepts explicit inputs.
3. Add one typed time-advance plan emitted from accepted action semantics.
4. Introduce named internal schedule sets for action mutation, Outpost steps,
   activity refresh, day-boundary mutation, and daily work.
5. Chain those sets under the existing top-level kernel pipeline.
6. Adapt normal Outpost turns, Tactical turns, and Rest to the typed plan.
7. Remove worker movement's direct dependency on a boolean
   `ShouldAdvanceTime.0`.
8. Recompute activity after assignment without movement and after each logical
   step.
9. Ensure restore recomputes activity on a non-advancing frame.
10. Project typed activity into view models without choosing glyphs yet.

#### Focused validation

```bash
cargo test -p bd_core time -- --test-threads=1
cargo test -p bd_core colony::survivors -- --test-threads=1
cargo test -p bd_core --test architecture -- --test-threads=1
cargo test -p bd_app --test survivor_work_contract new_assignment -- --test-threads=1
cargo test -p bd_app --test survivor_work_contract tactical_turns -- --test-threads=1
cargo test -p bd_app --test survivor_work_contract load_does_not_immediately -- --test-threads=1
```

#### Exit criteria

- There is one activity resolver and one time-plan authority.
- Assignment remains durable and activity remains derived.
- Paused interactions emit no time plan.
- Rest step count is explicit.
- Tactical turns cannot move colony workers.
- Load cannot trigger movement, production, or a time advance.
- Full `survivor_work_contract` may remain red only for F2/F3-owned pathing or
  work-evaluation assertions.

#### GDD check

GDD Sections 3 and 6, D-18 worker-activity and Rest clauses, Kernel Sections 7
and 9 schedule discipline. Confirm no real-time simulation or Product P2
survivor behavior was introduced.

### 22.8 Phase F2 — Deterministic pathing, work tiles, and reservations

#### Goal

Close all movement, target-occupancy, collision, Blocked, and Rest-position
failures through one deterministic movement resolver.

#### Primary production files

- `crates/bd_core/src/colony/survivors.rs`;
- `crates/bd_core/src/pathfinding.rs` only for reusable adapter extension;
- `crates/bd_core/src/lib.rs`;
- `crates/bd_core/src/gamelog.rs` only if typed transition formatting belongs
  there.

#### Tests first

The seven existing F2 red tests remain primary. Add missing atomic coverage:

- a worker already on a valid work tile does not move;
- a worker routes to the shortest reachable adjacent work tile, not the
  geometrically nearest blocked one;
- equal-length work-tile choices use the documented `y`, then `x` tie-break;
- resource-node target ties use reachable path length and stable position;
- the player tile is unavailable to a worker;
- a reserved tile is unavailable to a later worker;
- two workers cannot swap positions in one step;
- removing an assigned station produces `MissingTarget`;
- depleting the selected resource target causes deterministic retargeting;
- a matching node with no adjacent work tile reports the specific reason;
- repeated construction/entity allocation does not alter movement order;
- a worker that becomes reachable leaves Blocked on the next Outpost step;
- Rest and repeated Wait agree at every intermediate logical step in a
  schedule probe, not only at the final day.

#### Implementation tasks

1. Build one movement-step context from the map, player, permanent blockers,
   resource nodes, stations, and current survivor positions.
2. Sort survivors by unique name.
3. Resolve each worker's stable target and candidate cardinal work tiles.
4. Call the existing A* adapter for each candidate with the same blocker
   semantics.
5. Choose the deterministic best path.
6. Reserve the next tile before evaluating the next survivor.
7. Move at most one cardinal tile.
8. Never move into the target fixture, player, survivor, or reservation.
9. Refresh activity and emit transition feedback after the move.
10. Replay this exact function once per F1 logical Outpost step, including
    every step requested by Rest.

Keep target resolution, path choice, movement, and activity refresh as
separate single-responsibility functions. The Bevy system orchestrates them;
it does not contain four duplicate branches for task types.

#### Required green results

- `worker_uses_pathfinding_around_a_wall_blocker`;
- `unreachable_worker_stays_put_and_reports_a_specific_blocked_reason`;
- `station_worker_never_enters_the_station_tile`;
- `station_worker_stops_cardinally_adjacent_to_target`;
- `gatherer_never_enters_a_resource_node_tile`;
- `assigned_survivors_never_stack_on_one_tile`;
- `rest_and_individual_waits_produce_the_same_worker_position`.

#### Focused validation

```bash
cargo test -p bd_core pathfinding -- --test-threads=1
cargo test -p bd_core colony::survivors -- --test-threads=1
cargo test -p bd_app --test survivor_work_contract worker_ -- --test-threads=1
cargo test -p bd_app --test survivor_work_contract station_worker_ -- --test-threads=1
cargo test -p bd_app --test survivor_work_contract gatherer_never_ -- --test-threads=1
cargo test -p bd_app --test survivor_work_contract assigned_survivors_ -- --test-threads=1
cargo test -p bd_app --test survivor_work_contract rest_and_individual_waits_produce_the_same_worker_position -- --exact --test-threads=1
```

#### Exit criteria

- All seven F2 failures are green.
- All F1 activity/schedule tests remain green.
- Existing green movement guards remain green.
- The same path and reservation resolver serves station and gathering tasks.
- Movement behavior has no raw entity-order dependency.
- F3 production tests may remain red, but movement may no longer be their
  reason.

#### GDD check

GDD Section 6 Shelter and Colony, Section 8 MVP Foundation, D-18 physical
worker clauses. Confirm Idle survivors still do not wander and no needs,
morale AI, raids, or events were added.

### 22.9 Phase F3 — Physical station/gathering truth and forecast parity

#### Goal

Make actual daily output and forecast consume one physical-work evaluator.

#### Primary production files

- `crates/bd_core/src/colony/production.rs`;
- `crates/bd_core/src/colony/resources.rs`;
- `crates/bd_core/src/colony/survivors.rs`;
- `crates/bd_core/src/lib.rs`;
- `crates/bd_tui/src/view_models.rs` only to display authoritative forecast and
  activity.

#### Tests first

The six existing F3 red tests remain primary. Add:

- one station with two assigned adjacent workers contributes no more than its
  catalog staffing capacity;
- one worker cannot contribute to two stations in one day transaction;
- one gatherer adjacent to two matching nodes contributes once;
- a depleted matching node contributes zero;
- a missing assigned station contributes zero with a typed reason;
- a worker moved into Working on the final turn before a day boundary
  contributes once;
- a worker still EnRoute after the final step contributes zero;
- a Tactical day boundary uses the same physical state evaluator without
  moving colony workers;
- forecast and execution compare every contribution field for Idle, EnRoute,
  Working, Blocked, wrong-node, and depleted-node cases;
- daily summary names EnRoute/Blocked workers that contributed zero without
  claiming they worked.

#### Implementation tasks

1. Define an immutable `WorkerContribution`/`NoWorkReason` result.
2. Implement station evaluation against the exact assigned station and
   cardinal adjacency.
3. Implement gathering evaluation against the node physically adjacent to the
   worker and matching the task kind.
4. Make station production aggregate only valid contributions.
5. Make survivor gathering aggregate only valid contributions.
6. Make `forecast_colony` call the same evaluator and aggregation functions
   with current state.
7. Ensure the daily transaction executes exactly once for each emitted day
   boundary.
8. Preserve catalog-owned amounts and existing signed-delta mutation paths.
9. Remove old assignment-only loops once all consumers are migrated. Do not
   leave them as fallback behavior.
10. Project contribution/activity explanations without recomputing formulas in
    the TUI.

#### Required green results

- `assigned_but_enroute_station_worker_produces_nothing`;
- `adjacent_station_worker_produces_once`;
- `assigned_but_enroute_gatherer_produces_nothing`;
- `gatherer_at_wrong_node_type_produces_nothing`;
- `blocked_station_worker_produces_nothing`;
- `forecast_excludes_enroute_worker_output`.

#### Focused validation

```bash
cargo test -p bd_core colony::production -- --test-threads=1
cargo test -p bd_core colony::resources -- --test-threads=1
cargo test -p bd_app --test survivor_work_contract -- --test-threads=1
cargo test -p bd_app --test colony_day_cycle -- --test-threads=1
cargo test -p bd_app --test mvp_correction forecast -- --test-threads=1
```

#### Exit criteria

- All 13 `survivor_work_contract` failures are green.
- Positive station and gathering guards remain green.
- Zero-Supplies recovery remains physically reachable.
- Rest and Wait agree on both position and resources.
- Forecast and execution share code and agree field by field.
- There is no assignment-only production fallback.

#### GDD check

GDD Section 6 Shelter and Colony and Section 8 MVP Foundation; D-17 and D-18
economy/work clauses; Kernel signed-delta and DRY rules. Confirm no new balance
values or station effects were invented.

### 22.10 Phase F4 — Management mode separation and atomic input routing

#### Goal

Close the three management failures through one reducer with explicit task and
staffing choice models.

#### Primary production files

- `crates/bd_tui/src/lib.rs`;
- `crates/bd_tui/src/commands.rs`;
- `crates/bd_tui/src/view_models.rs`;
- `crates/bd_tui/src/screens.rs`;
- `crates/bd_core/src/colony/survivors.rs` only if a typed selection action
  requires a domain adapter.

#### Tests first

The three existing F4 red tests remain primary. Add production-key tests for:

- open `c`, navigate survivor, navigate task, confirm, and close;
- open `e`, navigate survivor, navigate built station, confirm, and close;
- each mode exposes only its authorized choice type;
- task and staffing choice order is stable after unrelated entity allocation;
- duplicate station types remain independently selectable by stable station
  selector;
- a station removed while its modal is open produces a typed denial and no
  unrelated assignment;
- Escape at each selection stage is atomic;
- `c`, gameplay key, Escape in one batch leaks no gameplay;
- `e`, gameplay key, Escape in one batch leaks no gameplay;
- Repeat/Release events do not confirm or navigate a press-only modal;
- confirmation emits exactly one action and one result;
- cancellation emits no action, replay record, time plan, resource delta, or
  worker movement;
- modal and footer controls agree for both modes.

Use normalized allowed-state diffs. Logs alone are insufficient.

#### Implementation tasks

1. Replace combined string choices with typed task and station choice records.
2. Replace raw entity-bit ordering with stable survivor and station selectors.
3. Implement the pure management reducer.
4. Route every event in an input batch through the reducer's predicted state.
5. Retain modal ownership through the end of a batch once entered.
6. Flush uncommitted modal events after cancellation.
7. Emit one validated typed action only after explicit confirmation.
8. Keep management paused and preserve the F1 zero-time-plan rule.
9. Build distinct view models and titles for task management and staffing.
10. Remove old index-to-action match logic and the shared mixed list after the
    typed path is complete.

#### Required green results

- `station_staffing_lists_station_assignments_not_gathering_tasks`;
- `task_management_lists_survivor_tasks_not_station_staffing_choices`;
- `management_cancel_is_atomic_and_discards_modal_gameplay_input`.

#### Focused validation

```bash
cargo test -p bd_app --test phase6_input station_staffing_ -- --test-threads=1
cargo test -p bd_app --test phase6_input task_management_ -- --test-threads=1
cargo test -p bd_app --test phase6_input management_cancel_ -- --test-threads=1
cargo test -p bd_tui commands -- --test-threads=1
cargo test -p bd_tui --lib management -- --test-threads=1
```

#### Exit criteria

- All three F4 failures are green.
- `c` and `e` are behaviorally and visually distinct.
- Query/entity allocation order cannot change selection identity.
- Cancellation is atomic even with same-batch buffered input.
- Management remains paused and assignment still does not move a worker
  immediately.

#### GDD check

GDD Sections 3 and 6, D-18 management clauses, MVP scenario explicit
management workflow. Confirm no deeper colony job system or autonomous AI was
added.

### 22.11 Phase F5 — Build selection/placement transaction

#### Goal

Make the visible preview, submitted target, core result, and selected catalog
detail one coherent paused workflow.

#### Primary production files

- `crates/bd_tui/src/lib.rs`;
- `crates/bd_tui/src/view_models.rs`;
- `crates/bd_tui/src/screens.rs`;
- `crates/bd_core/src/colony/stations.rs`;
- `crates/bd_core/src/actions.rs` only for typed build-result correlation.

#### Tests first

The three existing F5 red tests remain primary. Add:

- entering placement starts east of the player in the canonical fixture;
- if east is out of bounds, the shared candidate order chooses the next
  in-bounds cardinal tile;
- moving the preview never moves the player;
- valid confirmation builds on the exact visible preview tile;
- valid confirmation spends and advances exactly once;
- invalid confirmation changes no station, resource, time, replay-success, or
  player state;
- invalid confirmation preserves selected station, cursor, and visible reason;
- duplicate Enter while awaiting resolution submits once;
- matching success closes placement once;
- matching denial returns to Placing without a stale pending build;
- Escape from selection and placement is atomic;
- selected detail comes from the catalog for every Foundation station;
- Storage's complete unavailable reason remains visible;
- a mode change or load cannot restore a half-submitted transient build.

#### Implementation tasks

1. Introduce the single build interaction enum from Section 22.4.7.
2. Move all input transitions into one reducer.
3. Derive the initial cursor from player position and the shared cardinal
   order.
4. Use `validate_station_placement` for preview and submission legality.
5. Submit the exact selected station/cursor pair.
6. Correlate the eventual typed action result with AwaitingResolution.
7. Preserve Placing state and reason on denial.
8. Build selected detail directly from `StationCatalog`.
9. Ensure the preview and player use separate view-model fields and positions.
10. Remove independent writes to old menu/ghost/pending flags after migration.

#### Required green results

- `entering_build_placement_starts_on_a_visible_adjacent_candidate`;
- `invalid_build_confirmation_keeps_preview_active_and_is_atomic`;
- `build_placement_exposes_selected_station_name_cost_and_effect`.

#### Focused validation

```bash
cargo test -p bd_app --test phase6_input entering_build_placement_ -- --test-threads=1
cargo test -p bd_app --test phase6_input invalid_build_confirmation_ -- --test-threads=1
cargo test -p bd_tui --lib build_placement_exposes_ -- --test-threads=1
cargo test -p bd_app --test colony_spatial_contract -- --test-threads=1
cargo test -p bd_core colony::stations -- --test-threads=1
```

#### Exit criteria

- All three F5 failures are green.
- Visible preview and submitted tile are identical.
- Invalid placement stays correctable and is atomic.
- Catalog detail survives the transition from selection to placement.
- Valid placement still preserves shelter egress.
- Build interaction is paused except for one accepted build transaction.

#### GDD check

GDD Section 6 Shelter and Colony, Section 8 MVP Foundation, D-18 egress/build
clauses. Confirm this is usability and transaction repair, not expanded
construction content.

### 22.12 Phase F6 — Semantic map projection and data validation

#### Goal

Close the three ASCII failures and prevent future content from recreating the
same ambiguity.

#### Primary production/data files

- `crates/bd_tui/src/visual.rs`;
- `crates/bd_tui/src/view_models.rs`;
- `crates/bd_tui/src/screens.rs`;
- `crates/bd_core/src/content.rs`;
- `crates/bd_core/src/colony/stations.rs`;
- `crates/bd_data/src/loader.rs`;
- `content/symbols/default.ron`;
- `content/stations/foundation.ron`;
- relevant resource-node content.

#### Tests first

The three existing F6 red tests remain primary. Add:

- every Foundation survivor activity resolves to the locked fallback glyph;
- every station resolves distinct unstaffed and staffed symbols;
- every Foundation resource resolves to its locked symbol;
- all simultaneously visible Foundation categories have unique fallback
  glyphs;
- glyph/style uniqueness remains valid in monochrome mode;
- every station symbol reference resolves through loaded content;
- every resource symbol reference resolves through loaded content;
- a custom station missing either staffing-state symbol is rejected;
- a deliberate Altar/Idle collision is rejected with both content IDs named;
- a deliberate Workshop/Water collision is rejected with both content IDs
  named;
- the player, survivor, station, node, gate, and preview layers resolve in
  deterministic priority order;
- rendering does not hide a survivor through an illegal station/resource
  overlap;
- station staffing changes only the station's semantic state, not its content
  identity or position;
- Help legend records resolve from the same symbols used by map projection.

#### Implementation tasks

1. Extend semantic visual categories for worker activities, station staffing
   states, and the three Foundation resource types.
2. Migrate station data from one raw glyph to explicit unstaffed/staffed symbol
   references.
3. Migrate resource nodes to semantic symbol references.
4. Populate the Foundation lock from Section 22.4.8 in content.
5. Replace separate ad hoc glyph loops with one semantic map-visual
   projection.
6. Make the renderer resolve those visuals through the symbol registry.
7. Add active-category collision validation, including fallback glyphs.
8. Make validation errors identify both categories/content records and source
   locations.
9. Migrate all consumers, then remove the old raw
   `survivor_glyphs`/`station_glyphs`/`resource_glyphs` truth. Do not keep two
   active projection paths.
10. Preserve styles as additional information; never rely on color to repair a
    fallback collision.

#### Required green results

- `altar_and_idle_survivor_remain_distinct_without_color`;
- `workshop_and_water_source_remain_distinct_without_color`;
- `staffed_and_unstaffed_station_have_distinct_ascii_projection`.

#### Focused validation

```bash
cargo test -p bd_app --test phase6_input altar_and_idle_ -- --test-threads=1
cargo test -p bd_app --test phase6_input workshop_and_water_ -- --test-threads=1
cargo test -p bd_app --test phase6_input staffed_and_unstaffed_ -- --test-threads=1
cargo test -p bd_tui visual -- --test-threads=1
cargo test -p bd_data -- --test-threads=1
cargo run -p bd_app -- --validate
```

#### Exit criteria

- All three F6 failures are green.
- The exact Foundation fallback table is data-driven.
- Staffed/unstaffed projection is explicit, not uppercase logic.
- Loader validation catches reintroduced simultaneous collisions.
- The map has one semantic visual projection path.
- Styles and glyphs remain deterministic at 80x24 and 60x20.

#### GDD check

GDD Sections 3, 6, and 8; Kernel semantic ASCII and data-loading boundaries;
D-18 semantic visual clause. Confirm presentation clarifies existing systems
without introducing new content.

### 22.13 Phase F7 — Responsive Help and complete legend

#### Goal

Make the actual rendered Help screen explain the active Foundation visual
language at both supported profiles.

#### Primary production files

- `crates/bd_tui/src/commands.rs`;
- `crates/bd_tui/src/view_models.rs`;
- `crates/bd_tui/src/screens.rs`;
- `crates/bd_tui/src/lib.rs`.

#### Tests first

The existing F7 red test remains primary. Add:

- 80x24 Help contains every active Outpost control and legend entry;
- 60x20 Help contains every active Outpost control and legend entry;
- no required Help line ends mid-word;
- no Help content writes over its border/footer;
- body and footer advertise the same close control;
- changing a command binding changes Help without editing layout text;
- changing a semantic symbol changes Help and map projection together;
- Outpost Help omits deferred raid/event/sanity categories;
- Tactical Help does not inherit irrelevant Outpost station/resource entries;
- opening and closing Help leaves no stale cells;
- Help is deterministic across repeated renders and resize round-trips.

The required legend must include at least the player, worker activity states,
five station state pairs, Trees, Water Source, Wild Plants, gate, valid/invalid
preview, and off-screen target indicator when active.

#### Implementation tasks

1. Build a typed Help view model with separate control and legend sections.
2. Derive controls from `CommandBindings`.
3. Derive legend entries from the active symbol/station/resource catalogs.
4. Give Help the full inner screen instead of normal gameplay side panels.
5. Add responsive grouping/columns for 80x24 and 60x20.
6. Wrap only at word boundaries.
7. Keep one close command in body and footer.
8. Render from the typed sections without copied Foundation legend strings.

#### Required green result

- `rendered_outpost_help_contains_every_foundation_legend_at_supported_profiles`.

#### Focused validation

```bash
cargo test -p bd_tui --lib rendered_outpost_help_contains_every_foundation_legend_at_supported_profiles -- --exact --test-threads=1
cargo test -p bd_tui --test input_help -- --test-threads=1
cargo test -p bd_tui --lib help -- --test-threads=1
```

#### Exit criteria

- The final Ratatui buffer contains every required entry at both profiles.
- Help and map projection consume the same visual data.
- No required entry is clipped or truncated mid-word.
- Deferred systems are absent.
- Opening/closing/resizing Help leaves no stale output.

#### GDD check

GDD Section 3 clarity and deliberate play, Section 6 current controls, Section
8 Foundation scope, D-16 compact support, and D-18 legend clause.

### 22.14 Phase F8 — Persistence, fingerprint, registry, and integrated paths

#### Goal

Prove the repaired systems compose correctly and survive persistence without
making transient UI state durable.

#### Primary files

- `crates/bd_test_support/src/lib.rs`;
- `crates/bd_app/tests/persistence.rs`;
- `crates/bd_app/tests/foundation_scenario.rs`;
- `crates/bd_app/tests/survivor_work_contract.rs`;
- `crates/bd_app/tests/phase6_input.rs`;
- `testing/foundation-contracts.ron`;
- `testing/FOUNDATION-REQUIREMENT-MAP.md`;
- `testing/FOUNDATION-TEST-EVIDENCE.md`;
- `testing/VISUAL-ACCEPTANCE-MATRIX.md`.

#### Tests first

Add or complete:

- normalized fingerprint contains survivor task, position, derived activity,
  stable target, station staffing, node depletion, and catalog identity;
- fingerprint excludes raw entity bits and transient modal/build cursor state;
- save/load while a worker is EnRoute restores the same derived activity and
  next step;
- save/load while a worker is Working restores the same next daily
  contribution;
- save/load while a worker was Blocked restores the same derived reason
  without duplicate movement/production;
- save/load closes or safely reconstructs transient management/build
  interaction according to the existing persistence boundary;
- build success followed by staffing, movement, day production, save/load, and
  forecast remains equivalent;
- colony repairs do not alter fixed-dungeon exploration, combat, loot,
  extraction, or return state;
- repeated load does not duplicate worker activity, station assignment,
  resource output, logs, or view-model entries.

#### Implementation tasks

1. Complete the normalized Foundation fingerprint if any field remains absent.
2. Keep assignment and station relationships durable.
3. Recompute WorkerActivity after restore without advancing a frame of
   gameplay.
4. Exclude or explicitly close transient management/build interaction on load.
5. Compare pre-save and post-load state by stable fingerprint.
6. Run the canonical colony workflow:

   ```text
   inspect → Build → staff → assign gathering → Wait
   → observe EnRoute/Working → Rest → verify production
   → forecast → save → load → verify next step
   ```

7. Run the canonical dungeon workflow after the colony workflow:

   ```text
   enter → explore → fight → loot → extract → return
   ```

8. Update a contract from `Red` to `GreenUnreviewed` only after its primary,
   support, regression, and required profile tests pass.
9. Update requirement-map diagnostics with the actual implementation evidence.
10. Do not mark visual rows Accepted until their semantic, canvas, style,
    geometry, transition, and PTY columns satisfy Section 5.

#### Focused validation

```bash
cargo test -p bd_test_support -- --test-threads=1
cargo test -p bd_app --test persistence -- --test-threads=1
cargo test -p bd_app --test survivor_work_contract -- --test-threads=1
cargo test -p bd_app --test phase6_input -- --test-threads=1
cargo test -p bd_app --test foundation_scenario -- --test-threads=1
cargo test -p bd_test_support --test contract_registry -- --test-threads=1
```

#### Exit criteria

- All 23 baseline failures are green.
- Registry status matches passing evidence without premature acceptance.
- Save/load preserves durable truth and recomputes derived activity.
- Transient UI state cannot cause a duplicate or half-resolved action.
- Colony changes do not regress the dungeon MVP loop.
- Requirement and evidence documents name residual gaps honestly.

#### GDD check

GDD Sections 6–9, MVP scenario persistence and session boundaries, D-09,
D-10, D-18, and D-19. Confirm the repaired colony remains the Foundation slice
and the dungeon loop remains intact.

### 22.15 Phase F9 — Full validation and discovery/playtest gate

#### Goal

Verify the repaired build as a player-facing MVP foundation. Automated green
tests are necessary but not sufficient.

#### Automated gate

Run from the nested Rust workspace:

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo test --workspace -- --list
cargo test -p bd_app --test survivor_work_contract -- --test-threads=1
cargo test -p bd_app --test phase6_input -- --test-threads=1
cargo test -p bd_tui --lib -- --test-threads=1
cargo test -p bd_app --test colony_day_cycle -- --test-threads=1
cargo test -p bd_app --test persistence -- --test-threads=1
cargo test -p bd_app --test foundation_scenario -- --test-threads=1
cargo test -p bd_app --test colony_spatial_contract -- --test-threads=1
cargo test -p bd_test_support --test contract_registry -- --test-threads=1
cargo run -p bd_app -- --validate
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
```

Report:

- listed;
- passed;
- failed;
- ignored;
- doctest result;
- every ignored test name and its documented reason.

No active required test may fail or be newly ignored.

#### Real-terminal discovery/playtest

Use a clean isolated save/config root and the actual `bd` launcher at 80x24,
then repeat compact-critical steps at 60x20.

1. Launch a new game and verify the title and Outpost are well formed.
2. Open Help and verify every visible shelter glyph can be identified.
3. Confirm player, three survivors, gate, and all resource categories are
   visible or discoverable.
4. Open `c`, select a named survivor and gathering task, navigate, cancel, and
   confirm; verify time remains paused and input does not leak.
5. Open `e`, select a named survivor and built station, cancel, and confirm;
   verify the station list contains no gathering tasks.
6. Open Build, inspect full selected cost/effect, enter placement, and verify
   the first preview is adjacent and matches Enter.
7. Confirm an invalid tile; verify the reason, cursor, selection, player,
   resources, time, and station count remain unchanged.
8. Build one valid productive station and verify exactly one spend/time/result.
9. Assign one worker and observe Idle/EnRoute/Working as separate visible
   states.
10. Place or use a known blocker and verify deterministic routing.
11. Observe two assigned workers and verify no stacking or target occupancy.
12. Create an unreachable case and verify one specific Blocked message/state.
13. Verify EnRoute and Blocked workers produce zero at a day boundary.
14. Verify an adjacent Working station worker and matching gatherer produce
    once.
15. Verify forecast excludes non-working output and matches the resulting day
    transaction.
16. Compare Rest with equivalent Wait turns for final worker position and
    daily resources.
17. Save and load during EnRoute; verify no immediate step/output and the next
    step is unchanged.
18. Enter the fixed dungeon, explore, fight, loot, extract, and return.
19. Confirm repaired colony state and extracted loot survive the round trip.
20. Resize 60x20→80x24→60x20; verify Help, management, Build, map symbols,
    footer controls, and decisive feedback remain complete.
21. Quit and verify terminal/cursor restoration.

Record exact observations in `testing/VISUAL-ACCEPTANCE-MATRIX.md` and the F9
completion record.

#### Failure handling

If the playtest contradicts an automated claim:

1. do not mark F9 complete;
2. identify the smallest missing evidence layer;
3. add one atomic red test reproducing the player-visible defect;
4. assign it to F1–F7 by root cause;
5. update Section 22.3 and the requirement map;
6. return to that phase;
7. rerun F8 and the entire F9 gate afterward.

Do not patch a playtest defect directly without a reproducing test.

#### Final GDD drift audit

Read the complete GDD, then explicitly confirm:

- the game still presents survival, preparation, pressure, and consequence;
- the repaired shelter supports building, resources, assignments, stations,
  production, and extracted loot;
- the main dungeon loop remains enter, explore, fight, loot, extract, return;
- skills/virtue hooks remain intact but no deferred full virtue mapping was
  added;
- raids, colony events, sanity, final factions, procgen, and Product P2/P3
  remain inactive;
- no UX shortcut changes the product scope or invents new balance.

#### Exit criteria

- All automated gates pass.
- Both supported terminal profiles pass the player path.
- All 23 original failures have direct passing evidence.
- No previously green required contract regressed.
- No unexplained visual or behavioral deviation remains.
- Registry, requirement map, evidence ledger, visual matrix, MVP scenario, and
  this plan report the same status.
- Only then may this failure-remediation program be recorded complete.

### 22.16 Failure-to-phase traceability

| Phase | Original failures expected green at exit | Broader regression owner |
|---|---:|---|
| F0 | 0; baseline only | contract registry and green guards |
| F1 | supporting schedule/activity tests | architecture, time, save/load |
| F2 | 7 movement failures | pathfinding, occupancy, Rest equivalence |
| F3 | 6 production/forecast failures; all 13 worker failures green | day cycle, economy, zero-Supplies recovery |
| F4 | 3 management failures | input ordering, pause semantics, stable identity |
| F5 | 3 build failures | egress, atomic action, compact detail |
| F6 | 3 semantic ASCII failures | content validation, layering, monochrome |
| F7 | 1 Help failure | input Help, both terminal profiles |
| F8 | all 23 remain green | persistence, colony workflow, dungeon workflow |
| F9 | all 23 plus full suite and PTY remain green | complete Foundation acceptance |

### 22.17 Remediation completion checklist

- [x] F0 confirms the exact baseline and failure ownership.
- [x] One typed time plan owns elapsed turns and Outpost worker steps.
- [x] One derived activity resolver owns Idle/EnRoute/Working/Blocked state.
- [x] Rest replays the same worker steps as individual Outpost turns.
- [x] Tactical turns never move colony workers.
- [x] Existing A* routes workers to adjacent work tiles.
- [x] Player, fixtures, survivors, and reservations are respected.
- [x] Stable survivor order prevents nondeterministic stacking.
- [x] Blocked workers expose a specific reason without log spam.
- [x] One physical-work evaluator serves production, gathering, and forecast.
- [x] EnRoute, Blocked, wrong-node, and depleted-node workers produce zero.
- [x] Valid Working station workers and gatherers produce exactly once.
- [x] Zero-Supplies recovery remains reachable.
- [x] Task management exposes tasks only.
- [x] Station staffing exposes eligible built stations only.
- [x] Management cancellation is atomic under same-batch input.
- [x] Build uses one transaction state.
- [x] Visible preview and submitted build target agree.
- [x] Invalid build remains correctable and atomic.
- [x] Placement preserves selected catalog detail.
- [x] Foundation ASCII symbols match the locked data table.
- [x] Staffed and unstaffed stations are distinct without color.
- [x] Active-category collisions are rejected during content validation.
- [x] Help derives controls and legend from authoritative registries.
- [x] Help is complete at 80x24 and 60x20.
- [x] Save/load preserves durable state and recomputes derived state.
- [x] Normalized fingerprints use stable identity and include repaired colony
      state.
- [x] All 23 original failures pass.
- [x] Full workspace, doctests, formatting, strict Clippy, validation,
      registry, and diff checks pass.
- [x] Real 80x24 and 60x20 discovery/playtests pass.
- [x] Complete GDD audit finds no Foundation scope drift.

### 22.18 F0–F9 completion record — 2026-07-26

Scope:

- Closed the 23 recorded Foundation failures through shared time, activity,
  physical-work, management, build, semantic-visual, Help, and persistence
  authorities.
- Added stable, entity-independent fingerprints and integrated save/load
  continuity contracts.
- Kept broader visual snapshot rows `Green unreviewed` or `Open`; remediation
  completion does not bulk-accept the separate visual-testing program.

Red evidence:

- The original baseline was 542 listed, 517 passed, 23 failed, and 2 ignored.
- Additional red-first contracts exposed and then closed physical Bed
  recovery, transient restore state, compact Help completeness, loose dungeon
  item visibility, denied build resolution, one build authority, one semantic
  map projection, and one physical-work evaluator.
- Integrated persistence testing exposed a real same-frame schedule defect:
  combat RNG could advance before its emitted pool delta was applied.

Implementation:

- `TimeAdvancePlan` owns elapsed turns, Outpost worker steps, and cause.
- Stable-name ordered A* movement derives `WorkerActivity`, reserves
  destinations, and replays Rest one logical turn at a time.
- `evaluate_physical_work` is the non-mutating station/gathering authority for
  activity, forecast, production, gathering, summaries, and worker recovery.
- One `BuildInteraction` enum owns Selecting, Placing, AwaitingResolution, and
  Inactive states; denial returns to the same correctable cursor.
- One semantic `MapVisualVm` list feeds the renderer. Worker/resource glyphs
  resolve from the symbol registry; station staffing glyphs resolve from the
  station catalog; Help consumes those same authorities.
- Save/load restores durable relationships and positions, clears transient
  interaction state, and recomputes activity without movement, output, or
  duplicate feedback.
- Explicit mutation sub-sets order action effects before pool-delta
  application, closing the combat checkpoint inconsistency.

Focused validation:

- `survivor_work_contract`: 24 passed.
- `phase6_input`: 23 passed.
- `test_harness_contract`: 4 passed.
- `persistence`: 13 passed.
- `foundation_scenario`: 16 passed.
- `bd_tui --lib`: 59 passed.
- `input_help`: 22 passed.
- `contract_registry`: 23 passed.

Full regression:

- 556 tests listed; 554 active passed; 0 failed; 2 ignored.
- Workspace doctests passed.
- Formatting, workspace check, strict all-target Clippy, content validation,
  contract-registry validation, and `git diff --check` passed.
- Ignored diagnostic: `integration_diagnostic::diagnose_title_to_outpost_state`
  is a non-gating diagnostic snapshot.
- Ignored legacy terminal test:
  `legacy_terminal_first_keypress_in_outpost_is_move_not_build` requires a
  real terminal and is covered by the PTY gate.

Manual evidence:

- The actual `bd` launcher passed 80x24 and 60x20 title, Outpost, complete
  Help, Build selection/placement/success, task management, station staffing,
  worker EnRoute movement/edge indication, dungeon entry, visible loot,
  movement, combat, and clean terminal restoration checks.
- The earlier complete 80x24 run also passed pickup/use, extraction, return,
  and save/load continuity; automated canonical and persistence paths cover
  those outcomes at both profiles where state rather than terminal geometry
  is authoritative.

GDD review:

- Read the complete current GDD. The repaired game retains survival,
  preparation, pressure, consequence, a thin physical shelter, and the fixed
  enter/explore/fight/loot/extract/return dungeon loop.
- Skill growth and representative virtue expression remain intact.
- Procgen, raids, colony events, sanity, full overworld, reputation, final
  factions, and deeper theology mechanics remain deferred.
- Drift found: none.

Residual risk:

- Separate visual snapshot/style/transition rows remain explicitly open in
  `testing/VISUAL-ACCEPTANCE-MATRIX.md`; they are future test-system work, not
  hidden Foundation remediation failures.

Next phase ready: yes
