use bevy::prelude::{ButtonInput, Commands, KeyCode, NextState, Res, ResMut, State};

#[derive(Debug, Clone, Copy)]
pub struct EscapeContext {
    pub modal_open: bool,
    pub can_pause: bool,
}

use crate::core::gamelog::{GameLog, LogColor};
use crate::core::state::AppState;
use crate::core::turn::GameTime;
use crate::ui::gameover::DeathSummary;
use crate::ui::help_panel::HelpOpen;
use crate::ui::modal_priority::ModalBlockers;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EscapeAction {
    CloseModal,
    PauseGame,
    NoOp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EscapeGuidanceEvent {
    ShowHint,
    Suppressed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EscapeHintContext {
    ModalOpen,
    PauseAvailable,
    None,
}

#[derive(Debug, Clone, Copy)]
pub struct EscapeGuidanceEngine {
    acknowledged: bool,
    last_context: EscapeHintContext,
}

impl EscapeGuidanceEngine {
    pub fn new() -> Self {
        Self {
            acknowledged: false,
            last_context: EscapeHintContext::None,
        }
    }

    pub fn guidance(&mut self, context: EscapeContext) -> EscapeGuidanceEvent {
        if self.acknowledged {
            return EscapeGuidanceEvent::Suppressed;
        }

        let hint_context = to_hint_context(context);
        if hint_context == EscapeHintContext::None {
            self.last_context = EscapeHintContext::None;
            return EscapeGuidanceEvent::Suppressed;
        }

        if self.last_context == hint_context {
            return EscapeGuidanceEvent::Suppressed;
        }

        self.last_context = hint_context;
        EscapeGuidanceEvent::ShowHint
    }

    pub fn acknowledge(&mut self) {
        self.acknowledged = true;
    }
}

impl Default for EscapeGuidanceEngine {
    fn default() -> Self {
        Self::new()
    }
}

fn to_hint_context(context: EscapeContext) -> EscapeHintContext {
    if context.modal_open {
        return EscapeHintContext::ModalOpen;
    }

    if context.can_pause {
        return EscapeHintContext::PauseAvailable;
    }

    EscapeHintContext::None
}

pub fn resolve_escape_action(context: EscapeContext) -> EscapeAction {
    if context.modal_open {
        return EscapeAction::CloseModal;
    }

    if context.can_pause {
        return EscapeAction::PauseGame;
    }

    EscapeAction::NoOp
}

pub fn queue_game_over(
    commands: &mut Commands,
    next_state: &mut NextState<AppState>,
    log: &mut GameLog,
    turn: u32,
) {
    log.push("You have been slain...", LogColor::Death, turn);
    commands.insert_resource(DeathSummary {
        turns_survived: turn,
    });
    next_state.set(AppState::GameOver);
}

pub fn handle_escape_to_menu(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    help_open: Option<ResMut<HelpOpen>>,
    blockers: Option<Res<ModalBlockers>>,
    log: Option<ResMut<GameLog>>,
    time: Option<Res<GameTime>>,
) {
    if !keyboard.just_pressed(KeyCode::Escape) {
        return;
    }

    let turn = time.as_ref().map_or(0, |time| time.turn);

    if let Some(mut help_open) = help_open
        && help_open.0
    {
        help_open.0 = false;
        if let Some(mut log) = log {
            log.push_ux_message(crate::core::gamelog::UxMessage::EscHelpCloseHint, turn);
        }
        return;
    }

    if blockers
        .as_ref()
        .is_some_and(|blockers| blockers.critical_modal_active)
    {
        return;
    }

    match *state.get() {
        AppState::Overworld => {
            if let Some(mut log) = log {
                log.push_ux_message(crate::core::gamelog::UxMessage::EscOverworldBackHint, turn);
            }
            next_state.set(AppState::Colony);
        }
        AppState::Colony => {
            next_state.set(AppState::Menu);
        }
        _ => {}
    }
}
