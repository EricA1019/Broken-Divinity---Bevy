//! Overworld HUD panel — weather, travel info, node inspector.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::core::resources::ShelterResources;
use crate::core::state::AppState;
use crate::game::overworld::graphgen::NodeType;
use crate::game::overworld::map::{PlayerMapPosition, WorldMap};
use crate::game::overworld::travel::TravelState;
use crate::ui::input_hints::{OVERWORLD_RETURN_HINT_TEXT, SAVE_AND_QUIT_HINT_TEXT, SAVE_AND_QUIT_LABEL};
use crate::ui::runtime_action_language::RuntimeActionLanguage;
use crate::ui::ux_style_contract::runtime_shell_layout;

const ACTION_TO_HINT_SPACING_MULTIPLIER: f32 = 2.0;
const SUPPLY_RISK_FOOD_AND_WATER_TEXT: &str =
    "Supplies critical: secure food and water before long travel.";
const SUPPLY_RISK_FOOD_TEXT: &str = "Supplies critical: secure food before long travel.";
const SUPPLY_RISK_WATER_TEXT: &str =
    "Supplies critical: secure water before long travel.";

#[derive(Resource, Default)]
pub struct OverworldUiAction(pub Option<OverworldUiChoice>);

#[derive(Clone, Copy, Debug)]
pub enum OverworldUiChoice {
    SaveAndQuit,
}

pub fn primary_overworld_cta_label() -> &'static str {
    RuntimeActionLanguage::overworld_primary_cta_label()
}

pub(crate) fn overworld_supply_status_summary(resources: &ShelterResources) -> Option<&'static str> {
    match (resources.food == 0, resources.water == 0) {
        (true, true) => Some(SUPPLY_RISK_FOOD_AND_WATER_TEXT),
        (true, false) => Some(SUPPLY_RISK_FOOD_TEXT),
        (false, true) => Some(SUPPLY_RISK_WATER_TEXT),
        (false, false) => None,
    }
}

pub fn draw_overworld_panel(
    mut contexts: EguiContexts,
    state: Res<State<AppState>>,
    travel_state: Option<Res<TravelState>>,
    world_map: Option<Res<WorldMap>>,
    player_pos: Option<Res<PlayerMapPosition>>,
    resources: Option<Res<ShelterResources>>,
    mut action: ResMut<OverworldUiAction>,
) {
    if *state.get() != AppState::Overworld {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let Some(map) = world_map else {
        return;
    };
    let Some(pos) = player_pos else {
        return;
    };
    let shell_layout = runtime_shell_layout();

    egui::SidePanel::left("overworld_panel")
        .default_width(220.0)
        .resizable(false)
        .show(ctx, |ui| {
            if let Some(node) = map.0.node(pos.current_node) {
                ui.heading(&node.name);
                ui.label(format!("Type: {}", node_type_label(node.node_type)));
                ui.separator();
            }

            if let Some(ref travel) = travel_state {
                ui.heading("Traveling");
                ui.label(format!("Day {}", travel.day));
                ui.label(format!("Weather: {}", travel.current_weather.name()));

                let progress = if travel.distance_remaining > 0.0 {
                    let total = map
                        .0
                        .road_between(travel.from_node, travel.to_node)
                        .map_or(1.0, |road| road.distance);
                    1.0 - (travel.distance_remaining / total).clamp(0.0, 1.0)
                } else {
                    1.0
                };
                ui.add(egui::ProgressBar::new(progress).text(format!("{:.0}%", progress * 100.0)));

                if let Some(dest) = map.0.node(travel.to_node) {
                    ui.label(format!("Destination: {}", dest.name));
                }
                ui.separator();
            }

            if let Some(resources) = resources.as_ref() {
                let has_supply_warning = resources.food == 0 || resources.water == 0;
                if has_supply_warning {
                    ui.heading("Supply Risk");
                    if let Some(summary) = overworld_supply_status_summary(resources.as_ref()) {
                        ui.label(egui::RichText::new(summary).strong());
                    }
                    let danger = egui::Color32::from_rgb(220, 96, 96);
                    if resources.food == 0 {
                        ui.colored_label(danger, "No food: travel encounters become much riskier.");
                    }
                    if resources.water == 0 {
                        ui.colored_label(
                            danger,
                            "No water: the expedition is operating at the edge.",
                        );
                    }
                    ui.separator();
                }
            }

            ui.heading("Nearby");
            let neighbors = map.0.neighbors(pos.current_node);
            for &nid in &neighbors {
                let Some(neighbor) = map.0.node(nid) else {
                    continue;
                };
                if neighbor.discovered {
                    let dist = map
                        .0
                        .road_between(pos.current_node, nid)
                        .map_or(0.0, |road| road.distance);
                    ui.label(format!(
                        "  {} ({}) — {:.1} days",
                        neighbor.name,
                        node_type_label(neighbor.node_type),
                        dist
                    ));
                } else {
                    ui.label("  ??? — unknown");
                }
            }

            ui.separator();
            ui.label(egui::RichText::new(primary_overworld_cta_label()).strong());
            ui.label(OVERWORLD_RETURN_HINT_TEXT);

            ui.add_space(shell_layout.section_to_section_spacing * ACTION_TO_HINT_SPACING_MULTIPLIER);
            if ui.button(SAVE_AND_QUIT_LABEL).clicked() {
                action.0 = Some(OverworldUiChoice::SaveAndQuit);
            }
            ui.label(SAVE_AND_QUIT_HINT_TEXT);
        });
}

pub fn process_overworld_action(mut action: ResMut<OverworldUiAction>, mut commands: Commands) {
    let Some(choice) = action.0.take() else {
        return;
    };

    match choice {
        OverworldUiChoice::SaveAndQuit => {
            commands.insert_resource(crate::core::save::SaveAndQuitRequested(true));
        }
    }
}

fn node_type_label(node_type: NodeType) -> &'static str {
    match node_type {
        NodeType::Shelter => "Shelter",
        NodeType::Dungeon => "Dungeon",
        NodeType::Ruins => "Ruins",
        NodeType::Crossroads => "Crossroads",
        NodeType::Landmark => "Landmark",
    }
}

#[cfg(test)]
mod tests {
    use super::overworld_supply_status_summary;
    use crate::core::resources::ShelterResources;

    #[test]
    fn overworld_supply_summary_calls_out_missing_food_and_water() {
        let resources = ShelterResources {
            food: 0,
            water: 0,
            ..ShelterResources::default()
        };

        assert_eq!(
            overworld_supply_status_summary(&resources),
            Some("Supplies critical: secure food and water before long travel."),
        );
    }

    #[test]
    fn overworld_supply_summary_calls_out_single_missing_resource() {
        let food_risk = ShelterResources {
            food: 0,
            water: 2,
            ..ShelterResources::default()
        };
        let water_risk = ShelterResources {
            food: 3,
            water: 0,
            ..ShelterResources::default()
        };

        assert_eq!(
            overworld_supply_status_summary(&food_risk),
            Some("Supplies critical: secure food before long travel."),
        );
        assert_eq!(
            overworld_supply_status_summary(&water_risk),
            Some("Supplies critical: secure water before long travel."),
        );
    }
}
