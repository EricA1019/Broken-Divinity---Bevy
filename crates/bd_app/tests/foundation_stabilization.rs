//! Red-first acceptance queue for the Foundation stabilization plan.
//!
//! Every test drives the production Foundation plugin through typed actions
//! and transitions. Phase 1 intentionally checks these in before their owning
//! implementation phases make them green.

use bd_core::{direction::Direction, signals::PoolKind, spatial::GameMode};
use bd_test_support::FoundationDriver;

const FIXED_DUNGEON: &str = "dungeon.foundation";
const BUILD_COST: i32 = 2;
const ENTRY_COST: i32 = 2;

fn colony_driver() -> FoundationDriver {
    let mut driver = FoundationDriver::new(0);
    driver
        .start_colony()
        .expect("clean launch must reach the shelter");
    driver
}

fn dungeon_driver() -> FoundationDriver {
    let mut driver = colony_driver();
    driver
        .enter_dungeon(FIXED_DUNGEON)
        .expect("the canonical entry action must reach the fixed dungeon");
    driver
}

fn full_runtime_dungeon_driver() -> FoundationDriver {
    let mut driver = FoundationDriver::new_with_plugin(0, bd_tui::BdTuiPlugin);
    driver.install_command_error_capture();
    driver
        .start_colony()
        .expect("full runtime must reach the shelter");
    driver
        .enter_dungeon(FIXED_DUNGEON)
        .expect("full runtime must reach the fixed dungeon");
    driver
}

#[test]
fn construction_deducts_authoritative_colony_supplies_once() {
    let mut driver = colony_driver();
    let player = driver.player().expect("shelter player must exist");
    let before = driver
        .resource_current(PoolKind::Supplies)
        .expect("colony supplies must exist");
    let stations_before = driver.summary().stations;

    driver
        .submit_action_and_advance_result_frame(
            "build one Stove",
            player,
            "ability.build",
            Some(Direction::East),
            None,
        )
        .expect("valid construction must resolve");

    assert_eq!(
        driver.resource_current(PoolKind::Supplies),
        Some(before - BUILD_COST),
        "the authoritative colony pool must pay exactly one build cost"
    );
    assert_eq!(driver.summary().stations, stations_before + 1);
}

#[test]
fn construction_denial_preserves_all_state() {
    let mut driver = colony_driver();
    driver.fixture_set_colony_resource(PoolKind::Supplies, BUILD_COST - 1);
    let player = driver.player().expect("shelter player must exist");
    let before = driver.summary();
    let resources_before = driver.resource_current(PoolKind::Supplies);

    driver
        .expect_denied_action(
            "deny unaffordable construction",
            player,
            "ability.build",
            Some(Direction::East),
            None,
        )
        .expect("unaffordable construction must emit a typed denial");

    let after = driver.summary();
    assert_eq!(after.stations, before.stations);
    assert_eq!(after.turn, before.turn);
    assert_eq!(
        driver.resource_current(PoolKind::Supplies),
        resources_before
    );
}

#[test]
fn foundation_player_has_no_colony_supplies_pool() {
    let mut driver = colony_driver();

    assert!(
        !driver.player_pool_kinds().contains(&PoolKind::Supplies),
        "Foundation player Pools must not duplicate authoritative colony Supplies"
    );
}

#[test]
fn dungeon_entry_deducts_colony_supplies_once() {
    let mut driver = colony_driver();
    let before = driver
        .resource_current(PoolKind::Supplies)
        .expect("colony supplies must exist");
    let player = driver.player().expect("shelter player must exist");

    driver
        .submit_action_and_advance_result_frame(
            "paid Foundation dungeon entry",
            player,
            "ability.enter_foundation_dungeon",
            None,
            None,
        )
        .expect("entry action must resolve");
    driver.advance_transition_frame();

    assert_eq!(driver.summary().mode, GameMode::Tactical);
    assert_eq!(
        driver.resource_current(PoolKind::Supplies),
        Some(before - ENTRY_COST)
    );
}

