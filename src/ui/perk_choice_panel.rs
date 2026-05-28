use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::core::gamelog::{GameLog, LogColor};
use crate::core::perks::{PendingPerkChoices, PerkId, PlayerPerks};
use crate::core::turn::GameTime;

// ---------------------------------------------------------------------------
// Action resource
// ---------------------------------------------------------------------------

#[derive(Resource, Default)]
pub struct PerkChoiceUiAction(pub Option<PerkChoiceUiChoice>);

#[derive(Clone, Copy, Debug)]
pub enum PerkChoiceUiChoice {
    Unlock(PerkId),
}

// ---------------------------------------------------------------------------
// Draw — EguiPrimaryContextPass (read-only)
// ---------------------------------------------------------------------------

pub fn draw_perk_choice_panel(
    mut contexts: EguiContexts,
    pending: Res<PendingPerkChoices>,
    mut action: ResMut<PerkChoiceUiAction>,
) {
    if !pending.has_pending() {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let Some(perk) = pending.pending.first().copied() else {
        return;
    };

    egui::Window::new("Perk Unlocked")
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label(
                egui::RichText::new(perk.name())
                    .strong()
                    .size(18.0)
                    .color(egui::Color32::from_rgb(230, 210, 120)),
            );
            ui.label(
                egui::RichText::new(format!("Tier {}", perk.tier()))
                    .color(egui::Color32::LIGHT_GRAY),
            );
            let lane_color = if perk.lane_label().contains("Legacy") {
                egui::Color32::from_rgb(220, 140, 90)
            } else {
                egui::Color32::from_rgb(150, 190, 210)
            };
            ui.label(egui::RichText::new(perk.lane_label()).color(lane_color));
            ui.add_space(6.0);
            ui.label(perk.description());
            ui.add_space(10.0);

            if ui.button(format!("Unlock {}", perk.name())).clicked() {
                action.0 = Some(PerkChoiceUiChoice::Unlock(perk));
            }
        });
}

// ---------------------------------------------------------------------------
// Process — Update (mutations)
// ---------------------------------------------------------------------------

pub fn process_perk_choice_action(
    mut action: ResMut<PerkChoiceUiAction>,
    mut pending: ResMut<PendingPerkChoices>,
    mut player_q: Query<&mut PlayerPerks>,
    mut log: ResMut<GameLog>,
    time: Res<GameTime>,
) {
    let Some(choice) = action.0.take() else {
        return;
    };
    let Ok(mut player_perks) = player_q.single_mut() else {
        return;
    };

    match choice {
        PerkChoiceUiChoice::Unlock(perk) => {
            player_perks.unlock(perk);
            pending.pop_next();
            log.push(
                format!("Unlocked perk: {} [{}].", perk.name(), perk.lane_label()),
                LogColor::Status,
                time.turn,
            );
        }
    }
}
