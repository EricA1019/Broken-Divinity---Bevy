//! Overworld HUD panel — weather, travel info, node inspector.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::core::resources::ShelterResources;
use crate::core::state::AppState;
use crate::game::overworld::graphgen::NodeType;
use crate::game::overworld::map::{PlayerMapPosition, WorldMap};
use crate::game::overworld::travel::TravelState;

#[derive(Resource, Default)]
pub struct OverworldUiAction(pub Option<OverworldUiChoice>);

#[derive(Clone, Copy, Debug)]
pub enum OverworldUiChoice {
    SaveAndQuit,
}

pub fn primary_overworld_cta_label() -> &'static str {
    "Click a connected node to travel."
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
            ui.label("Press Esc to return to colony shelter.");

            ui.add_space(12.0);
            if ui.button("Save & Quit").clicked() {
                action.0 = Some(OverworldUiChoice::SaveAndQuit);
            }
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
