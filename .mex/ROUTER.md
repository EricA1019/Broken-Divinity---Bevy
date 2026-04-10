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
last_updated: 2026-04-10
---

# Session Bootstrap

If `AGENTS.md` exists, read it now. If it does not, use this router plus `.mex/context/*` as the session bootstrap source.

Then read this file fully before doing anything else in this session.

## Current Project State

**Working:**
- Project scaffold and build system (Cargo workspace compiles)
- Cargo project initialized with `Cargo.toml`, `src/main.rs`, `src/lib.rs`, `src/core/state.rs`, and initial `src/game/` module skeletons
- Copilot skill suite: 20+ skills covering all game domains (lore, combat, AI, colony, procgen, ECS patterns, etc.)
- Local `graphify/` checkout from `safishamsi/graphify` is installed in editable mode as `graphifyy 0.4.1`, and the Copilot CLI skill is registered at `~/.copilot/skills/graphify/SKILL.md`
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
- Current repo health: `cargo build`, `cargo test`, and `cargo clippy -p broken_divinity -- -W clippy::all` all pass; 115 tests currently pass

**Pending / Not yet built:**
- Travel-time sanity pressure is defined in weather data but not yet applied during travel
- Survivor persistence across save/load is still missing; colony re-entry currently respawns the starting trio
- Faction data is generated and saved but still has minimal gameplay integration
- Rendering pipeline (hybrid ASCII glyphs + sprite rendering) is still incomplete
- RON data files (rosters.ron, dialogue trees) are still pending
- Research table progression, colony construction/task-assignment UI, tactical raid flow, and live Bevy MCP screenshot smoke validation remain the main MVP gaps
- Late-tier perk behaviors that depend on missing mechanics remain partial (`CleaveStrike`, `Unstoppable`)

- BRP (Bevy Remote Protocol) is live under `--features dev`: ~45 types have `#[derive(Reflect)]`, 4 plugin files register types, HTTP transport on port 15702. All 19 BRP methods work: entity queries, component reads/writes, resource reads/writes, schema introspection, state transitions, entity spawning/despawning. Tested end-to-end across Menu, Colony, and Dungeon states.
- Three pre-existing query conflicts fixed (only surfaced at runtime, not in unit tests): `advance_turn_phase` in turn.rs, `gabriel_turn` in gabriel.rs, `enemy_ai_turn` in ai.rs

**Known issues:**
- Package name in Cargo.toml may need adjustment when workspace is initialized
- The documented 5-tier dependency graph has live violations in `src/core/items.rs`, `src/core/movement.rs`, and `src/core/save.rs`
- Some UI registrations in `src/main.rs` still run without explicit AppState gating (`gamelog_panel`, parts of `gameover`)
- 27 clippy warnings remain (mostly `too_many_arguments` and `type_complexity`, plus one `explicit_counter_loop`), and test-only `unused_must_use` warnings remain in `src/game/colony/stations.rs`

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
