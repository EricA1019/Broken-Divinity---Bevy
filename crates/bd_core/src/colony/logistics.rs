//! Durable, data-driven source-to-station production transitions.

use std::collections::HashSet;

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    colony::production::ColonyResources, components::Position, content::ColonyRecipeDefinition,
};

#[derive(Resource, Debug, Clone, Default)]
pub struct PendingRecipeAssignment(pub Option<String>);

pub fn register_assign_recipe_action() -> crate::actions::ActionDefinition {
    crate::actions::ActionDefinition {
        id: "ability.assign_recipe".into(),
        label: "Assign Production Recipe".into(),
        requirements: vec![
            crate::actions::Requirement::TargetExists,
            crate::actions::Requirement::TargetHasComponent("Survivor"),
        ],
        cost_effects: vec![],
        effects: vec![crate::actions::Effect::AssignTargetRecipe],
    }
}

pub(crate) fn process_recipe_assignments(
    mut commands: Commands,
    mut assignments: bevy_ecs::message::MessageReader<crate::signals::AssignRecipe>,
    content: Option<Res<crate::content::FoundationContent>>,
    survivors: Query<(), With<crate::colony::survivors::Survivor>>,
    cargo: Query<&Cargo>,
    mut colony_resources: ResMut<ColonyResources>,
    mut game_log: ResMut<crate::gamelog::GameLog>,
) {
    let Some(content) = content else {
        return;
    };
    for assignment in assignments.read() {
        if survivors.get(assignment.survivor).is_err()
            || !content
                .colony_recipes
                .iter()
                .any(|recipe| recipe.id == assignment.recipe_id)
        {
            game_log.push(
                "Cannot assign: survivor or recipe no longer exists.",
                crate::gamelog::LogLevel::Warn,
            );
            continue;
        }
        if let Ok(cargo) = cargo.get(assignment.survivor) {
            deposit_cargo(&mut colony_resources, cargo);
        }
        commands.entity(assignment.survivor).insert((
            crate::colony::survivors::SurvivorTask::Idle,
            LogisticsJob {
                recipe_id: assignment.recipe_id.clone(),
                stage: JobStage::ToSource,
                work_completed: 0,
                blocked: None,
            },
            Cargo::default(),
        ));
        game_log.push(
            format!("Production assigned: {}.", assignment.recipe_id),
            crate::gamelog::LogLevel::Info,
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobStage {
    ToSource,
    ReadyToGather,
    ToStation,
    ReadyToRefine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogisticsBlock {
    MissingSource,
    MissingStation,
    NoRoute,
    CargoMismatch,
    MissingInput,
}

#[derive(Component, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogisticsJob {
    pub recipe_id: String,
    pub stage: JobStage,
    #[serde(default)]
    pub work_completed: u32,
    #[serde(default)]
    pub blocked: Option<LogisticsBlock>,
}

#[derive(Component, Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cargo {
    pub resource_id: Option<String>,
    pub amount: u32,
}

pub fn deposit_cargo(resources: &mut ColonyResources, cargo: &Cargo) {
    if let Some(resource_id) = cargo.resource_id.as_ref().filter(|_| cargo.amount > 0) {
        *resources.raw.entry(resource_id.clone()).or_default() += cargo.amount;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogisticsTargetState {
    RouteStep(Position),
    AtWorkTile,
    Missing,
    NoRoute,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LogisticsTick {
    pub gathered_input: u32,
    pub consumed_input: u32,
    pub finished_output: u32,
    pub blocked: Option<LogisticsBlock>,
}

fn block_for_missing_target(stage: JobStage) -> LogisticsBlock {
    match stage {
        JobStage::ToSource | JobStage::ReadyToGather => LogisticsBlock::MissingSource,
        JobStage::ToStation | JobStage::ReadyToRefine => LogisticsBlock::MissingStation,
    }
}

pub fn tick_logistics(
    job: &mut LogisticsJob,
    cargo: &mut Cargo,
    position: &mut Position,
    recipe: &ColonyRecipeDefinition,
    target: LogisticsTargetState,
) -> LogisticsTick {
    let mut result = LogisticsTick::default();
    let denial = match target {
        LogisticsTargetState::Missing => Some(block_for_missing_target(job.stage)),
        LogisticsTargetState::NoRoute => Some(LogisticsBlock::NoRoute),
        _ => None,
    };
    if let Some(denial) = denial {
        job.blocked = Some(denial);
        result.blocked = Some(denial);
        return result;
    }
    job.blocked = None;

    match (job.stage, target) {
        (JobStage::ToSource | JobStage::ToStation, LogisticsTargetState::RouteStep(next)) => {
            *position = next;
        }
        (JobStage::ToSource, LogisticsTargetState::AtWorkTile) => {
            job.stage = JobStage::ReadyToGather;
            job.work_completed = 0;
        }
        (JobStage::ReadyToGather, LogisticsTargetState::AtWorkTile) => {
            if cargo
                .resource_id
                .as_deref()
                .is_some_and(|id| id != recipe.input_resource_id)
            {
                job.blocked = Some(LogisticsBlock::CargoMismatch);
                result.blocked = job.blocked;
                return result;
            }
            job.work_completed = job.work_completed.saturating_add(1);
            if job.work_completed >= recipe.gather_work_turns {
                cargo.resource_id = Some(recipe.input_resource_id.clone());
                cargo.amount = cargo.amount.saturating_add(recipe.input_amount);
                result.gathered_input = recipe.input_amount;
                job.stage = JobStage::ToStation;
                job.work_completed = 0;
            }
        }
        (JobStage::ToStation, LogisticsTargetState::AtWorkTile) => {
            job.stage = JobStage::ReadyToRefine;
            job.work_completed = 0;
        }
        (JobStage::ReadyToRefine, LogisticsTargetState::AtWorkTile) => {
            if cargo.resource_id.as_deref() != Some(recipe.input_resource_id.as_str())
                || cargo.amount < recipe.input_amount
            {
                job.blocked = Some(LogisticsBlock::MissingInput);
                result.blocked = job.blocked;
                return result;
            }
            job.work_completed = job.work_completed.saturating_add(1);
            if job.work_completed >= recipe.refine_work_turns {
                cargo.amount -= recipe.input_amount;
                if cargo.amount == 0 {
                    cargo.resource_id = None;
                }
                result.consumed_input = recipe.input_amount;
                result.finished_output = recipe.output_amount;
                job.stage = JobStage::ToSource;
                job.work_completed = 0;
            }
        }
        (JobStage::ReadyToGather | JobStage::ReadyToRefine, LogisticsTargetState::RouteStep(_)) => {
            job.blocked = Some(LogisticsBlock::NoRoute);
            result.blocked = job.blocked;
        }
        (_, LogisticsTargetState::Missing | LogisticsTargetState::NoRoute) => {
            unreachable!("missing and unreachable targets return before transition")
        }
    }
    result
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub(crate) fn process_logistics_workers(
    mut commands: Commands,
    mut workers: Query<
        (
            Entity,
            Option<&crate::components::Name>,
            &mut Position,
            &mut LogisticsJob,
            &mut Cargo,
        ),
        (With<crate::colony::survivors::Survivor>, With<LogisticsJob>),
    >,
    all_survivors: Query<
        &Position,
        (
            With<crate::colony::survivors::Survivor>,
            Without<LogisticsJob>,
        ),
    >,
    nodes: Query<
        (
            &Position,
            &crate::components::ResourceNode,
            Option<&crate::components::Name>,
        ),
        Without<crate::colony::survivors::Survivor>,
    >,
    stations: Query<
        (
            &Position,
            Option<&crate::components::ContentIdentity>,
            Option<&crate::components::Name>,
        ),
        (
            With<crate::colony::stations::Station>,
            Without<crate::colony::survivors::Survivor>,
            Without<crate::colony::stations::ConstructionSite>,
        ),
    >,
    player: Query<
        &Position,
        (
            With<crate::components::Player>,
            Without<crate::colony::survivors::Survivor>,
        ),
    >,
    map: Res<crate::map::SmokeMap>,
    mode: Res<crate::spatial::GameMode>,
    time_plan: Res<crate::time::TimeAdvancePlan>,
    content: Option<Res<crate::content::FoundationContent>>,
    mut resources: ResMut<crate::colony::production::ColonyResources>,
    mut game_log: ResMut<crate::gamelog::GameLog>,
) {
    if *mode != crate::spatial::GameMode::Outpost {
        return;
    }
    let Some(content) = content else {
        return;
    };

    let node_targets = nodes
        .iter()
        .map(|(position, node, name)| {
            (
                node.source_id.clone(),
                crate::colony::survivors::WorkTarget {
                    label: name.map_or_else(|| node.source_id.clone(), |name| name.0.clone()),
                    position: *position,
                },
            )
        })
        .collect::<Vec<_>>();
    let station_targets = stations
        .iter()
        .filter_map(|(position, identity, name)| {
            let identity = identity?;
            Some((
                identity.0.clone(),
                crate::colony::survivors::WorkTarget {
                    label: name.map_or_else(|| identity.0.clone(), |name| name.0.clone()),
                    position: *position,
                },
            ))
        })
        .collect::<Vec<_>>();
    let permanent = node_targets
        .iter()
        .map(|(_, target)| target.position)
        .chain(station_targets.iter().map(|(_, target)| target.position))
        .chain(player.iter().copied())
        .collect::<HashSet<_>>();
    let mut occupied = all_survivors.iter().copied().collect::<HashSet<_>>();
    occupied.extend(workers.iter().map(|(_, _, position, _, _)| *position));

    let mut order = workers
        .iter()
        .map(|(entity, name, _, _, _)| {
            (
                name.map_or_else(
                    || format!("Survivor {}", entity.to_bits()),
                    |name| name.0.clone(),
                ),
                entity,
            )
        })
        .collect::<Vec<_>>();
    order.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));

    let worker_steps = time_plan.outpost_worker_steps;
    for iteration in 0..worker_steps.max(1) {
        let allow_tick = iteration < worker_steps;
        for (_, entity) in &order {
            let Ok((_, worker_name, mut position, mut job, mut cargo)) = workers.get_mut(*entity)
            else {
                continue;
            };
            let Some(recipe) = content
                .colony_recipes
                .iter()
                .find(|recipe| recipe.id == job.recipe_id)
            else {
                if allow_tick {
                    job.blocked = Some(LogisticsBlock::MissingSource);
                }
                commands.entity(*entity).insert(
                    crate::colony::survivors::WorkerActivity::Blocked {
                        target: job.recipe_id.clone(),
                        target_position: None,
                        reason: crate::colony::survivors::WorkerBlockedReason::MissingTarget,
                    },
                );
                continue;
            };
            let (targets, missing_label) = match job.stage {
                JobStage::ToSource | JobStage::ReadyToGather => (
                    node_targets
                        .iter()
                        .filter(|(id, _)| id == &recipe.source_id)
                        .map(|(_, target)| target.clone())
                        .collect::<Vec<_>>(),
                    LogisticsBlock::MissingSource,
                ),
                JobStage::ToStation | JobStage::ReadyToRefine => (
                    station_targets
                        .iter()
                        .filter(|(id, _)| id == &recipe.station_id)
                        .map(|(_, target)| target.clone())
                        .collect::<Vec<_>>(),
                    LogisticsBlock::MissingStation,
                ),
            };
            let mut blocked = permanent.clone();
            blocked.extend(occupied.iter().copied());
            blocked.remove(&*position);
            let (target_state, target) = if targets.is_empty() {
                (LogisticsTargetState::Missing, None)
            } else {
                match crate::colony::survivors::choose_worker_path(
                    &map, *position, &targets, &blocked,
                ) {
                    Ok((target, path)) if path.len() <= 1 => {
                        (LogisticsTargetState::AtWorkTile, Some(target))
                    }
                    Ok((target, path)) => (LogisticsTargetState::RouteStep(path[1]), Some(target)),
                    Err(_) => (LogisticsTargetState::NoRoute, targets.first().cloned()),
                }
            };
            let before = *position;
            let observed_block = match target_state {
                LogisticsTargetState::Missing => Some(missing_label),
                LogisticsTargetState::NoRoute => Some(LogisticsBlock::NoRoute),
                _ => None,
            };
            let result = if allow_tick {
                tick_logistics(&mut job, &mut cargo, &mut position, recipe, target_state)
            } else {
                LogisticsTick::default()
            };
            occupied.remove(&before);
            occupied.insert(*position);
            if result.finished_output > 0
                && let Some(output) = content
                    .colony_resources
                    .iter()
                    .find(|resource| resource.id == recipe.output_resource_id)
                && let Some(kind) = output.pool_kind
                && let Some(pool) = resources.pools.get_mut(kind)
            {
                pool.current = (pool.current
                    + i32::try_from(result.finished_output).unwrap_or(i32::MAX))
                .min(pool.max);
                game_log.push(
                    format!(
                        "{} completed {}: +{} {}.",
                        worker_name.map_or("Survivor", |name| name.0.as_str()),
                        recipe.label,
                        result.finished_output,
                        output.label
                    ),
                    crate::gamelog::LogLevel::Info,
                );
            }
            let activity = if let Some(reason) = result.blocked.or(observed_block) {
                crate::colony::survivors::WorkerActivity::Blocked {
                    target: target.as_ref().map_or_else(
                        || format!("{missing_label:?}"),
                        |target| target.label.clone(),
                    ),
                    target_position: target.as_ref().map(|target| target.position),
                    reason: match reason {
                        LogisticsBlock::MissingSource | LogisticsBlock::MissingStation => {
                            crate::colony::survivors::WorkerBlockedReason::MissingTarget
                        }
                        LogisticsBlock::NoRoute => {
                            crate::colony::survivors::WorkerBlockedReason::NoRoute
                        }
                        LogisticsBlock::CargoMismatch | LogisticsBlock::MissingInput => {
                            crate::colony::survivors::WorkerBlockedReason::TargetUnavailable
                        }
                    },
                }
            } else if let Some(target) = target {
                match job.stage {
                    JobStage::ReadyToGather | JobStage::ReadyToRefine => {
                        crate::colony::survivors::WorkerActivity::Working {
                            target: target.label,
                            target_position: target.position,
                        }
                    }
                    JobStage::ToSource | JobStage::ToStation => {
                        crate::colony::survivors::WorkerActivity::EnRoute {
                            target: target.label,
                            target_position: target.position,
                            distance: (position.x - target.position.x).abs()
                                + (position.y - target.position.y).abs(),
                        }
                    }
                }
            } else {
                crate::colony::survivors::WorkerActivity::Blocked {
                    target: format!("{missing_label:?}"),
                    target_position: None,
                    reason: crate::colony::survivors::WorkerBlockedReason::MissingTarget,
                }
            };
            commands.entity(*entity).insert(activity);
        }
    }
}
