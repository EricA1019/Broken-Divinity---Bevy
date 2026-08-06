---
name: router
description: Session bootstrap and navigation hub. Read at the start of every session before any task. Contains project state, routing table, and behavioural contract.
edges:
  - target: context/architecture.md
    condition: when working on system design, integrations, or understanding how components connect
  - target: context/stack.md
    condition: when working with specific technologies, libraries, or making tech decisions
  - target: context/conventions.md
    condition: when writing new code, reviewing code, or unsure about project patterns
  - target: context/decisions.md
    condition: when making architectural choices or understanding why something is built a certain way
  - target: context/setup.md
    condition: when setting up the dev environment or running the project for the first time
  - target: patterns/INDEX.md
    condition: when starting a task — check the pattern index for a matching pattern file
last_updated: 2026-08-06
---

# Session Bootstrap

If `AGENTS.md` exists, read it now. If it does not, use this router plus the files under `.mex/context/` as the session bootstrap source.

Then read this file fully before doing anything else in this session.

## Current Project State

> **Note (2026-08-06):** Much of the detail below describes the legacy
> Bevy 0.14 + egui prototype (now archived in `legacy/src/`). The active
> Bevy 0.18 + Ratatui workspace lives in `crates/`. See `AGENTS.md`,
> `Kernel.md`, and `docs/GAME-STATUS-2026-08-01.md` for current state.
> This router has not yet been fully rewritten for the new architecture.

