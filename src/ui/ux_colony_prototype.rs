//! Colony UI prototype lab focused on building readability and colonist interaction flow.
//!
//! Run with: `cargo run --bin ux_colony_prototype`
//!
//! Controls:
//!   1/2/3/4/5/6 — switch layout approach
//!   Tab     — cycle focused building
//!   Esc     — quit

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use super::ux_style_contract::{style_for, VariantStyle};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ColonyLayout {
    SettlementView,
    WorkPriorities,
    DistrictOps,
    SelectionMode,
    BuildMode,
    CommandCenter,
}

impl ColonyLayout {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::SettlementView => "Settlement View",
            Self::WorkPriorities => "Work Priorities",
            Self::DistrictOps => "District Ops",
            Self::SelectionMode => "Selection Mode",
            Self::BuildMode => "Build Mode",
            Self::CommandCenter => "Command Center",
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Building {
    id: &'static str,
    name: &'static str,
    zone: &'static str,
    workers: u8,
    cap: u8,
    status: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct Colonist {
    name: &'static str,
    task: &'static str,
    target: &'static str,
    load: &'static str,
}

const BUILDINGS: [Building; 6] = [
    Building {
        id: "GEN",
        name: "Generator",
        zone: "Core",
        workers: 2,
        cap: 3,
        status: "Stable",
    },
    Building {
        id: "MED",
        name: "Infirmary",
        zone: "Core",
        workers: 1,
        cap: 2,
        status: "Busy",
    },
    Building {
        id: "FARM",
        name: "Hydro Farm",
        zone: "South",
        workers: 3,
        cap: 3,
        status: "Peak",
    },
    Building {
        id: "SHOP",
        name: "Workshop",
        zone: "West",
        workers: 2,
        cap: 4,
        status: "Idle",
    },
    Building {
        id: "COMM",
        name: "Comms",
        zone: "North",
        workers: 1,
        cap: 2,
        status: "Scanning",
    },
    Building {
        id: "GATE",
        name: "Gatehouse",
        zone: "East",
        workers: 2,
        cap: 2,
        status: "Alert",
    },
];

const COLONISTS: [Colonist; 7] = [
    Colonist {
        name: "Rhea",
        task: "Repair",
        target: "GEN",
        load: "78%",
    },
    Colonist {
        name: "Ivo",
        task: "Treat",
        target: "MED",
        load: "64%",
    },
    Colonist {
        name: "Mara",
        task: "Harvest",
        target: "FARM",
        load: "92%",
    },
    Colonist {
        name: "Seth",
        task: "Forge",
        target: "SHOP",
        load: "48%",
    },
    Colonist {
        name: "Naya",
        task: "Relay",
        target: "COMM",
        load: "52%",
    },
    Colonist {
        name: "Bram",
        task: "Patrol",
        target: "GATE",
        load: "88%",
    },
    Colonist {
        name: "Kane",
        task: "Haul",
        target: "SHOP",
        load: "69%",
    },
];

#[derive(Resource)]
pub(crate) struct ColonyProtoState {
    pub(crate) layout: ColonyLayout,
    pub(crate) selected_building: usize,
    pub(crate) elapsed: f32,
}

pub struct ColonyPrototypePlugin;

impl Plugin for ColonyPrototypePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ColonyProtoState {
            layout: ColonyLayout::SettlementView,
            selected_building: 0,
            elapsed: 0.0,
        })
        .add_systems(Startup, setup_camera)
        .add_systems(Update, (tick, handle_input))
        .add_systems(EguiPrimaryContextPass, draw_colony_prototype);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn tick(time: Res<Time>, mut state: ResMut<ColonyProtoState>) {
    state.elapsed += time.delta_secs();
}

fn handle_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<ColonyProtoState>,
    mut exit: MessageWriter<AppExit>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
        return;
    }

    if keys.just_pressed(KeyCode::Digit1) {
        state.layout = ColonyLayout::SettlementView;
    }
    if keys.just_pressed(KeyCode::Digit2) {
        state.layout = ColonyLayout::WorkPriorities;
    }
    if keys.just_pressed(KeyCode::Digit3) {
        state.layout = ColonyLayout::DistrictOps;
    }
    if keys.just_pressed(KeyCode::Digit4) {
        state.layout = ColonyLayout::SelectionMode;
    }
    if keys.just_pressed(KeyCode::Digit5) {
        state.layout = ColonyLayout::BuildMode;
    }
    if keys.just_pressed(KeyCode::Digit6) {
        state.layout = ColonyLayout::CommandCenter;
    }

    if keys.just_pressed(KeyCode::Tab) {
        state.selected_building = (state.selected_building + 1) % BUILDINGS.len();
    }
}

