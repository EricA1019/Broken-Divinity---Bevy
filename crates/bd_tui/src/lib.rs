//! bd_tui — Terminal UI layer for the BD Kernel.
//!
//! Renders Ratatui widgets from view models. Never queries ECS gameplay
//! internals directly.

use bevy_app::{App, Plugin};
use bevy_ecs::{
    message::{MessageReader, MessageWriter},
    query::With,
    schedule::IntoScheduleConfigs,
    system::{Query, Res, ResMut},
};
use bevy_ratatui::{RatatuiContext, event::KeyMessage};
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use bd_core::{
    BdSet,
    components::{Player, Position},
    direction::Direction,
    gamelog::{GameLog, LogLevel},
    map::SmokeMap,
    pools::Pools,
    signals::{MoveIntent, PoolKind},
};

/// TUI plugin — registers input mapping and render systems.
pub struct BdTuiPlugin;

impl Plugin for BdTuiPlugin {
    fn build(&self, app: &mut App) {
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

/// Map keyboard input to MoveIntent messages.
fn map_input_to_intents(
    mut messages: MessageReader<KeyMessage>,
    player: Query<bevy_ecs::entity::Entity, With<Player>>,
    mut move_writer: MessageWriter<MoveIntent>,
    mut exit: MessageWriter<bevy_app::AppExit>,
) {
    use crossterm::event::KeyCode;

    let Ok(player_entity) = player.single() else {
        return;
    };

    for key in messages.read() {
        let direction = match key.code {
            KeyCode::Char('w') | KeyCode::Up => Some(Direction::North),
            KeyCode::Char('s') | KeyCode::Down => Some(Direction::South),
            KeyCode::Char('d') | KeyCode::Right => Some(Direction::East),
            KeyCode::Char('a') | KeyCode::Left => Some(Direction::West),
            KeyCode::Char('q') | KeyCode::Esc => {
                exit.write_default();
                None
            }
            _ => None,
        };

        if let Some(dir) = direction {
            move_writer.write(MoveIntent {
                entity: player_entity,
                direction: dir,
            });
        }
    }
}

/// Draw the full TUI layout: map, stats panel, help, log, footer.
fn draw_ui(
    mut ctx: ResMut<RatatuiContext>,
    map: Res<SmokeMap>,
    player_pos: Query<&Position, With<Player>>,
    player_pools: Query<&Pools, With<Player>>,
    game_log: Res<GameLog>,
) {
    let _ = ctx.draw(|frame| {
        let area = frame.area();

        // Split into main area (left) and stats panel (right)
        let [main_area, stats_area] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(20)]).areas(area);

        // Main area: split into map and bottom panel
        let [map_area, bottom_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(7)]).areas(main_area);

        // Bottom: split into log and help
        let [log_area, help_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(bottom_area);

        // ---- Map ----
        let player_pos = player_pos.single().ok().copied();
        let pools = player_pools.single().ok();
        render_map(frame, map_area, &map, player_pos);

        // ---- Stats panel ----
        render_stats(frame, stats_area, pools);

        // ---- Log ----
        render_log(frame, log_area, &game_log);

        // ---- Help line ----
        render_help(frame, help_area);

        // ---- Footer ----
        render_footer(frame, area);
    });
}

fn render_map(
    frame: &mut ratatui::Frame,
    area: Rect,
    map: &SmokeMap,
    player_pos: Option<Position>,
) {
    let block = Block::default()
        .title(" Map ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Gray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Build the map text
    let mut lines: Vec<Line> = Vec::new();
    for y in 0..map.height.min(inner.height as i32) {
        let mut spans: Vec<Span> = Vec::new();
        for x in 0..map.width.min(inner.width as i32) {
            let is_player = player_pos == Some(Position { x, y });
            let tile = map.get(x, y).unwrap_or(bd_core::components::Tile::Wall);

            let (ch, style) = if is_player {
                ('@', Style::default().fg(Color::Yellow))
            } else {
                (
                    tile.glyph(),
                    match tile {
                        bd_core::components::Tile::Wall => Style::default().fg(Color::DarkGray),
                        bd_core::components::Tile::Floor => Style::default().fg(Color::Gray),
                    },
                )
            };

            spans.push(Span::styled(ch.to_string(), style));
        }
        lines.push(Line::from(spans));
    }

    let para = Paragraph::new(lines);
    frame.render_widget(para, inner);
}

fn render_stats(frame: &mut ratatui::Frame, area: Rect, pools: Option<&Pools>) {
    let block = Block::default()
        .title(" Stats ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Gray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let hp = pools
        .and_then(|p| p.get(PoolKind::Health))
        .map_or((0, 0), |p| (p.current, p.max));
    let ap = pools
        .and_then(|p| p.get(PoolKind::ActionPoints))
        .map_or((0, 0), |p| (p.current, p.max));

    let text = vec![
        Line::from(vec![
            Span::styled("HP: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}/{}", hp.0, hp.1),
                Style::default().fg(Color::Red),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("AP: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}/{}", ap.0, ap.1),
                Style::default().fg(Color::Blue),
            ),
        ]),
    ];

    let para = Paragraph::new(text);
    frame.render_widget(para, inner);
}

fn render_log(frame: &mut ratatui::Frame, area: Rect, log: &GameLog) {
    let block = Block::default()
        .title(" Log ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Gray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines: Vec<Line> = log
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

fn render_help(frame: &mut ratatui::Frame, area: Rect) {
    let text = Line::from(vec![
        Span::styled("WASD/Arrows", Style::default().fg(Color::Yellow)),
        Span::raw(" move | "),
        Span::styled(".", Style::default().fg(Color::Yellow)),
        Span::raw(" wait | "),
        Span::styled("q", Style::default().fg(Color::Yellow)),
        Span::raw(" quit"),
    ]);
    let para = Paragraph::new(text).style(Style::default().fg(Color::Gray));
    frame.render_widget(para, area);
}

fn render_footer(frame: &mut ratatui::Frame, area: Rect) {
    let version = env!("CARGO_PKG_VERSION");
    let text = format!("Broken Divinity Kernel v{version} | phase 1 | q quit");
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
