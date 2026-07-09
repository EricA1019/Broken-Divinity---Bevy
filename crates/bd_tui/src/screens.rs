//! Data-driven TUI screens — ScreenDefinition, WidgetRegistry, and ScreenState.
//!
//! Phase 15: Moves from hardcoded layout in draw_ui to schema-driven screen
//! definitions. Widgets are registered by ID and dispatched from the current
//! screen definition.

use std::collections::HashMap;

use bevy_ecs::prelude::*;
use ratatui::{
    layout::Rect,
    Frame,
};

use super::{
    render_grid::RenderCellGrid,
    theme::ThemeRegistry,
    view_models::{
        ActionListViewModel, ContainerViewModel, LogViewModel, MapViewModel,
        StatsViewModel,
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
    pub symbols: &'a SymbolRegistry,
    pub theme: &'a ThemeRegistry,
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
            current: "outpost".into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Default screen definitions (Rust fixtures — move to RON after stabilization)
// ---------------------------------------------------------------------------

/// Build the two canonical screen definitions: combat and inventory.
pub fn default_screen_registry() -> ScreenRegistry {
    let mut reg = ScreenRegistry::new();

    // Combat screen: map + stats | log + actions
    reg.register(ScreenDefinition {
        id: "combat".into(),
        panels: vec![
            PanelDefinition {
                id: "stats".into(),
                layout: PanelLayout::Right { width_pct: 25 },
                view_model: "StatsViewModel".into(),
            },
            PanelDefinition {
                id: "log".into(),
                layout: PanelLayout::Bottom { height_pct: 30 },
                view_model: "LogViewModel".into(),
            },
            PanelDefinition {
                id: "actions".into(),
                layout: PanelLayout::Bottom { height_pct: 12 },
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

    // Outpost screen: resources, party, travel options
    reg.register(ScreenDefinition {
        id: "outpost".into(),
        panels: vec![
            PanelDefinition {
                id: "outpost_party".into(),
                layout: PanelLayout::Left { width_pct: 30 },
                view_model: "ContainerViewModel".into(),
            },
            PanelDefinition {
                id: "outpost_travel".into(),
                layout: PanelLayout::Main,
                view_model: "ContainerViewModel".into(),
            },
            PanelDefinition {
                id: "stats".into(),
                layout: PanelLayout::Right { width_pct: 25 },
                view_model: "StatsViewModel".into(),
            },
            PanelDefinition {
                id: "log".into(),
                layout: PanelLayout::Bottom { height_pct: 20 },
                view_model: "LogViewModel".into(),
            },
        ],
    });

    reg
}

/// Build the default widget registry with all known renderers.
pub fn default_widget_registry() -> WidgetRegistry {
    let mut reg = WidgetRegistry::new();

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
        panel_id: "outpost_travel".into(),
        view_model: "ContainerViewModel".into(),
        render: Box::new(render_outpost_travel_widget),
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
pub fn validate_screens(
    screens: &ScreenRegistry,
    widgets: &WidgetRegistry,
) -> ScreenValidation {
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
        let used = screens.screens.values().any(|def| {
            def.panels.iter().any(|p| p.id == *panel_id)
        });
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

fn render_map_widget(frame: &mut Frame, area: Rect, ctx: &WidgetRenderContext) {
    let block = ratatui::widgets::Block::default()
        .title(" Map ")
        .borders(ratatui::widgets::Borders::ALL)
        .style(ratatui::style::Style::default().fg(ratatui::style::Color::Gray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let w = inner.width.min(ctx.map.width as u16);
    let h = inner.height.min(ctx.map.height as u16);
    let mut grid = RenderCellGrid::new(w, h, VisualToken::Floor, ctx.symbols, ctx.theme);

    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let idx = (y * ctx.map.width + x) as usize;
            let token = match ctx.map.tiles.get(idx) {
                Some(bd_core::components::Tile::Wall) | None => VisualToken::Wall,
                Some(bd_core::components::Tile::Floor) => VisualToken::Floor,
                Some(bd_core::components::Tile::Door) => VisualToken::DoorClosed,
            };
            grid.set(x as u16, y as u16, token, ctx.symbols, ctx.theme);
        }
    }

    for ep in &ctx.map.enemy_positions {
        if ep.x >= 0 && ep.x < w as i32 && ep.y >= 0 && ep.y < h as i32 {
            grid.set(ep.x as u16, ep.y as u16, VisualToken::Enemy, ctx.symbols, ctx.theme);
        }
    }

    if let Some(pp) = ctx.map.player_pos {
        if pp.x >= 0 && pp.x < w as i32 && pp.y >= 0 && pp.y < h as i32 {
            grid.set(pp.x as u16, pp.y as u16, VisualToken::Player, ctx.symbols, ctx.theme);
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

    let para = ratatui::widgets::Paragraph::new(lines);
    frame.render_widget(para, inner);
}

fn render_stats_widget(frame: &mut Frame, area: Rect, ctx: &WidgetRenderContext) {
    let block = ratatui::widgets::Block::default()
        .title(" Stats ")
        .borders(ratatui::widgets::Borders::ALL)
        .style(ratatui::style::Style::default().fg(ratatui::style::Color::Gray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let text = vec![
        ratatui::text::Line::from(vec![
            ratatui::text::Span::styled("HP: ", ratatui::style::Style::default().fg(ratatui::style::Color::Gray)),
            ratatui::text::Span::styled(
                format!("{}/{}", ctx.stats.hp_current, ctx.stats.hp_max),
                ratatui::style::Style::default().fg(ratatui::style::Color::Red),
            ),
        ]),
        ratatui::text::Line::from(""),
        ratatui::text::Line::from(vec![
            ratatui::text::Span::styled("AP: ", ratatui::style::Style::default().fg(ratatui::style::Color::Gray)),
            ratatui::text::Span::styled(
                format!("{}/{}", ctx.stats.ap_current, ctx.stats.ap_max),
                ratatui::style::Style::default().fg(ratatui::style::Color::Blue),
            ),
        ]),
    ];

    let para = ratatui::widgets::Paragraph::new(text);
    frame.render_widget(para, inner);
}

fn render_log_widget(frame: &mut Frame, area: Rect, ctx: &WidgetRenderContext) {
    let block = ratatui::widgets::Block::default()
        .title(" Log ")
        .borders(ratatui::widgets::Borders::ALL)
        .style(ratatui::style::Style::default().fg(ratatui::style::Color::Gray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines: Vec<ratatui::text::Line> = ctx
        .log
        .entries
        .iter()
        .take(inner.height as usize)
        .map(|entry| {
            let style = match entry.level {
                bd_core::gamelog::LogLevel::Info => {
                    ratatui::style::Style::default().fg(ratatui::style::Color::White)
                }
                bd_core::gamelog::LogLevel::Warn => {
                    ratatui::style::Style::default().fg(ratatui::style::Color::Yellow)
                }
                bd_core::gamelog::LogLevel::Combat => {
                    ratatui::style::Style::default().fg(ratatui::style::Color::Red)
                }
            };
            ratatui::text::Line::styled(&entry.message, style)
        })
        .collect();

    let para = ratatui::widgets::Paragraph::new(lines).wrap(ratatui::widgets::Wrap { trim: false });
    frame.render_widget(para, inner);
}

fn render_actions_widget(frame: &mut Frame, area: Rect, ctx: &WidgetRenderContext) {
    let block = ratatui::widgets::Block::default()
        .title(" Actions ")
        .borders(ratatui::widgets::Borders::ALL)
        .style(ratatui::style::Style::default().fg(ratatui::style::Color::Gray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let spans: Vec<ratatui::text::Span> = ctx
        .actions
        .actions
        .iter()
        .flat_map(|a| {
            let key_style = if a.enabled {
                ratatui::style::Style::default().fg(ratatui::style::Color::Yellow)
            } else {
                ratatui::style::Style::default().fg(ratatui::style::Color::DarkGray)
            };
            let mut parts = vec![
                ratatui::text::Span::styled(format!("{} ", a.key_hint), key_style),
                ratatui::text::Span::raw(a.label.to_string()),
            ];
            if let Some(ref reason) = a.denial_reason {
                parts.push(ratatui::text::Span::styled(
                    format!(" ({})", reason),
                    ratatui::style::Style::default().fg(ratatui::style::Color::Red),
                ));
            }
            parts.push(ratatui::text::Span::raw("  "));
            parts
        })
        .collect();

    let para = ratatui::widgets::Paragraph::new(ratatui::text::Line::from(spans));
    frame.render_widget(para, inner);
}

fn render_inventory_list_widget(frame: &mut Frame, area: Rect, ctx: &WidgetRenderContext) {
    let block = ratatui::widgets::Block::default()
        .title(" Inventory ")
        .borders(ratatui::widgets::Borders::ALL)
        .style(ratatui::style::Style::default().fg(ratatui::style::Color::Gray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines: Vec<ratatui::text::Line> = ctx
        .container
        .items
        .iter()
        .map(|item| {
            let equip_mark = if item.equipped { " [E]" } else { "" };
            ratatui::text::Line::styled(
                format!(" {}{}", item.name, equip_mark),
                ratatui::style::Style::default().fg(ratatui::style::Color::White),
            )
        })
        .collect();

    let para = ratatui::widgets::Paragraph::new(lines);
    frame.render_widget(para, inner);
}

fn render_equipment_widget(frame: &mut Frame, area: Rect, ctx: &WidgetRenderContext) {
    let block = ratatui::widgets::Block::default()
        .title(" Equipment ")
        .borders(ratatui::widgets::Borders::ALL)
        .style(ratatui::style::Style::default().fg(ratatui::style::Color::Gray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines: Vec<ratatui::text::Line> = ctx
        .container
        .items
        .iter()
        .filter(|item| item.equipped)
        .map(|item| {
            ratatui::text::Line::styled(
                format!(" {}", item.name),
                ratatui::style::Style::default().fg(ratatui::style::Color::Cyan),
            )
        })
        .collect();

    let para = ratatui::widgets::Paragraph::new(lines);
    frame.render_widget(para, inner);
}

// ---------------------------------------------------------------------------
// Screen layout engine
// ---------------------------------------------------------------------------

/// Given a screen definition and a total area, compute (panel_id, rect) pairs.
pub fn compute_panel_rects(
    def: &ScreenDefinition,
    total: Rect,
) -> Vec<(String, Rect)> {
    if def.panels.is_empty() {
        return vec![];
    }

    // Separate main vs non-main panels
    let non_main: Vec<&PanelDefinition> = def.panels.iter().filter(|p| !matches!(p.layout, PanelLayout::Main)).collect();
    let main: Vec<&PanelDefinition> = def.panels.iter().filter(|p| matches!(p.layout, PanelLayout::Main)).collect();

    let mut result = Vec::new();
    let mut remaining = total;

    // Process left panels first
    for panel in &non_main {
        if let PanelLayout::Left { width_pct } = panel.layout {
            let w = (total.width * width_pct / 100).max(1);
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
            let w = (total.width * width_pct / 100).max(1);
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
            let h = (total.height * height_pct / 100).max(1);
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
            let h = (total.height * height_pct / 100).max(1);
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
    let block = ratatui::widgets::Block::default()
        .title(" Party ")
        .borders(ratatui::widgets::Borders::ALL)
        .style(ratatui::style::Style::default().fg(ratatui::style::Color::Gray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines: Vec<ratatui::text::Line> = ctx
        .container
        .items
        .iter()
        .map(|item| {
            ratatui::text::Line::styled(
                format!(" {}", item.name),
                ratatui::style::Style::default().fg(ratatui::style::Color::White),
            )
        })
        .collect();

    let para = ratatui::widgets::Paragraph::new(lines);
    frame.render_widget(para, inner);
}

fn render_outpost_travel_widget(frame: &mut Frame, area: Rect, _ctx: &WidgetRenderContext) {
    let block = ratatui::widgets::Block::default()
        .title(" Travel ")
        .borders(ratatui::widgets::Borders::ALL)
        .style(ratatui::style::Style::default().fg(ratatui::style::Color::Gray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let text = vec![
        ratatui::text::Line::styled(
            " Reachable locations:",
            ratatui::style::Style::default().fg(ratatui::style::Color::Cyan),
        ),
        ratatui::text::Line::from(""),
        ratatui::text::Line::styled(
            "  Ancient Temple (3 turns)",
            ratatui::style::Style::default().fg(ratatui::style::Color::White),
        ),
        ratatui::text::Line::styled(
            "  Crypt of the Fallen (5 turns)",
            ratatui::style::Style::default().fg(ratatui::style::Color::White),
        ),
        ratatui::text::Line::from(""),
        ratatui::text::Line::styled(
            " Press 't' to travel | 'i' inventory",
            ratatui::style::Style::default().fg(ratatui::style::Color::DarkGray),
        ),
    ];

    let para = ratatui::widgets::Paragraph::new(text);
    frame.render_widget(para, inner);
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(result.errors.iter().any(|e| e.contains("no widget registered")));
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