#[test]
fn dungeon_entry_denial_preserves_mode_turn_and_resources() {
    let mut driver = colony_driver();
    driver.fixture_set_colony_resource(PoolKind::Supplies, ENTRY_COST - 1);
    let player = driver.player().expect("shelter player must exist");
    let before = driver.summary();
    let resources_before = driver.resource_current(PoolKind::Supplies);
    let entities_before = driver.entity_count();

    driver
        .expect_denied_action(
            "deny unaffordable Foundation entry",
            player,
            "ability.enter_foundation_dungeon",
            None,
            None,
        )
        .expect("entry denial must be typed");

    let after = driver.summary();
    assert_eq!(after.mode, before.mode);
    assert_eq!(after.turn, before.turn);
    assert_eq!(
        driver.resource_current(PoolKind::Supplies),
        resources_before
    );
    assert_eq!(driver.entity_count(), entities_before);
}

#[test]
fn dungeon_entry_replay_preserves_cost_and_transition() {
    let mut driver = colony_driver();
    let player = driver.player().expect("shelter player must exist");

    driver
        .submit_action_and_advance_result_frame(
            "record Foundation dungeon entry",
            player,
            "ability.enter_foundation_dungeon",
            None,
            None,
        )
        .expect("entry action must resolve");
    driver.advance_transition_frame();

    let summary = driver.summary();
    assert_eq!(summary.mode, GameMode::Tactical);
    assert_eq!(summary.dungeon_id.as_deref(), Some(FIXED_DUNGEON));
    assert_eq!(
        summary
            .replay_intents
            .iter()
            .filter(|record| record.action_id == "ability.enter_foundation_dungeon")
            .count(),
        1
    );
}

#[test]
fn construction_cost_survives_save_load() {
    let mut driver = colony_driver();
    let player = driver.player().expect("shelter player must exist");
    driver
        .submit_action_and_advance_result_frame(
            "build before save",
            player,
            "ability.build",
            Some(Direction::East),
            None,
        )
        .expect("construction must resolve");
    let expected = driver.resource_current(PoolKind::Supplies);
    let checkpoint = driver.checkpoint().expect("construction state must save");

    let restored =
        FoundationDriver::from_checkpoint(&checkpoint).expect("construction state must load");
    assert_eq!(restored.resource_current(PoolKind::Supplies), expected);
}

#[test]
fn paid_dungeon_entry_survives_save_load() {
    let mut driver = dungeon_driver();
    let expected = driver.resource_current(PoolKind::Supplies);
    let checkpoint = driver.checkpoint().expect("dungeon state must save");

    let mut restored =
        FoundationDriver::from_checkpoint(&checkpoint).expect("dungeon state must load");
    assert_eq!(restored.resource_current(PoolKind::Supplies), expected);
    assert_eq!(restored.summary().mode, GameMode::Tactical);
}

#[test]
fn two_builds_charge_twice_and_third_unaffordable_build_is_atomic() {
    let mut driver = colony_driver();
    let stations_before = driver.summary().stations;
    driver.fixture_set_colony_resource(PoolKind::Supplies, BUILD_COST * 2);
    let player = driver.player().expect("shelter player must exist");
    driver
        .submit_action_and_advance_result_frame(
            "first affordable build",
            player,
            "ability.build",
            Some(Direction::East),
            None,
        )
        .expect("first build must resolve");
    driver
        .submit_action_and_advance_result_frame(
            "move to a second safe footprint",
            player,
            "ability.move",
            Some(Direction::South),
            None,
        )
        .expect("player must reach a non-trapping build position");
    driver
        .submit_action_and_advance_result_frame(
            "second affordable build",
            player,
            "ability.build",
            Some(Direction::East),
            None,
        )
        .expect("second build must resolve");
    assert_eq!(driver.resource_current(PoolKind::Supplies), Some(0));
    assert_eq!(driver.summary().stations, stations_before + 2);
    let before = driver.summary();

    driver
        .expect_denied_action(
            "deny third build",
            player,
            "ability.build",
            Some(Direction::South),
            None,
        )
        .expect("third build must emit a typed denial");

    let after = driver.summary();
    assert_eq!(after.stations, before.stations);
    assert_eq!(after.turn, before.turn);
    assert_eq!(driver.resource_current(PoolKind::Supplies), Some(0));
}

