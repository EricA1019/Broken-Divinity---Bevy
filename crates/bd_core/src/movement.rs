//! Movement system — collects intents, validates, and resolves.
//!
//! This is the ONLY module that mutates `Position`.

use bevy_app::App;
use bevy_ecs::prelude::*;

use crate::{
    BdSet,
    components::{BlocksMovement, Player, Position},
    direction::Direction,
    gamelog::{GameLog, LogLevel},
    map::SmokeMap,
    signals::{EntityMoved, MoveBlockReason, MoveBlocked, MoveIntent},
};

/// Register movement systems into the app.
pub(crate) fn register_movement(app: &mut App) {
    // Register message types
    app.add_message::<MoveIntent>();
    app.add_message::<MoveBlocked>();
    app.add_message::<EntityMoved>();

    app.add_systems(
        bevy_app::Update,
        (
            collect_move_intents.in_set(BdSet::IntentCollection),
            validate_and_resolve_moves.in_set(BdSet::Mutation),
        ),
    );
}

/// Component placed on an entity to queue a pending move.
#[derive(Component, Debug, Clone)]
struct QueuedMove {
    direction: Direction,
}

/// Phase 1: Read `MoveIntent` messages and tag entities with `QueuedMove`.
fn collect_move_intents(
    mut messages: bevy_ecs::message::MessageReader<MoveIntent>,
    mut commands: Commands,
) {
    for intent in messages.read() {
        commands.entity(intent.entity).insert(QueuedMove {
            direction: intent.direction,
        });
    }
}

/// Phase 2: Validate each queued move and either execute it or block it.
/// This is the ONLY system that mutates `Position`.
fn validate_and_resolve_moves(
    mut commands: Commands,
    map: Res<SmokeMap>,
    mut game_log: ResMut<GameLog>,
    mut moved_writer: bevy_ecs::message::MessageWriter<EntityMoved>,
    mut blocked_writer: bevy_ecs::message::MessageWriter<MoveBlocked>,
    actors: Query<(Entity, &Position, &QueuedMove, Option<&Player>)>,
    blockers: Query<&Position, With<BlocksMovement>>,
) {
    // Collect positions occupied by blocking entities
    let blocked_positions: Vec<Position> = blockers.iter().copied().collect();

    for (entity, current_pos, queued, player_flag) in actors.iter() {
        let (dx, dy) = queued.direction.delta();
        let target = Position {
            x: current_pos.x + dx,
            y: current_pos.y + dy,
        };

        // Remove the queue marker regardless of outcome
        commands.entity(entity).remove::<QueuedMove>();

        // Validate
        let reason = if !map.is_walkable(target.x, target.y) {
            Some(MoveBlockReason::BlockedByWall)
        } else if blocked_positions.contains(&target) {
            Some(MoveBlockReason::BlockedByEntity)
        } else if map.get(target.x, target.y).is_none() {
            Some(MoveBlockReason::OutOfBounds)
        } else {
            None
        };

        match reason {
            None => {
                // Valid move — mutate position
                let from = *current_pos;
                commands.entity(entity).insert(target);

                if player_flag.is_some() {
                    game_log.push(
                        format!("You move {:?}.", direction_name(queued.direction)),
                        LogLevel::Info,
                    );
                }

                moved_writer.write(EntityMoved {
                    entity,
                    from,
                    to: target,
                });
            }
            Some(reason) => {
                if player_flag.is_some() {
                    let msg = match reason {
                        MoveBlockReason::OutOfBounds => "You can't go that way.",
                        MoveBlockReason::BlockedByWall => "There's a wall in the way.",
                        MoveBlockReason::BlockedByEntity => "Something is blocking the way.",
                    };
                    game_log.push(msg, LogLevel::Warn);
                }

                blocked_writer.write(MoveBlocked {
                    entity,
                    direction: queued.direction,
                    reason,
                });
            }
        }
    }
}

fn direction_name(dir: Direction) -> &'static str {
    match dir {
        Direction::North => "north",
        Direction::South => "south",
        Direction::East => "east",
        Direction::West => "west",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{Player, Position, Tile};
    use crate::map::SmokeMap;
    use bevy_app::App;
    use bevy_ecs::message::Messages;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(crate::BdCorePlugin);
        app
    }

    /// Helper: send a MoveIntent message in tests.
    fn send_move(app: &mut App, entity: bevy_ecs::entity::Entity, direction: Direction) {
        app.world_mut()
            .resource_mut::<Messages<MoveIntent>>()
            .write(MoveIntent { entity, direction });
    }

    #[test]
    fn valid_move_intent_changes_position() {
        let mut app = test_app();
        let player = app
            .world_mut()
            .spawn((Player, Position { x: 5, y: 5 }))
            .id();

        send_move(&mut app, player, Direction::East);
        app.update();

        let pos = app.world().get::<Position>(player).unwrap();
        assert_eq!(pos.x, 6);
        assert_eq!(pos.y, 5);
    }

    #[test]
    fn move_into_wall_is_rejected() {
        let mut app = test_app();
        let mut map = SmokeMap::new(10, 10, Tile::Floor);
        map.set(6, 5, Tile::Wall);
        app.world_mut().insert_resource(map);

        let player = app
            .world_mut()
            .spawn((Player, Position { x: 5, y: 5 }))
            .id();

        send_move(&mut app, player, Direction::East);
        app.update();

        let pos = app.world().get::<Position>(player).unwrap();
        assert_eq!(pos.x, 5);
        assert_eq!(pos.y, 5);
    }

    #[test]
    fn move_out_of_bounds_is_rejected() {
        let mut app = test_app();
        let map = SmokeMap::new(10, 10, Tile::Floor);
        app.world_mut().insert_resource(map);

        let player = app
            .world_mut()
            .spawn((Player, Position { x: 0, y: 0 }))
            .id();

        send_move(&mut app, player, Direction::West);
        app.update();

        let pos = app.world().get::<Position>(player).unwrap();
        assert_eq!(pos.x, 0);
    }

    #[test]
    fn blocked_by_entity_is_rejected() {
        let mut app = test_app();
        let map = SmokeMap::new(10, 10, Tile::Floor);
        app.world_mut().insert_resource(map);

        let player = app
            .world_mut()
            .spawn((Player, Position { x: 5, y: 5 }))
            .id();

        app.world_mut()
            .spawn((BlocksMovement, Position { x: 6, y: 5 }));

        send_move(&mut app, player, Direction::East);
        app.update();

        let pos = app.world().get::<Position>(player).unwrap();
        assert_eq!(pos.x, 5);
    }

    #[test]
    fn blocked_move_emits_denial_log() {
        let mut app = test_app();
        let mut map = SmokeMap::new(10, 10, Tile::Floor);
        map.set(6, 5, Tile::Wall);
        app.world_mut().insert_resource(map);

        let player = app
            .world_mut()
            .spawn((Player, Position { x: 5, y: 5 }))
            .id();

        send_move(&mut app, player, Direction::East);
        app.update();

        let log = app.world().resource::<GameLog>();
        let has_block_msg = log.iter().any(|e| e.message.contains("wall"));
        assert!(has_block_msg, "Log should contain wall block message");
    }
}
