//! UX prototype viewer — Crimson & Gold focused mockups across core screens.
//!
//! Run with: `cargo run --bin ux_prototypes`
//!
//! Controls:
//!   M/C/D/O   — switch screen (Menu / Colony / Dungeon HUD / Overworld)
//!   1/2/3     — switch animation profile (Subtle / Drift / Scanline)
//!   Esc       — quit

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use super::ux_style_contract::{MotionProfile, VariantStyle, style_for};

// ── Screens ────────────────────────────────────────────────────────────────

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtoScreen {
    Menu,
    Colony,
    DungeonHud,
    Overworld,
}

impl ProtoScreen {
    fn name(&self) -> &'static str {
        match self {
            Self::Menu => "Menu",
            Self::Colony => "Colony",
            Self::DungeonHud => "Dungeon HUD",
            Self::Overworld => "Overworld",
        }
    }
}

// ── Plugin ─────────────────────────────────────────────────────────────────

pub struct UxPrototypePlugin;

impl Plugin for UxPrototypePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ProtoState {
            screen: ProtoScreen::Menu,
        })
        .insert_resource(MotionState {
            profile: MotionProfile::Subtle,
            elapsed: 0.0,
        })
        .add_systems(Startup, setup_prototype_camera)
        .add_systems(Update, (handle_prototype_input, tick_motion_state))
        .add_systems(EguiPrimaryContextPass, draw_prototypes);
    }
}

#[derive(Resource)]
struct ProtoState {
    screen: ProtoScreen,
}

#[derive(Resource)]
struct MotionState {
    profile: MotionProfile,
    elapsed: f32,
}

// ── Camera ─────────────────────────────────────────────────────────────────

fn setup_prototype_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

// ── Input ──────────────────────────────────────────────────────────────────

fn handle_prototype_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<ProtoState>,
    mut motion: ResMut<MotionState>,
    mut exit: MessageWriter<AppExit>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
        return;
    }
    if keys.just_pressed(KeyCode::KeyM) {
        state.screen = ProtoScreen::Menu;
    }
    if keys.just_pressed(KeyCode::KeyC) {
        state.screen = ProtoScreen::Colony;
    }
    if keys.just_pressed(KeyCode::KeyD) {
        state.screen = ProtoScreen::DungeonHud;
    }
    if keys.just_pressed(KeyCode::KeyO) {
        state.screen = ProtoScreen::Overworld;
    }
    if keys.just_pressed(KeyCode::Digit1) {
        motion.profile = MotionProfile::Subtle;
    }
    if keys.just_pressed(KeyCode::Digit2) {
        motion.profile = MotionProfile::Drift;
    }
    if keys.just_pressed(KeyCode::Digit3) {
        motion.profile = MotionProfile::Scanline;
    }
}

fn tick_motion_state(time: Res<Time>, mut motion: ResMut<MotionState>) {
    motion.elapsed += time.delta_secs();
}

// ── Draw dispatch ──────────────────────────────────────────────────────────

fn draw_prototypes(
    mut contexts: EguiContexts,
    state: Res<ProtoState>,
    motion: Res<MotionState>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    // Dark background for all variants so they read like a terminal
    let bg = egui::Color32::from_rgb(12, 12, 18);

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(bg))
        .show(ctx, |ui| {
            let style = style_for();

            draw_locked_backdrop(ui, &style, &motion);

            // Status bar at the very top
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!(
                        " Screen: [{}]    Theme: [Crimson & Gold - Circuit Locked]  Motion: [{}]  M/C/D/O=screen  1/2/3=motion  Esc=quit",
                        state.screen.name(),
                        motion.profile.label(),
                    ))
                    .size(11.0)
                    .color(style.subtitle_color),
                );
            });
            ui.separator();

            draw_widget_motion_lab(ui, &style, &motion);
            ui.add_space(10.0 * style.spacing);

            match state.screen {
                ProtoScreen::Menu => draw_menu_mockup(ui, &style),
                ProtoScreen::Colony => draw_colony_mockup(ui, &style),
                ProtoScreen::DungeonHud => draw_dungeon_mockup(ui, &style),
                ProtoScreen::Overworld => draw_overworld_mockup(ui, &style, &motion),
            }
        });
}

