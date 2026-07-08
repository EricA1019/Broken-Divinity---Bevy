//! Action system — validated actions with requirements, costs, and effects.
//!
//! All player/AI actions go through: intent → validate → cost → effect.
//! No system directly mutates state outside this pipeline.

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
        ActionDenied, ActionIntent, DeltaTag, DenialReason, EntityMoved, MoveBlockReason,
        MoveBlocked, PoolDeltaRequested, PoolKind,
    },
    trace::SignalTrace,
};

// ── Action definition ──

/// Unique identifier for an action.
pub type ActionId = String;

/// A requirement that must be met for the action to proceed.
#[derive(Debug, Clone)]
pub enum Requirement {
    /// Actor must have at least N of a pool kind.
    HasPoolAtLeast(PoolKind, i32),
    /// Target tile must be walkable (for move actions).
    TileWalkable,
    /// Target entity must exist.
    TargetExists,
    /// Target entity must be hostile (not the actor, not Player).
    TargetHostile,
    /// Target must be within N tiles (Manhattan distance).
    TargetInRange(u32),
    /// Actor must be alive (health > min).
    EntityAlive,
}

/// An effect produced by a successful action.
#[derive(Debug, Clone)]
pub enum Effect {
    /// Apply a pool delta.
    PoolDelta {
        kind: PoolKind,
        amount: i32,
        tags: Vec<DeltaTag>,
        reason: String,
    },
    /// Move the actor in a direction.
    MoveEntity,
    /// Log a message.
    Log(String, LogLevel),
    /// Placeholder: apply a status (Phase 9).
    ApplyStatus(String),
}

/// Definition of an action: what it costs, requires, and produces.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ActionDefinition {
    pub id: ActionId,
    pub label: String,
    pub requirements: Vec<Requirement>,
    pub cost_effects: Vec<Effect>,
    pub effects: Vec<Effect>,
}

/// Registry of all available actions.
#[derive(Resource, Debug, Clone)]
pub struct ActionRegistry {
    definitions: Vec<ActionDefinition>,
}

impl ActionRegistry {
    pub fn get(&self, id: &str) -> Option<&ActionDefinition> {
        self.definitions.iter().find(|d| d.id == id)
    }

    /// Create the default Phase 3 action set.
    pub fn phase3_defaults() -> Self {
        Self {
            definitions: vec![
                ActionDefinition {
                    id: "ability.move".into(),
                    label: "Move".into(),
                    requirements: vec![
                        Requirement::HasPoolAtLeast(PoolKind::ActionPoints, 1),
                        Requirement::TileWalkable,
                    ],
                    cost_effects: vec![Effect::PoolDelta {
                        kind: PoolKind::ActionPoints,
                        amount: -1,
                        tags: vec![DeltaTag::MovementCost],
                        reason: "move".into(),
                    }],
                    effects: vec![
                        Effect::MoveEntity,
                        Effect::Log("You move.".into(), LogLevel::Info),
                    ],
                },
                ActionDefinition {
                    id: "ability.wait".into(),
                    label: "Wait".into(),
                    requirements: vec![Requirement::EntityAlive],
                    cost_effects: vec![],
                    effects: vec![
                        Effect::PoolDelta {
                            kind: PoolKind::ActionPoints,
                            amount: 1,
                            tags: vec![DeltaTag::Recovery],
                            reason: "wait".into(),
                        },
                        Effect::Log("You wait.".into(), LogLevel::Info),
                    ],
                },
                ActionDefinition {
                    id: "ability.attack".into(),
                    label: "Attack".into(),
                    requirements: vec![
                        Requirement::HasPoolAtLeast(PoolKind::ActionPoints, 1),
                        Requirement::TargetExists,
                        Requirement::TargetHostile,
                        Requirement::TargetInRange(1),
                    ],
                    cost_effects: vec![Effect::PoolDelta {
                        kind: PoolKind::ActionPoints,
                        amount: -1,
                        tags: vec![DeltaTag::Action],
                        reason: "attack cost".into(),
                    }],
                    effects: vec![
                        Effect::PoolDelta {
                            kind: PoolKind::Health,
                            amount: -5,
                            tags: vec![DeltaTag::Physical],
                            reason: "attack hit".into(),
                        },
                        Effect::Log("You attack!".into(), LogLevel::Combat),
                    ],
                },
                ActionDefinition {
                    id: "ability.guard".into(),
                    label: "Guard".into(),
                    requirements: vec![Requirement::HasPoolAtLeast(PoolKind::ActionPoints, 1)],
                    cost_effects: vec![Effect::PoolDelta {
                        kind: PoolKind::ActionPoints,
                        amount: -1,
                        tags: vec![DeltaTag::Action],
                        reason: "guard".into(),
                    }],
                    effects: vec![
                        Effect::ApplyStatus("status.guarded".into()),
                        Effect::Log("You brace for impact.".into(), LogLevel::Info),
                    ],
                },
            ],
        }
    }
}

