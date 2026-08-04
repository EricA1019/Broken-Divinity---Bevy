# Foundation Test Evidence Ledger

Status: Active evidence ledger
Authorities:

- `../docs/AUTHORITATIVE-TESTING-STANDARD-AND-MIGRATION-PLAN.md`
- `../docs/FOUNDATION-TEST-AND-UX-HARDENING-PLAN.md`

Baseline captured: 2026-07-25

This ledger classifies evidence; test count is not acceptance. A target may be
green while remaining ineligible to close a Foundation contract.

## Baseline

- `cargo fmt --all -- --check`: passed.
- `cargo check --workspace`: passed.
- `cargo test --workspace`: 462 listed: 460 passed, 0 failed, 2 ignored.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo run -p bd_app -- --validate`: passed.
- Root `src/` tests are outside the Cargo workspace and are not active
  evidence.
- Post-Phase 1 suite: 465 listed: 463 passed, 0 failed, 2 ignored.
- Red visual batch added 2026-07-26: 473 listed: 463 passed, 8 failed, 2
  ignored. The eight visual failures represent confirmed UI defects.
- Contract-registry test batch added 2026-07-26: 496 listed: 486 passed, 8
  failed, 2 ignored. The 23 new registry tests pass; the same eight known
  visual contracts remain intentionally red.

## Classification definitions

- **Acceptance:** production-boundary proof of an owner-approved MVP outcome.
- **Contract:** focused proof of an architectural or subsystem invariant.
- **Regression:** useful proof against a previously fixed defect, but
  insufficient by itself for player-facing acceptance.
- **Legacy:** retained historical coverage that does not use current
  production boundaries.
- **Deferred:** covers functionality outside Foundation MVP.
- **Diagnostic:** observation aid that is not a required gate.

## Active integration targets

| Target | Count | Class | Production boundary | Fixture mutation | Player path | Authority covered | Known blind spots | Foundation acceptance |
|---|---:|---|---|---|---|---|---|---|
| `crates/bd_app/tests/colony_day_cycle.rs` | 13 | Contract | Foundation app, typed actions/messages, day transaction | Explicit resource/station preconditions | No | GDD 6; D-17 | No physical worker arrival or visual day-summary proof | Partial |
| `crates/bd_app/tests/content_loading.rs` | 4 | Contract | Content loaders and validation | Temporary RON fixtures | No | D-03; data-driven | Only symbols/themes; no complete active visual-language collision validation | No |
| `crates/bd_app/tests/diagnostic.rs` | 1 ignored | Diagnostic | Application startup | Diagnostic capture | Partial | None | Ignored and non-gating | No |
| `crates/bd_app/tests/entity_scope.rs` | 9 | Contract | Foundation app and transition cleanup | Named setup entities | No | GDD 3/6; D-08 | No repeated production-path leak profile | Partial |
| `crates/bd_app/tests/foundation_actions.rs` | 13 | Contract | Typed `ActionIntent` pipeline | Explicit actor/action preconditions | No | Kernel; GDD 8 | Management timing is owned by the production-input target | Partial |
| `crates/bd_app/tests/foundation_scenario.rs` | 16 | Acceptance | Foundation app and canonical actions | Canonical fixture through support API | Partial | GDD 3/6/8/9; D-08 | No terminal controls; state comparison needs normalized fingerprint | Yes, for covered non-visual rows |
| `crates/bd_app/tests/foundation_stabilization.rs` | 21 | Acceptance | Foundation app, actions, transitions, persistence | Explicit named preconditions | Partial | D-14 through D-17 | No construction egress proof or visual placement proof | Yes, for covered rows |
| `crates/bd_app/tests/legacy_kernel_regressions.rs` | 13, 1 ignored | Legacy/Deferred | Mixed legacy fixture and some actions | Direct mutation and synthetic fixtures | Partial | Historical Kernel | Procgen is deferred; direct mutation bypasses current player path | No |
| `crates/bd_app/tests/mvp_correction.rs` | 19 | Acceptance/Contract | Foundation app, catalogs, actions, save/load | Explicit named preconditions | Partial | D-17; GDD 3/6/9 | Two names overclaim visible/discoverable behavior; no real render evidence | Partial until renamed/strengthened |
| `crates/bd_app/tests/persistence.rs` | 13 | Contract | Production save/restore | Explicit world preconditions and temporary slots | No | D-09; GDD 8 | Several comparisons are narrower than a complete normalized fingerprint | Partial |
| `crates/bd_app/tests/phase6_input.rs` | 23 | Acceptance/Contract | Production keys, modal reducers, build transaction, and semantic projection | Controlled command batches | Yes | D-16/D-18; Kernel | Complete visual snapshot review remains separate | Yes, for covered non-snapshot rows |
| `crates/bd_app/tests/progression_factions.rs` | 11 | Contract | Foundation app and content | Explicit actor/faction preconditions | No | D-03/D-05; GDD 4/5 | Representative only; full virtue/faction depth is deferred | Partial |
| `crates/bd_app/tests/stress.rs` | 7 | Contract/Regression/Deferred | Production Foundation cycles plus synthetic world, procgen, and save APIs | Direct mutation is isolated to explicitly named fixtures | No | General stability | Procgen/timing remain deferred; production scope-cycle test now covers the active leak contract | Partial |
| `crates/bd_app/tests/survivor_work_contract.rs` | 24 | Acceptance/Contract | Production actions, movement schedule, day transaction, and persistence | Named physical preconditions | Partial | THC-01/02/03; GDD 6/8 | Full worker-state visual snapshots remain open | Yes, for physical worker rules |
| `crates/bd_app/tests/test_harness_contract.rs` | 4 | Contract | Production checkpoint/restore plus normalized fingerprint | Named durable preconditions | No | D-09/D-19 | Visual snapshot equality remains open | Yes, for stable fingerprint and derived restore |
| `crates/bd_core/tests/architecture.rs` | 7 | Contract | Core plugin, actual configured sets, and trace resource | Minimal direct ECS setup | No | Kernel schedule | Schedule probe proves required set order; it does not prove every system inside each set | Yes, for required stage ordering |
| `crates/bd_tui/tests/input_help.rs` | 22 | Acceptance/Contract | Production key bindings, commands, footer/help projection | Controlled app/view state | Yes | D-02/D-16/D-18; GDD 8 | Mostly semantic token checks; not full canvas/style/geometry evidence | Partial |

## Unit-test groups

| Group | Count | Class | Boundary | Fixture mutation | Player path | Known blind spots | Foundation acceptance |
|---|---:|---|---|---|---|---|---|
| `bd_app` main/config/application units | 22 | Contract | Startup/config/persistence adapters | Temporary config/save roots | Partial | Does not exercise full terminal lifecycle | Partial |
| `bd_core` units | 199 | Contract/Regression/Deferred | Pure rules and individual systems | Frequent direct ECS/value setup | No | Includes deferred procgen, sanity, overworld, events; cannot prove integrated player outcomes | Partial by named rule only |
| `bd_data` units | 24 | Contract | Loader/registry/validation | Content fixtures | No | Does not prove projected presentation | Partial |
| `bd_test_support` units and registry integration | 37 | Contract | Test harness around Foundation app plus contract-governance boundary | Harness-owned state and RON metadata fixtures | No | Accepted-action result and Tactical enemy frames are explicit; remaining helper boundaries require their own audits | Registry validation and explicit-frame characterization pass |
| `bd_tui` units | 59 | Contract/Regression | Production render/view-model functions and `TestBackend` | Controlled app/render state | Partial | Full style/layer/stale-cell snapshot matrix remains open | Partial |

## Required remediation ledger

| Existing test/name | Confirmed problem | Required disposition | Owning phase | Status |
|---|---|---|---:|---|
| `zero_supply_colony_has_a_discoverable_recovery_path` | Direct resource/task assertions do not prove player-visible discoverability | Renamed to `three_explicit_supplies_assignments_recover_the_action_threshold`; separate visual discoverability proof remains Phase 5 | 1/5 | Phase 1 complete; Phase 5 open |
| `every_buildable_station_has_a_visible_implemented_effect` | Catalog/effect execution does not prove the effect is visible | Renamed to `every_buildable_station_catalog_entry_has_an_implemented_effect`; separate presentation proof remains Phase 9 | 1/9 | Phase 1 complete; Phase 9 open |
| `system_order_matches_declared_schedule` | Compared enum/debug declaration sequence, not executed Bevy schedule | Replaced by a production schedule probe; declaration vocabulary retained under an honest name | 1 | Complete |
| `trace_records_ordered_flow` | Required stage assertions were conditional and could pass when stages were missing | Replaced by exact signal occurrence/order assertions plus missing/duplicate/reordered sensitivity tests | 1 | Complete |
| `hundred_turn_simulation_does_not_leak_entities` | Hand-written spawn/despawn loop did not exercise the game | Renamed as synthetic coverage; added ten production colony/dungeon/colony scope cycles | 1 | Complete |
| `seed_batch_does_not_panic` | Invalid plans were explicitly accepted despite a broad stability name | Renamed and classified as deferred procgen smoke coverage | 1 | Complete |
| `event_queue_does_not_grow_unbounded` | Contained no postcondition | Replaced by exact 1,000-read, no-duplicate, and late-reader-expiry assertions | 1 | Complete |
| TUI functions containing `snapshot` | Most asserted only `output.contains(...)` | Renamed to `contains_required_tokens`; real snapshots remain Phase 4 | 1/4 | Phase 1 complete; Phase 4 open |

## Acceptance rules

- A `Partial` target closes only the exact row evidenced by its named tests.
- Legacy, Deferred, and Diagnostic targets never close Foundation gates.
- Fixture mutation may establish a precondition but may not manufacture the
  claimed outcome.
- Player-path evidence starts from production key events or semantic commands.
- Visual acceptance additionally follows
  `VISUAL-ACCEPTANCE-MATRIX.md`.

## Testing-governance implementation evidence — 2026-07-26

- `testing/foundation-contracts.ron` registers seven granular
  `FoundationRequired` visual contracts covering all eight confirmed visual
  failures. The 80x24 and 60x20 player-visibility cases share one contract
  because they are profile rows for the same rule.
- `bd_test_support::contract_registry` parses only governance metadata and
  contains no gameplay rule or expected-value implementation.
- `crates/bd_test_support/tests/contract_registry.rs` has 23 passing tests
  covering duplicate ownership and IDs, authority resolution, primary and
  supporting test references, ignored/deferred/retired protections, evidence
  and profile vocabularies, visual acceptance evidence, strict RON fields,
  deterministic diagnostics, and explicit report totals.
- Focused result: 30 passed, 0 failed, 0 ignored across `bd_test_support`.
- Workspace result with `--no-fail-fast`: 496 listed, 486 passed, 8 failed, 2
  ignored. The failure set is unchanged from the visual red batch.
- `cargo fmt --all -- --check`, strict `bd_test_support` Clippy, and
  `git diff --check` pass.
- This is an initial registry batch, not Phase 0 or Phase 1 completion. The
  complete suite inventory, full Foundation contract seed, and migration
  ledger remain open.

## Visual repair implementation evidence — 2026-07-26

- All eight tests in the visual red batch now pass. The repairs cover
  player-following viewport projection at both terminal profiles, shared
  player/resource projection, distinct station/resource styles, complete
  compact build details, typed egress rejection, distinct station-staffing
  identity, and the outpost resource legend.
- Three focused core tests define station-placement egress validation, and one
  application integration test proves rejection is typed and atomic through
  the production action boundary.
- `cargo test -p bd_tui --lib`: 53 passed, 0 failed, 0 ignored.
- `cargo test -p bd_tui --test input_help`: 22 passed, 0 failed, 0 ignored.
- `cargo test -p bd_core colony::stations::tests::placement_`: 3 passed, 0
  failed, 0 ignored.
- `cargo test -p bd_app --test colony_spatial_contract`: 1 passed, 0 failed, 0
  ignored.
- `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D
  warnings`, `cargo fmt --all -- --check`, and `git diff --check` pass.
- Full executable workspace result: 500 listed, 498 passed, 0 failed, 2
  ignored. Workspace doctests also pass (four targets, zero doctests).
- The final full-suite run used `RUST_MIN_STACK=16777216`,
  `CARGO_INCREMENTAL=0`, and one build job after the Rust compiler and rustdoc
  processes crashed under their default stack. The stabilized run completed
  without a code or test failure.
- The seven registered contracts are `GreenUnreviewed`, not `Accepted`.
  Their remaining semantic, canvas, style, geometry, transition, profile, and
  PTY evidence is governed by `VISUAL-ACCEPTANCE-MATRIX.md`.
- Historical red results above remain recorded as the before-state and do not
  describe the current automated result.

## Responsive title repair evidence — 2026-07-26

- The malformed ASCII-art splash was removed from the production renderer.
- The replacement uses one complete centered `BROKEN DIVINITY` wordmark,
  `FOUNDATION BUILD`, a begin prompt, and version text at both supported
  terminal profiles.
- Four focused title tests pass, including exact wordmark presence,
  horizontal geometry, resolved accent/bold styling, controls, and
  persistence-error visibility.
- Real PTY checks pass at 80x24 and 60x20 with isolated XDG roots.
- Post-repair workspace result: 502 listed, 500 passed, 0 failed, 2
  intentionally ignored.
- The visual matrix records the title as `Green unreviewed`; full snapshot
  review remains open.

## Physical survivor-work contract batch — 2026-07-26

Authority: GDD Sections 6 and 8, D-18, and THC-01 through THC-03.

- Added `crates/bd_app/tests/survivor_work_contract.rs` with 16 granular
  production-boundary contracts. Exact positions prove movement and occupancy;
  daily summaries prove production; forecasts and checkpoints use their
  production APIs.
- Added only test-support precondition adapters for stable survivor lookup,
  resource-node state, explicit positions, and explicit shelter tiles. The
  adapters contain no expected movement, pathing, activity, or production
  logic.
- Focused result: 16 listed, 5 passed, 11 failed, 0 ignored.
- Full executable workspace result with `--no-fail-fast`: 518 listed, 505
  passed, 11 failed, 2 ignored. The only failing target is
  `survivor_work_contract`; no pre-existing target regressed.
- Passing guards:
  - paused assignment grants no movement;
  - one accepted Outpost turn grants one cardinal step;
  - idle scheduler/render frames grant no movement;
  - Tactical turns do not move colony survivors;
  - save/load derives the same next worker step.
- Confirmed red behavior:
  - Rest does not replay equivalent worker movement;
  - workers cannot path around a wall and expose no named Blocked reason;
  - workers enter station and resource-node tiles;
  - multiple workers stack on one station tile;
  - a physically adjacent station worker is pulled into the station;
  - remote station and gathering assignments produce at the day boundary;
  - the forecast promises remote gatherer output.
- Existing `colony_day_cycle`, `mvp_correction`, and `phase6_input` targets
  remain green (39 passed total). This proves the red batch exposes previously
  uncovered GDD drift rather than breaking established fixtures.
- Seven machine-readable colony contracts were added to
  `testing/foundation-contracts.ron`: six `Red` and one
  `GreenUnreviewed` persistence contract.
- No production implementation was changed in this test-authoring batch.

## Holistic GDD contract sweep — 2026-07-26

Authority: GDD Sections 6–8, D-10, D-18, and testing-standard contract
families 8.1 through 8.10.

- Added `testing/FOUNDATION-REQUIREMENT-MAP.md` as a non-authoritative evidence
  index. It decomposes the locked Foundation into shell, controls, Build,
  management, viewport/visual language, workers, economy, fixed dungeon,
  persistence, progression/content, and deferred-boundary rows.
- Added 24 narrow tests:
  - eleven production-input and projected-visual contracts in
    `phase6_input`;
  - three fixed-dungeon action contracts;
  - one exhaustive shelter-placement property contract;
  - two rendered TUI contracts;
  - seven additional physical-worker positive/negative contracts.
- Added thirteen machine-readable contracts to
  `testing/foundation-contracts.ron`. The registry now contains 27 contracts
  and its 23 validation tests pass.
- The new green contracts prove:
  - `c` and `e` open distinct paused mode identities;
  - a named staffing confirmation changes only the selected survivor;
  - every accepted cardinal placement in the fixed 40x30 shelter preserves a
    route to the gate;
  - fixed-dungeon legal and wall movement have exact state changes;
  - premature extraction is typed and atomic;
  - Idle workers remain stationary on accepted turns;
  - adjacent matching gathering and zero-Supplies recovery have positive
    executable cases;
  - Rest and waits currently agree on daily resource deltas;
  - loading grants no immediate movement or production step.
- The twelve new red tests expose these eleven behavior groups:
  - Task management and station staffing expose one combined choice list.
  - A same-batch management open/gameplay/Escape sequence does not cancel
    atomically.
  - Build placement initially highlights the player tile while unchanged
    confirmation evaluates north.
  - Rejected build confirmation closes placement instead of allowing
    correction.
  - Placement omits selected station name, cost, and effect at both profiles.
  - Rendered Help clips the resource legend even at 80x24.
  - Altar and idle survivor both render as `A` without color.
  - Workshop and Water Source both render as `W` without color.
  - A staffed station has the same ASCII projection as an unstaffed station.
  - A worker sealed away from its station still counts as productive.
  - A Supplies gatherer beside the wrong node type still produces remotely
    from a Water Source.
- Full executable workspace result with `--no-fail-fast`: 542 listed, 517
  passed, 23 failed, 2 ignored. The only failing targets are
  `phase6_input` (8), `survivor_work_contract` (13), and `bd_tui --lib` (2).
  No previously green target regressed.
- Workspace doctests pass for all four library crates.
- `cargo fmt --all -- --check`, strict Clippy for all changed Rust targets,
  registry validation, and `git diff --check` pass.
- No production implementation was changed in this sweep.

## Foundation failure-remediation completion — 2026-07-26

Authority: GDD Sections 3 and 6–9, D-09/D-10/D-16 through D-19, Kernel
schedule and presentation boundaries, and Section 22 of the active hardening
plan.

### Baseline and final result

- Recorded baseline: 542 listed, 517 passed, 23 failed, 2 ignored.
- Final result: 556 listed, 554 active passed, 0 failed, 2 ignored.
- All 23 original failures remain active and pass; none was weakened or
  ignored.
- Workspace doctests pass.

### Implemented authorities

- `TimeAdvancePlan` owns elapsed turns, Outpost worker steps, and cause.
- One deterministic A* movement/activity resolver owns worker position and
  Idle/EnRoute/Working/Blocked projection.
- `evaluate_physical_work` is the shared non-mutating contribution evaluator
  for activity, station production, gathering, forecast, summaries, and
  station recovery effects.
- One `BuildInteraction` resource owns selection, placement, validation,
  awaiting resolution, denial recovery, and cancellation.
- One semantic `MapVisualVm` list owns projected map entities and fixtures.
  Worker and resource tokens resolve through `SymbolRegistry`; station state
  resolves through `StationCatalog`.
- Help derives controls from `CommandBindings` and legends from the same
  symbol/station registries used by map projection.
- `FoundationFingerprint` compares durable state by stable identity and
  excludes raw entity bits plus transient modal/build state.
- Restore clears transient interactions and recomputes activity without
  movement, production, or duplicate Blocked feedback.

### Additional red-first defects closed

- An assigned but remote Bed worker incorrectly received mood recovery.
- A restored worker could expose stale or missing derived activity.
- Help descriptions were ellipsized and staffing advertised the wrong cancel
  key.
- Loose fixed-dungeon loot existed in state but was absent from map
  projection.
- A denied build could remain stuck in `AwaitingResolution`.
- The build workflow had multiple independently writable state resources.
- Map categories were projected through separate raw glyph collections.
- Combat checkpoints could capture advanced RNG before a same-frame damage
  delta was applied. Explicit mutation sub-sets now order action effects
  before pool-delta application.

### Focused results

- `survivor_work_contract`: 24 passed.
- `phase6_input`: 23 passed.
- `test_harness_contract`: 4 passed.
- `persistence`: 13 passed.
- `foundation_scenario`: 16 passed.
- `colony_day_cycle`: 13 passed.
- `bd_tui --lib`: 59 passed.
- `input_help`: 22 passed.
- `contract_registry`: 23 passed.

### Static and content gates

- `cargo fmt --all -- --check`: passed.
- `cargo check --workspace`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo run -p bd_app -- --validate`: passed.
- `git diff --check`: passed.

