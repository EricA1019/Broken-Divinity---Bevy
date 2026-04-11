//! Colony UI — resource bar and survivor management panel.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::core::components::Position;
use crate::core::gamelog::{GameLog, LogColor};
use crate::core::resources::ShelterResources;
use crate::core::state::AppState;
use crate::core::stats::EntityName;
use crate::core::turn::GameTime;
use crate::game::colony::stations::{spawn_station, Station, StationType};
use crate::game::colony::survivors::{Survivor, SurvivorNeeds, SurvivorTask};

// ---------------------------------------------------------------------------
// Action resource
// ---------------------------------------------------------------------------

#[derive(Resource, Default)]
pub struct ColonyUiAction(pub Option<ColonyUiChoice>);

#[derive(Clone, Copy, Debug)]
pub enum ColonyUiChoice {
    SaveAndQuit,
    AssignToStation { survivor: Entity, station: Entity },
    UnassignSurvivor { survivor: Entity },
    BuildStation(StationType),
}

// ---------------------------------------------------------------------------
// Draw — EguiPrimaryContextPass (read-only)
// ---------------------------------------------------------------------------

/// Draw the resource bar at the top when in Colony state.
pub fn draw_resource_bar(
    mut contexts: EguiContexts,
    state: Res<State<AppState>>,
    resources: Option<Res<ShelterResources>>,
    mut action: ResMut<ColonyUiAction>,
) {
    if *state.get() != AppState::Colony {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let Some(res) = resources else { return };

    egui::TopBottomPanel::top("resource_bar")
        .frame(
            egui::Frame::NONE
                .fill(egui::Color32::from_rgb(25, 30, 20))
                .inner_margin(egui::Margin::symmetric(8, 4)),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                resource_label(ui, "Food", res.food, egui::Color32::from_rgb(200, 160, 80));
                ui.separator();
                resource_label(ui, "Water", res.water, egui::Color32::from_rgb(80, 160, 220));
                ui.separator();
                resource_label(ui, "Scrap", res.scrap, egui::Color32::from_rgb(180, 180, 180));
                ui.separator();
                resource_label(
                    ui,
                    "Medicine",
                    res.medicine,
                    egui::Color32::from_rgb(100, 220, 100),
                );
                ui.separator();
                resource_label(ui, "Ammo", res.ammo, egui::Color32::from_rgb(220, 180, 80));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Save & Quit").clicked() {
                        action.0 = Some(ColonyUiChoice::SaveAndQuit);
                    }
                });
            });
        });
}

// ---------------------------------------------------------------------------
// Process — Update (mutations)
// ---------------------------------------------------------------------------

