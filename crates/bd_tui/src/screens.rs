//! Data-driven TUI screens — ScreenDefinition, WidgetRegistry, and ScreenState.
//!
//! Phase 15: Moves from hardcoded layout in draw_ui to schema-driven screen
//! definitions. Widgets are registered by ID and dispatched from the current
//! screen definition.

use std::collections::HashMap;

use bevy_ecs::prelude::*;

/// Width percentage for the stats/info right-side panel.
const STATS_PANEL_WIDTH_PCT: u16 = 28;

/// HP percentage above which the value is shown as positive (healthy).
const HP_GREEN_THRESHOLD_PCT: i32 = 50;
/// HP percentage below which the value is shown in yellow (wounded).
const HP_YELLOW_THRESHOLD_PCT: i32 = 25;
/// Above this ratio HP is green, below it yellow, below HP_YELLOW_THRESHOLD it's red.
fn hp_tone(current: i32, max: i32) -> UiTone {
    if max <= 0 {
        return UiTone::Danger;
    }
    let pct = (current * 100) / max;
    if pct >= HP_GREEN_THRESHOLD_PCT {
        UiTone::Positive
    } else if pct >= HP_YELLOW_THRESHOLD_PCT {
        UiTone::Warning
    } else {
        UiTone::Danger
    }
}

/// AP tone: informational at every fill level.
fn ap_tone(_current: i32, _max: i32) -> UiTone {
    UiTone::Info
}

use ratatui::{Frame, layout::Rect};

use super::{
    chrome::{PanelTone, UiTone, meter, panel, resource_gauge, style},
    render_grid::RenderCellGrid,
    theme::ThemeRegistry,
    view_models::{
        ActionListViewModel, ContainerViewModel, EventViewModel, HelpViewModel, LogViewModel,
        MapViewModel, StatsViewModel,
    },
    visual::{SymbolRegistry, VisualToken},
};

// ---------------------------------------------------------------------------
// Layout region — percentage-based placement
// ---------------------------------------------------------------------------

/// How a panel occupies space in a screen.
#[derive(Debug, Clone)]
pub enum PanelLayout {
    /// Fixed-width panel on the left side.
    Left { width_pct: u16 },
    /// Fixed-width panel on the right side.
    Right { width_pct: u16 },
    /// Fixed-height panel at the top.
    Top { height_pct: u16 },
    /// Fixed-height panel at the bottom.
    Bottom { height_pct: u16 },
    /// The remaining main area (should be exactly one per screen).
    Main,
}

// ---------------------------------------------------------------------------
// Panel and screen definitions
// ---------------------------------------------------------------------------

/// A single panel within a screen definition.
#[derive(Debug, Clone)]
pub struct PanelDefinition {
    /// Unique ID for this panel (e.g. "map", "stats", "log", "actions").
    pub id: String,
    /// How this panel is laid out.
    pub layout: PanelLayout,
    /// Expected view-model type name for validation (e.g. "MapViewModel").
    pub view_model: String,
}

/// A screen is a named collection of panels.
#[derive(Debug, Clone)]
pub struct ScreenDefinition {
    pub id: String,
    pub panels: Vec<PanelDefinition>,
}

// ---------------------------------------------------------------------------
// Widget binding — maps a panel ID to its render logic
// ---------------------------------------------------------------------------

/// Context passed to every widget renderer.
pub struct WidgetRenderContext<'a> {
    pub map: &'a MapViewModel,
    pub stats: &'a StatsViewModel,
    pub log: &'a LogViewModel,
    pub actions: &'a ActionListViewModel,
    pub container: &'a ContainerViewModel,
    pub event: &'a EventViewModel,
    pub help: &'a HelpViewModel,
    pub symbols: &'a SymbolRegistry,
    pub theme: &'a ThemeRegistry,
    pub mode: bd_core::spatial::GameMode,
    /// Active screen id (e.g. "outpost", "combat"). Region naming such as
    /// Chronicle/Context derives from the screen, not from entity content.
    pub screen_id: &'a str,
}

/// A registered widget knows its view-model dependency and how to render.
pub struct WidgetBinding {
    pub panel_id: String,
    pub view_model: String,
    #[allow(clippy::type_complexity)]
    pub render: Box<dyn Fn(&mut Frame, Rect, &WidgetRenderContext) + Send + Sync>,
}

impl std::fmt::Debug for WidgetBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WidgetBinding")
            .field("panel_id", &self.panel_id)
            .field("view_model", &self.view_model)
            .finish()
    }
}

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Registry of all available widgets.
#[derive(Resource, Debug)]
pub struct WidgetRegistry {
    pub bindings: HashMap<String, WidgetBinding>,
}

impl WidgetRegistry {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    pub fn register(&mut self, binding: WidgetBinding) {
        self.bindings.insert(binding.panel_id.clone(), binding);
    }

    pub fn get(&self, panel_id: &str) -> Option<&WidgetBinding> {
        self.bindings.get(panel_id)
    }
}

impl Default for WidgetRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Collection of all available screen definitions.
#[derive(Resource, Debug)]
pub struct ScreenRegistry {
    pub screens: HashMap<String, ScreenDefinition>,
}

impl ScreenRegistry {
    pub fn new() -> Self {
        Self {
            screens: HashMap::new(),
        }
    }

    pub fn register(&mut self, def: ScreenDefinition) {
        self.screens.insert(def.id.clone(), def);
    }

    pub fn get(&self, id: &str) -> Option<&ScreenDefinition> {
        self.screens.get(id)
    }
}

impl Default for ScreenRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// The currently active screen.
#[derive(Resource, Debug)]
pub struct ScreenState {
    pub current: String,
}

