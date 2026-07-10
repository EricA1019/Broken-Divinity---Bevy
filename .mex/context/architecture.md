---
name: architecture
description: How the major pieces of this project connect and flow. Load when working on system design, integrations, or understanding how components interact.
triggers:
  - "architecture"
  - "system design"
  - "how does X connect to Y"
  - "integration"
  - "flow"
edges:
  - target: context/stack.md
    condition: when specific technology details are needed
  - target: context/decisions.md
    condition: when understanding why the architecture is structured this way
  - target: context/conventions.md
    condition: when checking code patterns for a given layer
last_updated: 2026-04-06
---

# Architecture

## System Overview

Menu Load Game → `load_game()` / `PendingLoad` / target `OnEnter` restoration →
runtime play resumes with persistent resources and `PlayerSnapshot` bridging states that do not keep a live player entity.

Player input → PendingAction / PendingAbility resources →
  combat / ability systems resolve actions (d100 skill check, damage calc, status infliction) →
  Messages propagate results (AttackEvent, DamageEvent, DeathEvent) →
  downstream systems consume messages (loot, xp grant, sanity tick, noise propagation, raid checks) →
  action budget check → all entities exhausted? → WorldTurn phase (Universal Time Tick System) →
  tick status effects, sanity drain, hunger/thirst, cooldowns, hallucination spawns →
  reset all budgets → AwaitingInput.

The overarching design goal is an **Endless-loop Rimworld survival mode**, balancing roguelike dungeon crawling with persistent colony management, where UI and gameplay loop reinforce continuous play loops.

Two orthogonal state machines drive everything:
- **AppState**: Menu → Colony / Overworld / Dungeon / Combat / GameOver
- **TurnState**: AwaitingInput → PlayerTurn → EnemyTurn → WorldTurn

## Key Components

- **components.rs (Tier 0)** — all shared ECS components: Health, CombatStats, Position, SkillSet, Equipment, RaidExposure, FactionId, Enemy markers, Player marker
- **resources.rs (Tier 0)** — all shared resources: GameLog, GameTime, PendingAction, WorldSeed, dungeon state, economy stockpiles
- **state.rs (Tier 0)** — AppState and TurnState enums
- **combat.rs (Tier 2)** — d100 skill checks, damage formulas, armor durability, AttackEvent pipeline
- **inventory.rs (Tier 2)** — slot-based inventory, item swapping, equipment slots, ammo reserve + clip state
- **ai.rs (Tier 3)** — MVP enemy behaviors (melee chase / ranged shoot), LOS detection, and minimal loud/quiet noise reaction
- **settlement / shelter_map / survivors (Tier 4)** — walkable shelter compound with stations, task assignment, resource production/consumption, and raids
- **dungeon.rs (Tier 4)** — BSP room generation, 3 MVP themes, anomaly placement, loot distribution, Gabriel intro room on the first dungeon run
- **overworld.rs (Tier 4)** — world-graph navigation, Delaunay road network, path-constrained road travel, weather rolls, node discovery
- **input.rs (Tier 5)** — orchestrates all player input, routes to appropriate systems
- **ui.rs (Tier 5)** — egui draw/process split, HUD, panels, modals
- **main.rs (Tier 5)** — system registration organized by AppState, not by source module

## Module Dependency Tiers

```
Tier 5 — Orchestration     input, ui, main
Tier 4 — Meta Systems      dungeon, settlement, shelter_map, survivors, overworld, save_load
Tier 3 — Behaviors          anomalies, sanity, hud, ai, rosters, stealth
Tier 2 — Mechanics          combat, abilities, economy, perks, map, procgen, fov
Tier 1 — Domain Data        skills, equipment, items, dialogue, crafting
Tier 0 — Core               components, states, resources (no cross-imports)
```

Lower tiers **never** import from higher tiers. Cross-tier communication uses Resources or Messages.

## External Dependencies

- **Bevy 0.18.1** — ECS framework, 2D rendering, input, audio; Messages API (not Events); `default-features = false, features = ["2d"]`
- **bevy_egui 0.39.1** — immediate-mode UI for panels, HUD, menus; draw in EguiPrimaryContextPass
- **bevy_ecs_tilemap 0.18.1** — GPU-chunked tilemap rendering (entity-per-tile) for all grid maps
- **pathfinding 4.15.0** — A*, Dijkstra for AI and player pathfinding
- **spade 2.15.1** — Delaunay triangulation for overworld road networks
- **fastnoise-lite 1.1.1** — noise generation for terrain variation, anomaly density
- **seahash 4.1.0** — seed hashing for procgen seed derivation
- **serde 1.0.228 / ron 0.12.1** — save/load serialization and RON data file loading
- **rand 0.10.0 / rand_chacha 0.10.0** — deterministic RNG via ChaCha8Rng with per-system derived seeds
- **bevy-inspector-egui 0.36.0** (dev-only) — runtime ECS inspector

## Documentation Layer

- **`docs/GDD.md`** — Single comprehensive game design document (overview, pillars, game loop, all systems)
- **`docs/lore/`** — Canonical lore source (10 topic files). Copilot lore skill points here for deep detail.
  - Reading order: the-sundering → the-world-now → factions → thaumaturgy → species → sanity → dungeon-themes → naming-conventions → tone-guide
  - Key: Faction system uses proc-gen archetypes (Caves of Qud-style), not monolithic factions
- **`docs/gameplay/`** — Implementation-ready gameplay mechanics, phase-tagged (MVP / Phase 2 / Phase 3).
  - Reading order: phase-roadmap → combat → colony → overworld → procgen → progression
  - Key: phase-roadmap.md is the master scope document — it defines what ships in MVP and what's deferred
- **`docs/tech/ui-design.md`** — Finalized UI design lockdown. Rendering pipeline, frameworks (`bevy_egui`, `bevy_ecs_tilemap`), component architecture.
- **`docs/tech/architecture.md`** — Technical architecture reference: dependency table, Bevy feature config, Cargo.toml skeleton, module architecture, rendering pipeline, procgen strategy, and dev workflow.
- **`docs/ui/`** — Complete UI design spec (4 files + README). ASCII wireframes, element inventories, keybind tables, sanity distortion tiers, phase tags.
  - Reading order: README (foundations, palette, keybinds, sanity distortion) → ingame → shelter → overworld → menus
  - Key: Minimal HUD philosophy — map is king, panels toggle on demand, vitals bar always visible
  - Key: Shelter uses tabbed single egui window (7 tabs), not separate panels
  - Key: Sanity distortion escalates by exposure tier, but MVP keeps the effect set narrower than the long-term Phase 2/3 vision

## What Does NOT Exist Here

- No networking or multiplayer — single-player only
- No 3D rendering — 2D orthographic with hybrid ASCII glyphs + 16×16 sprites
- No ECS plugin architecture — all systems registered directly in main.rs
- No asset pipeline beyond RON files and sprite sheets — no procedural mesh or shader generation
- No external database or file server — all persistence is local serde save files