// ═══════════════════════════════════════════════════════════════════════════
//  Variant styling helpers
// ═══════════════════════════════════════════════════════════════════════════

/// Rich text helper respecting variant mono/proportional choice.
fn styled(style: &VariantStyle, text: &str, size: f32, color: egui::Color32) -> egui::RichText {
    let mut rt = egui::RichText::new(text).size(size).color(color);
    if style.mono_all {
        rt = rt.monospace();
    }
    rt
}

fn btn_styled(style: &VariantStyle, text: &str, size: f32) -> egui::RichText {
    let mut rt = egui::RichText::new(text).size(size);
    if style.mono_all {
        rt = rt.monospace();
    }
    rt
}

fn draw_border_section(ui: &mut egui::Ui, style: &VariantStyle, label: &str) {
    if !style.decorated_borders {
        ui.separator();
        ui.label(styled(style, label, style.heading_size, style.title_color));
        return;
    }
    let total = 40usize;
    let label_len = label.len() + 4; // "╡ label ╞"
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

fn draw_widget_motion_lab(ui: &mut egui::Ui, style: &VariantStyle, motion: &MotionState) {
    draw_border_section(ui, style, " Widget Motion Lab ");

    let t = motion.elapsed;
    let (scan, spinner, signal_color) = match motion.profile {
        MotionProfile::Subtle => {
            let scan = ((t * 0.22).fract() * 100.0).clamp(0.0, 100.0);
            let spinner = ".";
            (scan, spinner, style.accent_color.gamma_multiply(0.85))
        }
        MotionProfile::Drift => {
            let scan = ((t * 0.35).fract() * 100.0).clamp(0.0, 100.0);
            let spinner_index = ((t * 3.5) as usize) % 4;
            let spinner = ["·", "•", "◦", "•"][spinner_index];
            (scan, spinner, style.info_color.gamma_multiply(0.9))
        }
        MotionProfile::Scanline => {
            let scan = ((t * 0.5).fract() * 100.0).clamp(0.0, 100.0);
            let spinner_index = ((t * 6.0) as usize) % 4;
            let spinner = ["|", "/", "-", "\\"][spinner_index];
            (scan, spinner, style.warn_color.gamma_multiply(0.9))
        }
    };

    ui.horizontal(|ui| {
        ui.label(styled(
            style,
            &format!("{} URGENCY", style.symbols.warning_icon),
            style.small_size,
            signal_color,
        ));
        ui.add(
            egui::ProgressBar::new(scan / 100.0)
                .show_percentage()
                .text(format!("scanner {:>3.0}%", scan)),
        );
        ui.label(styled(
            style,
            &format!("{} log stream", spinner),
            style.small_size,
            style.subtitle_color,
        ));
    });
}

fn draw_locked_backdrop(
    ui: &mut egui::Ui,
    style: &VariantStyle,
    motion: &MotionState,
) {
    let rect = ui.max_rect();
    let painter = ui.painter();
    painter.rect_filled(rect, 0.0, style.panel_bg);

    draw_circuit_lattice(painter, rect, style, motion.elapsed);
}

fn draw_circuit_lattice(
    painter: &egui::Painter,
    rect: egui::Rect,
    style: &VariantStyle,
    t: f32,
) {
    let step = 34.0;
    let stroke = egui::Stroke::new(1.0, style.info_color.gamma_multiply(0.12));

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

    let node = style.success_color.gamma_multiply(0.24);
    let frame_phase = ((t * 2.0) as i32).rem_euclid(4) as f32;
    let mut nx = rect.left() + step;
    while nx < rect.right() {
        let mut ny = rect.top() + step;
        while ny < rect.bottom() {
            let offset = ((nx + ny) / step).rem_euclid(4.0);
            let jitter = if offset == frame_phase { 1.2 } else { 0.0 };
            painter.circle_filled(egui::pos2(nx + jitter, ny), 2.2, node);
            ny += step;
        }
        nx += step;
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  MOCK DATA — shared across variants
// ═══════════════════════════════════════════════════════════════════════════

struct MockColony {
    food: i32,
    water: i32,
    scrap: i32,
    medicine: i32,
    ammo: i32,
    urgency: Option<(&'static str, bool)>, // (text, is_critical)
    objective: &'static str,
    stations: &'static [&'static str],
    survivors: &'static [(&'static str, &'static str, &'static str)], // (name, task, need)
}

const MOCK_COLONY: MockColony = MockColony {
    food: 42,
    water: 18,
    scrap: 7,
    medicine: 3,
    ammo: 12,
    urgency: Some(("Water running low", false)),
    objective: "Reach the shelter gate and press Enter.",
    stations: &[
        "Farm Plot      [Active]   2 workers",
        "Water Pump     [Idle]     0 workers",
        "Scrap Bench    [Active]   1 worker",
        "Medical Tent   [Active]   1 worker",
        "Watchtower     [Idle]     0 workers",
    ],
    survivors: &[
        ("Elena", "Farming", "Fed"),
        ("Marcus", "Scavenging", "Tired"),
        ("Thorn", "Idle", "Fed"),
        ("Sister Mira", "Healing", "Stressed"),
    ],
};

struct MockDungeon {
    hp: i32,
    hp_max: i32,
    ap: i32,
    ap_max: i32,
    turn: u32,
    sanity_pct: f32,
    sanity_label: &'static str,
    floor: i32,
    max_floors: i32,
    weapon: &'static str,
    #[allow(dead_code)]
    ammo_current: i32,
    #[allow(dead_code)]
    ammo_max: i32,
    armor: &'static str,
    armor_current: i32,
    armor_max: i32,
    log_entries: &'static [(&'static str, &'static str)], // (text, color_name)
}

const MOCK_DUNGEON: MockDungeon = MockDungeon {
    hp: 28,
    hp_max: 35,
    ap: 3,
    ap_max: 6,
    turn: 142,
    sanity_pct: 0.62,
    sanity_label: "Stressed",
    floor: 3,
    max_floors: 8,
    weapon: "Rusted Longsword",
    ammo_current: 0,
    ammo_max: 0,
    armor: "Patchwork Cuirass",
    armor_current: 18,
    armor_max: 25,
    log_entries: &[
        ("You strike the Husk for 8 damage.", "player"),
        ("The Husk claws at you...miss!", "miss"),
        ("You strike the Husk for 12 damage.  CRITICAL!", "critical"),
        ("The Husk collapses into ash.", "death"),
        ("A faint whisper echoes: '...the Virtues remember...'", "system"),
        ("You find 3 Scrap fragments.", "status"),
        ("Sanity check: Stressed — perception slightly warped.", "system"),
    ],
};

struct MockOverworld {
    day: i32,
    weather: &'static str,
    current: &'static str,
    food: i32,
    water: i32,
    hp: i32,
    sanity: i32,
    log_entries: &'static [&'static str],
}

