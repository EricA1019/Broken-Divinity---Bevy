use bevy::prelude::*;

use crate::core_state::AppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstructionPriority {
    Primary,
    Secondary,
    Tertiary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstructionEvent {
    PrimaryShown,
    SecondaryShown,
    TertiaryShown,
    SuppressedDuplicate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstructionDecision {
    pub priority: InstructionPriority,
    pub kind: InstructionEvent,
}

#[derive(Debug, Clone, Copy)]
pub struct ObjectivePromptPolicy {
    primary_when_objective_active: InstructionPriority,
    secondary_when_objective_inactive: InstructionPriority,
    tertiary_when_hints_disabled: InstructionPriority,
}

#[derive(Debug, Clone, Copy)]
pub struct ObjectivePromptEngine {
    policy: ObjectivePromptPolicy,
    last_secondary_turn: Option<u32>,
    primary_lock_active: bool,
    transition_success: bool,
}

impl ObjectivePromptPolicy {
    pub fn new() -> Self {
        Self {
            primary_when_objective_active: InstructionPriority::Primary,
            secondary_when_objective_inactive: InstructionPriority::Secondary,
            tertiary_when_hints_disabled: InstructionPriority::Tertiary,
        }
    }

    fn resolve(
        &self,
        objective_is_active: bool,
        ambient_hints_enabled: bool,
    ) -> InstructionPriority {
        if objective_is_active {
            return self.primary_when_objective_active;
        }

        if ambient_hints_enabled {
            return self.secondary_when_objective_inactive;
        }

        self.tertiary_when_hints_disabled
    }
}

impl Default for ObjectivePromptPolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl ObjectivePromptEngine {
    pub fn new(policy: ObjectivePromptPolicy) -> Self {
        Self {
            policy,
            last_secondary_turn: None,
            primary_lock_active: false,
            transition_success: false,
        }
    }

    pub fn next(
        &mut self,
        objective_is_active: bool,
        ambient_hints_enabled: bool,
        turn: u32,
    ) -> InstructionDecision {
        if objective_is_active {
            self.primary_lock_active = true;
        }

        let objective_should_persist = objective_is_active
            || (self.primary_lock_active && !self.transition_success);
        let priority = self
            .policy
            .resolve(objective_should_persist, ambient_hints_enabled);

        match priority {
            InstructionPriority::Primary => InstructionDecision {
                priority,
                kind: InstructionEvent::PrimaryShown,
            },
            InstructionPriority::Secondary => {
                let duplicate = self.last_secondary_turn == Some(turn);
                if duplicate {
                    return InstructionDecision {
                        priority,
                        kind: InstructionEvent::SuppressedDuplicate,
                    };
                }

                self.last_secondary_turn = Some(turn);
                InstructionDecision {
                    priority,
                    kind: InstructionEvent::SecondaryShown,
                }
            }
            InstructionPriority::Tertiary => InstructionDecision {
                priority,
                kind: InstructionEvent::TertiaryShown,
            },
        }
    }

    pub fn mark_transition_success(&mut self) {
        self.transition_success = true;
        self.primary_lock_active = false;
    }
}

pub fn select_primary_instruction(
    policy: &ObjectivePromptPolicy,
    objective_is_active: bool,
    ambient_hints_enabled: bool,
) -> InstructionPriority {
    policy.resolve(objective_is_active, ambient_hints_enabled)
}

pub type InstructionPriorityPolicy = ObjectivePromptPolicy;

pub const COLONY_OBJECTIVE_PROMPT_TEXT: &str =
    "Primary action: reach the shelter gate and press Enter.";

#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ColonyObjectivePromptState {
    pub has_reached_overworld: bool,
    pub visible_in_colony: bool,
}

pub fn refresh_colony_objective_prompt(
    current_state: Res<State<AppState>>,
    mut prompt: ResMut<ColonyObjectivePromptState>,
) {
    match current_state.get() {
        AppState::Overworld => {
            prompt.has_reached_overworld = true;
            prompt.visible_in_colony = false;
        }
        AppState::Colony => {
            prompt.visible_in_colony = !prompt.has_reached_overworld;
        }
        _ => {
            prompt.visible_in_colony = false;
        }
    }
}
