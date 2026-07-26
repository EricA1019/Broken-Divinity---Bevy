# Broken Divinity Foundation MVP Correction Plan

**Status:** Complete owner-authorized Foundation correction record

**Created:** 2026-07-25

**Product target:** Recoverable, legible, and honest Foundation MVP

**Current phase:** Phase 8 complete; Foundation MVP accepted

**Depends on:** Completed
[FOUNDATION-STABILIZATION-PLAN.md](FOUNDATION-STABILIZATION-PLAN.md)

**Does not authorize:** Product P2, procgen, overworld expansion, raids, colony
events, sanity, theology-driven mechanics, reputation, final factions, or new
runtime technology

## 1. Summary Judgment

This section records the pre-correction judgment: the Foundation was
technically stable but was not acceptable as a cohesive MVP. Section 22 records
the completed correction and current acceptance evidence.

The clean 2026-07-25 discovery run completed construction, staffing, Rest,
save/load, dungeon entry, combat, item use, extraction, defeat, and restart.
It also exposed failures that the stabilization gate did not model:

- a `DayAdvanced` boundary reached in Tactical mode advances the global day but
  does not run colony consumption, production, gathering, mood consequences,
  or a daily summary;
- non-Supplies station choices can exhaust Supplies while the recovery path
  through Water Source gathering is neither guaranteed by an acceptance
  invariant nor discoverable to a player;
- the build menu does not explain station effects, two offered stations have
  no implemented Foundation effect, and the current Workshop output does not
  match its player-facing identity;
- survivor tasks and station staffing are almost invisible and are changed
  through untargeted nearest-entity shortcuts;
- current-run state is labeled as “Last run,” and a new dungeon erases the
  displayed prior result;
- successful extraction returns the player at the dungeon coordinate rather
  than a named shelter return point;
- controls, logs, grammar, error messages, and long feedback still contradict
  or obscure actual behavior;
- the fixed 8x6 dungeon proves plumbing but does not yet create a meaningful
  exploration or tactical decision.

The prior acceptance status was therefore reopened when this plan was
authorized. Execution corrected the Foundation only; it did not use the
defects as permission to activate Product P2.

## 2. Authority and Reading Order

Implementation agents must read, in order:

1. [README.md](README.md);
2. root [GDD.md](../GDD.md);
3. [DECISIONS-TO-LOCK.md](DECISIONS-TO-LOCK.md);
4. relevant sections of root [Kernel.md](../Kernel.md);
5. [MVP-SCENARIO.md](MVP-SCENARIO.md);
6. this plan;
7. [FOUNDATION-STABILIZATION-PLAN.md](FOUNDATION-STABILIZATION-PLAN.md) as
   completed evidence;
8. current code, tests, and content.

The GDD owns product intent. `Kernel.md` owns architecture. This plan may repair
Foundation behavior but may not silently settle post-MVP design.

## 3. Confirmed Findings

| ID | Severity | Confirmed failure | Evidence |
|---|---|---|---|
| MC-001 | Critical | Tactical day boundaries advance `GameTime` but colony consumers return early outside Outpost | clean run crossed Day 1 to Day 2 in the dungeon; mode guards in production, gathering, and survivor systems |
| MC-002 | Critical | Low-Supplies recovery is opaque and not protected by an executable invariant | staffed Altar branch reached 0 Supplies; travel/build disabled; generic Gathering selected Materials |
| MC-003 | Major | Station choices are not truthful | build UI shows cost only; Bed/Storage have no active effect; Workshop produces Supplies |
| MC-004 | Major | Colony management state and targeting are unreadable | party shows names only; `c` and `e` target implicit nearest entities; station name/output/worker are absent |
| MC-005 | Major | “Last run” is current-session state | entering another dungeon resets `RunOutcome` to `None` and erases the displayed Extracted result |
| MC-006 | Major | Extraction has no explicit shelter return position | player returned at the former dungeon coordinate |
| MC-007 | Moderate | `Z` control is contradictory and functionally redundant | README says Combat screen, footer says Map, Tactical handling requests the already-active combat screen |
| MC-008 | Moderate | Feedback is duplicated or truncated | build/task actions log both generic and specific results; daily and virtue lines lose decisive values |
| MC-009 | Minor | Failure text leaks implementation detail | missing save displays raw operating-system I/O text |
| MC-010 | Product gap | Fixed dungeon is too small and combat too shallow to express the GDD’s intended pressure | 8x6 map, one Rat, one potion, one exit; Rat can die in one default attack |
| MC-011 | Process failure | Acceptance did not include adverse economy choices or cross-mode day boundaries | all prior gates passed while MC-001 and MC-002 remained |

### Confirmed versus not yet proven

MC-002 is not a claim that every zero-Supplies state is mathematically
irrecoverable. Water Source gathering can currently produce Supplies. The
confirmed defect is that recovery depends on hidden spatial/task behavior and
is not guaranteed, targetable, forecastable, or explained. Phase 1 must
determine the complete reachable-state boundary before economy behavior
changes.

## 4. Foundation Scope

### In scope