fn draw_colony_prototype(mut contexts: EguiContexts, state: Res<ColonyProtoState>) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let s = style_for();

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(s.panel_bg))
        .show(ctx, |ui| {
            draw_backdrop(ui, &s, state.elapsed);

            ui.label(
                egui::RichText::new(format!(
                    " Colony Prototype  |  Layout [{}]  |  1 Settlement  2 Priorities  3 DistrictOps  4 Selection  5 BuildMode  6 CommandCenter  Tab  Esc",
                    state.layout.label()
                ))
                .monospace()
                .size(11.0)
                .color(s.subtitle_color),
            );
            ui.separator();

            match state.layout {
                ColonyLayout::SettlementView => draw_settlement_view(ui, &s, &state),
                ColonyLayout::WorkPriorities => draw_work_priorities(ui, &s, &state),
                ColonyLayout::DistrictOps => draw_district_ops(ui, &s, &state),
                ColonyLayout::SelectionMode => draw_selection_mode(ui, &s, &state),
                ColonyLayout::BuildMode => draw_build_mode(ui, &s, &state),
                ColonyLayout::CommandCenter => draw_command_center(ui, &s, &state),
            }

            ui.add_space(8.0 * s.spacing);
            draw_bottom_command_bar(ui, &s);
        });
}

pub(crate) fn draw_settlement_view(ui: &mut egui::Ui, s: &VariantStyle, state: &ColonyProtoState) {
    draw_section(ui, s, " Approach A: Map-first settlement (RimWorld-inspired) ");

    ui.columns(2, |cols| {
        draw_section(&mut cols[0], s, " Colony Map ");
        let sel = BUILDINGS[state.selected_building].id;
        for line in [
            "┌──────────────────────────────────────┐",
            "│ [COMM] ███ conduit ███ [GEN]         │",
            "│    │                 │               │",
            "│ [MED]      [STOCK]   │               │",
            "│    │          │      │               │",
            "│ [BARR] -- [SHOP] -- [GATE]           │",
            "│                 │                    │",
            "│               [FARM]                 │",
            "└──────────────────────────────────────┘",
        ] {
            cols[0].label(styled(s, line, s.body_size, s.subtitle_color));
        }
        cols[0].label(styled(
            s,
            &format!(" Selected Building: {}", sel),
            s.body_size,
            s.title_color,
        ));
        cols[0].label(styled(
            s,
            " Colonists route to selected building are emphasized in side panel.",
            s.small_size,
            s.info_color,
        ));

        draw_section(&mut cols[1], s, " Building Inspector ");
        for (idx, b) in BUILDINGS.iter().enumerate() {
            let focused = idx == state.selected_building;
            let color = if focused { s.title_color } else { status_color(s, b.status) };
            cols[1].label(styled(
                s,
                &format!(" {:<5} {:<12} [{}/{}] {}", b.id, b.name, b.workers, b.cap, b.status),
                s.body_size,
                color,
            ));
        }

        cols[1].add_space(6.0 * s.spacing);
        draw_section(&mut cols[1], s, " Colonists On Route ");
        for c in COLONISTS {
            let linked = c.target == sel;
            let color = if linked {
                s.success_color
            } else {
                s.subtitle_color.gamma_multiply(0.55)
            };
            let marker = if linked { ">>" } else { ".." };
            cols[1].label(styled(
                s,
                &format!(" {} {:<6} {} {:<5} {:<10}", marker, c.name, c.target, c.load, c.task),
                s.small_size,
                color,
            ));
        }
    });
}