pub fn process_colony_action(
    mut action: ResMut<ColonyUiAction>,
    mut commands: Commands,
    mut survivors: Query<(&mut SurvivorTask, &EntityName), With<Survivor>>,
    mut stations: Query<(&mut Station, &Position)>,
    mut resources: ResMut<ShelterResources>,
    mut log: ResMut<GameLog>,
    time: Res<GameTime>,
) {
    let Some(choice) = action.0.take() else { return };

    match choice {
        ColonyUiChoice::SaveAndQuit => {
            commands.insert_resource(crate::core::save::SaveAndQuitRequested(true));
        }
        ColonyUiChoice::BuildStation(kind) => {
            let station_name = kind.name();

            // Check affordability
            let can_afford = kind
                .build_cost()
                .iter()
                .all(|&(res, amt)| resources.get(res) >= amt);

            if !can_afford {
                log.push(
                    format!("Not enough resources to build {station_name}"),
                    LogColor::Status,
                    time.turn,
                );
                return;
            }

            // Consume resources
            for &(res, amt) in kind.build_cost() {
                resources.try_consume(res, amt);
            }

            // Place at next grid slot based on current station count
            let station_count = stations.iter().count() as i32;
            spawn_station(&mut commands, kind, station_count * 2, 0);

            log.push(
                format!("Built {station_name}!"),
                LogColor::PlayerHit,
                time.turn,
            );
        }
        ColonyUiChoice::AssignToStation { survivor, station } => {
            let Ok((mut station_data, station_pos)) = stations.get_mut(station) else {
                return;
            };
            let station_name = station_data.kind.name().to_string();
            let station_ivec = station_pos.to_ivec2();
            station_data.workers_assigned += 1;

            let Ok((mut task, surv_name)) = survivors.get_mut(survivor) else {
                return;
            };
            *task = SurvivorTask::Working(station_ivec);
            log.push(
                format!("{} assigned to {}", surv_name.name, station_name),
                LogColor::System,
                time.turn,
            );
        }
        ColonyUiChoice::UnassignSurvivor { survivor } => {
            let Ok((mut task, surv_name)) = survivors.get_mut(survivor) else {
                return;
            };
            let surv_name_str = surv_name.name.clone();
            let old_pos = if let SurvivorTask::Working(pos) = *task {
                Some(pos)
            } else {
                None
            };
            *task = SurvivorTask::Idle;

            // Decrement workers_assigned on the matching station
            if let Some(work_pos) = old_pos {
                for (mut station, spos) in &mut stations {
                    if spos.to_ivec2() == work_pos && station.workers_assigned > 0 {
                        station.workers_assigned -= 1;
                        break;
                    }
                }
            }

            log.push(
                format!("{} unassigned", surv_name_str),
                LogColor::System,
                time.turn,
            );
        }
    }
}

fn resource_label(ui: &mut egui::Ui, name: &str, value: u32, color: egui::Color32) {
    ui.label(egui::RichText::new(format!("{name}: {value}")).color(color));
}

/// All station types available for building.
const BUILDABLE_STATIONS: &[StationType] = &[
    StationType::Cook,
    StationType::Purifier,
    StationType::Workbench,
    StationType::AmmoPress,
    StationType::Generator,
    StationType::MedicalBay,
    StationType::Quarters,
    StationType::ResearchTable,
    StationType::SecurityCheckpoint,
    StationType::MilitiaTraining,
];

/// Draw the station build panel on the left side.
pub fn draw_build_panel(
    mut contexts: EguiContexts,
    state: Res<State<AppState>>,
    resources: Option<Res<ShelterResources>>,
    mut action: ResMut<ColonyUiAction>,
) {
    if *state.get() != AppState::Colony {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let Some(res) = resources else { return };

    egui::SidePanel::left("build_panel")
        .default_width(220.0)
        .frame(
            egui::Frame::NONE
                .fill(egui::Color32::from_rgb(20, 22, 28))
                .inner_margin(egui::Margin::symmetric(8, 6)),
        )
        .show(ctx, |ui| {
            ui.label(
                egui::RichText::new("Build Station")
                    .strong()
                    .size(16.0)
                    .color(egui::Color32::from_rgb(220, 210, 180)),
            );
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                for &kind in BUILDABLE_STATIONS {
                    let cost = kind.build_cost();
                    let can_afford = cost.iter().all(|&(r, amt)| res.get(r) >= amt);

                    let cost_str: Vec<String> = cost
                        .iter()
                        .map(|(r, amt)| format!("{} {}", amt, r.name()))
                        .collect();
                    let label = format!("{}\n  Cost: {}", kind.name(), cost_str.join(", "));

                    ui.add_enabled_ui(can_afford, |ui| {
                        if ui.button(&label).clicked() && action.0.is_none() {
                            action.0 = Some(ColonyUiChoice::BuildStation(kind));
                        }
                    });
                    ui.add_space(2.0);
                }
            });
        });
}