- one mode-independent, exact-once colony day transaction;
- an explicit and tested Supplies recovery path;
- truthful station definitions, effects, costs, and descriptions;
- legible survivor/task/station management;
- persistent last-completed-run state;
- deterministic shelter return placement;
- coherent controls and single-owner player feedback;
- readable user-facing persistence errors;
- a content-only fixed-dungeon depth pass after behavior is stable;
- persistence and deterministic replay for every changed state;
- correction of acceptance documents and tests.

### Out of scope

- procedural generation;
- full overworld travel;
- raids or colony events;
- sanity;
- theology-driven rules;
- reputation or diplomacy;
- final faction names;
- additional dungeon themes/floors;
- new enemy systems;
- equipment expansion;
- deep crafting, trade, or dismantling;
- Product P2.

## 5. Locked Foundation Decisions

The owner approved C-01 through C-08 on 2026-07-25 by authorizing execution.

### C-01 — Day boundaries are mode-independent

Every emitted `DayAdvanced` value is consumed by one colony daily transaction
exactly once, regardless of whether the player is in Outpost, Tactical, or
Game Over when the boundary occurs.

The transaction mutates colony state in `bd_core`. Presentation may show the
summary immediately or retain it for the shelter, but mode may not suppress
the simulation.

### C-02 — Supplies recovery uses explicit survivor gathering

The Foundation recovery path reuses survivor work rather than adding trading,
free bailouts, or a second economy.

- Gathering receives an explicit resource target.
- The fixed shelter guarantees reachable Trees, Water Source, and Wild Plants.
- Water Source gathering produces Supplies.
- At zero Supplies, assigning all three initial survivors to gather Supplies
  and advancing one day must produce at least the 2 Supplies required for one
  meaningful recovery action.
- The UI shows the selected task and projected next-day resource deltas before
  Rest.

Exact yields remain named data values and may not be duplicated in TUI logic.

### C-03 — One station catalog owns simulation and presentation

Station ID, label, description, cost, staffing rule, and Foundation effect move
to one validated catalog. Build validation, production, save identity, build
menu text, colony inspection, and tests consume that catalog.

The intended Foundation meanings are:

- Stove: staffed Supplies production;
- Altar: staffed Faith production;
- Workshop: staffed Materials production;
- Bed: assigned-survivor mood recovery using the existing named mood bonus;
- Storage: owner decision required before Phase 3.

Storage keeps its type/content but construction is disabled with the visible
reason “No Foundation effect yet.” Real colony-storage capacity, extraction
overflow, and migration semantics remain outside this correction.

### C-04 — Management actions are explicit

`c` opens a colony-management interaction rather than mutating an implicit
nearest survivor immediately. The player selects a survivor and a task.

Station staffing selects both survivor and station. TUI emits typed intents and
does not mutate tasks or assignments directly.

### C-05 — Current and completed run state are separate

`RunSession` continues to own the active run. A separate persisted
`LastCompletedRun` owns the latest Extracted or Defeated result and loot count.
Starting a new dungeon does not erase it.

### C-06 — Foundation transitions have named spawn points

Entering the dungeon uses its entrance. Returning through extraction or
restart uses a named, validated shelter return point. Coordinates never leak
between location scopes.

### C-07 — Remove the redundant `Z` command

There is no separate map/combat presentation to toggle in Foundation. Remove
the no-op command from built-ins, configuration documentation, help, footer,
and README. Inventory already has a semantic Back control.

### C-08 — Content depth comes last

The fixed dungeon remains hand-authored and uses the existing enemy/item
systems. A later content pass may alter layout, placements, and named balance
values but may not add procgen, another floor, a new combat architecture, or a
new content pipeline.

## 6. Architecture Contract

### `bd_core`

Owns:

- `DayAdvanced` and one ordered colony-day transaction;
- colony resource mutation and recoverability rules;
- station catalog runtime identity/effects;
- survivor task and station-assignment validation;
- active and last-completed run state;
- transition spawn selection;
- canonical result messages;
- persistence snapshots and migrations.

### `bd_data`

Owns:

- station content schema/loading/validation if C-03 is approved;
- fixed shelter resource-node requirements;
- fixed dungeon topology and placement validation;
- duplicate IDs, missing references, invalid effects, and unreachable
  placements.

### `bd_tui`

Owns:

- colony-management, build, and inspection view models;
- semantic input and modal navigation;
- 80x24 and 60x20 rendering;
- concise projection of daily/resource/run feedback.

It may not calculate production, change tasks, staff stations, move the player,
or classify run results.

### `bd_app`

Owns:

- validated assembly of content/catalog resources;
- save paths and friendly I/O error classification;
- restored-screen routing;
- terminal lifecycle.

### DRY and encapsulation rules

- one station catalog; no second UI table;
- one daily transaction; no mode-specific duplicate consumers;
- one task/assignment intent path;
- one current run and one last-completed run owner;
- one semantic command catalog;
- one canonical gameplay result message per accepted action;
- forecasts call a pure `bd_core` projection over authoritative definitions;
  TUI does not reimplement formulas.

## 7. Implementation Protocol

Every behavior phase uses strict TDD:

1. add or extend the production-path acceptance test;
2. run it and record the expected red failure;
3. implement the smallest architectural correction;
4. rerun focused tests;
5. run the phase gate;
6. inspect the real terminal when the phase changes player interaction;
7. review GDD Sections 1–10 and C-01 through C-08;
8. record evidence before marking the phase complete.

Tests that directly insert final resources, tasks, station assignments,
location positions, or run outcomes do not count as acceptance proof.

At most one phase may be In progress.

## 8. Phase 0 — Authority, Baseline, and Decision Lock

### Purpose

Make status truthful and prevent implementation from inventing economy or
Storage behavior.

### Documentation work

- mark Foundation acceptance Reopened;
- list this file as a draft correction plan, not Product P2;
- append the discovery correction to the completed stabilization record;
- add C-01 through C-08 to `DECISIONS-TO-LOCK.md` only after owner approval;
- record the Storage choice explicitly;
- capture nested repository HEAD, dirty worktree, full gate, and current
  discovery evidence without overwriting existing work.

### Validation

```text
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo run -p bd_app -- --validate
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
```

### Internal completion proof

- no canonical current-status field claims acceptance without the later
  correction;
- exactly one proposed correction plan exists;
- no Product P2 system appears in scope;
- C-01 through C-08 have an owner disposition;
- baseline commands and unexpected results are recorded.

### GDD review

Sections 2, 3, 6, 8, and 10; D-01, D-08, D-12, D-15, and D-16.

### Phase 0 completion evidence — 2026-07-25

- Owner approval locked C-01 through C-08 and the disabled-Storage policy.
- D-17 records the correction contract without activating Product P2.
- Canonical status fields consistently report acceptance Reopened and name
  this file as the sole active plan.
- Baseline nested repository HEAD is `1ac674f`. The pre-existing dirty
  Foundation worktree was inventoried and preserved; no reset, deletion, or
  replacement occurred.
- Discovery evidence is mapped to MC-001 through MC-011.
- Baseline validation passed:
  - `cargo fmt --all -- --check`;
  - `cargo check --workspace`;
  - `cargo test --workspace`;
  - `cargo run -p bd_app -- --validate`;
  - `cargo clippy --workspace --all-targets -- -D warnings`;
  - `git diff --check`.
- Workspace results include 196 `bd_core`, 44 `bd_tui`, 21 `bd_data`, 20
  `bd_app`, 7 test-support, and all integration suites. The two documented
  legacy/diagnostic ignored tests remain outside acceptance and are replaced
  by neither automated nor manual evidence.
- GDD Sections 2, 3, 6, 8, and 10 plus D-01, D-08, D-12, D-15, D-16, and
  D-17 were reviewed. No deferred system entered scope.

**Phase 0 exit gate:** Pass.

## 9. Phase 1 — Red MVP Correction Harness

### Purpose

Prove the discovery failures through production paths before changing
behavior.

### Red tests

Add `crates/bd_app/tests/mvp_correction.rs` with initially failing tests:

- `tactical_day_boundary_applies_one_colony_transaction`;
- `tactical_day_boundary_survives_save_load_without_replay`;
- `every_legal_day_boundary_has_one_summary`;
- `zero_supply_colony_has_a_discoverable_recovery_path`;
- `every_buildable_station_has_a_visible_implemented_effect`;
- `management_targets_a_named_survivor_and_task`;
- `staffing_targets_a_named_survivor_and_station`;
- `new_dungeon_preserves_last_completed_run`;
- `extraction_uses_shelter_return_spawn`;
- `canonical_feedback_contains_no_duplicate_results`.

Add TUI red tests:

- both terminal profiles show station cost and effect;
- both profiles show survivor task and station worker;
- next-day projection shows consumption, production, and net Supplies;
- daily/virtue feedback preserves decisive values;
- no supported screen advertises `Z`;
- missing/corrupt/incompatible saves use player-facing language.

### Adverse economy matrix

The harness must cover:

- no station;
- each of the five current station choices;
- staffed and unstaffed variants;
- 0, 1, and 2 Supplies;
- one, two, and three gathering survivors;
- Water, Trees, and Wild Plants targets;
- save/load before and after recovery;
- day boundary in Outpost, Tactical, and Game Over.

### Exit gate

- every confirmed defect has a red production-path test;
- current behavior fails for the expected reason;
- no test manufactures the desired result;
- no implementation behavior changes.

### GDD review

Sections 3, 6, 8, and 10; D-10, D-12, D-15, and D-16.

### Phase 1 completion evidence — 2026-07-25

- Added the production-path correction contract in
  `crates/bd_app/tests/mvp_correction.rs`.
- Added read-only scenario observations and setup adapters without adding
  alternate gameplay resolution.
- All ten correction tests compile and fail against current behavior for the
  recorded MC-001 through MC-008 defects.
- Added a failing terminal control test proving the redundant `Z` binding.
- Formatting passes and no production behavior changed in this phase.
- Rechecked GDD Sections 3, 6, 8, and 10 and decisions D-10, D-12, D-15,
  D-16, and C-01 through C-08.

**Phase 1 exit gate:** Pass (expected-red contract established).