pub(crate) fn draw_work_priorities(ui: &mut egui::Ui, s: &VariantStyle, state: &ColonyProtoState) {
    draw_section(ui, s, " Approach B: Job priority matrix (Song of Syx-like macro control) ");

    let selected = BUILDINGS[state.selected_building].id;
    draw_section(ui, s, " Colonist  Build  Farm  Guard  Heal  Haul  Craft  Target ");
    for c in COLONISTS {
        let focus = if c.target == selected { s.title_color } else { s.subtitle_color };
        let row = format!(
            " {:<8}   2      3      2      1     2      3    {:<5}",
            c.name, c.target
        );
        ui.label(styled(s, &row, s.body_size, focus));
    }

    ui.add_space(6.0 * s.spacing);
    draw_section(ui, s, " Priority Notes ");
    ui.label(styled(
        s,
        " - Lower number = higher priority (classic colony-management readability)",
        s.small_size,
        s.info_color,
    ));
    ui.label(styled(
        s,
        " - Selected building rows are highlighted for immediate labor tuning",
        s.small_size,
        s.success_color,
    ));
}

pub(crate) fn draw_district_ops(ui: &mut egui::Ui, s: &VariantStyle, state: &ColonyProtoState) {
    draw_section(ui, s, " Approach C: District + throughput overlay (macro readability) ");

    let target = BUILDINGS[state.selected_building];
    ui.columns(2, |cols| {
        draw_section(&mut cols[0], s, " District Heat ");
        for line in [
            " NORTH [COMM]  pop 12  stress 18% ",
            " CORE  [GEN/MED] pop 24 stress 22% ",
            " WEST  [SHOP] pop 17 stress 31% ",
            " SOUTH [FARM] pop 19 stress 14% ",
            " EAST  [GATE] pop 15 stress 36% ",
        ] {
            cols[0].label(styled(s, line, s.body_size, s.subtitle_color));
        }

        cols[0].add_space(5.0 * s.spacing);
        cols[0].label(styled(
            s,
            &format!(" Focus district by building: {} ({})", target.name, target.zone),
            s.small_size,
            s.title_color,
        ));

        draw_section(&mut cols[1], s, " Throughput / Bottlenecks ");
        cols[1].label(styled(s, " Food chain: FARM => STOCK => BARR", s.body_size, s.success_color));
        cols[1].label(styled(s, " Craft chain: SHOP => GATE ammo", s.body_size, s.info_color));
        cols[1].label(styled(s, " Med chain: FARM herbs => MED", s.body_size, s.info_color));
        cols[1].label(styled(s, " ALERT: East gate queue saturation", s.body_size, s.warn_color));

        cols[1].add_space(6.0 * s.spacing);
        draw_section(&mut cols[1], s, " Colonist Paths ");
        for c in COLONISTS {
            let linked = c.target == target.id;
            let color = if linked { s.success_color } else { s.subtitle_color.gamma_multiply(0.65) };
            let arrow = if linked { "===>" } else { " -->" };
            cols[1].label(styled(
                s,
                &format!(" {:<6} {} {:<5}  task:{}", c.name, arrow, c.target, c.task),
                s.body_size,
                color,
            ));
        }
    });
}

