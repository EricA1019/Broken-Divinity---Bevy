---
name: conventions
description: How code is written in this project — naming, structure, patterns, and style. Load when writing new code or reviewing existing code.
triggers:
  - "convention"
  - "pattern"
  - "naming"
  - "style"
  - "how should I"
  - "what's the right way"
edges:
  - target: context/architecture.md
    condition: when a convention depends on understanding the system structure
  - target: context/stack.md
    condition: when checking which API or library version to use
last_updated: 2026-04-06
---

# Conventions

## Naming

- Files: snake_case (`state.rs`, `combat.rs`)
- Systems: verb_noun (`resolve_attack_events`, `tick_survivor_needs`, `spawn_enemies`)
- Components: PascalCase nouns (`Health`, `CombatStats`, `SanityState`, `FactionReputation`)
- Resources: PascalCase nouns (`GameLog`, `GameTime`, `PendingAction`)
- Messages: PascalCase with `Event` suffix (`AttackEvent`, `DamageEvent`, `DeathEvent`)
- Enums: PascalCase, variants are PascalCase (`AppState::Dungeon`, `DamageType::Ballistic`)
- Constants: SCREAMING_SNAKE_CASE (`MAX_HEALTH`, `BASE_SANITY`)

## Structure

- All shared components in `components.rs` (Tier 0) — never define components locally in higher-tier modules
- All shared resources in `resources.rs` (Tier 0) — module-private resources may live in their own module
- States in `src/core/state.rs` and `src/core/turn.rs` — `AppState` lives in `state.rs`; `TurnPhase` and turn-flow resources live in `turn.rs`
- Game data files in `native/assets/data/rosters.ron` and sibling `.ron` files — loaded via `OnceLock` at first access
- System registration in `main.rs` grouped by AppState lifecycle, NOT by source module
- egui UI uses draw/process split: draw system in `EguiPrimaryContextPass`, process system (handling UiAction) in `Update`
- Tests live inside each module as `#[cfg(test)] mod tests` — not in a separate tests/ directory

## Patterns

### Graceful Query Failure (mandatory everywhere)
```rust
// ✅ Always
let Ok((mut hp, stats)) = player_query.single_mut() else { return; };
let Ok(data) = query.get(entity) else { continue; };

// ❌ Never
let (hp, stats) = query.single_mut().unwrap();
```

### Message Pipeline (cross-system communication)
```rust
// Define
#[derive(Message)]
pub struct AttackEvent { pub attacker: Entity, pub defender: Entity }

// Register in main.rs
app.add_message::<combat::AttackEvent>();

// Send
fn attack(mut events: MessageWriter<AttackEvent>) {
    events.write(AttackEvent { attacker, defender });
}

// Receive
fn on_attack(messages: Messages<AttackEvent>) {
    for event in messages.drain() { /* ... */ }
}
```

### Save-Compatible Fields
```rust
// New fields MUST have serde defaults
#[derive(Serialize, Deserialize)]
pub struct SaveGame {
    pub health: i32,
    #[serde(default)]
    pub new_field: Option<String>,
    #[serde(default = "default_sanity")]
    pub sanity: f32,
}
```

### State-Gated System Registration
```rust
// ✅ Always gate to AppState (and TurnPhase where applicable)
.add_systems(Update, my_system.run_if(in_state(AppState::Dungeon)))
.add_systems(Update, enemy_ai.run_if(in_state(AppState::Dungeon)).run_if(in_state(TurnPhase::EnemyTurn)))

// ❌ Never leave ungated
.add_systems(Update, my_system)  // runs in every state — wrong
```

## Verify Checklist

Before presenting any code:
- [ ] Query access uses graceful failure (`let Ok(...) else { return; }`) — no `.unwrap()` on queries
- [ ] New systems are gated to AppState and/or TurnPhase
- [ ] Cross-system communication uses Messages, not direct function calls upward
- [ ] New components go in `components.rs`, new shared resources in `resources.rs`
- [ ] Module imports respect the 5-tier dependency hierarchy (lower tiers never import higher)
- [ ] New save fields have `#[serde(default)]` or `#[serde(default = "fn")]`
- [ ] egui draw systems are in `EguiPrimaryContextPass`, process systems in `Update`
- [ ] Bevy 0.18 API used — `Message` not `Event`, `MessageWriter` not `EventWriter`, `messages.drain()` not `events.read()`
