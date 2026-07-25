//! Enemy AI — basic combat behaviors for hostile entities.
//!
//! Each frame, `process_enemy_turns` reads enemy and player positions, then
//! writes `ActionIntent` messages for enemies within detection range.
//! Intents flow through the standard validation → cost → effect pipeline.

use bevy_app::App;
use bevy_ecs::prelude::*;

use crate::actions::{ActionDefinition, Effect, Requirement};
use crate::components::{BlocksMovement, Player, Position};
use crate::direction::Direction;
use crate::map::SmokeMap;
use crate::signals::{ActionIntent, DeltaTag, PoolKind};

// ── Constants ──

/// Maximum Manhattan distance at which an enemy detects and pursues the player.
const ENEMY_DETECT_RANGE: i32 = 8;

/// Base damage dealt by a basic enemy melee attack.
const ENEMY_ATTACK_DAMAGE: i32 = 5;
/// Action Points consumed by an enemy attack.
const ENEMY_ATTACK_AP_COST: i32 = 1;

// ── Plugin registration ──

pub(crate) fn register_enemy_ai(app: &mut App) {
    // Register the enemy-specific attack action (no TargetHostile — enemies target the player)
    app.world_mut()
        .resource_mut::<crate::actions::ActionRegistry>()
        .register(enemy_melee_action());

    // Register the AI system in Input stage (same stage as player input)
    app.add_systems(
        bevy_app::Update,
        process_enemy_turns.in_set(crate::BdSet::Input),
    );
    app.add_systems(
        bevy_app::Update,
        release_enemy_phase_lock.in_set(crate::BdSet::ResultEmission),
    );
}

/// Enemy melee attack action definition — no `TargetHostile` so enemies can target the player.
fn enemy_melee_action() -> ActionDefinition {
    ActionDefinition {
        id: "ability.enemy_melee".into(),
        label: "Enemy Melee".into(),
        requirements: vec![
            Requirement::HasPoolAtLeast(PoolKind::ActionPoints, ENEMY_ATTACK_AP_COST),
            Requirement::TargetExists,
            Requirement::TargetInRange(1),
        ],
        cost_effects: vec![Effect::PoolDelta {
            kind: PoolKind::ActionPoints,
            amount: -ENEMY_ATTACK_AP_COST,
            tags: vec![DeltaTag::Action],
            reason: "enemy attack cost".into(),
        }],
        effects: vec![Effect::PoolDelta {
            kind: PoolKind::Health,
            amount: -ENEMY_ATTACK_DAMAGE,
            tags: vec![DeltaTag::Physical],
            reason: "enemy melee hit".into(),
        }],
    }
}

// ── System ──

/// Reads enemy positions, checks range to player, and writes move/attack intents.
/// Runs in `BdSet::Input` alongside player input — both are consumed together
/// by validation.
#[allow(clippy::type_complexity)] // Query types are inherently complex in ECS
fn process_enemy_turns(
    enemies: Query<
        (Entity, &Position, Option<&crate::spatial::EntityScope>),
        (With<BlocksMovement>, Without<Player>),
    >,
    player: Query<(Entity, &Position, Option<&crate::spatial::EntityScope>), With<Player>>,
    map: Res<SmokeMap>,
    mode: Res<crate::spatial::GameMode>,
    foundation: Option<Res<crate::session::FoundationRuntime>>,
    mut turn: ResMut<crate::time::ShouldAdvanceTime>,
    blockers: Query<
        (&Position, Option<&crate::spatial::EntityScope>),
        (With<BlocksMovement>, Without<Player>),
    >,
    mut action_writer: bevy_ecs::message::MessageWriter<ActionIntent>,
) {
    if !turn.1 || *mode != crate::spatial::GameMode::Tactical {
        return;
    }

    let foundation_runtime = foundation.is_some();
    let Some((player_entity, player_pos, _)) = player
        .iter()
        .find(|(_, _, scope)| crate::spatial::entity_is_active(*scope, *mode, foundation_runtime))
    else {
        turn.1 = false;
        return; // No player alive — enemies idle
    };

    for (entity, pos, scope) in &enemies {
        if !crate::spatial::entity_is_active(scope, *mode, foundation_runtime) {
            continue;
        }
        let dx = player_pos.x - pos.x;
        let dy = player_pos.y - pos.y;
        let dist = dx.abs() + dy.abs();

        if dist > ENEMY_DETECT_RANGE {
            continue; // Too far — idle
        }

        if dist == 1 {
            // Adjacent — attack using enemy-specific action (no TargetHostile requirement)
            action_writer.write(ActionIntent {
                actor: entity,
                action_id: "ability.enemy_melee".into(),
                direction: None,
                target: Some(player_entity),
            });
        } else {
            // Move toward player
            let dir = Direction::toward(*pos, *player_pos);
            let (dx_step, dy_step) = dir.delta();
            let target_pos = Position {
                x: pos.x + dx_step,
                y: pos.y + dy_step,
            };

            // Check map walkability
            if !map.is_walkable(target_pos.x, target_pos.y) {
                continue; // Blocked by wall
            }

            // Check for blocking entities (other enemies occupy target)
            let is_occupied = blockers.iter().any(|(position, scope)| {
                crate::spatial::entity_is_active(scope, *mode, foundation_runtime)
                    && *position == target_pos
            });
            if is_occupied {
                continue; // Blocked by entity
            }

            action_writer.write(ActionIntent {
                actor: entity,
                action_id: "ability.move".into(),
                direction: Some(dir),
                target: None,
            });
        }
    }

    // Consume the request before validation/effect resolution. Any player
    // input observed in this same frame is rejected by AwaitingEnemyPhase.
    turn.1 = false;
}