pub(crate) fn draw_selection_mode(ui: &mut egui::Ui, s: &VariantStyle, state: &ColonyProtoState) {
    draw_section(ui, s, " Approach D: Selection-gizmo flow (object commands first) ");
    let b = BUILDINGS[state.selected_building];

    ui.columns(3, |cols| {
        draw_section(&mut cols[0], s, " Selected Object ");
        cols[0].label(styled(s, &format!(" Name: {}", b.name), s.body_size, s.title_color));
        cols[0].label(styled(s, &format!(" Zone: {}", b.zone), s.body_size, s.info_color));
        cols[0].label(styled(
            s,
            &format!(" Staffing: {}/{}", b.workers, b.cap),
            s.body_size,
            s.success_color,
        ));
        cols[0].label(styled(
            s,
            &format!(" Status: {}", b.status),
            s.body_size,
            status_color(s, b.status),
        ));
        cols[0].add_space(6.0 * s.spacing);
        cols[0].label(styled(s, " Entity Gizmos:", s.small_size, s.accent_color));
        cols[0].label(styled(s, " [Reassign] [Suspend] [Repair]", s.body_size, s.warn_color));
        cols[0].label(styled(s, " [Expand Zone] [Set Target]", s.body_size, s.warn_color));

        draw_section(&mut cols[1], s, " Assigned Colonists ");
        for c in COLONISTS {
            if c.target == b.id {
                cols[1].label(styled(
                    s,
                    &format!(" {:<6} {:<10} load {}", c.name, c.task, c.load),
                    s.body_size,
                    s.success_color,
                ));
            }
        }

        cols[1].add_space(6.0 * s.spacing);
        cols[1].label(styled(
            s,
            " Intent: the player clicks a building then gets immediate, concrete commands.",
            s.small_size,
            s.subtitle_color,
        ));

        draw_section(&mut cols[2], s, " Context Alerts ");
        cols[2].label(styled(s, " ! Missing components for SHOP", s.body_size, s.warn_color));
        cols[2].label(styled(s, " ! GATE ammo under target", s.body_size, s.danger_color));
        cols[2].label(styled(s, " i MED queue is growing", s.body_size, s.info_color));
        cols[2].label(styled(s, " i FARM output healthy", s.body_size, s.success_color));
    });
}

