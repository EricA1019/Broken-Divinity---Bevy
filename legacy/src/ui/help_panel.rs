//! Toggleable help overlay — press ? or F1 to show controls.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::core::state::AppState;
use crate::game::colony::raids::ActiveRaid;
use crate::ui::modal_priority::{ModalPriorityCoordinator, can_open_help_panel};
use crate::ui::objective_prompt::{
    COLONY_OBJECTIVE_PROMPT_TEXT, ColonyObjectivePromptState, InstructionPriorityPolicy,
};

/// Whether the help panel is currently open.
#[derive(Resource, Default)]
pub struct HelpOpen(pub bool);

/// Toggle help visibility when ? or F1 is pressed.
pub fn toggle_help(
    keys: Res<ButtonInput<KeyCode>>,
    mut open: ResMut<HelpOpen>,
    active_raid: Option<Res<ActiveRaid>>,
    coordinator: Option<Res<ModalPriorityCoordinator>>,
) {
    let requested_toggle = (keys.just_pressed(KeyCode::Slash)
        && (keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight)))
        || keys.just_pressed(KeyCode::F1);
    if !requested_toggle {
        return;
    }

    if open.0 {
        open.0 = false;
        return;
    }

    let coordinator = coordinator
        .as_deref()
        .cloned()
        .unwrap_or_default();
    if can_open_help_panel(active_raid.as_deref(), &coordinator) {
        open.0 = true;
    }
}

/// Draw the help window when open.
pub fn draw_help_panel(
    mut contexts: EguiContexts,
    state: Res<State<AppState>>,
    mut open: ResMut<HelpOpen>,
    objective_prompt: Option<Res<ColonyObjectivePromptState>>,
    instruction_priority_policy: Option<Res<InstructionPriorityPolicy>>,
) {
    if !open.0 {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::Window::new("Help")
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .collapsible(false)
        .resizable(false)
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(egui::Color32::from_rgba_unmultiplied(30, 30, 35, 240))
                .inner_margin(egui::Margin::same(16)),
        )
        .show(ctx, |ui| {
            ui.heading("Controls");
            ui.add_space(8.0);

            // Objective hint
            ui.label(
                egui::RichText::new("Explore dungeons, manage your shelter, and survive.")
                    .italics()
                    .color(egui::Color32::from_rgb(180, 180, 180)),
            );
            ui.add_space(12.0);

            // Context-sensitive controls based on AppState
            match *state.get() {
                AppState::Dungeon => {
                    ui.label(egui::RichText::new("Dungeon Controls").strong());
                    ui.add_space(4.0);
                    ui.label("WASD/Arrows: Move");
                    ui.label("Shift+Dir: Sprint");
                    ui.label("Bump: Melee Attack");
                    ui.label("R: Reload");
                    ui.label("I/Tab: Inventory");
                    ui.label("F: Shoot");
                    ui.label("1-9: Use Item");
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("Current Combat Lanes").strong());
                    ui.add_space(4.0);
                    ui.label("MLY: Thumos + Melee Training + modifiers");
                    ui.label("RNG: Prudence + Ranged Training + modifiers");
                    ui.label("DEF: Metis + Quiet Movement (enemy attack DV)");
                }
                AppState::Colony => {
                    let instruction_priority_policy = instruction_priority_policy
                        .as_deref()
                        .cloned()
                        .unwrap_or_default();
                    let show_secondary_hints = colony_help_shows_secondary_hints(
                        objective_prompt.as_deref(),
                        &instruction_priority_policy,
                    );

                    ui.label(egui::RichText::new("Colony Controls").strong());
                    ui.add_space(4.0);
                    if show_secondary_hints {
                        ui.label("Manage survivors and stations.");
                        ui.label("Build stations.");
                        ui.label("Assign workers.");
                    }
                    if objective_prompt.is_some_and(|prompt| prompt.visible_in_colony) {
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new(COLONY_OBJECTIVE_PROMPT_TEXT)
                                .italics()
                                .color(egui::Color32::from_rgb(200, 185, 120)),
                        );
                    }
                }
                AppState::Overworld => {
                    ui.label(egui::RichText::new("Overworld Controls").strong());
                    ui.add_space(4.0);
                    ui.label("Travel: click a connected node.");
                    ui.label("Esc: Return to shelter.");
                }
                AppState::Menu | AppState::Combat | AppState::GameOver => {
                    ui.label(egui::RichText::new("General Controls").strong());
                    ui.add_space(4.0);
                    ui.label("Follow on-screen prompts");
                }
            }

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(8.0);

            // Universal controls
            ui.label(egui::RichText::new("Universal").strong());
            ui.add_space(4.0);
            ui.label("F1/?: Toggle Help");
            ui.label("Esc: Back/Menu");

            ui.add_space(12.0);

            // Close button
            if ui.button("Close (Esc)").clicked() {
                open.0 = false;
            }

            // Also allow Esc to close
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                open.0 = false;
            }
        });
}

pub(crate) fn colony_help_shows_secondary_hints(
    objective_prompt: Option<&ColonyObjectivePromptState>,
    instruction_priority_policy: &InstructionPriorityPolicy,
) -> bool {
    if instruction_priority_policy.suppress_secondary_hints_when_primary_active
        && objective_prompt.is_some_and(|prompt| prompt.visible_in_colony)
    {
        return false;
    }

    true
}
