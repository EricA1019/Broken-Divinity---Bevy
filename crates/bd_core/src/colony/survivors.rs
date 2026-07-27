//! Survivors — colony inhabitants with tasks, needs, and mood tracking.

use std::{collections::HashSet, fmt};

use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use serde::{Deserialize, Serialize};

use crate::{
    actions::{ActionDefinition, Effect, Requirement},
    components::{Name, Position, ResourceNode},
    pathfinding::{AStarPathfinder, Pathfinder},
    pools::{Pool, Pools},
    signals::{DeltaTag, PoolKind},
};

// ── Constants ──

/// Maximum number of survivors allowed.
pub const MAX_SURVIVORS: u32 = 10;

/// Food units consumed per survivor per day.
pub const FOOD_PER_SURVIVOR_PER_DAY: i32 = 1;

/// Maximum mood value.
pub const MOOD_MAX: i32 = 100;

/// Mood penalty when a survivor goes unfed.
pub const MOOD_STARVATION_PENALTY: i32 = 10;

/// Mood bonus when a survivor rests.
pub const MOOD_REST_BONUS: i32 = 5;

// ── Movement constants ──

/// Maximum tiles from center an idle survivor will wander.
pub const SURVIVOR_WANDER_RADIUS: i32 = 8;
/// Manhattan range a gathering survivor will walk toward a resource node.
pub const SURVIVOR_GATHER_RANGE: i32 = 15;
/// Tiles per turn survivors move (currently 1).
pub const SURVIVOR_SPEED: i32 = 1;

// ── Components ──

/// Marker component for survivor entities.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Survivor;

/// Current task assignment for a survivor.
#[derive(Component, Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SurvivorTask {
    #[default]
    Idle,
    Gathering(PoolKind),
    Defending,
    AssignedTo(u64),
    Resting,
}

/// Player-facing state derived from a survivor's durable task and physical
/// relationship to its target.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub enum WorkerActivity {
    Idle,
    EnRoute {
        target: String,
        target_position: Position,
        distance: i32,
    },
    Working {
        target: String,
        target_position: Position,
    },
    Blocked {
        target: String,
        target_position: Option<Position>,
        reason: WorkerBlockedReason,
    },
    Resting,
    Defending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerBlockedReason {
    MissingTarget,
    TargetUnavailable,
    NoAdjacentWorkTile,
    NoRoute,
    DestinationReserved,
}

impl fmt::Display for WorkerBlockedReason {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::MissingTarget => "assigned target no longer exists",
            Self::TargetUnavailable => "no matching available target",
            Self::NoAdjacentWorkTile => "target has no adjacent work tile",
            Self::NoRoute => "no route to an adjacent work tile",
            Self::DestinationReserved => "next work tile is reserved",
        })
    }
}

pub fn cardinally_adjacent(left: Position, right: Position) -> bool {
    (left.x - right.x).abs() + (left.y - right.y).abs() == 1
}

// ── Default survivor pools ──

/// Create the default Pools for a survivor (Mood pool + basic stats).
pub fn default_survivor_pools() -> Pools {
    Pools::new(vec![
        Pool::new(PoolKind::Mood, MOOD_MAX, 0, MOOD_MAX),
        Pool::new(PoolKind::ActionPoints, 3, 0, 3),
    ])
}

// ── Action definitions ──

/// Assign the nearest survivor to gathering (produces +1 food/day).
pub fn register_assign_gathering_action() -> ActionDefinition {
    register_targeted_gathering_action(
        "ability.assign_gathering",
        "Assign Supplies Gathering",
        PoolKind::Supplies,
    )
}

pub fn register_gather_supplies_action() -> ActionDefinition {
    register_targeted_gathering_action(
        "ability.gather_supplies",
        "Gather Supplies",
        PoolKind::Supplies,
    )
}

pub fn register_gather_materials_action() -> ActionDefinition {
    register_targeted_gathering_action(
        "ability.gather_materials",
        "Gather Materials",
        PoolKind::Materials,
    )
}

