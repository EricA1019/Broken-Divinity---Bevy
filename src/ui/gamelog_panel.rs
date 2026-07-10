//! Game log egui panel — shows the last N combat messages at the bottom of the screen.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::core::gamelog::GameLog;
use crate::core::state::AppState;

/// Draw the game log panel at the bottom of the screen.
pub fn draw_gamelog_panel(
    mut contexts: EguiContexts,
    log: Res<GameLog>,
    state: Res<State<AppState>>,
) {
    if *state.get() != AppState::Dungeon {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::TopBottomPanel::bottom("game_log").show(ctx, |ui| {
        ui.set_min_height(100.0);
        ui.label(egui::RichText::new("Game Log").strong());
        ui.separator();

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for entry in log.last_n(20) {
                    let color = gamelog_color(&entry.color);
                    if entry.count > 1 {
                        ui.label(
                            egui::RichText::new(format!("{} x{}", entry.text, entry.count))
                                .color(color)
                                .size(13.0),
                        );
                    } else {
                        ui.label(egui::RichText::new(&entry.text).color(color).size(13.0));
                    }
                }
            });
    });
}

fn gamelog_color(color: &crate::core::gamelog::LogColor) -> egui::Color32 {
    use crate::core::gamelog::LogColor;
    match color {
        LogColor::Default => egui::Color32::WHITE,
        LogColor::PlayerHit => egui::Color32::from_rgb(50, 200, 50),
        LogColor::EnemyHit => egui::Color32::from_rgb(230, 75, 75),
        LogColor::Critical => egui::Color32::from_rgb(255, 215, 0),
        LogColor::Miss => egui::Color32::from_rgb(150, 150, 150),
        LogColor::Death => egui::Color32::from_rgb(200, 25, 25),
        LogColor::Status => egui::Color32::from_rgb(150, 100, 200),
        LogColor::System => egui::Color32::from_rgb(130, 180, 255),
    }
}
