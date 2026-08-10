# Broken Divinity Foundation Basic Colony Loop Plan

**Status:** C0–C7 implemented and validated; C8 implementation/PTY audit
passed on 2026-07-27, with final visual-contract acceptance still pending
owner review

**Created:** 2026-07-27

**Plan owner:** Project owner

**Execution target:** Coding agents working in small, independently validated
TDD batches

**Product boundary:** Foundation colony building, deterministic shelter resource
placement, physical survivor work, placeholder refining, persistence, and
player-facing Ratatui feedback

**Coordination:** This plan owns the proposed colony-loop task order after its
Phase C0 decisions are locked.
[AUTHORITATIVE-TESTING-STANDARD-AND-MIGRATION-PLAN.md](../authority/AUTHORITATIVE-TESTING-STANDARD-AND-MIGRATION-PLAN.md)
owns test policy, evidence sufficiency, metrics, contract registration, and
test retirement.
[FOUNDATION-TEST-AND-UX-HARDENING-PLAN.md](FOUNDATION-TEST-AND-UX-HARDENING-PLAN.md)
owns the already-locked D-18 movement, occupancy, viewport, management, and
semantic-visual behavior that this plan must preserve.

**Authorization:** The project owner approved every recommended Phase C0
default on 2026-07-27. D-20 records those choices and authorizes the test-first
execution sequence in this plan.

**Never authorizes:** Procedural dungeon or shelter-map topology generation,
overworld generation, raids, colony events, sanity, theology-driven mechanics,
faction reputation, final faction canon, broad balance work, asset-heavy
presentation, or a replacement runtime.

---

## 1. Purpose

The Foundation colony now has useful pieces:

- a physical 40x30 shelter;
- three named survivors;
- paused task and station management;
- deterministic worker movement and pathfinding;
- resource nodes;
- buildable stations;
- physical work adjacency;
- daily gathering and station production;
- colony resources, forecast, summary, and save/load;
- semantic Ratatui projection at 80x24 and 60x20.

Those pieces do not yet form the requested basic colony loop. Build placement
cannot move beyond one tile from the player. Shelter nodes are placed by a weak
map-dimension formula with no authoritative content catalog, spacing,
uniqueness, run-seed variation, or generation failure contract. Gathering and
station production are independent daily-boundary abstractions; a survivor
does not physically gather a raw resource, carry it to a station, and refine
it into a visible result.

This plan defines the smallest coherent vertical slice that closes that gap:

```text
inspect a generated source
  → move a build preview independently of the player
  → place or identify a compatible station
  → assign one named survivor to one named recipe
  → watch the survivor travel to the source
  → gather one raw unit
  → carry it to the station
  → refine it once
  → observe the resulting colony-resource change
  → save, load, and continue the same cycle
```

The goal is a basic playable colony foundation, not a complete production-sim
framework. Tests must protect this player path without prematurely requiring
upgrades, queues, storage logistics, automation priorities, worker skills,
station damage, depletion balance, or Product P2 colony depth.

---

## 2. Authority and Required Reading

Before changing any file for this plan, read:

1. [README.md](../README.md);
2. root [GDD.md](../../GDD.md), especially Shelter and Colony, Current Scope
   Anchors, and Minimum Colony Foundation;
3. [DECISIONS-TO-LOCK.md](../authority/DECISIONS-TO-LOCK.md), especially D-09, D-12,
   D-15, D-16, D-17, D-18, and D-19;
4. root [Kernel.md](../../Kernel.md), especially schedule discipline, content
   loading, stable IDs, semantic ASCII, view-model boundaries, and persistence;
5. [MVP-SCENARIO.md](../authority/MVP-SCENARIO.md);
6. [AUTHORITATIVE-TESTING-STANDARD-AND-MIGRATION-PLAN.md](../authority/AUTHORITATIVE-TESTING-STANDARD-AND-MIGRATION-PLAN.md);
7. [FOUNDATION-TEST-AND-UX-HARDENING-PLAN.md](FOUNDATION-TEST-AND-UX-HARDENING-PLAN.md);
8. [MIGRATION-AND-DEPRECATION.md](../authority/MIGRATION-AND-DEPRECATION.md);
9. `testing/foundation-contracts.ron`;
10. the current implementation and affected tests.

Authority remains:

| Source | Owns |
|---|---|
| GDD | Product intent and player experience |
| Locked decisions | Resolved Foundation behavior and scope |
| Kernel | Technical architecture and ownership boundaries |
| Testing standard | Test quality, evidence, metrics, and migration |
| D-18 hardening plan | Existing colony UX and worker contracts |
| This plan | Colony-loop task order after Phase C0 is locked |
| Contract registry | Machine-readable ownership, never product truth |
| Tests | Evidence only |

If a lower-authority source conflicts with the GDD or a locked decision, stop.
If Phase C0 remains unresolved, stop before production edits. Do not implement
the easiest interpretation and then encode it as a test.

---

## 3. Confirmed Baseline

The implementation audit preceding this plan confirmed:

### 3.1 Reusable behavior

- `BuildInteraction` already owns selection, placement cursor, validation, and
  resolution state.
- Build selection and placement are paused.
- Invalid placement remains correctable and atomic.
- Station placement already checks walkability, permanent occupancy, and
  shelter-gate egress.
- Survivor movement already uses deterministic Outpost worker steps,
  pathfinding, cardinal work tiles, reservations, and typed activities.
- Survivors already avoid station/resource target tiles and one another.
- Rest already reproduces equivalent Outpost worker movement.
- One physical-work evaluator already serves gathering, station production,
  forecast, and activity checks.
- `ColonyResources` already owns Foundation shelter resources.
- Station content is loaded and validated through the Foundation content
  bundle.
- Resource nodes and survivor assignments already persist.
- The test harness already provides stable survivor names, normalized
  fingerprints, replay/trace evidence, and deterministic seeds.

These systems must be extended or migrated, not replaced without evidence.

### 3.2 Confirmed defects and missing behavior

- Each build arrow input resets the cursor to one cardinal tile from the
  player. It does not move from the current cursor.
- Build confirmation reduces the cursor to a direction from the player,
  preventing an absolute distant target.
- Resource-node placement derives its seed only from map dimensions, so every
  normal shelter receives the same layout.