pub fn register_gather_plants_action() -> ActionDefinition {
    register_targeted_gathering_action(
        "ability.gather_plants",
        "Gather Plants",
        PoolKind::WildPlants,
    )
}

fn register_targeted_gathering_action(id: &str, label: &str, kind: PoolKind) -> ActionDefinition {
    ActionDefinition {
        id: id.into(),
        label: label.into(),
        requirements: vec![Requirement::TargetHasComponent(
            "bd_core::colony::survivors::Survivor",
        )],
        cost_effects: vec![],
        effects: vec![Effect::SetSurvivorTask(format!("Gather:{kind:?}"))],
    }
}

pub fn register_assign_defending_action() -> ActionDefinition {
    ActionDefinition {
        id: "ability.assign_defending".into(),
        label: "Assign Defending".into(),
        requirements: vec![Requirement::TargetHasComponent(
            "bd_core::colony::survivors::Survivor",
        )],
        cost_effects: vec![],
        effects: vec![Effect::SetSurvivorTask("Defending".into())],
    }
}

pub fn register_assign_resting_action() -> ActionDefinition {
    ActionDefinition {
        id: "ability.assign_resting".into(),
        label: "Assign Resting".into(),
        requirements: vec![Requirement::TargetHasComponent(
            "bd_core::colony::survivors::Survivor",
        )],
        cost_effects: vec![],
        effects: vec![Effect::SetSurvivorTask("Resting".into())],
    }
}

pub fn register_assign_idle_action() -> ActionDefinition {
    ActionDefinition {
        id: "ability.assign_idle".into(),
        label: "Assign Idle".into(),
        requirements: vec![Requirement::TargetHasComponent(
            "bd_core::colony::survivors::Survivor",
        )],
        cost_effects: vec![],
        effects: vec![Effect::SetSurvivorTask("Idle".into())],
    }
}

/// Register the unassign_task action.
pub fn register_unassign_task_action() -> ActionDefinition {
    ActionDefinition {
        id: "ability.unassign_task".into(),
        label: "Unassign Task".into(),
        requirements: vec![Requirement::TargetHasComponent(
            "bd_core::colony::survivors::Survivor",
        )],
        cost_effects: vec![],
        effects: vec![Effect::SetSurvivorTask("Idle".into())],
    }
}

// ── Systems ──

/// Process AssignToStation messages: set a survivor's SurvivorTask to
/// AssignedTo referencing the station entity by its raw index.
#[derive(SystemParam)]
pub struct StationAssignmentTargets<'w, 's> {
    names: Query<'w, 's, &'static crate::components::Name>,
    cargo: Query<'w, 's, &'static crate::colony::logistics::Cargo>,
    stations: Query<
        'w,
        's,
        Entity,
        (
            With<crate::colony::stations::Station>,
            Without<crate::colony::stations::ConstructionSite>,
        ),
    >,
    survivors: Query<'w, 's, Entity, With<Survivor>>,
}

pub fn process_station_assignments(
    mut commands: Commands,
    mut messages: bevy_ecs::message::MessageReader<crate::signals::AssignToStation>,
    mut game_log: ResMut<crate::gamelog::GameLog>,
    mut colony_resources: ResMut<crate::colony::production::ColonyResources>,
    targets: StationAssignmentTargets,
) {
    for msg in messages.read() {
        // Validate both entities still exist
        if targets.stations.get(msg.station).is_err() {
            game_log.push(
                "Cannot assign: station no longer exists.".to_string(),
                crate::gamelog::LogLevel::Warn,
            );
            continue;
        }
        if targets.survivors.get(msg.survivor).is_err() {
            game_log.push(
                "Cannot assign: survivor no longer exists.".to_string(),
                crate::gamelog::LogLevel::Warn,
            );
            continue;
        }
        let station_index = msg.station.to_bits();
        if let Ok(cargo) = targets.cargo.get(msg.survivor) {
            crate::colony::logistics::deposit_cargo(&mut colony_resources, cargo);
        }
        commands
            .entity(msg.survivor)
            .insert(SurvivorTask::AssignedTo(station_index))
            .remove::<(
                crate::colony::logistics::LogisticsJob,
                crate::colony::logistics::Cargo,
            )>();
        let survivor_name = targets
            .names
            .get(msg.survivor)
            .map(|n| n.0.as_str())
            .unwrap_or("Survivor");
        game_log.push(
            format!("{} assigned to station.", survivor_name),
            crate::gamelog::LogLevel::Info,
        );
    }
}

