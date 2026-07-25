//! Production & Resources — colony-wide resource pools and day-change production.

use std::collections::BTreeMap;

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    colony::survivors::{Survivor, SurvivorTask},
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

#[derive(Resource, Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LatestDailySummary(pub Option<DailySummary>);

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
        }
    }
}

/// Processes production at day change: stations produce (only if staffed), survivors eat.
pub(crate) fn process_production(
    mut colony_res: ResMut<ColonyResources>,
    survivors_query: Query<&SurvivorTask, With<Survivor>>,
    stations_query: Query<
        (Entity, &crate::colony::stations::StationType),
        With<crate::colony::stations::Station>,
    >,
    mode: Res<crate::spatial::GameMode>,
    mut days: bevy_ecs::message::MessageReader<crate::time::DayAdvanced>,
    mut draft: ResMut<DailyCycleDraft>,
) {
    if *mode != crate::spatial::GameMode::Outpost {
        return;
    }
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
    let survivor_count = survivors_query.iter().count() as i32;
    if let Some(supplies) = colony_res.pools.get_mut(PoolKind::Supplies) {
        let required = survivor_count * crate::colony::survivors::FOOD_PER_SURVIVOR_PER_DAY;
        summary.food_consumed = required.min(supplies.current);
        if supplies.current < required {
            summary.starved_survivors = survivor_count as u32;
        }
        supplies.current -= summary.food_consumed;
    }

    // Collect which station entities have at least one assigned worker
    use std::collections::HashSet;
    let staffed: HashSet<Entity> = survivors_query
        .iter()
        .filter_map(|t| match t {
            SurvivorTask::AssignedTo(idx) => Some(Entity::from_bits(*idx)),
            _ => None,
        })
        .collect();

    // Stations produce resources — only if at least one survivor is assigned
    let blueprints = crate::colony::stations::default_station_blueprints();
    for (station_entity, station_type) in stations_query.iter() {
        if !staffed.contains(&station_entity) {
            continue; // no worker assigned — skip production
        }
        summary.staffed_stations += 1;
        if let Some(bp) = blueprints.iter().find(|b| b.station_type == *station_type) {
            if let Some((kind, amount)) = bp.produces {
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
    game_log.push(
        format!(
            "--- Day {} --- Supplies:{} Materials:{} Plants:{} Faith:{} | Food: -{}",
            summary.day,
            summary.supplies_after,
            summary.materials_after,
            summary.wild_plants_after,
            summary.faith_after,
            summary.food_consumed
        ),
        LogLevel::Info,
    );
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
