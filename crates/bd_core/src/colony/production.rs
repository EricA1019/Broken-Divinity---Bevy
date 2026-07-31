//! Production & Resources — colony-wide resource pools and day-change production.

use std::collections::BTreeMap;

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    colony::survivors::{Survivor, SurvivorTask},
    components::{Position, ResourceNodeType},
    gamelog::{GameLog, LogLevel},
    pools::{Pool, Pools},
    signals::PoolKind,
};

// ── Constants ──

pub const INITIAL_SUPPLIES: i32 = 10;
pub const INITIAL_MATERIALS: i32 = 5;
pub const INITIAL_FAITH: i32 = 0;

/// Colony-wide resource pools (not entity-attached).
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct ColonyResources {
    pub pools: Pools,
    #[serde(default)]
    pub raw: BTreeMap<String, u32>,
}

/// Colony-owned item storage. Dungeon loot is transferred here by the
/// extraction transaction; the TUI never mutates this resource directly.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct ColonyStorage {
    pub items: BTreeMap<String, u32>,
}

#[derive(Message, Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DailySummary {
    pub day: u64,
    pub supplies_before: i32,
    pub supplies_after: i32,
    pub materials_before: i32,
    pub materials_after: i32,
    pub wild_plants_before: i32,
    pub wild_plants_after: i32,
    pub faith_before: i32,
    pub faith_after: i32,
    pub food_consumed: i32,
    pub staffed_stations: u32,
    pub station_supplies_produced: i32,
    pub gathered_supplies: i32,
    pub gathered_materials: i32,
    pub gathered_wild_plants: i32,
    pub gathering_units: u32,
    pub starved_survivors: u32,
}

impl DailySummary {
    pub fn display_lines(&self) -> Vec<String> {
        vec![
            format!("Day {}:", self.day),
            format!(
                "Supplies {}→{} ({:+});",
                self.supplies_before,
                self.supplies_after,
                self.supplies_after - self.supplies_before
            ),
            format!(
                "Materials {}→{} ({:+});",
                self.materials_before,
                self.materials_after,
                self.materials_after - self.materials_before
            ),
            format!(
                "Plants {}→{} ({:+});",
                self.wild_plants_before,
                self.wild_plants_after,
                self.wild_plants_after - self.wild_plants_before
            ),
            format!(
                "Faith {}→{} ({:+});",
                self.faith_before,
                self.faith_after,
                self.faith_after - self.faith_before
            ),
            format!("Food -{}.", self.food_consumed),
        ]
    }

    pub fn display_line(&self) -> String {
        self.display_lines().join(" ")
    }
}

