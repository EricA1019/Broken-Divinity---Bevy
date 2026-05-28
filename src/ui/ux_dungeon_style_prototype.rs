//! Dungeon styling prototype lab.
//!
//! This is intentionally separate from other prototype surfaces.
//! It focuses only on dungeon map styling variants.
//!
//! Run with: `cargo run --bin ux_dungeon_style_prototype`
//!
//! Controls:
//!   M/D       — switch screen (Main Menu / Dungeon)
//!   Arrow/WASD— move player
//!   Esc       — quit

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use super::ux_style_contract::{style_for, VariantStyle};

const DUNGEON: [&str; 16] = [
    "########################################",
    "#....g.........#............#.........>#",
    "#.######.#####.#.##########.#.#######.#",
    "#.#....#.#...#.#.#........#.#.#.....#.#",
    "#.#.~~.#.#.^.#.#.#....g...#.#.#.~~~.#.#",
    "#.#....#.#...#.#.#........#.#.#.....#.#",
    "#.######.#####.#.##########.#.#####.#.#",
    "#..........#...#.....+......#.....#...#",
    "#.########.#.#########.#########.#####.#",
    "#.#......#.#.........#.#.......#.....#.#",
    "#.#.g....#.#########.#.#.#####.#####.#.#",
    "#.#......#.......#...#.#.....#.....#.#.#",
    "#.###########.###.#.##.#####.#####.#.#.#",
    "#.........G...#...#..#.....#.....#...#.#",
    "#......................................#",
    "########################################",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProtoScreen {
    MainMenu,
    Dungeon,
}

impl ProtoScreen {
    fn label(self) -> &'static str {
        match self {
            Self::MainMenu => "Main Menu",
            Self::Dungeon => "Dungeon",
        }
    }
}

#[derive(Resource)]
struct DungeonStyleState {
    screen: ProtoScreen,
    elapsed: f32,
    player_x: i32,
    player_y: i32,
}

pub struct DungeonStylePrototypePlugin;

impl Plugin for DungeonStylePrototypePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DungeonStyleState {
            screen: ProtoScreen::MainMenu,
            elapsed: 0.0,
            player_x: 3,
            player_y: 3,
        })
        .add_systems(Startup, setup_camera)
        .add_systems(Update, (tick, handle_input))
        .add_systems(EguiPrimaryContextPass, draw_dungeon_style_prototype);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn tick(time: Res<Time>, mut state: ResMut<DungeonStyleState>) {
    state.elapsed += time.delta_secs();
}

fn handle_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<DungeonStyleState>,
    mut exit: MessageWriter<AppExit>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
        return;
    }

    if keys.just_pressed(KeyCode::KeyM) {
        state.screen = ProtoScreen::MainMenu;
    }
    if keys.just_pressed(KeyCode::KeyD) {
        state.screen = ProtoScreen::Dungeon;
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

    if dx != 0 || dy != 0 {
        let nx = state.player_x + dx;
        let ny = state.player_y + dy;
        if is_walkable(nx, ny) {
            state.player_x = nx;
            state.player_y = ny;
        }
    }
}

fn draw_dungeon_style_prototype(mut contexts: EguiContexts, state: Res<DungeonStyleState>) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let s = style_for();
    let palette = ember_palette();

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(palette.background))
        .show(ctx, |ui| {
            draw_backdrop(ui, &palette, state.elapsed);

            ui.label(
                egui::RichText::new(format!(
                    " Dungeon Style Proto  |  Locked Palette [Ember]  Locked Pattern [Echo Grid]  Screen [{}]  |  M/D screens  WASD move  Esc quit",
                    state.screen.label()
                ))
                .monospace()
                .size(11.0)
                .color(palette.ui_subtle),
            );

            ui.separator();
            match state.screen {
                ProtoScreen::MainMenu => draw_main_menu(ui, &s, &palette),
                ProtoScreen::Dungeon => {
                    draw_border(ui, &palette, " Dungeon Canvas :: Echo Grid ");
                    draw_dungeon(ui, &s, &palette, &state);
                }
            }
            ui.add_space(8.0 * s.spacing);
            draw_legend(ui, &s, &palette);
        });
}

fn draw_main_menu(ui: &mut egui::Ui, s: &VariantStyle, palette: &DungeonPalette) {
    draw_border(ui, palette, " Main Menu :: Locked Direction ");

    ui.add_space(8.0 * s.spacing);
    ui.label(styled(
        s,
        "BROKEN DIVINITY",
        s.heading_size + 3.0,
        palette.player,
    ));
    ui.label(styled(
        s,
        "Ash and sigil. Hunger and iron.",
        s.body_size,
        palette.ui_subtle,
    ));
    ui.add_space(10.0 * s.spacing);

    for item in ["[ Continue ]", "[ New Expedition ]", "[ Options ]", "[ Exit ]"] {
        ui.label(styled(s, item, s.heading_size, palette.ui_accent));
    }

    ui.add_space(8.0 * s.spacing);
    ui.label(styled(
        s,
        "M = this menu, D = dungeon prototype",
        s.small_size,
        palette.ui_subtle,
    ));
}

fn draw_dungeon(
    ui: &mut egui::Ui,
    s: &VariantStyle,
    palette: &DungeonPalette,
    state: &DungeonStyleState,
) {
    let player_glyph = if (state.elapsed * 3.5).sin() > 0.0 {
        '@'
    } else {
        '*'
    };

    for (y, row) in DUNGEON.iter().enumerate() {
        ui.horizontal(|ui| {
            for (x, mut ch) in row.chars().enumerate() {
                if x as i32 == state.player_x && y as i32 == state.player_y {
                    ch = player_glyph;
                }
                let (glyph, color) = pattern_glyph(state, ch, x as i32, y as i32, palette);
                ui.label(styled(s, &glyph.to_string(), 15.0, color));
            }
        });
    }
}

