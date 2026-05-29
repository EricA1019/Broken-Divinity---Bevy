//! Lore journal panel — toggled with J key during dungeon exploration.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::game::dungeon::lore::LoreJournal;
use crate::ui::input_hints::{JOURNAL_TOGGLE_HINT_TEXT, JOURNAL_TOGGLE_KEY};

/// Resource: whether the journal panel is currently visible.
#[derive(Resource, Default)]
pub struct JournalOpen(pub bool);

/// Toggle journal on J key press.
pub fn toggle_journal(keys: Res<ButtonInput<KeyCode>>, mut open: ResMut<JournalOpen>) {
    if keys.just_pressed(JOURNAL_TOGGLE_KEY) {
        open.0 = !open.0;
    }
}

/// Draw journal window when open.
pub fn draw_journal_panel(
    mut contexts: EguiContexts,
    open: Res<JournalOpen>,
    journal: Option<Res<LoreJournal>>,
) {
    if !open.0 {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::Window::new("Lore Journal")
        .default_size([350.0, 400.0])
        .resizable(true)
        .collapsible(true)
        .show(ctx, |ui| {
            ui.label(egui::RichText::new(JOURNAL_TOGGLE_HINT_TEXT).small());
            ui.separator();

            let Some(journal) = &journal else {
                ui.label("No journal available.");
                return;
            };

            if journal.fragments.is_empty() {
                ui.label("No lore fragments collected yet.");
                ui.label("Explore dungeons to find ancient writings.");
                return;
            }

            ui.label(format!("{} fragments collected", journal.fragments.len()));
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                for fragment in &journal.fragments {
                    ui.group(|ui| {
                        ui.label(
                            egui::RichText::new(&fragment.title)
                                .strong()
                                .color(egui::Color32::from_rgb(200, 160, 80)),
                        );
                        ui.label(&fragment.text);
                    });
                    ui.add_space(4.0);
                }
            });
        });
}
