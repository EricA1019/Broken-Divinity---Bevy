# Broken Divinity — Post-UX Implementation Plan v2

**Date**: 2026-07-13
**Current State**: UX Phases 0-4 complete (174 tests, 0 warnings)
**Score**: ~6.5/10
**Target**: Stable playable game loop
**Review status**: Revised per senior eng review. Phase ordering, test layers, system scheduling, and Open/Closed violations all fixed.

---

## PHASE 5: Core Combat Fix (1 hr)

### Fix strategy change
The primary fix moves to the **action validation layer** (`actions.rs`), not the TUI. The TUI guard is secondary. This makes the test correct (testing the pipeline, not TUI wiring).

### P5-A: Named constants

**File**: `crates/bd_core/src/actions.rs` — near the top, before `ActionDefinition` instances.

```rust
/// Base damage dealt by the default attack action.
pub const ATTACK_DAMAGE_BASE: i32 = 5;
/// AP cost of the default attack action.
pub const ATTACK_AP_COST: i32 = 1;
```

Replace hardcoded `-5` and `-1` in the attack action definition with these constants.

### P5-B: Add defensive NoTarget check in action validation

**File**: `crates/bd_core/src/actions.rs` — in `validate_action_intents`, the `Requirement::TargetExists` match arm.

**Current state**: `Requirement::TargetExists` is defined but only used by the guard action, not attack. Verify this arm exists and is wired for all actions that require targets:

```rust
match req {
    ...
    Requirement::TargetExists => {
        if intent.target.is_none() {
            denied = Some(DenialReason::NoTarget);
            break;
        }
    }
    ...
}
```

If the attack action's requirements already include `TargetExists` but the validation arm is missing or broken, fix the match arm.

### P5-C: Write test (TDD — tests action pipeline, not TUI)

```rust
#[test]
fn attack_no_target_does_not_self_harm() {
    let mut app = test_app();
    app.world_mut().insert_resource(SmokeMap::new(10, 10, Tile::Floor));
    let p = spawn_player(&mut app, 5, 5);
    // No enemies — send attack with no target
    send_action(&mut app, p, "ability.attack", None, None);
    app.update();
    let pools = app.world().get::<Pools>(p).unwrap();
    let hp = pools.get(PoolKind::Health).unwrap();
    assert_eq!(hp.current, hp.max, "Player should not take self-damage with no target");
}
```

**File**: `crates/bd_core/src/actions.rs` test module.

### P5-D: Guard attack input in TUI (secondary)

```rust
KeyCode::Char('f') => {
    if let Some(target) = find_nearest_enemy(player_pos.single().ok(), &enemies) {
        action_writer.write(ActionIntent {
            actor: player_entity,
            action_id: "ability.attack".into(),
            direction: None,
            target: Some(target),
        });
    } else {
        game_log.push("No targets in range.", LogLevel::Warn);
    }
}
```

**File**: `crates/bd_tui/src/lib.rs` — replace the existing `KeyCode::Char('f')` block (lines ~273-280).

### P5-E: Verify damage logging end-to-end

**Test**: `enemy_attack_targets_enemy` — place player next to enemy, press `f`, verify enemy HP drops and player HP unchanged.

```rust
#[test]
fn enemy_attack_targets_enemy() {
    let mut app = test_app();
    app.world_mut().insert_resource(SmokeMap::new(10, 10, Tile::Floor));
    let p = spawn_player(&mut app, 5, 5);
    let e = spawn_dummy(&mut app, 6, 5); // BlocksMovement + Health pool
    send_action(&mut app, p, "ability.attack", None, Some(e));
    app.update();
    let player_hp = app.world().get::<Pools>(p).unwrap().get(PoolKind::Health).unwrap().current;
    assert_eq!(player_hp, 20, "Player should NOT take damage when attacking an enemy");
    let enemy_hp = app.world().get::<Pools>(e).unwrap().get(PoolKind::Health).unwrap().current;
    assert!(enemy_hp < 10, "Enemy should take damage from attack");
}
```

---

## PHASE 6: Enemy AI (4-6 hrs)

### System scheduling change
Move enemy processing from `BdSet::Mutation` to **`BdSet::Input`** (same stage as player input), so enemy ActionIntents flow through the full validation→cost→effect pipeline in a single frame.

### P6-Arch: New module `enemy_ai.rs`

Create `crates/bd_core/src/enemy_ai.rs`.