// ── Plugin registration ──

pub(crate) fn register_actions(app: &mut App) {
    app.add_message::<ActionIntent>();
    app.add_message::<ActionDenied>();

    app.insert_resource(ActionRegistry::phase3_defaults());

    app.add_systems(
        bevy_app::Update,
        (
            validate_action_intents.in_set(BdSet::Validation),
            compile_action_costs.in_set(BdSet::CostResolution),
            resolve_action_effects.in_set(BdSet::Mutation),
        ),
    );
}

// ── Validation ──

#[derive(Component, Debug, Clone)]
struct PendingAction {
    action_id: ActionId,
    direction: Option<Direction>,
    target: Option<Entity>,
}

/// Read ActionIntent messages, validate against action definitions, and
/// either tag with PendingAction or emit ActionDenied.
#[allow(clippy::too_many_arguments)]
fn validate_action_intents(
    mut commands: Commands,
    registry: Res<ActionRegistry>,
    map: Res<SmokeMap>,
    mut messages: bevy_ecs::message::MessageReader<ActionIntent>,
    mut denied_writer: bevy_ecs::message::MessageWriter<ActionDenied>,
    mut game_log: ResMut<GameLog>,
    mut trace: ResMut<SignalTrace>,
    actors: Query<(Entity, &Position, Option<&Pools>, Option<&Player>)>,
    targets: Query<(Entity, &Position, Option<&Player>)>,
) {
    let target_positions: Vec<(Entity, &Position, Option<&Player>)> = targets.iter().collect();

    for intent in messages.read() {
        trace.push(
            "Validation",
            "ActionIntent",
            format!("actor={:?} action={}", intent.actor, intent.action_id),
        );
        let Ok((_, actor_pos, pools, player_flag)) = actors.get(intent.actor) else {
            continue;
        };

        let Some(def) = registry.get(&intent.action_id) else {
            denied_writer.write(ActionDenied {
                actor: intent.actor,
                action_id: intent.action_id.clone(),
                reason: DenialReason::Other("unknown action".into()),
            });
            continue;
        };

        // Check each requirement
        let mut denied = None;

        for req in &def.requirements {
            match req {
                Requirement::HasPoolAtLeast(kind, min) => {
                    let current = pools.and_then(|p| p.get(*kind)).map_or(0, |p| p.current);
                    if current < *min {
                        denied = Some(DenialReason::NotEnoughPool(*kind));
                        break;
                    }
                }
                Requirement::TileWalkable => {
                    let Some(dir) = intent.direction else {
                        denied = Some(DenialReason::Other("no direction".into()));
                        break;
                    };
                    let (dx, dy) = dir.delta();
                    let tx = actor_pos.x + dx;
                    let ty = actor_pos.y + dy;
                    if !map.is_walkable(tx, ty) {
                        denied = Some(DenialReason::BlockedTile);
                        break;
                    }
                    // Also check for blocking entities
                    // (we'll do this at effect time for now)
                }
                Requirement::TargetExists => {
                    if intent.target.is_none() {
                        denied = Some(DenialReason::NoTarget);
                        break;
                    }
                }
                Requirement::TargetHostile => {
                    let Some(t) = intent.target else {
                        denied = Some(DenialReason::NoTarget);
                        break;
                    };
                    // Hostile = not Player, not same entity
                    let is_hostile = target_positions
                        .iter()
                        .any(|(e, _, p)| *e == t && p.is_none() && *e != intent.actor);
                    if !is_hostile {
                        denied = Some(DenialReason::InvalidTarget);
                        break;
                    }
                }
                Requirement::TargetInRange(range) => {
                    let Some(t) = intent.target else {
                        denied = Some(DenialReason::NoTarget);
                        break;
                    };
                    let in_range = target_positions.iter().any(|(e, tpos, _)| {
                        *e == t
                            && (actor_pos.x - tpos.x).unsigned_abs()
                                + (actor_pos.y - tpos.y).unsigned_abs()
                                <= *range
                    });
                    if !in_range {
                        denied = Some(DenialReason::OutOfRange);
                        break;
                    }
                }
                Requirement::EntityAlive => {
                    // Default: entities without a Health pool are always alive
                    let hp = pools
                        .and_then(|p| p.get(PoolKind::Health))
                        .map_or(1, |p| p.current);
                    if hp <= 0 {
                        denied = Some(DenialReason::ActorDefeated);
                        break;
                    }
                }
            }
        }

        if let Some(reason) = denied {
            if player_flag.is_some() {
                let msg = match &reason {
                    DenialReason::NotEnoughPool(kind) => {
                        format!("Not enough {:?}.", kind)
                    }
                    DenialReason::BlockedTile => "Blocked.".into(),
                    DenialReason::OutOfRange => "Out of range.".into(),
                    DenialReason::NoTarget => "No target.".into(),
                    DenialReason::InvalidTarget => "Invalid target.".into(),
                    DenialReason::ActorDefeated => "Can't act while defeated.".into(),
                    DenialReason::Other(s) => s.clone(),
                };
                game_log.push(msg, LogLevel::Warn);
            }

            denied_writer.write(ActionDenied {
                actor: intent.actor,
                action_id: intent.action_id.clone(),
                reason,
            });
        } else {
            commands.entity(intent.actor).insert(PendingAction {
                action_id: intent.action_id.clone(),
                direction: intent.direction,
                target: intent.target,
            });
        }
    }
}

