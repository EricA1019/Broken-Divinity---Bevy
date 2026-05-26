//! Colony UI — resource bar and survivor management panel.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::core::components::Player;
use crate::core::components::Position;
use crate::core::gamelog::{GameLog, LogColor, blocked_action_message};
use crate::core::movement::MapTiles;
use crate::core::resources::ShelterResources;
use crate::core::state::AppState;
use crate::core::stats::EntityName;
use crate::core::turn::GameTime;
use crate::game::colony::research::{CompletedResearch, ResearchProject};
use crate::game::colony::stations::{Station, StationType, find_station_anchor, spawn_station};
use crate::game::colony::survivors::{Survivor, SurvivorNeeds, SurvivorTask};
use crate::ui::objective_prompt::{COLONY_OBJECTIVE_PROMPT_TEXT, ColonyObjectivePromptState};
use crate::ui::readability::contrast_ratio;

const RESOURCE_BAR_BACKGROUND_RGB: (u8, u8, u8) = (25, 30, 20);
const URGENCY_BANNER_CRITICAL_RGB: (u8, u8, u8) = (235, 128, 128);
const URGENCY_BANNER_LOW_RGB: (u8, u8, u8) = (238, 214, 140);
const RESOURCE_BAR_MIN_CONTRAST_RATIO: f32 = 4.5;
const RESOURCE_BAR_X_MARGIN: i8 = 8;
const RESOURCE_BAR_Y_MARGIN: i8 = 4;
const URGENCY_LABEL_PREFIX: &str = "Urgent:";
const OBJECTIVE_INDICATOR_RGB: (u8, u8, u8) = (200, 185, 120);
const OBJECTIVE_DETAIL_INLINE_BY_DEFAULT: bool = false;

#[derive(Debug, Clone, Copy)]
pub(crate) struct ColonyReadabilitySnapshot {
    pub minimum_banner_contrast_ratio: f32,
    pub urgency_banner_contrast_ratio: f32,
}

pub(crate) fn colony_readability_snapshot() -> ColonyReadabilitySnapshot {
    ColonyReadabilitySnapshot {
        minimum_banner_contrast_ratio: RESOURCE_BAR_MIN_CONTRAST_RATIO,
        urgency_banner_contrast_ratio: contrast_ratio(
            URGENCY_BANNER_CRITICAL_RGB,
            RESOURCE_BAR_BACKGROUND_RGB,
        ),
    }
}

pub(crate) fn colony_objective_indicator_text(
    objective_prompt: Option<&ColonyObjectivePromptState>,
) -> Option<&'static str> {
    if objective_prompt.is_some_and(|prompt| prompt.visible_in_colony) {
        Some(COLONY_OBJECTIVE_PROMPT_TEXT)
    } else {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ColonyTopBarObjectivePresentation {
    pub primary_label: &'static str,
    pub inline_detail: Option<&'static str>,
    pub hover_detail: Option<&'static str>,
}

pub(crate) fn colony_top_bar_objective_presentation(
    objective_prompt: Option<&ColonyObjectivePromptState>,
) -> Option<ColonyTopBarObjectivePresentation> {
    let objective_detail = colony_objective_indicator_text(objective_prompt)?;
    let inline_detail = OBJECTIVE_DETAIL_INLINE_BY_DEFAULT.then_some(objective_detail);
    let hover_detail = if inline_detail.is_none() {
        Some(objective_detail)
    } else {
        None
    };

    Some(ColonyTopBarObjectivePresentation {
        primary_label: primary_colony_cta_label(objective_prompt),
        inline_detail,
        hover_detail,
    })
}

pub(crate) fn primary_colony_cta_label(
    objective_prompt: Option<&ColonyObjectivePromptState>,
) -> &'static str {
    if objective_prompt.is_some_and(|prompt| prompt.visible_in_colony) {
        return "Primary action: Reach the shelter gate and press Enter.";
    }

    "Primary action: Manage survivors and station output."
}

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
    StartResearch(ResearchProject),
}

// ---------------------------------------------------------------------------
// Draw — EguiPrimaryContextPass (read-only)
// ---------------------------------------------------------------------------

