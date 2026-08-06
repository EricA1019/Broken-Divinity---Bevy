# Broken Divinity Foundation MVP Scenario

**Acceptance status:** Partially reopened on 2026-07-25 after a deeper
80x24/60x20 colony UX audit. Simulation, persistence, dungeon, extraction,
economy, and progression evidence from the completed correction gate remains
accepted. Colony spatial safety, viewport visibility, physical worker
activity, semantic presentation, compact completeness, and paused management
must pass [FOUNDATION-TEST-AND-UX-HARDENING-PLAN.md](FOUNDATION-TEST-AND-UX-HARDENING-PLAN.md)
before Foundation MVP acceptance is restored without qualification.

## Purpose

This is the canonical player and test scenario for the first Broken Divinity MVP. It is the contract used by implementation, integration tests, manual terminal smoke tests, save/load validation, and final GDD alignment review.

The scenario proves the foundation loop without requiring procedural dungeon
or shelter-topology generation, overworld travel, raids, events, sanity,
theology-driven mechanics, reputation, or final faction content.

## Canonical scenario

```text
Clean launch
  → arrive at colony/shelter
  → inspect three survivors, tasks, resources, station effects, and forecast
  → inspect one deterministic node for each Foundation source category
  → move a paused build preview independently of the player
  → build one implemented station at the exact selected legal coordinate
  → assign one named survivor to the starter processing station and one named recipe
  → advance Outpost turns while the survivor travels to the matching source
  → gather one raw unit without creating finished output
  → carry the raw unit to the station
  → refine it once and observe the exact finished-resource result
  → advance one day and verify one forecast-matching colony summary
  → save and reload the managed colony
  → enter the fixed dungeon
  → explore the fixed layout and optional healing-item route
  → cross a day boundary and verify one colony transaction
  → encounter one enemy
  → defeat the enemy through multiple ActionIntents
  → discover one healing item
  → pick up the item through an ActionIntent
  → use or store the item
  → move to the dungeon exit
  → explicitly request extraction
  → receive an extraction result
  → return to the colony
  → verify the extracted result is applied exactly once
  → inspect the named shelter return spawn and completed-run history
  → save
  → reload
  → verify the result remains correct
  → begin another run and verify completed extraction history remains visible
  → complete an ordinary defeat
  → save and reload Game Over
  → restart at the shelter spawn with one player and Defeated history
```

## Required foundation content

### Colony

- one fixed shelter map;
- three starter survivors;
- the existing five station types plus one data-defined basic processing
  station with one guaranteed starter instance;
- three data-defined placeholder source/raw/recipe/result chains;
- one deterministic generated node per configured Foundation source;
- station placement;
- survivor assignment;
- one physical gather, carry, and refine cycle;
- storage for extracted results.

Raids and colony events must not run during this scenario.

### Dungeon

- one deterministic, hand-authored map;
- one player start position;
- one encounter location;
- one enemy archetype;
- one healing item;
- one valid extraction point;
- one defeat outcome.

The dungeon must not call the procedural-generation path.

### Progression

The scenario must have typed, extensible support for:

- melee;
- ranged;
- repair;
- medicine.

The scenario must exercise at least one skill improvement and at least two representative virtue-expression hooks. Full virtue balance, perks, theology, and sanity are deferred.

### Factions

Exactly two stable, non-canonical placeholder faction records are loaded from content. They must be identifiable on encounter/entity data, but reputation and diplomacy are not part of the scenario.

## Required action semantics

Foundation gameplay must use the shared action pipeline:

```text
ActionIntent
  → validation
  → cost resolution
  → effect emission
  → mutation
  → result/log emission
```

The scenario must not directly mutate health, position, inventory, skills, virtues, colony resources, or extraction state to manufacture a result.

### Required outcomes

Every foundation action must resolve to a clear result category:

- accepted and completed;
- rejected with a reason;
- combat defeat;
- player defeat;
- extraction completed;
- invalid content/configuration.

Rejected actions must not partially mutate gameplay state.

## Required session transitions

The foundation session has one authoritative state owner and supports:

```text
Colony → Dungeon
Dungeon → Colony
Dungeon → Defeat
Colony → Title/Restart
Defeat → Title/Restart
```

