#![allow(clippy::type_complexity)]

//! Persistent HUD overlay — HP bar, action points, ammo, armor status.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::core::components::Player;
use crate::core::inventory::{ArmorDurability, Equipment, RangedWeaponState};
use crate::core::items::find_item;
use crate::core::sanity::RaidExposure;
use crate::core::state::AppState;
use crate::core::stats::{CombatStats, PlayerProgression, ProficiencyId, VirtueId};
use crate::core::turn::{ActionBudget, GameTime};
use crate::game::dungeon::spawn::DungeonState;
use crate::ui::ux_style_contract::runtime_shell_layout;

/// Draw the top HUD bar showing HP, AP, turn, weapon, and armor status.
pub fn draw_hud(
    mut contexts: EguiContexts,
    state: Res<State<AppState>>,
    time: Res<GameTime>,
    dungeon_state: Option<Res<DungeonState>>,
    query: Query<
        (
            &CombatStats,
            &ActionBudget,
            Option<&Equipment>,
            Option<&RangedWeaponState>,
            Option<&ArmorDurability>,
            Option<&RaidExposure>,
            Option<&PlayerProgression>,
        ),
        With<Player>,
    >,
) {
    if *state.get() != AppState::Dungeon {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let Ok((stats, budget, equipment, ranged, armor_dur, raid_exposure, progression)) =
        query.single()
    else {
        return;
    };
    let shell_layout = runtime_shell_layout();

    egui::TopBottomPanel::top("hud_panel")
        .frame(
            egui::Frame::NONE
                .fill(egui::Color32::from_rgb(20, 20, 25))
                .inner_margin(egui::Margin::symmetric(
                    shell_layout.header_to_content_spacing as i8,
                    shell_layout.action_to_hint_spacing as i8,
                )),
        )
        .show(ctx, |ui| {
            // Row 1: HP bar, AP, Turn
            ui.horizontal(|ui| {
                let hp_frac = if stats.hp_max > 0 {
                    stats.hp.max(0) as f32 / stats.hp_max as f32
                } else {
                    0.0
                };
                ui.add(
                    egui::ProgressBar::new(hp_frac)
                        .fill(egui::Color32::from_rgb(180, 40, 40))
                        .desired_width(140.0)
                        .corner_radius(2)
                        .text(format!("HP: {}/{}", stats.hp, stats.hp_max)),
                );

                ui.separator();
                ui.label(
                    egui::RichText::new(format!("AP: {}/{}", budget.remaining, budget.speed))
                        .color(egui::Color32::from_rgb(100, 200, 255)),
                );

                ui.separator();
                ui.label(
                    egui::RichText::new(format!("Turn {}", time.turn))
                        .color(egui::Color32::from_rgb(180, 180, 180)),
                );

                // Sanity bar
                if let Some(exposure) = raid_exposure {
                    ui.separator();
                    let frac = exposure.fraction();
                    let threshold = exposure.threshold();
                    let color = match threshold {
                        crate::core::sanity::SanityThreshold::Normal => {
                            egui::Color32::from_rgb(80, 200, 80)
                        }
                        crate::core::sanity::SanityThreshold::Stressed => {
                            egui::Color32::from_rgb(220, 200, 50)
                        }
                        crate::core::sanity::SanityThreshold::Shaken => {
                            egui::Color32::from_rgb(220, 130, 40)
                        }
                        crate::core::sanity::SanityThreshold::Breaking => {
                            egui::Color32::from_rgb(200, 40, 40)
                        }
                    };
                    ui.add(
                        egui::ProgressBar::new(frac)
                            .fill(color)
                            .desired_width(120.0)
                            .corner_radius(2)
                            .text(format!("SAN: {} ({})", exposure.current, threshold.name())),
                    );
                }

                // Floor indicator
                if let Some(ref ds) = dungeon_state {
                    ui.separator();
                    ui.label(
                        egui::RichText::new(format!("F{}/{}", ds.floor_number, ds.max_floors))
                            .color(egui::Color32::from_rgb(160, 140, 200)),
                    );
                }
            });

            // Row 2: Weapon + ammo, Armor + durability
            ui.horizontal(|ui| {
                // Weapon info
                if let Some(equip) = equipment {
                    if let Some(ref wep_id) = equip.weapon {
                        let name = find_item(wep_id).map(|d| d.name).unwrap_or("???");
                        if let Some(rw) = ranged {
                            ui.label(
                                egui::RichText::new(format!(
                                    "⚔ {} [{}/{}]",
                                    name, rw.clip_current, rw.clip_size
                                ))
                                .color(egui::Color32::from_rgb(220, 180, 80)),
                            );
                        } else {
                            ui.label(
                                egui::RichText::new(format!("⚔ {}", name))
                                    .color(egui::Color32::from_rgb(220, 180, 80)),
                            );
                        }
                    } else {
                        ui.label(
                            egui::RichText::new("⚔ Unarmed")
                                .color(egui::Color32::from_rgb(120, 120, 120)),
                        );
                    }

                    ui.separator();

                    // Armor info
                    if let Some(ref arm_id) = equip.armor {
                        let name = find_item(arm_id).map(|d| d.name).unwrap_or("???");
                        if let Some(dur) = armor_dur {
                            let dur_color = if dur.broken {
                                egui::Color32::from_rgb(200, 50, 50)
                            } else {
                                egui::Color32::from_rgb(100, 180, 220)
                            };
                            ui.label(
                                egui::RichText::new(format!(
                                    "🛡 {} [{}/{}]",
                                    name, dur.current, dur.max
                                ))
                                .color(dur_color),
                            );
                        } else {
                            ui.label(
                                egui::RichText::new(format!("🛡 {}", name))
                                    .color(egui::Color32::from_rgb(100, 180, 220)),
                            );
                        }
                    } else {
                        ui.label(
                            egui::RichText::new("🛡 None")
                                .color(egui::Color32::from_rgb(120, 120, 120)),
                        );
                    }
                } else {
                    ui.label(
                        egui::RichText::new("⚔ Unarmed")
                            .color(egui::Color32::from_rgb(120, 120, 120)),
                    );
                    ui.separator();
                    ui.label(
                        egui::RichText::new("🛡 None").color(egui::Color32::from_rgb(120, 120, 120)),
                    );
                }

                if let Some(progression) = progression {
                    let virtue_bonus = i32::from(progression.virtue_rank(VirtueId::Thumos)) * 5;
                    let melee_rating = progression.proficiency_rating(ProficiencyId::MeleeTraining);
                    let action_rating = progression.action_rating(
                        VirtueId::Thumos,
                        ProficiencyId::MeleeTraining,
                        0,
                        0,
                    );

                    ui.separator();
                    ui.label(
                        egui::RichText::new(format!(
                            "MLY {} [THU {} + TRN {}]",
                            action_rating, virtue_bonus, melee_rating
                        ))
                        .color(egui::Color32::from_rgb(210, 170, 90)),
                    );

                    ui.separator();

                    let ranged_virtue_bonus =
                        i32::from(progression.virtue_rank(VirtueId::Prudence)) * 5;
                    let ranged_rating =
                        progression.proficiency_rating(ProficiencyId::RangedTraining);
                    let ranged_action_rating = progression.action_rating(
                        VirtueId::Prudence,
                        ProficiencyId::RangedTraining,
                        0,
                        0,
                    );
                    ui.label(
                        egui::RichText::new(format!(
                            "RNG {} [PRU {} + TRN {}]",
                            ranged_action_rating, ranged_virtue_bonus, ranged_rating
                        ))
                        .color(egui::Color32::from_rgb(120, 190, 210)),
                    );

                    ui.separator();

                    let defense_virtue_bonus =
                        i32::from(progression.virtue_rank(VirtueId::Metis)) * 5;
                    let defense_rating =
                        progression.proficiency_rating(ProficiencyId::QuietMovement);
                    let defense_action_rating = progression.enemy_attack_dv();
                    ui.label(
                        egui::RichText::new(format!(
                            "DEF {} [MET {} + QUI {}]",
                            defense_action_rating, defense_virtue_bonus, defense_rating
                        ))
                        .color(egui::Color32::from_rgb(170, 200, 130)),
                    );
                }
            });
        });
}
