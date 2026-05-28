//! Overworld map rendering via egui.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::core::state::AppState;

use super::graphgen::{NodeType, OverworldGraph};
use super::travel::TravelState;

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Resource tracking which node the player is at.
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct PlayerMapPosition {
    pub current_node: usize,
}

/// Resource holding the generated overworld graph.
#[derive(Resource, Debug, Clone)]
pub struct WorldMap(pub OverworldGraph);

/// Resource set when player selects a destination node (consumed by travel start).
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct SelectedDestination(pub Option<usize>);

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Draw the overworld map as an egui CentralPanel.
pub fn draw_overworld_map(
    mut contexts: EguiContexts,
    state: Res<State<AppState>>,
    world_map: Option<Res<WorldMap>>,
    player_pos: Option<Res<PlayerMapPosition>>,
    mut selected: ResMut<SelectedDestination>,
    travel_state: Option<Res<TravelState>>,
) {
    if *state.get() != AppState::Overworld {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let Some(map) = world_map else { return };
    let Some(pos) = player_pos else { return };

    // If traveling, show travel status instead of map interaction
    if travel_state.is_some() {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Traveling...");
        });
        return;
    }

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(egui::Color32::from_rgb(15, 20, 15)))
        .show(ctx, |ui| {
            let rect = ui.available_rect_before_wrap();
            let center = rect.center();
            let scale = 25.0;

            // Collect clickable node rects first (needs mutable ui)
            let mut click_targets: Vec<(usize, egui::Response)> = Vec::new();
            for node in &map.0.nodes {
                if node.discovered && node.id != pos.current_node {
                    let screen_pos =
                        egui::pos2(center.x + node.x * scale, center.y - node.y * scale);
                    let radius = 7.0;
                    let node_rect = egui::Rect::from_center_size(
                        screen_pos,
                        egui::vec2(radius * 2.0, radius * 2.0),
                    );
                    let resp = ui.allocate_rect(node_rect, egui::Sense::click());
                    click_targets.push((node.id, resp));
                }
            }

            // Now paint with immutable borrow
            let painter = ui.painter();

            // Draw roads
            for road in &map.0.roads {
                let from = &map.0.nodes[road.from];
                let to = &map.0.nodes[road.to];
                if !from.discovered && !to.discovered {
                    continue;
                }

                let p1 = egui::pos2(center.x + from.x * scale, center.y - from.y * scale);
                let p2 = egui::pos2(center.x + to.x * scale, center.y - to.y * scale);
                painter.line_segment(
                    [p1, p2],
                    egui::Stroke::new(1.5, egui::Color32::from_rgb(80, 80, 60)),
                );
            }

            // Draw nodes
            for node in &map.0.nodes {
                let screen_pos = egui::pos2(center.x + node.x * scale, center.y - node.y * scale);
                let radius = if node.id == pos.current_node {
                    10.0
                } else {
                    7.0
                };
                let color = if !node.discovered {
                    egui::Color32::from_rgb(60, 60, 60)
                } else {
                    match node.node_type {
                        NodeType::Shelter => egui::Color32::from_rgb(80, 200, 80),
                        NodeType::Dungeon => egui::Color32::from_rgb(200, 60, 60),
                        NodeType::Ruins => egui::Color32::from_rgb(180, 160, 80),
                        NodeType::Crossroads => egui::Color32::from_rgb(160, 160, 160),
                        NodeType::Landmark => egui::Color32::from_rgb(160, 80, 200),
                    }
                };

                painter.circle_filled(screen_pos, radius, color);

                if node.id == pos.current_node {
                    painter.circle_stroke(
                        screen_pos,
                        radius + 2.0,
                        egui::Stroke::new(2.0, egui::Color32::WHITE),
                    );
                }

                if node.discovered {
                    painter.text(
                        egui::pos2(screen_pos.x, screen_pos.y + radius + 4.0),
                        egui::Align2::CENTER_TOP,
                        &node.name,
                        egui::FontId::proportional(11.0),
                        egui::Color32::from_rgb(200, 200, 200),
                    );
                } else {
                    painter.text(
                        egui::pos2(screen_pos.x, screen_pos.y + radius + 4.0),
                        egui::Align2::CENTER_TOP,
                        "?",
                        egui::FontId::proportional(11.0),
                        egui::Color32::from_rgb(100, 100, 100),
                    );
                }
            }

            // Process clicks (responses already captured)
            for (node_id, resp) in &click_targets {
                if resp.clicked() && map.0.neighbors(pos.current_node).contains(node_id) {
                    selected.0 = Some(*node_id);
                }
            }
        });
}
