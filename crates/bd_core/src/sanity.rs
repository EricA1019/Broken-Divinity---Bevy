//! Sanity system — mental health, hallucination thresholds, and breakdown.

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    components::{Player, Position},
    gamelog::{GameLog, LogLevel},
    pools::Pools,
    signals::{PoolDeltaRequested, PoolKind},
    spatial::GameMode,
    statuses::Statuses,
    time::GameTime,
};

pub const SANITY_MAX: i32 = 100;
pub const SANITY_HALLUCINATION_THRESHOLD: i32 = 50;
pub const SANITY_BREAKDOWN_THRESHOLD: i32 = 25;
pub const SANITY_RECOVERY_AT_SHELTER: i32 = 20;

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct SanityPressure {
    pub radius: u32,
    pub drain_per_turn: i32,
}

/// Drain sanity each turn when player is near a SanityPressure source.
pub fn process_sanity_drain(
    player: Query<(Entity, &Position), With<Player>>,
    pressure_sources: Query<(&Position, &SanityPressure)>,
    mut delta_writer: bevy_ecs::message::MessageWriter<PoolDeltaRequested>,
) {
    let Ok((entity, player_pos)) = player.single() else {
        return;
    };
    for (source_pos, pressure) in pressure_sources.iter() {
        let dx = (player_pos.x - source_pos.x).unsigned_abs();
        let dy = (player_pos.y - source_pos.y).unsigned_abs();
        let dist = dx.max(dy);
        if dist <= pressure.radius {
            delta_writer.write(PoolDeltaRequested {
                source: None,
                target: entity,
                kind: PoolKind::Sanity,
                amount: -(pressure.drain_per_turn as i32),
                tags: vec![],
                reason: "sanity drain".into(),
            });
        }
    }
}

/// Check sanity thresholds and apply effects.
pub fn process_sanity_thresholds(
    player: Query<(Entity, &Pools), With<Player>>,
    status_query: Query<&Statuses>,
    mut game_log: ResMut<GameLog>,
    mut commands: Commands,
) {
    let Ok((entity, pools)) = player.single() else {
        return;
    };
    let Some(sanity) = pools.get(PoolKind::Sanity) else {
        return;
    };
    if sanity.current < SANITY_BREAKDOWN_THRESHOLD {
        let mut needs_breakdown = true;
        if let Ok(statuses) = status_query.get(entity) {
            needs_breakdown = !statuses
                .instances
                .iter()
                .any(|s| s.status_id == "status.breakdown");
        }
        if needs_breakdown {
            let defs = crate::statuses::default_status_definitions();
            crate::statuses::apply_status(
                entity,
                "status.breakdown",
                3,
                None,
                &mut commands,
                &defs,
            );
            game_log.push(
                "Your mind fractures — Breakdown sets in.".to_string(),
                LogLevel::Warn,
            );
        }
    } else if sanity.current < SANITY_HALLUCINATION_THRESHOLD {
        game_log.push(
            "Shadows flicker at the edge of your vision...".to_string(),
            LogLevel::Warn,
        );
    }
}

/// Recover sanity at shelter when day changes.
pub fn process_sanity_recovery(
    player: Query<(Entity, &Pools), With<Player>>,
    mode: Option<Res<GameMode>>,
    time: Res<GameTime>,
    mut delta_writer: bevy_ecs::message::MessageWriter<PoolDeltaRequested>,
    mut last_day: Local<u64>,
) {
    let Some(mode) = mode else {
        return;
    };
    if *mode != GameMode::Outpost {
        *last_day = time.day;
        return;
    }
    if time.day == *last_day {
        return;
    }
    *last_day = time.day;
    let Ok((entity, pools)) = player.single() else {
        return;
    };
    let Some(sanity) = pools.get(PoolKind::Sanity) else {
        return;
    };
    if sanity.current < SANITY_MAX {
        delta_writer.write(PoolDeltaRequested {
            source: None,
            target: entity,
            kind: PoolKind::Sanity,
            amount: SANITY_RECOVERY_AT_SHELTER,
            tags: vec![],
            reason: "shelter recovery".into(),
        });
    }
}