- Node spawning does not guarantee uniqueness, configured spacing, complete
  type coverage, source reachability, or an adjacent work tile.
- Node types, labels, and node-to-resource mappings are partly hardcoded Rust
  branches.
- Resource gathering directly increments colony pools at a day boundary.
- Station production creates output independently from gathered input.
- No survivor cargo, recipe identity, gather/refine stage, or complete
  source-to-station job exists.
- Existing tests strongly cover old adjacency and daily-output rules but do
  not prove the requested physical production cycle.

### 3.3 Existing contract owners that must not be duplicated

The following registry records already own adjacent behavior:

- `INPUT-BUILD-001`;
- `INPUT-BUILD-002`;
- `COLONY-SPATIAL-001`;
- `COLONY-WORKER-TIME-001`;
- `COLONY-WORKER-PATH-001`;
- `COLONY-WORKER-OCCUPANCY-001`;
- `COLONY-GATHER-WORK-001`;
- `COLONY-STATION-WORK-001`;
- `COLONY-FORECAST-001`;
- `COLONY-WORKER-SAVE-001`;
- existing `VISUAL-BUILD-*`, `VISUAL-LANGUAGE-*`, `VISUAL-HELP-*`, and
  viewport contracts.

Before adding a new contract, decide whether the behavior strengthens an
existing owner, is supporting evidence, or is genuinely a separate player
outcome. Duplicate primary owners are a phase failure.

---

## 4. Phase C0 — Product Decisions Locked for Production

The owner approved every recommendation in this section on 2026-07-27. D-20
records the resulting behavior. The decisions below are binding for this
Foundation pass.

### C0-01 — Generation scope

**Locked:**

- shelter terrain/topology remains fixed;
- dungeon topology remains fixed;
- only resource-node coordinates are generated;
- the generation result is deterministic from the run seed and content;
- node positions are generated once per new colony and then persisted.

### C0-02 — Initial resource chains

**Locked content:**

| Source | Raw cargo | Placeholder result | Foundation effect |
|---|---|---|---|
| Trees | Raw Timber | Refined Materials | increases Materials |
| Water Source | Raw Water | Refined Supplies | increases Supplies |
| Wild Plants | Raw Plants | Refined Plants | increases Wild Plants |

The stable content IDs, not these temporary labels, own identity. Faith remains
a separate non-node station effect unless the owner explicitly changes it.

### C0-03 — Compatible station strategy

**Locked:** One data-defined basic processing station with a
guaranteed starter instance. Additional instances may be buildable later
through the same station catalog. This keeps the three placeholder chains
coherent and protects zero-Supplies recovery without duplicating direct
gathering rules.

### C0-04 — Assignment workflow

**Locked workflow:**

```text
e
  → select named survivor
  → select named compatible station
  → select named recipe when more than one is available
  → review source/input/output
  → Enter confirms
  → Escape cancels atomically
```

`c` continues to own non-station survivor tasks. `e` owns station-backed
production jobs. The two menus must remain visually and behaviorally distinct.

### C0-05 — Worker-tick semantics

**Locked:** One accepted Outpost worker tick performs at most one
operation:

- one cardinal movement step;
- one gathering operation; or
- one refining operation.

Arrival changes the worker to a ready/working state. Gathering or refining
uses the next worker tick. Rendering, paused UI, Tactical turns, saving, and
loading produce zero worker ticks. Rest replays the equivalent individual
Outpost ticks in the same schedule order.

### C0-06 — Node count, spacing, and lifecycle

**Locked:**

- Foundation content declares `spawn_count = 1` for each active source;
- the algorithm consumes configured counts instead of hardcoding three;
- minimum node spacing is a named content/configuration value;
- each node is walkable as a target fixture, unoccupied, and has at least one
  reachable cardinal work tile;
- nodes are renewable and non-depleting for this Foundation slice;
- no node regenerates, moves, or changes depletion on load or day advancement.

The exact spacing number must be named in content or a validated placement
profile. It must not appear as a raw literal in resolver logic.

### C0-07 — Cargo, cancellation, and blocked work

**Locked:**

- blocked workers retain cargo;
- a missing source before gathering produces no cargo;
- a missing station after gathering preserves cargo and output remains zero;
- cancellation or reassignment deposits cargo into the sole
  `ColonyResources` raw-resource ledger atomically;
- no raw input is silently destroyed;
- no finished output appears without a completed refine operation.

### C0-08 — Daily transaction and forecast interaction

**Locked:**

- physical gather/refine transitions apply through one exact-once worker
  transaction;
- the day boundary handles food, mood, summaries, and any explicitly retained
  daily station effects;
- a station converted to recipe-driven production cannot also receive legacy
  free daily output;
- forecast consumes the same pure transition rules or clearly separates
  “next worker result” from “next day result”;
- Rest and individual turns produce identical final resources and summaries.

### C0 completion gate

Phase C0 completed on 2026-07-27:

- all eight defaults were owner approved;
- D-20 was added to `DECISIONS-TO-LOCK.md`;
- GDD wording distinguishes colony fixture placement from deferred procedural
  dungeon/map generation;
- `MVP-SCENARIO.md` includes the locked basic colony path;
- this plan is owner approved;
- no code, content, registry status, or executable behavior changed before
  the authority update.

---

## 5. Locked Vocabulary After C0

The D-20 update must define these concepts precisely:

| Term | Required meaning |
|---|---|
| Source definition | Data record describing one harvestable fixture category |
| Resource node | One persisted source instance at a shelter coordinate |
| Raw resource | Recipe input gathered into survivor cargo or colony raw storage |
| Recipe | Data record connecting source, input, compatible station, work, and output |
| Finished resource | Placeholder recipe output owned by `ColonyResources` |
| Durable job | Player-confirmed survivor/station/recipe relationship |
| Production stage | Exact current state of a durable job |
| Worker tick | One authorized Outpost work opportunity |
| Work tile | Reachable walkable tile cardinally adjacent to source or station |
| Generated layout | Complete deterministic node-placement result before ECS mutation |

Do not use “task,” “activity,” “job,” and “stage” interchangeably:

- task/job is durable player intent;
- activity is player-facing current behavior;
- stage is the exact production state-machine position.

---