#[derive(Resource, Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LatestDailySummary(pub Option<DailySummary>);

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ColonyForecast {
    pub food_consumed: i32,
    pub station_supplies: i32,
    pub gathered_supplies: i32,
    pub supplies_net: i32,
    pub supplies_after: i32,
    pub materials_net: i32,
    pub plants_net: i32,
    pub faith_net: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurvivorWorkSnapshot {
    pub task: SurvivorTask,
    pub position: Position,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StationWorkSnapshot {
    pub entity_bits: u64,
    pub station_type: crate::colony::stations::StationType,
    pub position: Position,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceWorkSnapshot {
    pub kind: ResourceNodeType,
    pub position: Position,
    pub depleted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalWorkContribution {
    Station(StationWorkSnapshot),
    Resource(ResourceWorkSnapshot),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalWorkDenial {
    NotAssigned,
    MissingTarget,
    NotAdjacent,
    TargetDepleted,
    WrongResource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalWorkEvaluation {
    Contributes(PhysicalWorkContribution),
    NoContribution(PhysicalWorkDenial),
}

/// Evaluate current physical work from durable assignment and world
/// snapshots. This is the sole station/gathering contribution rule used by
/// production, gathering, forecast, recovery effects, and activity checks.
pub fn evaluate_physical_work(
    worker: &SurvivorWorkSnapshot,
    stations: &[StationWorkSnapshot],
    nodes: &[ResourceWorkSnapshot],
) -> PhysicalWorkEvaluation {
    match worker.task {
        SurvivorTask::AssignedTo(station_bits) => {
            let Some(station) = stations
                .iter()
                .find(|station| station.entity_bits == station_bits)
                .copied()
            else {
                return PhysicalWorkEvaluation::NoContribution(PhysicalWorkDenial::MissingTarget);
            };
            if !crate::colony::survivors::cardinally_adjacent(worker.position, station.position) {
                return PhysicalWorkEvaluation::NoContribution(PhysicalWorkDenial::NotAdjacent);
            }
            PhysicalWorkEvaluation::Contributes(PhysicalWorkContribution::Station(station))
        }
        SurvivorTask::Gathering(kind) => {
            let adjacent = nodes
                .iter()
                .filter(|node| {
                    crate::colony::survivors::cardinally_adjacent(worker.position, node.position)
                })
                .collect::<Vec<_>>();
            if let Some(node) = adjacent
                .iter()
                .find(|node| {
                    !node.depleted && crate::colony::resources::pool_for_node(node.kind) == kind
                })
                .copied()
                .copied()
            {
                return PhysicalWorkEvaluation::Contributes(PhysicalWorkContribution::Resource(
                    node,
                ));
            }
            if adjacent.iter().any(|node| {
                node.depleted && crate::colony::resources::pool_for_node(node.kind) == kind
            }) {
                return PhysicalWorkEvaluation::NoContribution(PhysicalWorkDenial::TargetDepleted);
            }
            if adjacent.is_empty() {
                PhysicalWorkEvaluation::NoContribution(PhysicalWorkDenial::NotAdjacent)
            } else {
                PhysicalWorkEvaluation::NoContribution(PhysicalWorkDenial::WrongResource)
            }
        }
        SurvivorTask::Idle | SurvivorTask::Defending | SurvivorTask::Resting => {
            PhysicalWorkEvaluation::NoContribution(PhysicalWorkDenial::NotAssigned)
        }
    }
}

pub fn forecast_colony(
    resources: &ColonyResources,
    survivors: &[SurvivorWorkSnapshot],
    stations: &[StationWorkSnapshot],
    nodes: &[ResourceWorkSnapshot],
    station_catalog: &crate::colony::stations::StationCatalog,
) -> ColonyForecast {
    let supplies_before = resource_value(resources, PoolKind::Supplies);
    let food_required =
        survivors.len() as i32 * crate::colony::survivors::FOOD_PER_SURVIVOR_PER_DAY;
    let food_consumed = food_required.min(supplies_before);
    let mut forecast = ColonyForecast {
        food_consumed,
        ..Default::default()
    };

    let mut credited_stations = std::collections::HashSet::new();
    for worker in survivors {
        match evaluate_physical_work(worker, stations, nodes) {
            PhysicalWorkEvaluation::Contributes(PhysicalWorkContribution::Resource(_)) => {}
            PhysicalWorkEvaluation::Contributes(PhysicalWorkContribution::Station(station)) => {
                if !credited_stations.insert(station.entity_bits) {
                    continue;
                }
                let Some(blueprint) = station_catalog.get(station.station_type) else {
                    continue;
                };
                if let crate::colony::stations::StationEffect::Produce { kind, amount } =
                    blueprint.effect
                {
                    match kind {
                        PoolKind::Supplies => forecast.station_supplies += amount,
                        PoolKind::Materials => forecast.materials_net += amount,
                        PoolKind::WildPlants => forecast.plants_net += amount,
                        PoolKind::Faith => forecast.faith_net += amount,
                        _ => {}
                    }
                }
            }
            PhysicalWorkEvaluation::NoContribution(_) => {}
        }
    }
    forecast.supplies_net =
        forecast.station_supplies + forecast.gathered_supplies - forecast.food_consumed;
    forecast.supplies_after = (supplies_before + forecast.supplies_net).clamp(0, 100);
    forecast
}

#[derive(Resource, Debug, Default)]
pub(crate) struct DailyCycleDraft(pub Option<DailySummary>);

fn resource_value(resources: &ColonyResources, kind: PoolKind) -> i32 {
    resources.pools.get(kind).map_or(0, |pool| pool.current)
}

impl ColonyStorage {
    pub fn add_item(&mut self, item_id: impl Into<String>) {
        *self.items.entry(item_id.into()).or_default() += 1;
    }

    pub fn count(&self, item_id: &str) -> u32 {
        self.items.get(item_id).copied().unwrap_or(0)
    }
}

impl Default for ColonyResources {
    fn default() -> Self {
        Self {
            pools: Pools::new(vec![
                Pool::new(PoolKind::Supplies, INITIAL_SUPPLIES, 0, 100),
                Pool::new(PoolKind::Faith, INITIAL_FAITH, 0, 100),
                Pool::new(PoolKind::Materials, 0, 0, 50),
                Pool::new(PoolKind::WildPlants, 0, 0, 50),
            ]),
            raw: BTreeMap::new(),
        }
    }
}

#[cfg(test)]
mod physical_work_contract_tests {
    use super::*;
    use crate::colony::stations::StationType;

    #[test]
    fn one_physical_work_evaluator_classifies_station_and_resource_contributions() {
        let station = StationWorkSnapshot {
            entity_bits: 7,
            station_type: StationType::Stove,
            position: Position { x: 5, y: 5 },
        };
        let node = ResourceWorkSnapshot {
            kind: ResourceNodeType::Trees,
            position: Position { x: 8, y: 8 },
            depleted: false,
        };

        let station_worker = SurvivorWorkSnapshot {
            task: SurvivorTask::AssignedTo(7),
            position: Position { x: 4, y: 5 },
        };
        assert_eq!(
            evaluate_physical_work(&station_worker, &[station], &[node]),
            PhysicalWorkEvaluation::Contributes(PhysicalWorkContribution::Station(station))
        );

        let gatherer = SurvivorWorkSnapshot {
            task: SurvivorTask::Gathering(PoolKind::Materials),
            position: Position { x: 8, y: 7 },
        };
        assert_eq!(
            evaluate_physical_work(&gatherer, &[station], &[node]),
            PhysicalWorkEvaluation::Contributes(PhysicalWorkContribution::Resource(node))
        );
    }
}

/// Processes production at day change: stations produce (only if staffed), survivors eat.
#[allow(clippy::type_complexity)]
pub(crate) fn process_production(
    mut colony_res: ResMut<ColonyResources>,
    survivors_query: Query<(&Position, &SurvivorTask), With<Survivor>>,
    stations_query: Query<
        (Entity, &Position, &crate::colony::stations::StationType),
        (
            With<crate::colony::stations::Station>,
            Without<crate::colony::stations::ConstructionSite>,
        ),
    >,
    mut days: bevy_ecs::message::MessageReader<crate::time::DayAdvanced>,
    mut draft: ResMut<DailyCycleDraft>,
    station_catalog: Res<crate::colony::stations::StationCatalog>,
) {
    let Some(day) = days.read().last().map(|event| event.day) else {
        return;
    };
    let mut summary = DailySummary {
        day,
        supplies_before: resource_value(&colony_res, PoolKind::Supplies),
        materials_before: resource_value(&colony_res, PoolKind::Materials),
        wild_plants_before: resource_value(&colony_res, PoolKind::WildPlants),
        faith_before: resource_value(&colony_res, PoolKind::Faith),
        ..Default::default()
    };

    // Survivors consume food
    let survivors = survivors_query
        .iter()
        .map(|(position, task)| SurvivorWorkSnapshot {
            task: task.clone(),
            position: *position,
        })
        .collect::<Vec<_>>();
    let stations = stations_query
        .iter()
        .map(|(entity, position, station_type)| StationWorkSnapshot {
            entity_bits: entity.to_bits(),
            station_type: *station_type,
            position: *position,
        })
        .collect::<Vec<_>>();
    let survivor_count = survivors.len() as i32;
    if let Some(supplies) = colony_res.pools.get_mut(PoolKind::Supplies) {
        let required = survivor_count * crate::colony::survivors::FOOD_PER_SURVIVOR_PER_DAY;
        summary.food_consumed = required.min(supplies.current);
        if supplies.current < required {
            summary.starved_survivors = survivor_count as u32;
        }
        supplies.current -= summary.food_consumed;
    }

    let staffed = survivors
        .iter()
        .filter_map(
            |worker| match evaluate_physical_work(worker, &stations, &[]) {
                PhysicalWorkEvaluation::Contributes(PhysicalWorkContribution::Station(station)) => {
                    Some(station.entity_bits)
                }
                _ => None,
            },
        )
        .collect::<std::collections::HashSet<_>>();

    for station in &stations {
        if !staffed.contains(&station.entity_bits) {
            continue;
        }
        summary.staffed_stations += 1;
        if let Some(bp) = station_catalog.get(station.station_type) {
            if let crate::colony::stations::StationEffect::Produce { kind, amount } = bp.effect {
                if let Some(pool) = colony_res.pools.get_mut(kind) {
                    let before = pool.current;
                    pool.current = (pool.current + amount).min(pool.max);
                    if kind == PoolKind::Supplies {
                        summary.station_supplies_produced += pool.current - before;
                    }
                }
            }
        }
    }
    draft.0 = Some(summary);
}

pub(crate) fn finalize_daily_cycle(
    resources: Res<ColonyResources>,
    mut draft: ResMut<DailyCycleDraft>,
    mut latest: ResMut<LatestDailySummary>,
    mut summaries: bevy_ecs::message::MessageWriter<DailySummary>,
    mut game_log: ResMut<GameLog>,
) {
    let Some(mut summary) = draft.0.take() else {
        return;
    };
    summary.supplies_after = resource_value(&resources, PoolKind::Supplies);
    summary.materials_after = resource_value(&resources, PoolKind::Materials);
    summary.wild_plants_after = resource_value(&resources, PoolKind::WildPlants);
    summary.faith_after = resource_value(&resources, PoolKind::Faith);
    game_log.push(summary.display_line(), LogLevel::Info);
    latest.0 = Some(summary.clone());
    summaries.write(summary);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colony_resources_start_with_defaults() {
        let res = ColonyResources::default();
        assert_eq!(
            res.pools.get(PoolKind::Supplies).unwrap().current,
            INITIAL_SUPPLIES
        );
    }

    #[test]
    fn colony_storage_counts_items_deterministically() {
        let mut storage = ColonyStorage::default();
        storage.add_item("item.healing_potion");
        storage.add_item("item.healing_potion");
        assert_eq!(storage.count("item.healing_potion"), 2);
        assert_eq!(storage.count("item.unknown"), 0);
    }
}
