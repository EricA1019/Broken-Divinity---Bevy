//! Overworld map visualisation prototype — 5 layout approaches.
//!
//! Run with: `cargo run --bin ux_overworld_prototype`
//!
//! Controls:
//!   1-5    — switch layout
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
// layout x,y in a playable graph
const NODE_POS: [(f32, f32); 9] = [
    (200.0, 420.0),  // Shelter Iris
    (380.0, 340.0),  // Forest Crossroads
    (580.0, 300.0),  // Shattered Labs
    (340.0, 480.0),  // Ruins of Malkov
    (520.0, 460.0),  // Sunken Pass
    (700.0, 400.0),  // Ember Vein
    (480.0, 200.0),  // The Still Gate
    (720.0, 200.0),  // Ashen Bazaar
    (820.0, 320.0),  // Hollow Spire
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
fn biome_color(b: u8, s: &VariantStyle) -> egui::Color32 {
    match b {
        1 => egui::Color32::from_rgb(84, 130, 76),   // forest
        2 => egui::Color32::from_rgb(140, 120, 100),  // mountain
        3 => egui::Color32::from_rgb(72, 116, 148),   // water
        4 => egui::Color32::from_rgb(160, 110, 95),   // ruins
        5 => egui::Color32::from_rgb(112, 104, 96),   // ash
        _ => s.subtitle_color,
    }
}

// ── hex grid helpers ──────────────────────────────────────────────────────────

fn hex_center(col: i32, row: i32, r: f32) -> (f32, f32) {
    let w = r * 2.0;
    let h = r * 1.732; // sqrt(3)
    let x = col as f32 * w * 0.75 + r;
    let y = row as f32 * h + if col % 2 == 1 { h * 0.5 } else { 0.0 } + r;
    (x, y)
}

fn hex_corners(cx: f32, cy: f32, r: f32) -> Vec<egui::Pos2> {
    let mut pts = Vec::with_capacity(6);
    for i in 0..6 {
        let angle = std::f32::consts::PI / 3.0 * i as f32 - std::f32::consts::PI / 6.0;
        pts.push(egui::pos2(cx + r * angle.cos(), cy + r * angle.sin()));
    }
    pts
}

// ── state ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OverworldLayout {
    TerrainTile,
    Constellation,
    HexTerrain,
    SignalDrift,
    CartographerSketch,
}

impl OverworldLayout {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::TerrainTile        => "Terrain Tile Map",
            Self::Constellation      => "Constellation",
            Self::HexTerrain         => "Hex Terrain",
            Self::SignalDrift        => "Signal Drift",
            Self::CartographerSketch => "Cartographer's Sketch",
        }
    }
}

#[derive(Resource)]
pub(crate) struct OverworldProtoState {
    pub(crate) layout: OverworldLayout,
    pub(crate) focus_node: usize,
    pub(crate) elapsed: f32,
    pub(crate) pan: (f32, f32),
}

pub struct OverworldPrototypePlugin;

impl Plugin for OverworldPrototypePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(OverworldProtoState {
            layout: OverworldLayout::TerrainTile,
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
    if keys.just_pressed(KeyCode::Digit1) { state.layout = OverworldLayout::TerrainTile; }
    if keys.just_pressed(KeyCode::Digit2) { state.layout = OverworldLayout::Constellation; }
    if keys.just_pressed(KeyCode::Digit3) { state.layout = OverworldLayout::HexTerrain; }
    if keys.just_pressed(KeyCode::Digit4) { state.layout = OverworldLayout::SignalDrift; }
    if keys.just_pressed(KeyCode::Digit5) { state.layout = OverworldLayout::CartographerSketch; }

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
                    " Overworld Proto  |  Layout [{}]  |  Focus: {}  |  1-5 layouts  WASD pan  Tab focus  R reset  Esc quit",
                    state.layout.label(),
                    NODE_NAMES[state.focus_node],
                ).as_str(),
                11.0,
                s.subtitle_color,
            ));
            ui.separator();

            match state.layout {
                OverworldLayout::TerrainTile        => draw_terrain_tile(ui, &s, &state),
                OverworldLayout::Constellation      => draw_constellation(ui, &s, &state),
                OverworldLayout::HexTerrain         => draw_hex_terrain(ui, &s, &state),
                OverworldLayout::SignalDrift        => draw_signal_drift(ui, &s, &state),
                OverworldLayout::CartographerSketch => draw_cartographer_sketch(ui, &s, &state),
            }

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

// === LAYOUT 1: TERRAIN TILE MAP (Qud-style) ==================================