Extraction is explicit. Merely standing on an exit tile may display a prompt, but must not award loot or transition the run without the extraction action.

## Persistence boundary

The MVP save must represent the state needed to resume or verify this scenario:

- save and content versions;
- session phase and outcome;
- run seed and deterministic replay state/origin;
- game day and turn;
- colony resources;
- stations and survivor assignments;
- colony storage;
- current dungeon identity and state;
- player position and pools;
- inventory/equipment;
- skills;
- virtues;
- faction identity;
- extraction result state.

Loading must not reapply extraction, production, skill gain, virtue expression, or other gameplay effects.

## Test ownership map

These are the required acceptance-test responsibilities. Test names may change during implementation, but ownership may not disappear.

| Responsibility | Required test coverage |
|---|---|
| Clean foundation app | `foundation_app_builds_without_terminal` |
| Deferred-system isolation | `foundation_app_does_not_register_deferred_systems` |
| Session transitions | `legal_session_transitions_succeed`, `illegal_session_transitions_are_rejected` |
| Determinism | `same_seed_and_actions_match` |
| Fixed dungeon | `fixed_dungeon_loads_without_procgen` |
| Movement | `movement_respects_fixed_map`, `invalid_movement_is_rejected` |
| Combat | `enemy_can_be_targeted`, `enemy_defeat_emits_result` |
| Loot | `pickup_requires_valid_position`, `extraction_awards_loot_once` |
| Colony return | `colony_to_dungeon_to_colony_scenario_passes` |
| Cross-mode day cycle | `tactical_day_boundary_applies_one_colony_transaction`, `tactical_day_boundary_survives_save_load_without_replay` |
| Recoverable economy | `fixed_shelter_has_every_reachable_gathering_target`, `zero_supply_recovery_survives_save_load`, `forecast_matches_adverse_gathering_matrix` |
| Explicit management | `management_targets_a_named_survivor_and_task`, `staffing_targets_a_named_survivor_and_station`, `management_requires_confirmation_and_cancel_is_atomic` |
| Station truth | `every_buildable_station_has_a_visible_implemented_effect`, `disabled_storage_rejection_is_atomic`, `sixth_station_loads_without_a_rust_branch` |
| Run history/spawn | `new_dungeon_preserves_last_completed_run`, `defeat_restart_uses_the_same_shelter_return_spawn` |
| Progression | `relevant_action_improves_skill`, `virtue_expression_is_emitted` |
| Factions | `two_foundation_factions_load`, `faction_identity_attaches_to_encounter` |
| Persistence | `save_load_round_trips_mvp_state`, `canonical_replay_matches_expected_summary` |
| Player path | `clean_launch_reaches_colony`, `manual_mvp_scenario_completes` |

## GDD alignment record

This scenario implements the following locked decisions:

| Decision | Scenario proof |
|---|---|
| D-01 | Kernel shell and playable foundation are the immediate target |
| D-02 | Bevy-Ratatui/Ratatui is the player-facing runtime |
| D-03 | Exactly two data-driven placeholder factions are loaded |
| D-04 | No sanity behavior is required or active |
| D-05 | Skills improve and actions express virtues |
| D-06 | Theology-driven mechanics are not required |
| D-09 | Existing systems are reused or explicitly deferred |
| D-10 | Enter, explore, fight, loot, extract, and return are playable |
| D-11 | The dungeon is fixed and hand-authored |
| D-12 | Basic colony management is active; raids/events are deferred |
| D-13 | Melee, ranged, repair, and medicine are the foundation skills |
| D-14 | Representative virtue hooks exist before full virtue expansion |
| D-15 | Deterministic run state, fixed dungeon, factions, loot, colony return, and persistence define the foundation |
| D-20 | Deterministic shelter fixtures and one physical gather-carry-refine cycle deepen the basic colony foundation |

## Out of scope

The following must not become hidden dependencies of this scenario:

- procedural dungeon, shelter-topology, or overworld generation;
- overworld travel;
- raids;
- colony events;
- sanity or sanity pressure;
- theology-driven rules;
- faction reputation or diplomacy;
- final faction names and lore;
- multiple dungeon floors or themes;
- full perk trees;
- deep narrative encounters;
- asset-heavy presentation.

## Completion rule

