//! Overworld map visualisation prototype — terrain-tile map layout.
//!
//! Run with: `cargo run --bin ux_overworld_prototype`
//!
//! Controls:
//!   Tab    — cycle expedition report
//!   R      — reset to first expedition
//!   Esc    — quit

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use super::ux_dungeon_style_prototype::styled as dng_styled;
use super::ux_style_contract::{style_for, VariantStyle};

// ── fake overworld graph data ─────────────────────────────────────────────────

pub(crate) const NODE_COUNT: usize = 9;
pub(crate) const COLONY_NODE: usize = 0;
const NODE_NAMES: [&str; 9] = [
    "Mock Colony",
    "Forest Crossroads",
    "Shattered Labs",
    "Ruins of Malkov",
    "Sunken Pass",
    "Ember Vein",
    "The Still Gate",
    "Ashen Bazaar",
    "Hollow Spire",
];
const NODE_TYPES: [&str; 9] = [
    "Shelter", "Crossroads", "Dungeon", "Ruins", "Landmark",
    "Dungeon", "Crossroads", "Ruins", "Dungeon",
];
// layout x,y centered on the colony with nearby and distant POIs
const NODE_POS: [(f32, f32); 9] = [
    (480.0, 300.0),  // Mock Colony - centered anchor
    (560.0, 250.0),  // Forest Crossroads - visible
    (615.0, 205.0),  // Shattered Labs - near edge of fog
    (405.0, 370.0),  // Ruins of Malkov - visible
    (360.0, 245.0),  // Sunken Pass - visible
    (700.0, 340.0),  // Ember Vein - beyond fog, reported only
    (525.0, 420.0),  // The Still Gate - visible
    (290.0, 175.0),  // Ashen Bazaar - beyond fog, reported only
    (770.0, 230.0),  // Hollow Spire - beyond fog, reported only
];
// road pairs (from, to)
const ROADS: [(usize, usize); 10] = [
    (0, 1), (0, 3), (0, 4), (1, 2), (1, 5),
    (3, 6), (4, 7), (5, 8), (2, 5), (6, 2),
];

fn node_glyph(nt: &str) -> &'static str {
    match nt {
        "Shelter"    => "▲",
        "Dungeon"    => "★",
        "Ruins"      => "■",
        "Crossroads" => "●",
        "Landmark"   => "◆",
        _            => "?",
    }
}

// ── biome terrain for tile-based layouts ──────────────────────────────────────

const MAP_W: usize = 40;
const MAP_H: usize = 30;
static BIOME: [[u8; MAP_W]; MAP_H] = {
    let mut grid = [[0u8; MAP_W]; MAP_H];
    // fill with forest(1), some mountain(2), water(3), ruins(4), ash(5)
    let mut y = 0;
    while y < MAP_H {
        let mut x = 0;
        while x < MAP_W {
            grid[y][x] = 1; // default forest
            // horizontal bands
            if y < 4 || (y >= 12 && y < 16) || y >= 26 {
                grid[y][x] = 2; // mountain
            }
            // water pockets
            if (x >= 5 && x <= 8 && y >= 6 && y <= 10)
                || (x >= 22 && x <= 27 && y >= 18 && y <= 22)
            {
                grid[y][x] = 3; // water
            }
            // ash bands
            if (y >= 8 && y <= 10) || (y >= 20 && y <= 22) {
                grid[y][x] = 5; // ash
            }
            x += 1;
        }
        y += 1;
    }
    // manually mark some ruin tiles
    grid[8][18] = 4;
    grid[9][19] = 4;
    grid[10][18] = 4;
    grid[8][19] = 4;
    grid[21][30] = 4;
    grid[22][31] = 4;
    grid[21][31] = 4;
    grid
};

fn biome_char(b: u8) -> &'static str {
    match b {
        1 => "♣",  // forest
        2 => "▲",  // mountain
        3 => "~",  // water
        4 => "▓",  // ruins
        5 => ":",  // ash
        _ => " ",
    }
}

