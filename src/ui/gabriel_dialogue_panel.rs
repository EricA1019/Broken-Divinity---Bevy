use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::core::gamelog::{GameLog, LogColor};
use crate::core::state::AppState;
use crate::core::turn::GameTime;
use crate::game::dungeon::gabriel::{
    Gabriel, GabrielCompanion, GabrielDialogueState, GabrielDialogueStep, GabrielEncounter,
    GabrielState,
};

#[derive(Resource, Default)]
pub struct GabrielDialogueUiAction(pub Option<GabrielDialogueUiChoice>);

#[derive(Clone, Copy, Debug)]
pub enum GabrielDialogueUiChoice {
    AskIdentity,
    AskThreat,
    AskAid,
    Accept,
}

pub fn draw_gabriel_dialogue_panel(
    mut contexts: EguiContexts,
    app_state: Res<State<AppState>>,
    dialogue: Res<GabrielDialogueState>,
    mut action: ResMut<GabrielDialogueUiAction>,
) {
    if *app_state.get() != AppState::Dungeon {
        return;
    }

    let Some(step) = dialogue.0 else {
        return;
    };
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::Window::new("Gabriel")
        .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -116.0))
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .fixed_size(egui::vec2(580.0, 190.0))
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(egui::Color32::from_rgba_unmultiplied(20, 24, 36, 245))
                .stroke(egui::Stroke::new(
                    1.0,
                    egui::Color32::from_rgb(205, 215, 255),
                )),
        )
        .show(ctx, |ui| {
            ui.label(
                egui::RichText::new("Gabriel")
                    .strong()
                    .color(egui::Color32::from_rgb(225, 232, 255))
                    .size(19.0),
            );
            ui.separator();
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(dialogue_text(step))
                    .color(egui::Color32::from_rgb(230, 230, 235))
                    .size(15.0),
            );
            ui.add_space(10.0);

            match step {
                GabrielDialogueStep::Warning => {
                    if ui.button("1. Who are you?").clicked() {
                        action.0 = Some(GabrielDialogueUiChoice::AskIdentity);
                    }
                    if ui.button("2. What is Michael's Host?").clicked() {
                        action.0 = Some(GabrielDialogueUiChoice::AskThreat);
                    }
                    if ui.button("3. Then help me finish this.").clicked() {
                        action.0 = Some(GabrielDialogueUiChoice::AskAid);
                    }
                }
                GabrielDialogueStep::Identity
                | GabrielDialogueStep::Threat
                | GabrielDialogueStep::Aid => {
                    if ui.button(accept_label(step)).clicked() {
                        action.0 = Some(GabrielDialogueUiChoice::Accept);
                    }
                }
            }
        });
}

pub fn process_gabriel_dialogue_action(
    mut commands: Commands,
    mut action: ResMut<GabrielDialogueUiAction>,
    mut dialogue: ResMut<GabrielDialogueState>,
    mut gabriel_state: ResMut<GabrielState>,
    mut gabriel_q: Query<&mut GabrielCompanion, With<Gabriel>>,
    mut log: ResMut<GameLog>,
    time: Res<GameTime>,
) {
    let Some(choice) = action.0.take() else {
        return;
    };

    match choice {
        GabrielDialogueUiChoice::AskIdentity => {
            dialogue.0 = Some(GabrielDialogueStep::Identity);
        }
        GabrielDialogueUiChoice::AskThreat => {
            dialogue.0 = Some(GabrielDialogueStep::Threat);
        }
        GabrielDialogueUiChoice::AskAid => {
            dialogue.0 = Some(GabrielDialogueStep::Aid);
        }
        GabrielDialogueUiChoice::Accept => {
            gabriel_state.encounter_completed = true;
            gabriel_state.joined = true;
            dialogue.close();
            commands.remove_resource::<GabrielEncounter>();

            if let Ok(mut companion) = gabriel_q.single_mut() {
                companion.active = true;
            } else {
                warn!("Gabriel entity not found during dialogue accept — skipping activation");
            }

            log.push(
                "Gabriel falls into step beside you.",
                LogColor::Status,
                time.turn,
            );
        }
    }
}

fn dialogue_text(step: GabrielDialogueStep) -> &'static str {
    match step {
        GabrielDialogueStep::Warning => {
            "At last, an Adam who still chooses their path. Michael's Host wakes beneath these ruins, and if you go deeper blind, you will deliver them a martyr."
        }
        GabrielDialogueStep::Identity => {
            "Gabriel. Once herald, now witness. I remember enough of the old order to know this place was never meant to sing with war."
        }
        GabrielDialogueStep::Threat => {
            "Michael's Host calls ruin a crusade. They gather relics, soldiers, and hungry certainties. Every floor below feeds that choir."
        }
        GabrielDialogueStep::Aid => {
            "Then do not descend alone. I can pass where flesh clogs the corridor, and I would rather blunt Michael's sermon here than bury another city for it."
        }
    }
}

fn accept_label(step: GabrielDialogueStep) -> &'static str {
    match step {
        GabrielDialogueStep::Identity => "1. Walk with me, then.",
        GabrielDialogueStep::Threat => "1. Then we stop them together.",
        GabrielDialogueStep::Aid => "1. I can use that kind of warning.",
        GabrielDialogueStep::Warning => "1. Continue",
    }
}
