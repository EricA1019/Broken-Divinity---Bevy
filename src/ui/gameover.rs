//! Game Over screen — shown when the player dies.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::core::state::AppState;

/// Snapshot of the run inserted by `check_player_death` before transitioning to GameOver.
#[derive(Resource, Debug, Clone, Default)]
pub struct DeathSummary {
    pub turns_survived: u32,
}

// ---------------------------------------------------------------------------
// Action resource
// ---------------------------------------------------------------------------

#[derive(Resource, Default)]
pub struct GameOverUiAction(pub Option<GameOverUiChoice>);

#[derive(Clone, Copy, Debug)]
pub enum GameOverUiChoice {
    NewGame,
    ReturnToMenu,
}

// ---------------------------------------------------------------------------
// Draw — EguiPrimaryContextPass (read-only)
// ---------------------------------------------------------------------------

/// Draw the Game Over screen with death summary and "New Game" button.
pub fn draw_gameover_screen(
    mut contexts: EguiContexts,
    state: Res<State<AppState>>,
    summary: Option<Res<DeathSummary>>,
    mut action: ResMut<GameOverUiAction>,
) {
    if *state.get() != AppState::GameOver {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else { return };

    let turns = summary.as_ref().map_or(0, |s| s.turns_survived);

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(egui::Color32::from_rgb(10, 10, 15)))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(160.0);
                ui.label(
                    egui::RichText::new("YOU DIED")
                        .size(48.0)
                        .color(egui::Color32::from_rgb(200, 25, 25))
                        .strong(),
                );
                ui.add_space(20.0);
                ui.label(
                    egui::RichText::new("The wasteland claims another soul.")
                        .size(16.0)
                        .color(egui::Color32::from_rgb(150, 150, 150)),
                );
                ui.add_space(15.0);
                ui.label(
                    egui::RichText::new(format!("You survived {turns} turns"))
                        .size(18.0)
                        .color(egui::Color32::from_rgb(200, 160, 80)),
                );
                ui.add_space(40.0);

                if ui
                    .add(
                        egui::Button::new(egui::RichText::new("New Game").size(20.0))
                            .min_size(egui::vec2(200.0, 40.0)),
                    )
                    .clicked()
                {
                    action.0 = Some(GameOverUiChoice::NewGame);
                }

                ui.add_space(10.0);

                if ui
                    .add(
                        egui::Button::new(egui::RichText::new("Return to Menu").size(20.0))
                            .min_size(egui::vec2(200.0, 40.0)),
                    )
                    .clicked()
                {
                    action.0 = Some(GameOverUiChoice::ReturnToMenu);
                }
            });
        });
}

// ---------------------------------------------------------------------------
// Process — Update (mutations)
// ---------------------------------------------------------------------------

pub fn process_gameover_action(
    mut action: ResMut<GameOverUiAction>,
    mut next_state: ResMut<NextState<AppState>>,
    mut commands: Commands,
) {
    let Some(choice) = action.0.take() else {
        return;
    };

    match choice {
        GameOverUiChoice::NewGame => {
            // Permadeath: delete save and clear game state before returning to menu.
            crate::core::save::delete_save();
            commands.remove_resource::<DeathSummary>();
            next_state.set(AppState::Menu);
        }
        GameOverUiChoice::ReturnToMenu => {
            // Just return to menu without deleting save (allows loading previous save).
            commands.remove_resource::<DeathSummary>();
            next_state.set(AppState::Menu);
        }
    }
}

/// System: check if the player is dead and transition to GameOver.
pub fn check_player_death(
    player_q: Query<&crate::core::stats::CombatStats, With<crate::core::components::Player>>,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut log: ResMut<crate::core::gamelog::GameLog>,
    game_time: Res<crate::core::turn::GameTime>,
    mut commands: Commands,
) {
    if *state.get() != AppState::Dungeon {
        return;
    }

    let Ok(stats) = player_q.single() else { return };

    if stats.is_dead() {
        log.push(
            "You have been slain...",
            crate::core::gamelog::LogColor::Death,
            game_time.turn,
        );
        commands.insert_resource(DeathSummary {
            turns_survived: game_time.turn,
        });
        next_state.set(AppState::GameOver);
    }
}
