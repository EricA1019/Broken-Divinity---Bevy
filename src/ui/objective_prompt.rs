use bevy::prelude::*;

use crate::core::state::AppState;
use crate::game::colony::raids::{ActiveRaid, RaidPhase};

pub const COLONY_OBJECTIVE_PROMPT_TEXT: &str =
    "Objective: Reach the shelter gate (stairs) and press Enter to travel to the overworld.";

#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct InstructionPriorityPolicy {
    pub suppress_secondary_hints_when_primary_active: bool,
}

impl Default for InstructionPriorityPolicy {
    fn default() -> Self {
        Self {
            suppress_secondary_hints_when_primary_active: true,
        }
    }
}

#[derive(Resource, Debug, Clone, Reflect, Default)]
#[reflect(Resource)]
pub struct ColonyObjectivePromptState {
    pub has_reached_overworld: bool,
    pub visible_in_colony: bool,
}

fn raid_blocks_prompt(active_raid: Option<&ActiveRaid>) -> bool {
    active_raid.is_some_and(|raid| raid.phase == RaidPhase::Planning)
}

pub fn refresh_colony_objective_prompt(
    state: Res<State<AppState>>,
    active_raid: Option<Res<ActiveRaid>>,
    mut prompt: ResMut<ColonyObjectivePromptState>,
) {
    match *state.get() {
        AppState::Overworld => {
            prompt.has_reached_overworld = true;
            prompt.visible_in_colony = false;
        }
        AppState::Colony => {
            prompt.visible_in_colony =
                !prompt.has_reached_overworld && !raid_blocks_prompt(active_raid.as_deref());
        }
        _ => {
            prompt.visible_in_colony = false;
        }
    }
}