pub(crate) fn draw_build_mode(ui: &mut egui::Ui, s: &VariantStyle, state: &ColonyProtoState) {
    draw_section(ui, s, " Approach E: Build mode with room designation from placed stations ");

    let station = BUILDINGS[state.selected_building];

    ui.columns(2, |cols| {
        draw_section(&mut cols[0], s, " Construction Canvas ");
        cols[0].label(styled(
            s,
            " Scrollable map prototype: larger colony footprint, buildable room shells, station-driven room identity.",
            s.small_size,
            s.info_color,
        ));
        egui::ScrollArea::both()
            .auto_shrink([false, false])
            .max_width(760.0)
            .max_height(380.0)
            .show(&mut cols[0], |ui| {
                let canvas = [
                    "┌────────────────────────────────────────────────────────────────────────────┐",
                    "│ ╔═════════════════D═════════════════╗      ╔══════════════D══════════════╗   │",
                    "│ ║.................................║      ║.............................║   │",
                    "│ ║....S............................║      ║.............................║   │",
                    "│ ║.................................║      ║.........S...................║   │",
                    "│ ║.................................║      ║.............................║   │",
                    "│ ╚═════════════════════════════════╝      ╚═════════════════════════════╝   │",
                    "│                                                                            │",
                    "│                  ======= main haul road =======                            │",
                    "│                                                                            │",
                    "│ ╔════════════════════════════╗    ╔════════════════════════════╗          │",
                    "│ ║............................║    ║............................║          │",
                    "│ ║............................║    ║............................║          │",
                    "│ ║.............S..............D════D.............S..............║          │",
                    "│ ║............................║    ║............................║          │",
                    "│ ║............................║    ║............................║          │",
                    "│ ╚════════════════════════════╝    ╚════════════════════════════╝          │",
                    "│                                                                            │",
                    "│          ╔════════════════D════════════════════╗                          │",
                    "│          ║.................................║                           │",
                    "│          ║.................................║                           │",
                    "│          ║..............S..................║                           │",
                    "│          ║.................................║                           │",
                    "│          ╚═════════════════════════════════╝                           │",
                    "│                                                                            │",
                    "│ Legend: box walls  D door aperture  S station  = route / corridor           │",
                    "└────────────────────────────────────────────────────────────────────────────┘",
                ]
                .join("\n");

                ui.label(styled(s, &canvas, s.body_size, s.subtitle_color));
            });
        cols[0].add_space(4.0 * s.spacing);
        cols[0].label(styled(
            s,
            "Idea: player drafts enclosure first, then drops a station inside.",
            s.small_size,
            s.info_color,
        ));
        cols[0].label(styled(
            s,
            "Once enclosed + accessible, room type resolves from the station.",
            s.small_size,
            s.success_color,
        ));

        draw_section(&mut cols[1], s, " Designation Flow ");
        cols[1].label(styled(s, " 1. Draw walls / corners", s.body_size, s.warn_color));
        cols[1].label(styled(s, " 2. Add at least one door", s.body_size, s.warn_color));
        cols[1].label(styled(s, " 3. Place station object", s.body_size, s.warn_color));
        cols[1].label(styled(s, " 4. Room is validated", s.body_size, s.success_color));
        cols[1].label(styled(s, " 5. Station designates room role", s.body_size, s.title_color));

        cols[1].add_space(6.0 * s.spacing);
        draw_section(&mut cols[1], s, " Station Resolver ");
        cols[1].label(styled(
            s,
            &format!(" Selected station: {} ({})", station.name, station.id),
            s.body_size,
            s.title_color,
        ));
        cols[1].label(styled(s, " If room closed + door present => valid room", s.small_size, s.info_color));
        cols[1].label(styled(s, " Workshop bench => Workshop", s.body_size, s.subtitle_color));
        cols[1].label(styled(s, " Pantry rack => Storehouse", s.body_size, s.subtitle_color));
        cols[1].label(styled(s, " Stove/hearth => Kitchen", s.body_size, s.subtitle_color));
        cols[1].label(styled(s, " Cot cluster => Barracks", s.body_size, s.subtitle_color));

        cols[1].add_space(6.0 * s.spacing);
        draw_section(&mut cols[1], s, " Readability Goals ");
        cols[1].label(styled(s, " - Keep architecture visible on map", s.small_size, s.info_color));
        cols[1].label(styled(s, " - Separate structure from designation", s.small_size, s.info_color));
        cols[1].label(styled(s, " - Colonists interact with station, not abstract room tag", s.small_size, s.success_color));
        cols[1].label(styled(s, " - Consistent wall glyph family should read as one construction language", s.small_size, s.success_color));
    });
}

// ── sparkline data: 12 ticks of fake history ─────────────────────────────
const FOOD_TREND:    [f32; 12] = [72.,68.,74.,71.,75.,78.,76.,80.,77.,82.,85.,83.];
const MORALE_TREND:  [f32; 12] = [60.,62.,59.,55.,50.,48.,52.,54.,58.,56.,60.,57.];
const STRESS_TREND:  [f32; 12] = [22.,24.,27.,30.,33.,31.,28.,26.,25.,28.,30.,32.];
const THREAT_TREND:  [f32; 12] = [10.,10.,12.,15.,18.,22.,20.,18.,16.,20.,24.,26.];

fn draw_sparkline(
    ui: &mut egui::Ui,
    data: &[f32],
    color: egui::Color32,
    width: f32,
    height: f32,
) {
    let (_, rect) = ui.allocate_space(egui::vec2(width, height));
    let painter = ui.painter_at(rect);

    let min = data.iter().cloned().fold(f32::MAX, f32::min);
    let max = data.iter().cloned().fold(f32::MIN, f32::max);
    let range = (max - min).max(1.0);
    let n = data.len();

    let pts: Vec<egui::Pos2> = data
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            let x = rect.left() + (i as f32 / (n - 1) as f32) * rect.width();
            let y = rect.bottom() - ((v - min) / range) * rect.height();
            egui::pos2(x, y)
        })
        .collect();

    for w in pts.windows(2) {
        painter.line_segment([w[0], w[1]], egui::Stroke::new(1.5, color));
    }
    // latest value dot
    if let Some(&last) = pts.last() {
        painter.circle_filled(last, 3.0, color);
    }
}