**Working:**
- Project scaffold and build system (Cargo workspace compiles)
- Cargo workspace with 5 crates (`bd_app`, `bd_core`, `bd_tui`, `bd_data`, `bd_test_support`) under `crates/`
- Legacy Bevy 0.14 + egui prototype archived to `legacy/src/` (2026-08-06) — does not compile under the current workspace
- Copilot skill suite: 35+ skills covering all game domains (lore, combat, AI, colony, procgen, ECS patterns, etc.) — audited Apr 2026: broken refs fixed (procgen, colony-management, enemy-roster), overlap tightened (change-docs↔conventional-commit), progressive disclosure applied to top 5 bloated skills (graphify, gameplay-mechanics, rust-bevy-patterns, node-structure, procgen)
- Copilot CLI MCP servers: 6 configured locally for this environment (bevy-brp, pantheon, sequential-thinking, memory, mermaid, second-brain)
- Local `graphify/` checkout is installed in editable mode as `graphifyy 0.4.1`, and the Copilot CLI graphify skill is registered in the local assistant environment
- MEX scaffold populated with architecture, stack, conventions, decisions, setup context
- Game design fully specified across skill files (mechanics, formulas, enemy roster, lore bible)
- **`docs/` directory**: GDD.md + 10 lore topic files in `docs/lore/` (canonical worldbuilding source)
- **`docs/gameplay/` directory**: 7 files — phase-roadmap, combat, colony, overworld, procgen, progression (implementation-ready mechanics, MVP/Phase 2/Phase 3 tagged)
- **`docs/gameplay/phase-roadmap.md`** is aligned again with the detailed gameplay docs: path-constrained overworld travel, Anomaly Storm weather, Gabriel in the first dungeon, sanity resets at shelter, limited MVP research/perks only
- **`docs/dev-plan.md`** exists and breaks MVP delivery into 9 vertical slices; Gabriel intro and save/load are now planned before late polish
- **`docs/ui/` directory**: 4 files + README — complete UI design spec with ASCII wireframes, element inventories, keybinds, sanity distortion, phase tags (ingame, shelter, overworld, menus)
- **`docs/tech/ui-design.md`**: Finalized UI design lockdown (framework choices and rendering pipeline).
- **`docs/tech/architecture.md`** now exists and covers dependency choices, Bevy feature config, module architecture, rendering pipeline, procgen strategy, and dev workflow
- Save/load snapshot layer now persists nested player, colony, overworld, dungeon, and lore state in JSON, with load-time compatibility for the old flat save shape and a queued `PendingLoad` handoff resource
- Load Game now restores from the main menu into Colony, Overworld, or Dungeon via target-state entry hooks, and Save & Quit returns cleanly to Menu through a dedicated request handler
- Runtime player continuity across Colony, Overworld, and Dungeon is now bridged by `PlayerSnapshot`, so the player no longer resets when the current scene despawns the entity
- Runtime world seed now flows into colony shelter generation, initial dungeon generation, and overworld setup/travel when `WorldSeed` exists, with deterministic literal fallbacks preserved when it does not
- Dungeon entry now preserves origin node metadata and uses deterministic per-site seeds, so dungeon setup can stage node-specific content and return to the same site consistently across save/load
- Melee and ranged combat now grant skill XP, queue threshold perk unlocks, and surface those unlocks through an egui popup that blocks input until claimed
- Consumable items (e.g. Medicine) can now be used from the inventory panel during dungeon exploration — heals HP capped at max, removes one from the stack, consumes 1 AP, and logs to GameLog
- Passive perk wiring now affects melee damage, ranged accuracy, reload tempo, incoming armor, low-health recovery, and sanity resistance without mutating the save schema again
- Sanity thresholds now affect runtime behavior: stressed/shaken penalties feed into combat checks, hallucinations can spawn as fake targetable enemies, breaking can override movement direction, and colony re-entry clears exposure from runtime snapshots
- The closest overworld dungeon is now tagged as Gabriel's intro site, floor 2 stages a scripted Gabriel dialogue encounter, and accepting the warning persists Gabriel as a ghost companion across later dungeon floors and saves
- All UI panels now follow the draw/process split convention: draw systems run in `EguiPrimaryContextPass` (read-only, write to action resource), process systems run in `Update` (consume action, mutate world). Migrated: menu, overworld_panel, colony_panel, gameover, perk_choice_panel, inventory_panel, gabriel_dialogue_panel. gamelog_panel and journal_panel are pure display (no mutations).
- Dungeon cleanup now properly resets all stale combat resources (`BumpAttackTarget`, `CombatRng`, `ShootTarget`, `PlayerSnapshot`), resets `TurnPhase` on exit, and has a 3-frame safety cap on `EnemyTurn` to prevent turn-phase lock
- Gabriel dialogue fallback no longer spawns duplicate entities; logs a warning instead
- `SelectedDestination` is preserved on load when travel state exists; `PlayerSnapshot` is cleared after consumption in dungeon setup
- Faction system currently uses 3 hardcoded anchor factions plus seeded proc-gen factions (Caves of Qud-style archetype approach)
- UI design lockdown complete, ready for incremental slice implementation
- Overworld encounters now fire during travel: `process_travel_day` calls `roll_encounter()` with deterministic per-day RNG; Hostile encounters cost 1 extra food, Scavenge encounters add 1-2 food/water to `ShelterResources`; `TravelState.encounters_seen` tracks count
- Travel-time sanity pressure now applies weather-based exposure via `RaidExposure` during travel days
- Colony runtime/save handoff now persists survivor and station state across Colony ↔ Overworld/Dungeon transitions: `PendingSurvivorLoad` and `PendingStationLoad` cache live shelter entities on exit, restore them on re-entry, and fall back into save snapshots while the player is away
- Colony and overworld simulation no longer advance every `Update` frame: `ColonyTickTimer` and `TravelDayTimer` now pace shelter ticks and travel days, and `GameTime.turn` advances across those real-time loops as the shared log/interval clock
- Returning to Menu now fully clears transient pacing/raid state: `ColonyTickTimer`, `TravelDayTimer`, `RaidChance`, and `ActiveRaid` reset alongside the rest of run-scoped resources
- Shelter readability is restored with a procedural placeholder atlas: colony and dungeon tilemaps render again, shelter gates/stations have visible markers, and build placement now anchors to valid floor tiles instead of hardcoded coordinates
- Starter survivors now spawn around the entrance/player on fresh colony setup, matching the survivor panel with the visible shelter state on frame one
- Travel with zero food/water remains allowed, but each day now applies real HP/exposure attrition and the overworld UI surfaces the risk clearly before and during travel
- Raid pacing is less hostile: first raids respect a grace window and colony simulation pauses while a raid is unresolved
- Floor-1 dungeon combat is no longer tuned around raw skill-as-damage: melee uses bounded skill damage, enemies use their per-definition melee profiles, floor 1 excludes elite table entries, and first-floor spawns are capped at 8 enemies total
- Fatal enemy hits now queue GameOver immediately and stop the rest of the enemy turn, preventing post-death extra shots/hits and duplicate death summaries
- BRP smoke evidence now covers restored colony readability and live floor-1 dungeon population limits (8 enemies observed after fresh arrival)
- Colony economy loop complete: stations produce resources (gated on workers), survivors consume food/water, player assigns survivors via UI, research table provides tech upgrades
- Research table with 4 projects (`ImprovedCooking`, `BetterFilters`, `ScrapArmor`, `MedicalTraining`) gated by `CompletedResearch` resource
- Interactive station assignment UI: assign/unassign survivors, build new stations with resource costs
- Raid flow now distinguishes staying home vs leaving: colony raids still start in `RaidPhase::Planning` with the modal, narrated at-home defense remains the placeholder path, and leaving the shelter during an active raid now auto-resolves it into a `PendingRaidReport` that is delivered on colony return
- BRP smoke now covers the away-raid split: abandoning a live colony raid queued a deferred report, preserved a modified workbench assignment/tier across Colony → Overworld → Colony, and returned with the expected survivor casualty and log summary
- Sprint mechanic: Shift+direction for 2-tile movement with 3-turn cooldown, persisted in save
- AwaitingInput→PlayerTurn transition fixed in `grid_movement` and `handle_shoot_input` (was previously a no-op — all PlayerTurn-gated systems now actually run)
- Loot ID validation: `reinforced_vest`→`scrap_vest`, `combat_rifle`→`military_rifle` fixed; `test_all_loot_ids_valid` test added
- BSP spawn point validation with BFS fallback to nearest walkable tile
- Status effect logging wired: bleed/stun damage now surfaces in GameLog
- Help overlay (F1/? toggle) with context-sensitive controls per AppState for tester onboarding
- Emergency escape hatches: Esc→Menu from any state, universal death detection (HP≤0 in all states), stuck hint after 20 idle turns in dungeon
- GameOver screen enhanced with "New Game" (permadeath) and "Return to Menu" (keep save) options
- Zero production `.unwrap()` calls — all replaced with graceful alternatives
- Testing gate script at `scripts/test-gate.sh`: debug build + tests + clippy + release build
- Build-artifact hygiene is now explicit: `scripts/prune-build-artifacts.sh` prunes rebuildable Cargo dependency, incremental, build, fingerprint, examples, generated-doc, test-artifact, and cxxbridge outputs while keeping the current top-level binaries by default
- **MVP STATUS: GATE PASSED** — 180 tests; debug, clippy, and release gates pass with the current known-warning backlog
- Current repo health: `cargo build`, `cargo test`, `cargo clippy`, and `cargo build --release` all pass; 180 tests

