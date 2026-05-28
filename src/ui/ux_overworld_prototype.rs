//! Overworld map visualisation prototype — terrain-tile map layout.
//!
//! Run with: `cargo run --bin ux_overworld_prototype`
//!
//! Controls:
//!   WASD   — pan the map
//!   Tab    — cycle focus node
//!   R      — reset pan
//!   Esc    — quit

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use super::ux_dungeon_style_prototype::styled as dng_styled;
use super::ux_style_contract::{style_for, VariantStyle};

// ── fake overworld graph data ─────────────────────────────────────────────────

const NODE_COUNT: usize = 9;
const NODE_NAMES: [&str; 9] = [
    "Shelter Iris",
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
// layout x,y in a more organic, Qud-like positioning
const NODE_POS: [(f32, f32); 9] = [
    (180.0, 440.0),  // Shelter Iris - slightly offset
    (420.0, 320.0),  // Forest Crossroads - more organic
    (620.0, 280.0),  // Shattered Labs - asymmetric
    (360.0, 460.0),  // Ruins of Malkov - irregular
    (540.0, 440.0),  // Sunken Pass - varied spacing
    (680.0, 380.0),  // Ember Vein - non-grid
    (460.0, 180.0),  // The Still Gate - scattered
    (700.0, 160.0),  // Ashen Bazaar - asymmetric
    (840.0, 300.0),  // Hollow Spire - irregular placement
];
// road pairs (from, to)
const ROADS: [(usize, usize); 11] = [
    (0, 1), (1, 2), (1, 3), (3, 4), (4, 5),
    (1, 6), (6, 7), (2, 5), (6, 2),
    (7, 8), (5, 8),
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
// biome_color removed — inline gamma_multiply in painter calls instead

// ── state ─────────────────────────────────────────────────────────────────────

#[derive(Resource)]
pub(crate) struct OverworldProtoState {
    pub(crate) focus_node: usize,
    pub(crate) elapsed: f32,
    pub(crate) pan: (f32, f32),
}

pub struct OverworldPrototypePlugin;

impl Plugin for OverworldPrototypePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(OverworldProtoState {
            focus_node: 0,
            elapsed: 0.0,
            pan: (0.0, 0.0),
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
        state.focus_node = (state.focus_node + 1) % NODE_COUNT;
    }
    if keys.just_pressed(KeyCode::KeyR) {
        state.pan = (0.0, 0.0);
    }

    let s = 24.0;
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft)  { state.pan.0 += s; }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) { state.pan.0 -= s; }
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp)    { state.pan.1 += s; }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown)  { state.pan.1 -= s; }
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
                    " Overworld Map  |  Focus: {}  |  WASD pan  Tab focus  R reset  Esc quit",
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

