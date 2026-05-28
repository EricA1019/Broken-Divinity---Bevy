//! Unified UX prototype — merges dungeon style and colony layouts into one surface.
//!
//! Run with: `cargo run --bin ux_unified_prototype`
//!
//! Controls:
//!   M            — Main Menu screen
//!   D            — Dungeon screen (Echo Grid, Ember palette)
//!   C            — Colony screen (last-active colony layout)
//!   1/2/3/4/5/6  — Colony screen with specific layout
//!   Tab          — cycle focused building (colony only)
//!   WASD/Arrow   — move player (dungeon only)
//!   Esc          — quit

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use super::ux_colony_prototype as col;
use super::ux_dungeon_style_prototype as dng;
use super::ux_style_contract::style_for;

// ── unified screen enum ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UnifiedScreen {
    MainMenu,
    Dungeon,
    Colony,
}

impl UnifiedScreen {
    fn label(self, state: &UnifiedState) -> String {
        match self {
            Self::MainMenu => "Main Menu".into(),
            Self::Dungeon => "Dungeon (Echo Grid)".into(),
            Self::Colony => format!("Colony [{}]", state.colony_layout.label()),
        }
    }
}

// ── unified state resource ───────────────────────────────────────────────────

#[derive(Resource)]
struct UnifiedState {
    screen: UnifiedScreen,
    elapsed: f32,
    // dungeon fields
    player_x: i32,
    player_y: i32,
    // colony fields
    colony_layout: col::ColonyLayout,
    selected_building: usize,
}

// ── plugin ───────────────────────────────────────────────────────────────────

pub struct UnifiedPrototypePlugin;

impl Plugin for UnifiedPrototypePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(UnifiedState {
            screen: UnifiedScreen::MainMenu,
            elapsed: 0.0,
            player_x: 3,
            player_y: 3,
            colony_layout: col::ColonyLayout::SettlementView,
            selected_building: 0,
        })
        .add_systems(Startup, setup_camera)
        .add_systems(Update, (tick, handle_input))
        .add_systems(EguiPrimaryContextPass, draw_unified_prototype);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn tick(time: Res<Time>, mut state: ResMut<UnifiedState>) {
    state.elapsed += time.delta_secs();
}

// ── input routing ────────────────────────────────────────────────────────────

fn handle_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<UnifiedState>,
    mut exit: MessageWriter<AppExit>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
        return;
    }

    // screen switching
    if keys.just_pressed(KeyCode::KeyM) {
        state.screen = UnifiedScreen::MainMenu;
    }
    if keys.just_pressed(KeyCode::KeyD) {
        state.screen = UnifiedScreen::Dungeon;
    }
    if keys.just_pressed(KeyCode::KeyC) {
        state.screen = UnifiedScreen::Colony;
    }
    if keys.just_pressed(KeyCode::Digit1) {
        state.screen = UnifiedScreen::Colony;
        state.colony_layout = col::ColonyLayout::SettlementView;
    }
    if keys.just_pressed(KeyCode::Digit2) {
        state.screen = UnifiedScreen::Colony;
        state.colony_layout = col::ColonyLayout::WorkPriorities;
    }
    if keys.just_pressed(KeyCode::Digit3) {
        state.screen = UnifiedScreen::Colony;
        state.colony_layout = col::ColonyLayout::DistrictOps;
    }
    if keys.just_pressed(KeyCode::Digit4) {
        state.screen = UnifiedScreen::Colony;
        state.colony_layout = col::ColonyLayout::SelectionMode;
    }
    if keys.just_pressed(KeyCode::Digit5) {
        state.screen = UnifiedScreen::Colony;
        state.colony_layout = col::ColonyLayout::BuildMode;
    }
    if keys.just_pressed(KeyCode::Digit6) {
        state.screen = UnifiedScreen::Colony;
        state.colony_layout = col::ColonyLayout::CommandCenter;
    }

    // colony: cycle building
    if matches!(state.screen, UnifiedScreen::Colony)
        && keys.just_pressed(KeyCode::Tab)
    {
        state.selected_building = (state.selected_building + 1) % 6;
    }

    // dungeon: player movement
    if matches!(state.screen, UnifiedScreen::Dungeon) {
        let mut dx = 0i32;
        let mut dy = 0i32;
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
            if dng::is_walkable(nx, ny) {
                state.player_x = nx;
                state.player_y = ny;
            }
        }
    }
}