fn draw_terrain_tile(ui: &mut egui::Ui, s: &VariantStyle, state: &OverworldProtoState) {
    draw_section(ui, s, " Layout 1 — Biome Grid (Caves of Qud inspired) ");
    egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
        let p = state.pan;
        for (gy, row) in BIOME.iter().enumerate() {
            ui.horizontal(|ui| {
                for (gx, &b) in row.iter().enumerate() {
                    let x = gx as f32 * 13.0 + p.0;
                    let y = gy as f32 * 13.0 + p.1;
                    // skip off-screen glyphs
                    if x < -20.0 || x > 1200.0 || y < -20.0 || y > 900.0 {
                        // still need to consume labels for alignment
                    }
                    let ch = biome_char(b);
                    let color = biome_color(b, s);
                    ui.label(dng_styled(s, ch, 11.0, color));
                }
            });
        }

        // overlay nodes on top as small label blocks
        let focus = state.focus_node;
        for (idx, &(nx, ny)) in NODE_POS.iter().enumerate() {
            let _sx = nx * 0.65 + p.0;
            let _sy = ny * 0.65 + p.1;
            let color = if idx == focus { s.title_color } else { s.warn_color };
            let name = NODE_NAMES[idx];
            ui.label(dng_styled(s, &format!(" {} {} ", node_glyph(NODE_TYPES[idx]), name), 13.0, color));
            // position hack: use spacing to approximate placement
            // for a real prototype this would use absolute positioning
        }
    });
}

// === LAYOUT 2: CONSTELLATION (Slay the Spire) =================================

fn draw_constellation(ui: &mut egui::Ui, s: &VariantStyle, state: &OverworldProtoState) {
    draw_section(ui, s, " Layout 2 — Constellation (Slay the Spire branching paths) ");
    let p = state.pan;
    let focus = state.focus_node;

    // — painter scope —
    let node_cards = {
        let painter = ui.painter();
        // roads as faint lines — collect then paint
        let mut road_lines: Vec<(egui::Pos2, egui::Pos2, egui::Color32)> = Vec::new();
        let mut cards: Vec<(String, egui::Color32)> = Vec::new();
        for &(a, b) in &ROADS {
            let (ax, ay) = NODE_POS[a];
            let (bx, by) = NODE_POS[b];
            let start = egui::pos2(ax + p.0, ay + p.1);
            let end = egui::pos2(bx + p.0, by + p.1);
            let highlight = a == focus || b == focus;
            let color = if highlight { s.accent_color } else { s.subtitle_color.gamma_multiply(0.3) };
            road_lines.push((start, end, color));
        }
        for (start, end, color) in &road_lines {
            painter.line_segment([*start, *end], egui::Stroke::new(2.0, *color));
        }
        for (idx, &(nx, ny)) in NODE_POS.iter().enumerate() {
            let sx = nx + p.0;
            let sy = ny + p.1;
            let is_focus = idx == focus;
            let color = if is_focus { s.title_color } else { s.info_color };
            let card_rect = egui::Rect::from_min_size(
                egui::pos2(sx - 2.0, sy - 2.0),
                egui::vec2(110.0, 22.0),
            );
            let fill = if is_focus {
                s.accent_color.gamma_multiply(0.12)
            } else {
                egui::Color32::from_rgba_premultiplied(0, 0, 0, 0)
            };
            painter.rect_filled(card_rect, 2.0, fill);
            painter.rect_stroke(card_rect, 2.0, egui::Stroke::new(1.0, color), egui::StrokeKind::Outside);
            cards.push((format!(" {}  {}", node_glyph(NODE_TYPES[idx]), NODE_NAMES[idx]), color));
        }
        cards
    };

    for (label, color) in &node_cards {
        ui.label(dng_styled(&s, label, 13.0, *color));
    }
}

// === LAYOUT 3: HEX TERRAIN ====================================================