#[test]
fn fatal_enemy_action_reaches_game_over_without_command_error() {
    let mut driver = full_runtime_dungeon_driver();
    let player = driver.player().expect("dungeon player must exist");

    driver
        .wait_for_player_defeat("normal enemy combat must defeat the player")
        .expect("defeat path must reach Game Over");
    driver.submit_buffered_action(player, "ability.wait", None, None);
    for _ in 0..20 {
        driver.advance_idle();
    }

    assert_eq!(driver.summary().mode, GameMode::GameOver);
    assert!(
        driver.command_errors().is_empty(),
        "fatal combat emitted Bevy command errors: {:?}",
        driver.command_errors()
    );
}

#[test]
fn fatal_action_emits_one_defeat_and_one_cleanup() {
    let mut driver = full_runtime_dungeon_driver();
    let player = driver.player().expect("dungeon player must exist");

    driver
        .wait_for_player_defeat("fatal action cleanup")
        .expect("defeat path must reach Game Over");

    assert_eq!(driver.last_defeat_count(), 1);
    assert!(
        !driver.entity_exists(player),
        "the single defeated player must be cleaned up"
    );
    assert!(driver.command_errors().is_empty());
}

#[test]
fn fatal_enemy_action_emits_one_entity_defeated() {
    let mut driver = full_runtime_dungeon_driver();

    driver
        .wait_for_player_defeat("single fatal enemy action")
        .expect("defeat path must reach Game Over");

    assert_eq!(driver.last_defeat_count(), 1);
}

#[test]
fn player_defeat_marks_session_once() {
    let mut driver = full_runtime_dungeon_driver();
    driver
        .wait_for_player_defeat("mark defeated session")
        .expect("defeat path must reach Game Over");
    let defeated = driver.summary();

    for _ in 0..20 {
        driver.advance_idle();
    }

    let idle = driver.summary();
    assert_eq!(defeated.outcome, bd_core::session::RunOutcome::Defeated);
    assert_eq!(idle.outcome, defeated.outcome);
    assert_eq!(idle.mode, GameMode::GameOver);
    assert_eq!(idle.turn, defeated.turn);
}

#[test]
fn game_over_save_load_contains_no_invalid_relationship() {
    let mut driver = full_runtime_dungeon_driver();
    driver
        .wait_for_player_defeat("save defeated state")
        .expect("defeat path must reach Game Over");
    let checkpoint = driver.checkpoint().expect("Game Over must save");

    let mut restored = FoundationDriver::from_checkpoint(&checkpoint).expect("Game Over must load");
    let summary = restored.summary();
    assert_eq!(summary.mode, GameMode::GameOver);
    assert_eq!(summary.outcome, bd_core::session::RunOutcome::Defeated);
    assert_eq!(restored.player_count(), 0);
}

#[test]
fn restart_after_defeat_creates_exactly_one_player() {
    let mut driver = full_runtime_dungeon_driver();
    driver
        .wait_for_player_defeat("restart after defeat")
        .expect("defeat path must reach Game Over");
    driver
        .request_transition("Game Over to title", GameMode::Title, None)
        .expect("Game Over must restart to title");
    driver
        .start_colony()
        .expect("restarted title must enter the shelter");

    assert_eq!(driver.player_count(), 1);
    assert_eq!(
        driver.scope_count(bd_core::spatial::EntityScope::DungeonTransient),
        0
    );
}