/// Draw the survivor management panel on the right side.
///
/// Shows each survivor with needs bars and a station assignment combo box.
/// Writes assignment choices to `ColonyUiAction` — never mutates world state.
pub fn draw_survivor_panel(
    mut contexts: EguiContexts,
    state: Res<State<AppState>>,
    survivors: Query<(Entity, &EntityName, &SurvivorNeeds, &SurvivorTask), With<Survivor>>,
    stations: Query<(Entity, &Station, &Position), Without<Survivor>>,
    mut action: ResMut<ColonyUiAction>,
) {
    if *state.get() != AppState::Colony {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };

    // Collect available stations (those with open slots).
    let station_list: Vec<(Entity, &str, u8, u8)> = stations
        .iter()
        .map(|(e, s, _)| (e, s.kind.name(), s.workers_assigned, s.worker_slots))
        .collect();

    egui::SidePanel::right("survivor_panel")
        .default_width(240.0)
        .frame(
            egui::Frame::NONE
                .fill(egui::Color32::from_rgb(20, 22, 28))
                .inner_margin(egui::Margin::symmetric(8, 6)),
        )
        .show(ctx, |ui| {
            ui.label(
                egui::RichText::new("Survivors")
                    .strong()
                    .size(16.0)
                    .color(egui::Color32::from_rgb(220, 210, 180)),
            );
            ui.separator();

            if survivors.is_empty() {
                ui.label(
                    egui::RichText::new("No survivors yet.")
                        .italics()
                        .color(egui::Color32::GRAY),
                );
                return;
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                for (surv_entity, name, needs, task) in &survivors {
                    ui.group(|ui| {
                        ui.label(
                            egui::RichText::new(&name.name)
                                .strong()
                                .color(egui::Color32::WHITE),
                        );
                        ui.label(
                            egui::RichText::new(task_label(task))
                                .small()
                                .color(egui::Color32::LIGHT_GRAY),
                        );

                        ui.add_space(2.0);
                        need_bar(ui, "Hunger", needs.hunger);
                        need_bar(ui, "Thirst", needs.thirst);
                        need_bar(ui, "Rest", needs.rest);

                        // Station assignment UI
                        ui.add_space(4.0);
                        let is_working = matches!(task, SurvivorTask::Working(_));
                        if is_working {
                            if ui.button("Unassign").clicked() && action.0.is_none() {
                                action.0 = Some(ColonyUiChoice::UnassignSurvivor {
                                    survivor: surv_entity,
                                });
                            }
                        } else {
                            // Show assign buttons for stations with open slots
                            let available: Vec<_> = station_list
                                .iter()
                                .filter(|(_, _, assigned, slots)| *slots > 0 && assigned < slots)
                                .collect();
                            if !available.is_empty() {
                                ui.label(
                                    egui::RichText::new("Assign to:")
                                        .small()
                                        .color(egui::Color32::LIGHT_GRAY),
                                );
                                for &(station_entity, station_name, assigned, slots) in &available {
                                    let label = format!(
                                        "{} ({}/{})",
                                        station_name, assigned, slots
                                    );
                                    if ui.small_button(&label).clicked() && action.0.is_none() {
                                        action.0 = Some(ColonyUiChoice::AssignToStation {
                                            survivor: surv_entity,
                                            station: *station_entity,
                                        });
                                    }
                                }
                            }
                        }
                    });
                    ui.add_space(4.0);
                }
            });
        });
}

fn task_label(task: &SurvivorTask) -> &'static str {
    match task {
        SurvivorTask::Idle => "Idle",
        SurvivorTask::Working(_) => "Working",
        SurvivorTask::Resting => "Resting",
        SurvivorTask::SeekingFood => "Seeking Food",
        SurvivorTask::SeekingWater => "Seeking Water",
        SurvivorTask::Patrolling => "Patrolling",
    }
}

fn need_bar(ui: &mut egui::Ui, label: &str, value: u32) {
    let frac = (value as f32 / 100.0).clamp(0.0, 1.0);
    let color = if value > 50 {
        egui::Color32::from_rgb(80, 200, 80)
    } else if value > 20 {
        egui::Color32::from_rgb(220, 200, 60)
    } else {
        egui::Color32::from_rgb(220, 60, 60)
    };

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(format!("{label}:")).small());
        let bar = egui::ProgressBar::new(frac)
            .desired_width(100.0)
            .fill(color);
        ui.add(bar);
    });
}
