# Changelog

All notable changes to Broken Divinity are documented here.

## [Unreleased] — 2026-08-09

### Developer CLI Console (`bd_console` crate)

#### Added
- **`bd_console` crate** — quakelike developer console toggled with backtick (`), renders as a bottom-40% terminal overlay
- **20+ debug commands**: supplies, materials, faith, plants (with `s`/`m`/`f`/`p` aliases), day, turn, skip_day, event, end_event, kill_all, heal, god on/off, survivor, task, spawn, goto, shelter, blueprints, events, stats, help, clear
- **Tab completion** against 24 known command names with common-prefix completion and multi-match suggestions
- **History search** (Up/Down) filtered by current buffer prefix
- **Welcome message** on console open orienting users
- **Signal-driven dispatch**: resource commands mutate `ColonyResources` directly; event/transition/spawn commands emit standard `EventTrigger`/`TransitionIntent`/`PoolDeltaRequested` messages
- **Color-coded output**: ERROR=red, OK=green, prompt=yellow
- **Entity completeness**: console-spawned survivors and entities include `EntityScope`, `PersistentEntity`, and blueprint `Statuses`
- **`GodMode` component** — console `god on/off` inserts/removes a marker component for invincibility

#### Changed
- **`BlueprintCatalog::blueprint_ids()`** — new method exposing all blueprint IDs
- **`EventRegistry::all_ids()`** — new method exposing all registered event IDs
- **`pools.rs` doc comment** — clarified that entity pool mutation goes through `PoolDeltaRequested`; colony resources use a separate direct-mutation model
- **`observe_player_defeat`** — documented the 3 reasons it bypasses `TransitionIntent` (entity despawn ordering, unconditional death, same-frame GameOver)

#### Fixed
- **Console resource commands were silent no-ops** — `supplies`/`materials`/`faith`/`plants` targeted the player entity (which lacks those pools) via `PoolDeltaRequested`, silently dropped by `resolve_pool_deltas`. Now mutates `ColonyResources.pools` directly.
- **KeyEventKind::Press filter** — console no longer double-processes key events from Release/Repeat

#### Tests
- 99 tests in `bd_console` across parser, input, dispatch, render, and integration
- Full workspace green (only pre-existing `contract_registry` seeded_registry failure)

## [Unreleased] — 2026-07-31

### Foundation Colony UX Hardening and UI Guide Evidence

#### Added
- **InvalidSelection visual token** (`!`, Danger style): invalid build previews
  differ from valid ones without relying on color alone
- **Blocked worker Danger style**: the blocked worker glyph resolves through the
  warning/danger style instead of the ally style
- **DailySummary display lines**: authoritative day deltas (Supplies, Materials,
  Plants, Faith, Food) projected as one readable summary line
- **PendingAssignmentFeedback**: assignment confirmation shows the named action
  label back to the player
- **SaveAvailability resource**: the title screen knows whether a manual slot
  exists so an unavailable Load is explained
- **UI red-first guide suite** (`bd_tui/src/ui_development_contract_tests.rs`):
  cell-level glyph/style/geometry observations rendered at both 80x24 and 60x20
- **Persistence checkpoint matrix** (`bd_app/tests/persistence_checkpoint_matrix.rs`,
  PERSIST-MATRIX-001): five projections round-trip the full normalized
  fingerprint through both the checkpoint and atomic manual-slot paths
- **Input press/repeat/release policy matrix**
  (`bd_app/tests/press_repeat_release_policy.rs`, INPUT-POLICY-001): only a
  physical Press mutates; Repeat and Release are inert for outpost controls
- **Foundation UI Improvement Plan** (`docs/FOUNDATION-UI-IMPROVEMENT-PLAN.md`):
  owner-approved red-first Ratatui presentation sequence

#### Changed
- **Contract registry** now owns 88 contracts; requirement-map evidence statuses
  reconciled (no contract is Red; automated layers green, PTY/owner review open)
- **Application boundary tests** cover missing/corrupt title load and single
  quit-key exit through the production app wiring

#### Tests
- 642 automated tests green across the workspace gate

## [Unreleased] — 2026-07-25/30

### Foundation Recovery and Colony Production Loop

#### Added
- Canonical Foundation acceptance harness with reproducible evidence totals
- Foundation entity-scope enforcement (run-persistent vs dungeon-transient)
  with an exact-one-player authority
- Complete Foundation persistence: manual slot, version/content guards, and
  atomic temp-file replacement
- Unified Foundation action pipeline with typed denial evidence and signal
  trace
- Exact-once colony day cycle and unified terminal commands/guidance
- Data-driven Foundation content startup with runtime action-link validation
- Foundational colony production loop: station-backed gather/carry/refine
  logistics over a data-defined pilot recipe chain
- Direct gathering with a data-defined three-tick work rule and an explicit
  zero-Supplies recovery path

#### Changed
- Project authority made repository-local: the repository `GDD.md`,
  `Kernel.md`, and the locked decisions are the sole current references
- Progression and faction semantics enforced: representative virtue hooks and
  two data-driven placeholder factions

#### Tests
- Machine-readable contract registry (`testing/foundation-contracts.ron`)
  owns one primary test per required Foundation contract

## [Unreleased] — 2026-07-23/24

### Colony Overhaul (Phases 0-2)

#### Added
- **BuildGhostState** resource: tracks ghost cursor position, active state, and selected station type for placement preview
- **BuildMenuState** resource: station selection popup with 5 buildable types, arrow key navigation, Enter to confirm
- **Build menu UI**: visual overlay showing station list with costs, highlight cursor, keyboard shortcuts
- **Build ghost preview**: lowercase ghost glyph (`f`/`a`/`w`/`b`/`s`) on shelter map during placement mode, rendered at highest priority (layer 100)
- **AssignToStation** message: new message pipeline for assigning survivors to stations
- **process_station_assignments**: system that reads AssignToStation messages and sets `SurvivorTask::AssignedTo(station_id)`
- **Station worker gating**: `process_production` now checks `HashSet<Entity>` of staffed stations — only assigned survivors trigger production
- **Assigned survivor movement**: survivors with `AssignedTo` task now walk toward their station each turn
- **Day-change gating**: `process_survivor_gathering` now only fires once per day (was running every frame)
- **Food consolidation**: `consume_shelter_resources` now checks colony-level `ColonyResources` instead of non-existent entity-level Supplies — fixes starvation bug where survivors always got -10 mood
- **Gabriel trigger fix**: now checks `StationType::Altar` specifically instead of matching any station
- **Station placement collision**: `Requirement::TileVacant` added to build validation — prevents building on occupied tiles
- **`e` key**: assigns nearest survivor to nearest station in Outpost mode
- **Build menu integration**: `b` opens menu → Enter selects → ghost appears → arrows position → Enter places
- **Resource node rendering**: Trees (`T`), Water Source (`W`), Wild Plants (`P`) rendered on shelter map at spawn

#### Fixed
- **Esc/q during build menu**: was quitting the game — now cancels the menu
- **Right/Left arrows during menu**: were moving the player — now no-oped
- **Register_assign_task_action**: removed broken action that always set survivors to Idle instead of AssignedTo
- **BuildModeState/TaskMenuState**: removed dead resources that were never queried
- **Production constants**: removed unused `STOVE_WILDPLANTS_COST`, `STOVE_SUPPLIES_OUTPUT`, `WORKSHOP_MATERIALS_COST`, `WORKSHOP_SUPPLIES_OUTPUT`
- **OutpostState.resources**: removed dead supply pool (duplicated by ColonyResources)
- **Event input gating**: re-merged into `map_input_to_intents` to fix duplicate key processing; removed `has_drained` param to stay within Bevy's 16-param system limit

#### Changed
- **Two-phase build flow**: `b` → menu with arrow/number keys → Enter/1-5 to confirm → ghost placement mode → Enter to place
- **Arrow keys in ghost mode**: reset cursor to `player_pos` before each move (prevents ghost from drifting away from player)
- **HelpViewModel**: added `e` key binding description
- **a-key cycle**: explicit `AssignedTo(_)` → `assign_idle` match arm
- **Title screen**: pressing `b` queues build mode activation for the frame after player spawns

#### Tests
- Added: `build_ghost_state_default_is_inactive`
- Added: `build_ghost_state_can_be_activated`
- Added: `build_ghost_deactivate_clears_selection`
- Added: `survivors_keep_mood_when_colony_has_food`
- Added: `survivors_starve_when_colony_has_no_food`
- Removed: `assign_task_action_has_correct_id`
- Removed: `outpost_resources_use_pool_like_system` (rewritten as `colony_resources_have_default_supplies`)

## [a7ff4ba] — Phases 5-8 (previous session)
## [fd1e73f] — Phase 4 — color thresholds, panel colors, title screen
## [66f8676] — Phase 3-D — help screen with keybindings overlay
## [36d0137] — Phase 3 UX — turn counter, travel hints, enemy glyphs, stats layout
## [0b22fc1] — Phase 2 UX — combat feedback loop
## [e1daf23] — Phase 1 UX — clean TUI, full AP startup, quit key handling