/// Consumes shelter resources each day change: food per survivor, mood penalties.
/// P0-A fix: checks colony-level ColonyResources instead of entity-level Supplies
/// (survivors don't have entity-level Supplies — they only have Mood + AP).
#[allow(clippy::type_complexity)]
pub(crate) fn consume_shelter_resources(
    query: Query<(Entity, &mut Pools, Option<&SurvivorTask>, Option<&Position>)>,
    stations: Query<
        (Entity, &Position, &crate::colony::stations::StationType),
        (
            With<crate::colony::stations::Station>,
            Without<crate::colony::stations::ConstructionSite>,
        ),
    >,
    mut days: bevy_ecs::message::MessageReader<crate::time::DayAdvanced>,
    mut pool_delta_writer: bevy_ecs::message::MessageWriter<crate::signals::PoolDeltaRequested>,
    draft: Res<crate::colony::production::DailyCycleDraft>,
    station_catalog: Res<crate::colony::stations::StationCatalog>,
) {
    if days.read().next().is_none() {
        return;
    }

    let starved = draft
        .0
        .as_ref()
        .is_some_and(|summary| summary.starved_survivors > 0);
    let station_snapshots = stations
        .iter()
        .map(
            |(entity, position, station_type)| crate::colony::production::StationWorkSnapshot {
                entity_bits: entity.to_bits(),
                station_type: *station_type,
                position: *position,
            },
        )
        .collect::<Vec<_>>();

    for (entity, pools, task, position) in query.iter() {
        // Skip non-survivors
        if pools.get(PoolKind::Mood).is_none() {
            continue;
        }

        if starved {
            // Starvation: mood penalty when colony is out of supplies
            pool_delta_writer.write(crate::signals::PoolDeltaRequested {
                source: None,
                target: entity,
                kind: PoolKind::Mood,
                amount: -MOOD_STARVATION_PENALTY,
                tags: vec![DeltaTag::Action],
                reason: "starvation".into(),
            });
        }

        // Resting survivors recover mood
        if matches!(task, Some(SurvivorTask::Resting)) {
            pool_delta_writer.write(crate::signals::PoolDeltaRequested {
                source: None,
                target: entity,
                kind: PoolKind::Mood,
                amount: MOOD_REST_BONUS,
                tags: vec![DeltaTag::Recovery],
                reason: "rest recovery".into(),
            });
        }

        if let (Some(task), Some(position)) = (task, position)
            && let crate::colony::production::PhysicalWorkEvaluation::Contributes(
                crate::colony::production::PhysicalWorkContribution::Station(station),
            ) = crate::colony::production::evaluate_physical_work(
                &crate::colony::production::SurvivorWorkSnapshot {
                    task: task.clone(),
                    position: *position,
                },
                &station_snapshots,
                &[],
            )
            && let Some(blueprint) = station_catalog.get(station.station_type)
            && let crate::colony::stations::StationEffect::RestoreWorkerMood { amount } =
                blueprint.effect
        {
            pool_delta_writer.write(crate::signals::PoolDeltaRequested {
                source: None,
                target: entity,
                kind: PoolKind::Mood,
                amount,
                tags: vec![DeltaTag::Recovery],
                reason: "bed recovery".into(),
            });
        }
    }
}

// ── Survivor Movement ──

#[derive(Debug, Clone)]
struct WorkerStepState {
    entity: Entity,
    name: String,
    position: Position,
    task: SurvivorTask,
    activity: Option<WorkerActivity>,
}

#[derive(Debug, Clone)]
pub(crate) struct WorkTarget {
    pub(crate) label: String,
    pub(crate) position: Position,
}

#[derive(Resource, Debug, Default)]
pub(crate) struct RecomputingWorkerActivity;

