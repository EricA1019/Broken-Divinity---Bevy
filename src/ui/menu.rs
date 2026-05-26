//! Main menu screen.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::core::gamelog::{GameLog, LogColor};
use crate::core::resources::WorldSeed;
use crate::core::state::AppState;
use crate::core::turn::GameTime;
use crate::ui::readability::contrast_ratio;

const MENU_BACKGROUND_RGB: (u8, u8, u8) = (10, 10, 15);
const MENU_TITLE_RGB: (u8, u8, u8) = (208, 174, 96);
const MENU_SUBTITLE_RGB: (u8, u8, u8) = (188, 188, 188);
const MENU_SEED_LABEL_RGB: (u8, u8, u8) = (226, 226, 226);
const MENU_SUBTITLE_FONT_SIZE: f32 = 14.0;
const MENU_TITLE_FONT_SIZE: f32 = 40.0;
const MENU_MIN_CONTRAST_RATIO: f32 = 4.5;
const MENU_ROOT_TOP_SPACING: f32 = 80.0;
const MENU_TITLE_SUBTITLE_SPACING: f32 = 10.0;
const MENU_SECTION_SPACING: f32 = 40.0;
const MENU_SEED_ROW_BOTTOM_SPACING: f32 = 20.0;
const MENU_NEW_GAME_BUTTON_WIDTH: f32 = 200.0;
const MENU_NEW_GAME_BUTTON_HEIGHT: f32 = 40.0;
const MENU_LOAD_GAME_BUTTON_HEIGHT: f32 = 36.0;
const MENU_QUIT_BUTTON_HEIGHT: f32 = 30.0;
const MENU_QUIT_SECTION_SPACING: f32 = 10.0;
const MENU_QUIT_CONFIRM_ROW_SPACING: f32 = 8.0;
const MENU_QUIT_CONFIRM_BUTTON_WIDTH: f32 = 96.0;
const MENU_QUIT_CONFIRM_BUTTON_HEIGHT: f32 = 28.0;
const MENU_SEED_HASH_MULTIPLIER: u64 = 31;
const MENU_HELPER_FONT_SIZE: f32 = 12.0;
const MENU_DEFAULT_SEED_PREVIEW: &str = "Leave blank for a random world seed.";
const MENU_SEEDED_PREVIEW_PREFIX: &str = "World seed:";
const MENU_NO_SAVE_HELPER_TEXT: &str = "No save found yet. Start a New Game to create one.";
const MENU_NO_HELPER_TEXT: &str = "";
const MENU_QUIT_CONFIRM_PROMPT_TEXT: &str = "Quit the game?";
const MENU_QUIT_CONFIRM_LABEL: &str = "Confirm";
const MENU_QUIT_CANCEL_LABEL: &str = "Cancel";

pub(crate) fn primary_menu_cta_label() -> &'static str {
    "New Game"
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MenuLoadAffordance {
    pub is_enabled: bool,
    pub helper_text: &'static str,
}

pub(crate) fn load_affordance_for_save_state(has_save: bool) -> MenuLoadAffordance {
    if has_save {
        return MenuLoadAffordance {
            is_enabled: true,
            helper_text: MENU_NO_HELPER_TEXT,
        };
    }

    MenuLoadAffordance {
        is_enabled: false,
        helper_text: MENU_NO_SAVE_HELPER_TEXT,
    }
}

pub(crate) fn seed_helper_text(resolved_seed: Option<u64>) -> String {
    match resolved_seed {
        Some(seed) => format!("{MENU_SEEDED_PREVIEW_PREFIX} {seed}"),
        None => MENU_DEFAULT_SEED_PREVIEW.to_string(),
    }
}