## 6. Architecture Boundaries

Implementation details may change, but these ownership rules are mandatory.

### 6.1 Content

Foundation content owns:

- source IDs and labels;
- raw and finished resource IDs and labels;
- configured source counts;
- semantic symbol references;
- recipe inputs, outputs, and positive amounts;
- compatible station IDs or validated station category;
- work duration if D-20 permits values other than one;
- placement profile values such as minimum spacing and seed salt.

Content loading must reject:

- duplicate IDs;
- missing source/resource/station references;
- zero or negative amounts;
- invalid configured counts;
- missing labels or semantic symbols;
- a recipe with no compatible active station;
- ambiguous active symbols;
- contradictory or impossible placement-profile values.

### 6.2 Node generation

Use one pure planner:

```text
fixed shelter map
  + run seed
  + validated source definitions
  + occupied/forbidden positions
  + placement profile
  → complete NodePlacementPlan or typed error
```

The planner performs no ECS mutation. A separate resolver commits the complete
validated plan. Generation failure must not leave partial nodes.

Use the existing deterministic RNG and pathfinding infrastructure. Do not add a
roguelike map-generation crate for placing a small number of constrained
fixtures.

### 6.3 Build interaction

`BuildInteraction` remains the sole transient selection/preview/resolution
authority. Cursor movement is a pure relative transition from the current
cursor. Confirmation emits a typed absolute placement request. The domain
resolver revalidates the coordinate immediately before payment and mutation.

The TUI may mutate transient UI interaction state and emit intents. It may not
place a station or deduct resources directly.

### 6.4 Logistics

Use one pure production-stage transition rule. It receives stable snapshots
and returns typed outcomes/deltas. Movement, gathering, refining, forecast,
summary, activity projection, and persistence must not implement competing
versions of the same transition.

Recommended module responsibilities:

| Responsibility | Owner |
|---|---|
| Content types and lookups | Foundation content/catalog |
| Cross-file validation | `bd_data` loader validation |
| Node placement planning | colony node-generation module |
| Station placement validation/payment | station/build resolver |
| Durable job and cargo state | colony logistics/job module |
| Worker path step and reservation | survivor movement module |
| Resource mutation | `ColonyResources` transaction |
| Day boundary and forecast | production/economy module using shared rules |
| Player-visible state | TUI view models and semantic visuals |
| Stable save relationships | save snapshot and restoration layer |

Do not add a public gameplay API solely for tests.

### 6.5 Resource ownership

`ColonyResources` remains the sole Foundation colony-resource owner. Raw cargo
may be survivor state while carried, but cancellation, storage, costs, and
finished output may not create competing global ledgers.

If existing `PoolKind` cannot support data-defined resource identity without
new Rust branches, migrate the colony-resource representation behind stable
accessors. Do not retain one enum ledger and add a second string-keyed ledger
that can disagree.

### 6.6 Persistence

Persistence uses:

- stable content IDs for source, raw resource, recipe, and output;
- save IDs for specific station/node instances;
- explicit stage and cargo;
- node coordinates and depletion state;
- run seed and existing replay origin.

Raw ECS entity bits never appear in normalized acceptance evidence or durable
save relationships.

---

## 7. Authoritative Core Test Inventory

This section defines completion before production implementation. Tests remain
evidence; their product behavior must point to D-20 after C0.

Every primary test must include:

```text
Contract:
Given:
When:
Then:
Must not change:
Evidence layers:
```

Every failure must include the diagnostics in Section 9.

### 7.1 Contract ownership map

| Contract | Ownership action | Primary outcome | Required evidence |
|---|---|---|---|
| `INPUT-BUILD-001` | Preserve | Initial preview and submitted candidate agree | Input state, projection |
| `INPUT-BUILD-002` | Preserve | Invalid confirmation is atomic and correctable | Input state, domain, state diff |
| `INPUT-BUILD-003` | New | Placement cursor moves cumulatively from itself | Input state, state diff |
| `INPUT-BUILD-004` | New | Distant confirmation submits the absolute cursor coordinate | Input state, domain, state diff |
| `COLONY-SPATIAL-001` | Preserve | Accepted build preserves gate egress | Domain, property |
| `COLONY-SPATIAL-002` | New | Accepted production station retains a reachable work tile | Domain, property |
| `VISUAL-BUILD-004` | New | Distant build preview remains visible and complete at both profiles | Projection, buffer, PTY |
| `CONTENT-COLONY-CHAIN-001` | New | Locked Foundation source/recipe chains load by stable ID | Domain |
| `COLONY-WORK-NODE-COVERAGE-001` | New | Generated nodes exactly satisfy configured source counts | Domain, state diff |
| `COLONY-WORK-NODE-SPATIAL-001` | New | Generated nodes are unique, spaced, unoccupied, and workable | Domain, property |
| `COLONY-WORK-NODE-SEED-001` | New | Same seed/content produces the same normalized layout | Domain, state diff |
| `PERSIST-COLONY-NODE-001` | New | Save/load preserves layout without regeneration | Persistence, state diff |
| `INPUT-MGMT-005` | New or strengthen existing owner after audit | Named production assignment is paused and explicit | Input state, workflow, state diff |
| `COLONY-WORKER-TIME-001` | Strengthen supporting matrix only | Existing Outpost/Rest worker-tick equivalence remains | Schedule, state diff |
| `COLONY-GATHER-WORK-001` | Migrate | Physical gather changes cargo/input exactly once, not finished output | Domain, schedule, state diff |
| `COLONY-STATION-WORK-001` | Migrate | Physical refine consumes input and credits output exactly once | Domain, schedule, state diff |
| `COLONY-WORK-CONSERVATION-001` | New | Raw and finished resources obey one conservation rule | Domain, property, state diff |
| `COLONY-WORKER-PATH-001` | Preserve | Reachable routes progress; impossible routes report Blocked | Domain, state diff |
| `COLONY-WORKER-OCCUPANCY-001` | Preserve | Survivors do not stack or occupy source/station tiles | Domain, property |
| `ECON-COLONY-LOGISTICS-001` | New or strengthen `COLONY-FORECAST-001` after audit | Day/Rest/forecast do not duplicate physical output | Domain, schedule, projection |
| `VISUAL-COLONY-WORK-001` | New | Player can identify job, stage, target, cargo, and result | Projection, buffer, PTY |
| `COLONY-WORKER-SAVE-001` | Strengthen | Carrying-stage save/load preserves the next deterministic tick | Persistence, state diff, workflow |
| `COLONY-WORK-CYCLE-001` | New | One production-key path completes one full physical cycle | Workflow, state diff, projection |