/// Release the player lock only after validation, enemy effects, and results
/// for the enemy phase have completed.
fn release_enemy_phase_lock(
    mut commands: Commands,
    turn: Res<crate::time::ShouldAdvanceTime>,
    player: Query<Entity, (With<Player>, With<crate::time::AwaitingEnemyPhase>)>,
) {
    if turn.1 {
        return;
    }
    for entity in &player {
        commands
            .entity(entity)
            .remove::<crate::time::AwaitingEnemyPhase>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Tile;
    use crate::map::SmokeMap;
    use crate::pools::{Pool, Pools};
    use crate::signals::PoolKind;
    use bevy_app::App;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(crate::BdCorePlugin);
        app
    }

    fn spawn_player(app: &mut App, x: i32, y: i32) -> Entity {
        app.world_mut()
            .spawn((
                Player,
                Position { x, y },
                Pools::new(vec![
                    Pool::new(PoolKind::Health, 20, 0, 20),
                    Pool::new(PoolKind::ActionPoints, 3, 0, 3),
                ]),
            ))
            .id()
    }

    fn spawn_enemy(app: &mut App, x: i32, y: i32) -> Entity {
        app.world_mut()
            .spawn((
                BlocksMovement,
                Position { x, y },
                Pools::new(vec![
                    Pool::new(PoolKind::Health, 10, 0, 10),
                    Pool::new(PoolKind::ActionPoints, 2, 0, 2),
                ]),
            ))
            .id()
    }

    #[test]
    fn enemy_moves_toward_player() {
        let mut app = test_app();
        let map = SmokeMap::new(20, 20, Tile::Floor);
        app.world_mut().insert_resource(map);
        let _player = spawn_player(&mut app, 10, 10);
        let enemy = spawn_enemy(&mut app, 10, 7); // 3 tiles north

        app.world_mut()
            .insert_resource(crate::spatial::GameMode::Tactical);
        app.world_mut()
            .resource_mut::<crate::time::ShouldAdvanceTime>()
            .1 = true;
        app.update();

        // Enemy should move 1 tile south toward player
        let pos = app.world().get::<Position>(enemy).unwrap();
        assert_eq!(
            *pos,
            Position { x: 10, y: 8 },
            "enemy should move south toward player"
        );
    }

    #[test]
    fn enemy_attacks_when_adjacent() {
        let mut app = test_app();
        let map = SmokeMap::new(20, 20, Tile::Floor);
        app.world_mut().insert_resource(map);
        let player = spawn_player(&mut app, 10, 10);
        let _enemy = spawn_enemy(&mut app, 10, 9); // 1 tile north — adjacent

        app.world_mut()
            .insert_resource(crate::spatial::GameMode::Tactical);
        app.world_mut()
            .resource_mut::<crate::time::ShouldAdvanceTime>()
            .1 = true;
        app.update();
        app.update(); // second frame to process pool deltas

        // Player should have taken damage
        let hp = app
            .world()
            .get::<Pools>(player)
            .unwrap()
            .get(PoolKind::Health)
            .unwrap()
            .current;
        // P13: d100 variance means damage is 0.5x/1.0x/1.5x of base 5
        // Expected: 3, 5, or 8 damage. Accept any valid variance.
        let damage_taken = 20 - hp;
        assert!(
            damage_taken >= 2 && damage_taken <= 8,
            "player should take 2-8 damage from base 5 with d100 variance, took {} (HP: 20 -> {})",
            damage_taken,
            hp
        );
        assert!(hp < 20, "player should take some damage, HP still 20");
    }

    #[test]
    fn enemy_idle_out_of_detect_range() {
        let mut app = test_app();
        let map = SmokeMap::new(50, 50, Tile::Floor);
        app.world_mut().insert_resource(map);
        let _player = spawn_player(&mut app, 10, 10);
        let enemy = spawn_enemy(&mut app, 10, 1); // 9 tiles north — > ENEMY_DETECT_RANGE(8)

        app.update();

        // Enemy should not have moved
        let pos = app.world().get::<Position>(enemy).unwrap();
        assert_eq!(
            *pos,
            Position { x: 10, y: 1 },
            "enemy should stay in place when out of range"
        );
    }
}