**Pending / Not yet built (post-MVP):**
- Faction data is generated and saved but still has minimal gameplay integration
- Rendering pipeline (hybrid ASCII glyphs + sprite rendering) is still incomplete
- RON data files (rosters.ron, dialogue trees) are still pending
- Live Bevy MCP screenshot smoke validation not yet automated
- Late-tier perk behaviors that depend on missing mechanics remain partial (`CleaveStrike`, `Unstoppable`)

- BRP (Bevy Remote Protocol) is live under `--features dev`: ~45 types have `#[derive(Reflect)]`, 4 plugin files register types, HTTP transport on port 15702. All 19 BRP methods work: entity queries, component reads/writes, resource reads/writes, schema introspection, state transitions, entity spawning/despawning. Tested end-to-end across Menu, Colony, and Dungeon states.
- Three pre-existing query conflicts fixed (only surfaced at runtime, not in unit tests): `advance_turn_phase` in turn.rs, `gabriel_turn` in gabriel.rs, `enemy_ai_turn` in ai.rs

**Known issues:**
- Package name in Cargo.toml may need adjustment when workspace is initialized
- The documented 5-tier dependency graph violations in the legacy `src/` codebase are moot — that code is archived in `legacy/src/`
- System registration is now in `crates/bd_core/src/lib.rs` (`BdCorePlugin` / `BdFoundationPlugin`) with explicit `BdSet` ordering
- 27 clippy warnings remain (mostly `too_many_arguments` and `type_complexity` — known Bevy-isms, allowed in gate script)

## Routing Table

Load the relevant file based on the current task. Always load `context/architecture.md` first if not already in context this session.

| Task type | Load |
|-----------|------|
| Understanding how the system works | `context/architecture.md` |
| Working with a specific technology | `context/stack.md` |
| Writing or reviewing code | `context/conventions.md` |
| Making a design decision | `context/decisions.md` |
| Setting up or running the project | `context/setup.md` |
| Any specific task | Check `patterns/INDEX.md` for a matching pattern |
| Exploring codebase architecture or relationships | `graphify-out/graph.html` (interactive) or `graphify-out/graph.json` (queryable) |

## Behavioural Contract

For every task, follow this loop:

1. **CONTEXT** — Load the relevant context file(s) from the routing table above. Check `patterns/INDEX.md` for a matching pattern. If one exists, follow it. Narrate what you load: "Loading architecture context..."
2. **BUILD** — Do the work. If a pattern exists, follow its Steps. If you are about to deviate from an established pattern, say so before writing any code — state the deviation and why.
3. **VERIFY** — Load `context/conventions.md` and run the Verify Checklist item by item. State each item and whether the output passes. Do not summarise — enumerate explicitly.
4. **DEBUG** — If verification fails or something breaks, check `patterns/INDEX.md` for a debug pattern. Follow it. Fix the issue and re-run VERIFY.
5. **GROW** — After completing the task:
   - If no pattern exists for this task type, create one in `patterns/` using the format in `patterns/README.md`. Add it to `patterns/INDEX.md`. Flag it: "Created `patterns/<name>.md` from this session."
   - If a pattern exists but you deviated from it or discovered a new gotcha, update it with what you learned.
   - If any `context/` file is now out of date because of this work, update it surgically — do not rewrite entire files.
   - Update the "Current Project State" section above if the work was significant.