#[test]
fn idle_on_extraction_tile_does_not_mutate_log() {
    let mut driver = dungeon_driver();
    driver
        .approach_and_defeat_first_hostile("clear extraction route")
        .expect("hostile must be cleared");
    driver
        .move_player_to_exit("stand on extraction tile")
        .expect("exit must be reachable");
    let before_log = driver.log_messages();
    let before_summary = driver.summary();
    let before_entities = driver.entity_count();
    let before_supplies = driver.resource_current(PoolKind::Supplies);

    for _ in 0..300 {
        driver.advance_idle();
    }

    assert_eq!(driver.log_messages(), before_log);
    assert_eq!(driver.summary(), before_summary);
    assert_eq!(driver.entity_count(), before_entities);
    assert_eq!(driver.resource_current(PoolKind::Supplies), before_supplies);
}

#[test]
fn idle_on_shelter_gate_does_not_mutate_log() {
    let mut driver = colony_driver();
    let gate = driver.exit_position().expect("shelter gate must exist");
    driver
        .move_player_to("stand on shelter gate", gate)
        .expect("shelter gate must be reachable");
    let before_log = driver.log_messages();
    let before_summary = driver.summary();
    let before_entities = driver.entity_count();
    let before_supplies = driver.resource_current(PoolKind::Supplies);

    for _ in 0..300 {
        driver.advance_idle();
    }

    assert_eq!(driver.log_messages(), before_log);
    assert_eq!(driver.summary(), before_summary);
    assert_eq!(driver.entity_count(), before_entities);
    assert_eq!(driver.resource_current(PoolKind::Supplies), before_supplies);
}

#[test]
fn rest_until_next_day_emits_one_day_boundary() {
    let mut driver = colony_driver();
    let player = driver.player().expect("shelter player must exist");
    let day_before = driver.summary().day;

    driver
        .submit_action_and_advance_result_frame(
            "rest until next day",
            player,
            "ability.rest_until_next_day",
            None,
            None,
        )
        .expect("Rest Until Next Day must resolve in the shelter");

    assert_eq!(driver.summary().day, day_before + 1);
    assert_eq!(driver.last_day_advanced_count(), 1);
}

#[test]
fn rest_until_next_day_is_rejected_outside_outpost() {
    let mut driver = dungeon_driver();
    let player = driver.player().expect("dungeon player must exist");
    let before = driver.summary();

    let denial = driver
        .expect_denied_action(
            "reject tactical rest",
            player,
            "ability.rest_until_next_day",
            None,
            None,
        )
        .expect("Rest must emit a typed denial outside the shelter");

    assert!(
        !format!("{denial:?}").contains("Unknown action"),
        "mode rejection must not be disguised as an unknown action"
    );
    let after = driver.summary();
    assert_eq!(after.mode, before.mode);
    assert_eq!(after.day, before.day);
    assert_eq!(after.turn, before.turn);
}

#[test]
fn rest_until_next_day_is_denied_during_build_interaction() {
    let mut driver = colony_driver();
    driver.fixture_set_build_interaction(true);
    let player = driver.player().expect("shelter player must exist");
    let before = driver.summary();

    driver
        .expect_denied_action(
            "deny rest during build interaction",
            player,
            "ability.rest_until_next_day",
            None,
            None,
        )
        .expect("build interaction must emit a typed Rest denial");

    let after = driver.summary();
    assert_eq!((after.day, after.turn), (before.day, before.turn));
    assert_eq!(driver.last_day_advanced_count(), 0);
}

#[test]
fn rest_until_next_day_is_denied_during_event_interaction() {
    let mut driver = colony_driver();
    driver.fixture_set_event_interaction(true);
    let player = driver.player().expect("shelter player must exist");
    let before = driver.summary();

    driver
        .expect_denied_action(
            "deny rest during event interaction",
            player,
            "ability.rest_until_next_day",
            None,
            None,
        )
        .expect("event interaction must emit a typed Rest denial");

    let after = driver.summary();
    assert_eq!((after.day, after.turn), (before.day, before.turn));
    assert_eq!(driver.last_day_advanced_count(), 0);
}