pub(crate) fn draw_command_center(ui: &mut egui::Ui, s: &VariantStyle, state: &ColonyProtoState) {
    let t = state.elapsed;

    // ── Row 1: Trend sparklines ─────────────────────────────────────────────
    draw_section(ui, s, " Command Center — Trends / Policy / Demographics / Factions ");

    ui.columns(4, |cols| {
        let groups: [(&str, &[f32], egui::Color32); 4] = [
            ("Food",   &FOOD_TREND,   s.success_color),
            ("Morale", &MORALE_TREND, s.info_color),
            ("Stress", &STRESS_TREND, s.warn_color),
            ("Threat", &THREAT_TREND, s.danger_color),
        ];
        for (i, (label, data, color)) in groups.into_iter().enumerate() {
            let last = data.last().copied().unwrap_or(0.0);
            let prev = data.get(data.len().saturating_sub(2)).copied().unwrap_or(last);
            let delta_sym = if last > prev { "▲" } else if last < prev { "▼" } else { "─" };
            cols[i].label(styled(s, &format!(" {} {:.0} {}", label, last, delta_sym), s.body_size, color));
            draw_sparkline(&mut cols[i], data, color, 180.0, 36.0);
        }
    });

    ui.add_space(6.0 * s.spacing);

    // ── Row 2: Policy + Demographics ────────────────────────────────────────
    ui.columns(2, |cols| {
        draw_section(&mut cols[0], s, " Colony Policy ");

        let policies: [(&str, bool, &str); 6] = [
            ("Open Rations",       true,  "Feed colonists from common stores"),
            ("Curfew Enforced",    false, "Restrict movement after dark"),
            ("Arms Stockpile",     true,  "Prioritise weapon crafting"),
            ("Medical Priority",   true,  "Route healers before builders"),
            ("Conscription Ready", false, "Draft civilians if attacked"),
            ("Trade Outpost",      false, "Allow external merchant access"),
        ];
        for (name, active, note) in policies {
            let (icon, color) = if active { ("[ON] ", s.success_color) } else { ("[OFF]", s.subtitle_color) };
            cols[0].horizontal(|ui| {
                ui.label(styled(s, icon, s.body_size, color));
                ui.label(styled(s, &format!(" {:<22} ", name), s.body_size, color));
            });
            cols[0].label(styled(s, &format!("       {}", note), s.small_size, s.subtitle_color.gamma_multiply(0.7)));
        }

        draw_section(&mut cols[1], s, " Demographics ");

        let demos: [(&str, f32, f32, egui::Color32); 6] = [
            ("Soldiers",    18., 60., s.danger_color),
            ("Crafters",    14., 60., s.warn_color),
            ("Farmers",     11., 60., s.success_color),
            ("Medics",       5., 60., s.info_color),
            ("Haulers",      8., 60., s.subtitle_color),
            ("Idle / Unassigned", 4., 60., s.accent_color),
        ];
        for (label, count, total, color) in demos {
            cols[1].label(styled(s, &format!(" {:<20} {:>2.0}/{:.0}", label, count, total), s.small_size, color));
            cols[1].add(
                egui::ProgressBar::new(count / total)
                    .desired_width(260.0)
                    .fill(color)
            );
        }
    });

    ui.add_space(6.0 * s.spacing);

    // ── Row 3: Tension + Factions ────────────────────────────────────────────
    ui.columns(2, |cols| {
        draw_section(&mut cols[0], s, " Internal Tension ");

        let tensions: [(&str, f32, &str); 5] = [
            ("Soldier / Civilian divide", 0.62, "High — curfew debates"),
            ("Food scarcity anxiety",     0.28, "Low — stores stable"),
            ("Leadership trust",          0.45, "Moderate"),
            ("Gang / Scavenger friction", 0.71, "High — patrol incident"),
            ("Ideological split",         0.33, "Low"),
        ];
        for (label, val, note) in tensions {
            let color = if val > 0.6 { s.danger_color } else if val > 0.35 { s.warn_color } else { s.success_color };
            cols[0].label(styled(s, &format!(" {}", label), s.small_size, color));
            cols[0].add(
                egui::ProgressBar::new(val)
                    .desired_width(240.0)
                    .fill(color)
                    .text(format!("{:.0}%", val * 100.0))
            );
            cols[0].label(styled(s, &format!("   {}", note), s.small_size, s.subtitle_color.gamma_multiply(0.7)));
        }

        draw_section(&mut cols[1], s, " Faction Relations ");

        let pulse = ((t * 1.5).sin() * 0.5 + 0.5) as f32;
        let factions: [(&str, f32, &str, &str); 7] = [
            ("Iron Vigil",       0.78, "Allied",   "internal"),
            ("Ashen Remnant",    0.55, "Neutral",  "internal"),
            ("Hollowed March",   0.22, "Hostile",  "internal"),
            ("──────────────",   0.0,  "",          "divider"),
            ("Merchant Caravan", 0.66, "Friendly", "external"),
            ("Warlord Syndicate",0.18, "Hostile",  "external"),
            ("Cult of the Veil", 0.41, "Wary",     "external"),
        ];
        for (name, standing, label, kind) in factions {
            if kind == "divider" {
                cols[1].label(styled(s, " ─── external ───────────────────────", s.small_size, s.subtitle_color.gamma_multiply(0.5)));
                continue;
            }
            let color = if standing > 0.6 { s.success_color }
                        else if standing > 0.35 { s.warn_color }
                        else { s.danger_color };
            // hostile factions pulse slightly
            let draw_color = if standing < 0.3 { color.gamma_multiply(0.7 + pulse * 0.3) } else { color };
            cols[1].horizontal(|ui| {
                ui.add(egui::ProgressBar::new(standing as f32).desired_width(120.0).fill(draw_color));
                ui.label(styled(s, &format!(" {:<22} {}", name, label), s.small_size, draw_color));
            });
        }
    });
}