/// Draw the resource bar at the top when in Colony state.
pub fn draw_resource_bar(
    mut contexts: EguiContexts,
    state: Res<State<AppState>>,
    resources: Option<Res<ShelterResources>>,
    objective_prompt: Option<Res<ColonyObjectivePromptState>>,
    mut action: ResMut<ColonyUiAction>,
) {
    if *state.get() != AppState::Colony {
        return;
    }
    let readability = colony_readability_snapshot();
    debug_assert!(
        readability.urgency_banner_contrast_ratio
            >= readability.minimum_banner_contrast_ratio,
        "Colony urgency banner readability baseline violated"
    );
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let Some(res) = resources else { return };

    egui::TopBottomPanel::top("resource_bar")
        .frame(
            egui::Frame::NONE
                .fill(rgb(RESOURCE_BAR_BACKGROUND_RGB))
                .inner_margin(egui::Margin::symmetric(
                    RESOURCE_BAR_X_MARGIN,
                    RESOURCE_BAR_Y_MARGIN,
                )),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                resource_label(ui, "Food", res.food, egui::Color32::from_rgb(200, 160, 80));
                ui.separator();
                resource_label(
                    ui,
                    "Water",
                    res.water,
                    egui::Color32::from_rgb(80, 160, 220),
                );
                ui.separator();
                resource_label(
                    ui,
                    "Scrap",
                    res.scrap,
                    egui::Color32::from_rgb(180, 180, 180),
                );
                ui.separator();
                resource_label(
                    ui,
                    "Medicine",
                    res.medicine,
                    egui::Color32::from_rgb(100, 220, 100),
                );
                ui.separator();
                resource_label(ui, "Ammo", res.ammo, egui::Color32::from_rgb(220, 180, 80));

                let warnings = collect_resource_warnings(res.as_ref());
                if !warnings.is_empty() {
                    ui.separator();
                    let text = warnings
                        .iter()
                        .map(|warning| warning.label.as_str())
                        .collect::<Vec<_>>()
                        .join(" | ");
                    let has_critical = warnings
                        .iter()
                        .any(|warning| warning.severity == ResourceSeverity::Critical);
                    let color = if has_critical {
                        rgb(URGENCY_BANNER_CRITICAL_RGB)
                    } else {
                        rgb(URGENCY_BANNER_LOW_RGB)
                    };
                    ui.label(
                        egui::RichText::new(format!("{URGENCY_LABEL_PREFIX} {text}"))
                            .strong()
                            .color(color),
                    );
                }

                if let Some(presentation) =
                    colony_top_bar_objective_presentation(objective_prompt.as_deref())
                {
                    ui.separator();
                    let primary_response = ui.label(
                        egui::RichText::new(presentation.primary_label)
                            .strong()
                            .color(rgb(OBJECTIVE_INDICATOR_RGB)),
                    );
                    if let Some(hover_detail) = presentation.hover_detail {
                        primary_response.on_hover_text(hover_detail);
                    }
                    if let Some(inline_detail) = presentation.inline_detail {
                        ui.label(egui::RichText::new(inline_detail).color(rgb(OBJECTIVE_INDICATOR_RGB)));
                    }
                }

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
    mut research: ResMut<CompletedResearch>,
    map: Option<Res<MapTiles>>,
    players: Query<&Position, With<Player>>,
) {
    let Some(choice) = action.0.take() else {
        return;
    };

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
                    blocked_action_message(
                        "Build Station",
                        &format!("Not enough resources for {station_name}"),
                        "Gather required materials and try again",
                    ),
                    LogColor::Status,
                    time.turn,
                );
                return;
            }

            let Some(map) = map else {
                log.push(
                    blocked_action_message(
                        "Build Station",
                        "No shelter map available",
                        "Re-enter shelter state and retry",
                    ),
                    LogColor::Status,
                    time.turn,
                );
                return;
            };

            let player_anchor = players
                .iter()
                .next()
                .map(|pos| pos.to_ivec2())
                .unwrap_or(IVec2::new(map.width as i32 / 2, map.height as i32 / 2));
            let occupied_positions = stations
                .iter()
                .map(|(_, pos)| pos.to_ivec2())
                .chain(std::iter::once(player_anchor));
            let Some(anchor) = find_station_anchor(&map, occupied_positions, player_anchor) else {
                log.push(
                    blocked_action_message(
                        "Build Station",
                        "No valid floor tile is available",
                        "Clear space or move and retry",
                    ),
                    LogColor::Status,
                    time.turn,
                );
                return;
            };

            // Consume resources
            for &(res, amt) in kind.build_cost() {
                resources.try_consume(res, amt);
            }

            spawn_station(&mut commands, kind, anchor.x, anchor.y);

            log.push(
                format!("Built {station_name} at ({}, {})!", anchor.x, anchor.y),
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
        ColonyUiChoice::StartResearch(project) => {
            if research.active.is_some() {
                log.push(
                    blocked_action_message(
                        "Start Research",
                        "Another project is already active",
                        "Wait for completion before starting a new project",
                    ),
                    LogColor::Status,
                    time.turn,
                );
                return;
            }

            let (res_kind, cost) = project.cost();
            if !resources.try_consume(res_kind, cost) {
                log.push(
                    blocked_action_message(
                        "Start Research",
                        &format!("Not enough {} for {}", res_kind.name(), project.name()),
                        "Collect resources and try again",
                    ),
                    LogColor::Status,
                    time.turn,
                );
                return;
            }

            research.active = Some((project, project.ticks_to_complete()));
            log.push(
                format!("Started research: {}!", project.name()),
                LogColor::PlayerHit,
                time.turn,
            );
        }
    }
}

fn resource_label(ui: &mut egui::Ui, name: &str, value: u32, color: egui::Color32) {
    ui.label(egui::RichText::new(format!("{name}: {value}")).color(color));
}