const MOCK_OVERWORLD: MockOverworld = MockOverworld {
    day: 14,
    weather: "Ashfall",
    current: "Forest Crossroads",
    food: 12,
    water: 8,
    hp: 74,
    sanity: 85,
    log_entries: &[
        "Arrived at Forest Crossroads.",
        "Ashfall is slowing movement (-50%).",
        "Supplies consumed: -1 food, -1 water.",
    ],
};

// ═══════════════════════════════════════════════════════════════════════════
//  MENU MOCKUP
// ═══════════════════════════════════════════════════════════════════════════

fn draw_menu_mockup(ui: &mut egui::Ui, s: &VariantStyle) {

    ui.vertical_centered(|ui| {
        ui.add_space(50.0 * s.spacing);

        // ── Title section ──
        if s.decorated_borders {
            ui.label(styled(&s, "╔════════════════════════════════════════╗", s.small_size, s.title_color));
            ui.label(styled(&s, "║                                      ║", s.small_size, s.title_color));
        }
        ui.label(styled(&s, "BROKEN DIVINITY", s.heading_size + 14.0, s.title_color));
        ui.label(styled(&s, "A post-apocalyptic roguelike", s.body_size + 2.0, s.subtitle_color));
        if s.decorated_borders {
            ui.label(styled(&s, "║                                      ║", s.small_size, s.title_color));
            ui.label(styled(&s, "╚════════════════════════════════════════╝", s.small_size, s.title_color));
        }

        ui.add_space(32.0 * s.spacing);

        // ── Seed section ──
        if s.decorated_borders {
            ui.label(styled(&s, "╔════ Seed ═════════════════════════════╗", s.small_size, s.accent_color));
        }
        ui.horizontal(|ui| {
            if s.decorated_borders { ui.label(styled(&s, "║ ", s.small_size, s.accent_color)); }
            ui.label(styled(&s, "Seed:", s.body_size, s.title_color));
            ui.label(styled(&s, "[________________]", s.body_size, s.accent_color));
            if s.decorated_borders { ui.label(styled(&s, " ║", s.small_size, s.accent_color)); }
        });
        ui.label(
            styled(&s, "  Leave blank for a random world seed.", s.small_size, s.subtitle_color),
        );
        if s.decorated_borders {
            ui.label(styled(&s, "╚════════════════════════════════════════╝", s.small_size, s.accent_color));
        }

        ui.add_space(24.0 * s.spacing);

        // ── Primary action stack (centered and high-visibility) ──
        draw_border_section(ui, &s, " Start ");
        let _ = ui.button(btn_styled(&s, "    ▶  NEW GAME    ", s.heading_size + 4.0));
        ui.add_space(8.0 * s.spacing);
        let _ = ui.button(btn_styled(&s, "      Load Game     ", s.body_size + 2.0));
        ui.label(styled(&s, "No save found yet - start a New Game to create one.", s.small_size, s.subtitle_color));

        ui.add_space(24.0 * s.spacing);

        // ── Quit section ──
        if s.decorated_borders {
            ui.label(styled(&s, "╔════ Quit ═════════════════════════════╗", s.small_size, s.accent_color));
        }
        ui.horizontal(|ui| {
            if s.decorated_borders { ui.label(styled(&s, "║ ", s.small_size, s.accent_color)); }
            let _ = ui.button(btn_styled(&s, "Quit", s.body_size));
            ui.label(styled(&s, " Quit the game? ", s.small_size, s.warn_color));
            let _ = ui.button(btn_styled(&s, "Yes", s.small_size + 1.0));
            let _ = ui.button(btn_styled(&s, "No", s.small_size + 1.0));
            if s.decorated_borders { ui.label(styled(&s, " ║", s.small_size, s.accent_color)); }
        });
        if s.decorated_borders {
            ui.label(styled(&s, "╚════════════════════════════════════════╝", s.small_size, s.accent_color));
        }

        ui.add_space(36.0 * s.spacing);
        ui.label(styled(&s, "[Enter] start   [Esc] quit", s.small_size, s.subtitle_color));
    });
}