fn adjacent_work_tiles(map: &crate::map::SmokeMap, target: Position) -> Vec<Position> {
    [
        Position {
            x: target.x + 1,
            y: target.y,
        },
        Position {
            x: target.x - 1,
            y: target.y,
        },
        Position {
            x: target.x,
            y: target.y + 1,
        },
        Position {
            x: target.x,
            y: target.y - 1,
        },
    ]
    .into_iter()
    .filter(|candidate| map.is_walkable(candidate.x, candidate.y))
    .collect()
}

pub(crate) fn choose_worker_path(
    map: &crate::map::SmokeMap,
    start: Position,
    targets: &[WorkTarget],
    blocked: &HashSet<Position>,
) -> Result<(WorkTarget, Vec<Position>), WorkerBlockedReason> {
    let mut saw_work_tile = false;
    let mut routes = Vec::new();
    for target in targets {
        for work_tile in adjacent_work_tiles(map, target.position) {
            if blocked.contains(&work_tile) && work_tile != start {
                continue;
            }
            saw_work_tile = true;
            let mut route_blockers = blocked.clone();
            route_blockers.remove(&start);
            route_blockers.remove(&work_tile);
            if let Some(path) = AStarPathfinder.find_path(map, start, work_tile, &route_blockers) {
                routes.push((path, target.clone(), work_tile));
            }
        }
    }
    if !saw_work_tile {
        return Err(WorkerBlockedReason::NoAdjacentWorkTile);
    }
    routes
        .into_iter()
        .min_by_key(|(path, target, work_tile)| {
            (
                path.len(),
                target.position.y,
                target.position.x,
                work_tile.y,
                work_tile.x,
            )
        })
        .map(|(path, target, _)| (target, path))
        .ok_or(WorkerBlockedReason::NoRoute)
}

fn target_candidates(
    task: &SurvivorTask,
    stations: &[(Entity, Position, String)],
    nodes: &[(Position, ResourceNode, String)],
) -> Result<Vec<WorkTarget>, WorkerBlockedReason> {
    match task {
        SurvivorTask::AssignedTo(bits) => stations
            .iter()
            .find(|(entity, _, _)| entity.to_bits() == *bits)
            .map(|(_, position, label)| {
                vec![WorkTarget {
                    label: label.clone(),
                    position: *position,
                }]
            })
            .ok_or(WorkerBlockedReason::MissingTarget),
        SurvivorTask::Gathering(kind) => {
            let targets = nodes
                .iter()
                .filter(|(_, node, _)| {
                    !node.depleted && crate::colony::resources::pool_for_node(node.kind) == *kind
                })
                .map(|(position, _, label)| WorkTarget {
                    label: label.clone(),
                    position: *position,
                })
                .collect::<Vec<_>>();
            if targets.is_empty() {
                Err(WorkerBlockedReason::TargetUnavailable)
            } else {
                Ok(targets)
            }
        }
        SurvivorTask::Idle | SurvivorTask::Defending | SurvivorTask::Resting => Ok(Vec::new()),
    }
}

fn is_currently_contributing(
    task: &SurvivorTask,
    position: Position,
    stations: &[(Entity, Position, String)],
    nodes: &[(Position, ResourceNode, String)],
) -> bool {
    let station_snapshots = stations
        .iter()
        .map(
            |(entity, position, _)| crate::colony::production::StationWorkSnapshot {
                entity_bits: entity.to_bits(),
                station_type: crate::colony::stations::StationType::Custom(0),
                position: *position,
            },
        )
        .collect::<Vec<_>>();
    let node_snapshots = nodes
        .iter()
        .map(
            |(position, node, _)| crate::colony::production::ResourceWorkSnapshot {
                kind: node.kind,
                position: *position,
                depleted: node.depleted,
            },
        )
        .collect::<Vec<_>>();
    matches!(
        crate::colony::production::evaluate_physical_work(
            &crate::colony::production::SurvivorWorkSnapshot {
                task: task.clone(),
                position,
            },
            &station_snapshots,
            &node_snapshots,
        ),
        crate::colony::production::PhysicalWorkEvaluation::Contributes(_)
    )
}