`INPUT-MGMT-005` and `ECON-COLONY-LOGISTICS-001` are provisional IDs. Phase C1
must first prove no existing registry owner already owns the exact outcome.

### 7.2 Build contracts

#### `INPUT-BUILD-003` — cumulative cursor movement

**Given:** Outpost placement mode at a named valid starting cursor.

**When:** Send two production `MoveEast` presses with explicit frames.

**Then:** Cursor position equals the initial cursor plus `(2, 0)`.

**Must not change:** Player position, day, turn, worker positions, colony
resources, station count, replay intents, or selected station.

The test must fail against the current reset-to-player behavior.

#### `INPUT-BUILD-004` — absolute placement

**Given:** A selected buildable station and a valid cursor more than one tile
from the player.

**When:** Confirm once through production input.

**Then:** Exactly one station with the selected stable content ID appears at
the exact cursor coordinate; cost and time apply exactly once according to the
locked build contract.

**Must not change:** Player position, unrelated stations, survivor jobs, nodes,
or any second coordinate along the player-to-cursor direction.

#### `COLONY-SPATIAL-002` — workable station footprint

Use a table of shelter candidate positions. Each accepted production station
must retain at least one walkable cardinal tile reachable from the worker
region. Rejected cases change no payment, time, or entity state.

Do not duplicate the existing gate-egress path contract.

### 7.3 Content and node contracts

#### `CONTENT-COLONY-CHAIN-001` — locked content

For each D-20 chain, assert exact stable IDs and valid relationships:

- source exists;
- raw resource exists;
- finished resource exists;
- recipe references the source/input/output;
- compatible station exists;
- amounts and configured counts are positive;
- labels and semantic symbols are nonempty.

Do not assert temporary prose beyond required completeness.

#### `COLONY-WORK-NODE-COVERAGE-001` — configured coverage

**Given:** Valid Foundation content and named run seed.

**When:** Start one new colony.

**Then:** The normalized multiset of node source IDs and counts exactly matches
the enabled source definitions and their configured counts.

**Must not change:** Shelter topology, player spawn, survivor count, gate
position, station count, or mode.

The test must not hardcode the generation algorithm or exact coordinates.

#### `COLONY-WORK-NODE-SPATIAL-001` — layout invariants

Use one table-driven rule across explicit map fixtures:

- normal shelter;
- constrained but valid shelter;
- occupied candidate region.

For every generated node:

- coordinate is in bounds;
- fixture tile accepts a blocking target;
- coordinate is unique;
- coordinate is not in the forbidden set;
- pairwise spacing satisfies the named profile;
- at least one cardinal work tile is reachable.

Expected values come from the fixture contract. Do not copy candidate ordering
or RNG math.

#### `COLONY-WORK-NODE-SEED-001` — determinism

- two independent colonies with the same seed/content must have equal
  normalized layouts;
- a curated different-seed matrix must preserve every invariant;
- the required gate does not demand every seed produce a unique layout;
- no test asserts a golden coordinate unless a future content migration
  explicitly requires coordinate stability.

### 7.4 Production transition contracts

Use one table-driven pure state-transition test. Each row exercises the same
rule:

| Case ID | Current stage | Preconditions | One worker tick |
|---|---|---|---|
| `to-source-step` | En route to source | route length greater than one | one cardinal move |
| `source-arrival` | En route to source | already on work tile | no move; ready to gather |
| `gather-success` | ready to gather | matching active source | exact cargo/input increase |
| `carry-step` | carrying/to station | route length greater than one | one cardinal move |
| `station-arrival` | en route to station | already on work tile | no move; ready to refine |
| `refine-success` | ready to refine | exact recipe input available | consume input; credit output |
| `missing-source` | before gathering | source missing | Blocked; zero cargo/output |
| `missing-station` | carrying | station missing | Blocked; preserve cargo |
| `no-route` | either route stage | target exists but unreachable | Blocked; preserve all resources |
| `no-worker-tick` | any stage | render/Tactical/load | no mutation |

Every row reports stable case ID, seed, stage, target IDs, input, output, cargo,
position, and typed denial.

#### Conservation invariant

For every transition:

```text
raw_before + gathered - consumed
  = raw_after + carried_raw_delta
```

And:

```text
finished_after - finished_before
  = completed_recipe_count × configured_output_amount
```

If no refine transition completed, finished-resource delta is zero.

The expected calculation may live in a test assertion helper only as this
algebraic contract. It must not copy production transition branches.

### 7.5 Schedule contract

Use explicit accepted-action plans:

- one ordinary Outpost turn;
- equivalent individual turns to next day;
- Rest Until Next Day;
- one Tactical turn;
- save;
- load;
- render-only frames.

Assert exact worker ticks, movement/work operations, day boundaries, and
resource transactions. Running additional render frames after resolution must
not repeat a gather/refine result.

The existing `COLONY-WORKER-TIME-001` remains the primary owner of movement
equivalence. New logistics cases support that owner unless Phase C1 proves a
distinct contract is needed.

### 7.6 Presentation contract

At every active production stage, the semantic view model exposes:

- named survivor;
- durable job/recipe label;
- current activity/stage;
- named source or station target;
- carried raw resource and amount when nonzero;
- typed blocked reason when blocked;
- completed finished-resource delta after refinement.

At 80x24 and 60x20:

- current target is visible or has a directional discoverability affordance;
- text is not clipped mid-word;
- source, station, worker, and placement preview remain semantically distinct;
- Help explains active source and worker categories;
- build and assignment controls match the actual input bindings.

Substring presence alone is not sufficient. Tests need semantic projection,
panel geometry, and resolved-cell evidence where applicable.

### 7.7 Canonical colony workflow

The workflow uses production inputs and named checkpoints:

```text
CW-01 clean launch reaches Outpost
CW-02 all configured source categories are discoverable
CW-03 required station is identified or built at a distant cursor
CW-04 named survivor and recipe are confirmed through paused management
CW-05 first accepted Outpost turn begins source travel
CW-06 survivor reaches a cardinal source work tile
CW-07 one gather tick produces exact cargo and no finished output
CW-08 survivor travels toward the compatible station
CW-09 survivor reaches a cardinal station work tile
CW-10 one refine tick consumes cargo and credits exact output once
CW-11 visible colony resources and feedback reflect the transaction
CW-12 save/load preserves the next deterministic cycle step
```

The workflow owns reachability and seams. It does not own:

- content-validation formulas;
- placement algorithm details;
- pathfinding internals;
- exact glyph/color values;
- persistence field serialization;
- balance beyond the named fixture.

On failure, report the last successful checkpoint and the next expected one.

---

## 8. Fixtures

Primary tests use stable named fixtures:

| Fixture | Purpose |
|---|---|
| `colony_build_distant_valid` | Valid distant station coordinate with egress and work tile |
| `colony_build_distant_invalid` | Wall, occupied, egress-blocking, and no-work-tile cases |
| `colony_nodes_foundation_seeded` | Normal fixed shelter and approved D-20 content |
| `colony_nodes_constrained_valid` | Narrow valid region proving planner constraints |
| `colony_nodes_impossible` | No complete placement plan; atomic typed failure |
| `colony_route_single_chain` | One survivor, one source, one compatible station, one recipe |
| `colony_route_blocked_source` | Source exists but no reachable work tile |
| `colony_route_blocked_station` | Gathered cargo with missing/unreachable station |
| `colony_route_two_workers` | Two named survivors and distinct production chains |
| `colony_save_carrying` | Survivor carrying input while en route to station |
| `colony_visual_baseline` | Canonical 80x24 production stage |
| `colony_visual_compact` | Canonical 60x20 production stage |

Rules:

- fixture setup may construct an explicit map/content state;
- primary action must use production inputs/resolvers;
- no fixture discovers “the first” survivor, node, station, or recipe;
- no fixture depends on ECS query order or raw entity bits;
- exact seeds, positions, content IDs, terminal size, and frame counts are
  declared in the fixture;
- fixture helpers do not contain gameplay resolution logic.

---

## 9. Failure Diagnostics

Every new primary helper reports:

```text
contract_id
case_id
fixture_id
seed
terminal_profile
workflow_step
input
frames_advanced
accepted_worker_ticks
survivor_name
job_id
recipe_id
stage_before
stage_after
source_id_and_position
station_id_and_position
cargo_before_after
colony_resources_before_after
expected
actual
normalized_state_diff
trace_tail
replay_tail
visual_crop
```

Fields that do not apply use `n/a`.

Generation failures additionally print:

- placement profile;
- configured source counts;
- complete normalized generated plan or partial candidate diagnostic;
- forbidden coordinates;
- exact violated invariant;
- typed generation error.

Presentation failures additionally print:

- terminal dimensions;
- expected panel rectangle;
- actual panel rectangle;
- semantic tokens;
- resolved glyph/style cells;
- cropped rendered buffer;
- expected and actual target indicators.

Do not emit a generic “assertion failed” for a primary contract.

---

## 10. TDD Execution Phases

### Phase C1 — Authority, inventory, and contract-definition preflight

**Goal:** Freeze the approved D-20 behavior and prove current ownership before
writing production code.

#### Tasks

1. Complete Phase C0 and update authority documents.
2. Run `git status --short`; stop on overlapping unexplained changes.
3. Run the current canonical test gate and record listed/passed/failed/ignored
   counts.
4. Generate the contract report and inspect all existing colony/build owners.
5. Map every proposed contract in Section 7 to:
   - existing primary strengthened;
   - existing supporting evidence;
   - new primary;
   - legacy owner requiring migration.
6. Record current behavior with green characterization tests where a later
   migration changes:
   - daily gathering;
   - free station production;
   - forecast;
   - save snapshots;
   - build direction submission.
7. Finalize fixture IDs, stable seeds, and primary test names.
8. Update the requirement map with `NotImplemented` rows. Do not add invalid
   registry records with missing primary tests.

#### Validation

```bash
cargo test -p bd_test_support contract_registry -- --test-threads=1
cargo run --quiet --locked -p bd_test_support --bin contract_report -- \
  --registry testing/foundation-contracts.ron \
  --listed <listed> --passed <passed> --failed <failed> --ignored <ignored>
bash scripts/test-gate.sh
git diff --check
```

#### GDD drift check

- Basic colony management remains the target.
- Dungeon/map procgen remains deferred.
- No Product P2 system entered scope.
- Zero-Supplies recovery remains reachable.

#### Exit gate

- D-20 is locked.
- Existing contract ownership is understood.
- No duplicate primary owner is planned.
- Baseline failures are explained.
- No production behavior changed.

### Phase C2 — Build cursor and absolute placement

**Goal:** Let the player move a paused build preview independently across the
shelter and place at the exact selected coordinate.

#### Red-first tests

1. Add `INPUT-BUILD-003`.
2. Add `INPUT-BUILD-004`.
3. Add or strengthen `COLONY-SPATIAL-002`.
4. Add `VISUAL-BUILD-004` for both terminal profiles.
5. Run each focused test and record the intended current failure.
6. Add registry records only when their named primary tests exist and compile.

Expected red causes:

- cursor resets to player adjacency;
- absolute target is reduced to a direction;
- distant ghost leaves the player-following viewport;
- station work-tile availability is not part of placement validation.

#### Implementation boundary

- move cursor relative to current cursor;
- clamp only at map boundaries;
- allow preview on invalid cells with typed feedback;
- emit an absolute placement request;
- revalidate absolute coordinate in the domain resolver;
- preserve player position and paused state;
- focus viewport on cursor during placement;
- restore player focus after cancel or resolution;
- preserve egress and add production work-tile validation.

Do not alter station costs, catalog contents, worker jobs, or resource nodes in
this phase.

#### Focused validation

```bash
cargo test -p bd_app --test phase6_input build -- --test-threads=1
cargo test -p bd_app --test colony_spatial_contract -- --test-threads=1
cargo test -p bd_tui build -- --test-threads=1
cargo run -p bd_app -- --validate
git diff --check
```