fn biome_color(b: u8) -> egui::Color32 {
    match b {
        1 => egui::Color32::from_rgb(111, 174, 104),
        2 => egui::Color32::from_rgb(164, 150, 108),
        3 => egui::Color32::from_rgb(78, 156, 190),
        4 => egui::Color32::from_rgb(186, 123, 91),
        5 => egui::Color32::from_rgb(199, 163, 104),
        _ => egui::Color32::from_rgb(120, 114, 98),
    }
}

// ── fog of war visibility system ─────────────────────────────────────────────

fn is_visible_overworld(focus_x: f32, focus_y: f32, tile_x: i32, tile_y: i32, _elapsed: f32) -> bool {
    let dx = tile_x as f32 - focus_x;
    let dy = tile_y as f32 - focus_y;
    let dist2 = dx * dx + dy * dy;
    dist2 <= 144.0  // 12 tile visibility radius
}

fn get_fog_glyph(tile_x: i32, tile_y: i32, focus_x: f32, focus_y: f32, elapsed: f32) -> (char, f32) {
    let dx = (tile_x as f32 - focus_x).abs() + (tile_y as f32 - focus_y).abs() + (elapsed * 2.0);
    let ring = dx.rem_euclid(8.0);
    let echo = match ring {
        0.0..=1.0 => '.',
        1.0..=2.0 => ':',
        2.0..=3.0 => ';',
        3.0..=4.0 => ',',
        4.0..=5.0 => '·',
        5.0..=6.0 => ' ',
        _ => ' ',
    };
    let alpha = 1.0 - (ring / 8.0);
    (echo, alpha)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RevealState {
    Visible,
    Reported,
    Hidden,
}

fn reveal_state(focus_x: f32, focus_y: f32, tile_x: i32, tile_y: i32) -> RevealState {
    if is_visible_overworld(focus_x, focus_y, tile_x, tile_y, 0.0) {
        RevealState::Visible
    } else {
        let dx = tile_x as f32 - focus_x;
        let dy = tile_y as f32 - focus_y;
        let dist2 = dx * dx + dy * dy;
        if dist2 <= 400.0 {
        RevealState::Reported
        } else {
            RevealState::Hidden
        }
    }
}

fn tile_tint(state: RevealState, base: egui::Color32, fog: egui::Color32) -> egui::Color32 {
    match state {
        RevealState::Visible => base,
        RevealState::Reported => base.gamma_multiply(0.35),
        RevealState::Hidden => fog,
    }
}

fn second_leg_target(from: usize) -> Option<usize> {
    if from == COLONY_NODE {
        return None;
    }

    ROADS
        .iter()
        .filter_map(|&(a, b)| {
            let n = if a == from {
                b
            } else if b == from {
                a
            } else {
                return None;
            };
            if n == COLONY_NODE {
                return None;
            }
            let (fx, fy) = NODE_POS[from];
            let (nx, ny) = NODE_POS[n];
            let dist2 = (nx - fx) * (nx - fx) + (ny - fy) * (ny - fy);
            Some((n, dist2))
        })
        .min_by(|(_, da), (_, db)| da.partial_cmp(db).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(n, _)| n)
}
// biome_color removed — inline gamma_multiply in painter calls instead

// ── state ─────────────────────────────────────────────────────────────────────

#[derive(Resource)]
pub(crate) struct OverworldProtoState {
    pub(crate) focus_node: usize,
    pub(crate) elapsed: f32,
}

pub struct OverworldPrototypePlugin;

impl Plugin for OverworldPrototypePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(OverworldProtoState {
            focus_node: 1,
            elapsed: 0.0,
        })
        .add_systems(Startup, setup_camera)
        .add_systems(Update, (tick, handle_input))
        .add_systems(EguiPrimaryContextPass, draw_overworld_prototype);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn tick(time: Res<Time>, mut state: ResMut<OverworldProtoState>) {
    state.elapsed += time.delta_secs();
}

fn handle_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<OverworldProtoState>,
    mut exit: MessageWriter<AppExit>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
        return;
    }

    if keys.just_pressed(KeyCode::Tab) {
        state.focus_node += 1;
        if state.focus_node >= NODE_COUNT {
            state.focus_node = 1;
        }
    }
    if keys.just_pressed(KeyCode::KeyR) {
        state.focus_node = 1;
    }
}

