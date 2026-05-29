use bevy::prelude::*;

use crate::game::colony::raids::ActiveRaid;
use crate::modal_priority::ModalBlockers;
use crate::objective_prompt::{ColonyObjectivePromptState, InstructionPriorityPolicy};
use crate::ui::input_hints::HELP_TOGGLE_KEY;

#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HelpOpen(pub bool);

pub fn toggle_help(
    keyboard: Res<ButtonInput<KeyCode>>,
    blockers: Option<Res<ModalBlockers>>,
    active_raid: Option<Res<ActiveRaid>>,
    mut help_open: ResMut<HelpOpen>,
) {
    if !keyboard.just_pressed(HELP_TOGGLE_KEY) {
        return;
    }

    if active_raid.is_some()
        || blockers
            .as_ref()
            .is_some_and(|blockers| blockers.critical_modal_active)
    {
        help_open.0 = false;
        return;
    }

    help_open.0 = !help_open.0;
}

pub fn colony_help_shows_secondary_hints(
    objective_prompt: Option<&ColonyObjectivePromptState>,
    _policy: &InstructionPriorityPolicy,
) -> bool {
    match objective_prompt {
        Some(prompt) => prompt.has_reached_overworld || !prompt.visible_in_colony,
        None => true,
    }
}
