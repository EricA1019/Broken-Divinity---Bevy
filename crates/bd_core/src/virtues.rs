//! Virtue foundation — 6 classical virtues + Kleos (glory) tracker.

use bevy_ecs::prelude::*;

use crate::{
    BdSet,
    components::Player,
    signals::{EntityDefeated, PoolDeltaRequested, PoolKind},
};

pub const VIRTUE_MAX: i32 = 100;
pub const FORTITUDE_COMBAT_SURVIVAL_GAIN: i32 = 5;
pub const FORTITUDE_BOSS_SURVIVAL_GAIN: i32 = 15;
pub const PRUDENCE_WISE_CHOICE_GAIN: i32 = 5;
pub const TEMPERANCE_CORRUPTION_RESIST_GAIN: i32 = 5;
pub const THUMOS_DECISIVE_ACTION_GAIN: i32 = 5;
pub const METIS_CUNNING_SOLUTION_GAIN: i32 = 5;
pub const JUSTICE_LAWFUL_RULING_GAIN: i32 = 5;
pub const KLEOS_BOSS_KILL_GAIN: i32 = 10;
pub const KLEOS_GABRIEL_ACCEPT_GAIN: i32 = 15;

/// All virtue PoolKind variants.
pub const ALL_VIRTUES: &[PoolKind] = &[
    PoolKind::Temperance,
    PoolKind::Justice,
    PoolKind::Prudence,
    PoolKind::Fortitude,
    PoolKind::Thumos,
    PoolKind::Metis,
    PoolKind::Kleos,
];

/// Observe EntityDefeated messages and award virtue gains to the player.
pub fn process_virtue_gains(
    mut defeated: bevy_ecs::message::MessageReader<EntityDefeated>,
    player: Query<Entity, With<Player>>,
    mut pool_deltas: bevy_ecs::message::MessageWriter<PoolDeltaRequested>,
) {
    let Ok(player_entity) = player.single() else {
        return;
    };
    // Award Fortitude for any enemy defeated
    for _ in defeated.read() {
        pool_deltas.write(PoolDeltaRequested {
            source: None,
            target: player_entity,
            kind: PoolKind::Fortitude,
            amount: FORTITUDE_COMBAT_SURVIVAL_GAIN,
            tags: vec![],
            reason: "combat survival".into(),
        });
    }
}

/// Award Kleos for notable achievements: boss kills, major story beats.
pub fn process_kleos_gains(
    mut defeated: bevy_ecs::message::MessageReader<crate::signals::EntityDefeated>,
    player: Query<Entity, With<Player>>,
    mut delta_writer: bevy_ecs::message::MessageWriter<crate::signals::PoolDeltaRequested>,
) {
    let Ok(player_entity) = player.single() else {
        return;
    };
    // Award Kleos for boss kills (any EntityDefeated)
    for _ in defeated.read() {
        delta_writer.write(crate::signals::PoolDeltaRequested {
            source: None,
            target: player_entity,
            kind: PoolKind::Kleos,
            amount: KLEOS_BOSS_KILL_GAIN,
            tags: vec![],
            reason: "boss kill".into(),
        });
    }
}

/// Register virtue gain systems.
pub fn register_virtues(app: &mut bevy_app::App) {
    app.add_systems(
        bevy_app::Update,
        (process_virtue_gains, process_kleos_gains).in_set(BdSet::ResultEmission),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(crate::BdCorePlugin);
        app
    }

    #[test]
    fn all_virtues_are_defined() {
        assert_eq!(ALL_VIRTUES.len(), 7);
    }

    #[test]
    fn kleos_increases_on_boss_kill() {
        let mut app = test_app();
        let player = app
            .world_mut()
            .spawn((
                Player,
                crate::pools::Pools::new(vec![crate::pools::Pool::new(PoolKind::Kleos, 0, 0, 100)]),
            ))
            .id();

        let enemy = app
            .world_mut()
            .spawn((crate::components::Name("Boss".into()),))
            .id();

        app.world_mut()
            .resource_mut::<bevy_ecs::message::Messages<EntityDefeated>>()
            .write(EntityDefeated {
                entity: enemy,
                kind: PoolKind::Health,
            });

        app.update();
        app.update();

        let pools = app.world().get::<crate::pools::Pools>(player).unwrap();
        let kleos = pools.get(PoolKind::Kleos).unwrap();
        assert!(
            kleos.current > 0,
            "Kleos should increase on boss kill (current={})",
            kleos.current
        );
    }

    #[test]
    fn fortitude_increases_on_combat_survival() {
        let mut app = test_app();
        let player = app
            .world_mut()
            .spawn((
                Player,
                crate::pools::Pools::new(vec![crate::pools::Pool::new(
                    PoolKind::Fortitude,
                    0,
                    0,
                    100,
                )]),
            ))
            .id();

        // Spawn a separate enemy entity to be defeated
        let enemy = app
            .world_mut()
            .spawn((crate::components::Name("Enemy".into()),))
            .id();

        // Fire an EntityDefeated message for the enemy (not the player)
        app.world_mut()
            .resource_mut::<bevy_ecs::message::Messages<EntityDefeated>>()
            .write(EntityDefeated {
                entity: enemy,
                kind: PoolKind::Health,
            });

        app.update();
        app.update(); // second frame to process pool deltas

        let pools = app.world().get::<crate::pools::Pools>(player).unwrap();
        let fortitude = pools.get(PoolKind::Fortitude).unwrap();
        assert!(
            fortitude.current > 0,
            "Fortitude should increase on combat survival (current={})",
            fortitude.current
        );
    }
}