// ── shared helpers ────────────────────────────────────────────────────────────

fn draw_section(ui: &mut egui::Ui, s: &VariantStyle, label: &str) {
    let line = format!("{} {} {}", "─".repeat(12), label, "─".repeat(12));
    ui.label(dng_styled(s, &line, s.small_size, s.accent_color));
}

// ── main draw dispatch ────────────────────────────────────────────────────────

fn draw_overworld_prototype(mut contexts: EguiContexts, state: Res<OverworldProtoState>) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let s = style_for();

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(s.panel_bg))
        .show(ctx, |ui| {
            // header
            ui.label(dng_styled(
                &s,
                format!(
                    " Mission Board  |  Expedition: {}  |  Tab cycle report  R reset report  Esc quit",
                    NODE_NAMES[state.focus_node],
                ).as_str(),
                11.0,
                s.subtitle_color,
            ));
            ui.separator();

            draw_terrain_tile(ui, &s, &state);

            ui.add_space(8.0 * s.spacing);

            // legend
            let nts = ["▲ Shelter", "★ Dungeon", "■ Ruins", "● Crossroads", "◆ Landmark"];
            ui.horizontal(|ui| {
                ui.label(dng_styled(&s, " Node types:", s.small_size, s.subtitle_color));
                for nt in nts {
                    ui.label(dng_styled(&s, nt, s.small_size, s.accent_color));
                }
            });
        });
}

// (other layouts Constellation, HexTerrain, SignalDrift, CartographerSketch removed)

// === LAYOUT 1: TERRAIN TILE MAP ==============================================

const TILE_W: f32 = 13.0;
const TILE_H: f32 = 16.0;

