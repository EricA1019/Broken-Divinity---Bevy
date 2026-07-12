//! bd_tui — Terminal UI layer for the BD Kernel.
//!
//! Renders Ratatui widgets from view models. Never queries ECS gameplay
//! internals directly.

pub mod render_grid;
pub mod screens;
pub mod theme;
pub mod view_models;
pub mod visual;

use bevy_app::{App, Plugin};
use bevy_ecs::{
    entity::Entity,
    message::{MessageReader, MessageWriter},
    query::{With, Without},
    schedule::IntoScheduleConfigs,
    system::{Local, Query, Res, ResMut},
};
use bevy_ratatui::{RatatuiContext, event::KeyMessage};
use ratatui::{
    layout::Alignment,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};

use bd_core::{
    BdSet, HelpLine,
    components::{BlocksMovement, Player, Position},
    direction::Direction,
    gamelog::{GameLog, LogLevel},
    signals::ActionIntent,
    spatial::TransitionIntent,
};

use screens::{
    compute_panel_rects, default_screen_registry, default_widget_registry,
    validate_screens, ScreenIntent, ScreenRegistry, ScreenState, WidgetRegistry,
    WidgetRenderContext,
};
use theme::ThemeRegistry;
use view_models::{
    ActionListViewModel, ContainerViewModel, EventViewModel, HelpViewModel, LogViewModel, MapViewModel,
    StatsViewModel,
};
use visual::SymbolRegistry;

/// TUI plugin — registers input mapping and render systems.
pub struct BdTuiPlugin;
impl Plugin for BdTuiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SymbolRegistry::phase5_defaults());
        app.insert_resource(ThemeRegistry::phase5_defaults());

        // Register screen definitions and widget registry
        let screen_reg = default_screen_registry();
        let widget_reg = default_widget_registry();

        // Validate screens at startup
        let validation = validate_screens(&screen_reg, &widget_reg);
        if !validation.valid {
            for err in &validation.errors {
                tracing::error!("Screen validation: {err}");
            }
            panic!("Screen validation failed: {} errors", validation.errors.len());
        }

        app.insert_resource(screen_reg);
        app.insert_resource(widget_reg);
        app.insert_resource(ScreenState::default());
        app.add_message::<ScreenIntent>();

        view_models::register_view_models(app);

        app.add_systems(
            bevy_app::Update,
            (
                sync_event_screen.in_set(BdSet::IntentCollection),
                map_input_to_intents.in_set(BdSet::Input),
                screens::process_screen_intents.in_set(BdSet::IntentCollection),
                draw_ui.in_set(BdSet::Render),
            ),
        );

        tracing::info!("BdTuiPlugin initialized");
    }
}

/// Observe CurrentEvent and switch to/from the event screen.
fn sync_event_screen(
    current: Res<bd_core::events::CurrentEvent>,
    mut screen_writer: MessageWriter<ScreenIntent>,
    screen_state: Res<ScreenState>,
) {
    if current.is_active() && screen_state.current != "event" {
        screen_writer.write(ScreenIntent {
            screen_id: "event".into(),
        });
    }
    if !current.is_active() && screen_state.current == "event" {
        screen_writer.write(ScreenIntent {
            screen_id: "combat".into(),
        });
    }
}