### Ignored tests

- `integration_diagnostic::diagnose_title_to_outpost_state`: diagnostic-only
  snapshot, intentionally outside the Foundation regression gate.
- `legacy_terminal_first_keypress_in_outpost_is_move_not_build`: requires a
  real terminal; the production-key behavior is covered by PTY discovery.

### Real-terminal evidence

- Actual `bd` launcher at 80x24: complete title and Outpost; complete Help;
  Build detail, preview, spend, station creation; distinct task/staffing
  modals; worker EnRoute movement and target direction; fixed dungeon with
  visible loot, pickup, combat, and clean terminal restoration.
- Actual `bd` launcher at 60x20: complete title, Outpost, Help without
  ellipsis, Build selection/placement/success, semantic station/resource/
  worker glyphs, fixed-dungeon loot visibility, and clean terminal
  restoration.
- Earlier full baseline-profile discovery also completed item use,
  extraction, return, and save/load continuity. Canonical scenario and
  persistence tests prove the same non-geometric outcomes after the final
  refactors.

### GDD drift audit

- The complete current GDD was read after implementation.
- The Foundation still expresses preparation, pressure, survival, colony
  continuity, and the fixed enter/explore/fight/loot/extract/return loop.
- Skills improve through actions and representative virtue expression remains
  intact.
