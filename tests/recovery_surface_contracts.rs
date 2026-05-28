use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use broken_divinity::core::state::AppState;
use broken_divinity::game::overworld::weather::{Weather, roll_weather};
use broken_divinity::ui::help_panel::{HelpOpen, colony_help_shows_secondary_hints, toggle_help};
use broken_divinity::ui::modal_priority::{
    ModalBlockers, ModalPriorityCoordinator, apply_modal_priority_policy,
};
use broken_divinity::ui::objective_prompt::{
    ColonyObjectivePromptState, InstructionPriorityPolicy, refresh_colony_objective_prompt,
};
use broken_divinity::ui::overworld_panel::primary_overworld_cta_label;
use broken_divinity::ui::readability::contrast_ratio;

const BLACK_RGB: (u8, u8, u8) = (0, 0, 0);
const WHITE_RGB: (u8, u8, u8) = (255, 255, 255);
const MAX_CONTRAST_RATIO: f32 = 21.0;
const CONTRAST_TOLERANCE: f32 = 0.05;
const WEATHER_TEST_SEED: u64 = 77;
const WEATHER_TEST_DAY: u32 = 5;

#[test]
fn readability_contrast_ratio_matches_wcag_extremes() {
    let ratio = contrast_ratio(WHITE_RGB, BLACK_RGB);
    let symmetric_ratio = contrast_ratio(BLACK_RGB, WHITE_RGB);

    assert!(
        (ratio - MAX_CONTRAST_RATIO).abs() <= CONTRAST_TOLERANCE,
        "expected WCAG max contrast ratio, got {ratio}"
    );
    assert!(
        (ratio - symmetric_ratio).abs() <= CONTRAST_TOLERANCE,
        "expected contrast ratio to be symmetric"
    );
}

#[test]
fn help_toggle_respects_critical_modal_blocker() {
    let mut world = World::new();
    let mut keys = ButtonInput::<KeyCode>::default();
    keys.press(KeyCode::F1);

    world.insert_resource(keys);
    world.insert_resource(HelpOpen(false));
    world.insert_resource(ModalBlockers {
        critical_modal_active: true,
    });

    let _ = world.run_system_once(toggle_help);

    assert!(
        !world.resource::<HelpOpen>().0,
        "expected critical modal blocker to keep help closed"
    );
}

#[test]
fn modal_priority_policy_closes_help_when_critical_modal_is_active() {
    let mut world = World::new();
    world.insert_resource(HelpOpen(true));
    world.insert_resource(ModalPriorityCoordinator);
    world.insert_resource(ModalBlockers {
        critical_modal_active: true,
    });

    let _ = world.run_system_once(apply_modal_priority_policy);

    assert!(
        !world.resource::<HelpOpen>().0,
        "expected modal priority policy to close help when critical modal is active"
    );
}

#[test]
fn colony_help_secondary_hints_unlock_after_primary_objective() {
    let policy = InstructionPriorityPolicy::default();
    let active_prompt = ColonyObjectivePromptState {
        has_reached_overworld: false,
        visible_in_colony: true,
    };
    let cleared_prompt = ColonyObjectivePromptState {
        has_reached_overworld: true,
        visible_in_colony: false,
    };

    assert!(
        !colony_help_shows_secondary_hints(Some(&active_prompt), &policy),
        "expected primary colony objective to suppress secondary help hints"
    );
    assert!(
        colony_help_shows_secondary_hints(Some(&cleared_prompt), &policy),
        "expected secondary help hints after overworld milestone is complete"
    );
}

#[test]
fn colony_objective_prompt_refresh_tracks_overworld_progress() {
    let mut world = World::new();
    world.insert_resource(State::new(AppState::Colony));
    world.insert_resource(ColonyObjectivePromptState::default());

    let _ = world.run_system_once(refresh_colony_objective_prompt);

    assert!(
        world
            .resource::<ColonyObjectivePromptState>()
            .visible_in_colony,
        "expected objective prompt to be visible on fresh colony entry"
    );

    world.insert_resource(State::new(AppState::Overworld));
    let _ = world.run_system_once(refresh_colony_objective_prompt);

    let prompt = world.resource::<ColonyObjectivePromptState>();
    assert!(
        prompt.has_reached_overworld,
        "expected overworld milestone to persist"
    );
    assert!(
        !prompt.visible_in_colony,
        "expected colony prompt to hide once overworld milestone is reached"
    );
}

#[test]
fn overworld_primary_cta_mentions_connected_node_travel() {
    assert!(
        primary_overworld_cta_label().contains("connected node"),
        "expected overworld CTA to emphasize connected node travel"
    );
}

#[test]
fn weather_roll_is_deterministic_and_ashfall_applies_pressure() {
    let first_roll = roll_weather(WEATHER_TEST_SEED, WEATHER_TEST_DAY);
    let second_roll = roll_weather(WEATHER_TEST_SEED, WEATHER_TEST_DAY);

    assert_eq!(
        first_roll, second_roll,
        "expected deterministic weather rolls"
    );
    assert_eq!(Weather::Ashfall.sanity_pressure(), 1);
    assert!(Weather::Clear.travel_speed() > 0.0);
    assert!(!Weather::Fog.name().is_empty());
}