// ═══════════════════════════════════════════════════════════════════════════
//  COLONY MOCKUP
// ═══════════════════════════════════════════════════════════════════════════

fn draw_colony_mockup(ui: &mut egui::Ui, s: &VariantStyle) {
    let m = &MOCK_COLONY;

    // ── Resource bar (top strip) ──
    if s.decorated_borders {
        ui.label(styled(&s, "╔════ Resources ═══════════════════════════════════════════╗", s.small_size, s.accent_color));
    }
    ui.horizontal(|ui| {
        if s.decorated_borders { ui.label(styled(&s, "║", s.small_size, s.accent_color)); }
        res_cell(ui, &s, "@", "Food", m.food, s.success_color);
        if s.decorated_borders { ui.label(styled(&s, "│", s.small_size, s.accent_color)); }
        res_cell(ui, &s, "~", "Water", m.water, s.info_color);
        if s.decorated_borders { ui.label(styled(&s, "│", s.small_size, s.accent_color)); }
        res_cell(ui, &s, "#", "Scrap", m.scrap, s.subtitle_color);
        if s.decorated_borders { ui.label(styled(&s, "│", s.small_size, s.accent_color)); }
        res_cell(ui, &s, "+", "Medicine", m.medicine, s.success_color);
        if s.decorated_borders { ui.label(styled(&s, "│", s.small_size, s.accent_color)); }
        res_cell(ui, &s, "»", "Ammo", m.ammo, s.warn_color);
        if s.decorated_borders { ui.label(styled(&s, "║", s.small_size, s.accent_color)); }

        if let Some((urgent_text, _is_critical)) = m.urgency {
            ui.label(styled(&s, &format!("  ! {}", urgent_text), s.body_size, s.warn_color));
        }
    });
    if s.decorated_borders {
        ui.label(styled(&s, "╚══════════════════════════════════════════════════════════╝", s.small_size, s.accent_color));
    }

    ui.add_space(10.0 * s.spacing);

    // ── Two-column layout: stations left, survivors right ──
    ui.columns(2, |cols| {
        draw_border_section(&mut cols[0], &s, " Stations ");
        for station in m.stations {
            let (name, status, workers) = parse_station(station);
            let color = if status.contains("Active") { s.success_color } else { s.subtitle_color };
            cols[0].label(styled(&s, &format!("  {}  [{}]  {}", name, status, workers), s.body_size, color));
        }

        draw_border_section(&mut cols[1], &s, " Survivors ");
        for (name, task, need) in m.survivors {
            let need_color = match *need {
                "Fed" => s.success_color,
                "Tired" => s.warn_color,
                "Stressed" => s.danger_color,
                _ => s.subtitle_color,
            };
            cols[1].label(styled(&s, &format!("  {:<14} {:<12} [{}]", name, task, need), s.body_size, need_color));
        }
    });

    ui.add_space(10.0 * s.spacing);

    // ── Objective CTA ──
    ui.label(styled(&s, m.objective, s.body_size, s.accent_color));

    ui.add_space(10.0 * s.spacing);

    // ── Action buttons — aligned in a row ──
    if s.decorated_borders {
        ui.label(styled(&s, "╔════ Actions ════════════════════════════════════════════╗", s.small_size, s.accent_color));
    }
    ui.horizontal(|ui| {
        if s.decorated_borders { ui.label(styled(&s, "║ ", s.small_size, s.accent_color)); }
        let _ = ui.button(btn_styled(&s, "[B]uild Station", s.body_size));
        let _ = ui.button(btn_styled(&s, "[A]ssign Worker", s.body_size));
        let _ = ui.button(btn_styled(&s, "[R]esearch", s.body_size));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let _ = ui.button(btn_styled(&s, "Save & Quit", s.body_size));
        });
        if s.decorated_borders { ui.label(styled(&s, " ║", s.small_size, s.accent_color)); }
    });
    if s.decorated_borders {
        ui.label(styled(&s, "╚══════════════════════════════════════════════════════════╝", s.small_size, s.accent_color));
    }
}

