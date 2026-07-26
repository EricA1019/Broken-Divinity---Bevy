# Broken Divinity Authoritative Testing Standard and Migration Plan

**Status:** Owner-approved active testing-governance and suite-migration plan

**Created:** 2026-07-26

**Plan owner:** Project owner

**Execution target:** Coding agents working in small, independently validated
batches

**Scope:** Foundation MVP requirement ownership, automated test architecture,
failure diagnostics, player workflows, visual evidence, metrics, and safe
retirement of weak or duplicate tests

**Coordination authority:** This plan owns how Foundation behavior is tested
and when evidence is sufficient.
[FOUNDATION-TEST-AND-UX-HARDENING-PLAN.md](FOUNDATION-TEST-AND-UX-HARDENING-PLAN.md)
continues to own the behavior being repaired and its implementation order.

**Does not authorize:** Product P2, new gameplay, changed balance, procgen in
the Foundation path, full overworld travel, raids, colony events, sanity,
theology-driven mechanics, faction reputation, final faction canon, or a new
runtime.

---

## 1. Purpose

Broken Divinity currently has a large test suite but weak acceptance
authority. Repeated clean-suite claims were later disproved by real play:

- the player and required targets disappear outside a fixed map crop;
- build placement can trap the player;
- worker movement, occupancy, and production disagree;
- management advances time despite a paused-management contract;
- semantic categories collide visually;
- compact output truncates decisive information;
- catalog existence and substring checks have been cited as visibility,
  discoverability, and readability evidence;
- synthetic stress and count-only persistence tests overclaim what they prove.

The goal is not to maximize test count or rewrite every test. The goal is to
create a smaller set of authoritative contracts supported by useful focused
tests, granular player workflows, high-information failures, and generated
metrics.

Foundation is not accepted because most tests pass. It is accepted only when
every required contract has the correct evidence and every required metric in
this plan passes.

---

## 2. Current Baseline

The first execution batch must reproduce rather than trust these observed
facts:

- 473 Rust `#[test]` declarations currently exist across 59 files;
- two tests are ignored;
- eight new TUI acceptance tests intentionally fail;
- therefore the expected current result is 463 passed, 8 failed, and 2
  ignored, not “473 passed”;
- approximately 54 UI assertions use substring presence;
- approximately 54 acceptance/support calls use `expect_action`, which
  currently settles more than one frame;
- stable identity is still violated by `first_survivor`, `first_station`, raw
  entity bits, or ECS iteration order;
- deferred procgen, sanity, overworld, and event tests remain mixed into broad
  workspace runs;
- `testing/README.md` describes an older Python analytics framework but does
  not establish that framework as current Foundation acceptance evidence.

These are audit inputs, not permanent magic numbers. Phase 0 replaces them
with generated inventory results.

---

## 3. Required Reading and Authority

Before executing any task, read:

1. [README.md](README.md);
2. root [GDD.md](../GDD.md), especially Sections 3, 6, 8, and 9;
3. [DECISIONS-TO-LOCK.md](DECISIONS-TO-LOCK.md), especially D-01 through D-18
   and D-19;
4. root [Kernel.md](../Kernel.md), especially testing, schedule, view-model,
   persistence, and data-ownership rules;
5. [MVP-SCENARIO.md](MVP-SCENARIO.md);
6. this plan in full;
7. [FOUNDATION-TEST-AND-UX-HARDENING-PLAN.md](FOUNDATION-TEST-AND-UX-HARDENING-PLAN.md);
8. [MIGRATION-AND-DEPRECATION.md](MIGRATION-AND-DEPRECATION.md);
9. `testing/FOUNDATION-TEST-EVIDENCE.md`;
10. `testing/VISUAL-ACCEPTANCE-MATRIX.md`;
11. current tests and production code.

Authority responsibilities:

| Source | Owns |
|---|---|
| GDD | Product intent and player experience |
| Locked decisions | Resolved Foundation behavior and scope |
| Kernel | Technical architecture and ownership boundaries |
| This plan | Test standard, evidence sufficiency, suite migration, metrics |
| UX hardening plan | Behavior implementation sequence |
| Contract registry | Machine-readable mapping, never new product truth |
| Tests | Evidence only |

If this plan and the UX hardening plan conflict about behavior, stop. If they
conflict only about evidence quality, this plan controls the evidence gate.

---

## 4. Scope Classification

Every test must have exactly one classification:

| Classification | Meaning | Foundation gate |
|---|---|---|
| `FoundationRequired` | Primary evidence for a locked Foundation contract | Yes |
| `FoundationSupport` | Focused support for a required contract | Runs, cannot close alone |
| `Regression` | Protects one named repaired defect | Runs, closes only that defect |
| `FutureProduct` | Product P2/P3 behavior | Separate profile |
| `DeferredInfrastructure` | Preserved non-Foundation infrastructure | Separate profile |
| `Diagnostic` | Investigation/reporting aid | Never acceptance |
| `LegacyPendingRetirement` | Old boundary awaiting replacement | Separate profile |

Rules:

- deferred tests may fail without changing Foundation acceptance status, but
  must remain visible in their own maintained profile;
- no ignored test can be `FoundationRequired`;
- no unclassified test may enter the required gate;
- no test may change classification merely to hide a regression;
- moving a test out of Foundation requires authority references and a
  replacement when it covered an active contract.

---

## 5. Contract Registry

### 5.1 Required artifact

Create:

```text
broken-divinity/testing/foundation-contracts.ron
```

Add a parser and validator in `bd_test_support` using the workspace `serde` and
`ron` versions. Do not add a second metadata format.

### 5.2 Contract record

Each record contains:

```text
id
title
scope
authority_references
player_outcome
primary_test
supporting_tests
evidence_layers
profiles
fixture_id
owner_phase
status
known_failure
```

Allowed status values:

```text
NotImplemented
Red
GreenUnreviewed
Accepted
Deferred
Retired
```

### 5.3 ID namespaces

Use:

- `SHELL-*`;
- `INPUT-*`;
- `COLONY-BUILD-*`;
- `COLONY-MGMT-*`;
- `COLONY-WORK-*`;
- `ECON-*`;
- `DUNGEON-*`;
- `PERSIST-*`;
- `PROGRESSION-*`;
- `CONTENT-*`;
- `VISUAL-*`;
- `RESILIENCE-*`.

IDs are stable after acceptance. Renaming a test does not rename its contract.

### 5.4 Registry invariants

Add tests proving:

- every `FoundationRequired` contract has exactly one primary test;
- every primary test maps to one contract;
- supporting tests map to an existing contract;
- every authority path/reference resolves;
- required evidence layers are non-empty;
- required profile names are valid;
- no required contract is ignored or deferred;
- no retired test remains a primary owner;
- every known failing test has status `Red`;
- every `Accepted` visual contract has completed matrix evidence.

---

## 6. Test Design Standard

### 6.1 One primary behavior

Each contract test has:

1. one named precondition;
2. one player action or domain trigger;
3. one primary outcome;
4. explicit forbidden mutations.

Do not combine unrelated requirements to reduce setup time.

### 6.2 Given/When/Then record

Every primary test documents:

```text
Contract:
Given:
When:
Then:
Must not change:
Evidence layers:
```

This may be a structured helper or concise test comment, but it must be
available in failure output.

### 6.3 Table-driven cases

Use one table when every row exercises the same rule:

- viewport edges;
- terminal profiles;
- station catalog entries;
- resource thresholds;
- visual category pairs;
- save checkpoints.

Each row requires a stable case ID and case-specific failure output.

### 6.4 Workflow tests

Workflow tests validate seams and player reachability. They do not own:

- formulas;
- catalog values;
- pathfinding internals;
- serialization field details;
- exact visual styling.

Those remain in focused contract tests.

### 6.5 Assertion stability

Use exact assertions for:

- resource and pool deltas;
- time;
- typed states;
- action/message counts;
- occupancy;
- relationships;
- persistence fingerprints;
- viewport coordinates.

Use semantic/structural assertions for:

- text completeness;
- controls;
- selected state;
- panel containment;
- category distinction;
- visibility/discoverability.

Use full snapshots only for approved canonical scenes. Never snapshot raw
entity IDs, host paths, full unbounded logs, or nondeterministic ordering.

### 6.6 Prohibited patterns in new primary tests

- `first_survivor` or `first_station`;
- raw entity-bit identity;
- ECS query-order identity;
- direct mutable `World` access;
- hidden frame settling;
- conditional required assertions;
- `output.contains(...)` as sole readability/visibility proof;
- count equality as sole persistence proof;
- sleeps;
- wall-clock timing as functional correctness;
- production logic copied into expected-value helpers;
- test-only gameplay resolvers.

---

## 7. Evidence and Diagnostic Model

### 7.1 Evidence layers