#### Exit gate

- all C2 tests pass;
- no input leaks from placement mode;
- valid and invalid distant previews are correctable;
- both terminal profiles retain complete selected-station information;
- existing build payment and egress contracts remain green;
- GDD/D-18 drift review passes.

### Phase C3 — Data-defined chains and deterministic node placement

**Goal:** Replace weak node spawning with a complete deterministic placement
plan driven by validated Foundation content.

#### Red-first tests

1. Add `CONTENT-COLONY-CHAIN-001`.
2. Add `COLONY-WORK-NODE-COVERAGE-001`.
3. Add `COLONY-WORK-NODE-SPATIAL-001`.
4. Add `COLONY-WORK-NODE-SEED-001`.
5. Add `PERSIST-COLONY-NODE-001`.
6. Add invalid content support cases for missing references and invalid
   amounts.
7. Observe and record intended failures before production changes.

#### Implementation boundary

- add only the D-20 content types;
- load them through the existing Foundation content bundle;
- validate stable IDs and cross-references;
- create a pure complete-or-error placement planner;
- seed from run seed, placement salt, and stable content identity;
- commit only complete plans;
- attach stable content identity to node instances;
- persist generated coordinates and node state;
- skip generation when persisted colony nodes already exist.

Do not generate shelter terrain, dungeon terrain, stations, survivors, or
events.

#### Focused validation

```bash
cargo test -p bd_app --test content_loading -- --test-threads=1
cargo test -p bd_core colony::resources -- --test-threads=1
cargo test -p bd_app --test colony_node_generation_contract -- --test-threads=1
cargo test -p bd_app --test persistence -- --test-threads=1
cargo run -p bd_app -- --validate
git diff --check
```

#### Exit gate

- configured counts, not Rust branches, own node coverage;
- same seed/content produces identical normalized layout;
- normal and constrained fixtures satisfy every spatial invariant;
- impossible planning is typed and atomic;
- save/load does not regenerate;
- the fixed shelter and dungeon remain fixed;
- existing resource visuals and worker path tests remain green.

### Phase C4 — One complete production chain

**Goal:** Prove the full source-to-station cycle for one named chain before
generalizing.

#### Red-first tests

1. Add the production transition matrix for the selected pilot chain.
2. Migrate `COLONY-GATHER-WORK-001` to cargo/input semantics.
3. Migrate `COLONY-STATION-WORK-001` to recipe/refine semantics.
4. Add `COLONY-WORK-CONSERVATION-001`.
5. Add paused named job-assignment evidence after ownership audit.
6. Strengthen schedule support for no-worker-tick causes.
7. Record expected failures:
   - no durable recipe job;
   - no cargo;
   - gathering credits finished resources directly;
   - station output does not consume input;
   - legacy daily output can duplicate recipe output.

#### Implementation boundary

- add durable job, stable recipe identity, stage, and cargo;
- extend assignment reducer with the locked workflow;
- reuse existing pathfinding, reservations, and cardinal work tiles;
- implement one pure stage transition;
- route exact resource mutations through `ColonyResources`;
- disable only the pilot station’s conflicting legacy free output when the
  recipe system owns it;
- derive player activity from job and physical stage;
- preserve Idle, Resting, Defending, and non-recipe station behavior.

Do not generalize through copied source/station branches.

#### Focused validation

```bash
cargo test -p bd_core colony::logistics -- --test-threads=1
cargo test -p bd_app --test survivor_work_contract -- --test-threads=1
cargo test -p bd_app --test colony_production_route_contract -- --test-threads=1
cargo test -p bd_app --test colony_day_cycle -- --test-threads=1
git diff --check
```

#### Exit gate

- one named survivor completes one named chain;
- gathering changes cargo/input only;
- refining consumes exact input and credits exact output once;
- resource conservation passes;
- render/Tactical/load causes produce no work;
- Rest and individual turns agree;
- no legacy output duplicates the pilot chain;
- zero-Supplies recovery remains possible under the locked D-20 setup.

### Phase C5 — All Foundation chains and multiple survivors

**Goal:** Prove data-driven reuse rather than a one-chain special case.

#### Red-first tests

1. Run the same transition matrix for every D-20 recipe ID.
2. Add a two-survivor/different-chain workflow.
3. Add same-target reservation support if the locked design permits shared
   stations.
4. Add content-order and ECS-order independence cases.
5. Add one fourth-chain extension support test; it is not a new Foundation
   content requirement.

#### Implementation boundary

- generalize catalog lookup and transition data;
- choose targets by stable deterministic ordering;
- reserve work destinations through the existing movement owner;
- ensure one worker cannot receive two job operations in one tick;
- ensure one cargo unit cannot be consumed twice;
- avoid source-specific and station-specific match branches.

#### Focused validation

```bash
cargo test -p bd_app --test colony_production_route_contract -- --test-threads=1
cargo test -p bd_app --test survivor_work_contract -- --test-threads=1
cargo test -p bd_app --test content_loading -- --test-threads=1
cargo test -p bd_app --test stress -- --test-threads=1
git diff --check
```

#### Exit gate

- every configured chain passes the same primary transition rules;
- two workers can make deterministic progress without stacking or duplicate
  credit;
- adding a fixture-only fourth chain requires content, not a new gameplay
  branch;
- existing station and task behavior remains coherent.

### Phase C6 — Presentation, persistence, and canonical workflow

**Goal:** Make the physical colony loop understandable and resumable.

#### Red-first tests

1. Add `VISUAL-COLONY-WORK-001`.
2. Strengthen Help/source-category evidence.
3. Strengthen `COLONY-WORKER-SAVE-001` with the carrying checkpoint.
4. Add `COLONY-WORK-CYCLE-001` with checkpoints CW-01 through CW-12.
5. Add 80x24 and 60x20 presentation cases.

#### Implementation boundary

- project recipe, stage, target, cargo, blocked reason, and result;
- add semantic tokens only where existing ones cannot express the locked
  distinctions;
- keep Help and runtime symbol/input sources shared;
- persist and restore stage, cargo, recipe, and target relationships;
- recompute derived activity after load without advancing work;
- emit one concise completion message;
- keep compact decisive text complete.

#### Focused validation

