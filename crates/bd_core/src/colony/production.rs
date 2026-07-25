//! Production & Resources — colony-wide resource pools and day-change production.

use std::collections::BTreeMap;

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    colony::survivors::{Survivor, SurvivorTask},
    gamelog::{GameLog, LogLevel},
    pools::{Pool, Pools},
    signals::PoolKind,
    time::GameTime,
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
pub fn process_production(
    mut colony_res: ResMut<ColonyResources>,
    survivors_query: Query<&SurvivorTask, With<Survivor>>,
    stations_query: Query<
        (Entity, &crate::colony::stations::StationType),
        With<crate::colony::stations::Station>,
    >,
    mode: Res<crate::spatial::GameMode>,
    game_time: Res<GameTime>,
    mut game_log: ResMut<GameLog>,
    mut last_day: bevy_ecs::system::Local<u64>,
) {
    if *mode != crate::spatial::GameMode::Outpost {
        return;
    }
    if game_time.day == *last_day || game_time.day == 0 {
        return;
    }
    *last_day = game_time.day;

    // Survivors consume food
    let survivor_count = survivors_query.iter().count() as i32;
    if let Some(supplies) = colony_res.pools.get_mut(PoolKind::Supplies) {
        supplies.current = (supplies.current
            - survivor_count * crate::colony::survivors::FOOD_PER_SURVIVOR_PER_DAY)
            .max(0);
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
        if let Some(bp) = blueprints.iter().find(|b| b.station_type == *station_type) {
            if let Some((kind, amount)) = bp.produces {
                if let Some(pool) = colony_res.pools.get_mut(kind) {
                    pool.current = (pool.current + amount).min(pool.max);
                }
            }
        }
    }

    // Daily summary log
    let supplies = colony_res
        .pools
        .get(PoolKind::Supplies)
        .map_or(0, |p| p.current);
    let materials = colony_res
        .pools
        .get(PoolKind::Materials)
        .map_or(0, |p| p.current);
    let plants = colony_res
        .pools
        .get(PoolKind::WildPlants)
        .map_or(0, |p| p.current);
    let faith = colony_res
        .pools
        .get(PoolKind::Faith)
        .map_or(0, |p| p.current);
    let food_consumed = survivor_count * crate::colony::survivors::FOOD_PER_SURVIVOR_PER_DAY;
    game_log.push(
        format!(
            "--- Day {} --- Supplies:{} Materials:{} Plants:{} Faith:{} | Food: -{}",
            game_time.day, supplies, materials, plants, faith, food_consumed
        ),
        LogLevel::Info,
    );
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
