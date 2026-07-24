//! bd_test_support — Shared test utilities for the BD Kernel.
//!
//! Provides deterministic RNG, minimal app builders, and snapshot helpers.

use bd_core::content::FoundationContent;
use bevy_app::App;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// Create a deterministic RNG from a fixed seed for reproducible tests.
pub fn seeded_rng(seed: u64) -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(seed)
}

/// Build a minimal Bevy app with just the core plugin for unit testing.
pub fn minimal_app() -> App {
    foundation_app()
}

/// Build the foundation-only app used by MVP tests.
///
/// This app intentionally excludes terminal rendering and all deferred game
/// systems so simulation tests remain deterministic and headless-safe.
pub fn foundation_app() -> App {
    let mut app = App::new();
    app.add_plugins(bd_core::BdFoundationPlugin);
    app.insert_resource(foundation_content());
    app
}

/// Load the same foundation bundle used by the application.
pub fn foundation_content() -> FoundationContent {
    let content_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("content");
    bd_data::loader::load_foundation_content(&content_dir)
        .expect("foundation content must validate for headless tests")
}

#[cfg(test)]
mod tests {
    use super::*;
    use bd_core::{
        components::{Name, Player, Position},
        pools::Pools,
        session::{RunOutcome, RunSession},
        signals::{ActionIntent, PoolKind},
        spatial::{GameMode, PersistentEntity, TransientEntity, TransitionIntent},
    };
    use bevy_ecs::{message::Messages, query::With};

    fn enter_foundation_dungeon(app: &mut App) {
        app.world_mut()
            .resource_mut::<Messages<TransitionIntent>>()
            .write(TransitionIntent {
                target: GameMode::Outpost,
                node_id: None,
            });
        app.update();
        app.world_mut()
            .resource_mut::<Messages<TransitionIntent>>()
            .write(TransitionIntent {
                target: GameMode::Tactical,
                node_id: Some("dungeon.foundation".into()),
            });
        app.update();
    }

    fn player_state(app: &mut App) -> (bevy_ecs::entity::Entity, Position, i32, i32) {
        let mut query = app
            .world_mut()
            .query_filtered::<(bevy_ecs::entity::Entity, &Position, &Pools), With<Player>>();
        let (entity, position, pools) = query
            .iter(app.world())
            .next()
            .expect("foundation dungeon should contain a player");
        let hp = pools
            .get(PoolKind::Health)
            .expect("player should have health")
            .current;
        let ap = pools
            .get(PoolKind::ActionPoints)
            .expect("player should have action points")
            .current;
        (entity, *position, hp, ap)
    }

    fn rat_state(app: &mut App) -> (Position, i32) {
        let mut query = app
            .world_mut()
            .query_filtered::<(&Position, &Pools), With<Name>>();
        query
            .iter(app.world())
            .find(|(_, pools)| pools.get(PoolKind::Health).is_some())
            .map(|(position, pools)| {
                (
                    *position,
                    pools
                        .get(PoolKind::Health)
                        .expect("hostile should have health")
                        .current,
                )
            })
            .expect("foundation dungeon should contain a named hostile")
    }

    #[test]
    fn foundation_app_excludes_deferred_resources() {
        let app = foundation_app();

        assert!(
            app.world()
                .get_resource::<bd_core::events::EventRegistry>()
                .is_none()
        );
        assert!(
            app.world()
                .get_resource::<bd_core::events::CurrentEvent>()
                .is_none()
        );
        assert!(
            app.world()
                .get_resource::<bd_core::factions::FactionReputation>()
                .is_none()
        );
        assert!(
            app.world()
                .get_resource::<bd_core::overworld::OverworldState>()
                .is_none()
        );
        assert!(
            app.world()
                .get_resource::<bd_core::overworld::TravelContext>()
                .is_none()
        );
        assert!(
            app.world()
                .get_resource::<bd_core::party::PartyState>()
                .is_none()
        );
        assert!(
            app.world()
                .get_resource::<bd_core::colony::raids::RaidState>()
                .is_none()
        );
        assert!(
            app.world()
                .get_resource::<bd_core::dialogue::DialogueLog>()
                .is_none()
        );
        assert!(
            app.world()
                .get_resource::<bd_core::gabriel::GabrielState>()
                .is_none()
        );
        assert!(app.world().get_resource::<FoundationContent>().is_some());
    }