```bash
cargo test -p bd_tui colony -- --test-threads=1
cargo test -p bd_tui --test input_help -- --test-threads=1
cargo test -p bd_app --test persistence -- --test-threads=1
cargo test -p bd_app --test colony_basic_workflow -- --test-threads=1
cargo run -p bd_app -- --validate
git diff --check
```

#### Exit gate

- player can distinguish all active nodes, stations, worker stages, and
  targets;
- both terminal profiles expose complete decisive information;
- carrying save/load continues with the same next deterministic tick;
- the canonical workflow passes without direct gameplay mutation;
- no unexplained snapshot or visual diff remains.

### Phase C7 — Hardening profile

**Goal:** Harden the green basic loop without turning every edge case into the
first-slice gate.

Add these after C6 is green:

- invalid content-reference matrix;
- impossible-layout atomic generation;
- named 128-seed generation property profile;
- two workers competing for one work tile;
- cancellation/reassignment while carrying;
- output-capacity/full-resource behavior if the owner locks it;
- persistence at every stage;
- repeated Rest and save/load equivalence;
- content-order permutation;
- deterministic replay of the complete colony workflow;
- production-key fuzz sequence with invariant checks;
- no intermittent failure across the repetition profile.

Validation:

```bash
cargo test -p bd_app --test colony_node_generation_contract -- --test-threads=1
cargo test -p bd_app --test colony_production_route_contract -- --test-threads=1
cargo test -p bd_app --test stress -- --test-threads=1
bash scripts/test-gate.sh
git diff --check
```

Exit gate:

- zero invariant failures in the named seed profile;
- zero ignored required tests;
- zero duplicate primary owners;
- zero intermittent failures;
- all new contract reports contain mandatory diagnostics.

### Phase C8 — Real-terminal acceptance and drift review

**Goal:** Prove that a human player can understand and complete the loop.

Use the managed `bd` launcher with isolated save/config roots.

At 80x24 and 60x20:

1. launch cleanly;
2. inspect all source categories;
3. open build mode;
4. move the preview several tiles independently of the player;
5. inspect one invalid reason;
6. place one valid station at the exact cursor;
7. open the locked production-assignment workflow;
8. select a named survivor, station, and recipe;
9. observe source travel;
10. observe gathering and cargo;
11. observe station travel;
12. observe refinement and exact resource feedback;
13. save while carrying on a separate run;
14. load and continue deterministically;
15. use Rest and confirm understandable final state;
16. open Help and verify every active symbol/control;
17. confirm the fixed dungeon loop still launches and returns correctly.

Record:

- terminal profile;
- seed;
- exact key transcript;
- checkpoints;
- observed deviations;
- screenshots/crops required by the visual matrix;
- final resources and survivor states;
- save/load continuation result.

Any player-visible deviation creates a named failure and remediation task. A
green automated suite does not override a failed real-terminal workflow.

---

## 11. Existing Test Migration

### Retain

- build pause/cancellation/atomicity;
- gate egress;
- worker pathfinding;
- work-tile occupancy;
- Rest equivalence;
- wrong-source and blocked-work regressions;
- semantic resource/station distinctions;
- save/load deterministic next worker step;
- daily exact-once transaction;
- zero-Supplies recovery;
- fixed-dungeon isolation.

### Strengthen

- initial build placement with cumulative distant movement;
- station placement with a required work tile;
- resource-node count with configured identity and spatial invariants;
- gathering with cargo and conservation;
- station work with input consumption;
- forecast with physical logistics;
- persistence with stage/cargo/relationships;
- visual activity with job/target/cargo/result.

### Retire only after successor acceptance

- count-range-only node spawn test;
- direct-gather output tests that conflict with D-20;
- assignment-only free station output tests for stations migrated to recipes;
- count-only persistence evidence;
- any broad workflow that duplicates formulas and no longer identifies the
  failing stage.

Retirement requires:

1. named successor contract;
2. passing primary replacement;
3. unchanged or stronger evidence layers;
4. requirement-map and registry update;
5. explicit migration record;
6. no hidden reduction in Foundation scope.

---

## 12. Validation Ladder

Every implementation task runs:

1. the focused red test;
2. the focused test after implementation;
3. neighboring module tests;
4. affected integration test;
5. affected production-key workflow;
6. content validation when content changed;
7. persistence tests when durable state changed;
8. both terminal profiles when presentation changed;
9. `bash scripts/test-gate.sh`;
10. GDD and D-20 drift review.

A phase cannot be declared complete because only its focused tests pass.