- Procgen, raids, colony events, sanity, full overworld, faction reputation,
  final faction canon, and deeper theology mechanics remain inactive.
- No Foundation scope drift found.

Visual rows without complete semantic/canvas/style/geometry/transition/PTY
evidence remain `Green unreviewed` or `Open` in the visual matrix. They were
not bulk-accepted.

## Foundation basic colony-loop evidence — 2026-07-27

### Automated evidence

- Data/content validation now covers all source, raw-resource, recipe,
  station, output, amount, and source/input compatibility references with
  offending IDs in diagnostics.
- Node generation passes complete-or-error atomicity, a named 128-seed
  profile, persistence, spacing/reachability, and source-file permutation.
- Logistics passes every stage transition, exact conservation, all configured
  recipes, a fixture fourth recipe, different-chain concurrency, sole-work-
  tile contention, cancellation/reassignment deposit, every-stage
  persistence, render/Tactical time isolation, and deterministic replay.
- Input/presentation passes the full production-key workflow, a deterministic
  256-step production-key fuzz profile, six-entry numeric Build selection,
  runtime Foundation Help at 60x20, and wrapped compact staffing choices.
- Five repeated node/route runs passed without an intermittent failure.
- Canonical gate before the final C8 visual fixes: 599 listed, 597 passed,
  0 failed, 2 allowlisted ignored; all 44 required contracts were
  `GreenUnreviewed`.
- Final canonical gate after C8 remediation: 10/10 steps passed; 601 listed,
  599 passed, 0 failed, 2 allowlisted ignored; all 44 required contracts
  remained green with zero duplicate primary owners.

### Real-terminal evidence

Runtime seed `0` was exercised through the actual managed-launcher target with
isolated XDG roots.

- 80x24: title, clean colony, complete Help, six-entry Build selection,
  cumulative preview, typed invalid reason, exact placement, paused
  survivor→processor→recipe selection, source travel, gathering, raw cargo,
  station travel, refining, resource credit, carrying save/load continuation,
  Rest, fixed-dungeon combat, extraction, and colony return all passed.
- 60x20: the same compact-critical colony stages, complete wrapped recipes,
  save/load, Rest, fixed-dungeon combat, extraction, and colony return passed.
- The terminal audit found three missing automated cases: stale `1-5` Build
  controls, a 60x20 Help panic caused by the sixth station legend, and compact
  staffing-row clipping. Each was reproduced by a focused failing test and
  fixed before the successful rerun.
- One deliberate rapid five-key burst produced the existing bounded-input
  warning and discarded one excess key. Normal paced input remained stable;
  the warning is expected by the bounded-input contract.

### Drift and acceptance

- GDD Sections 3, 6, 8, and 10 still describe the implemented
  survival-first physical colony and fixed dungeon Foundation.
- D-20 conservation, timing, data ownership, generation, build, persistence,
  and projection clauses remain satisfied.
- Kernel resolver, schedule, UI/view-model, semantic ASCII, content-ID, and
  save/load boundaries remain intact.
- Deferred procgen topology, raids, events, sanity, overworld generation,
  faction reputation, and Product P2 systems remain inactive.

No C8 behavior failure remains. Applicable visual records remain
`GreenUnreviewed` until owner review; they were not silently promoted to
`Accepted`.

## Turn-based work and idle construction evidence

- `COLONY-WORK-DURATION-001` proves three gather work turns and two refine
  work turns gate their configured yields with no partial or duplicate credit.
- `COLONY-CONSTRUCTION-001` proves paid site creation, automatic idle-worker
  travel, assigned-worker exclusion, immediate release when the player assigns
  gathering work, exact completion, persistence, and render-frame isolation.
- Invalid zero recipe work durations and zero buildable-station construction
  work are rejected by content validation with the offending record and file.
- Construction uses `%` while unfinished and exposes `completed/required`
  progress; unfinished sites are excluded from staffing, forecast, daily
  station effects, and recipe target selection.
- Legacy direct daily gathering is explicitly disjoint from durable logistics
  jobs, preventing one survivor from receiving both transaction paths.

Final canonical gate: 10/10 steps passed; 609 listed, 607 passed, 0 failed,
and 2 allowlisted ignored. All 46 required contracts are
`GreenUnreviewed`; none were silently promoted to `Accepted`.

A clean isolated 80x24 real-terminal run confirmed:

- Build selection and placement remained paused and readable;
- accepted placement showed `%` rather than an operational station glyph;
- the station panel showed `Stove construction — 0/4 work`;
- three idle survivors visibly changed to EnRoute and moved toward the site;
- one adjacent worker advanced visible progress 1/4, 2/4, 3/4, then 4/4;
- completion changed `%` to the operational Stove glyph exactly once, replaced
  construction progress with the Stove effect/staffing row, and logged one
  concise completion result;
- ordinary paced input produced no bounded-input warning or malformed frame.

The new construction visual remains `GreenUnreviewed` pending owner review.

## Direct gathering coherence red baseline

Authority: GDD Minimum Colony Foundation and D-22.

The test-first baseline adds eleven discrete required contracts without changing
production behavior:

- content ownership for source, output, amount, and work duration;
- exact three-tick completion and one-credit semantics;
- removal of legacy day-boundary direct-gather output;
- render/Tactical isolation and Rest equivalence;
- partial-progress persistence and reassignment reset;
- paused input workflow with source and progress;
- human recipe/resource labels;
- separately named worker completion and day upkeep;
- visible human-labelled raw stockpiles;
- blocked direct gathering with a named target and actionable reason.

Focused red execution:

- `colony_direct_gather_contract`: 9 listed; 1 passed and 8 failed. Rest versus
  equivalent individual turns is the one green characterization.
- `phase6_input`: 36 listed; 31 passed and 5 failed. The failures precisely
  identify missing direct-gather progress, leaked resource IDs, the ambiguous
  forecast, the absent raw-stockpile projection, and missing blocked-work
  feedback.
- `bd_tui --lib`: 63 passed, 0 failed. Synthetic projection evidence proves
  the required progress, stockpile, and split-forecast information fits both
  supported terminal profiles once the application supplies it.
- Existing construction override, production route, input isolation, and
  compact-layout tests remain green.
- Canonical gate inventory: 622 tests listed. Formatting, compilation,
  contract-registry validation, ignored-test allowlist validation, strict
  Clippy, content validation, and whitespace validation pass. Workspace tests
  stop at the intentional direct-gather red target (1 passed, 8 failed), so
  outcome-count and contract-metric checks also fail by design until
  implementation turns the required contracts green.

These contracts must remain `Red` until implementation satisfies the stated
outcomes. Test deletion, weakening, reclassification, or day-boundary
expectation changes are not valid fixes.

## Legacy test reliability migration — 2026-07-27

The owner-authorized reliability pass removed 23 tests whose assertions were
manufactured, tautological, count-only duplicates, ignored diagnostics,
single-profile substring duplicates, or prohibited wall-clock checks. Three
deferred procgen rules were moved into focused tests, and repeated persistence
stress was rewritten around the normalized fingerprint and deterministic next
action.

Current generated inventory and workspace result after the explicit-frame
migration:

- 603 tests listed;
- 588 passed, 15 failed, 0 ignored;
- the only failing targets are `colony_direct_gather_contract` (8),
  `phase6_input` (5), `mvp_correction` (1), and `persistence` (1);
