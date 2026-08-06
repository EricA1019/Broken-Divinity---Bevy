//! Emergency escape hatches for testers — Esc to menu, death detection, stuck hints.

use bevy::prelude::*;

use super::components::Player;
use super::gamelog::{GameLog, LogColor, UxMessage};
use super::state::AppState;
use super::stats::CombatStats;
use super::turn::GameTime;
use crate::ui::help_panel::HelpOpen;
use crate::ui::modal_priority::ModalBlockers;

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Tracks the last turn the player took an action (move/attack/wait).
#[derive(Resource, Default)]
pub struct LastActionTurn(pub u32);

#[derive(Default)]
pub struct EscReinforcementState {
    overworld_back_emitted: bool,
    help_close_emitted: bool,
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Universal Esc handler: press Escape to return to main menu from any state.
/// Does NOT fire when the help panel is open (Esc closes help first).
pub fn handle_escape_to_menu(
    keys: Res<ButtonInput<KeyCode>>,
    mut help_open: ResMut<HelpOpen>,
    blockers: Option<Res<ModalBlockers>>,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut log: Option<ResMut<GameLog>>,
    game_time: Option<Res<GameTime>>,
    mut reinforcement: Local<EscReinforcementState>,
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }
    let turn = game_time.as_ref().map_or(0, |time| time.turn);

    // Close topmost layer first before allowing global AppState transitions.
    if help_open.0 {
        help_open.0 = false;
        if !reinforcement.help_close_emitted {
            if let Some(ref mut log) = log {
                log.push_ux_message(UxMessage::EscHelpCloseHint, turn);
            }
            reinforcement.help_close_emitted = true;
        }
        return;
    }

    // Critical blocking modal keeps the player in-context.
    if blockers.is_some_and(|blockers| blockers.critical_modal_active) {
        return;
    }

    // Don't trigger if already in menu.
    if *state.get() == AppState::Menu {
        return;
    }

    // Overworld Esc is a local back action to shelter; all other gameplay states return to menu.
    let target_state = match *state.get() {
        AppState::Overworld => {
            if !reinforcement.overworld_back_emitted {
                if let Some(ref mut log) = log {
                    log.push_ux_message(UxMessage::EscOverworldBackHint, turn);
                }
                reinforcement.overworld_back_emitted = true;
            }
            AppState::Colony
        }
        _ => AppState::Menu,
    };
    next_state.set(target_state);
}

pub fn queue_game_over(
    commands: &mut Commands,
    next_state: &mut NextState<AppState>,
    log: &mut GameLog,
    turn: u32,
) {
    log.push("You have been slain...", LogColor::Death, turn);
    commands.insert_resource(crate::ui::gameover::DeathSummary {
        turns_survived: turn,
    });
    next_state.set(AppState::GameOver);
}

/// Check if the player is dead and transition to GameOver.
/// Runs in all non-menu states to catch death anywhere.
pub fn check_player_death_universal(
    player_q: Query<&CombatStats, With<Player>>,
    state: Res<State<AppState>>,
    death_summary: Option<Res<crate::ui::gameover::DeathSummary>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut log: ResMut<GameLog>,
    game_time: Res<GameTime>,
    mut commands: Commands,
) {
    // Only check during gameplay states, not Menu or GameOver
    match *state.get() {
        AppState::Menu | AppState::GameOver => return,
        _ => {}
    }

    if death_summary.is_some() {
        return;
    }

    let Ok(stats) = player_q.single() else { return };

    if stats.is_dead() {
        queue_game_over(&mut commands, next_state.as_mut(), &mut log, game_time.turn);
    }
}

/// Detect if the player hasn't moved or acted for 20 turns in Dungeon state.
/// Logs a hint: "Press ? for help or Esc to return to menu."
pub fn detect_stuck_player(
    game_time: Res<GameTime>,
    last_action: Res<LastActionTurn>,
    mut log: ResMut<GameLog>,
) {
    let turns_since_action = game_time.turn.saturating_sub(last_action.0);

    // Log hint every 20 turns of inactivity
    if turns_since_action > 0 && turns_since_action.is_multiple_of(20) {
        log.push(
            "Press ? for help or Esc to return to menu.",
            LogColor::System,
            game_time.turn,
        );
    }
}

/// Update the last action turn when the player turn phase completes.
/// This system should run in PlayerTurn phase, after the player has acted.
pub fn update_last_action_turn(game_time: Res<GameTime>, mut last_action: ResMut<LastActionTurn>) {
    last_action.0 = game_time.turn;
}