fn parse_station<'a>(station: &'a &str) -> (&'a str, &'a str, &'a str) {
    if let Some((name, rest)) = station.split_once('[') {
        let status_end = rest.find(']').unwrap_or(0);
        let status = &rest[..status_end];
        let workers = rest[status_end + 1..].trim();
        (name.trim(), status, workers)
    } else {
        (*station, "", "")
    }
}

fn res_cell(ui: &mut egui::Ui, s: &VariantStyle, icon: &str, label: &str, value: i32, color: egui::Color32) {
    ui.horizontal(|ui| {
        ui.label(styled(s, icon, s.body_size, color));
        ui.label(styled(s, &format!("{}:{}", label, value), s.body_size, color));
    });
}

// ═══════════════════════════════════════════════════════════════════════════
//  DUNGEON HUD MOCKUP
// ═══════════════════════════════════════════════════════════════════════════

fn draw_dungeon_mockup(ui: &mut egui::Ui, s: &VariantStyle) {
    let d = &MOCK_DUNGEON;

    // ── Top HUD strip ──
    if s.decorated_borders {
        ui.label(styled(&s, "╔════ Status ═════════════════════════════════════════════╗", s.small_size, s.accent_color));
    }
    ui.horizontal(|ui| {
        if s.decorated_borders { ui.label(styled(&s, "║", s.small_size, s.accent_color)); }

        let hp_frac = d.hp as f32 / d.hp_max as f32;
        let hp_color = if hp_frac < 0.3 { s.danger_color } else if hp_frac < 0.6 { s.warn_color } else { s.success_color };
        ui.label(styled(&s, &format!("HP: {}/{}", d.hp, d.hp_max), s.body_size, hp_color));
        if s.decorated_borders { ui.label(styled(&s, "│", s.small_size, s.accent_color)); }

        ui.label(styled(&s, &format!("AP: {}/{}", d.ap, d.ap_max), s.body_size, s.info_color));
        if s.decorated_borders { ui.label(styled(&s, "│", s.small_size, s.accent_color)); }

        ui.label(styled(&s, &format!("Turn {}", d.turn), s.body_size, s.subtitle_color));
        if s.decorated_borders { ui.label(styled(&s, "│", s.small_size, s.accent_color)); }

        let san_color = match d.sanity_label {
            "Normal" => s.success_color,
            "Stressed" => s.warn_color,
            "Shaken" => s.danger_color,
            _ => s.danger_color,
        };
        let san_pct = (d.sanity_pct * 100.0) as i32;
        ui.label(styled(&s, &format!("SAN: {}% ({})", san_pct, d.sanity_label), s.body_size, san_color));
        if s.decorated_borders { ui.label(styled(&s, "│", s.small_size, s.accent_color)); }

        ui.label(styled(&s, &format!("F{}/{}", d.floor, d.max_floors), s.body_size, s.info_color));
        if s.decorated_borders { ui.label(styled(&s, "║", s.small_size, s.accent_color)); }
    });

    // Weapon / Armor row
    ui.horizontal(|ui| {
        if s.decorated_borders { ui.label(styled(&s, "║", s.small_size, s.accent_color)); }
        ui.label(styled(&s, &format!("⚔  {}", d.weapon), s.body_size, s.accent_color));
        if s.decorated_borders { ui.label(styled(&s, "│", s.small_size, s.accent_color)); }
        let arm_color = if d.armor_current < d.armor_max / 2 { s.danger_color } else { s.info_color };
        ui.label(styled(&s, &format!("🛡  {} [{}/{}]", d.armor, d.armor_current, d.armor_max), s.body_size, arm_color));
        if s.decorated_borders { ui.label(styled(&s, "║", s.small_size, s.accent_color)); }
    });
    if s.decorated_borders {
        ui.label(styled(&s, "╚══════════════════════════════════════════════════════════╝", s.small_size, s.accent_color));
    }

    ui.add_space(10.0 * s.spacing);

    // ── Main workspace: viewport + tracking panel ──
    ui.columns(2, |cols| {
        draw_border_section(&mut cols[0], s, " Viewport ");
        cols[0].label(styled(s, "  ##..##..##..##..##", s.body_size, s.subtitle_color));
        cols[0].label(styled(s, "  #..g...#....+....#", s.body_size, s.warn_color));
        cols[0].label(styled(s, "  #....#....#......#", s.body_size, s.subtitle_color));
        cols[0].label(styled(s, "  #..@..#..G.#...^..#", s.body_size, s.title_color));
        cols[0].label(styled(s, "  #....#....#......#", s.body_size, s.subtitle_color));
        cols[0].label(styled(s, "  #..~....#....!...#", s.body_size, s.info_color));
        cols[0].label(styled(s, "  ##..##..##..##..##", s.body_size, s.subtitle_color));
        cols[0].add_space(4.0 * s.spacing);
        cols[0].label(styled(
            s,
            "  Focus ring: @ Player  |  Threat markers: G g",
            s.small_size,
            s.info_color,
        ));

        draw_border_section(&mut cols[1], s, " Tracking ");
        cols[1].label(styled(s, "  PLAYER LOCK: ACTIVE", s.body_size, s.success_color));
        cols[1].label(styled(s, "  Camera: centered + soft lead", s.small_size, s.subtitle_color));
        cols[1].add_space(4.0 * s.spacing);
        cols[1].label(styled(s, "  Minimap", s.small_size, s.title_color));
        cols[1].label(styled(s, "  +---------+", s.small_size, s.accent_color));
        cols[1].label(styled(s, "  |..##..G..|", s.small_size, s.subtitle_color));
        cols[1].label(styled(s, "  |...[@]...|", s.small_size, s.title_color));
        cols[1].label(styled(s, "  |..^..!..>|", s.small_size, s.warn_color));
        cols[1].label(styled(s, "  +---------+", s.small_size, s.accent_color));
        cols[1].add_space(6.0 * s.spacing);
        cols[1].label(styled(s, "  Target: Husk Ravager", s.body_size, s.warn_color));
        cols[1].label(styled(s, "  Range: 4  Cover: Half", s.small_size, s.subtitle_color));
        cols[1].label(styled(s, "  Hit 55%   Dmg 8-15", s.small_size, s.info_color));
    });

    ui.add_space(10.0 * s.spacing);

    // ── Short tactical log ──
    draw_border_section(ui, s, " Tactical Log ");
    for (text, color_name) in d.log_entries.iter().take(4) {
        let color = match *color_name {
            "player" => s.success_color,
            "miss" => s.subtitle_color,
            "critical" => s.warn_color,
            "death" => s.danger_color,
            "system" => s.info_color,
            "status" => s.accent_color,
            _ => s.subtitle_color,
        };
        ui.label(styled(s, text, s.body_size, color));
    }

    // Bottom command strip
    ui.add_space(6.0 * s.spacing);
    ui.label(styled(
        s,
        "h j k l move  f fire  x examine  i pack  tab cycle  esc pause",
        s.small_size,
        s.subtitle_color,
    ));
}