fn current_unix_seed() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub(crate) fn resolve_menu_seed(seed_text: &str, unix_seed_fallback: u64) -> u64 {
    if seed_text.is_empty() {
        return unix_seed_fallback;
    }

    if let Ok(parsed_seed) = seed_text.parse::<u64>() {
        return parsed_seed;
    }

    seed_text
        .bytes()
        .enumerate()
        .fold(0u64, |hash, (index, byte)| {
            hash.wrapping_add(
                (byte as u64)
                    .wrapping_mul(MENU_SEED_HASH_MULTIPLIER.wrapping_pow(index as u32)),
            )
        })
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct MenuReadabilitySnapshot {
    pub minimum_contrast_ratio: f32,
    pub subtitle_contrast_ratio: f32,
    pub seed_row_contrast_ratio: f32,
}

pub(crate) fn menu_readability_snapshot() -> MenuReadabilitySnapshot {
    MenuReadabilitySnapshot {
        minimum_contrast_ratio: MENU_MIN_CONTRAST_RATIO,
        subtitle_contrast_ratio: contrast_ratio(MENU_SUBTITLE_RGB, MENU_BACKGROUND_RGB),
        seed_row_contrast_ratio: contrast_ratio(MENU_SEED_LABEL_RGB, MENU_BACKGROUND_RGB),
    }
}

// ---------------------------------------------------------------------------
// Action resource
// ---------------------------------------------------------------------------

#[derive(Resource, Default)]
pub struct MenuUiAction(pub Option<MenuUiChoice>);

#[derive(Clone, Debug)]
pub enum MenuUiChoice {
    NewGame { seed: u64 },
    LoadGame,
    Quit,
    ConfirmQuit,
    CancelQuit,
}

// ---------------------------------------------------------------------------
// Draw — EguiPrimaryContextPass (read-only)
// ---------------------------------------------------------------------------

/// System: draw main menu.
pub fn draw_main_menu(
    mut contexts: EguiContexts,
    state: Res<State<AppState>>,
    mut seed_text: Local<String>,
    mut quit_confirmation_pending: Local<bool>,
    mut action: ResMut<MenuUiAction>,
) {
    if *state.get() != AppState::Menu {
        return;
    }
    let readability = menu_readability_snapshot();
    debug_assert!(
        readability.subtitle_contrast_ratio >= readability.minimum_contrast_ratio
            && readability.seed_row_contrast_ratio >= readability.minimum_contrast_ratio,
        "Menu readability baseline violated"
    );
    let Ok(ctx) = contexts.ctx_mut() else { return };

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(rgb(MENU_BACKGROUND_RGB)))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(MENU_ROOT_TOP_SPACING);
                ui.heading(
                    egui::RichText::new("BROKEN DIVINITY")
                        .size(MENU_TITLE_FONT_SIZE)
                        .color(rgb(MENU_TITLE_RGB))
                        .strong(),
                );
                ui.add_space(MENU_TITLE_SUBTITLE_SPACING);
                ui.label(
                    egui::RichText::new("A post-apocalyptic roguelike")
                        .size(MENU_SUBTITLE_FONT_SIZE)
                        .color(rgb(MENU_SUBTITLE_RGB)),
                );
                ui.add_space(MENU_SECTION_SPACING);

                // Seed input
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Seed:").color(rgb(MENU_SEED_LABEL_RGB)));
                    ui.text_edit_singleline(&mut *seed_text);
                });
                let resolved_seed = (!seed_text.is_empty())
                    .then(|| resolve_menu_seed(seed_text.as_str(), 0));
                ui.label(
                    egui::RichText::new(seed_helper_text(resolved_seed))
                        .size(MENU_HELPER_FONT_SIZE),
                );
                ui.add_space(MENU_SEED_ROW_BOTTOM_SPACING);

                let load_affordance =
                    load_affordance_for_save_state(crate::core::save::save_exists());

                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new(primary_menu_cta_label())
                                .size(20.0)
                                .strong(),
                        )
                            .min_size(egui::vec2(
                                MENU_NEW_GAME_BUTTON_WIDTH,
                                MENU_NEW_GAME_BUTTON_HEIGHT,
                            )),
                    )
                    .clicked()
                {
                    let seed = resolve_menu_seed(seed_text.as_str(), current_unix_seed());
                    action.0 = Some(MenuUiChoice::NewGame { seed });
                }

                if ui
                    .add_enabled(
                        load_affordance.is_enabled,
                        egui::Button::new(egui::RichText::new("Load Game").size(18.0)).min_size(
                            egui::vec2(MENU_NEW_GAME_BUTTON_WIDTH, MENU_LOAD_GAME_BUTTON_HEIGHT),
                        ),
                    )
                    .clicked()
                {
                    action.0 = Some(MenuUiChoice::LoadGame);
                }

                if !load_affordance.helper_text.is_empty() {
                    ui.label(
                        egui::RichText::new(load_affordance.helper_text)
                            .size(MENU_HELPER_FONT_SIZE),
                    );
                }

                ui.add_space(MENU_QUIT_SECTION_SPACING);

                if !*quit_confirmation_pending {
                    if ui
                        .add(
                            egui::Button::new(egui::RichText::new("Quit").size(16.0))
                                .min_size(egui::vec2(
                                    MENU_NEW_GAME_BUTTON_WIDTH,
                                    MENU_QUIT_BUTTON_HEIGHT,
                                )),
                        )
                        .clicked()
                    {
                        *quit_confirmation_pending = true;
                    }
                } else {
                    ui.label(egui::RichText::new(MENU_QUIT_CONFIRM_PROMPT_TEXT).strong());
                    ui.add_space(MENU_QUIT_CONFIRM_ROW_SPACING);
                    ui.horizontal(|ui| {
                        if ui
                            .add(
                                egui::Button::new(MENU_QUIT_CONFIRM_LABEL).min_size(egui::vec2(
                                    MENU_QUIT_CONFIRM_BUTTON_WIDTH,
                                    MENU_QUIT_CONFIRM_BUTTON_HEIGHT,
                                )),
                            )
                            .clicked()
                        {
                            action.0 = Some(MenuUiChoice::ConfirmQuit);
                            *quit_confirmation_pending = false;
                        }

                        if ui
                            .add(
                                egui::Button::new(MENU_QUIT_CANCEL_LABEL).min_size(egui::vec2(
                                    MENU_QUIT_CONFIRM_BUTTON_WIDTH,
                                    MENU_QUIT_CONFIRM_BUTTON_HEIGHT,
                                )),
                            )
                            .clicked()
                        {
                            action.0 = Some(MenuUiChoice::CancelQuit);
                            *quit_confirmation_pending = false;
                        }
                    });
                }
            });
        });
}

