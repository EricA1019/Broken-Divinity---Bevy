//! bd_tui — Terminal UI layer for the BD Kernel.
//!
//! Renders Ratatui widgets from view models. Never queries ECS gameplay
//! internals directly.

pub mod render_grid;
pub mod theme;
pub mod view_models;
pub mod visual;

use bevy_app::{App, Plugin};
use bevy_ecs::{
    entity::Entity,
    message::{MessageReader, MessageWriter},
    query::{With, Without},
    schedule::IntoScheduleConfigs,
    system::{Query, Res, ResMut},
};
use bevy_ratatui::{RatatuiContext, event::KeyMessage};
use ratatui::{
    layout::Alignment,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use bd_core::{
    BdSet,
    components::{BlocksMovement, Player, Position},
    direction::Direction,
    gamelog::LogLevel,
    signals::ActionIntent,
};

use render_grid::RenderCellGrid;
use theme::ThemeRegistry;
use view_models::{ActionListViewModel, LogViewModel, MapViewModel, StatsViewModel};
use visual::{SymbolRegistry, VisualToken};

/// TUI plugin — registers input mapping and render systems.
pub struct BdTuiPlugin;

impl Plugin for BdTuiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SymbolRegistry::phase5_defaults());
        app.insert_resource(ThemeRegistry::phase5_defaults());

        view_models::register_view_models(app);

        app.add_systems(
            bevy_app::Update,
            (
                map_input_to_intents.in_set(BdSet::Input),
                draw_ui.in_set(BdSet::Render),
            ),
        );

        tracing::info!("BdTuiPlugin initialized");
    }
}

/// Map keyboard input to ActionIntent messages.
#[allow(clippy::type_complexity)]
fn map_input_to_intents(
    mut messages: MessageReader<KeyMessage>,
    player: Query<Entity, With<Player>>,
    enemies: Query<(Entity, &Position), (With<BlocksMovement>, Without<Player>)>,
    player_pos: Query<&Position, With<Player>>,
    mut action_writer: MessageWriter<ActionIntent>,
    mut exit: MessageWriter<bevy_app::AppExit>,
) {
    use crossterm::event::KeyCode;

    let Ok(player_entity) = player.single() else {
        return;
    };

    for key in messages.read() {
        match key.code {
            // Movement
            KeyCode::Char('w') | KeyCode::Up => {
                action_writer.write(ActionIntent {
                    actor: player_entity,
                    action_id: "ability.move".into(),
                    direction: Some(Direction::North),
                    target: None,
                });
            }
            KeyCode::Char('s') | KeyCode::Down => {
                action_writer.write(ActionIntent {
                    actor: player_entity,
                    action_id: "ability.move".into(),
                    direction: Some(Direction::South),
                    target: None,
                });
            }
            KeyCode::Char('d') | KeyCode::Right => {
                action_writer.write(ActionIntent {
                    actor: player_entity,
                    action_id: "ability.move".into(),
                    direction: Some(Direction::East),
                    target: None,
                });
            }
            KeyCode::Char('a') | KeyCode::Left => {
                action_writer.write(ActionIntent {
                    actor: player_entity,
                    action_id: "ability.move".into(),
                    direction: Some(Direction::West),
                    target: None,
                });
            }
            // Wait
            KeyCode::Char('.') => {
                action_writer.write(ActionIntent {
                    actor: player_entity,
                    action_id: "ability.wait".into(),
                    direction: None,
                    target: None,
                });
            }
            // Attack — target nearest enemy
            KeyCode::Char('f') => {
                let nearest = find_nearest_enemy(player_pos.single().ok(), &enemies);
                action_writer.write(ActionIntent {
                    actor: player_entity,
                    action_id: "ability.attack".into(),
                    direction: None,
                    target: nearest,
                });
            }
            // Guard
            KeyCode::Char('g') => {
                action_writer.write(ActionIntent {
                    actor: player_entity,
                    action_id: "ability.guard".into(),
                    direction: None,
                    target: None,
                });
            }
            // Quit
            KeyCode::Char('q') | KeyCode::Esc => {
                exit.write_default();
            }
            _ => {}
        }
    }
}

/// Find the nearest enemy to the player by Manhattan distance.
#[allow(clippy::type_complexity)]
fn find_nearest_enemy(
    player_pos: Option<&Position>,
    enemies: &Query<(Entity, &Position), (With<BlocksMovement>, Without<Player>)>,
) -> Option<Entity> {
    let pp = player_pos?;
    enemies
        .iter()
        .min_by_key(|(_, pos)| (pos.x - pp.x).unsigned_abs() + (pos.y - pp.y).unsigned_abs())
        .map(|(e, _)| e)
}