    #[test]
    fn foundation_colony_dungeon_round_trip_preserves_colony_state() {
        let mut app = foundation_app();

        app.world_mut()
            .resource_mut::<Messages<TransitionIntent>>()
            .write(TransitionIntent {
                target: GameMode::Outpost,
                node_id: None,
            });
        app.update();

        let survivor_count = app
            .world_mut()
            .query_filtered::<(), With<bd_core::colony::survivors::Survivor>>()
            .iter(app.world())
            .count();
        assert_eq!(survivor_count, 3);
        let supplies_before = app
            .world()
            .resource::<bd_core::colony::production::ColonyResources>()
            .pools
            .get(bd_core::signals::PoolKind::Supplies)
            .unwrap()
            .current;

        app.world_mut()
            .resource_mut::<Messages<TransitionIntent>>()
            .write(TransitionIntent {
                target: GameMode::Tactical,
                node_id: Some("dungeon.foundation".into()),
            });
        app.update();

        assert_eq!(
            app.world().resource::<GameMode>().clone(),
            GameMode::Tactical
        );
        assert_eq!(app.world().resource::<bd_core::map::SmokeMap>().width, 8);
        assert_eq!(app.world().resource::<bd_core::map::SmokeMap>().height, 6);
        let factioned_enemies = app
            .world_mut()
            .query_filtered::<&bd_core::relationships::FactionMember, With<bd_core::relationships::FactionMember>>()
            .iter(app.world())
            .map(|faction| faction.0.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            factioned_enemies,
            vec!["faction.placeholder_a"],
            "fixed encounter should carry content faction identity"
        );
        let player = app
            .world_mut()
            .query_filtered::<(bevy_ecs::entity::Entity, &Position), With<Player>>()
            .iter(app.world())
            .next()
            .map(|(entity, position)| (entity, *position))
            .expect("fixed dungeon should provide a player");
        let potion = app
            .world_mut()
            .query_filtered::<bevy_ecs::entity::Entity, With<bd_core::inventory::Item>>()
            .iter(app.world())
            .next()
            .expect("fixed dungeon should provide loot");
        app.world_mut()
            .entity_mut(potion)
            .insert(bd_core::relationships::ContainedIn(player.0));
        assert_eq!(player.1, Position { x: 1, y: 1 });

        app.world_mut()
            .entity_mut(player.0)
            .insert(Position { x: 6, y: 4 });

        app.world_mut()
            .resource_mut::<Messages<bd_core::signals::ActionIntent>>()
            .write(bd_core::signals::ActionIntent {
                actor: player.0,
                action_id: "ability.extract".into(),
                direction: None,
                target: None,
            });
        app.update();
        app.update();

        assert_eq!(
            app.world().resource::<GameMode>().clone(),
            GameMode::Outpost
        );
        assert_eq!(
            app.world().resource::<RunSession>().outcome,
            RunOutcome::Extracted
        );
        assert_eq!(
            app.world().resource::<RunSession>().extraction_applied,
            true
        );
        let survivor_count_after = app
            .world_mut()
            .query_filtered::<(), With<bd_core::colony::survivors::Survivor>>()
            .iter(app.world())
            .count();
        assert_eq!(survivor_count_after, 3);
        let supplies_after = app
            .world()
            .resource::<bd_core::colony::production::ColonyResources>()
            .pools
            .get(bd_core::signals::PoolKind::Supplies)
            .unwrap()
            .current;
        assert_eq!(supplies_after, supplies_before);
        assert_eq!(
            app.world()
                .resource::<bd_core::colony::production::ColonyStorage>()
                .count("item.healing_potion"),
            1,
            "extracted carried loot should enter colony storage exactly once",
        );
        let transient_count = app
            .world_mut()
            .query_filtered::<(), With<TransientEntity>>()
            .iter(app.world())
            .count();
        assert_eq!(transient_count, 0);
        assert!(
            app.world_mut()
                .query_filtered::<(), With<PersistentEntity>>()
                .iter(app.world())
                .count()
                >= 4
        );
    }

    #[test]
    fn foundation_turn_idle_does_not_run_enemy_ai() {
        let mut app = foundation_app();
        enter_foundation_dungeon(&mut app);

        let (_, _, hp_before, _) = player_state(&mut app);
        let (rat_position_before, _) = rat_state(&mut app);
        let turn_before = app.world().resource::<bd_core::time::GameTime>().turn;

        for _ in 0..3 {
            app.update();
        }

        let (_, _, hp_after, _) = player_state(&mut app);
        let (rat_position_after, _) = rat_state(&mut app);
        let turn_after = app.world().resource::<bd_core::time::GameTime>().turn;

        assert_eq!(
            hp_after, hp_before,
            "idle frames must not damage the player"
        );
        assert_eq!(
            rat_position_after, rat_position_before,
            "idle frames must not move enemies"
        );
        assert_eq!(turn_after, turn_before, "idle frames must not advance time");
    }

