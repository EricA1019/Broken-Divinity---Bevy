//! Stats and progression panel — press K to open/close.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::core::components::Player;
use crate::core::perks::{PendingPerkChoices, PlayerPerks};
use crate::core::state::AppState;
use crate::core::stats::{PlayerProgression, ProficiencyId, VirtueId};
use crate::ui::input_hints::{STATS_TOGGLE_HINT_TEXT, STATS_TOGGLE_KEY};

#[derive(Resource, Default)]
pub struct StatsProgressionOpen(pub bool);

pub fn toggle_stats_progression(
    keys: Res<ButtonInput<KeyCode>>,
    mut open: ResMut<StatsProgressionOpen>,
) {
    if keys.just_pressed(STATS_TOGGLE_KEY) {
        open.0 = !open.0;
    }
}

pub fn draw_stats_progression_panel(
    mut contexts: EguiContexts,
    state: Res<State<AppState>>,
    open: Res<StatsProgressionOpen>,
    player_q: Query<(&PlayerProgression, Option<&PlayerPerks>), With<Player>>,
    pending_perks: Option<Res<PendingPerkChoices>>,
) {
    if !open.0 {
        return;
    }

    let is_allowed_state = matches!(
        state.get(),
        AppState::Dungeon | AppState::Colony | AppState::Overworld
    );
    if !is_allowed_state {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let Ok((progression, perks)) = player_q.single() else {
        return;
    };

    egui::Window::new("Stats & Progression")
        .default_size([520.0, 600.0])
        .collapsible(true)
        .resizable(true)
        .show(ctx, |ui| {
            ui.label(
                egui::RichText::new("Hybrid progression: virtues + proficiencies + kleos + perks")
                    .color(egui::Color32::from_rgb(190, 190, 190)),
            );
            ui.label(
                egui::RichText::new(STATS_TOGGLE_HINT_TEXT)
                    .small()
                    .color(egui::Color32::from_rgb(130, 130, 130)),
            );
            ui.separator();

            // Kleos summary
            let kleos = progression.kleos;
            let kleos_state = kleos_band_name(kleos);
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("Kleos: {}", kleos))
                        .strong()
                        .color(egui::Color32::from_rgb(220, 190, 100)),
                );
                ui.separator();
                ui.label(
                    egui::RichText::new(kleos_state)
                        .color(egui::Color32::from_rgb(180, 180, 210)),
                );
            });

            ui.add_space(6.0);
            ui.separator();
            ui.label(
                egui::RichText::new("Virtues")
                    .strong()
                    .color(egui::Color32::from_rgb(220, 210, 170)),
            );

            egui::Grid::new("virtues_grid")
                .num_columns(3)
                .spacing([14.0, 6.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("Virtue").strong());
                    ui.label(egui::RichText::new("Rank").strong());
                    ui.label(egui::RichText::new("Focus").strong());
                    ui.end_row();

                    for virtue in VirtueId::all() {
                        let rank = progression.virtue_rank(*virtue);
                        ui.label(virtue.name());
                        ui.label(rank.to_string());
                        ui.label(virtue_focus_text(*virtue));
                        ui.end_row();
                    }
                });

            ui.add_space(8.0);
            ui.separator();
            ui.label(
                egui::RichText::new("Practical Proficiencies")
                    .strong()
                    .color(egui::Color32::from_rgb(170, 210, 220)),
            );

            egui::Grid::new("proficiencies_grid")
                .num_columns(6)
                .spacing([10.0, 6.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("Proficiency").strong());
                    ui.label(egui::RichText::new("Rating").strong());
                    ui.label(egui::RichText::new("Level").strong());
                    ui.label(egui::RichText::new("XP").strong());
                    ui.label(egui::RichText::new("Next").strong());
                    ui.label(egui::RichText::new("Action Lane").strong());
                    ui.end_row();

                    for prof in ProficiencyId::all() {
                        let (rating, level, xp, next_xp) = progression
                            .proficiencies
                            .get(prof)
                            .map(|p| (p.effective(), p.level, p.xp, p.xp_for_next_level()))
                            .unwrap_or((0, 0, 0, 0));

                        ui.label(prof.name());
                        ui.label(rating.to_string());
                        ui.label(level.to_string());
                        ui.label(xp.to_string());
                        ui.label(next_xp.to_string());
                        ui.label(proficiency_lane_text(*prof));
                        ui.end_row();
                    }
                });

            ui.add_space(8.0);
            ui.separator();
            ui.label(
                egui::RichText::new("Current Action Ratings")
                    .strong()
                    .color(egui::Color32::from_rgb(210, 180, 140)),
            );
            ui.horizontal_wrapped(|ui| {
                let melee = progression.action_rating(VirtueId::Thumos, ProficiencyId::MeleeTraining, 0, 0);
                let ranged = progression.action_rating(VirtueId::Prudence, ProficiencyId::RangedTraining, 0, 0);
                let defense = progression.enemy_attack_dv();
                ui.label(format!("Melee: {}", melee));
                ui.separator();
                ui.label(format!("Ranged: {}", ranged));
                ui.separator();
                ui.label(format!("Defense DV: {}", defense));
            });

            ui.add_space(8.0);
            ui.separator();
            ui.label(
                egui::RichText::new("Perks")
                    .strong()
                    .color(egui::Color32::from_rgb(200, 170, 220)),
            );

            if let Some(perks) = perks {
                if perks.unlocked.is_empty() {
                    ui.label(
                        egui::RichText::new("No unlocked perks yet")
                            .color(egui::Color32::from_rgb(140, 140, 140)),
                    );
                } else {
                    for perk in &perks.unlocked {
                        ui.label(format!("- {} (T{}): {}", perk.name(), perk.tier(), perk.lane_label()));
                    }
                }
            } else {
                ui.label(
                    egui::RichText::new("Perk data unavailable")
                        .color(egui::Color32::from_rgb(140, 140, 140)),
                );
            }

            if let Some(pending) = pending_perks {
                if !pending.pending.is_empty() {
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new("Pending unlock choices")
                            .strong()
                            .color(egui::Color32::from_rgb(240, 180, 120)),
                    );
                    for perk in &pending.pending {
                        ui.label(format!("* {} [{}]", perk.name(), perk.lane_label()));
                    }
                }
            }
        });
}

fn kleos_band_name(kleos: u32) -> &'static str {
    match kleos {
        0..=9 => "Unknown",
        10..=24 => "Noticed",
        25..=44 => "Named",
        45..=69 => "Oath-Weighted",
        _ => "Legendary",
    }
}

fn virtue_focus_text(virtue: VirtueId) -> &'static str {
    match virtue {
        VirtueId::Temperance => "Control / corruption resistance",
        VirtueId::Justice => "Oath / legitimacy",
        VirtueId::Prudence => "Planning / discernment",
        VirtueId::Fortitude => "Endurance / survival",
        VirtueId::Thumos => "Courage / battle-drive",
        VirtueId::Metis => "Cunning / adaptation",
    }
}

fn proficiency_lane_text(prof: ProficiencyId) -> &'static str {
    match prof {
        ProficiencyId::MeleeTraining => "Combat technique",
        ProficiencyId::RangedTraining => "Aiming discipline",
        ProficiencyId::QuietMovement => "Stealth / evasion",
        ProficiencyId::Repair => "Engineering / salvage",
        ProficiencyId::Medicine => "Trauma treatment",
        ProficiencyId::Ritecraft => "Ritual execution",
    }
}
