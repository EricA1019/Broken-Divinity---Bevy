//! Raids — enemy attacks on the shelter.

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{BdSet, signals::PoolKind};

pub const RAID_CHANCE_PER_DAY: f32 = 0.15;
pub const RAID_MIN_DAY: u64 = 3;
pub const RAID_ENEMY_COUNT_MIN: u32 = 1;
pub const RAID_ENEMY_COUNT_MAX: u32 = 3;
pub const RAID_SUPPLIES_LOST_IF_UNDEFENDED: i32 = 5;

#[derive(Resource, Debug, Default, Clone, Serialize, Deserialize)]
pub enum RaidState {
    #[default]
    Inactive,
    Active {
        turn_started: u64,
        enemy_count: u32,
    },
}

/// Marker component for raid-spawned enemies.
#[derive(Component, Debug, Clone)]
pub struct RaidEnemy;

/// Roll for raid at day change. Spawn enemies if raid triggers.
pub fn process_raids(
    mut raid_state: ResMut<RaidState>,
    mut colony_res: ResMut<crate::colony::production::ColonyResources>,
    game_time: Res<crate::time::GameTime>,
    mut last_day: Local<u64>,
    mut commands: Commands,
    mut game_log: ResMut<crate::gamelog::GameLog>,
    raid_enemies: Query<Entity, With<RaidEnemy>>,
) {
    // Only run on day change
    if game_time.day == *last_day || game_time.day == 0 {
        return;
    }
    *last_day = game_time.day;

    // Check if active raid needs resolution: if all enemies dead, end raid
    if matches!(*raid_state, RaidState::Active { .. }) {
        if raid_enemies.iter().next().is_none() {
            *raid_state = RaidState::Inactive;
            game_log.push(
                "The raid has been repelled!".to_string(),
                crate::gamelog::LogLevel::Info,
            );
        }
        return;
    }

    // Don't raid before minimum day
    if game_time.day < RAID_MIN_DAY {
        return;
    }

    // Roll for raid: use day as seed for deterministic roll
    let seed = game_time.day.wrapping_mul(2654435761);
    let roll = (seed % 100) as f32 / 100.0;
    if roll >= RAID_CHANCE_PER_DAY {
        return;
    }

    // Raid triggers! Determine enemy count
    let enemy_count = (seed / 100 % (RAID_ENEMY_COUNT_MAX - RAID_ENEMY_COUNT_MIN + 1) as u64
        + RAID_ENEMY_COUNT_MIN as u64) as u32;

    *raid_state = RaidState::Active {
        turn_started: game_time.turn,
        enemy_count,
    };

    // Spawn raiders at random positions on the shelter map edges
    let positions: [(i32, i32); 3] = [(3, 2), (16, 2), (10, 10)];
    for i in 0..enemy_count.min(3) {
        let (x, y) = positions[i as usize];
        commands.spawn((
            crate::components::Position { x, y },
            crate::components::Name(format!("Raider {}", i + 1)),
            crate::components::BlocksMovement,
            RaidEnemy,
            crate::pools::Pools::new(vec![
                crate::pools::Pool::new(PoolKind::Health, 8, 0, 8),
                crate::pools::Pool::new(PoolKind::ActionPoints, 2, 0, 2),
            ]),
        ));
    }

    game_log.push(
        format!(
            "Raiders attack the shelter! {} enemies sighted.",
            enemy_count
        ),
        crate::gamelog::LogLevel::Combat,
    );

    // If player has no defenders, immediate supply loss
    if colony_res
        .pools
        .get(PoolKind::Supplies)
        .map_or(0, |p| p.current)
        > 0
    {
        if let Some(supplies) = colony_res.pools.get_mut(PoolKind::Supplies) {
            let loss = RAID_SUPPLIES_LOST_IF_UNDEFENDED.min(supplies.current);
            supplies.current -= loss;
            game_log.push(
                format!("Raiders steal {} supplies before you can react!", loss),
                crate::gamelog::LogLevel::Warn,
            );
        }
    }
}

pub fn register_raids(app: &mut bevy_app::App) {
    app.init_resource::<RaidState>();
    app.add_systems(bevy_app::Update, process_raids.in_set(BdSet::Mutation));
}