// ── main draw dispatch ───────────────────────────────────────────────────────

fn draw_unified_prototype(mut contexts: EguiContexts, state: Res<UnifiedState>) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let s = style_for();
    let palette = dng::ember_palette();

    let bg = match state.screen {
        UnifiedScreen::MainMenu | UnifiedScreen::Dungeon => palette.background,
        UnifiedScreen::Colony => s.panel_bg,
    };

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(bg))
        .show(ctx, |ui| {
            // backdrop
            match state.screen {
                UnifiedScreen::MainMenu | UnifiedScreen::Dungeon => {
                    dng_backdrop(ui, &palette, state.elapsed);
                }
                UnifiedScreen::Colony => {
                    col::draw_backdrop(ui, &s, state.elapsed);
                }
            }

            // header
            let header_color = match state.screen {
                UnifiedScreen::MainMenu | UnifiedScreen::Dungeon => palette.ui_subtle,
                UnifiedScreen::Colony => s.subtitle_color,
            };
            ui.label(
                egui::RichText::new(format!(
                    " Unified UX Proto  |  Screen [{}]  |  M main-menu  D dungeon  C colony  1-6 colony layouts  Tab/WASD  Esc quit",
                    state.screen.label(&state)
                ))
                .monospace()
                .size(11.0)
                .color(header_color),
            );
            ui.separator();

            // screen dispatch
            match state.screen {
                UnifiedScreen::MainMenu => {
                    dng::draw_main_menu(ui, &s, &palette);
                    ui.add_space(8.0 * s.spacing);
                    dng::draw_legend(ui, &s, &palette);
                }
                UnifiedScreen::Dungeon => {
                    dng::draw_border(ui, &palette, " Dungeon Canvas :: Echo Grid ");
                    let dng_state = dng::DungeonStyleState {
                        screen: dng::ProtoScreen::Dungeon,
                        elapsed: state.elapsed,
                        player_x: state.player_x,
                        player_y: state.player_y,
                    };
                    dng::draw_dungeon(ui, &s, &palette, &dng_state);
                    ui.add_space(8.0 * s.spacing);
                    dng::draw_legend(ui, &s, &palette);
                }
                UnifiedScreen::Colony => {
                    let col_state = col::ColonyProtoState {
                        layout: state.colony_layout,
                        selected_building: state.selected_building,
                        elapsed: state.elapsed,
                    };
                    match state.colony_layout {
                        col::ColonyLayout::SettlementView => col::draw_settlement_view(ui, &s, &col_state),
                        col::ColonyLayout::WorkPriorities => col::draw_work_priorities(ui, &s, &col_state),
                        col::ColonyLayout::DistrictOps => col::draw_district_ops(ui, &s, &col_state),
                        col::ColonyLayout::SelectionMode => col::draw_selection_mode(ui, &s, &col_state),
                        col::ColonyLayout::BuildMode => col::draw_build_mode(ui, &s, &col_state),
                        col::ColonyLayout::CommandCenter => col::draw_command_center(ui, &s, &col_state),
                    }
                    ui.add_space(8.0 * s.spacing);
                    col::draw_bottom_command_bar(ui, &s);
                }
            }
        });
}

// ── dungeon backdrop (replicated; uses DungeonPalette) ────────────────────────

fn dng_backdrop(ui: &mut egui::Ui, palette: &dng::DungeonPalette, t: f32) {
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