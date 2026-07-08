//! Movement system — collects intents, compiles AP costs, validates, and resolves.
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
    pools::Pools,
    signals::{
        DeltaTag, EntityMoved, MoveBlockReason, MoveBlocked, MoveIntent, PoolDeltaRequested,
        PoolKind,
    },
};

pub(crate) fn register_movement(app: &mut App) {
    app.add_message::<MoveIntent>();
    app.add_message::<MoveBlocked>();
    app.add_message::<EntityMoved>();

    app.add_systems(
        bevy_app::Update,
        (
            collect_move_intents.in_set(BdSet::IntentCollection),
            compile_move_costs.in_set(BdSet::CostResolution),
            validate_and_resolve_moves.in_set(BdSet::Mutation),
        ),
    );
}

#[derive(Component, Debug, Clone)]
struct QueuedMove {
    direction: Direction,
}

fn collect_move_intents(
    mut messages: bevy_ecs::message::MessageReader<MoveIntent>,
    mut _commands: Commands,
) {
    for intent in messages.read() {
        _commands.entity(intent.entity).insert(QueuedMove {
            direction: intent.direction,
        });
    }
}

fn compile_move_costs(
    mut _commands: Commands,
    mut game_log: ResMut<GameLog>,
    mut delta_writer: bevy_ecs::message::MessageWriter<PoolDeltaRequested>,
    mut blocked_writer: bevy_ecs::message::MessageWriter<MoveBlocked>,
    actors: Query<(Entity, &QueuedMove, Option<&Pools>, Option<&Player>)>,
) {
    for (entity, queued, pools, player_flag) in actors.iter() {
        let has_ap = pools
            .and_then(|p| p.get(PoolKind::ActionPoints))
            .is_none_or(|pool| pool.current >= 1);

        if !has_ap {
            _commands.entity(entity).remove::<QueuedMove>();
            if player_flag.is_some() {
                game_log.push("Not enough Action Points.", LogLevel::Warn);
            }
            blocked_writer.write(MoveBlocked {
                entity,
                direction: queued.direction,
                reason: MoveBlockReason::NotEnoughAP,
            });
        } else {
            delta_writer.write(PoolDeltaRequested {
                source: Some(entity),
                target: entity,
                kind: PoolKind::ActionPoints,
                amount: -1,
                tags: vec![DeltaTag::MovementCost],
                reason: format!("move {:?}", direction_name(queued.direction)),
            });
        }
    }
}

fn validate_and_resolve_moves(
    mut _commands: Commands,
    map: Res<SmokeMap>,
    mut game_log: ResMut<GameLog>,
    mut moved_writer: bevy_ecs::message::MessageWriter<EntityMoved>,
    mut blocked_writer: bevy_ecs::message::MessageWriter<MoveBlocked>,
    actors: Query<(Entity, &Position, &QueuedMove, Option<&Player>)>,
    blockers: Query<&Position, With<BlocksMovement>>,
) {
    let blocked_positions: Vec<Position> = blockers.iter().copied().collect();

    for (entity, current_pos, queued, player_flag) in actors.iter() {
        let (dx, dy) = queued.direction.delta();
        let target = Position {
            x: current_pos.x + dx,
            y: current_pos.y + dy,
        };
        _commands.entity(entity).remove::<QueuedMove>();

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
                let from = *current_pos;
                _commands.entity(entity).insert(target);
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
                        MoveBlockReason::NotEnoughAP => "Not enough Action Points.",
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
    use crate::components::Tile;
    use crate::map::SmokeMap;
    use crate::pools::Pool;
    use bevy_app::App;
    use bevy_ecs::message::Messages;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(crate::BdCorePlugin);
        app
    }

    fn send_move(app: &mut App, entity: Entity, direction: Direction) {
        app.world_mut()
            .resource_mut::<Messages<MoveIntent>>()
            .write(MoveIntent { entity, direction });
    }

    #[test]
    fn valid_move_intent_changes_position() {
        let mut app = test_app();
        let player = app
            .world_mut()
            .spawn((
                Player,
                Position { x: 5, y: 5 },
                Pools::new(vec![Pool::new(PoolKind::ActionPoints, 3, 0, 3)]),
            ))
            .id();
        send_move(&mut app, player, Direction::East);
        app.update();
        assert_eq!(app.world().get::<Position>(player).unwrap().x, 6);
    }

    #[test]
    fn move_into_wall_is_rejected() {
        let mut app = test_app();
        let mut map = SmokeMap::new(10, 10, Tile::Floor);
        map.set(6, 5, Tile::Wall);
        app.world_mut().insert_resource(map);
        let player = app
            .world_mut()
            .spawn((
                Player,
                Position { x: 5, y: 5 },
                Pools::new(vec![Pool::new(PoolKind::ActionPoints, 3, 0, 3)]),
            ))
            .id();
        send_move(&mut app, player, Direction::East);
        app.update();
        assert_eq!(app.world().get::<Position>(player).unwrap().x, 5);
    }

    #[test]
    fn move_out_of_bounds_is_rejected() {
        let mut app = test_app();
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        let player = app
            .world_mut()
            .spawn((
                Player,
                Position { x: 0, y: 0 },
                Pools::new(vec![Pool::new(PoolKind::ActionPoints, 3, 0, 3)]),
            ))
            .id();
        send_move(&mut app, player, Direction::West);
        app.update();
        assert_eq!(app.world().get::<Position>(player).unwrap().x, 0);
    }

    #[test]
    fn blocked_by_entity_is_rejected() {
        let mut app = test_app();
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        let player = app
            .world_mut()
            .spawn((
                Player,
                Position { x: 5, y: 5 },
                Pools::new(vec![Pool::new(PoolKind::ActionPoints, 3, 0, 3)]),
            ))
            .id();
        app.world_mut()
            .spawn((BlocksMovement, Position { x: 6, y: 5 }));
        send_move(&mut app, player, Direction::East);
        app.update();
        assert_eq!(app.world().get::<Position>(player).unwrap().x, 5);
    }

    #[test]
    fn blocked_move_emits_denial_log() {
        let mut app = test_app();
        let mut map = SmokeMap::new(10, 10, Tile::Floor);
        map.set(6, 5, Tile::Wall);
        app.world_mut().insert_resource(map);
        let player = app
            .world_mut()
            .spawn((
                Player,
                Position { x: 5, y: 5 },
                Pools::new(vec![Pool::new(PoolKind::ActionPoints, 3, 0, 3)]),
            ))
            .id();
        send_move(&mut app, player, Direction::East);
        app.update();
        let log = app.world().resource::<GameLog>();
        assert!(log.iter().any(|e| e.message.contains("wall")));
    }

    #[test]
    fn movement_spends_ap_through_pool_delta() {
        let mut app = test_app();
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        let player = app
            .world_mut()
            .spawn((
                Player,
                Position { x: 5, y: 5 },
                Pools::new(vec![Pool::new(PoolKind::ActionPoints, 3, 0, 3)]),
            ))
            .id();
        for _ in 0..3 {
            send_move(&mut app, player, Direction::East);
            app.update();
        }
        let ap = app
            .world()
            .get::<Pools>(player)
            .unwrap()
            .get(PoolKind::ActionPoints)
            .unwrap()
            .current;
        assert_eq!(ap, 0);
        send_move(&mut app, player, Direction::East);
        app.update();
        assert_eq!(app.world().get::<Position>(player).unwrap().x, 8);
    }
}