fn rgb((red, green, blue): (u8, u8, u8)) -> egui::Color32 {
    egui::Color32::from_rgb(red, green, blue)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResourceSeverity {
    Low,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResourceWarning {
    label: String,
    severity: ResourceSeverity,
}

fn warning_for_resource(name: &str, value: u32) -> Option<ResourceWarning> {
    let (critical, low) = match name {
        "Food" | "Water" => (5, 15),
        "Medicine" | "Ammo" => (3, 8),
        _ => return None,
    };

    if value <= critical {
        Some(ResourceWarning {
            label: format!("{name} CRITICAL"),
            severity: ResourceSeverity::Critical,
        })
    } else if value <= low {
        Some(ResourceWarning {
            label: format!("{name} LOW"),
            severity: ResourceSeverity::Low,
        })
    } else {
        None
    }
}

fn collect_resource_warnings(resources: &ShelterResources) -> Vec<ResourceWarning> {
    let mut warnings = Vec::new();
    for (name, value) in [
        ("Food", resources.food),
        ("Water", resources.water),
        ("Medicine", resources.medicine),
        ("Ammo", resources.ammo),
    ] {
        if let Some(warning) = warning_for_resource(name, value) {
            warnings.push(warning);
        }
    }
    warnings
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

/// Draw the research panel below the build panel on the left side.
pub fn draw_research_panel(
    mut contexts: EguiContexts,
    state: Res<State<AppState>>,
    resources: Option<Res<ShelterResources>>,
    research: Res<CompletedResearch>,
    mut action: ResMut<ColonyUiAction>,
) {
    if *state.get() != AppState::Colony {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let Some(res) = resources else { return };

    egui::Window::new("Research")
        .default_pos(egui::pos2(8.0, 350.0))
        .default_width(220.0)
        .collapsible(true)
        .resizable(false)
        .show(ctx, |ui| {
            // Active research progress
            if let Some((project, ticks_remaining)) = research.active {
                ui.label(
                    egui::RichText::new(format!("Researching: {}", project.name()))
                        .strong()
                        .color(egui::Color32::from_rgb(180, 220, 255)),
                );
                let total = project.ticks_to_complete();
                let progress = 1.0 - (ticks_remaining as f32 / total as f32);
                ui.add(
                    egui::ProgressBar::new(progress)
                        .desired_width(200.0)
                        .text(format!("{}/{} ticks", total - ticks_remaining, total))
                        .fill(egui::Color32::from_rgb(80, 140, 220)),
                );
                ui.add_space(6.0);
                ui.separator();
            }

            // Available projects
            let has_available = ResearchProject::ALL
                .iter()
                .any(|p| research.is_available(*p));

            if !has_available && research.active.is_none() {
                ui.label(
                    egui::RichText::new("All research complete!")
                        .italics()
                        .color(egui::Color32::from_rgb(100, 220, 100)),
                );
                return;
            }

            for &project in ResearchProject::ALL {
                if research.is_completed(project) {
                    ui.label(
                        egui::RichText::new(format!("✓ {}", project.name()))
                            .color(egui::Color32::from_rgb(100, 200, 100)),
                    );
                    continue;
                }

                if research
                    .active
                    .is_some_and(|(active, _)| active == project)
                {
                    continue; // Shown in progress bar above
                }

                let (res_kind, cost) = project.cost();
                let can_afford = res.get(res_kind) >= cost && research.active.is_none();
                let label = format!(
                    "{}\n  {} {} · {} ticks\n  {}",
                    project.name(),
                    cost,
                    res_kind.name(),
                    project.ticks_to_complete(),
                    project.description(),
                );

                ui.add_enabled_ui(can_afford, |ui| {
                    if ui.button(&label).clicked() && action.0.is_none() {
                        action.0 = Some(ColonyUiChoice::StartResearch(project));
                    }
                });
                ui.add_space(2.0);
            }
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
                                    let label =
                                        format!("{} ({}/{})", station_name, assigned, slots);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_warning_for_resource_thresholds() {
        assert_eq!(
            warning_for_resource("Food", 4),
            Some(ResourceWarning {
                label: "Food CRITICAL".to_string(),
                severity: ResourceSeverity::Critical,
            })
        );
        assert_eq!(
            warning_for_resource("Water", 10),
            Some(ResourceWarning {
                label: "Water LOW".to_string(),
                severity: ResourceSeverity::Low,
            })
        );
        assert_eq!(warning_for_resource("Scrap", 2), None);
        assert_eq!(warning_for_resource("Ammo", 12), None);
    }

    #[test]
    fn test_collect_resource_warnings_orders_by_resource_bar() {
        let resources = ShelterResources {
            food: 12,
            water: 2,
            scrap: 999,
            medicine: 7,
            ammo: 1,
        };

        let warnings = collect_resource_warnings(&resources);
        let labels: Vec<_> = warnings.iter().map(|warning| warning.label.as_str()).collect();
        assert_eq!(
            labels,
            vec!["Food LOW", "Water CRITICAL", "Medicine LOW", "Ammo CRITICAL"]
        );
    }
}