- all direct-gather and projection failures remain intentional D-22 red
  evidence;
- the `mvp_correction` failure proves the legacy next-day forecast still
  credits direct worker output;
- the strengthened persistence failure proves an active Tactical restore
  changes survivor activity from `Idle` to `Unresolved`.

Quality metrics after migration:

- zero `first_survivor` or `first_station` selectors remain;
- zero ignored tests remain;
- zero wall-clock functional assertions remain in `bd_app` integration tests;
- the committed diagnostic and manufactured direct-mutation targets are gone;
- TUI `contains` calls fell from 52 to 43 after removing superseded
  single-profile token checks;
- `INPUT-MOVE-001` passes through the production physical-key path and proves
  the first Outpost movement key cannot open or create Build state;
- 59 required contracts are registered, including the newly red
  `PERSIST-DUNGEON-001`.
- zero `expect_action` definitions or call sites remain;
- all 93 former call sites now name their one action-result frame, while
  Tactical enemy responses, mode transitions, and day resolution use separate
  purpose-specific frame calls.

Canonical gate result after the explicit-frame rerun: 8 steps pass and 3 fail.
Formatting, compile-all, registry validation, the now-empty ignored allowlist,
603-test inventory,
strict Clippy, content validation, and whitespace pass. Workspace execution
stops at the first intentional red target, so the outcome-count and contract
metrics steps also fail from the partial 46 observed outcomes. The separate
`--no-fail-fast` workspace run is the authoritative 588/15/0 result.

The detailed old-proof, replacement, and validation record is in
`docs/MIGRATION-AND-DEPRECATION.md`. The accepted-action frame-control
migration is complete. Denial, buffered-input, and higher-level transition
helpers are outside that 93-call migration and remain subject to the general
hidden-update audit rather than being silently declared complete.

## D-22 implementation and regression closure — 2026-07-27

Authority: GDD Minimum colony foundation and D-22.

The implementation closed all 15 failures exposed by the explicit-frame
migration without deleting, ignoring, or weakening their replacement
contracts:

- data-defined direct-gather source, output, amount, and three-turn duration;
- exactly-once direct output on accepted Outpost worker ticks;
- no movement/arrival, render, Tactical, persistence, or day-boundary work;
- Rest equivalence, reassignment reset, and partial-progress persistence;
- no legacy direct-gather contribution in next-day forecasting;
- human colony work labels, source/progress/result/blocked feedback, visible
  raw stockpiles, and separate next-worker/next-day projections;
- stable survivor activity across active-Tactical save/load.

Validation:

- focused `colony_direct_gather_contract`: 9 passed;
- focused `phase6_input`: 37 passed;
- neighboring `mvp_correction`: 17 passed;
- neighboring `persistence`: 13 passed;
- neighboring `survivor_work_contract`: 22 passed;
- neighboring `colony_day_cycle`: 12 passed;
- neighboring `colony_production_route_contract`: 18 passed;
- `bd_core --lib`: 200 passed;
- `bd_data --lib`: 32 passed;
- `bd_tui --lib`: 58 passed;
- locked workspace inventory: 603 listed, 603 passed, 0 failed, 0 ignored.

All 59 required registry contracts are now automated-green and recorded as
`GreenUnreviewed`. Visual contracts are not `Accepted`: unresolved style and
real-PTY cells remain open in `testing/VISUAL-ACCEPTANCE-MATRIX.md`.

## Intensive Foundation development-guide pass — 2026-07-27

This pass changed tests and testing governance only. It did not repair the
production behavior exposed by the new red contracts.

### Scope and contract inventory

- Read the current GDD, locked decisions, Kernel boundaries, authoritative
  testing standard, requirement map, visual matrix, registry, and existing
  suite before authoring tests.
- Added 16 tests and 14 independently owned Foundation contracts.
- Registry total: 73 required contracts.
- Contract status: 69 `GreenUnreviewed`, 4 `Red`, 0 `Accepted`, and no hidden
  deferred or ignored required contract.
- Every new primary includes Given/When/Then, forbidden-mutation, evidence-layer,
  stable case-ID, and contract-ID diagnostics.

The added automated coverage owns:

- missing and corrupt Title-load atomicity, classified feedback, and New Game
  recovery through physical F9/Enter input;
- exactly one application-exit event from one physical quit input;
- an advertised-control routing matrix for Title, Outpost, Tactical, Build, and
  Game Over;
- exact management confirmation results for direct gathering and station
  staffing;
- zero-Supplies denial and reachable gathering recovery guidance;
- full production-key fixed-dungeon extraction and defeat/restart workflows;
- player-visible save/load equivalence for colony and Tactical checkpoints;
- every shelter coordinate under both production-derived viewport sizes and
  relative-geometry preservation while panning;
- blocked-worker target/reason/progress fit at 80x24 and 60x20;
- modal/footer agreement and complete daily-delta visibility at both supported
  profiles.

### Validation

- `cargo fmt --all -- --check`: pass.
- `cargo check --workspace --all-targets --locked`: pass.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: pass.
- `git diff --check`: pass.
- Contract-registry validation: 24 passed, including the seeded 73-contract
  inventory.
- Workspace inventory: 619 listed.
- Full `cargo test --workspace --locked --no-fail-fast`: 615 passed, 4 failed,
  0 ignored.
- All pre-existing tests pass. The only failures are the four newly registered
  red primaries below.

| Contract | Exact failure | Implementation direction |
|---|---|---|
| `INPUT-MGMT-007` | `Task set to Gather:Supplies for survivor.` omits Survivor 2, human task label, Water target, and EnRoute | Build the result from the confirmed named survivor, data-owned task/source labels, and derived worker activity; emit once |
| `INPUT-MGMT-008` | `Survivor 2 assigned to station.` omits Stove and EnRoute | Build the result from the confirmed station identity and derived worker activity; emit once |
| `VISUAL-MGMT-002` | Management modal shows confirm/cancel while the footer advertises normal Travel/Move/Build controls | Project one explicit management interaction state into footer guidance at both profiles |
| `VISUAL-COLONY-STATE-002` | The Log clips after Supplies, hiding Materials, Plants, Faith, and Food | Give authoritative day results a wrapping or dedicated presentation that retains every delta at 80x24 and 60x20 |

The suite is intentionally red. Deleting, weakening, ignoring, or
reclassifying these tests is not a valid implementation. The next production
batch should repair the four contracts in the order above, run each focused
owner after its change, then rerun the no-fail-fast workspace suite and update
the registry/matrices from observed evidence only.

## Intensive Foundation guide-contract closure — 2026-07-27

The four red contracts above were repaired without deleting, ignoring,
weakening, or reclassifying their primary tests.

- Task and station assignment mutation now attach one transient feedback
  request to the named survivor. A separate post-movement system emits the
  decisive result only after authoritative `WorkerActivity` derivation, so the
  message names the survivor, data-owned assignment label, physical target, and
  actual activity exactly once.
- Task and station management are explicit interaction modes. The footer uses
  the same active mode as the modal and displays only `1-9:select`,
  `Enter:confirm`, and the correct `c/e/Esc:cancel` control.
- `DailySummary` owns one DRY structured display representation. The ordinary
  log retains its causal one-line result, while the Outpost party panel renders
  six complete Day-result lines so no authoritative delta depends on the
  two-row Log width.
- The latest structured Day result is part of `StatsViewModel` and the
  persistence visible-projection fingerprint.

Validation:

- assignment-feedback owners: 2 passed;
- complete `phase6_input`: 44 passed;
- complete `bd_tui --lib`: 63 passed;
- complete `input_help`: 23 passed;
- `cargo test --workspace --locked --no-fail-fast`: 619 passed, 0 failed,
  0 ignored;
- strict workspace Clippy: pass;
- canonical development gate: 10/10 steps passed;
- gate inventory: 619 listed, 619 passed, 0 failed, 0 ignored;
- registry: 73 required, 73 `GreenUnreviewed`, 0 red, no duplicate primary
  owners.

These automated contracts are green, not owner-accepted. The visual matrix
still requires its listed resolved-style and real-PTY review cells before any
affected visual contract can become `Accepted`.

## Foundation UI development red baseline — 2026-07-28

Authority: GDD player loop, Minimum colony foundation, Minimum dungeon
foundation, D-18 through D-20, and
`docs/FOUNDATION-UI-IMPROVEMENT-PLAN.md`.

This pass changed tests, registry ownership, and evidence only. It did not
implement the UI behavior exposed by the new red contracts.

### Added coverage

- 17 tests:
  - 16 focused Ratatui semantic/canvas/style/geometry tests;
  - 1 production-input worker-projection workflow.
- 13 independently owned required contracts:
  - 2 `GreenUnreviewed` observation/cleanup contracts;
  - 11 intentional `Red` implementation targets.
- Registry total: 86 required contracts.
- No new ignored or deferred required test.

The new tests prove or expose:

- glyph, foreground, modifier, geometry, and deterministic-render observation;
- non-overlapping panel geometry and map-area hierarchy;
- off-screen target identity and distance;
- target distance in the authoritative worker row;
- color-independent valid/invalid placement presentation;
- exact unaffordable-build shortage;
- explicit task and staffing workflow stages;
- blocked-worker resolved style;
- decisive-result retention under routine-log pressure;
- rendered dungeon action denials, carried loot, and extraction readiness;
- title Load availability;
- modal-close cleanup and deterministic supported-profile resize round trip.

### Validation

- `cargo fmt --all`: pass.
- Contract-registry validation: 24 passed.
- Registry: 86 required; 75 `GreenUnreviewed`; 11 `Red`.
- Workspace inventory: 636 listed.
- Full `cargo test --workspace --locked --no-fail-fast`: 625 passed, 11
  failed, 0 ignored.
