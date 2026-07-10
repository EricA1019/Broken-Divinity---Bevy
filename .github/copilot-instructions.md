# Broken Divinity — Copilot Instructions

A post-apocalyptic religious horror roguelike RPG in Rust/Bevy 0.18, featuring hybrid ASCII/sprite rendering, d100 skill-based combat, dual sanity systems, and a walkable Rimworld-style shelter colony.

## Session Start

**Read `.mex/ROUTER.md` before every session.** It contains current project state, a routing table for context files, and a behavioural contract. For any task, also check `.mex/patterns/INDEX.md` for a matching pattern file before writing code.

## Commands

```sh
cargo build -p broken_divinity               # debug build
cargo run -p broken_divinity                 # run game
cargo run -p broken_divinity --features dev  # run with dynamic linking (faster iteration)
cargo test -p broken_divinity                # all tests
cargo test -p broken_divinity <test_name>    # single test
cargo clippy -p broken_divinity -- -W clippy::all  # lint
cargo build -p broken_divinity --release     # optimized build
```

## Architecture

Two orthogonal state machines drive everything:
- **`AppState`**: `Menu → Colony / Overworld / Dungeon / Combat / GameOver`
- **`TurnState`**: `AwaitingInput → PlayerTurn → EnemyTurn → WorldTurn`

All code is organized into a strict **5-tier dependency graph** — lower tiers never import higher:

```
Tier 5 — Orchestration   input, ui, main.rs
Tier 4 — Meta Systems    dungeon, settlement, shelter_map, survivors, overworld, save_load
Tier 3 — Behaviors       anomalies, sanity, hud, ai, rosters, stealth
Tier 2 — Mechanics       combat, abilities, economy, perks, map, procgen, fov
Tier 1 — Domain Data     skills, equipment, items, dialogue, crafting
Tier 0 — Core            components.rs, resources.rs, state.rs  (no cross-imports)
```

Cross-tier communication uses **Resources or Messages only** — never direct function calls upward.

**System registration** lives entirely in `main.rs`, grouped by `AppState` lifecycle — not by source module.

**Data flow:** Player input → `PendingAction`/`PendingAbility` resources → combat/ability systems (d100 check, damage calc) → Messages propagate results → downstream systems consume → action budget exhausted → `WorldTurn` ticks status effects/sanity/hunger → reset budgets → `AwaitingInput`.

**Save/load:** Snapshot layer (`PendingLoad` resource) restores nested player, colony, overworld, and dungeon state from JSON. All save fields require `#[serde(default)]` for backward compatibility. `PlayerSnapshot` bridges states that despawn the player entity.

## Non-Negotiables

- **Never unwrap queries** — always use `let Ok(...) = query.single() else { return; };` or `let Ok(data) = query.get(entity) else { continue; };`
- **Never use `bevy::prelude::Event`** — this project uses Bevy 0.18's Messages API: `#[derive(Message)]`, `MessageWriter<T>`, `Messages<T>`, `app.add_message::<T>()`
- **Always gate systems** to `AppState` and/or `TurnState` — no bare `.add_systems(Update, sys)`
- **Respect the tier hierarchy** — never import from a higher module tier
- **All shared components go in `src/core/components.rs` (Tier 0)** — never define components locally in higher-tier modules; same for shared resources in `src/core/resources.rs`
- **New save fields require `#[serde(default)]`** — ensures backward-compatible save files

## Key Conventions

### Naming
- Systems: `verb_noun` (`resolve_attack_events`, `tick_survivor_needs`)
- Components/Resources: `PascalCase` nouns (`Health`, `CombatStats`, `WorldSeed`)
- Messages: `PascalCase` + `Event` suffix (`AttackEvent`, `DamageEvent`)
- Constants: `SCREAMING_SNAKE_CASE`

### Graceful Query Failure (mandatory everywhere)
```rust
// ✅ Always
let Ok((mut hp, stats)) = player_query.single_mut() else { return; };
let Ok(data) = query.get(entity) else { continue; };

// ❌ Never
let (hp, stats) = query.single_mut().unwrap();
```

### Messages API (not Events)
```rust
#[derive(Message)]
pub struct AttackEvent { pub attacker: Entity, pub defender: Entity }

// Register in main.rs
app.add_message::<AttackEvent>();

// Send
fn attack(mut events: MessageWriter<AttackEvent>) { events.write(AttackEvent { .. }); }

// Receive
fn on_attack(messages: Messages<AttackEvent>) { for event in messages.drain() { .. } }
```

### egui Draw/Process Split
- **Draw systems** run in `EguiPrimaryContextPass` — read game state, write to `UiAction` resource only, never mutate world
- **Process systems** run in `Update` — read `UiAction`, perform actual mutations
- Both systems must be state-gated

### State-Gated Registration
```rust
// ✅
.add_systems(Update, my_system.run_if(in_state(AppState::Dungeon)))
.add_systems(EguiPrimaryContextPass, draw_panel.run_if(in_state(AppState::Colony)))
```

### Tests
Tests live inside each module as `#[cfg(test)] mod tests` — not in a separate `tests/` directory. Use `MinimalPlugins` and `run_system_once` for ECS system tests.

## Key Files

- `src/core/state.rs` — `AppState` and `TurnState` (only state definitions)
- `src/core/components.rs` — all shared ECS components
- `src/core/resources.rs` — all shared resources
- `src/main.rs` — all system registration, grouped by AppState
- `native/assets/data/rosters.ron` — game data (enemies, items); loaded via `OnceLock` at first access
- `docs/gameplay/phase-roadmap.md` — master MVP scope document
- `docs/GDD.md` — full game design document

## After Every Task

Update `.mex/ROUTER.md` project state. If no pattern existed for the task type just completed, create one in `.mex/patterns/`. If a `context/` file is now out of date, update it surgically.