## 10. Phase 2 — Mode-Independent Colony Day Transaction

### Purpose

Make one global day mean one colony transaction.

### Implementation sequence

1. Introduce an ordered daily-cycle coordinator in `bd_core`.
2. Consume each `DayAdvanced` exactly once.
3. Remove Outpost-only suppression from production, gathering, and mood
   consequences.
4. Preserve deterministic order:

```text
open summary
  → consume daily Supplies
  → apply staffed station effects
  → apply targeted gathering
  → apply mood/starvation/rest effects
  → finalize one summary
```

5. Keep simulation mode-independent.
6. Retain the latest summary and log it without requiring an Outpost screen.
7. Persist any state needed to prevent replay after loading.

### Required tests

- boundaries in Outpost and Tactical produce identical colony deltas from the
  same snapshot;
- each boundary produces one summary;
- Rest and 24 individual actions remain equivalent;
- extraction/defeat on a boundary does not lose or duplicate the transaction;
- save immediately before/after a Tactical boundary remains exact;
- multiple updates without another boundary do nothing.

### Internal validation

```text
cargo test -p bd_app --test colony_day_cycle -- --test-threads=1
cargo test -p bd_app --test mvp_correction -- --test-threads=1
cargo test -p bd_app --test persistence -- --test-threads=1
```

Then run the common workspace gate.

### Exit gate

No mode can advance the displayed day while silently skipping colony state.

### GDD review

Sections 2, 3, 6, and 8; D-12, D-15, D-16; C-01.

### Phase 2 completion evidence — 2026-07-25

- Removed mode suppression from all three consumers of `DayAdvanced`.
- Preserved the ordered single transaction: consumption/station opening,
  targeted gathering, mood consequences, then one finalized summary.
- Tactical boundary, Tactical save/load, Outpost boundary, Rest equivalence,
  idle non-replay, colony cycle, and persistence tests pass.
- Formatting, workspace check, content validation, strict clippy, and diff
  hygiene pass. Future-phase expected-red tests remain intentionally red.
- Rechecked GDD Sections 2, 3, 6, and 8 and D-12, D-15, D-16, and C-01.

**Phase 2 exit gate:** Pass.

## 11. Phase 3 — Recoverable Economy and Truthful Stations

### Purpose

Eliminate hidden mandatory choices and make every spend honest.

### Implementation sequence

1. Implement the approved single station catalog.
2. Validate stable IDs, labels, costs, effects, staffing rules, descriptions,
   and unsupported effects.
3. Route construction, production, save identity, and view-model projection
   through the catalog.
4. Implement explicit gathering targets and guaranteed reachable fixed-shelter
   node types.
5. Add a pure colony forecast over the same authoritative definitions.
6. Implement the approved Bed behavior.
7. Apply the owner-approved Storage choice.
8. Reject unsupported construction before payment.
9. Bump save/content versions only if the schema requires it; reject older
   development saves readably.

### Economy invariants

- zero Supplies is recoverable without entering a dungeon or constructing a
  station;
- one day with all three starter survivors gathering Supplies reaches at least
  the 2-Supplies action threshold;
- construction can never charge for an unsupported station effect;
- every displayed output equals the actual next-day effect;
- rejected construction/gathering/assignment is atomic;
- no hidden resource owner is introduced.

### Required tests

- every station catalog entry round-trips through content and save identity;
- a sixth station data record validates without a new Rust match branch;
- each implemented effect applies exactly once when staffed;
- unstaffed production is zero;
- Bed mood recovery uses the named mood constant;
- all fixed shelter resource targets are reachable;
- recovery works from 0 Supplies before and after save/load;
- forecast and actual next-day deltas match for the adverse economy matrix.

### Internal validation

```text
cargo test -p bd_data
cargo test -p bd_core colony
cargo test -p bd_app --test colony_day_cycle -- --test-threads=1
cargo test -p bd_app --test mvp_correction -- --test-threads=1
cargo test -p bd_app --test persistence -- --test-threads=1
```

Then run the common workspace gate.

### Exit gate

The player cannot spend scarce Supplies on an unexplained or unimplemented
Foundation choice, and every legal colony state has a tested recovery path.

### GDD review

Sections 3, 6, 8, and 10; D-09, D-12, D-15, D-16; C-02 and C-03.

## 12. Phase 4 — Explicit Colony Management UX

### Purpose

Make survivor work and station staffing inspectable and targetable.

### Player contract

At 80x24 and 60x20, the player can:

- open colony management with `c`;
- see all three survivor names, current tasks, and relevant mood;
- select one survivor;
- select Idle, Gather Supplies, Gather Materials, Gather Plants, Defend, Rest,
  or an implemented station assignment;
- see station name, effect, staffing requirement, and current worker;
- see current resources and forecast next-day deltas;
- confirm or cancel without advancing time;
- receive one specific result after confirmation.

The exact list must hide or visibly disable tasks with no Foundation effect.

### Architecture

- `bd_core` exposes typed availability and forecast projections;
- `bd_tui` owns modal focus/navigation only;
- confirmation emits the existing or extended typed action intent;
- no nearest-entity query decides an accepted management action;
- `e` may open the station-assignment section but may not mutate immediately.