pub(crate) fn draw_bottom_command_bar(ui: &mut egui::Ui, s: &VariantStyle) {
    draw_section(ui, s, " Command Bar ");
    ui.horizontal(|ui| {
        for cmd in [
            "[Architect]",
            "[Build]",
            "[Work]",
            "[Zones]",
            "[Schedule]",
            "[Assign]",
            "[Research]",
            "[Logistics]",
            "[Orders]",
        ] {
            ui.label(styled(s, cmd, s.body_size, s.accent_color));
        }
    });
}

pub(crate) fn draw_backdrop(ui: &mut egui::Ui, s: &VariantStyle, t: f32) {
    let rect = ui.max_rect();
    let painter = ui.painter();

    let step = 36.0;
    let stroke = egui::Stroke::new(1.0, s.info_color.gamma_multiply(0.14));

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

    let pulse = ((t * 1.7).sin() + 1.0) * 0.5;
    painter.circle_filled(rect.center(), 24.0, s.accent_color.gamma_multiply(0.08 + pulse * 0.08));
}

pub(crate) fn status_color(s: &VariantStyle, status: &str) -> egui::Color32 {
    match status {
        "Peak" => s.warn_color,
        "Busy" => s.info_color,
        "Alert" => s.danger_color,
        "Stable" => s.success_color,
        _ => s.subtitle_color,
    }
}

pub(crate) fn draw_section(ui: &mut egui::Ui, s: &VariantStyle, label: &str) {
    let line = format!("{} {} {}", "─".repeat(12), label, "─".repeat(12));
    ui.label(styled(s, &line, s.small_size, s.accent_color));
}

pub(crate) fn styled(style: &VariantStyle, text: &str, size: f32, color: egui::Color32) -> egui::RichText {
    let mut rt = egui::RichText::new(text).size(size).color(color);
    if style.mono_all {
        rt = rt.monospace();
    }
    rt
}