fn draw_terrain_tile(ui: &mut egui::Ui, s: &VariantStyle, state: &OverworldProtoState) {
    let t = state.elapsed;
    let focus = state.focus_node;
    let mut pan = state.pan;
    let focus_node_type = NODE_TYPES[focus];
    let focus_name = NODE_NAMES[focus];
    let focus_pos = NODE_POS[focus];
    
    // Auto-center map on focus node
    let rect = ui.available_rect_before_wrap();
    let center_x = rect.width() / 2.0;
    let center_y = rect.height() / 2.0;
    pan.0 = center_x - focus_pos.0;
    pan.1 = center_y - focus_pos.1;

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

            // biome backdrop with fog of war — organic Qud-style rendering
            let cols_visible = ((rect.width() / TILE_W).ceil() as i32).max(1);
            let rows_visible = ((rect.height() / TILE_H).ceil() as i32).max(1);
            let ox = (-pan.0 / TILE_W).floor() as i32;
            let oy = (-pan.1 / TILE_H).floor() as i32;

            let mut map_str = String::with_capacity((cols_visible * rows_visible) as usize * 2);
            let focus_world_x = focus_pos.0 / TILE_W;
            let focus_world_y = focus_pos.1 / TILE_H;
            
            // Add organic background texture for Qud aesthetic
            let organic_noise = ((t * 0.5).sin() * 0.3 + 0.7) as f32;
            
            for row in oy..oy + rows_visible {
                for col in ox..ox + cols_visible {
                    let gx = col.rem_euclid(MAP_W as i32) as usize;
                    let gy = row.rem_euclid(MAP_H as i32) as usize;
                    let b = BIOME[gy][gx];
                    
                    if is_visible_overworld(focus_world_x, focus_world_y, col, row, t) {
                        let base_char = biome_char(b);
                        // Add organic variation for Qud feel
                        let variation = if (gx + gy + t as usize) % 7 == 0 {
                            match b {
                                1 => "♠", // forest variation
                                2 => "♦", // mountain variation  
                                3 => "≈", // water variation
                                4 => "◈", // ruins variation
                                5 => "∴", // ash variation
                                _ => base_char,
                            }
                        } else {
                            base_char
                        };
                        map_str.push_str(variation);
                    } else {
                        let (fog_char, _alpha) = get_fog_glyph(col, row, focus_world_x, focus_world_y, t);
                        map_str.push(fog_char);
                    }
                }
                map_str.push('\n');
            }
            
            // Render with organic, slightly irregular spacing for Qud aesthetic
            let font_size = TILE_W - 2.0 + (t.sin() * 0.5); // Subtle size variation
            let color = s.subtitle_color.gamma_multiply(0.3 * organic_noise);
            painter.text(
                rect.left_top(),
                egui::Align2::LEFT_TOP,
                &map_str,
                egui::FontId::monospace(font_size),
                color,
            );

            // roads between nodes with fog of war
            let screens: Vec<egui::Pos2> = NODE_POS
                .iter()
                .map(|&(nx, ny)| egui::pos2(nx + pan.0, ny + pan.1))
                .collect();

            for &(a, b) in &ROADS {
                let pa = screens[a];
                let pb = screens[b];
                if (pa.x < rect.left() - 50.0 && pb.x < rect.left() - 50.0)
                    || (pa.x > rect.right() + 50.0 && pb.x > rect.right() + 50.0)
                    || (pa.y < rect.top() - 50.0 && pb.y < rect.top() - 50.0)
                    || (pa.y > rect.bottom() + 50.0 && pb.y > rect.bottom() + 50.0)
                {
                    continue;
                }
                
                // Check if both nodes are visible
                let node_a_world_x = NODE_POS[a].0 / TILE_W;
                let node_a_world_y = NODE_POS[a].1 / TILE_H;
                let node_b_world_x = NODE_POS[b].0 / TILE_W;
                let node_b_world_y = NODE_POS[b].1 / TILE_H;
                
                let a_visible = is_visible_overworld(focus_world_x, focus_world_y, node_a_world_x as i32, node_a_world_y as i32, t);
                let b_visible = is_visible_overworld(focus_world_x, focus_world_y, node_b_world_x as i32, node_b_world_y as i32, t);
                
                // Only draw road if at least one node is visible
                if !a_visible && !b_visible {
                    continue;
                }
                
                let highlight = a == focus || b == focus;
                let mut road_color = if highlight {
                    s.accent_color
                } else {
                    s.subtitle_color.gamma_multiply(0.45)
                };
                
                // Apply fog alpha if only one node is visible
                if a_visible && !b_visible {
                    let (_, alpha) = get_fog_glyph(node_b_world_x as i32, node_b_world_y as i32, focus_world_x, focus_world_y, t);
                    road_color = road_color.gamma_multiply(alpha);
                } else if !a_visible && b_visible {
                    let (_, alpha) = get_fog_glyph(node_a_world_x as i32, node_a_world_y as i32, focus_world_x, focus_world_y, t);
                    road_color = road_color.gamma_multiply(alpha);
                }
                
                if highlight {
                    painter.line_segment([pa, pb], egui::Stroke::new(2.0, road_color));
                } else {
                    let steps = 8;
                    for i in 0..steps {
                        let u1 = i as f32 / steps as f32;
                        let u2 = (i as f32 + 0.6) / steps as f32;
                        painter.line_segment(
                            [
                                egui::pos2(pa.x + (pb.x - pa.x) * u1, pa.y + (pb.y - pa.y) * u1),
                                egui::pos2(pa.x + (pb.x - pa.x) * u2, pa.y + (pb.y - pa.y) * u2),
                            ],
                            egui::Stroke::new(1.2, road_color),
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
                let is_visible = is_visible_overworld(focus_world_x, focus_world_y, node_world_x as i32, node_world_y as i32, t);
                
                if !is_visible && idx != focus {
                    continue; // Skip rendering invisible nodes (except focus)
                }
                
                let is_focus = idx == focus;
                let pulse = if is_focus {
                    ((t * 3.0).sin() * 0.5 + 0.5) as f32 * 0.3 + 0.7
                } else {
                    1.0
                };
                let glyph = node_glyph(NODE_TYPES[idx]);
                let mut node_color = if is_focus { s.title_color } else { s.warn_color };
                
                // Apply fog alpha to non-focus nodes
                if !is_focus {
                    let (_, alpha) = get_fog_glyph(node_world_x as i32, node_world_y as i32, focus_world_x, focus_world_y, t);
                    node_color = node_color.gamma_multiply(alpha);
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
                if is_focus || (pos - screens[focus]).length() < 120.0 {
                    painter.text(
                        egui::pos2(pos.x, pos.y + 16.0),
                        egui::Align2::CENTER_TOP,
                        NODE_NAMES[idx],
                        egui::FontId::proportional(11.0),
                        node_color,
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
        cols[1].label(dng_styled(s, " [Tab]  Travel Route", s.body_size, s.title_color));
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