#### Constants
```rust
/// Maximum distance at which enemies detect and pursue the player.
pub const ENEMY_DETECT_RANGE: i32 = 8;
```

#### Direction helper
```rust
/// Resolve the direction an enemy should move to approach the target.
fn direction_toward(from: Position, to: Position) -> Direction {
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    if dx.abs() >= dy.abs() {
        if dx > 0 { Direction::East } else { Direction::West }
    } else {
        if dy > 0 { Direction::South } else { Direction::North }
    }
}
```

#### Core system
```rust
pub fn process_enemy_turns(
    mode: Res<GameMode>,
    player: Query<&Position, With<Player>>,
    mut enemies: Query<(Entity, &Position, &mut Pools, Option<&Name>), (With<BlocksMovement>, Without<Player>)>,
    mut action_writer: MessageWriter<ActionIntent>,
) {
    if *mode != GameMode::Tactical { return; }
    let Ok(player_pos) = player.single() else { return; }

    for (entity, pos, mut pools, _name) in enemies.iter_mut() {
        let Some(ap) = pools.get(PoolKind::ActionPoints) else { continue; };
        if ap.current <= 0 { continue; }

        let dist = (pos.x - player_pos.x).abs().max((pos.y - player_pos.y).abs());

        if dist <= 1 {
            action_writer.write(ActionIntent {
                actor: entity,
                action_id: "ability.attack".into(),
                direction: None,
                target: Some(player_pos.single().ok().unwrap()),
            });
        } else if dist <= ENEMY_DETECT_RANGE {
            let dir = direction_toward(*pos, *player_pos);
            action_writer.write(ActionIntent {
                actor: entity,
                action_id: "ability.move".into(),
                direction: Some(dir),
                target: None,
            });
        } else {
            action_writer.write(ActionIntent {
                actor: entity,
                action_id: "ability.wait".into(),
                direction: None,
                target: None,
            });
        }
    }
}
```

#### Registration
```rust
// In lib.rs build():
mod enemy_ai;
// ...
app.add_systems(Update, enemy_ai::process_enemy_turns.in_set(BdSet::Input));
```

### P6-T: Tests (write first, TDD)

#### Test: enemy_moves_toward_player
```rust
#[test]
fn enemy_moves_toward_player() {
    let mut app = test_app();
    app.world_mut().insert_resource(GameMode::Tactical);
    app.world_mut().insert_resource(SmokeMap::new(20, 20, Tile::Floor));
    app.world_mut().spawn((
        Player, Position { x: 10, y: 10 },
        Pools::new(vec![Pool::new(PoolKind::Health, 20, 0, 20)]),
    ));
    let enemy = app.world_mut().spawn((
        BlocksMovement, Name("Rat".into()), Position { x: 5, y: 10 },
        Pools::new(vec![
            Pool::new(PoolKind::Health, 5, 0, 5),
            Pool::new(PoolKind::ActionPoints, 2, 0, 2),
        ]),
    )).id();
    app.update();
    let new_pos = app.world().get::<Position>(enemy).unwrap();
    assert!(new_pos.x > 5, "Enemy should move east toward player at x=10, got x={}", new_pos.x);
}
```

#### Test: enemy_attacks_when_adjacent
```rust
#[test]
fn enemy_attacks_when_adjacent() {
    let mut app = test_app();
    app.world_mut().insert_resource(GameMode::Tactical);
    app.world_mut().insert_resource(SmokeMap::new(20, 20, Tile::Floor));
    let player = app.world_mut().spawn((
        Player, Position { x: 10, y: 10 },
        Pools::new(vec![
            Pool::new(PoolKind::Health, 20, 0, 20),
            Pool::new(PoolKind::ActionPoints, 3, 0, 3),
        ]),
    )).id();
    app.world_mut().spawn((
        BlocksMovement, Name("Rat".into()), Position { x: 9, y: 10 },
        Pools::new(vec![
            Pool::new(PoolKind::Health, 5, 0, 5),
            Pool::new(PoolKind::ActionPoints, 2, 0, 2),
        ]),
    ));
    app.update();
    let player_hp = app.world().get::<Pools>(player).unwrap().get(PoolKind::Health).unwrap().current;
    assert!(player_hp < 20, "Enemy adjacent should damage player");
}
```