fn draw_hex_terrain(ui: &mut egui::Ui, s: &VariantStyle, state: &OverworldProtoState) {
    draw_section(ui, s, " Layout 3 — Hex Terrain Grid ");
    let p = state.pan;
    let hex_r = 18.0;
    let focus = state.focus_node;

    // — painter scope —
    let hex_labels = {
        let painter = ui.painter();
        // paint hex grid
        for row in 0..14 {
            for col in 0..18 {
                let (cx, cy) = hex_center(col, row, hex_r);
                let sx = cx + p.0;
                let sy = cy + p.1;
                if sx < 0.0 || sx > 1200.0 || sy < 0.0 || sy > 900.0 { continue; }
                let mut base = ((row * 7 + col * 13) % 5) as u8 + 1;
                if base > 5 { base = 1; }
                let color = biome_color(base, s).gamma_multiply(0.45);
                let corners = hex_corners(sx, sy, hex_r - 1.0);
                painter.add(egui::Shape::convex_polygon(
                    corners, color,
                    egui::Stroke::new(0.5, s.subtitle_color.gamma_multiply(0.2)),
                ));
            }
        }
        // overlay roads
        for &(a, b) in &ROADS {
            let (ax, ay) = NODE_POS[a];
            let (bx, by) = NODE_POS[b];
            painter.line_segment(
                [egui::pos2(ax + p.0, ay + p.1), egui::pos2(bx + p.0, by + p.1)],
                egui::Stroke::new(2.0, s.accent_color.gamma_multiply(0.5)),
            );
        }
        // node circles then collect labels
        let mut labels: Vec<(String, egui::Color32)> = Vec::new();
        for (idx, &(nx, ny)) in NODE_POS.iter().enumerate() {
            let sx = nx + p.0;
            let sy = ny + p.1;
            let color = if idx == focus { s.title_color } else { s.warn_color };
            painter.circle_filled(egui::pos2(sx, sy), 8.0, s.panel_bg);
            painter.circle_stroke(egui::pos2(sx, sy), 8.0, egui::Stroke::new(2.0, color));
            labels.push((format!(" {}  {}", node_glyph(NODE_TYPES[idx]), NODE_NAMES[idx]), color));
        }
        labels
    };

    for (label, color) in &hex_labels {
        ui.label(dng_styled(&s, label, 13.0, *color));
    }
}

// === LAYOUT 4: SIGNAL DRIFT (non-standard — CRT triangulation) ================

fn draw_signal_drift(ui: &mut egui::Ui, s: &VariantStyle, state: &OverworldProtoState) {
    draw_section(ui, s, " Layout 4 — Signal Drift (CRT radio-triangulation aesthetic) ");
    let t = state.elapsed;
    let p = state.pan;
    let focus = state.focus_node;

    // — painter scope — static overlay, scanlines, roads, beacons, labels
    let node_labels = {
        let painter = ui.painter();
        let rect = ui.available_rect_before_wrap();
        // static overlay
        let static_color = egui::Color32::from_rgba_premultiplied(60, 70, 50, 8);
        painter.rect_filled(rect, 0.0, static_color);
        // horizontal scanlines
        let mut sy = rect.top() + (t * 40.0).rem_euclid(4.0);
        while sy < rect.bottom() {
            painter.line_segment(
                [egui::pos2(rect.left(), sy), egui::pos2(rect.right(), sy)],
                egui::Stroke::new(1.0, egui::Color32::from_rgba_premultiplied(0, 0, 0, 20)),
            );
            sy += 4.0;
        }
        // roads as interference lines
        let mut road_segments: Vec<(egui::Pos2, egui::Pos2, egui::Color32)> = Vec::new();
        let mut labels: Vec<(String, egui::Color32)> = Vec::new();
        for &(a, b) in &ROADS {
            let (ax, ay) = NODE_POS[a];
            let (bx, by) = NODE_POS[b];
            let start = egui::pos2(ax + p.0, ay + p.1);
            let end = egui::pos2(bx + p.0, by + p.1);
            let jitter = (t * 3.0 + a as f32 * 1.7 + b as f32 * 2.3).sin() * 3.0;
            let mid = egui::pos2((start.x + end.x) / 2.0 + jitter, (start.y + end.y) / 2.0);
            let noise_color = s.success_color.gamma_multiply(0.35);
            road_segments.push((start, mid, noise_color));
            road_segments.push((mid, end, noise_color));
        }
        for (start, end, color) in &road_segments {
            painter.line_segment([*start, *end], egui::Stroke::new(1.5, *color));
        }
        // nodes as pulsing radio beacons
        for (idx, &(nx, ny)) in NODE_POS.iter().enumerate() {
            let sx = nx + p.0;
            let sy = ny + p.1;
            let is_foc = idx == focus;
            let pulse = ((t * 2.0 + idx as f32 * 0.9).sin() * 0.5 + 0.5) as f32;
            let radius = 6.0 + pulse * 12.0;
            let ring_color = if is_foc { s.title_color } else { s.success_color }
                .gamma_multiply(0.2 + pulse * 0.3);
            painter.circle_stroke(egui::pos2(sx, sy), radius, egui::Stroke::new(1.0, ring_color));
            painter.circle_stroke(egui::pos2(sx, sy), radius + 8.0, egui::Stroke::new(0.5, ring_color.gamma_multiply(0.3)));
            let core_color = if is_foc { s.title_color } else { s.success_color };
            painter.circle_filled(egui::pos2(sx, sy), 3.0, core_color);
            let freq = 88.0 + idx as f32 * 12.5 + (t * 0.1).sin() * 2.0;
            let freq_str = if is_foc {
                format!(" {} MHz  <<< {}", freq as u32, NODE_NAMES[idx])
            } else {
                format!(" {} MHz", freq as u32)
            };
            labels.push((freq_str, core_color));
        }
        labels
    };

    for (label, color) in &node_labels {
        ui.label(dng_styled(&s, label, s.small_size, *color));
    }
}