fn draw_legend(ui: &mut egui::Ui, s: &VariantStyle, palette: &DungeonPalette) {
    draw_border(ui, palette, " Legend ");
    ui.label(styled(s, "# wall", s.body_size, palette.wall));
    ui.label(styled(s, ". floor", s.body_size, palette.floor));
    ui.label(styled(s, "g/G enemy", s.body_size, palette.enemy));
    ui.label(styled(s, "~ hazard", s.body_size, palette.hazard));
    ui.label(styled(s, "^ trap", s.body_size, palette.trap));
    ui.label(styled(s, "> exit", s.body_size, palette.exit));
    ui.label(styled(s, "@ player", s.body_size, palette.player));
    ui.label(styled(
        s,
        "Locked display pattern: Echo Grid",
        s.small_size,
        palette.ui_subtle,
    ));
}

fn draw_border(ui: &mut egui::Ui, palette: &DungeonPalette, label: &str) {
    let line = format!("{} {} {}", "=".repeat(18), label, "=".repeat(18));
    ui.label(
        egui::RichText::new(line)
            .monospace()
            .size(10.0)
            .color(palette.ui_accent),
    );
}

fn styled(style: &VariantStyle, text: &str, size: f32, color: egui::Color32) -> egui::RichText {
    let mut rt = egui::RichText::new(text).size(size).color(color);
    if style.mono_all {
        rt = rt.monospace();
    }
    rt
}

fn draw_backdrop(ui: &mut egui::Ui, palette: &DungeonPalette, t: f32) {
    let rect = ui.max_rect();
    let painter = ui.painter();

    let spacing = 30.0;
    let mut x = rect.left();
    while x <= rect.right() {
        painter.line_segment(
            [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
            egui::Stroke::new(1.0, palette.ui_grid.gamma_multiply(0.2)),
        );
        x += spacing;
    }

    let mut y = rect.top();
    while y <= rect.bottom() {
        painter.line_segment(
            [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
            egui::Stroke::new(1.0, palette.ui_grid.gamma_multiply(0.2)),
        );
        y += spacing;
    }

    let pulse = ((t * 2.0).sin() + 1.0) * 0.5;
    let orb_color = palette.ui_accent.gamma_multiply(0.12 + 0.12 * pulse);
    painter.circle_filled(rect.center(), 26.0, orb_color);
}

fn base_colorize(ch: char, p: &DungeonPalette) -> (char, egui::Color32) {
    match ch {
        '#' => (ch, p.wall),
        '.' => (ch, p.floor),
        'g' | 'G' => (ch, p.enemy),
        '~' => (ch, p.hazard),
        '^' => (ch, p.trap),
        '+' => (ch, p.objective),
        '>' => (ch, p.exit),
        '@' | '*' => ('@', p.player),
        _ => (ch, p.floor),
    }
}

fn pattern_glyph(
    state: &DungeonStyleState,
    ch: char,
    x: i32,
    y: i32,
    palette: &DungeonPalette,
) -> (char, egui::Color32) {
    let is_player = x == state.player_x && y == state.player_y;
    if is_player {
        return ('@', palette.player);
    }

    let visible = is_visible(state, x, y);
    if visible {
        return base_colorize(ch, palette);
    }

    let dx = (x - state.player_x).abs()
        + (y - state.player_y).abs()
        + (state.elapsed * 3.0) as i32;
    let ring = dx.rem_euclid(5);
    let echo = match ring {
        0 => '.',
        1 => ':',
        2 => ';',
        3 => ',',
        _ => ' ',
    };
    (echo, palette.ui_grid.gamma_multiply(0.9))
}

fn is_visible(state: &DungeonStyleState, x: i32, y: i32) -> bool {
    let dx = x - state.player_x;
    let dy = y - state.player_y;
    let dist2 = dx * dx + dy * dy;
    dist2 <= 64
}

struct DungeonPalette {
    background: egui::Color32,
    wall: egui::Color32,
    floor: egui::Color32,
    enemy: egui::Color32,
    hazard: egui::Color32,
    trap: egui::Color32,
    objective: egui::Color32,
    exit: egui::Color32,
    player: egui::Color32,
    ui_subtle: egui::Color32,
    ui_accent: egui::Color32,
    ui_grid: egui::Color32,
}

fn ember_palette() -> DungeonPalette {
    DungeonPalette {
        background: egui::Color32::from_rgb(14, 4, 5),
        wall: egui::Color32::from_rgb(147, 74, 63),
        floor: egui::Color32::from_rgb(112, 64, 59),
        enemy: egui::Color32::from_rgb(255, 88, 76),
        hazard: egui::Color32::from_rgb(124, 87, 155),
        trap: egui::Color32::from_rgb(240, 166, 67),
        objective: egui::Color32::from_rgb(255, 188, 76),
        exit: egui::Color32::from_rgb(118, 201, 112),
        player: egui::Color32::from_rgb(255, 230, 198),
        ui_subtle: egui::Color32::from_rgb(175, 102, 92),
        ui_accent: egui::Color32::from_rgb(244, 146, 66),
        ui_grid: egui::Color32::from_rgb(122, 66, 60),
    }
}

fn tile_at(x: i32, y: i32) -> char {
    if x < 0 || y < 0 {
        return '#';
    }
    let ux = x as usize;
    let uy = y as usize;
    if uy >= DUNGEON.len() {
        return '#';
    }
    DUNGEON[uy].chars().nth(ux).unwrap_or('#')
}

fn is_walkable(x: i32, y: i32) -> bool {
    !matches!(tile_at(x, y), '#')
}