#### Test: enemy_does_not_act_out_of_detection_range
```rust
#[test]
fn enemy_does_not_act_out_of_detection_range() {
    let mut app = test_app();
    app.world_mut().insert_resource(GameMode::Tactical);
    app.world_mut().insert_resource(SmokeMap::new(20, 20, Tile::Floor));
    app.world_mut().spawn((Player, Position { x: 10, y: 10 }));
    let enemy = app.world_mut().spawn((
        BlocksMovement, Position { x: 1, y: 1 },
        Pools::new(vec![Pool::new(PoolKind::ActionPoints, 2, 0, 2)]),
    )).id();
    let before = *app.world().get::<Position>(enemy).unwrap();
    app.update();
    let after = *app.world().get::<Position>(enemy).unwrap();
    assert_eq!(before, after, "Enemy should not move if out of detection range (8)");
}
```

---

## PHASE 7: Game Over Screen (1.5 hrs)

### P7-A: Add `GameMode::GameOver` variant

**File**: `crates/bd_core/src/spatial.rs`

```rust
pub enum GameMode {
    #[default] Title,
    Outpost,
    Travel,
    Tactical,
    /// Player HP reached 0 — show death screen.
    GameOver,
}
```

Add `GameMode::GameOver => {}` to the match in `process_transitions`.

### P7-B: NEW observer `observe_player_defeat` (Open/Closed — do NOT modify `cleanup_defeated_entities`)

**File**: `crates/bd_core/src/pools.rs` — new system:

```rust
/// Watch EntityDefeated for the player entity and transition to GameOver.
fn observe_player_defeat(
    mut defeated: bevy_ecs::message::MessageReader<EntityDefeated>,
    mut mode: ResMut<GameMode>,
    player: Query<(), With<Player>>,
) {
    for msg in defeated.read() {
        if player.get(msg.entity).is_ok() {
            *mode = GameMode::GameOver;
            return;
        }
    }
}
```

**Registration**: In `register_pools` or a new `register_game_over` function:
```rust
app.add_systems(Update, observe_player_defeat.in_set(BdSet::ResultEmission));
```

### P7-C: Register game over screen

**File**: `crates/bd_tui/src/screens.rs`

Add screen definition:
```rust
reg.register(ScreenDefinition {
    id: "game_over".into(),
    panels: vec![
        PanelDefinition {
            id: "game_over_text".into(),
            layout: PanelLayout::Main,
            view_model: "StatsViewModel".into(),
        },
        PanelDefinition {
            id: "stats".into(),
            layout: PanelLayout::Right { width_pct: STATS_PANEL_WIDTH_PCT },
            view_model: "StatsViewModel".into(),
        },
    ],
});
```

Add widget binding + render function:
```rust
fn render_game_over_widget(frame: &mut Frame, area: Rect, ctx: &WidgetRenderContext) {
    let text = vec![
        ratatui::text::Line::from(""),
        ratatui::text::Line::styled(
            "  GAME OVER",
            ratatui::style::Style::default().fg(ratatui::style::Color::Red)
                .add_modifier(ratatui::style::Modifier::BOLD),
        ),
        ratatui::text::Line::from(""),
        ratatui::text::Line::styled(
            format!("  Day {} | HP: {}/{}", ctx.stats.day, ctx.stats.hp_current, ctx.stats.hp_max),
            ratatui::style::Style::default().fg(ratatui::style::Color::White),
        ),
        ratatui::text::Line::from(""),
        ratatui::text::Line::styled(
            "  Press 'q' to quit",
            ratatui::style::Style::default().fg(MUTED_COLOR),
        ),
        ratatui::text::Line::styled(
            "  Press 'r' to restart",
            ratatui::style::Style::default().fg(MUTED_COLOR),
        ),
    ];
    let para = ratatui::widgets::Paragraph::new(text)
        .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(para, area);
}
```

### P7-D: Game over input gating

**File**: `crates/bd_tui/src/lib.rs` — in `map_input_to_intents`, merge with Title handler:

```rust
// Check Title/GameOver mode before requiring player entity
match *mode {
    GameMode::Title => {
        if messages.read().next().is_some() {
            *mode = GameMode::Outpost;
            screen_writer.write(ScreenIntent { screen_id: "outpost".into() });
        }
        return;
    }
    GameMode::GameOver => {
        for key in messages.read() {
            match key.code {
                KeyCode::Char('q') => {
                    use std::io::Write;
                    let _ = crossterm::terminal::disable_raw_mode();
                    let _ = crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen);
                    std::process::exit(0);
                }
                KeyCode::Char('r') => {
                    *mode = GameMode::Title;
                    screen_writer.write(ScreenIntent { screen_id: "title".into() });
                }
                _ => {}
            }
        }
        return;
    }
    _ => {}
}
// Then player.single() check and normal input handling...
```