### Red-first tests

- selecting Survivor 2 cannot mutate Survivor 1;
- selecting Altar cannot staff Stove;
- cancel changes nothing and advances no time;
- assignment replacement is explicit;
- unavailable tasks include a reason;
- management state survives save/load;
- 80x24 and 60x20 snapshots contain names, tasks, outputs, forecast, confirm,
  and cancel controls;
- pasted/modal input remains bounded and ordered.

### Manual gate

From a clean shelter:

1. inspect all survivors and stations;
2. assign different gathering targets;
3. build and staff a productive station;
4. verify forecast;
5. Rest;
6. compare forecast with summary;
7. save/reload and inspect the same assignments.

### Exit gate

A first-time player can explain who is doing what, where they are assigned,
what tomorrow costs, and how Supplies can recover without reading logs or
source code.

### GDD review

Sections 3, 6, and 8; D-02, D-12, D-16; C-02 through C-04.

## 13. Phase 5 — Honest Run History and Location Transitions

### Purpose

Separate active-run state from completed history and eliminate coordinate
leakage.

### Implementation sequence

1. Add persisted `LastCompletedRun`.
2. Update it exactly once on extraction or defeat.
3. Do not clear it in `begin_dungeon`.
4. Rename any current-run projection that still exposes `RunSession.outcome`.
5. Add a named shelter return/spawn point to authoritative shelter content.
6. Use it for extraction and defeat restart.
7. Validate all location entry/return points.
8. Preserve one player authority and location-scoped cleanup.

### Required tests

- Extracted remains visible while the next run is active;
- Defeated replaces the previous completed result exactly once;
- active outcome and last-completed outcome survive save/load;
- extraction returns to shelter spawn from every dungeon coordinate;
- defeat restart returns to the same shelter spawn;
- no dungeon coordinate is retained in Outpost;
- no duplicate player or location-owned entity survives a transition.

### Internal validation

```text
cargo test -p bd_app --test entity_scope -- --test-threads=1
cargo test -p bd_app --test foundation_scenario -- --test-threads=1
cargo test -p bd_app --test persistence -- --test-threads=1
cargo test -p bd_app --test mvp_correction -- --test-threads=1
```

Then run the common workspace gate.

### Exit gate

“Last run” always means the latest completed run, and every location
transition lands at a validated location-owned spawn.

### GDD review

Sections 3, 6, and 8; D-10, D-15; C-05 and C-06.

## 14. Phase 6 — Controls, Feedback, and Friendly Failures

### Purpose

Remove contradictory controls and make results readable once.

### Implementation sequence

1. Remove `CombatScreen`/`Z` from the Foundation semantic catalog and docs.
2. Keep `i` as Inventory/Back.
3. Assign one owner for accepted-action result text.
4. Remove generic/specific duplicate build and task logs.
5. Render content labels with correct articles or article-free grammar.
6. Add compact structured daily feedback with before, change, and after
   values.
7. Keep skill and virtue names visible at supported widths.
8. Classify persistence failures into:
   - no save exists;
   - save is corrupt;
   - save version is incompatible;
   - I/O access failed.
9. Keep diagnostic detail in tracing, not player-facing text.

### Required tests

- built-ins, config, help, footer, action panel, and README contain no `Z`;
- each accepted build/task/staff action emits one result;
- “Altar” grammar is correct;
- both terminal profiles show decisive daily and progression values;
- missing/corrupt/incompatible save messages are distinct and readable;
- logs remain chronological after save/load.

### Manual gate

Repeat help, build, management, inventory, save/load failures, daily summary,
combat progression, and extraction at both terminal profiles.

### Exit gate

Every displayed control works in its current context, and every action or
failure produces one understandable player-facing result.

### GDD review

Sections 2, 3, 6, 8, and 9; D-02, D-07, D-16; C-07.

## 15. Phase 7 — Fixed Dungeon MVP Experience Pass

### Purpose

Make the existing fixed slice express exploration, preparation, and tactical
pressure without adding a new system.

### Constraints

- fixed hand-authored dungeon only;
- existing map/content pipeline;
- existing enemy archetype is sufficient;
- no procgen, new floor, dialogue, lore system, sanity, or overworld;
- all balance values live in content or named definitions.

### Content acceptance

The revised dungeon must provide:

- at least one navigation choice rather than a single open room;
- an optional route to the healing item;
- an extraction route that does not automatically collect the item;
- enough enemy placement/health pressure that the canonical fight is not
  always resolved by one default attack;
- readable entrance, explored space, items, enemies, and extraction;
- a viable extract-with-loot branch and a viable item-use branch;
- deterministic fixed-seed results.

The plan deliberately does not prescribe raw dimensions, enemy HP, or damage
numbers. Those values must be selected in content, named, tested, and adjusted
through play evidence.

### Red-first tests

- entrance and extraction remain connected;
- item route is reachable but not on the mandatory shortest path;
- canonical combat requires more than one accepted action for the selected
  fixed seed;