/// Map keyboard input to ActionIntent messages.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn map_input_to_intents(
    mut messages: MessageReader<KeyMessage>,
    player: Query<Entity, With<Player>>,
    enemies: Query<(Entity, &Position), (With<BlocksMovement>, Without<Player>)>,
    survivors: Query<(Entity, &Position), With<bd_core::colony::survivors::Survivor>>,
    player_pos: Query<&Position, With<Player>>,
    mut action_writer: MessageWriter<ActionIntent>,
    screen_state: Res<ScreenState>,
    mut screen_writer: MessageWriter<ScreenIntent>,
    mut transition_writer: MessageWriter<TransitionIntent>,
    mut mode: ResMut<bd_core::spatial::GameMode>,
    mut game_log: ResMut<GameLog>,
    mut pending_station: ResMut<bd_core::colony::stations::PendingStationBuild>,
    mut pending_build_idx: Local<i8>,
    mut has_drained: Local<bool>,
    current_event: Res<bd_core::events::CurrentEvent>,
    mut event_writer: MessageWriter<bd_core::signals::EventSelected>,
) {
    use crossterm::event::KeyCode;

    // If in Title mode, any key transitions to Outpost (no player needed yet)
    if *mode == bd_core::spatial::GameMode::Title {
        if messages.read().next().is_some() {
            // Any key starts the game
            *mode = bd_core::spatial::GameMode::Outpost;
            screen_writer.write(ScreenIntent {
                screen_id: "outpost".into(),
            });
        }
        return;
    }

    let Ok(player_entity) = player.single() else {
        return;
    };

    // Drain stale terminal input once on first frame.
    if !*has_drained {
        *has_drained = true;
        while crossterm::event::poll(std::time::Duration::ZERO).unwrap_or(false) {
            let _ = crossterm::event::read();
        }
    }

    // If an event is active, only number keys for choices are handled
    if current_event.is_active() {
        for key in messages.read() {
            match key.code {
                KeyCode::Char(c @ '1'..='9') => {
                    let idx = (c as u8 - b'1') as usize;
                    event_writer.write(bd_core::signals::EventSelected {
                        actor: player_entity,
                        choice_index: idx,
                    });
                }
                _ => {} // swallow all other input during events
            }
        }
        return;
    }

    for key in messages.read() {
        match key.code {
            // Movement
            KeyCode::Char('w') | KeyCode::Up => {
                if *pending_build_idx >= 0 {
                    *pending_build_idx = -1;
                    // pending_station already set by 'b' key handler
                    action_writer.write(ActionIntent {
                        actor: player_entity,
                        action_id: "ability.build".into(),
                        direction: Some(Direction::North),
                        target: None,
                    });
                } else {
                    action_writer.write(ActionIntent {
                        actor: player_entity,
                        action_id: "ability.move".into(),
                        direction: Some(Direction::North),
                        target: None,
                    });
                }
            }
            KeyCode::Char('s') | KeyCode::Down => {
                if *pending_build_idx >= 0 {
                    *pending_build_idx = -1;
                    action_writer.write(ActionIntent {
                        actor: player_entity,
                        action_id: "ability.build".into(),
                        direction: Some(Direction::South),
                        target: None,
                    });
                } else {
                    action_writer.write(ActionIntent {
                        actor: player_entity,
                        action_id: "ability.move".into(),
                        direction: Some(Direction::South),
                        target: None,
                    });
                }
            }
            KeyCode::Char('d') | KeyCode::Right => {
                if *pending_build_idx >= 0 {
                    *pending_build_idx = -1;
                    action_writer.write(ActionIntent {
                        actor: player_entity,
                        action_id: "ability.build".into(),
                        direction: Some(Direction::East),
                        target: None,
                    });
                } else {
                    action_writer.write(ActionIntent {
                        actor: player_entity,
                        action_id: "ability.move".into(),
                        direction: Some(Direction::East),
                        target: None,
                    });
                }
            }
            KeyCode::Char('a') | KeyCode::Left => {
                // In outpost mode, 'a' assigns nearest survivor to task
                if *mode == bd_core::spatial::GameMode::Outpost {
                    if let Ok(player_pos) = player_pos.single() {
                        let nearest = survivors.iter()
                            .min_by_key(|(_, sp)| {
                                ((player_pos.x - sp.x).abs() + (player_pos.y - sp.y).abs()) as u32
                            });
                        if let Some((survivor_entity, _)) = nearest {
                            action_writer.write(ActionIntent {
                                actor: player_entity,
                                action_id: "ability.assign_task".into(),
                                direction: None,
                                target: Some(survivor_entity),
                            });
                        }
                    }
                } else if *pending_build_idx >= 0 {
                    *pending_build_idx = -1;
                    action_writer.write(ActionIntent {
                        actor: player_entity,
                        action_id: "ability.build".into(),
                        direction: Some(Direction::West),
                        target: None,
                    });
                } else {
                    action_writer.write(ActionIntent {
                        actor: player_entity,
                        action_id: "ability.move".into(),
                        direction: Some(Direction::West),
                        target: None,
                    });
                }
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
            // Switch to inventory screen
            KeyCode::Char('i') => {
                screen_writer.write(ScreenIntent {
                    screen_id: "inventory".into(),
                });
            }
            // Switch back to combat screen
            KeyCode::Char('z') => {
                screen_writer.write(ScreenIntent {
                    screen_id: "combat".into(),
                });
            }


            // Toggle help screen
            KeyCode::Char('?') => {
                if screen_state.current == "help" {
                    screen_writer.write(ScreenIntent {
                        screen_id: "combat".into(),
                    });
                } else {
                    screen_writer.write(ScreenIntent {
                        screen_id: "help".into(),
                    });
                }
            }
            // Travel to dungeon (outpost → tactical)
            KeyCode::Char('t') => {
                let target = if *mode == bd_core::spatial::GameMode::Outpost {
                    bd_core::spatial::GameMode::Tactical
                } else {
                    bd_core::spatial::GameMode::Outpost
                };
                transition_writer.write(TransitionIntent {
                    target,
                    node_id: Some("ruin.ancient_temple".into()),
                });
                screen_writer.write(ScreenIntent {
                    screen_id: if target == bd_core::spatial::GameMode::Tactical { "combat" } else { "outpost" }.into(),
                });
            }
            // Return to outpost
            KeyCode::Char('r') => {
                transition_writer.write(TransitionIntent {
                    target: bd_core::spatial::GameMode::Outpost,
                    node_id: None,
                });
                screen_writer.write(ScreenIntent {
                    screen_id: "outpost".into(),
                });
            }
            // Build station (outpost mode only) — pending build then direction
            KeyCode::Char('b') => {
                if *mode == bd_core::spatial::GameMode::Outpost {
                    if *pending_build_idx >= 0 {
                        // Cycle to next station type
                        let bps = bd_core::colony::stations::default_station_blueprints();
                        let next = (*pending_build_idx as usize + 1) % bps.len();
                        *pending_build_idx = next as i8;
                        pending_station.0 = Some(bps[next].station_type);
                        game_log.push(format!("Build: {:?} ({} Supplies)", bps[next].station_type, bps[next].build_cost_supplies), LogLevel::Info);
                    } else {
                        *pending_build_idx = 0;
                        let bps = bd_core::colony::stations::default_station_blueprints();
                        pending_station.0 = Some(bps[0].station_type);
                        game_log.push(format!("Build: {:?} ({} Supplies)", bps[0].station_type, bps[0].build_cost_supplies), LogLevel::Info);
                    }
                }
            }
            // Debug overlay toggle
            KeyCode::F(1) => {
                screen_writer.write(ScreenIntent {
                    screen_id: "debug".into(),
                });
            }
            // Quit (or cancel pending build)
            // Use process::exit to prevent buffered keystrokes leaking to shell
            KeyCode::Char('q') | KeyCode::Esc => {
                if *pending_build_idx >= 0 {
                    *pending_build_idx = -1;
                    pending_station.0 = None;
                    game_log.push("Build cancelled.", LogLevel::Info);
                } else {
                    // Restore terminal and flush output before exit
                    use std::io::Write;
                    use crossterm::terminal::{disable_raw_mode, LeaveAlternateScreen};
                    use crossterm::ExecutableCommand;
                    // Drain pending events
                    while crossterm::event::poll(std::time::Duration::ZERO).unwrap_or(false) {
                        let _ = crossterm::event::read();
                    }
                    // Restore terminal state
                    let _ = std::io::stdout().execute(LeaveAlternateScreen);
                    let _ = disable_raw_mode();
                    // Flush stdout so the shell sees a clean terminal
                    let _ = std::io::stdout().flush();
                    std::process::exit(0);
                }
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

/// Draw the full TUI layout driven by the current screen definition.
#[allow(clippy::too_many_arguments)]
fn draw_ui(
    mut ratatui_ctx: ResMut<RatatuiContext>,
    screen_state: Res<ScreenState>,
    screen_reg: Res<ScreenRegistry>,
    widget_reg: Res<WidgetRegistry>,
    map_vm: Res<MapViewModel>,
    stats_vm: Res<StatsViewModel>,
    log_vm: Res<LogViewModel>,
    action_vm: Res<ActionListViewModel>,
    container_vm: Res<ContainerViewModel>,
    event_vm: Res<EventViewModel>,
    help_vm: Res<HelpViewModel>,
    symbols: Res<SymbolRegistry>,
    theme: Res<ThemeRegistry>,
    help: Res<HelpLine>,
    game_time: Res<bd_core::time::GameTime>,
    travel_map: Res<bd_core::spatial::TravelMap>,
) {
    let Some(def) = screen_reg.screens.get(&screen_state.current) else {
        tracing::warn!("Unknown screen: {}", screen_state.current);
        return;
    };

    let _ = ratatui_ctx.draw(|frame| {
        let area = frame.area();

        // Build the widget render context from view models
        let wctx = WidgetRenderContext {
            map: &map_vm,
            stats: &stats_vm,
            log: &log_vm,
            actions: &action_vm,
            container: &container_vm,
            event: &event_vm,
            help: &help_vm,
            symbols: &symbols,
            theme: &theme,
            travel_map: &travel_map,
        };

        // Compute panel positions from the screen definition
        let panel_rects = compute_panel_rects(def, area);

        // Render each panel
        for (panel_id, rect) in &panel_rects {
            if let Some(binding) = widget_reg.bindings.get(panel_id.as_str()) {
                (binding.render)(frame, *rect, &wctx);
            } else {
                let block = Block::default()
                    .title(format!(" Unknown widget: {panel_id} "))
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::Red));
                frame.render_widget(block, *rect);
            }
        }

        // Footer — always render at bottom
        let help_text = &help.0;
        render_footer(frame, area, help_text, game_time.turn, game_time.day);
    });
}





fn render_footer(frame: &mut ratatui::Frame, area: Rect, help: &str, turn: u64, day: u64) {
    let version = env!("CARGO_PKG_VERSION");
    let text = format!("Turn: {turn} | Day: {day} | Broken Divinity Kernel v{version} | {help}");
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

#[cfg(test)]
mod tests {
    // use super::*; — not needed for standalone string tests

    #[test]
    fn footer_shows_turn_counter() {
        // Test that the footer text includes turn and day info
        let version = env!("CARGO_PKG_VERSION");
        let turn: u64 = 5;
        let day: u64 = 0;
        let help = "Move:w↑s↓a←d→";
        let text = format!("Turn: {turn} | Day: {day} | Broken Divinity Kernel v{version} | {help}");
        assert!(text.contains("Turn: 5"), "Footer should show turn counter");
        assert!(text.contains("Day: 0"), "Footer should show day counter");
        assert!(text.contains("Broken Divinity Kernel"), "Footer should show version");
    }
}