- Every failure is a newly registered primary. All 619 tests from the prior
  baseline still pass.

| Contract | Confirmed failure |
|---|---|
| `VISUAL-LAYOUT-001` | At 80x24 the map owns 308 cells while the Party panel owns 630 |
| `VISUAL-VIEWPORT-005` | Off-screen target renders only a direction arrow, with no name or distance |
| `VISUAL-COLONY-WORK-006` | Worker row names Water but omits the current 16-tile distance |
| `VISUAL-BUILD-005` | Valid and invalid previews use the same glyph and modifier |
| `VISUAL-BUILD-006` | Unaffordable Workshop is red but does not state the three-Supplies shortage |
| `VISUAL-MGMT-003` | Task modal has no Survivor → Task → Confirm stage indicator |
| `VISUAL-MGMT-004` | Staffing modal has no Survivor → Station → Recipe → Confirm stage indicator |
| `VISUAL-LANGUAGE-004` | Working and Blocked workers both resolve to the same green Ally style |
| `VISUAL-FEEDBACK-001` | Routine movement immediately buries a decisive build rejection |
| `VISUAL-DUNGEON-001` | Dungeon status says Stored loot and omits extraction readiness |
| `VISUAL-SHELL-001` | Title advertises Load when no save exists without explaining availability |

The suite is intentionally red. Do not delete, weaken, ignore, or reclassify
these contracts to restore aggregate green. Implement them one at a time in
the phase order owned by the UI improvement plan.

## Foundation UI implementation evidence — 2026-07-28

The 11 UI development contracts above were implemented without deleting,
ignoring, or weakening their primary tests. Their registry status is now
`GreenUnreviewed`; visual owner acceptance remains separate.

- The shelter map is the largest interactive panel at both supported profiles,
  while panel-overlap geometry remains green.
- Off-screen assignments and survivor rows expose target identity and numeric
  Manhattan distance.
- Invalid placement uses a data-driven danger token and distinct `!` glyph;
  unaffordable selection reports the exact Supplies shortage.
- Task and station management display their complete workflow stages.
- Blocked workers resolve to a danger style distinct from working workers.
- The newest warning remains visible when routine log rows overflow.
- Tactical stats project carried transient loot and extraction readiness from
  authoritative ECS state.
- Title Load availability is projected from the real manual save slot and is
  updated after a successful save.

Validation:

- `cargo fmt --all -- --check`: pass.
- `cargo test -p bd_tui --lib`: 79 passed, 0 failed, 0 ignored.
- `cargo test -p bd_app --bin bd`: 25 passed, 0 failed, 0 ignored.
- `cargo test --workspace`: pass, including 636 tests and 0 ignored.

## Shared semantic TUI theme and chrome — 2026-08-01

Authority: Kernel Goal 5, the UI plan's DRY/registry ownership rules, and UI1-A.

- Added `VISUAL-THEME-001` as `GreenUnreviewed`; no visual contract was
  promoted to `Accepted` without separate owner review.
- Added semantic UI roles for text, muted, accent, positive, warning,
  informational, danger, key hints, panel chrome, and modal chrome to the
  existing `StyleToken`/`ThemeRegistry` authority and shipped theme content.
- Added one reusable chrome module for standard, modal, and danger panels.
- Migrated colony, dungeon, inventory, help, title, game-over, event, footer,
  and terminal-error renderers away from renderer-owned Ratatui colors.
- Supported-profile buffer evidence confirms neutral panel borders with
  emphasized theme-owned titles at 80x24 and 60x20.
- Real PTY review passed clean title-to-Outpost rendering and terminal/cursor
  restoration at 80x24 and 60x20 using isolated XDG roots.

Red evidence:

- `tui_renderers_delegate_terminal_colors_to_the_theme_layer` failed on the
  pre-change renderer, identifying every raw `Color::` choice in `screens.rs`.

Green evidence:

- `bd_tui --lib`: 83 passed, 0 failed, 0 ignored.
- `bd_tui --test input_help`: 23 passed, 0 failed, 0 ignored.
- Contract-registry target: 24 passed, 0 failed, 0 ignored.
- Canonical gate: 10/10 steps passed.
- Gate inventory: 646 listed, 646 passed, 0 failed, 0 ignored.
- Registry: 89 required, 89 `GreenUnreviewed`, 0 red.
- Formatting, compilation, strict Clippy, content validation, ignored-test
  allowlist, contract metrics, and whitespace checks passed.

GDD drift review: presentation became more coherent without changing colony
simulation, dungeon behavior, balance, content scope, input semantics, or any
deferred Product P2 system.

## Cinder Rite visual identity red baseline — 2026-08-01

Authority: Kernel Goal 5 and the owner-approved acceptance target in
`docs/FOUNDATION-UI-STYLE-MOCKUPS.md`.

This batch changes tests, contract ownership, and evidence only. It does not
implement the selected theme or frame behavior.

- Added `VISUAL-IDENTITY-001` as one `FoundationRequired` red contract.
- Primary evidence:
  `selected_cinder_rite_identity_frames_colony_and_reusable_screens`.
- Profiles: 80x24 and 60x20.
- Representative reuse cases: Outpost, Combat, and Inventory.
- Registry target: 90 required contracts; 89 `GreenUnreviewed`; 1 `Red`.
- No required test is ignored or deferred.

The gate is intentionally exact about:

- the owner-selected semantic Cinder Rite colors;
- bold lit-copper title hierarchy;
- warm-bone body hierarchy on colony and combat canvases;
- at least one substantial closed double-line Ruined Reliquary frame per
  representative screen and supported profile.

The gate intentionally does not own:

- screen prose or punctuation;
- exact panel counts, coordinates, or percentages;
- map glyphs or gameplay state;
- a full-buffer snapshot.

Focused red evidence:

```text
cargo +stable test -p bd_tui --lib \
  selected_cinder_rite_identity_frames_colony_and_reusable_screens -- --nocapture
```

Result: 0 passed, 1 failed, 0 ignored. The failure names each unresolved legacy
ANSI role and reports one compact visual crop per screen/profile showing the
current single-line frame. Compilation succeeds.

Neighboring green evidence:

- reusable chrome semantics: 2 passed, 0 failed, 0 ignored;
- shared Outpost chrome projection: 1 passed, 0 failed, 0 ignored;
- seeded registry ownership: 1 passed, 0 failed, 0 ignored.

Aggregate evidence:

- full workspace with `--no-fail-fast`: 646 passed, 1 failed, 0 ignored;
- independent inventory: 647 tests listed;
- canonical gate: 8 steps passed and 3 failed;
- the workspace-test failure is `VISUAL-IDENTITY-001`;
- the contract-metrics mismatch is derivative: the canonical workspace command
  stops before the final 23-test `input_help` target after the intentional
  `bd_tui` library failure, so it observes 623 passed and 1 failed rather than
  all 647 outcomes.

The suite is intentionally red. Do not delete, ignore, loosen, or reclassify
`VISUAL-IDENTITY-001` to restore aggregate green. Implement the shared theme and
chrome target, then rerun buffer and real-PTY evidence before changing its
status to `GreenUnreviewed`.

## Cinder Rite observer-integrity strengthening — 2026-08-01

Authority: Section 6.6 of the authoritative testing standard and
`VISUAL-IDENTITY-001`.

This batch strengthens test evidence only; it does not change production
rendering.

- Replaced corner-presence frame detection with a universal assertion over the
  final composed terminal perimeter.
- The primary test now verifies all four corners, every horizontal segment,
  every vertical segment or explicitly allowed side junction, and the primary
  B3 foreground on every structural cell.
- Frame diagnostics report the total violation count, the first twelve exact
  coordinate/glyph/style failures, and a top-and-bottom visual crop.
- Warm-bone body evidence is restricted to non-map regions so terrain sharing
  the same resolved RGB value cannot create a false green.
- Added
  `closed_double_frame_observer_rejects_single_cell_breaks` as registered
  supporting evidence. Its valid control passes, while independent one-cell
  glyph, resolved-style, and geometry mutations are rejected at their exact
  coordinates.

Focused evidence:

- observer-integrity support: 1 passed, 0 failed, 0 ignored;
- primary identity contract: 0 passed, 1 failed, 0 ignored;
- all six Outpost, Combat, and Inventory cases at 80x24 and 60x20 report the
  disconnected or overwritten terminal perimeter;
- both Combat profiles additionally report zero non-map warm-bone body cells.

Neighboring and aggregate evidence:

- `bd_tui --lib`: 84 passed, 1 failed, 0 ignored;
- contract registry: 24 passed, 0 failed, 0 ignored;
- independent inventory: 648 tests listed;
- canonical gate: 8 steps passed and 3 failed;
- canonical observed outcomes before the downstream target skipped by Cargo:
  624 passed, 1 failed, 0 ignored.

The contract remains accurately `Red`. Production work must create a real
continuous outer frame that survives final composition and a semantic Combat
body hierarchy before this test or its evidence status changes.

## Cinder Rite workflow and implementation-guidance strengthening — 2026-08-01

Authority: Section 6.6 of the authoritative testing standard,
`VISUAL-IDENTITY-001`, and the forbidden-regression policy.

This batch changes test and evidence code only; it does not change production
rendering.

- Extended the primary contract from normal screens to the complete final
  buffers for normal Outpost, Build Selection, Build Placement, Combat, and
  Inventory at 80x24 and 60x20.
- Added shared-primitive evidence requiring major modal chrome to use the
  Cinder Rite double rule. The top-edge observer accepts semantic title cells,
  so it owns structure and resolved style without locking title copy.
