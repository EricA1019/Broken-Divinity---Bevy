//! Raids — enemy attacks on the shelter.

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

pub const RAID_CHANCE_PER_DAY: f32 = 0.15;
pub const RAID_MIN_DAY: u64 = 3;
pub const RAID_ENEMY_COUNT_MIN: u32 = 1;
pub const RAID_ENEMY_COUNT_MAX: u32 = 3;

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub enum RaidState {
    Inactive,
    Active { turn_started: u64, enemy_count: u32 },
}

impl Default for RaidState {
    fn default() -> Self {
        Self::Inactive
    }
}