fn resolve_worker_activity(
    worker: &WorkerStepState,
    map: &crate::map::SmokeMap,
    stations: &[(Entity, Position, String)],
    nodes: &[(Position, ResourceNode, String)],
    blocked: &HashSet<Position>,
) -> (Position, WorkerActivity) {
    match worker.task {
        SurvivorTask::Idle => (worker.position, WorkerActivity::Idle),
        SurvivorTask::Resting => (worker.position, WorkerActivity::Resting),
        SurvivorTask::Defending => (worker.position, WorkerActivity::Defending),
        SurvivorTask::AssignedTo(_) | SurvivorTask::Gathering(_) => {
            let targets = match target_candidates(&worker.task, stations, nodes) {
                Ok(targets) => targets,
                Err(reason) => {
                    return (
                        worker.position,
                        WorkerActivity::Blocked {
                            target: match &worker.task {
                                SurvivorTask::AssignedTo(_) => "assigned station".into(),
                                SurvivorTask::Gathering(kind) => format!("{kind:?} node"),
                                _ => "target".into(),
                            },
                            target_position: None,
                            reason,
                        },
                    );
                }
            };
            match choose_worker_path(map, worker.position, &targets, blocked) {
                Ok((target, path))
                    if path.len() <= 1
                        && is_currently_contributing(
                            &worker.task,
                            worker.position,
                            stations,
                            nodes,
                        ) =>
                {
                    (
                        worker.position,
                        WorkerActivity::Working {
                            target: target.label,
                            target_position: target.position,
                        },
                    )
                }
                Ok((target, path)) if path.len() <= 1 => (
                    worker.position,
                    WorkerActivity::Blocked {
                        target: target.label,
                        target_position: Some(target.position),
                        reason: WorkerBlockedReason::TargetUnavailable,
                    },
                ),
                Ok((target, path)) => {
                    let next = path[1];
                    let activity = if cardinally_adjacent(next, target.position) {
                        WorkerActivity::Working {
                            target: target.label,
                            target_position: target.position,
                        }
                    } else {
                        WorkerActivity::EnRoute {
                            target: target.label,
                            target_position: target.position,
                            distance: i32::try_from(path.len().saturating_sub(2))
                                .unwrap_or(i32::MAX),
                        }
                    };
                    (next, activity)
                }
                Err(reason) => (
                    worker.position,
                    WorkerActivity::Blocked {
                        target: targets
                            .first()
                            .map_or_else(|| "target".into(), |target| target.label.clone()),
                        target_position: targets.first().map(|target| target.position),
                        reason,
                    },
                ),
            }
        }
    }
}

fn observe_worker_activity(
    worker: &WorkerStepState,
    map: &crate::map::SmokeMap,
    stations: &[(Entity, Position, String)],
    nodes: &[(Position, ResourceNode, String)],
    blocked: &HashSet<Position>,
) -> WorkerActivity {
    match worker.task {
        SurvivorTask::Idle => WorkerActivity::Idle,
        SurvivorTask::Resting => WorkerActivity::Resting,
        SurvivorTask::Defending => WorkerActivity::Defending,
        SurvivorTask::AssignedTo(_) | SurvivorTask::Gathering(_) => {
            let fallback = match &worker.task {
                SurvivorTask::AssignedTo(_) => "assigned station".to_string(),
                SurvivorTask::Gathering(kind) => format!("{kind:?} node"),
                _ => "target".to_string(),
            };
            let targets = match target_candidates(&worker.task, stations, nodes) {
                Ok(targets) => targets,
                Err(reason) => {
                    return WorkerActivity::Blocked {
                        target: fallback,
                        target_position: None,
                        reason,
                    };
                }
            };
            match choose_worker_path(map, worker.position, &targets, blocked) {
                Ok((target, path))
                    if path.len() <= 1
                        && is_currently_contributing(
                            &worker.task,
                            worker.position,
                            stations,
                            nodes,
                        ) =>
                {
                    WorkerActivity::Working {
                        target: target.label,
                        target_position: target.position,
                    }
                }
                Ok((target, path)) if path.len() <= 1 => WorkerActivity::Blocked {
                    target: target.label,
                    target_position: Some(target.position),
                    reason: WorkerBlockedReason::TargetUnavailable,
                },
                Ok((target, path)) => WorkerActivity::EnRoute {
                    target: target.label,
                    target_position: target.position,
                    distance: i32::try_from(path.len().saturating_sub(1)).unwrap_or(i32::MAX),
                },
                Err(reason) => WorkerActivity::Blocked {
                    target: targets
                        .first()
                        .map_or(fallback, |target| target.label.clone()),
                    target_position: targets.first().map(|target| target.position),
                    reason,
                },
            }
        }
    }
}