pub(crate) fn draw_terrain_tile(ui: &mut egui::Ui, s: &VariantStyle, state: &OverworldProtoState) {
    let t = state.elapsed;
    let focus = state.focus_node;
    let colony_pos = NODE_POS[COLONY_NODE];
    let focus_node_type = NODE_TYPES[focus];
    let focus_name = NODE_NAMES[focus];
    let focus_pos = NODE_POS[focus];

    // neighbors for route preview
    let neighbors: Vec<usize> = ROADS
        .iter()
        .filter_map(|&(a, b)| if a == focus { Some(b) } else if b == focus { Some(a) } else { None })
        .collect();

    // ── top travel strip ──────────────────────────────────────────────────────
    let day = ((t / 0.8) as u32) % 45 + 1;
    let weather_list = ["CLEAR", "CLOUDY", "RAIN", "ASHFALL", "CLEAR"];
    let weather = weather_list[(day / 3) as usize % weather_list.len()];
    let weather_color = match weather {
        "CLEAR" => s.success_color,
        "CLOUDY" => s.info_color,
        "RAIN" => s.info_color,
        "ASHFALL" => s.warn_color,
        _ => s.subtitle_color,
    };
    ui.horizontal(|ui| {
        ui.label(dng_styled(s, &format!(" DAY {:>2} ", day), s.body_size, s.title_color));
        ui.separator();
        ui.label(dng_styled(s, &format!(" {} ", weather), s.body_size, weather_color));
        ui.separator();
        ui.label(dng_styled(s, &format!(" {} ", focus_name), s.body_size, s.accent_color));
        ui.separator();
        ui.label(dng_styled(s, " FOOD 12 ", s.body_size, s.success_color));
        ui.separator();
        ui.label(dng_styled(s, " WATER 8 ", s.body_size, s.info_color));
        ui.separator();
        ui.label(dng_styled(s, " HP 74 ", s.body_size, s.success_color));
        ui.separator();
        if day % 6 == 0 {
            ui.label(dng_styled(s, " ⚠ RAID SOON ", s.body_size, s.danger_color));
        }
    });
    ui.separator();

    // ── two-column: map + route preview ───────────────────────────────────────
    ui.columns(2, |cols| {
        // ─── LEFT: map canvas (all painting in a scope block) ────────────────
        let node_screen = {
            let rect = cols[0].available_rect_before_wrap();
            cols[0].allocate_rect(rect, egui::Sense::hover());
            let painter = cols[0].painter();

            // Keep camera fixed on colony for mission-planning board feel.
            let camera_pan = (
                rect.center().x - colony_pos.0,
                rect.center().y - colony_pos.1,
            );

            // biome backdrop with fog of war — organic Qud-style rendering
            let cols_visible = ((rect.width() / TILE_W).ceil() as i32).max(1);
            let rows_visible = ((rect.height() / TILE_H).ceil() as i32).max(1);
            let ox = (-camera_pan.0 / TILE_W).floor() as i32;
            let oy = (-camera_pan.1 / TILE_H).floor() as i32;
            let focus_world_x = focus_pos.0 / TILE_W;
            let focus_world_y = focus_pos.1 / TILE_H;
            
            for row in oy..oy + rows_visible {
                for col in ox..ox + cols_visible {
                    let gx = col.rem_euclid(MAP_W as i32) as usize;
                    let gy = row.rem_euclid(MAP_H as i32) as usize;
                    let b = BIOME[gy][gx];
                    let reveal = reveal_state(focus_world_x, focus_world_y, col, row);
                    let screen = egui::pos2(
                        rect.left() + (col as f32 * TILE_W) + camera_pan.0,
                        rect.top() + (row as f32 * TILE_H) + camera_pan.1,
                    );
                    if reveal == RevealState::Hidden {
                        continue;
                    }

                    let glyph = if reveal == RevealState::Visible {
                        if (gx + gy + t as usize) % 7 == 0 {
                            match b {
                                1 => "♠",
                                2 => "♦",
                                3 => "≈",
                                4 => "◈",
                                5 => "∴",
                                _ => biome_char(b),
                            }
                        } else {
                            biome_char(b)
                        }
                    } else {
                        let (fog_char, _alpha) = get_fog_glyph(col, row, focus_world_x, focus_world_y, t);
                        &fog_char.to_string()
                    };

                    let color = tile_tint(
                        reveal,
                        biome_color(b),
                        s.subtitle_color.gamma_multiply(0.35),
                    );

                    painter.text(
                        screen,
                        egui::Align2::LEFT_TOP,
                        glyph,
                        egui::FontId::monospace(TILE_W - 2.0),
                        color,
                    );
                }
            }

            // route-plan lines only: colony -> selected expedition -> optional second leg
            let screens: Vec<egui::Pos2> = NODE_POS
                .iter()
                .map(|&(nx, ny)| egui::pos2(nx + camera_pan.0, ny + camera_pan.1))
                .collect();

            if focus != COLONY_NODE {
                let start = screens[COLONY_NODE];
                let mid = screens[focus];
                painter.line_segment(
                    [start, mid],
                    egui::Stroke::new(2.6, s.accent_color.gamma_multiply(0.85)),
                );

                if let Some(next) = second_leg_target(focus) {
                    let end = screens[next];
                    let steps = 10;
                    for i in 0..steps {
                        let u1 = i as f32 / steps as f32;
                        let u2 = (i as f32 + 0.55) / steps as f32;
                        painter.line_segment(
                            [
                                egui::pos2(mid.x + (end.x - mid.x) * u1, mid.y + (end.y - mid.y) * u1),
                                egui::pos2(mid.x + (end.x - mid.x) * u2, mid.y + (end.y - mid.y) * u2),
                            ],
                            egui::Stroke::new(1.5, s.info_color.gamma_multiply(0.7)),
                        );
                    }
                }
            }

            // node markers with fog of war
            for (idx, &pos) in screens.iter().enumerate() {
                if pos.x < rect.left() - 40.0 || pos.x > rect.right() + 40.0
                    || pos.y < rect.top() - 40.0 || pos.y > rect.bottom() + 40.0
                {
                    continue;
                }
                
                // Check if node is visible based on fog of war
                let node_world_x = NODE_POS[idx].0 / TILE_W;
                let node_world_y = NODE_POS[idx].1 / TILE_H;
                let reveal = reveal_state(focus_world_x, focus_world_y, node_world_x as i32, node_world_y as i32);
                if matches!(reveal, RevealState::Hidden) && idx != focus && idx != COLONY_NODE {
                    continue;
                }
                
                let is_focus = idx == focus;
                let pulse = if is_focus {
                    ((t * 3.0).sin() * 0.5 + 0.5) as f32 * 0.3 + 0.7
                } else {
                    1.0
                };
                let glyph = node_glyph(NODE_TYPES[idx]);
                let mut node_color = if idx == COLONY_NODE {
                    s.success_color
                } else if is_focus {
                    s.title_color
                } else {
                    s.warn_color
                };
                
                if !is_focus {
                    node_color = match reveal {
                        RevealState::Visible => node_color,
                        RevealState::Reported => node_color.gamma_multiply(0.35),
                        RevealState::Hidden => node_color.gamma_multiply(0.15),
                    };
                }

                let r = if is_focus { 10.0 } else { 7.0 };
                painter.circle_filled(pos, r, s.panel_bg.gamma_multiply(0.85));
                painter.circle_stroke(pos, r, egui::Stroke::new(2.0, node_color.gamma_multiply(pulse)));

                // glyph above node
                painter.text(
                    egui::pos2(pos.x, pos.y - 16.0),
                    egui::Align2::CENTER_CENTER,
                    glyph,
                    egui::FontId::proportional(14.0),
                    node_color,
                );

                // name below if focus or nearby
                if is_focus || idx == COLONY_NODE || matches!(reveal, RevealState::Visible) {
                    painter.text(
                        egui::pos2(pos.x, pos.y + 16.0),
                        egui::Align2::CENTER_TOP,
                        NODE_NAMES[idx],
                        egui::FontId::proportional(11.0),
                        node_color,
                    );
                } else if matches!(reveal, RevealState::Reported) {
                    painter.text(
                        egui::pos2(pos.x, pos.y + 16.0),
                        egui::Align2::CENTER_TOP,
                        "reported",
                        egui::FontId::proportional(10.0),
                        node_color.gamma_multiply(0.8),
                    );
                }
            }

            // current-location pulsing ring with organic Qud-style effects
            let ring_r = 14.0 + ((t * 2.5).sin() * 0.5 + 0.5) as f32 * 6.0;
            painter.circle_stroke(
                screens[focus],
                ring_r,
                egui::Stroke::new(1.5, s.title_color.gamma_multiply(0.5)),
            );

            // Colony anchor ring stays stable at screen center.
            painter.circle_stroke(
                screens[COLONY_NODE],
                12.0,
                egui::Stroke::new(1.8, s.success_color.gamma_multiply(0.85)),
            );
            
            // Add organic aura effect for Qud feel
            let aura_r = ring_r + 8.0 + ((t * 1.8).cos() * 0.3 + 0.7) as f32 * 4.0;
            painter.circle_stroke(
                screens[focus],
                aura_r,
                egui::Stroke::new(0.8, s.accent_color.gamma_multiply(0.2)),
            );

            screens
        };
        let _ring_center = node_screen[focus];

        // ─── RIGHT: route preview ─────────────────────────────────────────────
        draw_section(&mut cols[1], s, &format!(" {}  {} ", node_glyph(focus_node_type), focus_name));
        cols[1].label(dng_styled(s, &format!(" Type: {}", focus_node_type), s.body_size, s.info_color));
        cols[1].add_space(4.0 * s.spacing);

        draw_section(&mut cols[1], s, " Reachable Destinations ");
        if neighbors.is_empty() {
            cols[1].label(dng_styled(s, " (no connected nodes)", s.small_size, s.subtitle_color.gamma_multiply(0.6)));
        } else {
            for &n in &neighbors {
                let ntype = NODE_TYPES[n];
                let nname = NODE_NAMES[n];
                let (nx, ny) = NODE_POS[n];
                let (fx, fy) = focus_pos;
                let dist = ((nx - fx).powi(2) + (ny - fy).powi(2)).sqrt();
                let days = (dist / 65.0).ceil().max(1.0) as u32;
                let food_cost = days;
                let water_cost = days;

                let threat = if ntype == "Dungeon" { "Severe" }
                    else if ntype == "Ruins" { "Moderate" }
                    else { "Low" };
                let threat_color = match threat {
                    "Severe" => s.danger_color,
                    "Moderate" => s.warn_color,
                    _ => s.success_color,
                };

                let is_focus_neighbor = true; // all in the list are neighbors
                let row_color = if is_focus_neighbor { s.accent_color } else { s.subtitle_color };

                cols[1].horizontal(|ui| {
                    ui.label(dng_styled(s, node_glyph(ntype), s.body_size, row_color));
                    ui.vertical(|ui| {
                        ui.label(dng_styled(s, &format!(" {}", nname), s.body_size, row_color));
                        ui.horizontal(|ui| {
                            ui.label(dng_styled(s, &format!(" {}d | -{} food | -{} water | ", days, food_cost, water_cost), s.small_size, s.subtitle_color.gamma_multiply(0.7)));
                            ui.label(dng_styled(s, threat, s.small_size, threat_color));
                        });
                    });
                });
            }
        }

        cols[1].add_space(6.0 * s.spacing);
        draw_section(&mut cols[1], s, " Actions ");
        cols[1].label(dng_styled(s, " [Tab]  Cycle Expedition Report", s.body_size, s.title_color));
        cols[1].label(dng_styled(s, " [R]    Reset Report Selection", s.body_size, s.info_color));
        cols[1].label(dng_styled(s, " [X]    Inspect Node", s.body_size, s.info_color));
        cols[1].label(dng_styled(s, " [W]    Check Weather", s.body_size, s.info_color));
        cols[1].label(dng_styled(s, " [C]    Camp / Rest", s.body_size, s.info_color));

        cols[1].add_space(6.0 * s.spacing);
        draw_section(&mut cols[1], s, " Supply Packs ");
        cols[1].label(dng_styled(s, " Rations:    12 days", s.small_size, s.success_color));
        cols[1].add(egui::ProgressBar::new(12.0 / 20.0).desired_width(180.0).fill(s.success_color));
        cols[1].label(dng_styled(s, " Water:       8 days", s.small_size, s.info_color));
        cols[1].add(egui::ProgressBar::new(8.0 / 15.0).desired_width(180.0).fill(s.info_color));
        cols[1].label(dng_styled(s, " Medkits:         4", s.small_size, s.info_color.gamma_multiply(0.8)));
        cols[1].label(dng_styled(s, " Fuel cells:       2", s.small_size, s.info_color.gamma_multiply(0.8)));
    });

    // ── legend bar ────────────────────────────────────────────────────────────
    ui.add_space(4.0 * s.spacing);
    ui.separator();
    ui.horizontal(|ui| {
        ui.label(dng_styled(s, " ▲ Shelter  ", s.small_size, s.warn_color));
        ui.label(dng_styled(s, " ★ Dungeon  ", s.small_size, s.danger_color));
        ui.label(dng_styled(s, " ■ Ruins  ", s.small_size, s.info_color));
        ui.label(dng_styled(s, " ● Crossroads  ", s.small_size, s.success_color));
        ui.label(dng_styled(s, " ◆ Landmark  ", s.small_size, s.accent_color));
        ui.label(dng_styled(s, " | Biomes:  ♣ Forest  ▲ Mtn  ~ Water  : Ash  ▓ Ruins", s.small_size, s.subtitle_color.gamma_multiply(0.6)));
    });
}

// (end of file — other layouts removed)