fn rgb((red, green, blue): (u8, u8, u8)) -> egui::Color32 {
    egui::Color32::from_rgb(red, green, blue)
}

// ---------------------------------------------------------------------------
// Process — Update (mutations)
// ---------------------------------------------------------------------------

pub fn process_menu_action(
    mut action: ResMut<MenuUiAction>,
    mut next_state: ResMut<NextState<AppState>>,
    mut log: ResMut<GameLog>,
    game_time: Option<Res<GameTime>>,
    mut commands: Commands,
    mut exit: MessageWriter<AppExit>,
) {
    let Some(choice) = action.0.take() else {
        return;
    };
    let turn = game_time.as_ref().map_or(0, |time| time.turn);

    match choice {
        MenuUiChoice::NewGame { seed } => {
            commands.insert_resource(WorldSeed(seed));
            log.push(
                format!("New game started with seed {seed}."),
                LogColor::System,
                turn,
            );
            next_state.set(AppState::Colony);
        }
        MenuUiChoice::LoadGame => {
            match crate::core::save::load_game_detailed() {
                Ok(save) => {
                    commands.insert_resource(WorldSeed(save.seed));
                    commands.insert_resource(crate::core::save::PlayerSnapshot(Some(
                        save.player.clone(),
                    )));
                    crate::core::save::queue_loaded_game(&mut commands, save.clone());
                    log.push(
                        crate::core::save::load_success_message(),
                        LogColor::System,
                        turn,
                    );
                    next_state.set(save.app_state.into_runtime_state());
                }
                Err(error) => {
                    log.push(
                        crate::core::save::load_error_message(error),
                        LogColor::Status,
                        turn,
                    );
                }
            }
        }
        MenuUiChoice::Quit | MenuUiChoice::ConfirmQuit => {
            exit.write(AppExit::Success);
        }
        MenuUiChoice::CancelQuit => {}
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_menu_seed;

    const TEST_FALLBACK_SEED: u64 = 777;

    #[test]
    fn resolve_menu_seed_uses_numeric_value() {
        assert_eq!(resolve_menu_seed("12345", TEST_FALLBACK_SEED), 12345);
    }

    #[test]
    fn resolve_menu_seed_hashes_text_deterministically() {
        let first = resolve_menu_seed("abc", TEST_FALLBACK_SEED);
        let second = resolve_menu_seed("abc", TEST_FALLBACK_SEED);
        assert_eq!(first, second);
        assert_ne!(first, TEST_FALLBACK_SEED);
    }

    #[test]
    fn resolve_menu_seed_uses_fallback_for_empty_seed() {
        assert_eq!(resolve_menu_seed("", TEST_FALLBACK_SEED), TEST_FALLBACK_SEED);
    }
}
