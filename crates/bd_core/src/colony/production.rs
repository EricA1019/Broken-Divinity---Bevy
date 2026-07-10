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
    query: Query<(Entity, &SurvivorTask), With<Survivor>>,
    game_time: Res<GameTime>,
    mut delta_writer: bevy_ecs::message::MessageWriter<crate::signals::PoolDeltaRequested>,
) {
    if game_time.turn != 0 {
        return;
    }

    // Survivors consume food - directly modify colony resources
    let supplies = colony_res.pools.get_mut(PoolKind::Supplies);
    if let Some(s) = supplies {
        s.current = (s.current - FOOD_PER_SURVIVOR_PER_DAY * query.iter().count() as i32).max(0);
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