### P7-T: Tests

#### Test: defeat_detects_player_death
```rust
#[test]
fn defeat_detects_player_death() {
    let mut app = test_app();
    *app.world_mut().resource_mut::<GameMode>() = GameMode::Tactical;
    let p = app.world_mut().spawn((
        Player, Name("Player".into()),
        Pools::new(vec![Pool::new(PoolKind::Health, 1, 0, 20)]),
    )).id();
    send_delta(&mut app, p, PoolKind::Health, -5);
    app.update();
    assert_eq!(*app.world().resource::<GameMode>(), GameMode::GameOver);
    assert!(app.world().entities().contains(p), "Player should not be despawned on death");
}
```

#### Test: game_over_screen_is_registered
```rust
#[test]
fn game_over_screen_is_registered() {
    let registry = default_screen_registry();
    let screen = registry.get("game_over");
    assert!(screen.is_some(), "Game over screen should be registered");
    let panel_ids: Vec<&str> = screen.unwrap().panels.iter().map(|p| p.id.as_str()).collect();
    assert!(panel_ids.contains(&"game_over_text"));
}
```

---

## PHASE 8: Save/Load Polish (2 hrs)

### P8-A: Complete serde audit — all types that need annotations

Checklist:
- [ ] `crate::time::ShouldAdvanceTime` — add `Serialize, Deserialize, Default`
- [ ] `crate::time::GameTime` — already has serde
- [ ] `crate::map::SmokeMap` — already has serde
- [ ] `crate::spatial::GameMode` — add `Serialize, Deserialize`
- [ ] `crate::spatial::OutpostState` — already has serde
- [ ] `crate::spatial::TravelMap` — add `Serialize, Deserialize`
- [ ] `crate::spatial::TravelNode` — add `Serialize, Deserialize`
- [ ] `crate::pools::Pool` — already has serde
- [ ] `crate::pools::Pools` — add `Serialize, Deserialize`
- [ ] `crate::statuses::Statuses` — check
- [ ] `crate::events::CurrentEvent` — add `Serialize, Deserialize`
- [ ] `crate::events::EventRegistry` — add `Serialize, Deserialize`
- [ ] `crate::gabriel::GabrielState` — already has serde
- [ ] `crate::factions::FactionReputation` — add `Serialize, Deserialize`
- [ ] `crate::overworld::OverworldState` — already has serde
- [ ] `crate::overworld::Weather` — add `Serialize, Deserialize`
- [ ] `crate::colony::production::ColonyResources` — add `Serialize, Deserialize`
- [ ] `crate::colony::stations::PendingStationBuild` — add `Serialize, Deserialize`
- [ ] `crate::combat::CombatRng` — add `Serialize, Deserialize`
- [ ] `crate::gamelog::GameLog` — add `Serialize, Deserialize`
- [ ] `crate::dialogue::DialogueLog` — add `Serialize, Deserialize`
- [ ] `crate::party::PartyState` — add `Serialize, Deserialize`
- [ ] `crate::signals::PoolKind` — already has serde
- [ ] `crate::signals::DeltaTag` — already has serde
- [ ] `crate::components::Tile` — already has serde
- [ ] `bd_tui::view_models::HelpViewModel` — add `Serialize, Deserialize, Default`

### P8-B: Resolve save file path

Use `config::data_dir()` (already exists in `bd_app`) which resolves to `$XDG_DATA_HOME/broken-divinity/save.ron`. The save/load functions in `save.rs` already take a path parameter.

### P8-C: Write save roundtrip test (TDD)

```rust
#[test]
fn save_roundtrip_preserves_state() {
    let mut app = test_app();
    let p = app.world_mut().spawn((
        Player, Position { x: 5, y: 5 },
        Pools::new(vec![Pool::new(PoolKind::Health, 15, 0, 20)]),
    )).id();
    let snapshot = save_world(app.world());
    app.world_mut().entity_mut(p).insert(Position { x: 10, y: 10 });
    load_world(app.world_mut(), &snapshot);
    let pos = app.world().get::<Position>(p).unwrap();
    assert_eq!(*pos, Position { x: 5, y: 5 });
    let hp = app.world().get::<Pools>(p).unwrap().get(PoolKind::Health).unwrap().current;
    assert_eq!(hp, 15);
}
```