The scenario is complete only when:

1. a headless deterministic test completes it;
2. a clean terminal launch supports it without debug commands;
3. save/load preserves the expected result;
4. deferred-system checks prove excluded systems did not affect it; and
5. the implementation and this scenario still agree with `GDD.md` and D-01 through D-16.

## Acceptance record — 2026-07-24

The Foundation scenario was accepted by the original recovery gate.

- The headless canonical suite passes 14/14.
- Clean terminal extraction, in-dungeon resume, exact-once loot, and defeated
  save/load paths pass.
- The shelter retains its survivors, built station, assignment, resources,
  production day, storage, and run outcome across dungeon travel and loading.
- Normal combat and item-use actions visibly improve representative skills and
  express two virtues.
- The Foundation runtime contains one authoritative player entity and keeps all
  deferred systems outside the active path.

This record does not activate any item listed under **Out of scope**.

## Post-acceptance correction — 2026-07-24

The acceptance status is reopened. A later clean-session audit confirmed that
the original scenario and terminal evidence did not prove:

- station construction deducting the authoritative colony resource;
- paid Foundation dungeon entry through the shared action pipeline;
- warning-free entity cleanup on defeat;
- stable exit/gate guidance during idle frames;
- invalidation-driven terminal rendering;
- readable exact-size 80x24 and 60x20 layouts;
- visible build and inventory navigation;
- accessible exact-once daily-cycle interaction;
- bounded, non-silent buffered input;
- agreement among built-in bindings, shipped configuration, runtime guidance,
  and README;
- a strict warning-free Clippy gate.

The expanded acceptance contract and repair sequence live in
[FOUNDATION-STABILIZATION-PLAN.md](FOUNDATION-STABILIZATION-PLAN.md). Foundation
is not accepted again until that plan's final gate passes.

## Stabilization progress — 2026-07-25

Stabilization Phases 0–7 repaired and independently validated every reopened
area listed above:

- construction and dungeon entry use the authoritative colony resource owner;
- normal defeat, Game Over persistence, and restart are warning-free;
- idle frames do not mutate logs or redraw unchanged terminal output;
- 80x24 and 60x20 layouts expose complete build, inventory, action, and global
  controls;
- Rest reaches one exact daily boundary and the semantic input queue is
  ordered, bounded, and visibly rejects overflow;
- built-in defaults, shipped TOML, configured runtime input, help, footers,
  action panels, and README controls share verified semantic binding ownership;
- strict workspace Clippy is warning-free.

This is progress evidence, not renewed acceptance. Phase 8 must still pass the
complete automated gate and canonical clean-terminal scenario before the
acceptance status at the top of this document changes.

## Renewed acceptance record — 2026-07-25

The expanded Foundation stabilization gate passed.

- The complete workspace suite, canonical scenario, persistence matrix,
  daily-cycle suite, stress suite, input/help suite, content validation,
  strict warning-free Clippy, formatting, compilation, and whitespace gates
  pass.
- Clean 80x24 play covers shelter setup, all five station choices, exact
  construction and entry costs, staffing, Rest, one daily summary, combat,
  Melee/Thumos, item use with Medicine/Temperance, retained-loot extraction,
  exact-once colony storage, save/load, defeat, Game Over persistence, restart,
  one player authority, and terminal restoration.
- The supported 60x20 profile exposes readable shelter, resource, build,
  inventory, combat, and Game Over state. Stable screens produce no repeated
  output, and a live resize produces one correct redraw.
- Missing-save feedback is visible from the title. Automated failure coverage
  also proves atomic resource denials, blocked and unavailable actions,
  corrupt/incompatible saves, invalid content/configuration, bounded input
  overflow, and observable draw failure.
- Save version 7 preserves exact log text and severity without duplicated
  prefixes. Loading does not repeat costs, extraction, production,
  progression, virtue expression, or daily summaries.
- GDD Sections 1–10 and D-01 through D-16 were reconciled with no failed
  Foundation requirement. Every excluded system remains explicitly deferred.

This acceptance closes Foundation stabilization only. It does not authorize
Product P2 or any deferred system.

## Post-stabilization correction — 2026-07-25

A subsequent clean discovery run reopened acceptance after proving:

- a day boundary reached in Tactical mode advances the day but skips colony
  consumption, production, gathering, mood consequences, and summary;
- non-Supplies station choices can exhaust Supplies while recovery through
  gathering is hidden, untargeted, and not protected by an acceptance
  invariant;
- station effects, worker assignments, task selection, completed-run history,
  shelter return placement, controls, and key feedback remain insufficiently
  truthful;
- the fixed dungeon proves plumbing but not yet a meaningful exploration or
  tactical choice.

The proposed correction sequence and adverse acceptance matrix are in
[FOUNDATION-MVP-CORRECTION-PLAN.md](FOUNDATION-MVP-CORRECTION-PLAN.md).
The owner approved execution and the disabled-Storage policy on 2026-07-25.

## Correction acceptance record — 2026-07-25

The owner-authorized correction plan completed and restored acceptance.

- Tactical and Outpost day boundaries now execute the same exact-once colony
  transaction, including consumption, staffed production, explicit gathering,
  mood consequences, and one daily summary.
- Zero Supplies is recoverable through visible, named survivor assignments.
  All three fixed-shelter gathering targets exist and are reachable.
- One validated station catalog owns labels, costs, effects, construction,
  production, save identity, and presentation. Storage remains represented but
  is disabled with `No Foundation effect yet` before payment.
- Colony management explicitly selects a survivor, task, or station and
  requires confirmation; cancel is atomic.
- Active run state and persisted completed-run history are independent.
  Extraction and defeat restart use the named shelter spawn.
- The fixed dungeon supports optional item use, retained-loot extraction,
  multi-action combat, and ordinary defeat.
- The complete workspace and focused acceptance gates pass warning-free.
  Clean isolated 80x24 and 60x20 runs prove management, recovery, daily
  results, save/load, both extraction branches, defeat, restart, completed-run
  history, and terminal restoration.

GDD Sections 1–10, D-01 through D-16, and C-01 through C-08 were reconciled
with no Failed Foundation item. Product P2 and all deferred systems remain
unauthorized.

### Build interaction correction — 2026-07-25

The accepted construction path is an explicit paused interaction:

```text
Normal shelter play
  → B opens station selection
  → number or Up/Down highlights without mutation
  → Enter changes to adjacent-tile placement
  → movement selects the tile without moving the player
  → Enter submits one build action
  → B or Escape cancels either phase without mutation
```

Only physical key Press events may drive these transitions. Release/Repeat
events and queued gameplay cannot close, skip, or advance the modal. This is
covered by `build_interaction_is_a_paused_press_only_state_machine`, supported
profile snapshots, the full workspace gate, and a clean real-terminal build.

## Deep colony UX reopening — 2026-07-25

A later, deeper real-terminal playtest preserved the accepted simulation,
persistence, economy, dungeon, extraction, and progression evidence above but
reopened the colony player-experience and test-trust gates after proving:

- a legal Stove-east and Altar-south sequence can trap the player at the named
  shelter spawn;
- the 40x30 shelter is rendered as a fixed top-left crop, so the player,
  workers, and required targets can disappear at both supported terminal
  profiles;
- 60x20 exposes no initial resource node and 80x24 exposes only one unexplained
  node glyph;
- assigned survivors move only on later actions, can enter target tiles, are
  hidden by later render layers, and produce without physically reaching work;
- survivor, station, and resource glyphs collide and do not follow one complete
  semantic visual language;
- compact station details and decisive feedback are truncated;
- task and station assignment advance time despite the accepted paused
  management contract;
- multiple green tests prove catalog/action existence or substring presence
  while claiming discoverability, visibility, readability, schedule order, or
  stress behavior.

D-18 locks the correction behavior.
[FOUNDATION-TEST-AND-UX-HARDENING-PLAN.md](FOUNDATION-TEST-AND-UX-HARDENING-PLAN.md)
owns behavior and implementation order.
[AUTHORITATIVE-TESTING-STANDARD-AND-MIGRATION-PLAN.md](AUTHORITATIVE-TESTING-STANDARD-AND-MIGRATION-PLAN.md)
owns test policy, evidence sufficiency, metrics, and suite migration. Both
plans must pass before acceptance closes.
Product P2 and all previously deferred systems remain unauthorized.
