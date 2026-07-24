//! Action system — validated actions with requirements, costs, and effects.
//!
//! All player/AI actions go through: intent → validate → cost → effect.
//! No system directly mutates state outside this pipeline.

use bevy_app::App;
use bevy_ecs::prelude::*;

use crate::{
    BdSet,
    colony::production::ColonyResources,
    colony::survivors::{Survivor, SurvivorTask},
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
use serde::{Deserialize, Serialize};

// ── Constants ──

/// Base damage dealt by a basic melee attack.
const ATTACK_DAMAGE_BASE: i32 = 5;
/// Action Points consumed by a basic attack.
const ATTACK_AP_COST: i32 = 1;
/// Log message emitted on attack.
const ATTACK_LOG: &str = "You attack!";

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
    /// A global resource pool must be at or above a threshold.
    ResourcePoolAbove(PoolKind, i32),
    /// Target must have a specific component.
    TargetHasComponent(&'static str),
    /// Target item must be contained by the acting entity.
    TargetContainedByActor,
    /// Target must be idle (has idle task).
    TargetTaskIsIdle,
    /// Player must be within range of the target entity.
    PlayerHasEntityInRange(u32),
    /// Target tile must be vacant (no blocking entities) — for build actions.
    TileVacant,
    /// Actor must be standing on an extraction/exit tile.
    AtExit,
}

/// An effect produced by a successful action.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Spawn an entity from a blueprint at the target position.
    SpawnEntity(String),
    /// Set a survivor's task.
    SetSurvivorTask(String),
    /// Set a named flag (event state, global markers).
    Flag(String, bool),
    /// Request a mode transition after the action resolves.
    RequestTransition(crate::spatial::GameMode),
    /// Request the inventory pipeline to use the pending target item.
    RequestUseItem,
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

    /// Register a new action definition.
    pub fn register(&mut self, def: ActionDefinition) {
        self.definitions.push(def);
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
                    effects: vec![Effect::Log("You wait.".into(), LogLevel::Info)],
                },
                ActionDefinition {
                    id: "ability.attack".into(),
                    label: "Attack".into(),
                    requirements: vec![
                        Requirement::HasPoolAtLeast(PoolKind::ActionPoints, ATTACK_AP_COST),
                        Requirement::TargetExists,
                        Requirement::TargetHostile,
                        Requirement::TargetInRange(1),
                    ],
                    cost_effects: vec![Effect::PoolDelta {
                        kind: PoolKind::ActionPoints,
                        amount: -ATTACK_AP_COST,
                        tags: vec![DeltaTag::Action],
                        reason: "attack cost".into(),
                    }],
                    effects: vec![
                        Effect::PoolDelta {
                            kind: PoolKind::Health,
                            amount: -ATTACK_DAMAGE_BASE,
                            tags: vec![DeltaTag::Physical],
                            reason: "attack hit".into(),
                        },
                        Effect::Log(ATTACK_LOG.into(), LogLevel::Combat),
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
                ActionDefinition {
                    id: "ability.extract".into(),
                    label: "Extract".into(),
                    requirements: vec![Requirement::EntityAlive, Requirement::AtExit],
                    cost_effects: vec![],
                    effects: vec![
                        Effect::RequestTransition(crate::spatial::GameMode::Outpost),
                        Effect::Log("You extract from the dungeon.".into(), LogLevel::Info),
                    ],
                },
                ActionDefinition {
                    id: "ability.repair".into(),
                    label: "Repair".into(),
                    requirements: vec![
                        Requirement::HasPoolAtLeast(PoolKind::ActionPoints, 1),
                        Requirement::EntityAlive,
                    ],
                    cost_effects: vec![Effect::PoolDelta {
                        kind: PoolKind::ActionPoints,
                        amount: -1,
                        tags: vec![DeltaTag::Action],
                        reason: "repair cost".into(),
                    }],
                    effects: vec![Effect::Log(
                        "You repair your equipment.".into(),
                        LogLevel::Info,
                    )],
                },
                ActionDefinition {
                    id: "ability.use_item".into(),
                    label: "Use Item".into(),
                    requirements: vec![
                        Requirement::HasPoolAtLeast(PoolKind::ActionPoints, 1),
                        Requirement::EntityAlive,
                        Requirement::TargetExists,
                        Requirement::TargetHasComponent("bd_core::inventory::Usable"),
                        Requirement::TargetContainedByActor,
                    ],
                    cost_effects: vec![Effect::PoolDelta {
                        kind: PoolKind::ActionPoints,
                        amount: -1,
                        tags: vec![DeltaTag::Action],
                        reason: "use item cost".into(),
                    }],
                    effects: vec![Effect::RequestUseItem],
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
    mode: Res<crate::spatial::GameMode>,
    mut messages: bevy_ecs::message::MessageReader<ActionIntent>,
    mut denied_writer: bevy_ecs::message::MessageWriter<ActionDenied>,
    mut game_log: ResMut<GameLog>,
    mut trace: ResMut<SignalTrace>,
    colony_res: Res<ColonyResources>,
    actors: Query<(
        Entity,
        &Position,
        Option<&Pools>,
        Option<&Player>,
        Option<&crate::time::AwaitingEnemyPhase>,
    )>,
    targets: Query<(Entity, &Position, Option<&Player>)>,
    all_entities: Query<Entity>,
    contained_items: Query<(Entity, &crate::relationships::ContainedIn)>,
    survivors: Query<(Entity, &SurvivorTask), With<Survivor>>,
    blocked_positions: Query<&Position, With<BlocksMovement>>,
    exit_positions: Query<&Position, With<crate::components::ExitTile>>,
) {
    let target_positions: Vec<(Entity, &Position, Option<&Player>)> = targets.iter().collect();

    for intent in messages.read() {
        trace.push(
            "Validation",
            "ActionIntent",
            format!("actor={:?} action={}", intent.actor, intent.action_id),
        );
        let Ok((_, actor_pos, pools, player_flag, awaiting_enemy_phase)) = actors.get(intent.actor)
        else {
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

        if player_flag.is_some() && awaiting_enemy_phase.is_some() {
            denied = Some(DenialReason::Other(
                "Enemy phase is resolving; wait for the next turn.".into(),
            ));
        }

        for req in &def.requirements {
            if denied.is_some() {
                break;
            }
            match req {
                Requirement::HasPoolAtLeast(kind, min) => {
                    // P21: Free movement in colony/outpost mode
                    let is_colony_move = *kind == PoolKind::ActionPoints
                        && *mode == crate::spatial::GameMode::Outpost
                        && intent.action_id == "ability.move";
                    if !is_colony_move {
                        let current = pools.and_then(|p| p.get(*kind)).map_or(0, |p| p.current);
                        if current < *min {
                            denied = Some(DenialReason::NotEnoughPool(*kind));
                            break;
                        }
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
                Requirement::ResourcePoolAbove(kind, min) => {
                    // Check a global ColonyResources pool instead of entity pool
                    let current = colony_res.pools.get(*kind).map_or(0, |p| p.current);
                    if current < *min {
                        denied = Some(DenialReason::NotEnoughPool(*kind));
                        break;
                    }
                }
                Requirement::TargetHasComponent(component_name) => {
                    let has_component = intent.target.map_or(false, |t| match *component_name {
                        "Survivor" => survivors.iter().any(|(e, _)| e == t),
                        _ => all_entities.iter().any(|e| e == t),
                    });
                    if !has_component {
                        denied = Some(DenialReason::InvalidTarget);
                        break;
                    }
                }
                Requirement::TargetContainedByActor => {
                    let contained = intent.target.is_some_and(|target| {
                        contained_items
                            .iter()
                            .any(|(item, container)| item == target && container.0 == intent.actor)
                    });
                    if !contained {
                        denied = Some(DenialReason::InvalidTarget);
                        break;
                    }
                }
                Requirement::TargetTaskIsIdle => {
                    let is_idle = intent.target.map_or(false, |t| {
                        survivors
                            .iter()
                            .find(|(e, _)| *e == t)
                            .map_or(false, |(_, task)| matches!(task, SurvivorTask::Idle))
                    });
                    if !is_idle {
                        denied = Some(DenialReason::Other("target not idle".into()));
                        break;
                    }
                }
                Requirement::PlayerHasEntityInRange(range) => {
                    let player_pos = actors
                        .iter()
                        .find(|(_, _, _, player, _)| player.is_some())
                        .map(|(_, p, _, _, _)| *p);
                    let target_pos = intent.target.and_then(|t| {
                        target_positions
                            .iter()
                            .find(|(e, _, _)| *e == t)
                            .map(|(_, p, _)| *p)
                    });
                    match (player_pos, target_pos) {
                        (Some(pp), Some(tp)) => {
                            let dist = (pp.x - tp.x).unsigned_abs() + (pp.y - tp.y).unsigned_abs();
                            if dist > *range {
                                denied = Some(DenialReason::OutOfRange);
                                break;
                            }
                        }
                        _ => {
                            denied = Some(DenialReason::NoTarget);
                            break;
                        }
                    }
                }
                Requirement::TileVacant => {
                    // Compute target tile position from actor position + direction
                    let actor_pos = actors
                        .iter()
                        .find(|(e, _, _, _, _)| *e == intent.actor)
                        .map(|(_, p, _, _, _)| *p);
                    if let (Some(pos), Some(dir)) = (actor_pos, intent.direction) {
                        let (dx, dy) = dir.delta();
                        let target_pos = crate::components::Position {
                            x: pos.x + dx,
                            y: pos.y + dy,
                        };
                        // Check if any blocking entity occupies that tile
                        let occupied = blocked_positions
                            .iter()
                            .any(|bp| bp.x == target_pos.x && bp.y == target_pos.y);
                        if occupied {
                            denied = Some(DenialReason::Other("tile occupied".into()));
                            break;
                        }
                    }
                }
                Requirement::AtExit => {
                    if !exit_positions.iter().any(|exit| *exit == *actor_pos) {
                        denied = Some(DenialReason::Other(
                            "You must stand at the exit to extract.".into(),
                        ));
                        break;
                    }
                }
            }
        }

        if let Some(reason) = denied {
            if player_flag.is_some() {
                let msg = match &reason {
                    DenialReason::NotEnoughPool(PoolKind::ActionPoints) => {
                        "Not enough ActionPoints. Wait (.) to restore 1 AP.".into()
                    }
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
    mut transition_writer: bevy_ecs::message::MessageWriter<crate::spatial::TransitionIntent>,
    mut action_result_writer: bevy_ecs::message::MessageWriter<crate::progression::ActionResolved>,
    mut use_item_writer: bevy_ecs::message::MessageWriter<crate::inventory::UseItemIntent>,
    mut pending_station: ResMut<crate::colony::stations::PendingStationBuild>,
    mut should_advance: ResMut<crate::time::ShouldAdvanceTime>,
    mut session: ResMut<crate::session::RunSession>,
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
        // Accepted gameplay actions resolve one player turn. UI-only actions
        // never enter this system, and extraction is a mode transition rather
        // than a combat turn.
        if player_flag.is_some() && is_turn_action(&pending.action_id) {
            should_advance.0 = true;
            should_advance.1 = true;
            commands
                .entity(entity)
                .insert(crate::time::AwaitingEnemyPhase);
        }
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
                Effect::ApplyStatus(status_id) => {
                    // Apply status to the entity
                    let defs = crate::statuses::default_status_definitions();
                    crate::statuses::apply_status(
                        entity,
                        status_id,
                        0, // use default duration from definition
                        None,
                        &mut commands,
                        &defs,
                    );
                    if player_flag.is_some() {
                        game_log.push(format!("You guard with {}.", status_id), LogLevel::Info);
                    }
                }
                Effect::SpawnEntity(blueprint_id) => {
                    use crate::colony::stations::{Station, StationType};
                    // Determine station type: check PendingStationBuild resource first,
                    // then fall back to parsing the blueprint_id.
                    let station_type = if let Some(st) = pending_station.0.take() {
                        st
                    } else {
                        match blueprint_id.as_str() {
                            "blueprint.station" | "blueprint.station.stove" => StationType::Stove,
                            "blueprint.station.altar" => StationType::Altar,
                            "blueprint.station.workshop" => StationType::Workshop,
                            "blueprint.station.bed" => StationType::Bed,
                            "blueprint.station.storage" => StationType::Storage,
                            _ => StationType::Stove,
                        }
                    };
                    // Build at tile in direction offset, not on player
                    let build_pos = pending
                        .direction
                        .map(|dir| {
                            let (dx, dy) = dir.delta();
                            crate::components::Position {
                                x: pos.x + dx,
                                y: pos.y + dy,
                            }
                        })
                        .unwrap_or(*pos);
                    commands.spawn((
                        Station,
                        station_type,
                        build_pos,
                        crate::components::BlocksMovement,
                        crate::components::Name(format!("{:?}", station_type)),
                    ));
                    if player_flag.is_some() {
                        game_log.push(format!("You build a {:?}.", station_type), LogLevel::Info);
                    }
                }
                Effect::SetSurvivorTask(task) => {
                    // Set the survivor's task based on the parameter string
                    if let Some(target) = pending.target {
                        use crate::colony::survivors::SurvivorTask;
                        let new_task = match task.as_str() {
                            "Idle" => SurvivorTask::Idle,
                            "Resting" => SurvivorTask::Resting,
                            "Gathering" => SurvivorTask::Gathering,
                            "Defending" => SurvivorTask::Defending,
                            _ => SurvivorTask::Idle,
                        };
                        commands.entity(target).insert(new_task);
                        if player_flag.is_some() {
                            game_log.push(
                                format!("Task set to {} for survivor.", task),
                                LogLevel::Info,
                            );
                        }
                    }
                }
                Effect::Flag(flag_name, value) => {
                    if player_flag.is_some() {
                        game_log.push(
                            format!("Flag '{}' set to {}", flag_name, value),
                            LogLevel::Info,
                        );
                    }
                }
                Effect::RequestTransition(target) => {
                    transition_writer.write(crate::spatial::TransitionIntent {
                        target: *target,
                        node_id: None,
                    });
                }
                Effect::RequestUseItem => {
                    if let Some(item) = pending.target {
                        use_item_writer.write(crate::inventory::UseItemIntent {
                            actor: entity,
                            item,
                        });
                    }
                }
            }
        }

        // Record only actions that reached effect resolution. Denied intents
        // never enter the run replay, keeping the replay stream meaningful.
        session.record_intent(pending.action_id.as_str());
        action_result_writer.write(crate::progression::ActionResolved {
            actor: entity,
            action_id: pending.action_id.clone(),
        });
        commands.entity(entity).remove::<PendingAction>();
    }
}

fn is_turn_action(action_id: &str) -> bool {
    matches!(
        action_id,
        "ability.move"
            | "ability.wait"
            | "ability.attack"
            | "ability.quick_attack"
            | "ability.aimed_attack"
            | "ability.guard"
            | "ability.repair"
            | "ability.use_item"
    )
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
    fn wait_advances_turn_and_triggers_ap_regen_to_max() {
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
        // P21: AP restored to max by regenerate_action_points, not by wait PoolDelta
        send_action(&mut app, p, "ability.wait", None, None);
        app.update();
        app.update(); // second frame: TurnJustAdvanced consumed, AP regen fires
        let ap = app
            .world()
            .get::<Pools>(p)
            .unwrap()
            .get(PoolKind::ActionPoints)
            .unwrap()
            .current;
        assert_eq!(ap, 3, "After wait, AP should regenerate to max (3), not +1");
    }

    #[test]
    fn attack_no_target_does_not_self_harm() {
        let mut app = test_app();
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        let p = spawn_player(&mut app, 5, 5);
        // Send attack with no target — should be denied by TargetExists
        send_action(&mut app, p, "ability.attack", None, None);
        app.update();
        // Player HP should be unchanged (no self-damage)
        let hp = app
            .world()
            .get::<Pools>(p)
            .unwrap()
            .get(PoolKind::Health)
            .unwrap()
            .current;
        assert_eq!(hp, 20, "attack with no target must not damage self");
        // Game log should contain the denial hint
        let log = app.world().resource::<GameLog>();
        let has_denial = log.iter().any(|e| e.message.contains("No target"));
        assert!(
            has_denial,
            "denial should log 'No target' hint, got: {:?}",
            log.iter().map(|e| &e.message).collect::<Vec<_>>()
        );
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
    fn extract_action_requests_outpost_transition() {
        let mut app = test_app();
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        app.world_mut()
            .insert_resource(crate::spatial::GameMode::Tactical);
        let player = spawn_player(&mut app, 5, 5);
        app.world_mut()
            .spawn((crate::components::ExitTile, Position { x: 5, y: 5 }));

        send_action(&mut app, player, "ability.extract", None, None);
        app.update();
        app.update();

        assert_eq!(
            *app.world().resource::<crate::spatial::GameMode>(),
            crate::spatial::GameMode::Outpost
        );
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
        app.update(); // second frame to process pool deltas from action effects
        // Damage goes to enemy, not self
        let dummy_hp = app
            .world()
            .get::<Pools>(dummy)
            .unwrap()
            .get(PoolKind::Health)
            .unwrap()
            .current;
        // P13: d100 variance means damage is 0.5x/1.0x/1.5x of base 5
        // Expected: 3, 5, or 8 damage. Accept any valid variance.
        let damage_dealt = 10 - dummy_hp;
        assert!(
            damage_dealt >= 2 && damage_dealt <= 8,
            "dummy should take 2-8 damage from base 5 with d100 variance, took {} (HP: {} -> {})",
            damage_dealt,
            10,
            dummy_hp
        );
        assert!(dummy_hp < 10, "dummy should take some damage, HP still 10");
        let player_hp = app
            .world()
            .get::<Pools>(p)
            .unwrap()
            .get(PoolKind::Health)
            .unwrap()
            .current;
        assert_eq!(player_hp, 20, "player should NOT take self-damage");
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

    #[test]
    fn ap_denial_includes_wait_hint() {
        let mut app = test_app();
        use crate::pools::Pools;
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
        app.world_mut()
            .resource_mut::<Messages<crate::signals::ActionIntent>>()
            .write(ActionIntent {
                actor: p,
                action_id: "ability.move".into(),
                direction: Some(Direction::East),
                target: None,
            });
        app.update();
        let log = app.world().resource::<GameLog>();
        let has_hint = log.iter().any(|e| e.message.contains("Wait (.)"));
        assert!(
            has_hint,
            "AP denial should include wait hint, got: {:?}",
            log.iter().map(|e| &e.message).collect::<Vec<_>>()
        );
    }

    #[test]
    fn item_pickup_logs_item_name() {
        let mut app = test_app();
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        use crate::inventory::Item;
        // Player at (5,5) with 1 AP (enough for one move)
        let p = app
            .world_mut()
            .spawn((
                Player,
                Position { x: 5, y: 5 },
                Pools::new(vec![
                    Pool::new(PoolKind::ActionPoints, 1, 0, 3),
                    Pool::new(PoolKind::Health, 10, 0, 10),
                ]),
            ))
            .id();
        // Item at (6,5) — NOT BlocksMovement so player can walk on it
        let _item = app
            .world_mut()
            .spawn((
                crate::components::Name("Healing Potion".into()),
                Item,
                Position { x: 6, y: 5 },
            ))
            .id();
        // Move east onto the item
        send_action(&mut app, p, "ability.move", Some(Direction::East), None);
        app.update();

        let log = app.world().resource::<GameLog>();
        let has_pickup = log.iter().any(|e| e.message.contains("Healing Potion"));
        assert!(
            has_pickup,
            "Item pickup log should mention item name, got: {:?}",
            log.iter().map(|e| &e.message).collect::<Vec<_>>()
        );
    }

    // ── P21: Turn model tests ──

    #[test]
    fn every_accepted_player_action_advances_time_once() {
        use crate::time::GameTime;
        let mut app = test_app();
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        app.world_mut()
            .insert_resource(crate::spatial::GameMode::Tactical);
        let p = spawn_player(&mut app, 5, 5);

        let turn_before = app.world().resource::<GameTime>().turn;

        // Move — accepted gameplay action advances one turn.
        send_action(&mut app, p, "ability.move", Some(Direction::East), None);
        app.update();
        let turn_after_move = app.world().resource::<GameTime>().turn;
        assert_eq!(
            turn_after_move,
            turn_before + 1,
            "Move should advance time once. Turn was {} before, {} after.",
            turn_before,
            turn_after_move
        );

        // Let the one-shot enemy phase boundary release the player lock before
        // issuing the next player action in this unit-level sequence.
        app.update();

        // Wait — also advances exactly one turn.
        send_action(&mut app, p, "ability.wait", None, None);
        app.update();
        let turn_after_wait = app.world().resource::<GameTime>().turn;
        assert_eq!(
            turn_after_wait,
            turn_before + 2,
            "Wait should advance time once. Turn was {} before, {} after.",
            turn_before,
            turn_after_wait
        );
    }
}
