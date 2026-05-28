//! Focused dungeon-map prototype lab.
//!
//! This surface is intentionally separate from the general UX prototype so map/FOV
//! readability can be iterated rapidly without destabilizing other screens.
//!
//! Run with: `cargo run --bin ux_dungeon_map_prototypes`
//!
//! Controls:
//!   1/2/3     — layout mode (Focus / Tactical / Minimal)
//!   F         — toggle FOV mode (Classic / Tight)
//!   V/B       — next/previous render variant
//!   Arrow/WASD— move player
//!   Esc       — quit

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use super::ux_style_contract::{style_for, VariantStyle};

const MAP_ROWS: [&str; 15] = [
    "##################################",
    "#....g..............#...........>#",
    "#.######.##########.#.##########.#",
    "#.#....#.#........#.#.#........#.#",
    "#.#....#.#..g.....#.#.#..~~....#.#",
    "#.#....#.#........#.#.#........#.#",
    "#.####.#.######.###.#.#######.##.#",
    "#......#......#.....#.......#....#",
    "#.##########.#########.###.#####.#",
    "#.#........#.........#.#.#.....#.#",
    "#.#..^.....#########.#.#.#####.#.#",
    "#.#........#.......#.#.#.....#.#.#",
    "#.##########...g...#.#.#####.#.#.#",
    "#............#####.#.#.......#...#",
    "##################################",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MapLayout {
    Focus,
    Tactical,
    Minimal,
}

impl MapLayout {
    fn label(self) -> &'static str {
        match self {
            Self::Focus => "Focus",
            Self::Tactical => "Tactical",
            Self::Minimal => "Minimal",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FovMode {
    Classic,
    Tight,
}

impl FovMode {
    fn label(self) -> &'static str {
        match self {
            Self::Classic => "Classic",
            Self::Tight => "Tight",
        }
    }

    fn radius(self) -> i32 {
        match self {
            Self::Classic => 9,
            Self::Tight => 6,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RenderVariant {
    Standard,
    HighContrast,
    ThreatSilhouette,
    HeatTrail,
    SonarEcho,
    NegativeSpace,
}

impl RenderVariant {
    fn label(self) -> &'static str {
        match self {
            Self::Standard => "Standard",
            Self::HighContrast => "High Contrast",
            Self::ThreatSilhouette => "Threat Silhouette",
            Self::HeatTrail => "Heat Trail",
            Self::SonarEcho => "Sonar Echo*",
            Self::NegativeSpace => "Negative Space*",
        }
    }

    fn next(self) -> Self {
        match self {
            Self::Standard => Self::HighContrast,
            Self::HighContrast => Self::ThreatSilhouette,
            Self::ThreatSilhouette => Self::HeatTrail,
            Self::HeatTrail => Self::SonarEcho,
            Self::SonarEcho => Self::NegativeSpace,
            Self::NegativeSpace => Self::Standard,
        }
    }

    fn prev(self) -> Self {
        match self {
            Self::Standard => Self::NegativeSpace,
            Self::HighContrast => Self::Standard,
            Self::ThreatSilhouette => Self::HighContrast,
            Self::HeatTrail => Self::ThreatSilhouette,
            Self::SonarEcho => Self::HeatTrail,
            Self::NegativeSpace => Self::SonarEcho,
        }
    }
}

#[derive(Resource)]
struct DungeonMapProtoState {
    layout: MapLayout,
    fov: FovMode,
    variant: RenderVariant,
    elapsed: f32,
    player_x: i32,
    player_y: i32,
    discovered: Vec<bool>,
    seen_at: Vec<f32>,
}

pub struct DungeonMapPrototypePlugin;

impl Plugin for DungeonMapPrototypePlugin {
    fn build(&self, app: &mut App) {
        let (w, h) = map_size();
        app.insert_resource(DungeonMapProtoState {
            layout: MapLayout::Focus,
            fov: FovMode::Classic,
            variant: RenderVariant::Standard,
            elapsed: 0.0,
            player_x: 3,
            player_y: 3,
            discovered: vec![false; (w * h) as usize],
            seen_at: vec![-10_000.0; (w * h) as usize],
        })
        .add_systems(Startup, setup_prototype_camera)
        .add_systems(Update, (handle_input, tick_state))
        .add_systems(EguiPrimaryContextPass, draw_dungeon_map_prototype);
    }
}

fn setup_prototype_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn tick_state(time: Res<Time>, mut state: ResMut<DungeonMapProtoState>) {
    state.elapsed += time.delta_secs();
}

fn handle_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<DungeonMapProtoState>,
    mut exit: MessageWriter<AppExit>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
        return;
    }

    if keys.just_pressed(KeyCode::Digit1) {
        state.layout = MapLayout::Focus;
    }
    if keys.just_pressed(KeyCode::Digit2) {
        state.layout = MapLayout::Tactical;
    }
    if keys.just_pressed(KeyCode::Digit3) {
        state.layout = MapLayout::Minimal;
    }

    if keys.just_pressed(KeyCode::KeyF) {
        state.fov = match state.fov {
            FovMode::Classic => FovMode::Tight,
            FovMode::Tight => FovMode::Classic,
        };
    }

    if keys.just_pressed(KeyCode::KeyV) {
        state.variant = state.variant.next();
    }
    if keys.just_pressed(KeyCode::KeyB) {
        state.variant = state.variant.prev();
    }

    let mut dx = 0;
    let mut dy = 0;

    if keys.just_pressed(KeyCode::KeyW) || keys.just_pressed(KeyCode::ArrowUp) {
        dy = -1;
    } else if keys.just_pressed(KeyCode::KeyS) || keys.just_pressed(KeyCode::ArrowDown) {
        dy = 1;
    } else if keys.just_pressed(KeyCode::KeyA) || keys.just_pressed(KeyCode::ArrowLeft) {
        dx = -1;
    } else if keys.just_pressed(KeyCode::KeyD) || keys.just_pressed(KeyCode::ArrowRight) {
        dx = 1;
    }

    if dx == 0 && dy == 0 {
        return;
    }

    let nx = state.player_x + dx;
    let ny = state.player_y + dy;
    if is_walkable(nx, ny) {
        state.player_x = nx;
        state.player_y = ny;
    }
}

fn draw_dungeon_map_prototype(
    mut contexts: EguiContexts,
    mut state: ResMut<DungeonMapProtoState>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let s = style_for();

    let panel_bg = egui::Color32::from_rgb(10, 9, 10);
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(panel_bg))
        .show(ctx, |ui| {
            draw_backdrop_grid(ui, &s, state.elapsed);
            reveal_visible_tiles(&mut state);

            ui.label(
                egui::RichText::new(format!(
                    " Dungeon Map Lab  |  Layout [{}]  FOV [{}]  Variant [{}]  |  1/2/3 layout  F toggle  V/B variant  WASD/Arrows move  Esc quit",
                    state.layout.label(),
                    state.fov.label(),
                    state.variant.label(),
                ))
                .monospace()
                .size(11.0)
                .color(s.subtitle_color),
            );
            ui.separator();

            match state.layout {
                MapLayout::Focus => draw_focus_layout(ui, &s, &state),
                MapLayout::Tactical => draw_tactical_layout(ui, &s, &state),
                MapLayout::Minimal => draw_minimal_layout(ui, &s, &state),
            }
        });
}

fn draw_focus_layout(ui: &mut egui::Ui, s: &VariantStyle, state: &DungeonMapProtoState) {
    ui.columns(2, |cols| {
        draw_border_section(&mut cols[0], s, " Viewport ");
        draw_ascii_map(&mut cols[0], s, state, 16.0);

        draw_border_section(&mut cols[1], s, " Tracking ");
        cols[1].label(styled(s, "  Player lock: active", s.body_size, s.success_color));
        cols[1].label(styled(
            s,
            &format!("  Pos: ({}, {})", state.player_x, state.player_y),
            s.body_size,
            s.info_color,
        ));
        cols[1].label(styled(
            s,
            &format!("  FOV radius: {}", state.fov.radius()),
            s.small_size,
            s.subtitle_color,
        ));
        cols[1].add_space(8.0 * s.spacing);
        draw_border_section(&mut cols[1], s, " Minimap ");
        draw_ascii_map(&mut cols[1], s, state, 10.0);
    });

    ui.add_space(8.0 * s.spacing);
    draw_border_section(ui, s, " Tactical Feed ");
    ui.label(styled(s, "You hear skittering east of your position.", s.body_size, s.warn_color));
    ui.label(styled(
        s,
        "A husk shifts in and out of view at the FOV edge.",
        s.body_size,
        s.info_color,
    ));
}

fn draw_tactical_layout(ui: &mut egui::Ui, s: &VariantStyle, state: &DungeonMapProtoState) {
    draw_border_section(ui, s, " Tactical Viewport ");
    draw_ascii_map(ui, s, state, 15.0);

    ui.add_space(8.0 * s.spacing);
    ui.columns(3, |cols| {
        draw_border_section(&mut cols[0], s, " Threats ");
        cols[0].label(styled(s, "g  Husk Thrall", s.body_size, s.warn_color));
        cols[0].label(styled(s, "G  Ravager", s.body_size, s.danger_color));

        draw_border_section(&mut cols[1], s, " Terrain ");
        cols[1].label(styled(s, "# Wall", s.body_size, s.subtitle_color));
        cols[1].label(styled(s, "~ Hazard liquid", s.body_size, s.info_color));
        cols[1].label(styled(s, "^ Trap", s.body_size, s.warn_color));

        draw_border_section(&mut cols[2], s, " Readability ");
        cols[2].label(styled(s, "Bright: visible", s.small_size, s.title_color));
        cols[2].label(styled(s, "Dim: remembered", s.small_size, s.subtitle_color));
        cols[2].label(styled(s, "Blank: unknown", s.small_size, s.accent_color));
        cols[2].label(styled(s, "* Sonar/Negative are atypical", s.small_size, s.info_color));
    });
}

fn draw_minimal_layout(ui: &mut egui::Ui, s: &VariantStyle, state: &DungeonMapProtoState) {
    draw_border_section(ui, s, " Pure Map ");
    draw_ascii_map(ui, s, state, 18.0);

    ui.add_space(6.0 * s.spacing);
    ui.label(styled(
        s,
        "h j k l / arrows move  |  f fire  x examine  tab cycle target",
        s.small_size,
        s.subtitle_color,
    ));
}

fn draw_ascii_map(ui: &mut egui::Ui, s: &VariantStyle, state: &DungeonMapProtoState, glyph_size: f32) {
    let (w, h) = map_size();
    let player_glyph = if (state.elapsed * 4.0).sin() >= 0.0 {
        '@'
    } else {
        '◉'
    };

    for y in 0..h {
        ui.horizontal(|ui| {
            for x in 0..w {
                let idx = map_index(x, y, w);
                let visible = is_visible(state, x, y);
                let known = state.discovered[idx];

                let mut ch = tile_at(x, y);
                if x == state.player_x && y == state.player_y {
                    ch = player_glyph;
                }

                if !visible && !known {
                    ui.label(styled(s, " ", glyph_size, s.panel_bg));
                    continue;
                }

                let seen_age = state.elapsed - state.seen_at[idx];
                let (draw_ch, color) = variant_styled_glyph(state, ch, x, y, visible, known, seen_age, s);

                let mut rt = styled(s, &draw_ch.to_string(), glyph_size, color);
                if x == state.player_x && y == state.player_y {
                    rt = rt.background_color(egui::Color32::from_rgb(74, 28, 34));
                }
                ui.label(rt);
            }
        });
    }
}

fn variant_styled_glyph(
    state: &DungeonMapProtoState,
    ch: char,
    x: i32,
    y: i32,
    visible: bool,
    known: bool,
    seen_age: f32,
    s: &VariantStyle,
) -> (char, egui::Color32) {
    match state.variant {
        RenderVariant::Standard => {
            let color = if visible {
                glyph_color(ch, s)
            } else {
                egui::Color32::from_rgb(72, 66, 62)
            };
            (ch, color)
        }
        RenderVariant::HighContrast => {
            let color = if visible {
                match ch {
                    '#' => egui::Color32::from_rgb(244, 226, 196),
                    '.' => egui::Color32::from_rgb(166, 146, 124),
                    'g' | 'G' => egui::Color32::from_rgb(255, 76, 64),
                    '@' | '◉' => egui::Color32::from_rgb(255, 245, 194),
                    _ => glyph_color(ch, s),
                }
            } else {
                egui::Color32::from_rgb(56, 52, 49)
            };
            (ch, color)
        }
        RenderVariant::ThreatSilhouette => {
            if !visible {
                return (ch, egui::Color32::from_rgb(60, 54, 52));
            }
            let color = match ch {
                'g' => egui::Color32::from_rgb(255, 160, 80),
                'G' => egui::Color32::from_rgb(255, 60, 60),
                '@' | '◉' => egui::Color32::from_rgb(255, 247, 206),
                '#' => egui::Color32::from_rgb(110, 97, 90),
                _ => egui::Color32::from_rgb(132, 118, 110),
            };
            (ch, color)
        }
        RenderVariant::HeatTrail => {
            if visible {
                return (ch, glyph_color(ch, s));
            }
            let hot = egui::Color32::from_rgb(165, 78, 54);
            let cool = egui::Color32::from_rgb(62, 60, 70);
            let t = (seen_age / 8.0).clamp(0.0, 1.0);
            let color = lerp_color(hot, cool, t);
            (ch, color)
        }
        RenderVariant::SonarEcho => {
            if visible {
                return (ch, glyph_color(ch, s));
            }
            if !known {
                return (' ', s.panel_bg);
            }

            // Uncommon experimental mode: memory appears as pulsed echo rings.
            let dx = (x - state.player_x).abs();
            let dy = (y - state.player_y).abs();
            let ring = (dx + dy + (state.elapsed * 4.0) as i32).rem_euclid(4);
            let echo_glyph = match ring {
                0 => '.',
                1 => ':',
                2 => ';',
                _ => ',',
            };
            let color = egui::Color32::from_rgb(98, 128, 132);
            (echo_glyph, color)
        }
        RenderVariant::NegativeSpace => {
            // Uncommon experimental mode: terrain mostly disappears; entities/hazards float.
            if !visible {
                return (' ', s.panel_bg);
            }

            match ch {
                '@' | '◉' => (ch, egui::Color32::from_rgb(255, 250, 210)),
                'g' | 'G' => ('✶', egui::Color32::from_rgb(255, 92, 76)),
                '^' | '~' | '>' => (ch, glyph_color(ch, s)),
                '#' => ('·', egui::Color32::from_rgb(82, 72, 68)),
                _ => (' ', s.panel_bg),
            }
        }
    }
}

fn lerp_color(a: egui::Color32, b: egui::Color32, t: f32) -> egui::Color32 {
    let clamped = t.clamp(0.0, 1.0);
    let r = (a.r() as f32 + (b.r() as f32 - a.r() as f32) * clamped) as u8;
    let g = (a.g() as f32 + (b.g() as f32 - a.g() as f32) * clamped) as u8;
    let bch = (a.b() as f32 + (b.b() as f32 - a.b() as f32) * clamped) as u8;
    egui::Color32::from_rgb(r, g, bch)
}

fn glyph_color(ch: char, s: &VariantStyle) -> egui::Color32 {
    match ch {
        '#' => egui::Color32::from_rgb(120, 106, 96),
        '.' => s.subtitle_color,
        '@' | '◉' => s.title_color,
        'g' => s.warn_color,
        'G' => s.danger_color,
        '~' => s.info_color,
        '^' => s.warn_color,
        '>' => s.success_color,
        '+' => s.accent_color,
        _ => s.subtitle_color,
    }
}

fn draw_backdrop_grid(ui: &mut egui::Ui, s: &VariantStyle, t: f32) {
    let rect = ui.max_rect();
    let painter = ui.painter();
    let step = 34.0;
    let stroke = egui::Stroke::new(1.0, s.info_color.gamma_multiply(0.12));

    let mut x = rect.left();
    while x <= rect.right() {
        painter.line_segment([egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())], stroke);
        x += step;
    }

    let mut y = rect.top();
    while y <= rect.bottom() {
        painter.line_segment([egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)], stroke);
        y += step;
    }

    let node = s.success_color.gamma_multiply(0.22);
    let frame_phase = ((t * 2.0) as i32).rem_euclid(4) as f32;
    let mut nx = rect.left() + step;
    while nx < rect.right() {
        let mut ny = rect.top() + step;
        while ny < rect.bottom() {
            let offset = ((nx + ny) / step).rem_euclid(4.0);
            let jitter = if offset == frame_phase { 1.1 } else { 0.0 };
            painter.circle_filled(egui::pos2(nx + jitter, ny), 2.0, node);
            ny += step;
        }
        nx += step;
    }
}