/// Draw the full TUI layout from view models only.
#[allow(clippy::too_many_arguments)]
fn draw_ui(
    mut ctx: ResMut<RatatuiContext>,
    map_vm: Res<MapViewModel>,
    stats_vm: Res<StatsViewModel>,
    log_vm: Res<LogViewModel>,
    action_vm: Res<ActionListViewModel>,
    symbols: Res<SymbolRegistry>,
    theme: Res<ThemeRegistry>,
) {
    let _ = ctx.draw(|frame| {
        let area = frame.area();

        let [main_area, stats_area] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(20)]).areas(area);

        let [map_area, bottom_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(8)]).areas(main_area);

        let [log_area, action_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(3)]).areas(bottom_area);

        // ---- Map ----
        render_map(frame, map_area, &map_vm, &symbols, &theme);

        // ---- Stats ----
        render_stats(frame, stats_area, &stats_vm);

        // ---- Log ----
        render_log(frame, log_area, &log_vm);

        // ---- Action bar ----
        render_action_bar(frame, action_area, &action_vm);

        // ---- Footer ----
        render_footer(frame, area);
    });
}

fn render_map(
    frame: &mut ratatui::Frame,
    area: Rect,
    vm: &MapViewModel,
    symbols: &SymbolRegistry,
    theme: &ThemeRegistry,
) {
    let block = Block::default()
        .title(" Map ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Gray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let w = inner.width.min(vm.width as u16);
    let h = inner.height.min(vm.height as u16);
    let mut grid = RenderCellGrid::new(w, h, VisualToken::Floor, symbols, theme);

    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let idx = (y * vm.width + x) as usize;
            let token = match vm.tiles.get(idx) {
                Some(bd_core::components::Tile::Wall) | None => VisualToken::Wall,
                Some(bd_core::components::Tile::Floor) => VisualToken::Floor,
            };
            grid.set(x as u16, y as u16, token, symbols, theme);
        }
    }

    for ep in &vm.enemy_positions {
        if ep.x >= 0 && ep.x < w as i32 && ep.y >= 0 && ep.y < h as i32 {
            grid.set(ep.x as u16, ep.y as u16, VisualToken::Enemy, symbols, theme);
        }
    }

    if let Some(pp) = vm.player_pos {
        if pp.x >= 0 && pp.x < w as i32 && pp.y >= 0 && pp.y < h as i32 {
            grid.set(
                pp.x as u16,
                pp.y as u16,
                VisualToken::Player,
                symbols,
                theme,
            );
        }
    }

    let mut lines: Vec<Line> = Vec::new();
    for row in grid.rows() {
        let spans: Vec<Span> = row
            .into_iter()
            .map(|(_, _, glyph, style)| Span::styled(glyph.to_string(), style))
            .collect();
        lines.push(Line::from(spans));
    }

    let para = Paragraph::new(lines);
    frame.render_widget(para, inner);
}

fn render_stats(frame: &mut ratatui::Frame, area: Rect, vm: &StatsViewModel) {
    let block = Block::default()
        .title(" Stats ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Gray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let text = vec![
        Line::from(vec![
            Span::styled("HP: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}/{}", vm.hp_current, vm.hp_max),
                Style::default().fg(Color::Red),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("AP: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}/{}", vm.ap_current, vm.ap_max),
                Style::default().fg(Color::Blue),
            ),
        ]),
    ];

    let para = Paragraph::new(text);
    frame.render_widget(para, inner);
}

fn render_log(frame: &mut ratatui::Frame, area: Rect, vm: &LogViewModel) {
    let block = Block::default()
        .title(" Log ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Gray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines: Vec<Line> = vm
        .entries
        .iter()
        .take(inner.height as usize)
        .map(|entry| {
            let style = match entry.level {
                LogLevel::Info => Style::default().fg(Color::White),
                LogLevel::Warn => Style::default().fg(Color::Yellow),
                LogLevel::Combat => Style::default().fg(Color::Red),
            };
            Line::styled(&entry.message, style)
        })
        .collect();

    let para = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(para, inner);
}

fn render_action_bar(frame: &mut ratatui::Frame, area: Rect, vm: &ActionListViewModel) {
    let block = Block::default()
        .title(" Actions ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Gray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let spans: Vec<Span> = vm
        .actions
        .iter()
        .flat_map(|a| {
            let key_style = if a.enabled {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let mut parts = vec![
                Span::styled(format!("{} ", a.key_hint), key_style),
                Span::raw(a.label.to_string()),
            ];
            if let Some(ref reason) = a.denial_reason {
                parts.push(Span::styled(
                    format!(" ({})", reason),
                    Style::default().fg(Color::Red),
                ));
            }
            parts.push(Span::raw("  "));
            parts
        })
        .collect();

    let para = Paragraph::new(Line::from(spans));
    frame.render_widget(para, inner);
}

fn render_footer(frame: &mut ratatui::Frame, area: Rect) {
    let version = env!("CARGO_PKG_VERSION");
    let text = format!("Broken Divinity Kernel v{version} | phase 6 | q quit");
    let footer_area = Rect {
        y: area.height.saturating_sub(1),
        height: 1,
        ..area
    };
    let para = Paragraph::new(text)
        .alignment(Alignment::Right)
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(para, footer_area);
}