pub fn register_sanity(app: &mut bevy_app::App) {
    app.add_systems(
        bevy_app::Update,
        (
            process_sanity_drain.in_set(crate::BdSet::Mutation),
            process_sanity_thresholds.in_set(crate::BdSet::ResultEmission),
            process_sanity_recovery.in_set(crate::BdSet::ResultEmission),
        ),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        components::Tile,
        gamelog::GameLog,
        map::SmokeMap,
        pools::{Pool, Pools},
        signals::PoolDeltaRequested,
        spatial::GameMode,
        time::GameTime,
    };
    use bevy_app::App;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(crate::BdCorePlugin);
        app
    }

    fn send_sanity_delta(app: &mut App, target: Entity, amount: i32) {
        app.world_mut()
            .resource_mut::<bevy_ecs::message::Messages<PoolDeltaRequested>>()
            .write(PoolDeltaRequested {
                source: None,
                target,
                kind: PoolKind::Sanity,
                amount,
                tags: vec![],
                reason: "test".into(),
            });
    }

    #[test]
    fn sanity_constants_are_sane() {
        assert!(SANITY_BREAKDOWN_THRESHOLD < SANITY_HALLUCINATION_THRESHOLD);
        assert!(SANITY_HALLUCINATION_THRESHOLD < SANITY_MAX);
    }

    #[test]
    fn sanity_drains_near_pressure_source() {
        let mut app = test_app();
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        let player = app
            .world_mut()
            .spawn((
                Player,
                Position { x: 5, y: 5 },
                Pools::new(vec![Pool::new(PoolKind::Sanity, SANITY_MAX, 0, SANITY_MAX)]),
            ))
            .id();
        // Write drain message directly to verify pool pipeline works for Sanity
        send_sanity_delta(&mut app, player, -5);
        app.update();
        let pools = app.world().get::<Pools>(player).unwrap();
        let sanity = pools.get(PoolKind::Sanity).unwrap();
        assert!(
            sanity.current < SANITY_MAX,
            "Direct sanity delta should work (current={})",
            sanity.current
        );

        // Now test the pressure source system integration
        let current_before = app
            .world()
            .get::<Pools>(player)
            .unwrap()
            .get(PoolKind::Sanity)
            .unwrap()
            .current;
        app.world_mut().spawn((
            Position { x: 5, y: 6 },
            SanityPressure {
                radius: 2,
                drain_per_turn: 5,
            },
        ));
        // process_sanity_drain runs in Mutation set, writes message
        // resolve_pool_deltas runs in same set but before drain (insertion order)
        // So drain message is processed NEXT frame
        app.update();
        app.update();
        let sanity_after = app
            .world()
            .get::<Pools>(player)
            .unwrap()
            .get(PoolKind::Sanity)
            .unwrap()
            .current;
        assert!(
            sanity_after < current_before,
            "Sanity should decrease from pressure source (was={}, now={})",
            current_before,
            sanity_after
        );
    }

    #[test]
    fn sanity_below_threshold_triggers_hallucination() {
        let mut app = test_app();
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        let player = app
            .world_mut()
            .spawn((
                Player,
                Position { x: 5, y: 5 },
                Pools::new(vec![Pool::new(
                    PoolKind::Sanity,
                    SANITY_HALLUCINATION_THRESHOLD - 1,
                    0,
                    SANITY_MAX,
                )]),
            ))
            .id();
        // Directly set sanity below threshold and let process_sanity_thresholds fire
        send_sanity_delta(&mut app, player, 0); // triggers resolve_pool_deltas which updates pools
        app.update();
        // After update, sanity is below threshold and process_sanity_thresholds ran
        let log = app.world().resource::<GameLog>();
        let has_hallucination = log.iter().any(|e| e.message.contains("Shadows"));
        assert!(
            has_hallucination,
            "Hallucination log should appear below threshold"
        );
    }

    #[test]
    fn sanity_recovers_at_shelter() {
        let mut app = test_app();
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        app.world_mut().insert_resource(GameMode::Outpost);
        let player = app
            .world_mut()
            .spawn((
                Player,
                Position { x: 5, y: 5 },
                Pools::new(vec![Pool::new(PoolKind::Sanity, 50, 0, SANITY_MAX)]),
            ))
            .id();
        // Set day to 1 (day change triggers recovery on next frame)
        app.world_mut().resource_mut::<GameTime>().day = 1;
        app.update();
        // process_sanity_recovery wrote a PoolDeltaRequested for +20
        // resolve_pool_deltas processes it next frame
        app.update();
        let pools = app.world().get::<Pools>(player).unwrap();
        let sanity = pools.get(PoolKind::Sanity).unwrap();
        assert!(
            sanity.current > 50,
            "Sanity should recover at shelter on day change (current={})",
            sanity.current
        );
    }
}