fn draw_border_section(ui: &mut egui::Ui, style: &VariantStyle, label: &str) {
    let total = 42usize;
    let label_len = label.len() + 4;
    let side = (total.saturating_sub(label_len)) / 2;
    let line = format!(
        "{}{}╡ {} ╞{}{}",
        style.symbols.box_h.repeat(side),
        if side > 0 { "" } else { "" },
        label,
        if side > 0 { "" } else { "" },
        style.symbols.box_h.repeat(side),
    );

    ui.label(
        egui::RichText::new(line)
            .monospace()
            .size(style.small_size)
            .color(style.accent_color),
    );
}

fn styled(style: &VariantStyle, text: &str, size: f32, color: egui::Color32) -> egui::RichText {
    let mut rt = egui::RichText::new(text).size(size).color(color);
    if style.mono_all {
        rt = rt.monospace();
    }
    rt
}

fn reveal_visible_tiles(state: &mut DungeonMapProtoState) {
    let (w, h) = map_size();
    for y in 0..h {
        for x in 0..w {
            if is_visible(state, x, y) {
                let idx = map_index(x, y, w);
                state.discovered[idx] = true;
                state.seen_at[idx] = state.elapsed;
            }
        }
    }
}

fn is_visible(state: &DungeonMapProtoState, x: i32, y: i32) -> bool {
    let dx = x - state.player_x;
    let dy = y - state.player_y;
    let r = state.fov.radius();

    // Circular-ish field with a small directional softness on diagonals.
    let dist2 = dx * dx + dy * dy;
    let radial = dist2 <= r * r;
    let diamond_soft = dx.abs() + dy.abs() <= r + 2;
    radial && diamond_soft
}

fn map_size() -> (i32, i32) {
    (MAP_ROWS[0].chars().count() as i32, MAP_ROWS.len() as i32)
}

fn map_index(x: i32, y: i32, width: i32) -> usize {
    (y * width + x) as usize
}

fn tile_at(x: i32, y: i32) -> char {
    if x < 0 || y < 0 {
        return '#';
    }
    let (w, h) = map_size();
    if x >= w || y >= h {
        return '#';
    }
    MAP_ROWS[y as usize].chars().nth(x as usize).unwrap_or('#')
}

fn is_walkable(x: i32, y: i32) -> bool {
    !matches!(tile_at(x, y), '#')
}