fn draw_overworld_mockup(ui: &mut egui::Ui, s: &VariantStyle, motion: &MotionState) {
    let o = &MOCK_OVERWORLD;

    if s.decorated_borders {
        ui.label(styled(
            s,
            "╔════ Route Status ════════════════════════════════════════╗",
            s.small_size,
            s.accent_color,
        ));
    }
    ui.horizontal(|ui| {
        if s.decorated_borders {
            ui.label(styled(s, "║", s.small_size, s.accent_color));
        }
        ui.label(styled(s, &format!("DAY {}", o.day), s.body_size, s.title_color));
        ui.separator();
        ui.label(styled(s, o.weather, s.body_size, s.warn_color));
        ui.separator();
        ui.label(styled(s, o.current, s.body_size, s.info_color));
        ui.separator();
        ui.label(styled(
            s,
            &format!("Food {}  Water {}  HP {}  SAN {}", o.food, o.water, o.hp, o.sanity),
            s.body_size,
            s.subtitle_color,
        ));
        if s.decorated_borders {
            ui.label(styled(s, "║", s.small_size, s.accent_color));
        }
    });
    if s.decorated_borders {
        ui.label(styled(
            s,
            "╚═══════════════════════════════════════════════════════════╝",
            s.small_size,
            s.accent_color,
        ));
    }

    ui.add_space(10.0 * s.spacing);

    ui.columns(2, |cols| {
        draw_border_section(&mut cols[0], s, " Route Workspace ");
        cols[0].label(styled(s, "  ┌──────────────── ASCII Route Grid ────────────────┐", s.small_size, s.accent_color));
        cols[0].label(styled(s, "  │                                                   │", s.small_size, s.subtitle_color));
        cols[0].label(styled(s, "  │                ■ Ruins of Malkov                  │", s.body_size, s.subtitle_color));
        cols[0].label(styled(s, "  │               / \\                                 │", s.small_size, s.subtitle_color));
        cols[0].label(styled(s, "  │      ▲ Shelter      [●] Forest Crossroads         │", s.body_size, s.success_color));
        cols[0].label(styled(s, "  │         \\               |                        │", s.small_size, s.subtitle_color));
        cols[0].label(styled(s, "  │          \\              |                        │", s.small_size, s.subtitle_color));
        cols[0].label(styled(s, "  │        ◆ Pilgrim Spire   |----[★] Shattered Labs  │", s.body_size, s.title_color));
        cols[0].label(styled(s, "  │                                                   │", s.small_size, s.subtitle_color));
        cols[0].label(styled(s, "  └───────────────────────────────────────────────────┘", s.small_size, s.accent_color));
        cols[0].add_space(6.0 * s.spacing);

        let route_scan = ((motion.elapsed * 0.28).fract() * 100.0).clamp(0.0, 100.0);
        cols[0].add(
            egui::ProgressBar::new(route_scan / 100.0)
                .text(format!("route preview {:>3.0}%", route_scan))
                .fill(s.info_color.gamma_multiply(0.85)),
        );
        cols[0].label(styled(
            s,
            "Legend: ▲ shelter  ● crossroads  ★ dungeon  ◆ landmark  ■ ruins",
            s.small_size,
            s.subtitle_color,
        ));
        cols[0].label(styled(
            s,
            "Exploration keeps graph structure: choose connected nodes only.",
            s.small_size,
            s.info_color,
        ));

        draw_border_section(&mut cols[1], s, " Focus + Actions ");
        cols[1].label(styled(s, "Shattered Labs [Dungeon]", s.body_size, s.title_color));
        cols[1].label(styled(s, "Threat: Severe", s.body_size, s.warn_color));
        cols[1].label(styled(s, "Distance: 3 days", s.body_size, s.info_color));
        cols[1].label(styled(s, "Route cost: -3 food  -3 water", s.body_size, s.accent_color));
        cols[1].add_space(6.0 * s.spacing);
        cols[1].label(styled(s, "Connected Nodes", s.small_size, s.info_color));
        cols[1].label(styled(s, "  → Malkov Ruins (Ruins) 2 days", s.small_size, s.subtitle_color));
        cols[1].label(styled(s, "  → Shattered Labs (Dungeon) 3 days", s.small_size, s.subtitle_color));
        cols[1].label(styled(s, "  → Pilgrim Spire (Landmark) 4 days", s.small_size, s.subtitle_color));
        cols[1].add_space(8.0 * s.spacing);
        cols[1].label(styled(s, "Route Forecast", s.small_size, s.info_color));
        cols[1].label(styled(s, "  Day 1: Ashfall  (slow)", s.small_size, s.warn_color));
        cols[1].label(styled(s, "  Day 2: Fog      (risky)", s.small_size, s.warn_color));
        cols[1].label(styled(s, "  Day 3: Clear    (stable)", s.small_size, s.success_color));
        cols[1].add_space(8.0 * s.spacing);
        let _ = cols[1].button(btn_styled(s, "[Enter] Travel", s.body_size));
        let _ = cols[1].button(btn_styled(s, "[X] Inspect Node", s.body_size));
        let _ = cols[1].button(btn_styled(s, "[W] Weather", s.body_size));
    });

    ui.add_space(10.0 * s.spacing);
    draw_border_section(ui, s, " Travel Log ");
    for line in o.log_entries {
        ui.label(styled(s, line, s.body_size, s.info_color));
    }

    ui.add_space(4.0 * s.spacing);
    ui.label(styled(
        s,
        "h j k l move focus  enter travel  x inspect  w weather  esc back",
        s.small_size,
        s.subtitle_color,
    ));
}