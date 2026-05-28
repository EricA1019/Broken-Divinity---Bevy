//! Unified UX prototype — merges dungeon style and colony layouts into one surface.
//!
//! Run with: `cargo run --bin ux_unified_prototype`
//!
//! Controls:
//!   M            — Main Menu screen
//!   D            — Dungeon screen (Echo Grid, Ember palette)
//!   C            — Colony screen (last-active colony layout)
//!   O            — Overworld mission board screen
//!   P            — Character dossier (stats/progression)
//!   I            — Inventory + Equipment screen
//!   1/2/3/4/5/6  — Colony screen with specific layout
//!   Tab          — cycle focused building / expedition report
//!   R            — reset expedition report (overworld)
//!   WASD/Arrow   — move player (dungeon only)
//!   Esc          — quit

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use super::ux_colony_prototype as col;
use super::ux_dungeon_style_prototype as dng;
use super::ux_inventory_equipment_prototype as inv;
use super::ux_overworld_prototype as ow;
use super::ux_style_contract::style_for;

// ── unified screen enum ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UnifiedScreen {
    MainMenu,
    Dungeon,
    Colony,
    Overworld,
    Dossier,
    InventoryEquipment,
}

impl UnifiedScreen {
    fn label(self, state: &UnifiedState) -> String {
        match self {
            Self::MainMenu => "Main Menu".into(),
            Self::Dungeon => "Dungeon (Echo Grid)".into(),
            Self::Colony => format!("Colony [{}]", state.colony_layout.label()),
            Self::Overworld => "Overworld (Mission Board)".into(),
            Self::Dossier => format!("Dossier [{}]", state.dossier_tab.label()),
            Self::InventoryEquipment => "Inventory + Equipment".into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DossierTab {
    Summary,
    Virtues,
    Proficiencies,
    Perks,
    Kleos,
}

impl DossierTab {
    fn label(self) -> &'static str {
        match self {
            Self::Summary => "Summary",
            Self::Virtues => "Virtues",
            Self::Proficiencies => "Proficiencies",
            Self::Perks => "Perks",
            Self::Kleos => "Kleos",
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
    // overworld fields
    overworld_focus: usize,
    // dossier fields
    dossier_tab: DossierTab,
    // inventory fields
    inventory_state: inv::InventoryProtoState,
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
            overworld_focus: 1,
            dossier_tab: DossierTab::Summary,
            inventory_state: inv::inventory_seed_state(),
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
    if keys.just_pressed(KeyCode::KeyO) {
        state.screen = UnifiedScreen::Overworld;
    }
    if keys.just_pressed(KeyCode::KeyP) {
        state.screen = UnifiedScreen::Dossier;
    }
    if keys.just_pressed(KeyCode::KeyI) {
        state.screen = UnifiedScreen::InventoryEquipment;
    }

    if matches!(state.screen, UnifiedScreen::Dossier) {
        if keys.just_pressed(KeyCode::Digit1) {
            state.dossier_tab = DossierTab::Summary;
        }
        if keys.just_pressed(KeyCode::Digit2) {
            state.dossier_tab = DossierTab::Virtues;
        }
        if keys.just_pressed(KeyCode::Digit3) {
            state.dossier_tab = DossierTab::Proficiencies;
        }
        if keys.just_pressed(KeyCode::Digit4) {
            state.dossier_tab = DossierTab::Perks;
        }
        if keys.just_pressed(KeyCode::Digit5) {
            state.dossier_tab = DossierTab::Kleos;
        }
    }
    if !matches!(state.screen, UnifiedScreen::Dossier) && keys.just_pressed(KeyCode::Digit1) {
        state.screen = UnifiedScreen::Colony;
        state.colony_layout = col::ColonyLayout::SettlementView;
    }
    if !matches!(state.screen, UnifiedScreen::Dossier) && keys.just_pressed(KeyCode::Digit2) {
        state.screen = UnifiedScreen::Colony;
        state.colony_layout = col::ColonyLayout::WorkPriorities;
    }
    if !matches!(state.screen, UnifiedScreen::Dossier) && keys.just_pressed(KeyCode::Digit3) {
        state.screen = UnifiedScreen::Colony;
        state.colony_layout = col::ColonyLayout::DistrictOps;
    }
    if !matches!(state.screen, UnifiedScreen::Dossier) && keys.just_pressed(KeyCode::Digit4) {
        state.screen = UnifiedScreen::Colony;
        state.colony_layout = col::ColonyLayout::SelectionMode;
    }
    if !matches!(state.screen, UnifiedScreen::Dossier) && keys.just_pressed(KeyCode::Digit5) {
        state.screen = UnifiedScreen::Colony;
        state.colony_layout = col::ColonyLayout::BuildMode;
    }
    if !matches!(state.screen, UnifiedScreen::Dossier) && keys.just_pressed(KeyCode::Digit6) {
        state.screen = UnifiedScreen::Colony;
        state.colony_layout = col::ColonyLayout::CommandCenter;
    }

    // colony: cycle building
    if matches!(state.screen, UnifiedScreen::Colony)
        && keys.just_pressed(KeyCode::Tab)
    {
        state.selected_building = (state.selected_building + 1) % 6;
    }

    if matches!(state.screen, UnifiedScreen::Overworld)
        && keys.just_pressed(KeyCode::Tab)
    {
        state.overworld_focus += 1;
        if state.overworld_focus >= ow::NODE_COUNT {
            state.overworld_focus = 1;
        }
    }
    if matches!(state.screen, UnifiedScreen::Overworld)
        && keys.just_pressed(KeyCode::KeyR)
    {
        state.overworld_focus = 1;
    }

    if matches!(state.screen, UnifiedScreen::InventoryEquipment) {
        inv::handle_inventory_equipment_input(&keys, &mut state.inventory_state);
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

fn draw_unified_prototype(mut contexts: EguiContexts, mut state: ResMut<UnifiedState>) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let s = style_for();
    let palette = dng::ember_palette();

    let bg = match state.screen {
        UnifiedScreen::MainMenu | UnifiedScreen::Dungeon => palette.background,
        UnifiedScreen::Colony | UnifiedScreen::Overworld | UnifiedScreen::Dossier | UnifiedScreen::InventoryEquipment => s.panel_bg,
    };

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(bg))
        .show(ctx, |ui| {
            // backdrop
            match state.screen {
                UnifiedScreen::MainMenu | UnifiedScreen::Dungeon => {
                    dng_backdrop(ui, &palette, state.elapsed);
                }
                UnifiedScreen::Colony | UnifiedScreen::Overworld | UnifiedScreen::Dossier | UnifiedScreen::InventoryEquipment => {
                    col::draw_backdrop(ui, &s, state.elapsed);
                }
            }

            // header
            let header_color = match state.screen {
                UnifiedScreen::MainMenu | UnifiedScreen::Dungeon => palette.ui_subtle,
                UnifiedScreen::Colony | UnifiedScreen::Overworld | UnifiedScreen::Dossier | UnifiedScreen::InventoryEquipment => s.subtitle_color,
            };
            ui.label(
                egui::RichText::new(format!(
                    " Unified UX Proto  |  Screen [{}]  |  M main-menu  D dungeon  C colony  O overworld  P dossier  I inventory  1-6 tabs/layouts  Tab/R/WASD  Esc quit",
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
                UnifiedScreen::Overworld => {
                    let ow_state = ow::OverworldProtoState {
                        focus_node: state.overworld_focus,
                        elapsed: state.elapsed,
                    };
                    ow::draw_terrain_tile(ui, &s, &ow_state);
                }
                UnifiedScreen::Dossier => {
                    draw_dossier_sheet(ui, &s, &state);
                }
                UnifiedScreen::InventoryEquipment => {
                    inv::draw_inventory_equipment_content(ui, &s, &mut state.inventory_state);
                }
            }
        });
}

fn draw_dossier_sheet(ui: &mut egui::Ui, s: &super::ux_style_contract::VariantStyle, state: &UnifiedState) {
    ui.label(dng::styled(
        s,
        " DOSSIER  [1 Summary] [2 Virtues] [3 Proficiencies] [4 Perks] [5 Kleos] ",
        s.body_size,
        s.title_color,
    ));
    ui.separator();

    ui.horizontal(|ui| {
        ui.label(dng::styled(s, " Name: Brother Marcus ", s.body_size, s.accent_color));
        ui.separator();
        ui.label(dng::styled(s, " State: Wounded, burdened, alert ", s.body_size, s.subtitle_color));
    });

    ui.horizontal(|ui| {
        ui.label(dng::styled(s, " AP 2/2 ", s.small_size, s.success_color));
        ui.separator();
        ui.label(dng::styled(s, " Armor Intact ", s.small_size, s.info_color));
        ui.separator();
        ui.label(dng::styled(s, " Ammo 12 reserve 60 ", s.small_size, s.subtitle_color));
        ui.separator();
        ui.label(dng::styled(s, " Exposure 85 ", s.small_size, s.warn_color));
        ui.separator();
        ui.label(dng::styled(s, " Floor 2 / 4 ", s.small_size, s.subtitle_color));
    });

    ui.add_space(6.0 * s.spacing);

    match state.dossier_tab {
        DossierTab::Summary => {
            ui.label(dng::styled(s, " Current Read ", s.heading_size, s.title_color));
            ui.label(dng::styled(s, " - Strongest lane: Prudence + Ranged Training", s.body_size, s.accent_color));
            ui.label(dng::styled(s, " - Weakest lane: Ritecraft under stress", s.body_size, s.warn_color));
            ui.label(dng::styled(s, " - Active myth: Noticed", s.body_size, s.info_color));
            ui.label(dng::styled(s, " - Current pressure: Wound and rising exposure", s.body_size, s.danger_color));
        }
        DossierTab::Virtues => {
            ui.label(dng::styled(s, " Temperance   2   steadies fear, corruption, overcommitment", s.body_size, s.info_color));
            ui.label(dng::styled(s, " Justice      1   governs oath, obligation, lawful force", s.body_size, s.info_color));
            ui.label(dng::styled(s, " Prudence     3   sharpest current virtue", s.body_size, s.accent_color));
            ui.label(dng::styled(s, " Fortitude    2   keeps the body moving through pain", s.body_size, s.info_color));
            ui.label(dng::styled(s, " Thumos       1   low zeal, low reckless pressure", s.body_size, s.info_color));
            ui.label(dng::styled(s, " Metis        1   narrow cunning under stress", s.body_size, s.info_color));
        }
        DossierTab::Proficiencies => {
            ui.label(dng::styled(s, " Melee Training      6   familiar", s.body_size, s.subtitle_color));
            ui.label(dng::styled(s, " Ranged Training    12   trained", s.body_size, s.accent_color));
            ui.label(dng::styled(s, " Quiet Movement      0   untrained", s.body_size, s.warn_color));
            ui.label(dng::styled(s, " Repair              6   familiar", s.body_size, s.subtitle_color));
            ui.label(dng::styled(s, " Medicine           12   trained", s.body_size, s.accent_color));
            ui.label(dng::styled(s, " Ritecraft           0   untrained", s.body_size, s.warn_color));
            ui.add_space(4.0 * s.spacing);
            ui.label(dng::styled(s, " Action rating = proficiency + (virtue rank * 5) + gear + perk", s.small_size, s.subtitle_color));
        }
        DossierTab::Perks => {
            ui.label(dng::styled(s, " Unlocked", s.heading_size, s.title_color));
            ui.label(dng::styled(s, " - Steady Hands (Temperance + Ranged)", s.body_size, s.accent_color));
            ui.label(dng::styled(s, " - Hold Fast (Fortitude + Melee)", s.body_size, s.info_color));
            ui.add_space(4.0 * s.spacing);
            ui.label(dng::styled(s, " Near-term gates", s.heading_size, s.title_color));
            ui.label(dng::styled(s, " - T1 Ritecraft lane: Justice 2 + Ritecraft 10", s.body_size, s.warn_color));
            ui.label(dng::styled(s, " - T2 Marksman lane: Prudence 3 + Ranged 18", s.body_size, s.warn_color));
            ui.label(dng::styled(s, " - T3 Signature: Virtue 4 + Proficiency 24 + Kleos 25", s.body_size, s.subtitle_color));
        }
        DossierTab::Kleos => {
            ui.label(dng::styled(s, " Kleos: 14 (Noticed)", s.heading_size, s.accent_color));
            ui.label(dng::styled(s, " Public myth: recognized by minor factions", s.body_size, s.info_color));
            ui.add_space(4.0 * s.spacing);
            ui.label(dng::styled(s, " Standing", s.heading_size, s.title_color));
            ui.label(dng::styled(s, " - Settlement: Trusted", s.body_size, s.success_color));
            ui.label(dng::styled(s, " - Michael's Host: Watched", s.body_size, s.warn_color));
            ui.label(dng::styled(s, " - Older Powers: Rumored meddler", s.body_size, s.warn_color));
            ui.label(dng::styled(s, " - Active vow: No child left outside", s.body_size, s.subtitle_color));
        }
    }

    ui.add_space(6.0 * s.spacing);
    ui.separator();
    ui.label(dng::styled(
        s,
        " 1-5 tab  M/D/C/O/P screens  Esc quit ",
        s.small_size,
        s.subtitle_color,
    ));
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