- both item-use and retained-loot branches remain possible;
- defeat remains possible;
- entity count and turn queue remain bounded;
- no procgen resource/plugin enters the Foundation app.

### Manual gate

Three clean runs:

1. cautious item-use extraction;
2. retained-loot extraction;
3. ordinary defeat.

Record turns, damage, item choice, supplies spent, result, and any unclear
feedback. The goal is a meaningful Foundation loop, not content volume.

### Exit gate

The dungeon proves explore, fight, choose, loot/use, and extract/defeat as a
player experience rather than a sequence of adjacent test fixtures.

### GDD review

Sections 2, 3, 6, 8, and 9; D-10, D-11, D-15; C-08.

## 16. Phase 8 — Persistence, Regression, and Final MVP Audit

### Purpose

Prove the corrected Foundation holistically and restore acceptance only with
current evidence.

### Automated gate

```text
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo test -p bd_app --test mvp_correction -- --test-threads=1
cargo test -p bd_app --test foundation_scenario -- --test-threads=1
cargo test -p bd_app --test persistence -- --test-threads=1
cargo test -p bd_app --test colony_day_cycle -- --test-threads=1
cargo test -p bd_app --test stress -- --test-threads=1
cargo test -p bd_tui --test input_help -- --test-threads=1
cargo run -p bd_app -- --validate
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
```

Zero warnings or failures are allowed. Ignored terminal tests do not substitute
for the manual gate.

### Canonical manual scenario

Use clean isolated data at 80x24:

1. launch and inspect three survivors, tasks, resources, station effects, and
   next-day forecast;
2. prove zero-Supplies recovery through explicit gathering;
3. build and staff a productive station;
4. Rest and verify forecast equals one daily summary;
5. save/reload;
6. enter the fixed dungeon and cross a day boundary;
7. verify the colony transaction occurred once;
8. fight, use an item, and observe skill/virtue feedback;
9. complete a retained-loot extraction branch;
10. verify named shelter return spawn and last-completed result;
11. start another run and verify completed history remains;
12. complete defeat, save/load, restart, and one-player validation;
13. verify terminal restoration.

Repeat the full screen/idle/resize audit at 60x20.

### Adverse scenarios

- each station choice;
- zero/one/two Supplies;
- all gathering targets;
- staffed and unstaffed day boundary;
- Outpost/Tactical/Game Over day boundary;
- assignment cancel/replacement;
- construction and travel denial;
- missing/corrupt/incompatible save;
- rapid modal/gameplay input;
- extraction and defeat exactly on a day boundary;
- save/load around every changed transaction;
- terminal draw failure diagnostic seam.

### Final GDD reconciliation

Review GDD Sections 1–10, D-01 through D-16, and approved C-01 through C-08.
Record each Foundation item as Proven, Explicitly deferred, or Failed.

No Failed item may be called complete.

### Final documentation work

Only after every gate passes:

- mark this plan Complete;
- restore `MVP-SCENARIO.md` to Accepted with dated evidence;
- update `docs/README.md`, inventory, migration record, and repository README;
- append final evidence to this plan and the stabilization record;
- keep Product P2 unauthorized.

## 17. Common Phase Gate

After focused tests in every implementation phase:

```text
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo run -p bd_app -- --validate
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
```

Player-facing phases also require a real-terminal test at 80x24 and 60x20.

## 18. Validation Matrix

| Finding | Primary phase | Automated proof | Manual proof |
|---|---|---|---|
| MC-001 | 1–2 | cross-mode exact-once day tests | cross boundary in dungeon |
| MC-002 | 1, 3–4 | adverse economy/recovery matrix | recover from zero Supplies |
| MC-003 | 3–4 | catalog/effect/forecast tests | inspect every station |
| MC-004 | 4 | targeted management tests | assign named survivor/station |
| MC-005 | 5 | active/history persistence tests | second run retains prior result |
| MC-006 | 5 | transition spawn/scope tests | extract from varied coordinates |
| MC-007 | 6 | binding/docs parity tests | displayed controls work |
| MC-008 | 6 | one-result/log snapshot tests | read complete feedback |
| MC-009 | 6 | classified error tests | missing/corrupt save attempts |
| MC-010 | 7 | topology/combat branch tests | three canonical runs |
| MC-011 | 1, 8 | expanded acceptance ownership | complete clean-session audit |

## 19. Protocol Review

### TDD

Every behavior phase starts with a named production-path red test. The final
gate cannot replace phase-local red/green evidence.

### DRY

The plan removes duplicated station facts, daily consumers, task selection,
run-result meaning, command labels, and result messages.

### SRP

- daily transaction mutates colony state;
- station catalog describes stations;
- action resolver validates and emits effects;
- session/history own run semantics;
- transition resolver owns spawn selection;
- view models project;
- TUI renders and emits semantic input;
- application classifies filesystem failures.

### Open/Closed

New station records and fixed content extend validated catalogs rather than
adding widget or command branches.

### Data-driven

Station definitions and dungeon layout/placements are validated content.
Named constants remain for kernel-wide timing and mood rules.

### Encapsulation

No TUI system writes resources, tasks, assignments, run outcomes, positions,
or persistence snapshots.