// ── Cost compilation ──

/// Compile cost effects into PoolDeltaRequested messages.
fn compile_action_costs(
    mut commands: Commands,
    registry: Res<ActionRegistry>,
    mut trace: ResMut<SignalTrace>,
    mut delta_writer: bevy_ecs::message::MessageWriter<PoolDeltaRequested>,
    actors: Query<(Entity, &PendingAction)>,
) {
    for (entity, pending) in actors.iter() {
        trace.push(
            "CostResolution",
            "CostCompile",
            format!("entity={:?} action={}", entity, pending.action_id),
        );
        let Some(def) = registry.get(&pending.action_id) else {
            commands.entity(entity).remove::<PendingAction>();
            continue;
        };

        for effect in &def.cost_effects {
            if let Effect::PoolDelta {
                kind,
                amount,
                tags,
                reason,
            } = effect
            {
                delta_writer.write(PoolDeltaRequested {
                    source: Some(entity),
                    target: entity,
                    kind: *kind,
                    amount: *amount,
                    tags: tags.clone(),
                    reason: reason.clone(),
                });
            }
        }
    }
}

// ── Effect resolution ──

/// Resolve action effects: move entities, apply pool deltas (via messages), log, etc.
#[allow(clippy::too_many_arguments)]
fn resolve_action_effects(
    mut commands: Commands,
    registry: Res<ActionRegistry>,
    map: Res<SmokeMap>,
    mut game_log: ResMut<GameLog>,
    mut trace: ResMut<SignalTrace>,
    mut delta_writer: bevy_ecs::message::MessageWriter<PoolDeltaRequested>,
    mut moved_writer: bevy_ecs::message::MessageWriter<EntityMoved>,
    mut blocked_writer: bevy_ecs::message::MessageWriter<MoveBlocked>,
    actors: Query<(Entity, &Position, &PendingAction, Option<&Player>)>,
    blockers: Query<&Position, With<BlocksMovement>>,
) {
    let blocked_positions: Vec<Position> = blockers.iter().copied().collect();

    for (entity, pos, pending, player_flag) in actors.iter() {
        trace.push(
            "Mutation",
            "EffectResolve",
            format!("entity={:?} action={}", entity, pending.action_id),
        );
        let Some(def) = registry.get(&pending.action_id) else {
            commands.entity(entity).remove::<PendingAction>();
            continue;
        };

        for effect in &def.effects {
            match effect {
                Effect::MoveEntity => {
                    let Some(dir) = pending.direction else {
                        continue;
                    };
                    let (dx, dy) = dir.delta();
                    let target = Position {
                        x: pos.x + dx,
                        y: pos.y + dy,
                    };

                    // Re-validate movement at effect time (blocking entities)
                    if !map.is_walkable(target.x, target.y) || blocked_positions.contains(&target) {
                        if player_flag.is_some() {
                            game_log.push("Blocked.", LogLevel::Warn);
                        }
                        blocked_writer.write(MoveBlocked {
                            entity,
                            direction: dir,
                            reason: MoveBlockReason::BlockedByWall,
                        });
                        continue;
                    }

                    let from = *pos;
                    commands.entity(entity).insert(target);
                    moved_writer.write(EntityMoved {
                        entity,
                        from,
                        to: target,
                    });
                }
                Effect::PoolDelta {
                    kind,
                    amount,
                    tags,
                    reason,
                } => {
                    // Pool deltas from effects target the actor unless it's damage to target
                    let target = if *kind == PoolKind::Health && *amount < 0 {
                        pending.target.unwrap_or(entity)
                    } else {
                        entity
                    };
                    delta_writer.write(PoolDeltaRequested {
                        source: Some(entity),
                        target,
                        kind: *kind,
                        amount: *amount,
                        tags: tags.clone(),
                        reason: reason.clone(),
                    });
                }
                Effect::Log(msg, level) => {
                    if player_flag.is_some() {
                        game_log.push(msg.clone(), *level);
                    }
                }
                Effect::ApplyStatus(_status_id) => {
                    // Placeholder — status system arrives in Phase 9
                    if player_flag.is_some() {
                        game_log.push("You guard. (status not yet implemented)", LogLevel::Info);
                    }
                }
            }
        }

        commands.entity(entity).remove::<PendingAction>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{BlocksMovement, Player, Tile};
    use crate::map::SmokeMap;
    use crate::pools::Pool;
    use bevy_app::App;
    use bevy_ecs::message::Messages;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(crate::BdCorePlugin);
        app
    }

    fn send_action(
        app: &mut App,
        actor: Entity,
        action_id: &str,
        direction: Option<Direction>,
        target: Option<Entity>,
    ) {
        app.world_mut()
            .resource_mut::<Messages<ActionIntent>>()
            .write(ActionIntent {
                actor,
                action_id: action_id.into(),
                direction,
                target,
            });
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

    fn spawn_dummy(app: &mut App, x: i32, y: i32) -> Entity {
        app.world_mut()
            .spawn((
                BlocksMovement,
                Position { x, y },
                Pools::new(vec![
                    Pool::new(PoolKind::Health, 10, 0, 10),
                    Pool::new(PoolKind::ActionPoints, 0, 0, 3),
                ]),
            ))
            .id()
    }

    #[test]
    fn move_denied_without_ap() {
        let mut app = test_app();
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        let p = app
            .world_mut()
            .spawn((
                Player,
                Position { x: 5, y: 5 },
                Pools::new(vec![Pool::new(PoolKind::ActionPoints, 0, 0, 3)]),
            ))
            .id();
        send_action(&mut app, p, "ability.move", Some(Direction::East), None);
        app.update();
        assert_eq!(app.world().get::<Position>(p).unwrap().x, 5);
    }

    #[test]
    fn move_denied_into_wall() {
        let mut app = test_app();
        let mut map = SmokeMap::new(10, 10, Tile::Floor);
        map.set(6, 5, Tile::Wall);
        app.world_mut().insert_resource(map);
        let p = spawn_player(&mut app, 5, 5);
        send_action(&mut app, p, "ability.move", Some(Direction::East), None);
        app.update();
        assert_eq!(app.world().get::<Position>(p).unwrap().x, 5);
    }

    #[test]
    fn move_costs_compile_to_pool_delta() {
        let mut app = test_app();
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        let p = spawn_player(&mut app, 5, 5);
        send_action(&mut app, p, "ability.move", Some(Direction::East), None);
        app.update();
        let ap = app
            .world()
            .get::<Pools>(p)
            .unwrap()
            .get(PoolKind::ActionPoints)
            .unwrap()
            .current;
        assert_eq!(ap, 2);
    }

    #[test]
    fn wait_restores_ap() {
        let mut app = test_app();
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        let p = app
            .world_mut()
            .spawn((
                Player,
                Position { x: 5, y: 5 },
                Pools::new(vec![Pool::new(PoolKind::ActionPoints, 1, 0, 3)]),
            ))
            .id();
        send_action(&mut app, p, "ability.wait", None, None);
        app.update();
        let ap = app
            .world()
            .get::<Pools>(p)
            .unwrap()
            .get(PoolKind::ActionPoints)
            .unwrap()
            .current;
        assert_eq!(ap, 2);
    }

    #[test]
    fn attack_denied_out_of_range() {
        let mut app = test_app();
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        let p = spawn_player(&mut app, 5, 5);
        let dummy = spawn_dummy(&mut app, 9, 5); // 4 tiles away, range requires 1
        send_action(&mut app, p, "ability.attack", None, Some(dummy));
        app.update();
        // Dummy HP should be unchanged
        let hp = app
            .world()
            .get::<Pools>(dummy)
            .unwrap()
            .get(PoolKind::Health)
            .unwrap()
            .current;
        assert_eq!(hp, 10);
    }

    #[test]
    fn attack_emits_health_pool_delta() {
        let mut app = test_app();
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        let p = spawn_player(&mut app, 5, 5);
        let dummy = spawn_dummy(&mut app, 6, 5); // adjacent
        send_action(&mut app, p, "ability.attack", None, Some(dummy));
        app.update();
        let hp = app
            .world()
            .get::<Pools>(dummy)
            .unwrap()
            .get(PoolKind::Health)
            .unwrap()
            .current;
        assert_eq!(hp, 5); // 10 - 5
    }

    #[test]
    fn guard_emits_no_errors() {
        let mut app = test_app();
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        let p = spawn_player(&mut app, 5, 5);
        send_action(&mut app, p, "ability.guard", None, None);
        app.update();
        // Should not panic, AP should be spent
        let ap = app
            .world()
            .get::<Pools>(p)
            .unwrap()
            .get(PoolKind::ActionPoints)
            .unwrap()
            .current;
        assert_eq!(ap, 2);
    }

    #[test]
    fn denial_reason_is_displayable() {
        let reasons = vec![
            DenialReason::NotEnoughPool(PoolKind::ActionPoints),
            DenialReason::BlockedTile,
            DenialReason::OutOfRange,
            DenialReason::NoTarget,
            DenialReason::InvalidTarget,
            DenialReason::ActorDefeated,
            DenialReason::Other("test".into()),
        ];
        // All should have non-empty Debug output
        for r in &reasons {
            assert!(!format!("{:?}", r).is_empty());
        }
    }
}