Required final automated commands:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo run -p bd_app -- --validate
bash scripts/test-gate.sh
git diff --check
```

Report:

- listed/passed/failed/ignored totals;
- required contract status;
- deferred and legacy profile status separately;
- snapshot/visual diffs;
- repetition/property profile results;
- PTY evidence status.

---

## 13. Risk Register

| Risk | Likely failure | Required control |
|---|---|---|
| Unlocked D-20 behavior | Tests canonize an invented design | Phase C0 stop |
| Procgen scope drift | Fixed shelter/dungeon replaced by map generation | Fixture-placement-only authority |
| Duplicate resource owners | Pools, cargo, and new ledger disagree | `ColonyResources` sole-owner audit |
| Legacy production duplication | Recipe output plus free daily output | Exact-once schedule and conservation |
| Raw entity identity | Save/load or query order breaks jobs | Stable content IDs and save IDs |
| Giant survivor module | Movement, jobs, economy, and UI become coupled | SRP boundaries in Section 6 |
| TUI gameplay mutation | Build or assignment bypasses resolver | Typed intents and domain revalidation |
| Seed brittleness | Golden-coordinate tests block valid improvements | Invariant and equality tests |
| Hidden zero-Supplies deadlock | Required station cannot be obtained | D-20 station/recovery decision and workflow |
| Multi-worker nondeterminism | Stacking or duplicate consumption | Stable ordering, reservations, two-worker test |
| Save version regression | Existing saves fail or duplicate work | Version/migration decision and checkpoint tests |
| Test explosion | Overlapping failures obscure root cause | One primary owner and core/hardening split |
| Technically green, visually inert | Player cannot tell work is happening | Semantic projection plus PTY workflow |

---

## 14. Core Completion Checklist

The basic colony pass is complete only when:

- [x] D-20 is owner approved and reflected in the GDD/decision register.
- [x] Dungeon and shelter topology remain fixed.
- [x] Build cursor moves cumulatively and independently of the player.
- [x] Distant confirmation places at the exact selected coordinate.
- [x] Invalid placement remains typed, visible, atomic, and correctable.
- [x] Accepted production stations preserve gate egress and a work tile.
- [x] Foundation source/recipe chains are data-defined and validated.
- [x] Generated node counts follow content rather than Rust branches.
- [x] Generated nodes are unique, spaced, unoccupied, reachable, and workable.
- [x] Same seed/content yields the same normalized layout.
- [x] Node layouts persist and do not regenerate on load.
- [x] Named paused management creates one stable production job.
- [x] One worker physically reaches a source, gathers, carries, reaches a
      station, and refines.
- [x] Gathering does not directly create finished output.
- [x] Refining consumes exact input and creates exact output once.
- [x] Resource conservation passes for every transition.
- [x] Tactical turns, rendering, saving, and loading produce zero worker work.
- [x] Rest and individual turns agree.
- [x] Multiple survivors do not stack or duplicate resource credit.
- [x] Zero-Supplies recovery remains reachable through the locked physical
      loop.
- [x] Job, stage, target, cargo, blocked reason, and result are visible.
- [x] 80x24 and 60x20 workflows both pass.
- [x] Carrying-stage save/load resumes deterministically.
- [x] The canonical production-key workflow passes.
- [x] Existing dungeon, extraction, progression, and persistence gates remain
      green.
- [x] No required test is ignored, flaky, or owned twice.
- [ ] No unexplained visual diff or pending snapshot remains.
- [x] A real-terminal playtest confirms the loop is understandable.
- [x] Final GDD/D-20 drift review records no deviation.

---

## 15. Phase Completion Record Template

Append one record only after a phase passes:

```text
### Colony Phase Cn completion — YYYY-MM-DD

Authority:
- D-20 clauses:
- GDD sections:
- existing contracts preserved:
- new or migrated contracts:

Red evidence:
- focused test:
- expected failure:
- failure diagnostics:

Implementation:
- production files:
- content files:
- test files:
- documentation/registry files:

Validation:
- focused:
- neighboring:
- workflow:
- content validation:
- persistence:
- terminal profiles:
- canonical gate:
- listed/passed/failed/ignored:

Drift review:
- GDD:
- D-20:
- Kernel:
- deferred systems:

Result:
- complete / blocked
- known remaining failures:
- next authorized phase:
```

Do not mark a phase complete when required evidence is `GreenUnreviewed`,
missing PTY evidence, or contradicted by a discovery playtest.

---

## 16. Execution Record — 2026-07-27

### Colony phases C0–C3

- C0 locked D-20, the fixed-topology boundary, three placeholder chains,
  starter processor, renewable nodes, cancellation conservation, and the
  supported terminal profiles.
- C1 registered the colony contracts and preserved one primary owner per
  requirement.
- C2 delivered a cumulative paused build cursor, absolute placement,
  typed/atomic rejection, cursor-following viewport, egress preservation, and
  reachable station work tiles.
- C3 delivered validated RON source/resource/recipe/placement data,
  complete-or-error deterministic fixture planning, persisted node identity,
  a named 128-seed profile, and content-order-independent placement.

Validation: focused tests, neighboring integration tests, content validation,
strict formatting/lint, and the canonical gate passed.

### Colony phases C4–C6

- C4 delivered one durable source→gather→carry→processor→refine transition
  with exact conservation and no legacy free-production duplication.
- C5 applied the same transition to all three configured chains, a fixture
  fourth chain, and concurrent workers with reservation/occupancy protection.
- C6 delivered the paused `e` survivor→processor→recipe workflow, semantic
  worker/job/cargo projection, guaranteed starter processing, and
  carrying-stage deterministic persistence.

Validation: production-path workflow, persistence, compact/baseline buffers,
and the canonical gate passed.

### Colony phase C7 completion

Hardening covers invalid reference diagnostics, impossible layout atomicity,
128 seeds, content-order permutation, sole-work-tile contention,
cancellation/reassignment with raw-cargo deposit, every persisted job stage,
render/Tactical/load time isolation, deterministic complete-workflow replay,
and a 256-step production-key fuzz profile. Five repeated node/route suite
runs completed with zero intermittent failures.

### Colony phase C8 acceptance record

Runtime seed: `0`. Launcher: `scripts/bd` (the managed launcher target).
Isolated roots: `/tmp/bd-c8-80b-*`, `/tmp/bd-c8-60b-*`, and
`/tmp/bd-c8-60c-*`.

The 80x24 path covered title, clean shelter, Help, six-entry Build selection,
cumulative distant movement, visible non-walkable rejection, exact placement,
paused recipe assignment, EnRoute/Working/cargo/refine projection, exact
Materials credit, carrying save, restart/load, deterministic continuation,
Rest, fixed-dungeon combat, extraction, and colony return.

The 60x20 path covered the same compact-critical states, including all three
wrapped recipe choices, cargo/refinement, save/load turn restoration, Rest,
fixed-dungeon combat, extraction, and colony return.

Final canonical gate: 10/10 steps passed; 601 tests listed, 599 passed,
0 failed, and 2 allowlisted ignored. All 44 required contracts are green with
zero duplicate primary owners; all remain `GreenUnreviewed`.

Discovery found and closed three real defects:

1. Build advertised and accepted only keys `1-5` after a sixth station became
   data-defined. The input range and displayed hint now derive up to entry 9.
2. Runtime Foundation Help crashed at 60x20 because the sixth station glyph
   made one two-column entry too wide. Station legends now split into bounded
   data-driven groups.
3. Wrapped duplicate processor choices could extend under the compact modal
   border. Management height now counts wrapped rows.

Final drift review found no GDD, D-20, or Kernel boundary deviation. Shelter
and dungeon topology remain fixed; raids, events, sanity, broad procgen,
overworld generation, faction reputation, and Product P2 systems remain
inactive.

C8 behavior and PTY evidence are green. Formal phase acceptance remains
pending because applicable registry and visual-matrix records are still
`GreenUnreviewed`; this document does not substitute agent inspection for the
owner review required by D-19.