- Added explicit preservation evidence for the complete established
  `Turn: 0 | Day: 3 | Broken Divinity Kernel v0.1.0` footer status. This is a
  named forbidden regression rather than incidental prose.
- Restricted warm-bone body evidence to semantic content inside the gameplay
  content area and outside the map and footer.
- Added implementation comments for smaller renderer agents: one reusable
  shell owns the perimeter and inner rectangle; overlays must compose inside
  that rectangle; the footer retains status, contextual controls, and global
  controls; palette and chrome stay theme-owned; no screen-name branches,
  one-off coordinates, or weakened observers may restore green.
- Diagnostics identify state, screen, profile, exact perimeter coordinates,
  missing status, and a bounded visual crop. They do not own general screen
  copy, panel count, exact panel rectangles, terrain glyphs, or gameplay state.

Focused red evidence:

- primary identity contract: 0 passed, 1 failed, 0 ignored;
- shared major-modal primitive: 39 structural/style violations, bounded to the
  first twelve in diagnostics;
- compact Build Selection: 16 outer-rail violations at 60x20;
- Build Placement: 10 outer-rail violations at both 80x24 and 60x20;
- required status footer: absent in all ten workflow/profile cases;
- normal Outpost, Combat, and Inventory retain continuous outer perimeters.

Real-PTY discovery evidence at 80x24 and 60x20 confirmed the same production
behavior before strengthening: normal frames render, compact Build Selection
and both Build Placement profiles overwrite the side rails, status text is
absent, and terminal/cursor restoration is clean.

Aggregate evidence:

- `bd_tui --lib`: 84 passed, 1 failed, 0 ignored;
- contract registry: 24 passed, 0 failed, 0 ignored;
- independent inventory: 648 tests listed;
- canonical gate: 8 steps passed and 3 failed;
- canonical observed outcomes: 624 passed, 1 failed, 0 ignored;
- the 23 unobserved outcomes are the downstream `input_help` target skipped by
  Cargo after the intentional library failure; they are not additional failing
  tests;
- formatting, compilation, contract registry, ignored-test allowlist, strict
  Clippy, content validation, and whitespace checks pass.

`VISUAL-IDENTITY-001` remains intentionally and accurately `Red`. Green now
requires reusable production composition that satisfies every normal and build
workflow case while preserving the status footer, followed by buffer and PTY
verification. Do not delete guidance, cases, or assertions to change status.

## Recent UI contract-structure audit — 2026-08-02

Authority: Sections 6.2, 6.5, and 6.6 of the authoritative testing standard;
`VISUAL-BUILD-001`, `VISUAL-THEME-001`, and `VISUAL-IDENTITY-001`.

This batch changes tests and evidence only. Production rendering is unchanged.

- `compact_build_selection_shows_complete_selected_effect` now carries the
  complete contract record and diagnostic fields. Its observer locates the
  titled modal in the final buffer and reads only the inner semantic region,
  independent of approved border glyphs. It still requires the complete
  selected station label, cost, and long effect at 60x20.
- The previous focused failure was reproduced before repair: 0 passed, 1
  failed. The rendered effect was complete; an incomplete single-line glyph
  strip list caused the false failure against double-line chrome.
- `tui_renderers_delegate_terminal_colors_to_the_theme_layer` now scans the
  shared chrome source it claimed to own and reports precondition/action plus
  forbidden-regression context.
- `outpost_panels_render_shared_chrome_at_supported_profiles` no longer assumes
  title text begins at `x + 2`; it finds a nonblank emphasized title cell on the
  owned top edge at each profile.
- `closed_double_frame_observer_rejects_single_cell_breaks` now records its
  full Given/When/Then boundary and complete diagnostic context while retaining
  independent glyph, style, and geometry mutations.
- `selected_cinder_rite_identity_frames_colony_and_reusable_screens` now states
  only the status footer as its executable forbidden regression and labels
  panel count, internal coordinates, map glyphs, gameplay state, and general
  copy as unowned rather than implying unexecuted assertions.

Focused and neighboring evidence:

- the five audited UI tests each pass independently;
- `bd_tui --lib`: 85 passed, 0 failed, 0 ignored;
- contract registry: 24 passed, 0 failed, 0 ignored;
- canonical gate: 10 steps passed, 0 failed;
- canonical inventory: 648 listed, 648 passed, 0 failed, 0 ignored;
- formatting, compilation, strict Clippy, content validation, ignored-test
  allowlist, contract metrics, and whitespace checks pass.

Partial current PTY evidence at 80x24 and 60x20 confirms Outpost and Build
Selection chrome, status, outer rails, and clean alternate-screen/cursor
restoration; compact Build Placement also passed. Complete placement/profile
PTY coverage and owner/DRY review remain open, so `VISUAL-IDENTITY-001` is
`GreenUnreviewed`, not `Accepted` or `ReviewedGreen`.

## Cinder Rite panel, meter, and ribbon correction — 2026-08-02

Authority: the owner-reviewed B3 direction in
`docs/FOUNDATION-UI-STYLE-MOCKUPS.md`, Kernel Goal 5, UI plan Sections 5 and 8,
and `VISUAL-THEME-001` / `VISUAL-IDENTITY-001`.

Owner review found that the previous green implementation changed the palette
and terminal perimeter but left ordinary panels and bars on the preceding
presentation. Both contracts were returned to `Red` before production work.
The strengthened final-buffer tests reproduced these specific gaps:

- generic major-panel double boxes and headings without the Reliquary marker;
- plain HP/AP numbers without responsive ASCII tracks;
- the previous status/control footer without mode or command ribbons; and
- no reusable observer proving complete inner-panel structure or meter style.

Shared `bd_tui::chrome` primitives now own ordinary single-rule panel framing,
major double-rule modal framing, responsive exact-value ASCII meters, the
mode/day/turn/version ribbon, and styled command-key chips. Outpost, Combat,
Inventory, title, game-over, build, and management composition consume those
primitives. A production-source scan finds no renderer-owned `Block` or
`BorderType` construction outside the shared chrome owner.

The first green buffer pass used `HP 8/10`. Real 60x20 PTY evidence then exposed
that the live `HP 30/30` value removed the compact track. The identity fixture
was strengthened to the live-sized `24/30` partial case, the named compact Stats
allocation was widened while retaining the map as the largest panel, and the
final PTY rerun showed `HP30/30[##]` and `AP3/3[##]`. During that rerun, the
meter observer itself was corrected to use terminal-cell indices instead of
UTF-8 byte offsets after the leading Unicode border exposed a false failure.

Evidence after correction:

- focused panel, meter-observer, and identity tests pass;
- `bd_tui --lib`: 86 passed, 0 failed, 0 ignored;
- complete `bd_tui` target: 109 passed, 0 failed, 0 ignored;
- contract registry: 24 passed, 0 failed, 0 ignored;
- canonical gate: 10 steps passed, 0 failed;
- canonical inventory: 649 listed, 649 passed, 0 failed, 0 ignored;
- 80x24 and 60x20 real PTY runs confirm normal Outpost, Build Selection,
  responsive live-sized meters, mode/command ribbons, modal cancellation,
  alternate-screen restoration, and cursor restoration.

`VISUAL-THEME-001` and `VISUAL-IDENTITY-001` are `GreenUnreviewed`. Complete
Build Placement coverage at every profile and owner review of the corrected
visual result remain open, so this is not `Accepted` or `ReviewedGreen`.

## Provisions and contextual colony interaction red handoff — 2026-08-02

Authority: GDD Shelter and Minimum Colony Foundation; D-18 through D-22;
UI plan Section 17; authoritative testing standard Sections 5 through 7.

This batch adds planning, tests, registry records, and evidence only. It does
not implement UI or gameplay behavior. Three atomic contracts prevent one
partial visual fix from standing in for the complete upgrade:

- `VISUAL-ECON-002` owns an exact shared Supplies gauge, text pressure state,
  and visible next-day delta/result at 80x24 and 60x20 while preserving HP/AP;
- `COLONY-PROXIMITY-001` owns accepted-movement entry into station/node range,
  one deduplicated named `NEARBY` Chronicle fact, semantic Interact
  availability, and forbidden colony mutations;
- `VISUAL-CONTEXT-001` owns final Context presentation for station, node, and
  colonist targets, station/node Chronicle feedback, and visibly disabled
  `Set Production — Coming later`.

Focused red evidence:

- `provisions_show_stock_pressure_and_dawn_outlook_at_supported_profiles`:
  0 passed, 1 failed, 0 ignored. The first low/80x24 case reports no semantic
  `SUP 10 [...]` gauge; the final buffer shows flat `Sup:10`.
- `entering_adjacent_range_emits_one_deduplicated_nearby_hint`: 0 passed,
  1 failed, 0 ignored. Production movement reaches the expected Water Source
  work-adjacent tile and records `ability.move`, but emits zero `NEARBY` facts.
- `nearby_context_presentation_serves_station_node_and_colonist_at_supported_profiles`:
  0 passed, 1 failed, 0 ignored. The first station/80x24 case reports generic
  `Log` instead of Chronicle; the bounded buffer also shows generic `Actions`
  and clipped station status.
- all three focused tests compile before failing; no setup panic, missing type,
  hidden frame settling, or ignored test is being used as red evidence.
- the seeded registry validation passes with exactly these three records Red.

Neighboring and canonical evidence:

- existing Cinder Rite identity contract: 1 passed, 0 failed, 0 ignored;
- neighboring buffered semantic-input contract: 1 passed, 0 failed, 0 ignored;
- canonical inventory: 652 tests listed independently;
- canonical gate: 8 steps passed and 3 failed;
- Cargo stopped after the intentional `COLONY-PROXIMITY-001` failure, so the
  measured canonical prefix is 220 passed, 1 failed, 0 ignored across 221
  observed outcomes; the two independently reproduced visual reds were not
  reached by that aggregate run and are not being hidden or counted as passes;
