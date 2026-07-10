//! Production & Resources — colony-wide resource pools and day-change production.

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    pools::{Pool, Pools},
    signals::PoolKind,
    time::GameTime,
    colony::survivors::{Survivor, SurvivorTask, FOOD_PER_SURVIVOR_PER_DAY},
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

impl Default for ColonyResources {
    fn default() -> Self {
        Self {
            pools: Pools::new(vec![
                Pool::new(PoolKind::Supplies, INITIAL_SUPPLIES, 0, 100),
                Pool::new(PoolKind::Faith, INITIAL_FAITH, 0, 100),
            ]),
        }
    }
}

/// Processes production at day change: stations produce, survivors eat.
pub fn process_production(
    mut colony_res: ResMut<ColonyResources>,
    survivors_query: Query<&SurvivorTask, With<Survivor>>,
    stations_query: Query<&crate::colony::stations::StationType, With<crate::colony::stations::Station>>,
    game_time: Res<GameTime>,
    mut last_day: bevy_ecs::system::Local<u64>,
) {
    if game_time.day == *last_day || game_time.day == 0 {
        return;
    }
    *last_day = game_time.day;

    // Survivors consume food
    let survivor_count = survivors_query.iter().count() as i32;
    if let Some(supplies) = colony_res.pools.get_mut(PoolKind::Supplies) {
        supplies.current = (supplies.current - survivor_count * crate::colony::survivors::FOOD_PER_SURVIVOR_PER_DAY).max(0);
    }

    // Stations produce resources
    let blueprints = crate::colony::stations::default_station_blueprints();
    for station_type in stations_query.iter() {
        if let Some(bp) = blueprints.iter().find(|b| b.station_type == *station_type) {
            if let Some((kind, amount)) = bp.produces {
                if let Some(pool) = colony_res.pools.get_mut(kind) {
                    pool.current = (pool.current + amount).min(pool.max);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colony_resources_start_with_defaults() {
        let res = ColonyResources::default();
        assert_eq!(res.pools.get(PoolKind::Supplies).unwrap().current, INITIAL_SUPPLIES);
    }
}