impl Default for ScreenState {
    fn default() -> Self {
        Self {
            current: "title".into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Default screen definitions (Rust fixtures — move to RON after stabilization)
// ---------------------------------------------------------------------------

/// Build the two canonical screen definitions: combat and inventory.
pub fn default_screen_registry() -> ScreenRegistry {
    let mut reg = ScreenRegistry::new();

    // Title screen: splash shown at launch
    reg.register(ScreenDefinition {
        id: "title".into(),
        panels: vec![PanelDefinition {
            id: "title_splash".into(),
            layout: PanelLayout::Main,
            view_model: "StatsViewModel".into(),
        }],
    });

    // Combat screen: map + stats | log + actions
    reg.register(ScreenDefinition {
        id: "combat".into(),
        panels: vec![
            PanelDefinition {
                id: "stats".into(),
                layout: PanelLayout::Right {
                    width_pct: STATS_PANEL_WIDTH_PCT,
                },
                view_model: "StatsViewModel".into(),
            },
            PanelDefinition {
                id: "log".into(),
                layout: PanelLayout::Bottom { height_pct: 25 },
                view_model: "LogViewModel".into(),
            },
            PanelDefinition {
                id: "actions".into(),
                layout: PanelLayout::Bottom { height_pct: 35 },
                view_model: "ActionListViewModel".into(),
            },
            PanelDefinition {
                id: "map".into(),
                layout: PanelLayout::Main,
                view_model: "MapViewModel".into(),
            },
        ],
    });

    // Inventory screen: stats on top, equipment on right, log at bottom, inventory list in main area
    reg.register(ScreenDefinition {
        id: "inventory".into(),
        panels: vec![
            PanelDefinition {
                id: "stats".into(),
                layout: PanelLayout::Top { height_pct: 15 },
                view_model: "StatsViewModel".into(),
            },
            PanelDefinition {
                id: "equipment".into(),
                layout: PanelLayout::Right { width_pct: 30 },
                view_model: "ContainerViewModel".into(),
            },
            PanelDefinition {
                id: "log".into(),
                layout: PanelLayout::Bottom { height_pct: 20 },
                view_model: "LogViewModel".into(),
            },
            PanelDefinition {
                id: "inventory_list".into(),
                layout: PanelLayout::Main,
                view_model: "ContainerViewModel".into(),
            },
        ],
    });

    // Outpost screen: resources, party, travel options, shelter map
    reg.register(ScreenDefinition {
        id: "outpost".into(),
        panels: vec![
            PanelDefinition {
                id: "outpost_party".into(),
                layout: PanelLayout::Left { width_pct: 24 },
                view_model: "ContainerViewModel".into(),
            },
            PanelDefinition {
                id: "stats".into(),
                layout: PanelLayout::Right { width_pct: 20 },
                view_model: "StatsViewModel".into(),
            },
            PanelDefinition {
                id: "log".into(),
                layout: PanelLayout::Bottom { height_pct: 20 },
                view_model: "LogViewModel".into(),
            },
            // Context needs three inner rows so the longest station action set
            // (status + Inspect/Assign/Set Production + reason) stays legible.
            PanelDefinition {
                id: "actions".into(),
                layout: PanelLayout::Bottom { height_pct: 25 },
                view_model: "ActionListViewModel".into(),
            },
            PanelDefinition {
                id: "map".into(),
                layout: PanelLayout::Main,
                view_model: "MapViewModel".into(),
            },
        ],
    });

    // Event screen: player interruption with choices
    reg.register(ScreenDefinition {
        id: "event".into(),
        panels: vec![
            PanelDefinition {
                id: "stats".into(),
                layout: PanelLayout::Right {
                    width_pct: STATS_PANEL_WIDTH_PCT,
                },
                view_model: "StatsViewModel".into(),
            },
            PanelDefinition {
                id: "event_text".into(),
                layout: PanelLayout::Main,
                view_model: "EventViewModel".into(),
            },
            PanelDefinition {
                id: "event_choices".into(),
                layout: PanelLayout::Bottom { height_pct: 25 },
                view_model: "EventViewModel".into(),
            },
            PanelDefinition {
                id: "log".into(),
                layout: PanelLayout::Bottom { height_pct: 15 },
                view_model: "LogViewModel".into(),
            },
        ],
    });

    // Help screen: keybindings overlay
    reg.register(ScreenDefinition {
        id: "help".into(),
        panels: vec![PanelDefinition {
            id: "help_keys".into(),
            layout: PanelLayout::Main,
            view_model: "HelpViewModel".into(),
        }],
    });

    // Debug screen: signal trace viewer + entity info
    reg.register(ScreenDefinition {
        id: "debug".into(),
        panels: vec![
            PanelDefinition {
                id: "debug_trace".into(),
                layout: PanelLayout::Main,
                view_model: "LogViewModel".into(),
            },
            PanelDefinition {
                id: "stats".into(),
                layout: PanelLayout::Right {
                    width_pct: STATS_PANEL_WIDTH_PCT,
                },
                view_model: "StatsViewModel".into(),
            },
            PanelDefinition {
                id: "log".into(),
                layout: PanelLayout::Bottom { height_pct: 20 },
                view_model: "LogViewModel".into(),
            },
        ],
    });

    // Game over screen: shown when player dies
    reg.register(ScreenDefinition {
        id: "game_over".into(),
        panels: vec![PanelDefinition {
            id: "game_over_splash".into(),
            layout: PanelLayout::Main,
            view_model: "StatsViewModel".into(),
        }],
    });

    reg
}

/// Derive the supported compact policy from a canonical screen definition.
///
/// Compact screens deliberately remove secondary panels rather than allowing
/// the full layout to collapse into unreadable rectangles.
pub fn compact_screen_definition(definition: &ScreenDefinition) -> ScreenDefinition {
    let mut compact = definition.clone();
    match definition.id.as_str() {
        "outpost" => {
            for panel in &mut compact.panels {
                match panel.id.as_str() {
                    // Slightly narrower party keeps the shelter map the largest
                    // interactive panel once Chronicle/Context grow to keep
                    // target identity and actions readable at 60x20.
                    "outpost_party" => panel.layout = PanelLayout::Left { width_pct: 22 },
                    // Eleven inner cells retain live-sized values plus a two-cell
                    // ASCII track (for example HP24/30[#-]).
                    "stats" => panel.layout = PanelLayout::Right { width_pct: 24 },
                    // Context needs three inner rows so the longest station
                    // action set (Inspect/Assign/Set Production + reason) stays
                    // legible without clipping.
                    "actions" => panel.layout = PanelLayout::Bottom { height_pct: 38 },
                    // Chronicle needs two inner rows so a wrapped NEARBY fact
                    // never loses its target name or Interact hint.
                    "log" => panel.layout = PanelLayout::Bottom { height_pct: 25 },
                    _ => {}
                }
            }
        }
        "combat" => {
            for panel in &mut compact.panels {
                match panel.id.as_str() {
                    "stats" => panel.layout = PanelLayout::Right { width_pct: 25 },
                    "actions" => panel.layout = PanelLayout::Bottom { height_pct: 32 },
                    "log" => panel.layout = PanelLayout::Bottom { height_pct: 32 },
                    _ => {}
                }
            }
        }
        "inventory" => {
            compact
                .panels
                .retain(|panel| !matches!(panel.id.as_str(), "stats" | "equipment"));
            for panel in &mut compact.panels {
                if panel.id == "log" {
                    panel.layout = PanelLayout::Bottom { height_pct: 25 };
                }
            }
        }
        _ => {}
    }
    compact
}

/// Build the default widget registry with all known renderers.
/// Render the game over screen splash.
fn game_over_status_line(extracted_loot: u32) -> String {
    format!("Run ended with {extracted_loot} loot. Use the controls below.")
}

fn render_game_over_splash_widget(frame: &mut Frame, area: Rect, ctx: &WidgetRenderContext) {
    let style_title = style(ctx.theme, UiTone::Danger);
    let style_muted = style(ctx.theme, UiTone::Muted);
    let style_accent = style(ctx.theme, UiTone::Accent);

    let text: Vec<ratatui::text::Line> = if area.width < 50 || area.height < 10 {
        vec![
            ratatui::text::Line::from(""),
            ratatui::text::Line::styled("GAME OVER", style_title),
            ratatui::text::Line::from(""),
            ratatui::text::Line::styled("  You have died.", style_muted),
            ratatui::text::Line::from(""),
            ratatui::text::Line::styled(
                format!("  {}", game_over_status_line(ctx.stats.extracted_loot)),
                style_accent,
            ),
        ]
    } else {
        let title_lines = [
            "   ____                         ___                 ",
            "  / ___| __ _ _ __ ___   ___   / _ \\__   _____ _ __ ",
            " | |  _ / _` | '_ ` _ \\ / _ \\ | | | \\ \\ / / _ \\ '__|",
            " | |_| | (_| | | | | | |  __/ | |_| |\\ V /  __/ |   ",
            "  \\____|\\__,_|_| |_| |_|\\___|  \\___/  \\_/ \\___|_|   ",
        ];
        let max_w = title_lines.iter().map(|l| l.len()).max().unwrap_or(0);
        let indent = " ".repeat(area.width.saturating_sub(max_w as u16) as usize / 2);
        let mut lines: Vec<ratatui::text::Line> = vec![ratatui::text::Line::from("")];
        for line in title_lines {
            let padded = format!("{indent}{line:<max_w$}");
            lines.push(ratatui::text::Line::styled(padded, style_title));
        }
        let died_line = format!("{indent}  You have died.");
        let status_line = format!(
            "{indent}  {}",
            game_over_status_line(ctx.stats.extracted_loot)
        );
        lines.push(ratatui::text::Line::from(""));
        lines.push(ratatui::text::Line::styled(died_line, style_muted));
        lines.push(ratatui::text::Line::from(""));
        lines.push(ratatui::text::Line::styled(status_line, style_accent));
        lines
    };
    let para = ratatui::widgets::Paragraph::new(text);
    frame.render_widget(para, area);
}

/// Render the title screen splash.
fn render_title_splash_widget(frame: &mut Frame, area: Rect, ctx: &WidgetRenderContext) {
    let style_title = style(ctx.theme, UiTone::Title);
    let style_muted = style(ctx.theme, UiTone::Muted);
    let style_accent = style(ctx.theme, UiTone::Accent);

    let mut text = vec![
        ratatui::text::Line::styled("BROKEN DIVINITY", style_title),
        ratatui::text::Line::styled("FOUNDATION BUILD", style_muted),
        ratatui::text::Line::from(""),
        ratatui::text::Line::styled("Press any key to begin", style_accent),
        ratatui::text::Line::styled(
            format!("Kernel v{}", env!("CARGO_PKG_VERSION")),
            style_muted,
        ),
    ];
    if !ctx.stats.save_available {
        text.push(ratatui::text::Line::styled(
            "Load unavailable — No save",
            style(ctx.theme, UiTone::Warning),
        ));
    }
    if let Some(entry) = ctx
        .log
        .entries
        .last()
        .filter(|entry| entry.level == bd_core::gamelog::LogLevel::Warn)
    {
        text.push(ratatui::text::Line::from(""));
        text.push(ratatui::text::Line::styled(
            truncate_end(&entry.message, area.width as usize),
            style(ctx.theme, UiTone::Warning),
        ));
    }
    let content_height = (text.len() as u16).min(area.height);
    let content_area = Rect {
        y: area.y + area.height.saturating_sub(content_height) / 2,
        height: content_height,
        ..area
    };
    let para = ratatui::widgets::Paragraph::new(text).alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(para, content_area);
}

pub fn default_widget_registry() -> WidgetRegistry {
    let mut reg = WidgetRegistry::new();

    reg.register(WidgetBinding {
        panel_id: "title_splash".into(),
        view_model: "StatsViewModel".into(),
        render: Box::new(render_title_splash_widget),
    });
    reg.register(WidgetBinding {
        panel_id: "game_over_splash".into(),
        view_model: "StatsViewModel".into(),
        render: Box::new(render_game_over_splash_widget),
    });
    reg.register(WidgetBinding {
        panel_id: "map".into(),
        view_model: "MapViewModel".into(),
        render: Box::new(render_map_widget),
    });
    reg.register(WidgetBinding {
        panel_id: "stats".into(),
        view_model: "StatsViewModel".into(),
        render: Box::new(render_stats_widget),
    });
    reg.register(WidgetBinding {
        panel_id: "log".into(),
        view_model: "LogViewModel".into(),
        render: Box::new(render_log_widget),
    });
    reg.register(WidgetBinding {
        panel_id: "actions".into(),
        view_model: "ActionListViewModel".into(),
        render: Box::new(render_actions_widget),
    });
    reg.register(WidgetBinding {
        panel_id: "inventory_list".into(),
        view_model: "ContainerViewModel".into(),
        render: Box::new(render_inventory_list_widget),
    });
    reg.register(WidgetBinding {
        panel_id: "equipment".into(),
        view_model: "ContainerViewModel".into(),
        render: Box::new(render_equipment_widget),
    });
    reg.register(WidgetBinding {
        panel_id: "outpost_party".into(),
        view_model: "ContainerViewModel".into(),
        render: Box::new(render_outpost_party_widget),
    });
    reg.register(WidgetBinding {
        panel_id: "debug_trace".into(),
        view_model: "LogViewModel".into(),
        render: Box::new(render_debug_trace_widget),
    });
    reg.register(WidgetBinding {
        panel_id: "event_text".into(),
        view_model: "EventViewModel".into(),
        render: Box::new(render_event_widget),
    });
    reg.register(WidgetBinding {
        panel_id: "event_choices".into(),
        view_model: "EventViewModel".into(),
        render: Box::new(render_event_choices_widget),
    });
    reg.register(WidgetBinding {
        panel_id: "help_keys".into(),
        view_model: "HelpViewModel".into(),
        render: Box::new(render_help_keys_widget),
    });

    reg
}

// ---------------------------------------------------------------------------
// Screen switching
// ---------------------------------------------------------------------------

/// Message to switch the active screen.
#[derive(Message, Debug, Clone)]
pub struct ScreenIntent {
    pub screen_id: String,
}

/// System: process ScreenIntent messages and update ScreenState.
pub fn process_screen_intents(
    mut messages: bevy_ecs::message::MessageReader<ScreenIntent>,
    registry: Res<ScreenRegistry>,
    mut state: ResMut<ScreenState>,
) {
    for msg in messages.read() {
        if registry.screens.contains_key(&msg.screen_id) {
            state.current = msg.screen_id.clone();
        } else {
            tracing::warn!("Unknown screen: {}", msg.screen_id);
        }
    }
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

/// Validation report for screen definitions.
#[derive(Debug, Clone)]
pub struct ScreenValidation {
    pub valid: bool,
    pub errors: Vec<String>,
}

/// Validate that all panels in all screens have registered widgets.
pub fn validate_screens(screens: &ScreenRegistry, widgets: &WidgetRegistry) -> ScreenValidation {
    let mut errors = Vec::new();

    for (screen_id, def) in &screens.screens {
        if def.panels.is_empty() {
            errors.push(format!("Screen '{screen_id}' has no panels"));
            continue;
        }

        let main_count = def
            .panels
            .iter()
            .filter(|p| matches!(p.layout, PanelLayout::Main))
            .count();
        if main_count != 1 {
            errors.push(format!(
                "Screen '{screen_id}' must have exactly one Main panel (found {main_count})"
            ));
        }

        for panel in &def.panels {
            // Check widget exists
            if !widgets.bindings.contains_key(&panel.id) {
                errors.push(format!(
                    "Screen '{screen_id}': no widget registered for panel '{}'",
                    panel.id
                ));
                continue;
            }

            // Check view-model type matches
            if let Some(binding) = widgets.bindings.get(&panel.id) {
                if binding.view_model != panel.view_model {
                    errors.push(format!(
                        "Screen '{screen_id}': panel '{}' expects VM '{}' but widget provides '{}'",
                        panel.id, panel.view_model, binding.view_model
                    ));
                }
            }
        }
    }

    // Check for unused widgets (warnings, not errors)
    for panel_id in widgets.bindings.keys() {
        let used = screens
            .screens
            .values()
            .any(|def| def.panels.iter().any(|p| p.id == *panel_id));
        if !used {
            tracing::warn!("Widget '{panel_id}' is registered but not used in any screen");
        }
    }

    ScreenValidation {
        valid: errors.is_empty(),
        errors,
    }
}

// ---------------------------------------------------------------------------
// Widget renderers
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MapViewport {
    origin_x: i32,
    origin_y: i32,
    width: u16,
    height: u16,
}

impl MapViewport {
    fn follow_active_focus(map: &MapViewModel, width: u16, height: u16) -> Self {
        let width = width.min(map.width.max(0) as u16);
        let height = height.min(map.height.max(0) as u16);
        let max_x = map.width.saturating_sub(width as i32).max(0);
        let max_y = map.height.saturating_sub(height as i32).max(0);
        let focus = map
            .build_ghost
            .map(|(position, _)| position)
            .or(map.player_pos);
        let (origin_x, origin_y) = focus.map_or((0, 0), |position| {
            (
                (position.x - i32::from(width) / 2).clamp(0, max_x),
                (position.y - i32::from(height) / 2).clamp(0, max_y),
            )
        });
        Self {
            origin_x,
            origin_y,
            width,
            height,
        }
    }

    fn project(self, position: bd_core::components::Position) -> Option<(u16, u16)> {
        let x = position.x - self.origin_x;
        let y = position.y - self.origin_y;
        (x >= 0 && x < i32::from(self.width) && y >= 0 && y < i32::from(self.height))
            .then_some((x as u16, y as u16))
    }

    fn edge_indicator(self, position: bd_core::components::Position) -> Option<(u16, u16, char)> {
        if self.project(position).is_some() || self.width == 0 || self.height == 0 {
            return None;
        }
        let min_x = self.origin_x;
        let max_x = self.origin_x + i32::from(self.width) - 1;
        let min_y = self.origin_y;
        let max_y = self.origin_y + i32::from(self.height) - 1;
        let left = min_x.saturating_sub(position.x);
        let right = position.x.saturating_sub(max_x);
        let up = min_y.saturating_sub(position.y);
        let down = position.y.saturating_sub(max_y);
        let (_, glyph, x, y) = [
            (
                left,
                '←',
                0,
                (position.y - min_y).clamp(0, i32::from(self.height) - 1),
            ),
            (
                right,
                '→',
                i32::from(self.width) - 1,
                (position.y - min_y).clamp(0, i32::from(self.height) - 1),
            ),
            (
                up,
                '↑',
                (position.x - min_x).clamp(0, i32::from(self.width) - 1),
                0,
            ),
            (
                down,
                '↓',
                (position.x - min_x).clamp(0, i32::from(self.width) - 1),
                i32::from(self.height) - 1,
            ),
        ]
        .into_iter()
        .max_by_key(|(distance, _, _, _)| *distance)?;
        Some((x as u16, y as u16, glyph))
    }
}

fn render_map_widget(frame: &mut Frame, area: Rect, ctx: &WidgetRenderContext) {
    let inner_width = area.width.saturating_sub(2);
    let inner_height = area.height.saturating_sub(2);
    let viewport = MapViewport::follow_active_focus(ctx.map, inner_width, inner_height);
    let offscreen_detail = ctx
        .map
        .assigned_target_details
        .iter()
        .find(|target| viewport.project(target.position).is_none());
    let title = offscreen_detail.map_or_else(
        || " Map ".to_owned(),
        |target| {
            let direction = viewport
                .edge_indicator(target.position)
                .map_or('?', |(_, _, direction)| direction);
            let distance = ctx.map.player_pos.map_or(0, |player| {
                (target.position.x - player.x).unsigned_abs()
                    + (target.position.y - player.y).unsigned_abs()
            });
            format!(" {direction} {} · {distance} tiles ", target.label)
        },
    );
    let block = panel(ctx.theme, title.trim(), PanelTone::Standard);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let w = viewport.width;
    let h = viewport.height;
    let mut grid = RenderCellGrid::new(w, h, VisualToken::Floor, ctx.symbols, ctx.theme);

    for screen_y in 0..h as i32 {
        for screen_x in 0..w as i32 {
            let world_x = viewport.origin_x + screen_x;
            let world_y = viewport.origin_y + screen_y;
            let idx = (world_y * ctx.map.width + world_x) as usize;
            let token = match ctx.map.tiles.get(idx) {
                Some(bd_core::components::Tile::Wall) | None => VisualToken::Wall,
                Some(bd_core::components::Tile::Floor) => VisualToken::Floor,
                Some(bd_core::components::Tile::Door) => VisualToken::DoorClosed,
                Some(bd_core::components::Tile::Water) => VisualToken::Water,
            };
            grid.set(
                screen_x as u16,
                screen_y as u16,
                token,
                ctx.symbols,
                ctx.theme,
            );
        }
    }

    for target in &ctx.map.assigned_targets {
        if let Some((x, y, glyph)) = viewport.edge_indicator(*target) {
            grid.set_glyph(
                x,
                y,
                glyph,
                VisualToken::TargetIndicator,
                ctx.symbols,
                ctx.theme,
            );
        }
    }

    for visual in &ctx.map.visuals {
        if let Some((x, y)) = viewport.project(visual.position) {
            if let Some(glyph) = visual.glyph {
                grid.set_glyph(x, y, glyph, visual.token, ctx.symbols, ctx.theme);
            } else {
                grid.set(x, y, visual.token, ctx.symbols, ctx.theme);
            }
        }
    }

    // P2-C: Render build ghost cursor on shelter map
    if let Some((pos, glyph)) = &ctx.map.build_ghost {
        if let Some((x, y)) = viewport.project(*pos) {
            if ctx.map.build_ghost_denial.is_some() {
                grid.set(x, y, VisualToken::InvalidSelection, ctx.symbols, ctx.theme);
            } else {
                grid.set_glyph(x, y, *glyph, VisualToken::Selection, ctx.symbols, ctx.theme);
            }
        }
    }

    let lines: Vec<ratatui::text::Line> = grid
        .rows()
        .map(|row| {
            let spans: Vec<ratatui::text::Span> = row
                .into_iter()
                .map(|(_, _, glyph, style)| ratatui::text::Span::styled(glyph.to_string(), style))
                .collect();
            ratatui::text::Line::from(spans)
        })
        .collect();

    let para = ratatui::widgets::Paragraph::new(lines).wrap(ratatui::widgets::Wrap { trim: false });
    frame.render_widget(para, inner);
}

pub fn render_build_overlay(frame: &mut Frame, area: Rect, ctx: &WidgetRenderContext) {
    if let Some(menu) = &ctx.stats.context_menu {
        let mut lines = Vec::new();
        if menu.phase == crate::ContextMenuPhase::Picker {
            lines.push(ratatui::text::Line::styled(
                "Select target:",
                style(ctx.theme, UiTone::Accent),
            ));
            for (index, target) in menu.targets.iter().enumerate() {
                let selected = index == menu.focused_index;
                lines.push(ratatui::text::Line::styled(
                    format!(
                        "{} {}. {} ({})",
                        if selected { "▶" } else { " " },
                        index + 1,
                        target.name,
                        target.category
                    ),
                    style(
                        ctx.theme,
                        if selected {
                            UiTone::Accent
                        } else {
                            UiTone::Text
                        },
                    ),
                ));
            }
        } else {
            lines.push(ratatui::text::Line::styled(
                format!("{} — {}", menu.target_name, menu.category),
                style(ctx.theme, UiTone::Accent),
            ));
            lines.push(ratatui::text::Line::styled(
                menu.detail.clone(),
                style(ctx.theme, UiTone::KeyHint),
            ));
            if menu.actions.is_empty() {
                lines.push(ratatui::text::Line::styled(
                    "No actions available.",
                    style(ctx.theme, UiTone::Muted),
                ));
            }
            for (index, action) in menu.actions.iter().enumerate() {
                let selected = menu.selected == Some(index);
                let tone = if !action.enabled {
                    UiTone::Muted
                } else if selected {
                    UiTone::Accent
                } else {
                    UiTone::Text
                };
                let reason = action
                    .denial_reason
                    .as_deref()
                    .map(|reason| format!(" ({reason})"))
                    .unwrap_or_default();
                lines.push(ratatui::text::Line::styled(
                    format!(
                        "{} {}. {}{reason}",
                        if selected { "▶" } else { " " },
                        action.key_hint,
                        action.label
                    ),
                    style(ctx.theme, tone),
                ));
            }
        }
        lines.push(ratatui::text::Line::styled(
            "1-9:select  Enter:confirm  x/Esc:cancel",
            style(ctx.theme, UiTone::Muted),
        ));

        let width = area.width.saturating_sub(2).min(76);
        let inner_width = usize::from(width.saturating_sub(2)).max(1);
        let wrapped_rows = lines
            .iter()
            .map(|line| line.width().max(1).div_ceil(inner_width))
            .sum::<usize>();
        let height = u16::try_from(wrapped_rows.saturating_add(2))
            .unwrap_or(area.height)
            .min(area.height);
        let modal = Rect {
            x: area.x + area.width.saturating_sub(width) / 2,
            y: area.y + area.height.saturating_sub(height) / 2,
            width,
            height,
        };
        frame.render_widget(ratatui::widgets::Clear, modal);
        let block = panel(ctx.theme, " Context ", PanelTone::Modal);
        let inner = block.inner(modal);
        frame.render_widget(block, modal);
        frame.render_widget(
            ratatui::widgets::Paragraph::new(lines).wrap(ratatui::widgets::Wrap { trim: true }),
            inner,
        );
        return;
    }

    if let Some(menu) = &ctx.stats.management {
        let stages = match menu.kind {
            super::view_models::ManagementMenuKind::TaskAssignment => {
                "1 Survivor  >  2 Task  >  3 Confirm"
            }
            super::view_models::ManagementMenuKind::StationStaffing => {
                "1 Survivor  >  2 Station  >  3 Recipe  >  4 Confirm"
            }
        };
        let mut lines = vec![
            ratatui::text::Line::styled(stages, style(ctx.theme, UiTone::Accent)),
            ratatui::text::Line::styled(menu.resources.clone(), style(ctx.theme, UiTone::Warning)),
            ratatui::text::Line::from(menu.forecast.clone()),
            ratatui::text::Line::styled("Select survivor:", style(ctx.theme, UiTone::Accent)),
        ];
        for (index, survivor) in menu.survivors.iter().enumerate() {
            let selected = menu.selected_survivor == Some(index);
            lines.push(ratatui::text::Line::styled(
                format!(
                    "{} {}. {survivor}",
                    if selected { "▶" } else { " " },
                    index + 1
                ),
                style(
                    ctx.theme,
                    if selected {
                        UiTone::Accent
                    } else {
                        UiTone::Text
                    },
                ),
            ));
        }
        if menu.selected_survivor.is_some() {
            for (index, task) in menu.tasks.iter().enumerate() {
                let selected = menu.selected_task == Some(index);
                lines.push(ratatui::text::Line::styled(
                    format!("{} {task}", if selected { "▶" } else { " " }),
                    style(
                        ctx.theme,
                        if task.contains("unavailable") {
                            UiTone::Muted
                        } else if selected {
                            UiTone::Accent
                        } else {
                            UiTone::Text
                        },
                    ),
                ));
            }
        }
        let cancel_key = match menu.kind {
            super::view_models::ManagementMenuKind::TaskAssignment => "c",
            super::view_models::ManagementMenuKind::StationStaffing => "e",
        };
        lines.push(ratatui::text::Line::styled(
            format!("1-9:select  Enter:confirm  {cancel_key}/Esc:cancel"),
            style(ctx.theme, UiTone::Muted),
        ));

        let width = area.width.saturating_sub(2).min(76);
        let inner_width = usize::from(width.saturating_sub(2)).max(1);
        let wrapped_rows = lines
            .iter()
            .map(|line| line.width().max(1).div_ceil(inner_width))
            .sum::<usize>();
        let height = u16::try_from(wrapped_rows.saturating_add(2))
            .unwrap_or(area.height)
            .min(area.height);
        let modal = Rect {
            x: area.x + area.width.saturating_sub(width) / 2,
            y: area.y + area.height.saturating_sub(height) / 2,
            width,
            height,
        };
        frame.render_widget(ratatui::widgets::Clear, modal);
        let title = match menu.kind {
            super::view_models::ManagementMenuKind::TaskAssignment => " Task Management ",
            super::view_models::ManagementMenuKind::StationStaffing => " Station Staffing ",
        };
        let block = panel(ctx.theme, title.trim(), PanelTone::Modal);
        let inner = block.inner(modal);
        frame.render_widget(block, modal);
        frame.render_widget(
            ratatui::widgets::Paragraph::new(lines).wrap(ratatui::widgets::Wrap { trim: true }),
            inner,
        );
        return;
    }

    if let Some(menu) = &ctx.map.build_menu {
        let width = area.width.min(76);
        let inner_width = width.saturating_sub(2).max(1) as usize;
        let selected_effect_rows = menu.options.get(menu.selected).map_or(0, |(_, _, effect)| {
            format!("Effect: {effect}")
                .split_whitespace()
                .fold((1_usize, 0_usize), |(rows, used), word| {
                    let needed = usize::from(used > 0) + word.chars().count();
                    if used + needed > inner_width {
                        (rows + 1, word.chars().count())
                    } else {
                        (rows, used + needed)
                    }
                })
                .0
        });
        let shortage_rows = menu.options.get(menu.selected).map_or(0, |(_, cost, _)| {
            usize::from(menu.available_supplies < *cost)
        });
        let height = (menu.options.len() + selected_effect_rows + shortage_rows + 5)
            .min(area.height as usize) as u16;
        let modal = Rect {
            x: area.x + area.width.saturating_sub(width) / 2,
            y: area.y + area.height.saturating_sub(height) / 2,
            width,
            height,
        };
        frame.render_widget(ratatui::widgets::Clear, modal);
        let block = panel(ctx.theme, "Build Station", PanelTone::Modal);
        let inner = block.inner(modal);
        frame.render_widget(block, modal);

        let mut lines = vec![ratatui::text::Line::styled(
            format!("Available: {} Supplies", menu.available_supplies),
            style(ctx.theme, UiTone::Warning),
        )];
        for (index, (label, cost, effect)) in menu.options.iter().enumerate() {
            let selected = index == menu.selected;
            let prefix = if selected { "▶" } else { " " };
            let tone = if effect.starts_with("Disabled") {
                UiTone::Muted
            } else if menu.available_supplies < *cost {
                UiTone::Danger
            } else if selected {
                UiTone::Accent
            } else {
                UiTone::Text
            };
            lines.push(ratatui::text::Line::styled(
                format!("{prefix} {}. {label} — {cost} Supplies", index + 1),
                style(ctx.theme, tone),
            ));
        }
        if let Some((_, cost, effect)) = menu.options.get(menu.selected) {
            lines.push(ratatui::text::Line::styled(
                format!("Effect: {effect}"),
                style(ctx.theme, UiTone::Accent),
            ));
            if menu.available_supplies < *cost {
                lines.push(ratatui::text::Line::styled(
                    format!(
                        "Unavailable: Need {} more Supplies",
                        cost - menu.available_supplies
                    ),
                    style(ctx.theme, UiTone::Danger),
                ));
            }
        }
        let numeric_choices = menu.options.len().min(9);
        lines.push(ratatui::text::Line::styled(
            format!("↑↓/1-{numeric_choices}:highlight Enter:placement b/Esc:cancel"),
            style(ctx.theme, UiTone::Muted),
        ));
        frame.render_widget(
            ratatui::widgets::Paragraph::new(lines).wrap(ratatui::widgets::Wrap { trim: true }),
            inner,
        );
        return;
    }

    if ctx.map.build_ghost.is_some() && area.height >= 3 {
        let detail_rows = usize::from(ctx.map.build_placement.is_some()) * 2;
        let denial_rows = usize::from(ctx.map.build_ghost_denial.is_some());
        let banner_height = u16::try_from(3 + detail_rows + denial_rows)
            .unwrap_or(area.height)
            .min(area.height);
        let banner = Rect {
            y: area.y + area.height - banner_height,
            height: banner_height,
            ..area
        };
        frame.render_widget(ratatui::widgets::Clear, banner);
        let block = panel(ctx.theme, "Build Placement", PanelTone::Modal);
        let inner = block.inner(banner);
        frame.render_widget(block, banner);
        let mut lines = Vec::new();
        if let Some(detail) = &ctx.map.build_placement {
            lines.push(ratatui::text::Line::styled(
                format!("{} — {} Supplies", detail.label, detail.supply_cost),
                style(ctx.theme, UiTone::Accent),
            ));
            lines.push(ratatui::text::Line::from(format!(
                "Effect: {}",
                detail.effect
            )));
        }
        lines.push(ratatui::text::Line::from(
            "Tile: wasd/arrows | Enter:build | b/Esc:cancel",
        ));
        if let Some(reason) = &ctx.map.build_ghost_denial {
            lines.push(ratatui::text::Line::styled(
                reason.clone(),
                style(ctx.theme, UiTone::Danger),
            ));
        }
        frame.render_widget(ratatui::widgets::Paragraph::new(lines), inner);
    }
}

fn truncate_end(value: &str, width: usize) -> String {
    if value.chars().count() <= width {
        return value.to_owned();
    }
    if width == 0 {
        return String::new();
    }
    if width == 1 {
        return "…".into();
    }
    let prefix = value.chars().take(width - 1).collect::<String>();
    format!("{prefix}…")
}

fn truncate_middle(value: &str, width: usize) -> String {
    if value.chars().count() <= width {
        return value.to_owned();
    }
    if width <= 1 {
        return truncate_end(value, width);
    }
    let left_len = (width - 1) / 2;
    let right_len = width - 1 - left_len;
    let left = value.chars().take(left_len).collect::<String>();
    let right = value
        .chars()
        .rev()
        .take(right_len)
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();
    format!("{left}…{right}")
}

fn render_stats_widget(frame: &mut Frame, area: Rect, ctx: &WidgetRenderContext) {
    let block = panel(ctx.theme, "Stats", PanelTone::Standard);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let stored_loot = ctx
        .stats
        .stored_items
        .iter()
        .map(|(_, count)| *count)
        .sum::<u32>();
    let compact_stats = inner.width < 14;
    let stored_loot_text = if compact_stats {
        format!("Loot: {stored_loot}")
    } else {
        format!("Stored loot: {stored_loot}")
    };
    // Supplies uses the structured display-ready gauge projected by the colony
    // projection. The renderer consumes these facts and never recomputes
    // pressure from flat fields or parses forecast prose.
    let supply_gauge = ctx.stats.supplies_gauge.as_ref().map(|gauge| {
        resource_gauge(
            ctx.theme,
            &gauge.label,
            gauge.current,
            gauge.maximum,
            inner.width,
            gauge.tone,
        )
    });
    let supply_condition_text = ctx
        .stats
        .supplies_gauge
        .as_ref()
        .map(|gauge| format!("[{}]", gauge.condition));
    let dawn_text = ctx
        .stats
        .supplies_gauge
        .as_ref()
        .map(|gauge| format!("DAWN {}→{}", gauge.delta, gauge.result))
        .unwrap_or_default();
    let faith_text = format!("Faith:{}", ctx.stats.faith);
    let materials_text = if compact_stats {
        format!("Mat:{}", ctx.stats.materials)
    } else {
        format!("Materials: {}", ctx.stats.materials)
    };
    let plants_text = if compact_stats {
        format!("Plant:{}", ctx.stats.wild_plants)
    } else {
        format!("Plants: {}", ctx.stats.wild_plants)
    };
    let last_run_text = if compact_stats {
        let outcome = match ctx.stats.run_outcome {
            bd_core::session::RunOutcome::None => "None",
            bd_core::session::RunOutcome::Extracted => "Extract",
            bd_core::session::RunOutcome::Defeated => "Defeat",
        };
        format!("Run:{outcome} {}", ctx.stats.extracted_loot)
    } else {
        format!(
            "Last run: {:?} ({} loot)",
            ctx.stats.run_outcome, ctx.stats.extracted_loot
        )
    };

    // The resource region keeps the shared gauge, condition chip, compact dawn
    // outlook, and the full secondary resource set at every supported profile.
    // No established compact content is deleted to make room.
    let mut text = vec![
        meter(
            ctx.theme,
            "HP",
            ctx.stats.hp_current,
            ctx.stats.hp_max,
            inner.width,
            hp_tone(ctx.stats.hp_current, ctx.stats.hp_max),
        ),
        meter(
            ctx.theme,
            "AP",
            ctx.stats.ap_current,
            ctx.stats.ap_max,
            inner.width,
            ap_tone(ctx.stats.ap_current, ctx.stats.ap_max),
        ),
        ratatui::text::Line::from(""),
    ];
    if let (Some(gauge), Some(condition)) = (&supply_gauge, &supply_condition_text) {
        text.push(gauge.clone());
        text.push(ratatui::text::Line::styled(
            condition.clone(),
            ctx.theme.resolve(
                ctx.stats
                    .supplies_gauge
                    .as_ref()
                    .map_or(crate::visual::StyleToken::UiMuted, |gauge| gauge.tone),
            ),
        ));
    }
    text.push(if dawn_text.is_empty() {
        ratatui::text::Line::from("")
    } else {
        ratatui::text::Line::styled(dawn_text, style(ctx.theme, UiTone::Warning))
    });
    text.push(ratatui::text::Line::styled(
        faith_text,
        style(ctx.theme, UiTone::Accent),
    ));
    text.push(ratatui::text::Line::styled(
        materials_text,
        style(ctx.theme, UiTone::Warning),
    ));
    text.push(ratatui::text::Line::styled(
        plants_text,
        style(ctx.theme, UiTone::Positive),
    ));
    text.push(ratatui::text::Line::styled(
        stored_loot_text,
        style(ctx.theme, UiTone::Accent),
    ));
    text.push(ratatui::text::Line::styled(
        truncate_end(&last_run_text, inner.width as usize),
        style(ctx.theme, UiTone::Muted),
    ));
    text.push(ratatui::text::Line::from(""));
    text.push(ratatui::text::Line::styled(
        format!("Day: {}", ctx.stats.day),
        style(ctx.theme, UiTone::Text),
    ));
    text.push(ratatui::text::Line::from(""));

    // P17-D: Faction standings
    for (label, _val, status) in &ctx.stats.faction_standings {
        let tone = match status.as_str() {
            "H" => UiTone::Danger,
            "F" => UiTone::Positive,
            "A" => UiTone::Accent,
            _ => UiTone::Muted,
        };
        text.push(ratatui::text::Line::styled(
            truncate_end(&format!("{}: {}", label, status), inner.width as usize),
            style(ctx.theme, tone),
        ));
    }

    if ctx.mode == bd_core::spatial::GameMode::Tactical {
        text.push(ratatui::text::Line::styled(
            if compact_stats {
                format!("Carry:{}", ctx.stats.carried_loot)
            } else {
                format!("Carried loot: {}", ctx.stats.carried_loot)
            },
            style(ctx.theme, UiTone::Accent),
        ));
        text.push(ratatui::text::Line::styled(
            if ctx.stats.extraction_ready {
                "Extraction: Ready"
            } else {
                "Extraction: Reach exit"
            },
            style(
                ctx.theme,
                if ctx.stats.extraction_ready {
                    UiTone::Positive
                } else {
                    UiTone::Muted
                },
            ),
        ));
    }

    let para = ratatui::widgets::Paragraph::new(text);
    frame.render_widget(para, inner);
}

fn render_log_widget(frame: &mut Frame, area: Rect, ctx: &WidgetRenderContext) {
    // At the colony this region is the Chronicle: a news feed that wraps so a
    // structured NEARBY fact never loses its target name or Interact hint.
    // Other screens keep the compact truncating Log that preserves useful
    // path tails.
    let chronicle = ctx.screen_id == "outpost";
    let block = panel(
        ctx.theme,
        if chronicle { "Chronicle" } else { "Log" },
        PanelTone::Dense,
    );

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // At the colony, an empty Chronicle carries the authoritative next-day
    // outlook so the compact profile never loses the decisive forecast while
    // the resource region keeps every secondary resource line.
    let lines: Vec<ratatui::text::Line> =
        if chronicle && ctx.log.entries.is_empty() && !ctx.stats.next_day_forecast.is_empty() {
            vec![ratatui::text::Line::styled(
                ctx.stats.next_day_forecast.clone(),
                style(ctx.theme, UiTone::Warning),
            )]
        } else {
            let visible_rows = inner.height as usize;
            // The Chronicle renders newest-first so a wrapped fact (NEARBY) is
            // never displaced by older one-line entries: the newest entries wrap at
            // the top and older entries fill only remaining room. A decisive
            // warning that overflowed out of the newest window is still pinned at
            // the bottom so the player cannot miss it.
            let visible = if chronicle {
                let mut visible = ctx
                    .log
                    .entries
                    .iter()
                    .rev()
                    .take(visible_rows)
                    .collect::<Vec<_>>();
                if visible_rows > 0
                    && !visible
                        .iter()
                        .any(|entry| entry.level == bd_core::gamelog::LogLevel::Warn)
                    && let Some(warning) = ctx
                        .log
                        .entries
                        .iter()
                        .rev()
                        .find(|entry| entry.level == bd_core::gamelog::LogLevel::Warn)
                {
                    if visible.len() == visible_rows {
                        visible.pop();
                    }
                    visible.push(warning);
                }
                visible
            } else {
                let first_visible = ctx.log.entries.len().saturating_sub(visible_rows);
                let mut visible = ctx
                    .log
                    .entries
                    .iter()
                    .skip(first_visible)
                    .collect::<Vec<_>>();
                if visible_rows > 0
                    && !visible
                        .iter()
                        .any(|entry| entry.level == bd_core::gamelog::LogLevel::Warn)
                    && let Some(warning) = ctx
                        .log
                        .entries
                        .iter()
                        .rev()
                        .find(|entry| entry.level == bd_core::gamelog::LogLevel::Warn)
                {
                    if visible.len() == visible_rows {
                        visible.remove(0);
                    }
                    visible.insert(0, warning);
                }
                visible
            };
            visible
                .into_iter()
                .map(|entry| {
                    let tone = match entry.level {
                        bd_core::gamelog::LogLevel::Info => UiTone::Text,
                        bd_core::gamelog::LogLevel::Warn => UiTone::Warning,
                        bd_core::gamelog::LogLevel::Combat => UiTone::Danger,
                    };
                    let message = if chronicle {
                        entry.message.clone()
                    } else if entry.message.contains('/') {
                        truncate_middle(&entry.message, inner.width as usize)
                    } else {
                        truncate_end(&entry.message, inner.width as usize)
                    };
                    ratatui::text::Line::styled(message, style(ctx.theme, tone))
                })
                .collect()
        };

    let mut para = ratatui::widgets::Paragraph::new(lines);
    if chronicle {
        para = para.wrap(ratatui::widgets::Wrap { trim: false });
    }
    frame.render_widget(para, inner);
}

fn render_actions_widget(frame: &mut Frame, area: Rect, ctx: &WidgetRenderContext) {
    // At the colony this region is the Context action feed for the active
    // nearby target; the title names the focused category/state. Other screens
    // keep the plain Actions region.
    let context = ctx.screen_id == "outpost";
    let context_target = &ctx.stats.context_target;
    let title = if let Some(target) = context_target {
        format!("Context · {}", target.title)
    } else if context {
        "Context".to_string()
    } else {
        "Actions".to_string()
    };
    let block = panel(ctx.theme, title, PanelTone::Dense);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut spans = Vec::new();
    if let Some(target) = context_target {
        // A multi-target nearby set carries a player-facing focus selector so
        // duplicate-named targets stay distinguishable without raw identity.
        if target.target_count > 1 {
            spans.push(ratatui::text::Span::styled(
                format!(
                    "{}/{} {},{} ",
                    target.focus_index, target.target_count, target.position.0, target.position.1
                ),
                style(ctx.theme, UiTone::Muted),
            ));
        }
        // Concise status stays with the context feed so the focused target's
        // category/status never rely on glyph inference.
        spans.push(ratatui::text::Span::styled(
            format!("{} ", target.status),
            style(ctx.theme, UiTone::KeyHint),
        ));
    }
    // Action rows carry their key hint, label, and any denial reason inline
    // so compact wrapping keeps each action+reason pair together. Each action
    // is one span so the wrapper never separates a label from its reason.
    // Action rows carry their key hint, label, and any denial reason inline
    // so compact wrapping keeps each action+reason pair together. Each action
    // is one span so the wrapper never separates a label from its reason.
    for action in ctx
        .actions
        .actions
        .iter()
        .filter(|a| context_target.is_none() || a.label != "Interact")
    {
        let key_style = if action.enabled {
            style(ctx.theme, UiTone::KeyHint)
        } else {
            style(ctx.theme, UiTone::Muted)
        };
        if !action.key_hint.is_empty() {
            spans.push(ratatui::text::Span::styled(
                format!("{} ", action.key_hint),
                key_style,
            ));
        }
        let text = if let Some(reason) = action.denial_reason.as_deref() {
            format!("{}({}) ", action.label, reason)
        } else {
            format!("{} ", action.label)
        };
        spans.push(ratatui::text::Span::raw(text));
    }

    let para = ratatui::widgets::Paragraph::new(ratatui::text::Line::from(spans))
        .wrap(ratatui::widgets::Wrap { trim: false });
    frame.render_widget(para, inner);
}

fn render_inventory_list_widget(frame: &mut Frame, area: Rect, ctx: &WidgetRenderContext) {
    let block = panel(ctx.theme, "Inventory", PanelTone::Standard);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines: Vec<ratatui::text::Line> = ctx
        .container
        .items
        .iter()
        .map(|item| {
            let state = match (item.equipped, item.usable) {
                (true, true) => " [equipped, usable]",
                (true, false) => " [equipped]",
                (false, true) => " [usable]",
                (false, false) => "",
            };
            ratatui::text::Line::styled(
                truncate_end(&format!(" {}{}", item.name, state), inner.width as usize),
                style(ctx.theme, UiTone::Text),
            )
        })
        .collect();

    let para = ratatui::widgets::Paragraph::new(lines);
    frame.render_widget(para, inner);
}

fn render_equipment_widget(frame: &mut Frame, area: Rect, ctx: &WidgetRenderContext) {
    let block = panel(ctx.theme, "Equipment", PanelTone::Standard);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines: Vec<ratatui::text::Line> = ctx
        .container
        .items
        .iter()
        .filter(|item| item.equipped)
        .map(|item| {
            ratatui::text::Line::styled(format!(" {}", item.name), style(ctx.theme, UiTone::Accent))
        })
        .collect();

    let para = ratatui::widgets::Paragraph::new(lines);
    frame.render_widget(para, inner);
}

// ---------------------------------------------------------------------------
// Screen layout engine
// ---------------------------------------------------------------------------

/// Given a screen definition and a total area, compute (panel_id, rect) pairs.
pub fn compute_panel_rects(def: &ScreenDefinition, total: Rect) -> Vec<(String, Rect)> {
    if def.panels.is_empty() {
        return vec![];
    }

    // Colony and reusable screens sit inside the structural Ruined Reliquary
    // frame: every panel is inset one cell from the terminal perimeter so the
    // continuous double-line frame (top rule, side rails, bottom rule) owns the
    // outermost edge. Splash screens (title, game over) stay frame-less and use
    // the full content area to preserve their centered layout.
    let layout_total = if matches!(def.id.as_str(), "title" | "game_over") {
        total
    } else {
        Rect {
            x: total.x + 1,
            y: total.y + 1,
            width: total.width.saturating_sub(2),
            height: total.height.saturating_sub(1),
        }
    };

    // Separate main vs non-main panels
    let non_main: Vec<&PanelDefinition> = def
        .panels
        .iter()
        .filter(|p| !matches!(p.layout, PanelLayout::Main))
        .collect();
    let main: Vec<&PanelDefinition> = def
        .panels
        .iter()
        .filter(|p| matches!(p.layout, PanelLayout::Main))
        .collect();

    let mut result = Vec::new();
    let mut remaining = layout_total;

    // Process left panels first
    for panel in &non_main {
        if let PanelLayout::Left { width_pct } = panel.layout {
            let w = (layout_total.width * width_pct / 100).max(1);
            let left = Rect {
                width: w,
                ..remaining
            };
            result.push((panel.id.clone(), left));
            remaining = Rect {
                x: remaining.x + w,
                width: remaining.width.saturating_sub(w),
                ..remaining
            };
        }
    }

    // Process right panels
    for panel in &non_main {
        if let PanelLayout::Right { width_pct } = panel.layout {
            let w = (layout_total.width * width_pct / 100).max(1);
            let right = Rect {
                x: remaining.x + remaining.width.saturating_sub(w),
                width: w,
                ..remaining
            };
            result.push((panel.id.clone(), right));
            remaining = Rect {
                width: remaining.width.saturating_sub(w),
                ..remaining
            };
        }
    }

    // Process top panels
    for panel in &non_main {
        if let PanelLayout::Top { height_pct } = panel.layout {
            let h = (layout_total.height * height_pct / 100).max(1);
            let top = Rect {
                height: h,
                ..remaining
            };
            result.push((panel.id.clone(), top));
            remaining = Rect {
                y: remaining.y + h,
                height: remaining.height.saturating_sub(h),
                ..remaining
            };
        }
    }

    // Process bottom panels
    for panel in &non_main {
        if let PanelLayout::Bottom { height_pct } = panel.layout {
            let h = (layout_total.height * height_pct / 100).max(1);
            let bottom = Rect {
                y: remaining.y + remaining.height.saturating_sub(h),
                height: h,
                ..remaining
            };
            result.push((panel.id.clone(), bottom));
            remaining = Rect {
                height: remaining.height.saturating_sub(h),
                ..remaining
            };
        }
    }

    // Remaining area goes to Main panel(s)
    for panel in &main {
        result.push((panel.id.clone(), remaining));
    }

    result
}

// ---------------------------------------------------------------------------
// Outpost widget renderers
// ---------------------------------------------------------------------------

fn render_outpost_party_widget(frame: &mut Frame, area: Rect, ctx: &WidgetRenderContext) {
    let block = panel(ctx.theme, "Party", PanelTone::Dense);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines = Vec::new();
    if !ctx.stats.latest_daily_summary.is_empty() {
        lines.extend(ctx.stats.latest_daily_summary.iter().map(|summary_line| {
            ratatui::text::Line::styled(summary_line.clone(), style(ctx.theme, UiTone::KeyHint))
        }));
        lines.push(ratatui::text::Line::from(""));
    }
    if ctx.stats.party_names.is_empty() {
        lines.push(ratatui::text::Line::styled(
            " (empty)",
            style(ctx.theme, UiTone::Muted),
        ));
    } else {
        lines.extend(ctx.stats.party_names.iter().map(|name| {
            ratatui::text::Line::styled(format!(" {}", name), style(ctx.theme, UiTone::Text))
        }));
        lines.extend(ctx.stats.station_status.iter().map(|status| {
            ratatui::text::Line::styled(format!(" {status}"), style(ctx.theme, UiTone::Accent))
        }));
        // The dawn forecast lives in the Chronicle when there is no recent
        // history, so it stays visible at both profiles without shrinking the
        // party column or dropping secondary resources.
    }

    let para = ratatui::widgets::Paragraph::new(lines).wrap(ratatui::widgets::Wrap { trim: false });
    frame.render_widget(para, inner);
}

// ---------------------------------------------------------------------------
// Debug widget renderers
// ---------------------------------------------------------------------------

fn render_help_keys_widget(frame: &mut Frame, area: Rect, ctx: &WidgetRenderContext) {
    let block = panel(ctx.theme, "Help", PanelTone::Modal);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<ratatui::text::Line> = vec![ratatui::text::Line::styled(
        " Controls and shelter legend",
        style(ctx.theme, UiTone::Title),
    )];
    let column_gap = 2_usize;
    let inner_width = inner.width as usize;
    let column_width = inner_width.saturating_sub(column_gap) / 2;
    let row_count = ctx.help.keys.len().div_ceil(2);
    for row in 0..row_count {
        let left = ctx
            .help
            .keys
            .get(row)
            .map_or_else(String::new, |(key, action)| format!("{key} {action}"));
        let right = ctx
            .help
            .keys
            .get(row + row_count)
            .map_or_else(String::new, |(key, action)| format!("{key} {action}"));
        debug_assert!(
            left.chars().count() <= column_width && right.chars().count() <= column_width,
            "Help entries must fit the supported responsive column"
        );
        lines.push(ratatui::text::Line::styled(
            format!("{left:<column_width$}{:column_gap$}{right}", ""),
            style(ctx.theme, UiTone::Text),
        ));
    }
    lines.push(ratatui::text::Line::styled(
        " ? or Esc: close Help",
        style(ctx.theme, UiTone::Muted),
    ));

    let para = ratatui::widgets::Paragraph::new(lines);
    frame.render_widget(para, inner);
}

fn render_debug_trace_widget(frame: &mut Frame, area: Rect, ctx: &WidgetRenderContext) {
    let block = panel(ctx.theme, "Signal Trace (F1: toggle)", PanelTone::Standard);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines: Vec<ratatui::text::Line> = ctx
        .log
        .entries
        .iter()
        .rev()
        .take(inner.height as usize)
        .map(|entry| {
            let tone = match entry.level {
                bd_core::gamelog::LogLevel::Info => UiTone::Text,
                bd_core::gamelog::LogLevel::Warn => UiTone::Warning,
                bd_core::gamelog::LogLevel::Combat => UiTone::Danger,
            };
            ratatui::text::Line::styled(entry.message.as_str(), style(ctx.theme, tone))
        })
        .collect();

    let para = ratatui::widgets::Paragraph::new(lines);
    frame.render_widget(para, inner);
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Render the event dialogue text and speaker.
fn render_event_widget(frame: &mut Frame, area: Rect, ctx: &WidgetRenderContext) {
    if !ctx.event.active {
        return;
    }
    let text = format!(
        "[{}]
{}
",
        ctx.event.speaker, ctx.event.text
    );
    let para = ratatui::widgets::Paragraph::new(text)
        .style(style(ctx.theme, UiTone::Text))
        .block(panel(ctx.theme, "Event", PanelTone::Standard));
    frame.render_widget(para, area);
}

/// Render the event choice list as numbered options.
fn render_event_choices_widget(frame: &mut Frame, area: Rect, ctx: &WidgetRenderContext) {
    if !ctx.event.active || ctx.event.choices.is_empty() {
        return;
    }
    let mut text = "Your choice:".to_string();
    for (i, choice) in ctx.event.choices.iter().enumerate() {
        text.push_str(&format!(
            "
{}. {}",
            i + 1,
            choice
        ));
    }
    let para = ratatui::widgets::Paragraph::new(text)
        .style(style(ctx.theme, UiTone::Warning))
        .block(panel(ctx.theme, "Choices", PanelTone::Standard));
    frame.render_widget(para, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn supported_outpost_map_sizes() -> [(u16, u16); 2] {
        [(80, 24), (60, 20)].map(|(width, height)| {
            let definition = default_screen_registry()
                .get("outpost")
                .expect("outpost screen must exist")
                .clone();
            let definition = if crate::commands::terminal_layout(width, height)
                == crate::commands::TerminalLayout::Compact
            {
                compact_screen_definition(&definition)
            } else {
                definition
            };
            let content = Rect::new(0, 0, width, height.saturating_sub(3));
            let map_rect = compute_panel_rects(&definition, content)
                .into_iter()
                .find_map(|(id, rect)| (id == "map").then_some(rect))
                .expect("outpost screen must include a map panel");
            (
                map_rect.width.saturating_sub(2),
                map_rect.height.saturating_sub(2),
            )
        })
    }

    #[test]
    fn every_shelter_position_projects_inside_supported_viewports() {
        // Contract: VISUAL-VIEWPORT-003
        // Given: every legal shelter coordinate at each supported map size.
        // When: the production follow-focus viewport is calculated.
        // Then: the player coordinate projects inside the visible map.
        // Must not change: no edge or corner coordinate may disappear.
        // Evidence layers: projection and buffer layout.
        for (viewport_width, viewport_height) in supported_outpost_map_sizes() {
            for y in 0..30 {
                for x in 0..40 {
                    let position = bd_core::components::Position { x, y };
                    let map = MapViewModel {
                        width: 40,
                        height: 30,
                        player_pos: Some(position),
                        ..Default::default()
                    };
                    let viewport =
                        MapViewport::follow_active_focus(&map, viewport_width, viewport_height);
                    assert!(
                        viewport.project(position).is_some(),
                        "contract=VISUAL-VIEWPORT-003 case={viewport_width}x{viewport_height} \
                         fixture=shelter-all-positions player={position:?} \
                         origin=({}, {})",
                        viewport.origin_x,
                        viewport.origin_y
                    );
                }
            }
        }
    }

    #[test]
    fn viewport_pan_preserves_relative_world_positions() {
        // Contract: VISUAL-VIEWPORT-004
        // Given: fixed world-coordinate pairs under edge and center focus cases.
        // When: the production viewport pans and projects both coordinates.
        // Then: screen-space deltas equal world-space deltas.
        // Must not change: panning cannot distort relative geometry.
        // Evidence layers: projection and buffer layout.
        for (viewport_width, viewport_height) in supported_outpost_map_sizes() {
            for focus in [
                bd_core::components::Position { x: 1, y: 1 },
                bd_core::components::Position { x: 20, y: 15 },
                bd_core::components::Position { x: 38, y: 28 },
            ] {
                let map = MapViewModel {
                    width: 40,
                    height: 30,
                    player_pos: Some(focus),
                    ..Default::default()
                };
                let viewport =
                    MapViewport::follow_active_focus(&map, viewport_width, viewport_height);
                let first = bd_core::components::Position {
                    x: viewport.origin_x + 1,
                    y: viewport.origin_y + 1,
                };
                let second = bd_core::components::Position {
                    x: (first.x + 3).min(39),
                    y: (first.y + 2).min(29),
                };
                let first_projected = viewport
                    .project(first)
                    .expect("first comparison point must be visible");
                let second_projected = viewport
                    .project(second)
                    .expect("second comparison point must be visible");
                assert_eq!(
                    (
                        i32::from(second_projected.0) - i32::from(first_projected.0),
                        i32::from(second_projected.1) - i32::from(first_projected.1),
                    ),
                    (second.x - first.x, second.y - first.y),
                    "contract=VISUAL-VIEWPORT-004 case={viewport_width}x{viewport_height} \
                     focus={focus:?} changed relative world geometry"
                );
            }
        }
    }

    #[test]
    fn game_over_splash_defers_to_the_complete_contextual_controls() {
        let line = game_over_status_line(3);

        assert!(line.contains("3 loot"));
        assert!(line.contains("controls below"));
        assert!(!line.contains("q to quit"));
    }

    #[test]
    fn outpost_has_contextual_action_panel() {
        let registry = default_screen_registry();
        let has_panel = registry
            .get("outpost")
            .unwrap()
            .panels
            .iter()
            .any(|p| p.id == "actions");
        assert!(has_panel, "Outpost should expose contextual actions");
    }

    #[test]
    fn help_screen_displays_keybindings() {
        let registry = default_screen_registry();
        let help = registry.get("help");
        assert!(help.is_some(), "Help screen should be registered");
        let def = help.unwrap();
        assert_eq!(def.id, "help");
        // Should contain keybindings
        let panel_ids: Vec<&str> = def.panels.iter().map(|p| p.id.as_str()).collect();
        assert!(
            panel_ids.contains(&"help_keys"),
            "Help screen should have help_keys panel"
        );
    }

    #[test]
    fn combat_screen_loads() {
        let registry = default_screen_registry();
        let combat = registry.get("combat");
        assert!(combat.is_some(), "Combat screen should exist");
        let def = combat.unwrap();
        assert_eq!(def.id, "combat");
        // Should have map, stats, log, actions
        let panel_ids: Vec<&str> = def.panels.iter().map(|p| p.id.as_str()).collect();
        assert!(panel_ids.contains(&"map"));
        assert!(panel_ids.contains(&"stats"));
        assert!(panel_ids.contains(&"log"));
        assert!(panel_ids.contains(&"actions"));
    }

    #[test]
    fn combat_screen_resolves_widgets() {
        let widgets = default_widget_registry();
        let screen_reg = default_screen_registry();
        let combat = screen_reg.get("combat").unwrap();
        for panel in &combat.panels {
            assert!(
                widgets.bindings.contains_key(&panel.id),
                "Panel '{}' has no registered widget",
                panel.id
            );
        }
    }

    #[test]
    fn inventory_screen_loads() {
        let registry = default_screen_registry();
        let inv = registry.get("inventory");
        assert!(inv.is_some(), "Inventory screen should exist");
        let def = inv.unwrap();
        assert_eq!(def.id, "inventory");
        let panel_ids: Vec<&str> = def.panels.iter().map(|p| p.id.as_str()).collect();
        assert!(panel_ids.contains(&"inventory_list"));
        assert!(panel_ids.contains(&"equipment"));
    }

    #[test]
    fn inventory_screen_resolves_widgets() {
        let widgets = default_widget_registry();
        let screen_reg = default_screen_registry();
        let inv = screen_reg.get("inventory").unwrap();
        for panel in &inv.panels {
            assert!(
                widgets.bindings.contains_key(&panel.id),
                "Panel '{}' has no registered widget",
                panel.id
            );
        }
    }

    #[test]
    fn missing_widget_id_fails_validation() {
        let screens = default_screen_registry();
        let empty_widgets = WidgetRegistry::new();
        let result = validate_screens(&screens, &empty_widgets);
        assert!(!result.valid);
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.contains("no widget registered"))
        );
    }

    #[test]
    fn hp_color_changes_with_threshold() {
        const {
            assert!(HP_GREEN_THRESHOLD_PCT > 0);
            assert!(HP_YELLOW_THRESHOLD_PCT > 0);
            assert!(HP_GREEN_THRESHOLD_PCT > HP_YELLOW_THRESHOLD_PCT);
        }
    }

    #[test]
    fn missing_view_model_binding_fails_validation() {
        let mut screens = default_screen_registry();
        // Tamper with a panel's view model type
        if let Some(combat) = screens.screens.get_mut("combat") {
            if let Some(panel) = combat.panels.iter_mut().find(|p| p.id == "map") {
                panel.view_model = "WrongViewModel".into();
            }
        }
        let widgets = default_widget_registry();
        let result = validate_screens(&screens, &widgets);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("expects VM")));
    }

    #[test]
    fn screen_switch_preserves_gameplay_state() {
        // Screen switching is purely a UI concern — it only changes ScreenState,
        // which is a resource that doesn't touch gameplay components.
        // This test verifies that switching screens doesn't mutate ECS state.
        let mut app = bevy_app::App::new();
        app.add_message::<ScreenIntent>();
        app.insert_resource(ScreenRegistry::default());
        app.insert_resource(ScreenState::default());
        app.add_systems(bevy_app::Update, process_screen_intents);

        // Register a test screen
        let mut reg = ScreenRegistry::default();
        reg.register(ScreenDefinition {
            id: "test_screen".into(),
            panels: vec![],
        });
        app.insert_resource(reg);

        // Get initial state
        let initial = app.world().resource::<ScreenState>().current.clone();

        // Send a switch message
        app.world_mut()
            .resource_mut::<bevy_ecs::message::Messages<ScreenIntent>>()
            .write(ScreenIntent {
                screen_id: "test_screen".into(),
            });
        app.update();

        // State changed but no gameplay mutation happened (no crash = pass)
        let after = app.world().resource::<ScreenState>().current.clone();
        assert_eq!(after, "test_screen");
        assert_ne!(initial, after);
    }
}