### P8-D: Wire save/load to keys

```rust
// Save game
KeyCode::F(5) => {
    game_log.push("Game saved.", LogLevel::Info);
    // save_world(path) ...
}
// Load game
KeyCode::F(9) => {
    game_log.push("Game loaded.", LogLevel::Info);
    // load_world(path) ...
}
```

**File**: `crates/bd_tui/src/lib.rs` — add as new key handlers.

---

## IMPLEMENTATION ORDER

```
Phase 5: Combat Fix (1 hr) ← PRIMARY FIX IN VALIDATION LAYER
├── P5-A: Extract ATTACK_DAMAGE_BASE + ATTACK_AP_COST constants
├── P5-C: Write test — attack_no_target_does_not_self_harm (FAILS first)
├── P5-B: Add NoTarget denial in validate_action_intents (test PASSES)
├── P5-D: TUI guard (secondary)
├── P5-E: Write test — enemy_attack_targets_enemy
└── Full test suite

Phase 6: Enemy AI (4-6 hrs) ← REGISTERED IN BdSet::Input
├── Create enemy_ai.rs with constants + direction_toward helper
├── Write test — enemy_moves_toward_player (FAILS)
├── Implement movement logic (test PASSES)
├── Write test — enemy_attacks_when_adjacent (FAILS)
├── Implement attack logic (test PASSES)
├── Write test — enemy_does_not_act_out_of_detection_range (FAILS)
├── Implement detection range gate (test PASSES)
├── Register in lib.rs
└── Full test suite

Phase 7: Game Over (1.5 hrs) ← NEW OBSERVER, NOT MODIFYING cleanup
├── P7-A: GameMode::GameOver variant
├── P7-B: observe_player_defeat system (new, separate)
├── Write test — defeat_detects_player_death (TDD)
├── P7-C: Game over screen + widget
├── Write test — game_over_screen_is_registered (TDD)
├── P7-D: Merge Title/GameOver input gating
└── Full test suite

Phase 8: Save/Load (2 hrs) ← SERDE AUDIT FIRST
├── P8-A: Serde audit — add all missing annotations
├── P8-B: Resolve save file path
├── Write test — save_roundtrip_preserves_state (TDD)
├── P8-D: Wire F5/F9 keys
└── Full test suite
```

## TEST INVENTORY

| Test | Phase | File | Tests what |
|------|-------|------|-----------|
| `attack_no_target_does_not_self_harm` | P5 | actions.rs | Validator denies attack with no target |
| `enemy_attack_targets_enemy` | P5 | actions.rs | Damage goes to enemy, not self |
| `enemy_moves_toward_player` | P6 | enemy_ai.rs | Movement direction toward player |
| `enemy_attacks_when_adjacent` | P6 | enemy_ai.rs | Adjacent enemy deals damage |
| `enemy_does_not_act_out_of_detection_range` | P6 | enemy_ai.rs | Detection range gate |
| `defeat_detects_player_death` | P7 | pools.rs | Player death → GameOver mode |
| `game_over_screen_is_registered` | P7 | screens.rs | Screen definition exists |
| `save_roundtrip_preserves_state` | P8 | save.rs | Save/load round-trip fidelity |

## FILES CHANGED

| File | Phase | Change |
|------|-------|--------|
| `crates/bd_core/src/actions.rs` | P5 | Constants + NoTarget validation arm + tests |
| `crates/bd_tui/src/lib.rs` | P5,P7,P8 | Attack guard, Title/GameOver merge, F5/F9 keys |
| `crates/bd_core/src/enemy_ai.rs` | P6 | NEW — enemy turn processing |
| `crates/bd_core/src/lib.rs` | P6 | Register enemy_ai module |
| `crates/bd_core/src/spatial.rs` | P7 | GameOver variant + serde |
| `crates/bd_core/src/pools.rs` | P7 | NEW observe_player_defeat system |
| `crates/bd_tui/src/screens.rs` | P7 | Game over screen + widget |
| `crates/bd_core/src/time.rs` | P8 | ShouldAdvanceTime serde |
| `crates/bd_tui/src/view_models.rs` | P8 | HelpViewModel serde |
| Various resource files | P8 | Serde annotations audit |