| Layer | Proves |
|---|---|
| Domain | Rule or calculation |
| Schedule | Order and exact-once execution |
| State diff | Authorized and forbidden mutations |
| Projection | Semantic player-visible state |
| Buffer/layout | Glyph, style, geometry, clipping |
| Input state machine | Physical controls and modal behavior |
| Workflow | Complete player journey |
| Persistence | State and continuation equivalence |
| PTY | Real terminal behavior |

A registry record states which layers are mandatory.

### 7.2 Standard failure report

Every primary helper reports:

```text
contract_id
case_id
fixture_id
seed
profile
workflow_step
input
frames_advanced
expected
actual
state_diff
trace_tail
replay_tail
visual_crop
```

Fields not applicable are marked `n/a`; they are not silently omitted.

### 7.3 Normalized state diff

The diff includes stable:

- mode/session phase;
- day/turn;
- player identity, position, pools, inventory;
- survivor name/ID, position, task, target, activity;
- station content ID, position, staffing, effect;
- resource node identity/type, position, depletion;
- colony resources/storage;
- active/completed run state;
- progression and virtues;
- entity counts by scope;
- RNG/replay origin where relevant.

Collections are sorted by stable identity. Raw ECS IDs are excluded.

### 7.4 Workflow transcript

Each player workflow records:

- physical key;
- semantic command;
- interaction state before/after;
- accepted/denied result;
- time change;
- mode change;
- decisive visible feedback.

On failure, print the last successful step and the next expected step.

### 7.5 Visual diagnostics

Visual failures print:

- terminal size;
- panel rectangles;
- viewport world bounds;
- semantic projections;
- relevant buffer crop with coordinates;
- glyph/style/layer/priority;
- allowed and unexpected changed regions.

---

## 8. Foundation Contract Families

### 8.1 Shell and lifecycle

Create primary contracts for:

- New Game reaches Outpost exactly once;
- title Load does not start a new run;
- missing/corrupt save feedback is recoverable;
- quit requests one clean shutdown;
- terminal alternate screen and cursor restore;
- stable idle state does not redraw;
- resize redraws once without stale cells.

### 8.2 Input and controls

- every advertised control executes in its mode;
- unadvertised controls do not mutate gameplay;
- Press/Repeat/Release behavior is explicit;
- queue order is preserved;
- queue overflow is bounded and visible;
- modal input never leaks;
- footer, Help, action panel, configuration, and runtime agree.

### 8.3 Build workflow

Workflow:

```text
B → select → navigate → Enter → place → move preview
→ Enter build or Escape cancel
```

Atomic contracts:

- opening, selection, navigation, and cancellation are paused;
- selection exposes complete cost/effect/availability/staffing;
- placement does not move the player;
- valid and invalid preview states differ semantically;
- invalid preview exposes typed reason;
- rejection changes no payment/time/entity state;
- acceptance spends and advances exactly once;
- accepted placement preserves gate reachability.

### 8.4 Task management and station staffing

Create separate workflows for `c` and `e`.

Contracts:

- distinct mode/title;
- named stable survivor selection;
- named stable station selection;
- paused open/navigation/confirmation/cancellation;
- confirmation changes only intended relationship;
- cancellation is atomic;
- assignment does not immediately move workers;
- feedback identifies survivor, target, and activity.

### 8.5 Viewport and visual language

- player visible at every shelter position for each profile;
- viewport clamps at each edge;
- every layer uses one transform;
- assigned off-screen targets remain discoverable;
- every active semantic category has symbol/style/legend;
- station/resource tokens are distinct;
- staffed/unstaffed and worker activities are distinct;
- player/survivor cannot be hidden by lower gameplay layers;
- ASCII fallback does not depend on color;
- compact decisive text is complete.

### 8.6 Worker movement and production

- Idle does not move;
- assignment becomes EnRoute without movement;
- one Outpost turn permits at most one cardinal step;
- Tactical turns do not move colony workers;
- blockers and reservations are respected;
- survivors do not stack or occupy target tiles;
- valid arrival becomes Working;
- no route becomes Blocked with reason;
- EnRoute/Blocked produce zero;
- correctly Working produces once;
- wrong resource type produces zero;
- Rest equals equivalent individual turns.

### 8.7 Economy

- every day boundary runs one transaction;
- Tactical/Outpost boundaries agree;
- food, station output, gathering, mood, and summary occur once;
- summary equals authoritative delta;
- forecast equals execution;
- zero-Supplies recovery remains reachable and discoverable;
- Storage rejects before payment;
- catalog owns station facts.