    #[test]
    fn accepted_move_advances_exactly_one_turn() {
        let mut app = foundation_app();
        enter_foundation_dungeon(&mut app);
        let (player, _, _, ap_before) = player_state(&mut app);
        let turn_before = app.world().resource::<bd_core::time::GameTime>().turn;

        app.world_mut()
            .resource_mut::<Messages<ActionIntent>>()
            .write(ActionIntent {
                actor: player,
                action_id: "ability.move".into(),
                direction: Some(bd_core::direction::Direction::East),
                target: None,
            });
        app.update();

        let (_, _, _, ap_after) = player_state(&mut app);
        let turn_after = app.world().resource::<bd_core::time::GameTime>().turn;
        assert_eq!(
            turn_after,
            turn_before + 1,
            "one accepted move must advance exactly one turn"
        );
        assert_eq!(ap_after, ap_before - 1, "one move must spend one AP");
    }

    #[test]
    fn rejected_action_does_not_start_enemy_phase() {
        let mut app = foundation_app();
        enter_foundation_dungeon(&mut app);
        let (player, _, hp_before, ap_before) = player_state(&mut app);
        let (rat_position_before, _) = rat_state(&mut app);

        let rat = app
            .world_mut()
            .query_filtered::<bevy_ecs::entity::Entity, With<Name>>()
            .iter(app.world())
            .next()
            .expect("foundation dungeon should contain a named hostile");
        app.world_mut()
            .resource_mut::<Messages<ActionIntent>>()
            .write(ActionIntent {
                actor: player,
                action_id: "ability.attack".into(),
                direction: None,
                target: Some(rat),
            });
        app.update();

        let (_, _, hp_after, ap_after) = player_state(&mut app);
        let (rat_position_after, _) = rat_state(&mut app);
        assert_eq!(
            hp_after, hp_before,
            "rejected actions must not trigger enemy damage"
        );
        assert_eq!(ap_after, ap_before, "rejected actions must not spend AP");
        assert_eq!(
            rat_position_after, rat_position_before,
            "rejected actions must not trigger enemy movement"
        );
    }

    #[test]
    fn accepted_action_runs_enemy_phase_once() {
        let mut app = foundation_app();
        enter_foundation_dungeon(&mut app);
        let (player, _, _, _) = player_state(&mut app);

        app.world_mut()
            .resource_mut::<Messages<ActionIntent>>()
            .write(ActionIntent {
                actor: player,
                action_id: "ability.move".into(),
                direction: Some(bd_core::direction::Direction::East),
                target: None,
            });
        app.update();

        let (rat_after_player, _) = rat_state(&mut app);
        app.update();
        let (rat_after_enemy, _) = rat_state(&mut app);
        app.update();
        let (rat_after_idle, _) = rat_state(&mut app);

        assert_ne!(
            rat_after_enemy, rat_after_player,
            "accepted player action should permit one enemy phase"
        );
        assert_eq!(
            rat_after_idle, rat_after_enemy,
            "one player action must not permit repeated enemy phases"
        );
    }

    #[test]
    fn player_action_is_locked_during_enemy_phase() {
        let mut app = foundation_app();
        enter_foundation_dungeon(&mut app);
        let (player, _, _, ap_before) = player_state(&mut app);

        app.world_mut()
            .resource_mut::<Messages<ActionIntent>>()
            .write(ActionIntent {
                actor: player,
                action_id: "ability.move".into(),
                direction: Some(bd_core::direction::Direction::East),
                target: None,
            });
        app.update();
        let (_, _, _, ap_after_first) = player_state(&mut app);

        // This intent arrives while the enemy phase is pending and must not
        // become a second player turn in the same resolution window.
        app.world_mut()
            .resource_mut::<Messages<ActionIntent>>()
            .write(ActionIntent {
                actor: player,
                action_id: "ability.move".into(),
                direction: Some(bd_core::direction::Direction::East),
                target: None,
            });
        app.update();

        let (_, player_position, _, ap_after) = player_state(&mut app);
        assert_eq!(player_position, Position { x: 2, y: 1 });
        assert!(
            ap_after > ap_after_first,
            "enemy-phase input must not spend AP; expected regeneration without a second cost (before {}, after {})",
            ap_after_first,
            ap_after
        );
        assert!(ap_before >= ap_after);
    }
}
