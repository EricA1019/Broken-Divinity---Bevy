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
