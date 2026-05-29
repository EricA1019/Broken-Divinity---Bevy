use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

use broken_divinity::core::state::AppState;
use broken_divinity::game::colony::raids::ActiveRaid;
use broken_divinity::game::colony::raids::RaidPhase;
use broken_divinity::ui::help_panel::{HelpOpen, colony_help_shows_secondary_hints, toggle_help};
use broken_divinity::ui::modal_priority::{
    ModalBlockers, ModalPriorityCoordinator, apply_modal_priority_policy,
};
use broken_divinity::ui::objective_prompt::{
    COLONY_OBJECTIVE_PROMPT_TEXT, ColonyObjectivePromptState, InstructionPriorityPolicy,
    refresh_colony_objective_prompt,
};

const HELP_TOGGLE_KEY: KeyCode = KeyCode::F1;

fn keyboard_with_pressed(key: KeyCode) -> ButtonInput<KeyCode> {
    let mut keyboard = ButtonInput::<KeyCode>::default();
    keyboard.press(key);
    keyboard
}

#[test]
fn help_toggle_suppresses_open_state_when_raid_is_active() {
    let mut world = World::new();
    world.insert_resource(keyboard_with_pressed(HELP_TOGGLE_KEY));
    world.insert_resource(HelpOpen(true));
    world.insert_resource(ActiveRaid {
        raider_count: 3,
        raider_strength: 7,
        casualties: 0,
        resources_stolen: 0,
        phase: RaidPhase::Planning,
    });

    let _ = world.run_system_once(toggle_help);

    let help_open = world.resource::<HelpOpen>();
    assert!(
        !help_open.0,
        "expected active raid to suppress help panel"
    );
}

#[test]
fn modal_priority_policy_sets_blocker_when_raid_becomes_active() {
    let mut world = World::new();
    world.insert_resource(HelpOpen(true));
    world.insert_resource(ModalPriorityCoordinator);
    world.insert_resource(ModalBlockers {
        critical_modal_active: false,
    });
    world.insert_resource(ActiveRaid {
        raider_count: 2,
        raider_strength: 5,
        casualties: 0,
        resources_stolen: 0,
        phase: RaidPhase::Warning,
    });

    let _ = world.run_system_once(apply_modal_priority_policy);

    let blockers = world.resource::<ModalBlockers>();
    let help_open = world.resource::<HelpOpen>();

    assert!(blockers.critical_modal_active);
    assert!(!help_open.0);
}

#[test]
fn colony_objective_visibility_tracks_state_progression() {
    let mut world = World::new();
    world.insert_resource(State::new(AppState::Colony));
    world.insert_resource(ColonyObjectivePromptState::default());

    let _ = world.run_system_once(refresh_colony_objective_prompt);

    let prompt = world.resource::<ColonyObjectivePromptState>();
    assert!(prompt.visible_in_colony);
    assert!(!prompt.has_reached_overworld);

    world.insert_resource(State::new(AppState::Overworld));
    let _ = world.run_system_once(refresh_colony_objective_prompt);

    let prompt = world.resource::<ColonyObjectivePromptState>();
    assert!(!prompt.visible_in_colony);
    assert!(prompt.has_reached_overworld);

    world.insert_resource(State::new(AppState::Colony));
    let _ = world.run_system_once(refresh_colony_objective_prompt);

    let prompt = world.resource::<ColonyObjectivePromptState>();
    assert!(!prompt.visible_in_colony);
}

#[test]
fn colony_top_bar_presentation_prefers_primary_cta_and_hover_detail() {
    let policy = InstructionPriorityPolicy::default();
    let prompt = ColonyObjectivePromptState {
        has_reached_overworld: false,
        visible_in_colony: true,
    };

    assert!(
        !colony_help_shows_secondary_hints(Some(&prompt), &policy),
        "expected primary objective to suppress secondary hints"
    );
    assert!(
        COLONY_OBJECTIVE_PROMPT_TEXT.contains("press Enter"),
        "expected objective prompt to keep Enter key hint"
    );
}