/// Resolve derived activity every frame and move assigned survivors once for
/// each logical Outpost worker step compiled from the accepted action.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub(crate) fn process_survivor_movement(
    mut commands: Commands,
    survivors: Query<
        (
            Entity,
            Option<&Name>,
            &Position,
            &SurvivorTask,
            Option<&WorkerActivity>,
        ),
        (
            With<Survivor>,
            Without<crate::colony::logistics::LogisticsJob>,
            Without<crate::colony::stations::AutoConstructing>,
        ),
    >,
    nodes: Query<(&Position, &ResourceNode, Option<&Name>), Without<Survivor>>,
    stations: Query<
        (Entity, &Position, Option<&Name>),
        (With<crate::colony::stations::Station>, Without<Survivor>),
    >,
    player: Query<&Position, (With<crate::components::Player>, Without<Survivor>)>,
    map: Res<crate::map::SmokeMap>,
    mode: Res<crate::spatial::GameMode>,
    mut time_plan: ResMut<crate::time::TimeAdvancePlan>,
    mut game_log: ResMut<crate::gamelog::GameLog>,
    recomputing: Option<Res<RecomputingWorkerActivity>>,
) {
    if *mode != crate::spatial::GameMode::Outpost {
        time_plan.outpost_worker_steps = 0;
        return;
    }

    let mut workers = survivors
        .iter()
        .map(|(entity, name, position, task, activity)| WorkerStepState {
            entity,
            name: name
                .map(|name| name.0.clone())
                .unwrap_or_else(|| format!("Survivor {}", entity.to_bits())),
            position: *position,
            task: task.clone(),
            activity: activity.cloned(),
        })
        .collect::<Vec<_>>();
    workers.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then_with(|| left.entity.to_bits().cmp(&right.entity.to_bits()))
    });
    let stations = stations
        .iter()
        .map(|(entity, position, name)| {
            (
                entity,
                *position,
                name.map_or_else(|| "Station".into(), |name| name.0.clone()),
            )
        })
        .collect::<Vec<_>>();
    let nodes = nodes
        .iter()
        .map(|(position, node, name)| {
            (
                *position,
                node.clone(),
                name.map_or_else(|| format!("{:?}", node.kind), |name| name.0.clone()),
            )
        })
        .collect::<Vec<_>>();
    let permanent = stations
        .iter()
        .map(|(_, position, _)| *position)
        .chain(nodes.iter().map(|(position, _, _)| *position))
        .chain(player.iter().copied())
        .collect::<HashSet<_>>();

    let steps = time_plan.outpost_worker_steps;
    time_plan.outpost_worker_steps = 0;
    let iterations = steps.max(1);
    for iteration in 0..iterations {
        let allow_movement = iteration < steps;
        let mut occupied = workers
            .iter()
            .map(|worker| worker.position)
            .collect::<HashSet<_>>();
        for worker in &mut workers {
            let mut blocked = permanent.clone();
            blocked.extend(occupied.iter().copied());
            blocked.remove(&worker.position);

            let (next_position, final_activity) = if allow_movement {
                resolve_worker_activity(worker, &map, &stations, &nodes, &blocked)
            } else {
                (
                    worker.position,
                    observe_worker_activity(worker, &map, &stations, &nodes, &blocked),
                )
            };

            occupied.remove(&worker.position);
            occupied.insert(next_position);
            worker.position = next_position;
            if matches!(final_activity, WorkerActivity::Blocked { .. })
                && worker.activity.as_ref() != Some(&final_activity)
                && recomputing.is_none()
                && let WorkerActivity::Blocked { target, reason, .. } = &final_activity
            {
                game_log.push(
                    format!(
                        "{} is Blocked en route to {}: {}.",
                        worker.name, target, reason
                    ),
                    crate::gamelog::LogLevel::Warn,
                );
            }
            worker.activity = Some(final_activity);
        }
    }

    for worker in workers {
        commands.entity(worker.entity).insert((
            worker.position,
            worker.activity.unwrap_or(WorkerActivity::Idle),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Position;
    use bevy_app::App;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(crate::BdCorePlugin);
        app.world_mut()
            .insert_resource(crate::spatial::GameMode::Outpost);
        app
    }

    fn spawn_survivor(app: &mut App, x: i32, y: i32) -> Entity {
        app.world_mut()
            .spawn((
                Survivor,
                SurvivorTask::Idle,
                Position { x, y },
                default_survivor_pools(),
            ))
            .id()
    }

    #[test]
    fn survivor_has_mood_pool() {
        let mut app = test_app();
        let s = spawn_survivor(&mut app, 5, 5);
        let pools = app.world().get::<Pools>(s).unwrap();
        let mood = pools.get(PoolKind::Mood).unwrap();
        assert_eq!(mood.current, MOOD_MAX);
        assert_eq!(mood.max, MOOD_MAX);
    }

    #[test]
    fn survivor_max_count_is_ten() {
        assert_eq!(MAX_SURVIVORS, 10);
    }

    #[test]
    fn unassign_task_action_has_correct_id() {
        let def = register_unassign_task_action();
        assert_eq!(def.id, "ability.unassign_task");
    }

    // ── P0-A: Starvation bug tests ──

    #[test]
    fn survivors_keep_mood_when_colony_has_food() {
        let mut app = test_app();
        // Spawn 3 survivors
        for i in 0..3 {
            spawn_survivor(&mut app, 5 + i * 5, 5);
        }
        // Set colony supplies to 10
        app.world_mut()
            .resource_mut::<crate::colony::production::ColonyResources>()
            .pools
            .get_mut(PoolKind::Supplies)
            .unwrap()
            .current = 10;
        for _ in 0..crate::time::TURNS_PER_DAY {
            app.world_mut()
                .resource_mut::<crate::time::ShouldAdvanceTime>()
                .0 = true;
            app.update();
        }
        // Process the emitted day boundary transaction.
        app.update();
        // All survivors should still have max mood
        let mut query = app.world_mut().query::<&crate::pools::Pools>();
        for pools in query.iter(app.world()) {
            if let Some(mood) = pools.get(PoolKind::Mood) {
                assert_eq!(
                    mood.current, MOOD_MAX,
                    "Survivor mood should stay at {} when colony has food, got {}",
                    MOOD_MAX, mood.current
                );
                break;
            }
        }
    }

    #[test]
    fn survivors_starve_when_colony_has_no_food() {
        let mut app = test_app();
        // Spawn 3 survivors
        for i in 0..3 {
            spawn_survivor(&mut app, 5 + i * 5, 5);
        }
        // Set colony supplies to 0
        app.world_mut()
            .resource_mut::<crate::colony::production::ColonyResources>()
            .pools
            .get_mut(PoolKind::Supplies)
            .unwrap()
            .current = 0;
        for _ in 0..crate::time::TURNS_PER_DAY {
            app.world_mut()
                .resource_mut::<crate::time::ShouldAdvanceTime>()
                .0 = true;
            app.update();
        }
        app.update();
        // Survivors should have lost mood
        let mut query = app.world_mut().query::<&crate::pools::Pools>();
        for pools in query.iter(app.world()) {
            if let Some(mood) = pools.get(PoolKind::Mood) {
                assert!(
                    mood.current < MOOD_MAX,
                    "Survivor mood should drop when colony has no food, got {}",
                    mood.current
                );
                break;
            }
        }
    }
}