### 8.8 Fixed dungeon

Workflow:

```text
Outpost → paid entry → move/explore → encounter → combat
→ loot → exit → extraction → Outpost
```

Contracts:

- entry costs exactly two Supplies;
- denial is atomic;
- fixed content loads without procgen;
- entrance/enemy/loot/exit are reachable;
- movement visibly changes state;
- default enemy survives one default attack;
- enemy phase occurs once;
- invalid attacks are atomic;
- pickup is explicit;
- extraction requires exit and explicit action;
- loot applies once;
- colony state survives;
- defeat grants no loot;
- restart uses shelter return spawn.

### 8.9 Persistence

Required checkpoint cases:

- clean Outpost;
- built station;
- assigned worker;
- EnRoute;
- Working;
- before day boundary;
- after day boundary;
- active dungeon;
- carrying loot;
- extracted;
- Game Over.

For each applicable case:

- fingerprint equality;
- relationship validity;
- projection equality;
- deterministic next action;
- no duplicate costs/results;
- failed load atomicity.

### 8.10 Progression, factions, and content

- Quick Attack improves Melee once;
- Quick Attack expresses Thumos once;
- combat survival expresses Fortitude once;
- item use improves Medicine and expresses Temperance once;
- rejected actions grant nothing;
- six virtues plus Kleos exist;
- two placeholder factions load from data;
- a third valid faction needs no Rust branch;
- hostility uses disposition;
- invalid content reports path and IDs;
- ambiguous active symbols fail validation.

---

## 9. Metrics

### 9.1 Generated report

Add a report command under `bd_test_support` that reads the contract registry
and test evidence. It outputs human-readable text and deterministic JSON.

The report must not infer pass status from source text alone. Test execution
results and snapshot/PTY evidence are explicit inputs.

### 9.2 Acceptance metrics

Foundation acceptance requires:

- 100% required contracts registered;
- 100% required contracts with exactly one primary owner;
- 100% required evidence layers passing;
- zero orphaned active tests;
- zero duplicate primary owners;
- zero ignored required tests;
- zero acceptance tests using unstable identity;
- zero acceptance helpers with hidden updates;
- zero conditional required assertions;
- zero pending snapshots;
- zero unexplained visual diffs;
- every canonical workflow passing at required profiles;
- every advertised control executable;
- every required failure report containing mandatory diagnostics;
- zero intermittent failures in the repetition profile;
- all selected critical mutation probes killed.

### 9.3 Non-acceptance metrics

Record but do not use alone:

- total test count;
- line/branch coverage;
- wall-clock duration;
- snapshot count;
- assertion count.

Runtime budgets are set only after Phase 0 measures a clean baseline on the
project machine. Hardware-dependent wall-clock thresholds do not become
functional correctness tests.

---

## 10. Current Suite Migration Rules

### Retain

- pure pool/combat/pathfinding/relationship/ID tests;
- strengthened schedule tests;
- loader/registry validation;
- atomic save validation;
- input queue and render invalidation tests.

### Strengthen

- `foundation_actions.rs`;
- `colony_day_cycle.rs`;
- `phase6_input.rs`;
- `persistence.rs`;
- `entity_scope.rs`;
- `progression_factions.rs`;
- `input_help.rs`.

### Consolidate after replacements pass

- `foundation_scenario.rs`;
- `foundation_stabilization.rs`;
- `mvp_correction.rs`.

### Move to separate profiles

- procgen;
- sanity;
- overworld;
- colony events;
- deferred narrative/Gabriel;
- wall-clock procgen timing.

### Retire only after successor acceptance

- direct-mutation legacy combat/pickup tests;
- ignored diagnostic snapshot;
- duplicate count-only persistence tests;
- duplicate summary-only determinism tests;
- superseded substring-only tests;
- superseded synthetic stress tests.

Every retirement record names:

```text
old test
old proof
reason insufficient
replacement contract
replacement test
replacement validation result
date
```

---

## 11. Execution Protocol

For every task:

1. inspect the registry row and authority;
2. identify the current primary/supporting tests;
3. write or strengthen the smallest red test;
4. run only that test;
5. capture the intended failure report;
6. implement only the authorized production or harness change;
7. rerun focused tests;
8. run affected workflow;
9. run phase gate;
10. update registry/report;
11. perform GDD drift check;
12. record completion.

Stop when:

- a requirement is absent from authority;
- a test requires a deferred feature;
- two contracts claim the same primary behavior;
- replacing a test would lose unique coverage;
- a red test passes before implementation and the reason is unknown;
- a public production API would exist only for tests;
- diagnostics require duplicating production logic;
- a snapshot change is unexplained.

---

## 12. Phase 0 — Reproducible Inventory and Authority Map

### Tasks

#### 0.1 Generate the real suite inventory

Record:

- tests listed;
- passed/failed/ignored;
- target and module;
- runtime profile;
- classification;
- use of unstable identity;
- direct fixture mutation;
- hidden settling;
- substring/snapshot behavior;
- authority mapping.

Correct every existing report that confuses listed with passed.

#### 0.2 Audit the legacy Python testing framework

Classify every file under `testing/` as:

- active and reproducible;
- useful diagnostic;
- obsolete;
- unverified;
- generated artifact.

Do not install Python dependencies or cite the old framework as evidence
without a clean reproducible run and explicit owner.

#### 0.3 Seed the contract registry

Translate GDD Foundation scope and D-01–D-18 into atomic records. Mark the
eight current visual failures `Red`.

#### 0.4 Create the migration ledger

Every current test receives one disposition:

- retain;
- strengthen;
- consolidate;
- move;
- retire after replacement.

### Validation

```bash
cargo test --workspace -- --list
rg -n "#\\[ignore|first_survivor|first_station|output\\.contains|expect_action" crates
git diff --check
```

### Completion gate

- counts are generated and arithmetically consistent;
- every current test is inventoried;
- every required contract has an initial status;
- no test is deleted.

---

## 13. Phase 1 — Registry Validation and Metrics Report

### Primary files

- `testing/foundation-contracts.ron`;
- `crates/bd_test_support/Cargo.toml`;
- new contract metadata/report modules in `bd_test_support`;
- new `crates/bd_test_support/tests/contract_registry.rs`;
- evidence ledgers.

### Red tests

- duplicate primary owner is rejected;
- missing authority is rejected;
- missing primary test is rejected;
- ignored required test is rejected;
- deferred required test is rejected;
- unknown evidence layer/profile is rejected;
- accepted visual contract without matrix evidence is rejected.

### Completion gate

- registry parser has no gameplay knowledge;
- report is deterministic;
- text and JSON totals agree;
- invalid fixture tests prove every validator branch.

---

## 14. Phase 2 — Harness Truth and Diagnostic Infrastructure

### Tasks

#### 2.1 Explicit frame control

Replace hidden settling with:

- submit input/action;
- advance exactly one frame;
- settle with named predicate and bounded frame count.

#### 2.2 Stable selectors

Select player, survivors, stations, nodes, and content by stable identity.

#### 2.3 Normalized fingerprint and diff

Implement Section 7.3 without raw ECS IDs.

#### 2.4 Workflow transcript

Record physical input through visible result.

#### 2.5 Diagnostic sensitivity

Deliberately alter one field in test fixtures and prove the failure report
identifies that field, contract, and step.

### Completion gate

- no primary test uses hidden updates or query order;
- diagnostics are deterministic;
- no mutable world is exposed;
- no expected-value helper duplicates a game rule.

---

## 15. Phase 3 — Classify and Assign Current Tests

### Tasks

- assign every test to a registry contract/classification;
- identify primary ownership conflicts;
- demote broad tests to supporting evidence where appropriate;
- separate deferred profiles;
- identify exact replacement for every retirement candidate;
- prohibit new uncatalogued tests.

### Completion gate

- zero orphaned tests;
- zero duplicate primary owners;
- no required behavior owned only by legacy/deferred coverage;
- no deletion yet.

---

## 16. Phase 4 — Atomic Domain and Schedule Contracts

Strengthen actions, time, economy, occupancy, progression, transition,
persistence validation, and schedule ordering.

Each contract receives:

- exact input;
- exact result count;
- exact allowed state diff;
- forbidden mutation diff;
- deterministic case ID.

Consolidate only duplicated support tests whose stronger primary replacement is
already green.

---

## 17. Phase 5 — Input and Menu Workflows

Create dedicated production-key targets:

- shell/title lifecycle;
- Build selection/placement;
- task management;
- station staffing;
- Help/inventory;
- save/load feedback.

Each modal tests open, navigate, confirm, cancel, Repeat, Release, and leakage
as discrete cases.

Run at 80x24 and compact-critical 60x20 checkpoints.