- formatting, compilation, registry validation, ignored-test allowlist,
  strict Clippy, content validation, and whitespace checks pass.

Preservation risks called out in executable assertions and guidance include
HP/AP removal, map de-prioritization, renderer forecast parsing, glyph-derived
identity, render-time logging, fixture/name special cases, duplicated entity
menus, normal-world actions leaking into Context, fake enabled production,
resource/task mutation, and test/observer weakening.

The physical Interact key is deliberately not locked by these tests because
D-20/THC-01 currently owns `e` as direct station staffing. The UI plan requires
an explicit owner decision before the input/menu-shell contract is authored.

Status is `NotComplete`: the intentional red contracts are an implementation
handoff, not CandidateGreen, VerifiedGreen, ReviewedGreen, or acceptance.

## UI9 false-green audit and reinforced red handoff — 2026-08-02

The first implementation pass made all three UI9 primary tests and the
655-test canonical inventory pass, but post-green source review rejected the
result. The prior observers proved visible strings without proving their
production cause or structured ownership:

- the Context category matrix injected finished Chronicle and action rows, so
  renaming the Outpost Actions panel could pass while production supplied only
  a generic `Interact` row and no colonist adapter;
- the Supplies test observed the right final text but did not execute its
  prohibition against parsing forecast prose or calculating pressure inside
  UI chrome; and
- the proximity workflow proved one positive movement case but did not reject
  initialization being inferred as movement from a changed position.

The test skill, repository contract, and authoritative testing policy now
require cause → structured projection → final composition evidence, executable
false-green challenges for completion-critical invalid shortcuts, causal
negative cases for edge-triggered behavior, and a status-alignment gate. A
zero-failure aggregate with a required contract still marked `Red` is now
rejected instead of being labeled `VerifiedGreen`.

The strengthened primary tests compile and reproduce the actual gaps:

- `VISUAL-ECON-002`: 0 passed, 1 failed, 0 ignored. The test builds the live
  colony projection, then poisons only legacy flat Supplies/prose inputs. The
  80x24 low case renders `SUP-777[----] [CRITICAL]` with no dawn outlook,
  proving final rendering still reconstructs state from the forbidden seam.
- `VISUAL-CONTEXT-001`: 0 passed, 1 failed, 0 ignored. A real adjacent Basic
  Processing entity produces only generic `Interact`, Travel, Build, Assign
  task, Staff station, Rest, Move, and Wait rows; no production `Inspect Basic
  Processing`, target detail, category action set, or disabled Set Production
  placeholder exists.
- `COLONY-PROXIMITY-001`: 0 passed, 1 failed, 0 ignored. Building the initial
  adjacent Water Source projection with no input increases historical NEARBY
  facts from zero to one, proving position-difference polling mistakes
  initialization for accepted movement.
- the contract-report status-alignment unit matrix passes 3 of 3 cases: it
  rejects required Red drift only for a zero-failure suite, preserves normal
  red-first TDD while failures are observed, and accepts reviewed non-Red
  status.

Canonical red-baseline evidence lists 658 tests. Cargo reaches the strengthened
proximity primary first and observes 220 passed, 1 failed, and 0 ignored before
stopping; the two independently reproduced UI reds are therefore not counted
as aggregate passes or hidden failures. The gate reports 8 steps passed and 3
failed because workspace execution is intentionally red and the measured
prefix cannot equal the full inventory. Formatting, all-target compilation,
registry validation, ignored-test review, strict Clippy, content validation,
and whitespace checks pass.

The three registry records remain accurately `Red`. The current implementation
is preserved as evidence for the next agent; no contract status is promoted.

Pre-handoff validation on 2026-08-03 repaired two observer defects without
changing production behavior. The Context fixture now begins two tiles away
and enters range through the production `a` movement path, so a correct fix for
silent proximity initialization cannot break the Context contract. The
initialization negative and accepted-movement positive use separate app
instances. Context continues to require station/node Chronicle feedback, while
colonists require current nearby/Context projection only, matching the owner
request. Category and status are asserted semantically rather than locking the
`·` punctuation. All three focused tests still compile and fail at the intended
production gaps after these corrections.

## UI9 action-truth and multi-target false-green repair — 2026-08-03

Post-green review confirmed real progress: the Supplies renderer now consumes
structured authoritative gauge facts; proximity feedback is driven by an
accepted player `EntityMoved` result rather than initialization polling; and
station, node, and colonist Context rows now originate from the production
nearby projection. The focused contracts and the 658-test canonical gate were
green before this review.

The review nevertheless rejected completion because the passing observers did
not distinguish an applicable preview from a bound, reachable, executable
action. Production advertised `Interact` as enabled while its binding was
`unbound`, advertised `Enter` for Inspect without a Context reducer, and used
`a` for enabled assignment previews even though `a` remained the normal-world
Move West command. The Context projection also omitted required operational,
renewable, and target detail; two duplicate-named targets produced identical
Inspect labels; and simultaneous station/node entry emitted two Chronicle
facts rather than one deterministic focused fact plus a count.

The plan and testing standard now require explicit applicability,
binding/reachability, and executable-state evidence. Before UI9-D receives an
owner-approved binding and production reducer, Context actions are preview
rows and must remain disabled with truthful reasons. Multi-target coverage now
includes unlike categories for historical fact aggregation and duplicate
display names for player-visible target disambiguation.

The strengthened evidence consists of:

- `COLONY-PROXIMITY-001` primary: a real accepted move enters range of a
  station and node simultaneously, requiring one focused `NEARBY` fact with a
  semantic count and a complete deterministic two-target current projection;
- `VISUAL-CONTEXT-001` primary: each real category must project and finally
  compose its required operational/renewal/target detail, not category/status
  alone;
- `context_detail_and_actions_follow_authoritative_target_state`: construction
  and depleted variants must replace ordinary detail and remove inapplicable
  assignment/production actions, preventing category-prose hardcoding;
- `passive_context_never_advertises_unroutable_actions_as_enabled`: all UI9-C
  previews remain disabled with truthful reasons while Interact is unbound;
  and
- `duplicate_named_nearby_targets_remain_distinguishable_in_context`: two
  same-name stations retain distinct stable identities and player-visible
  Inspect selectors at both supported profiles.

`VISUAL-ECON-002` remains `GreenUnreviewed`. `COLONY-PROXIMITY-001` and
`VISUAL-CONTEXT-001` return to `Red` until the focused failures above are
implemented and independently reviewed. This is `NotComplete`, regardless of
the earlier zero-exit aggregate.

Validated red evidence after the repair:

- the proximity primary compiles and fails with two observed `NEARBY` facts
  (`Water Source` and `Alpha Relay`) where one focused fact plus a count is
  required;
- the Context primary compiles and fails because the operational Basic
  Processing target projects only `Station · Unstaffed` and no operational
  state;
- the action-truth support test compiles and fails on `Interact` projected as
  `enabled: true`, `key_hint: "unbound"`, and no denial reason;
- the state-variant support test compiles and first fails because a
  construction site still inherits enabled `Assign Worker` and disabled `Set
  Production` rows from the operational-station category menu; its later
  depleted-node case also guards the missing depletion detail;
- the duplicate-selector support test compiles and fails on two identical
  `Inspect Basic Processing` labels for distinct target identities;
- `VISUAL-ECON-002` still passes independently; both complete affected test
  targets compile with `--no-run`; contract-registry validation passes 24 of
  24; and strict workspace Clippy passes; and
- the canonical red baseline lists 661 tests, observes 220 passed and the
  intended proximity failure before Cargo stops, and reports 8 gate steps
  passed and 3 failed. The workspace and measured-total/contract-metrics steps
  fail as expected for this red-first handoff; formatting, compilation,
  registry, ignored-test review, inventory, Clippy, content, and whitespace
  pass.

## UI9 independently observable smaller-agent handoff — 2026-08-03

The red handoff was audited with the project-agnostic
`$authoritative-test-pipeline`. The earlier matrix tests allowed the first
station/multi-target panic to hide later node, colonist, depleted, re-entry,
action-truth, and final-composition cases. They are now individually named and
registered. The duplicate-target observer no longer rewards one Inspect and
category action set per nearby target: it requires a complete deterministic
target projection but exactly one focused action set, followed by a visible
focus/count/location cue in both final profiles.

No production behavior was changed by this handoff repair. Both affected test
targets compile with `--no-run`, the seeded registry validation passes, and
every completion-critical row was run independently with `--exact`:

- `COLONY-PROXIMITY-001` station entry: red because `Interact` is `unbound` but
  enabled with no denial reason;
- its Water Source support: independently red for the same false executable
  state;
- its simultaneous station/node support: red with two `NEARBY` facts where one
  deterministic focused fact plus a semantic count is required;
- its leave/re-enter support: green, preserving silent exit and exactly one
  fresh fact on re-entry;
- `VISUAL-CONTEXT-001` station, node, and colonist projection rows:
  independently red for missing operational, renewable, and target detail;
- construction and depleted-node state rows: independently red for inherited
  operational actions and missing depleted detail, respectively;
- station, node, and colonist action-truth rows: independently red because
  unbound/borrowed-key previews are enabled without truthful denial reasons;
- station, node, and colonist final-composition rows: independently red at the
  first 80x24 observation for the same missing operational, renewable, and
  target detail, proving the final screen seam is not hidden behind the
  projection assertions; and
- the duplicate-name row: red because two `Inspect Basic Processing` actions
  flatten both targets into the focused Context action region.

