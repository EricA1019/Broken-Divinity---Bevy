# Changelog

All notable changes to Broken Divinity are documented here.

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