---

## 18. Phase 6 — Visual Contract Infrastructure

Implement:

- semantic observations;
- canvas snapshots;
- style-cell snapshots;
- geometry observations;
- transition diffs;
- canonical deterministic fixtures.

Move the eight current red tests out of the broad TUI unit module into named
visual contract targets without weakening them.

No snapshot is accepted until structural and semantic assertions pass.

---

## 19. Phase 7 — Colony Player Paths

Execute the Build, assignment, staffing, worker movement, physical work, Rest,
day transaction, recovery, viewport, egress, and Help contracts.

Validate each atomic contract first, then the canonical colony workflow.

Do not let the workflow own formulas already tested by atomic contracts.

---

## 20. Phase 8 — Dungeon Player Path

Create one canonical production-key workflow with named checkpoints:

1. paid entry;
2. visible arrival;
3. exploration movement;
4. encounter;
5. tactical action;
6. enemy response;
7. defeat hostile;
8. loot detour;
9. pickup;
10. reach exit;
11. explicit extraction;
12. colony result.

Add a separate defeat/restart workflow. Report the last successful checkpoint
on failure.

---

## 21. Phase 9 — Persistence Matrix

Replace scattered count comparisons with Section 8.9 checkpoint cases.

For each case, prove:

- fingerprint equality;
- relationship validity;
- visual equality where applicable;
- deterministic continuation;
- no duplicate side effects.

Retire duplicate persistence tests only after every unique proof is mapped.

---

## 22. Phase 10 — Content, Progression, and Extensibility

Add data-driven matrices for:

- station catalog;
- placeholder factions;
- visual symbol registry;
- fixed dungeon content;
- representative skills/virtues.

Tests must prove extension without adding Rust branches. They must not require
final faction canon or complete virtue balance.

---

## 23. Phase 11 — Adverse, Property, and Stress Profiles

### Required deterministic matrices

- resource minimum/zero/threshold/max;
- every shelter edge;
- every station placement candidate;
- every worker target state;
- every visual category pair;
- every persistence checkpoint boundary.

### Stress

- production colony/dungeon cycles;
- bounded input/message behavior;
- entity scope stability;
- repeated save/load;
- deterministic seeded action sequences.

Wall-clock performance is reported, not used as gameplay correctness.

---

## 24. Phase 12 — Safe Consolidation and Retirement

For every candidate:

1. run old test;
2. run replacement;
3. compare proof;
4. confirm registry ownership;
5. preserve unique regression cases;
6. record retirement;
7. remove or move old test;
8. run full affected profiles.

The suite may become smaller. Contract coverage and diagnostic quality may not
decrease.

---

## 25. Phase 13 — Final Acceptance

### Required automated gates

- registry validation;
- Foundation primary contracts;
- supporting regression suite;
- canonical workflows;
- visual matrix;
- persistence matrix;
- deterministic adverse/property profile;
- content validation;
- formatting, compilation, strict Clippy, and whitespace.

### Required real-terminal gates

At 80x24 and 60x20:

- title/new/load;
- Help;
- Build valid/invalid/cancel;
- task management;
- staffing;
- worker progression;
- Rest/day summary;
- save/load;
- dungeon loop;
- extraction;
- defeat/restart;
- resize and terminal restoration.

### Final GDD review

Review Sections 3, 6, 8, and 9 and every locked decision. Confirm:

- Foundation loop is playable;
- no deferred system became required;
- practical survival remains the focus;
- tests prove player behavior rather than module existence.

### Final completion criteria

This plan is complete only when:

- all Section 9 acceptance metrics pass;
- every required contract is `Accepted`;
- no known visual red test remains;
- no required test is ignored;
- no pending snapshot exists;
- no unexplained test retirement exists;
- all player workflows pass at required profiles;
- real terminal and automated evidence agree;
- documentation, registry, ledgers, and actual test results report the same
  status.

---

## 26. Phase Completion Record

Append:

```text
### Testing Phase N completion — YYYY-MM-DD

Contracts affected:
Tests retained:
Tests strengthened:
Tests added:
Tests moved:
Tests retired:

Red evidence:
Green evidence:
Diagnostic evidence:

Focused commands:
Profile commands:
Result counts:

Metrics before:
Metrics after:

GDD sections reviewed:
Drift:

Residual risks:
Next phase ready: yes/no
```

A phase is not complete when any field is omitted, required tests are red, or
reported metrics do not match actual execution.