### Magic numbers

Costs, yields, effects, capacities, and content balance values require named
catalog fields or constants. Tests refer to those names instead of restating
raw values.

## 20. Risks and Controls

| Risk | Control |
|---|---|
| Mode-independent day cycles duplicate effects around transition frames | one coordinator, boundary IDs/day numbers, save-before/after tests |
| Station migration breaks existing saves | stable content IDs, version bump, readable incompatibility, no development compatibility promise |
| Forecast diverges from simulation | pure projection over the same catalog and task definitions |
| Management modal becomes a second simulation | typed intents only; no TUI mutation |
| Economy fix becomes broad crafting/trading work | explicit gathering recovery only |
| Storage semantics balloon scope | owner lock before Phase 3; recommended disable-with-reason |
| Dungeon pass hides system regressions | content phase occurs after behavioral gates |
| New completion claim repeats prior mistake | adverse matrix, cross-mode boundaries, and clean manual audit are mandatory |

## 21. Approval and Stop Conditions

The owner approved C-01 through C-08, selected the recommended disabled-Storage
policy in C-03, and authorized Phase 0 on 2026-07-25.

Stop and ask when:

- a proposed fix requires a deferred system;
- Storage requires extraction-loss rules not approved here;
- a data migration would discard useful saves without a readable rejection;
- station or task semantics conflict with the GDD;
- the fixed-dungeon pass requires a new combat or content system;
- a phase cannot produce its expected red failure;
- canonical documents disagree.

Approval of this plan authorizes correction phases only. It does not authorize
Product P2.

## 22. Completion Evidence — 2026-07-25

### Phase results

| Phase | Result | Validated outcome |
|---|---|---|
| 0 — Authority | Complete | One correction plan, locked C-01–C-08 decisions, Reopened acceptance status, and no Product P2 authorization. |
| 1 — Red harness | Complete | Production-path tests failed for every confirmed correction defect before implementation. |
| 2 — Day transaction | Complete | Outpost and Tactical day boundaries run one ordered colony transaction; save/load cannot skip or replay it. |
| 3 — Economy/stations | Complete | Explicit gathering recovers zero Supplies; all fixed node targets exist and are reachable; one validated station catalog owns costs/effects/labels; Storage is disabled before payment. |
| 4 — Management UX | Complete | Named survivor, task, and station selection require confirmation; cancel is atomic; resources, workers, effects, and forecast are visible at 80x24 and 60x20. |
| 5 — Run/location state | Complete | Active and completed run state are independent and persisted; extraction and defeat restart use the named shelter spawn; completed history survives a subsequent run and restart. |
| 6 — Controls/feedback | Complete | Redundant `Z` is absent; accepted actions have one canonical result; daily/progression values fit supported profiles; persistence failures are classified. |
| 7 — Dungeon pass | Complete | The 12x8 fixed dungeon has a navigation choice, optional healing branch, multi-action combat, retained-loot and item-use extraction branches, and an ordinary defeat path. |
| 8 — Final audit | Complete | Full workspace, focused suites, content validation, strict Clippy, diff hygiene, isolated persistence, and real-terminal scenarios pass. |

Every phase was rechecked against its listed GDD sections and locked
decisions before being marked Complete.

### Final automated gate

The following commands pass with zero failures or warnings:

```text
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo test -p bd_app --test mvp_correction -- --test-threads=1
cargo test -p bd_app --test foundation_scenario -- --test-threads=1
cargo test -p bd_app --test persistence -- --test-threads=1
cargo test -p bd_app --test colony_day_cycle -- --test-threads=1
cargo test -p bd_app --test stress -- --test-threads=1
cargo test -p bd_tui --test input_help -- --test-threads=1
cargo run -p bd_app -- --validate
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
```

The workspace gate includes 19 correction tests, 16 canonical Foundation
scenario tests, 13 persistence tests, 13 colony-cycle tests, 6 stress tests,
22 input/help tests, 196 kernel tests, 45 TUI tests, and the remaining
workspace suites. Two explicitly diagnostic or real-terminal legacy tests
remain ignored; neither substitutes for the manual gate.

### Manual discovery and play evidence

Clean isolated XDG roots were used. No existing user save or configuration
participated.

- At 80x24, the shelter exposed all three survivors, current tasks and mood,
  all resources, all five station choices and effects, the disabled Storage
  reason, staffing, and a next-day forecast.
- Supplies were driven to zero, all three survivors were explicitly assigned
  to gather Supplies, and the next daily transaction recovered to the legal
  action threshold. A staffed Bed applied its named mood effect. Forecast and
  daily summary agreed.
- Save/load preserved resources, survivors, tasks, stations, staffing, mood,
  day state, and chronological feedback.
- A clean Tactical run crossed the day boundary after combat and displayed one
  Day 1 colony summary with Supplies `8→5 (-3)`; no second summary appeared.
- The optional item route supported a potion-use run with visible
  Medicine/Temperance and Melee/Thumos feedback.
- A separate retained-loot run extracted one item, displayed
  `Extracted; loot secured: 1`, stored one item, survived save/load, and kept
  `Extracted` visible when the next dungeon run began.