// === LAYOUT 5: CARTOGRAPHER'S SKETCH (non-standard — ink field journal) =======

fn draw_cartographer_sketch(ui: &mut egui::Ui, s: &VariantStyle, state: &OverworldProtoState) {
    draw_section(ui, s, " Layout 5 — Cartographer's Sketch (hand-drawn ink journal aesthetic) ");
    let t = state.elapsed;
    let p = state.pan;
    let focus = state.focus_node;

    // — painter scope — paper, grid, ink roads, hatching, circles, labels
    let (ink_labels, ink_faint) = {
        let painter = ui.painter();
        let rect = ui.available_rect_before_wrap();
        // paper background
        let paper_color = egui::Color32::from_rgb(225, 215, 195);
        let ink_color = egui::Color32::from_rgb(38, 32, 28);
        let ink_faint = ink_color.gamma_multiply(0.4);
        painter.rect_filled(rect, 0.0, paper_color);
        // grid lines — faint blue pencil
        let pencil = egui::Color32::from_rgba_premultiplied(120, 140, 180, 40);
        let mut gx = rect.left() + 20.0;
        while gx < rect.right() {
            painter.line_segment(
                [egui::pos2(gx, rect.top()), egui::pos2(gx, rect.bottom())],
                egui::Stroke::new(0.5, pencil),
            );
            gx += 20.0;
        }
        let mut gy = rect.top() + 20.0;
        while gy < rect.bottom() {
            painter.line_segment(
                [egui::pos2(rect.left(), gy), egui::pos2(rect.right(), gy)],
                egui::Stroke::new(0.5, pencil),
            );
            gy += 20.0;
        }
        // roads as hand-drawn ink lines
        let mut segments: Vec<(egui::Pos2, egui::Pos2, egui::Color32)> = Vec::new();
        let mut labels: Vec<(String, egui::Color32)> = Vec::new();
        for &(a, b) in &ROADS {
            let (ax, ay) = NODE_POS[a];
            let (bx, by) = NODE_POS[b];
            let start = egui::pos2(ax + p.0, ay + p.1);
            let end = egui::pos2(bx + p.0, by + p.1);
            let wob = (t * 0.5 + a as f32 * 3.7 + b as f32 * 5.3).sin() * 1.5;
            let mid = egui::pos2((start.x + end.x) / 2.0 + wob, (start.y + end.y) / 2.0);
            let draw_ink = if a == focus || b == focus { ink_color } else { ink_faint };
            segments.push((start, mid, draw_ink));
            segments.push((mid, end, draw_ink));
        }
        for (start, end, color) in &segments {
            painter.line_segment([*start, *end], egui::Stroke::new(1.8, *color));
        }
        for (idx, &(nx, ny)) in NODE_POS.iter().enumerate() {
            let sx = nx + p.0;
            let sy = ny + p.1;
            // cross-hatch
            for hatch in 0..4 {
                let ha = hatch as f32 * 1.57 + idx as f32 * 0.7;
                let hx = sx + 14.0 * ha.cos() + (t * 0.3).sin() * 2.0;
                let hy = sy + 14.0 * ha.sin();
                painter.line_segment(
                    [egui::pos2(sx, sy), egui::pos2(hx, hy)],
                    egui::Stroke::new(0.5, ink_faint),
                );
            }
            // hand-drawn circle
            let r = 14.0;
            let circ_color = if idx == focus { ink_color } else { ink_faint };
            for seg in 0..8 {
                let a1 = seg as f32 * 0.785;
                let a2 = (seg + 1) as f32 * 0.785;
                let p1 = egui::pos2(sx + r * a1.cos(), sy + r * a1.sin());
                let p2 = egui::pos2(sx + r * a2.cos(), sy + r * a2.sin());
                painter.line_segment([p1, p2], egui::Stroke::new(1.5, circ_color));
            }
            labels.push((format!(" {}  {}", node_glyph(NODE_TYPES[idx]), NODE_NAMES[idx]),
                if idx == focus { ink_color } else { ink_color.gamma_multiply(0.75) }));
        }
        (labels, ink_faint)
    };

    for (label, color) in &ink_labels {
        ui.label(dng_styled(&s, label, 12.0, *color));
    }

    let annotate = format!("field journal · day {} · rec: {}", (t as u32) % 60 + 1, NODE_NAMES[focus]);
    ui.label(dng_styled(&s, &format!("   {}   ", annotate), 10.0, ink_faint));
}