`VISUAL-ECON-002` passes independently and remains accurately
`GreenUnreviewed`; the requirement map and visual matrix were corrected from
stale Red wording. `COLONY-PROXIMITY-001` and `VISUAL-CONTEXT-001` remain
accurately `Red`. UI9-D remains unauthorized pending the Section 17.2 owner
decision.

The canonical red baseline now lists 672 tests. It observes 221 passed, 3
failed, and 0 ignored before workspace execution stops after the `bd_app`
proximity target; all three failures are the registered intentional reds above.
The gate reports 8 steps passed and 3 failed. Formatting, all-target
compilation, registry validation, ignored-test review, inventory, strict
Clippy, content validation, and whitespace pass. Workspace tests,
listed-versus-observed totals, and contract metrics fail as required for this
`NotComplete` red-first handoff.

## Signed candidate/reviewer gate protocol — 2026-08-03

The test protocol now separates implementation evidence from acceptance
authority. A reviewer-created, SHA-256-signed RON manifest names the exact Red
contracts and hashes protected plans, tests, observers, fixtures, authority,
and status ledgers. Candidate mode verifies that seal before and after the
workspace suite, requires the named contracts to remain Red, rejects any
unlisted required Red contract, and reports only `STATUS=CandidateGreen`.
Argument-free canonical mode remains the independent reviewer gate.

Executable protocol evidence:

- four `candidate_handoff` integration tests accept an unchanged signed
  handoff and reject manifest rewriting, protected-file mutation, and omission
  of a gate-required authority file;
- seven `contract_report` unit tests include exact candidate-set acceptance,
  self-promotion rejection, unlisted-Red rejection, and repeatable contract
  arguments while preserving canonical stale-Red detection;
- repository governance verifies the candidate manifest/digest interface,
  mandatory protected authority/status files, exact candidate contract
  reporting, and the `CandidateGreen`-only result; and
- all 49 `bd_test_support` tests pass.

The full canonical gate lists and passes 681 tests with 0 failed and 0 ignored;
all 10 gate steps pass and automated status is `VerifiedGreen`. This protocol
change does not independently accept the current UI9 production diff or
retroactively validate status edits made before the signed handoff existed.
Those still require the production-diff, false-green, final-profile, and owner
reviews required for `ReviewedGreen`.

## UI9-C active-state corrective red handoff — 2026-08-03

The post-green audit found that the Context contract's default unstaffed,
renewable, and idle category rows passed while the active authoritative states
required by UI plan Section 17.3 were not observed. The test author restored
`VISUAL-CONTEXT-001` to `Red`; no production behavior was changed in this
corrective test batch. `COLONY-PROXIMITY-001` remains `GreenUnreviewed`: its
station, node, simultaneous-entry, and exit/re-entry cases each pass
independently.

Eleven completion-critical cases now fail independently for their named
missing behavior:

- staffed station projection omits Mara, Refine Water, and 1/2 progress;
- assigned Water Source projection omits Mara and 1/3 progress;
- assigned colonist projection omits Water Source and 1/3 progress;
- carrying colonist projection reports Idle/No target and omits Basic
  Processing and Raw Water cargo;
- blocked colonist projection reports generic Gathering and omits the blocker;
- a configured `x` Interact binding with no normal-Outpost reducer route is
  falsely enabled; and
- five separately named final-composition cases reproduce the station, node,
  assigned-colonist, carrying-colonist, and blocked-colonist omissions at the
  first 80x24 observation rather than hiding them behind projection failures.

The registry validation and self-protected candidate-gate governance tests
pass. The canonical red baseline lists 692 tests and reaches 658 passed, 11
failed, and 0 ignored before Cargo stops after the `bd_tui` library target; 23
later listed tests are consequently unobserved. Eight gate steps pass and
three fail (workspace tests, listed-versus-observed totals, and contract
metrics), producing `STATUS=NotComplete` as required for the sealed red
handoff.

## UI9-C replacement observer validation — 2026-08-03

Independent review withdrew the preceding eleven-red candidate handoff. Its
active-state fixtures combined or injected states that normal production
updates did not preserve, and final active station/node composition also
required an old walk-by Chronicle line to survive many later worker turns.
Those were observer defects, not implementation requirements.

The replacement observers now establish their decisive states through the
real paused management controls (`c` for direct gathering and `e` for station
recipes) followed by normal worker ticks. The blocked case is produced by a
real `MissingSource` logistics outcome. Geometry-only repositioning happens
only after the decisive domain state is asserted. Ordinary station/node rows
continue to own immediate Chronicle entry feedback; active final-composition
rows own the current Context state after later worker history.

A new adversarial shared-owner seam gives the authoritative nearby target a
distinctive `detail` value while legacy parallel fields contain forbidden
decoys. It fails unless final Context consumes the shared projection instead
of independently rebuilding the same domain detail downstream.

Measured independent results:

- all eleven active projection/action and paired final-composition commands
  were run separately: nine pass and two fail; the separately run shared-owner
  seam is the third failure;
- `staffed_station_context_includes_worker_recipe_and_progress` fails because
  the real `ReadyToRefine 1/2` logistics job projects Basic Processing as
  `Unstaffed` with no worker, recipe, or progress;
- `staffed_station_recipe_progress_survives_final_composition` independently
  fails because the final 80x24 Context remains `Station Unstaffed` while the
  Party panel shows Mara, Refine Water, ReadyToRefine, and 1/2;
- `final_context_consumes_the_shared_detail_projection_once` independently
  fails because final Context shows the forbidden worker/recipe/99/99 decoys
  and omits `Shared Detail Probe`;
- the full `bd_tui` library target reports 111 tests: 108 passed, 3 failed,
  0 ignored;
- the four exact `COLONY-PROXIMITY-001` station/node/focus/re-entry workflows
  each pass independently; and
- `cargo test -p bd_tui --test input_help` reports 23 passed, 0 failed,
  0 ignored.

This is an intentionally red, implementation-ready baseline. The earlier
manifest and digest remain withdrawn. The replacement handoff must use the v2
baseline inventory, v2 manifest, and separately supplied v2 digest; no
candidate may edit these observer or status records.

## UI9-C v2 mixed-source false-green audit — 2026-08-03

The v2 candidate stayed inside its exact write set and its signed candidate
gate reported 694 listed, 694 passed, 0 failed, and 0 ignored. Independent
production-diff review nevertheless rejected CandidateGreen because the
shared-owner shortcut checklist had two `Yes` answers:

- the Context view model parsed formatted `detail` using category and staffing
  prefixes, dropping semantic segments and recovering presentation elsewhere;
- the Context title independently derived `Staffed` from the poisoned legacy
  `worker` field even though authoritative `status` and `detail` both said
  `Unstaffed`.

The original adversarial observer asserted only that `Shared Detail Probe`
appeared and the three conspicuous decoy literals did not. That proved the
decoy strings were hidden, but it did not observe every derivative of the
poisoned worker field or require complete semantic-segment transport. The
resulting final panel visibly combined `Context · Station Staffed` with an
Unstaffed Chronicle/shared projection while the test remained green.

The reviewer preserved the valid production-reachable station staffing work
and split the deficient observer into two independently runnable cases:

- `context_view_model_transports_shared_detail_without_semantic_parsing`
  fails with expected `Station Unstaffed Operational Shared Detail Probe` and
  actual `Operational Shared Detail Probe`, naming the stripped transport
  segments; and
- `final_context_consumes_the_shared_detail_projection_once` fails at 80x24
  with expected title `Context · Station Unstaffed` and actual title
  `Context · Station Staffed`, naming the mixed-source composition.

The full `bd_tui` library target now lists 112 tests and reports 110 passed,
2 failed, and 0 ignored. Preservation checks remain green: `phase6_input`
49/49, `input_help` 23/23, contract registry 24/24, candidate-handoff
governance 4/4, and repository governance 7/7. `VISUAL-CONTEXT-001` remains
accurately `Red`. The v2 prompt, baseline, manifest, and digest are withdrawn;
a new implementation run requires a newly sealed v3 handoff after independent
red validation. Forward testing also found that v2 protected the older prompt
but not its active v2 instruction file. The revised protocol requires a
digest-free active instruction body inside the protected manifest and supplies
the manifest digest separately, preventing that circular-hash seal gap.

## UI9-C v3 scope stop and v4 handoff hardening — 2026-08-03

The v3 candidate correctly repaired complete shared-detail transport and
mixed-source title coherence. Independent reproduction reports each corrective
row green, while the full `bd_tui` library target reports 109 passed and three
failed final-composition rows at 60x20:

- default station clips the `Coming later` Set Production reason;
- staffed station clips `Assign Worker` and later denial content; and
- assigned node clips the complete `Assign Gatherer` row and denial content.

The signed v3 candidate gate was independently reproduced in an isolated build
directory: 9 steps passed, 3 failed; 695 tests listed, 669 passed, 3 failed,
0 ignored; `STATUS=NotComplete`. The fixed three-row compact renderer cannot
retain the complete authoritative detail, required action labels, and truthful
denial reasons through a legal `view_models.rs`-only change. V3 is withdrawn
and its artifacts remain historical evidence.

Review also found an unauthorized untracked execution-handoff document that
the implementation report incorrectly omitted from its claimed delta. A new
candidate-handoff governance test reproduced that false acceptance, and the
version-2 manifest/guard now signs the baseline path and exact production write
set and rejects Git-visible additions/untracked paths, modifications,
deletions, or renames outside it. Candidate-handoff governance now reports 7/7.
Implementation and reviewer reports are pasted into chat; creating a repository
handoff/report is forbidden unless its exact path is explicitly authorized.

The next reviewer-owned v4 handoff permits only the existing shared Context
view-model owner and generic screen composition owner. UI9-D input/menu work,
tests, fixtures, authority, evidence, and status remain protected.