- At 60x20, the full compact shelter and management controls, all forecast
  deltas, HP/AP, combat, Game Over, save/load controls, and completed-run state
  remained visible. An ordinary defeat saved and loaded successfully, restarted
  at the shelter spawn with one player and full pools, and displayed
  `Run: Defeated`.
- Both processes restored the terminal alternate screen and cursor state on
  exit.

The play audit exposed two late failures that were not waived:

1. the documented `save_dir_override` was loaded but ignored by application
   wiring; a red application test was added and the runtime now resolves the
   configured path, while isolated manual runs use standard XDG variables;
2. direct combat defeat updated the active session but not
   `LastCompletedRun`; a red restart-history assertion was added and defeat
   completion now records both states through one session operation.

Both corrections passed focused tests, the complete gate, and the exact
terminal replay that exposed them.

### Final GDD and decision reconciliation

| Authority | Foundation result |
|---|---|
| GDD 1 — Game statement | Proven at Foundation scale through one persistent shelter, costly dungeon entry, combat, loot, extraction/defeat, and return. Full overworld and sacred-legitimacy systems remain explicitly deferred. |
| GDD 2 — Design pillars | Proven for preparation, pressure, consequence, and inheritance through persistent colony/run state. Theology-driven systems remain deferred without contradictory substitutes. |
| GDD 3 — Core experience | Proven for maintain, prepare, enter, explore, fight, choose item use or retention, extract, return, and stabilize. Overworld traversal and power/law choices are deferred. |
| GDD 4 — World foundations | Explicitly deferred narrative/world depth; no Foundation content changes canon. |
| GDD 5 — Factions/narrative | Proven for exactly two extensible placeholder faction records and typed hostility. Named factions, diplomacy, and investigation are deferred. |
| GDD 6 — Gameplay structure | Proven for tactical combat, a physical shelter, survivors, explicit tasks, stations, production, resources, loot, extraction, and defeat. Procgen, raids, events, sanity, and full overworld are inactive. |
| GDD 7 — Virtues/progression | Proven for the four practical skill lanes and representative Melee/Thumos and Medicine/Temperance behavior. Full balance and mappings remain deferred. |
| GDD 8 — Scope anchors | Proven for every Foundation inclusion and exclusion. |
| GDD 9 — Constraints | Proven: the correction clarifies the existing loop without inventing generic theology mechanics or replacing virtues with conventional attributes. |
| GDD 10 — Open questions | Proven only to the locked Foundation minimum; deeper mappings, final factions, and post-MVP content remain open. |
| D-01–D-03 | Proven: kernel-first Ratatui Foundation with exactly two data-driven placeholder factions. |
| D-04–D-06 | D-05 is proven by skill growth plus virtue expression; sanity and theology-driven mechanics remain explicitly deferred under D-04 and D-06. |
| D-07–D-09 | Proven through canonical documentation ownership, accepted Foundation evidence, and preservation of reusable/deferred code. |
| D-10–D-12 | Proven through the complete fixed dungeon loop and explicit colony management; procgen, raids, and events remain deferred. |
| D-13–D-16 | Proven through four skill lanes, representative virtue hooks, deterministic content/persistence, one colony resource owner, exact entry cost, one daily transaction, and both terminal profiles. |
| C-01–C-08 | Proven by the phase results above. C-08 added fixed-content depth only after behavioral gates passed. |

No Foundation authority item is Failed. “Explicitly deferred” means excluded
from this runtime, not partially complete.

### Final handoff

The corrected Foundation MVP is Accepted. This file is now a completed
execution and evidence record, not an active authorization for further work.
Product P2 and every deferred system still require a separate owner-approved
plan.

## 23. Post-acceptance Build Modal Correction — 2026-07-25

A physical-key discovery run found that enhanced terminal input emitted
Press, Repeat, and Release events while the TUI treated all three as commands.
One physical `B` could therefore open and immediately cancel construction; one
Enter could enter placement and immediately submit it. Buffered gameplay could
also resolve while the construction UI was visible.

The correction was implemented red-first:

- terminal gameplay and modal routing now acts only on `KeyEventKind::Press`;
- `B` changes from normal play to station selection and clears queued gameplay;
- number keys and Up/Down only change the highlighted station;
- Enter explicitly changes selection to placement;
- placement accepts only tile movement, Enter to build, and `B`/Escape to
  cancel;
- neither phase advances time or resolves queued gameplay;
- selection and placement instructions fit both 80x24 and 60x20.

`build_interaction_is_a_paused_press_only_state_machine` proves physical
Press/Release behavior, explicit transitions, paused time, discarded queued
actions, and atomic cancel. The complete workspace suite, content validation,
strict Clippy, formatting, and diff gates pass. A clean 80x24 launch through
the `bd` shell launcher kept the selector open, highlighted Altar, changed to
placement, ignored a gameplay Wait, placed the station east of the player,
deducted exactly 2 Supplies, and restored the terminal on exit.

This is a Foundation UX/input correction consistent with GDD Sections 3, 6,
and 8 and D-02, D-12, and D-16. It changes no product scope and does not
authorize Product P2.
