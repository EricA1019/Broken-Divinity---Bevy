//! bd_tui — Terminal UI layer for the BD Kernel.
//!
//! Renders Ratatui widgets from view models. Never queries ECS gameplay
//! internals directly.

pub mod commands;
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
    system::{Query, Res, ResMut, SystemParam},
};
use bevy_ratatui::{RatatuiContext, event::KeyMessage};
use ratatui::{
    layout::Alignment,
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
};

use bd_core::{
    BdSet,
    components::{BlocksMovement, Player, Position},
    direction::Direction,
    gamelog::{GameLog, LogLevel},
    signals::ActionIntent,
    spatial::TransitionIntent,
};

use screens::{
    ScreenIntent, ScreenRegistry, ScreenState, WidgetRegistry, WidgetRenderContext,
    compute_panel_rects, default_screen_registry, default_widget_registry, validate_screens,
};
use theme::ThemeRegistry;
use view_models::{
    ActionListViewModel, ContainerViewModel, EventViewModel, HelpViewModel, LogViewModel,
    MapViewModel, StatsViewModel,
};
use visual::SymbolRegistry;

/// TUI plugin — registers input mapping and render systems.
pub struct BdTuiPlugin;
impl Plugin for BdTuiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<commands::CommandBindings>();
        app.init_resource::<commands::ApplicationExitRequest>();
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
            panic!(
                "Screen validation failed: {} errors",
                validation.errors.len()
            );
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
    current: Option<Res<bd_core::events::CurrentEvent>>,
    mut screen_writer: MessageWriter<ScreenIntent>,
    screen_state: Res<ScreenState>,
) {
    let Some(current) = current else {
        return;
    };
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

#[derive(SystemParam)]
struct InputQueries<'w, 's> {
    player: Query<
        'w,
        's,
        (
            Entity,
            &'static Position,
            Option<&'static bd_core::spatial::EntityScope>,
        ),
        With<Player>,
    >,
    enemies: Query<
        'w,
        's,
        (
            Entity,
            &'static Position,
            Option<&'static bd_core::spatial::EntityScope>,
        ),
        (With<BlocksMovement>, Without<Player>),
    >,
    items: Query<
        'w,
        's,
        (
            Entity,
            Option<&'static Position>,
            Option<&'static bd_core::inventory::Usable>,
            Option<&'static bd_core::relationships::ContainedIn>,
            Option<&'static bd_core::spatial::EntityScope>,
        ),
        With<bd_core::inventory::Item>,
    >,
    survivors: Query<
        'w,
        's,
        (
            Entity,
            &'static Position,
            &'static bd_core::colony::survivors::SurvivorTask,
            Option<&'static bd_core::spatial::EntityScope>,
        ),
        With<bd_core::colony::survivors::Survivor>,
    >,
    stations: Query<
        'w,
        's,
        (
            Entity,
            &'static Position,
            Option<&'static bd_core::spatial::EntityScope>,
        ),
        With<bd_core::colony::stations::Station>,
    >,
}

#[derive(SystemParam)]
struct PersistenceRequests<'w> {
    save: ResMut<'w, bd_core::save::SaveRequest>,
    load: ResMut<'w, bd_core::save::LoadRequest>,
}

#[derive(SystemParam)]
struct UiRuntimeState<'w> {
    game_time: Res<'w, bd_core::time::GameTime>,
    bindings: Res<'w, commands::CommandBindings>,
    mode: Res<'w, bd_core::spatial::GameMode>,
    build_ghost: Res<'w, bd_core::colony::stations::BuildGhostState>,
    build_menu: Res<'w, bd_core::colony::stations::BuildMenuState>,
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn map_input_to_intents(
    mut messages: MessageReader<KeyMessage>,
    input: InputQueries,
    mut action_writer: MessageWriter<ActionIntent>,
    screen_state: Res<ScreenState>,
    mut screen_writer: MessageWriter<ScreenIntent>,
    mut transition_writer: MessageWriter<TransitionIntent>,
    mode: Res<bd_core::spatial::GameMode>,
    mut game_log: ResMut<GameLog>,
    mut pending_station: ResMut<bd_core::colony::stations::PendingStationBuild>,
    (mut build_ghost, mut build_menu): (
        ResMut<bd_core::colony::stations::BuildGhostState>,
        ResMut<bd_core::colony::stations::BuildMenuState>,
    ),
    mut persistence: PersistenceRequests,
    current_event: Option<Res<bd_core::events::CurrentEvent>>,
    mut event_writer: MessageWriter<bd_core::signals::EventSelected>,
    bindings: Res<commands::CommandBindings>,
    mut exit_request: ResMut<commands::ApplicationExitRequest>,
) {
    use crossterm::event::KeyCode;

    // If in GameOver mode, offer a clean restart while preserving explicit quit.
    if *mode == bd_core::spatial::GameMode::GameOver {
        if screen_state.current != "game_over" {
            screen_writer.write(ScreenIntent {
                screen_id: "game_over".into(),
            });
        } else if let Some(key) = messages.read().next() {
            let command =
                bindings.command_for_key_in(&key.code, *mode, commands::InteractionMode::GameOver);
            if command == Some(commands::UiCommand::Restart) {
                transition_writer.write(TransitionIntent {
                    target: bd_core::spatial::GameMode::Title,
                    node_id: None,
                });
                screen_writer.write(ScreenIntent {
                    screen_id: "title".into(),
                });
                game_log.push("Restarting the run.", LogLevel::Info);
            } else if command == Some(commands::UiCommand::Quit) {
                exit_request.0 = true;
            }
        }
        return;
    }

    // If in Title mode, any key transitions to Outpost (no player needed yet)
    if *mode == bd_core::spatial::GameMode::Title {
        for key in messages.read() {
            // Any key starts the game
            transition_writer.write(TransitionIntent {
                target: bd_core::spatial::GameMode::Outpost,
                node_id: None,
            });
            screen_writer.write(ScreenIntent {
                screen_id: "outpost".into(),
            });
            // Remember if user pressed 'b' — auto-enter build mode once player spawns
            if matches!(key.code, KeyCode::Char('b')) {
                build_ghost.active = true;
                // cursor will be set once player position is available
            }
        }
        return;
    }

    let Some((player_entity, player_pos, _)) = input
        .player
        .iter()
        .find(|(_, _, scope)| scope_is_active(*scope, *mode))
    else {
        // Player not spawned yet (spawn_outpost_player runs after Input set).
        // If build mode was queued from title, position cursor now.
        if build_ghost.active {
            // Can't position cursor without player — will be positioned on next frame
            // when the player exists and this guard passes.
        }
        return;
    };

    // If build mode was queued from title screen, position the ghost cursor now
    // that we have the player position.
    if build_ghost.active && build_ghost.cursor.x == 0 && build_ghost.cursor.y == 0 {
        build_ghost.cursor = *player_pos;
        game_log.push("BUILD MODE: 1=Stove 2=Altar 3=Workshop 4=Bed 5=Storage | arrows=move | Enter=place | b=cancel".to_string(), LogLevel::Info);
    }

    // If an event is active, only number keys for choices are handled
    if current_event
        .as_ref()
        .is_some_and(|event| event.is_active())
    {
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
        let interaction = if build_menu.active || build_ghost.active {
            commands::InteractionMode::Build
        } else {
            commands::InteractionMode::Normal
        };
        let command = bindings.command_for_key_in(&key.code, *mode, interaction);
        match (command, key.code) {
            // Movement — ghost cursor in build mode, normal movement otherwise
            (Some(commands::UiCommand::MoveNorth), _) => {
                if build_menu.active {
                    build_menu.selected = build_menu.selected.saturating_sub(1);
                    return;
                }
                if build_ghost.active {
                    build_ghost.cursor = *player_pos;
                    build_ghost.cursor.y -= 1;
                    return;
                }
                action_writer.write(ActionIntent {
                    actor: player_entity,
                    action_id: commands::command_action_id(commands::UiCommand::MoveNorth)
                        .unwrap()
                        .into(),
                    direction: Some(Direction::North),
                    target: None,
                });
            }
            (Some(commands::UiCommand::MoveSouth), _) => {
                if build_menu.active {
                    let option_count =
                        bd_core::colony::stations::default_station_blueprints().len();
                    if build_menu.selected + 1 < option_count {
                        build_menu.selected += 1;
                    }
                    return;
                }
                if build_ghost.active {
                    build_ghost.cursor = *player_pos;
                    build_ghost.cursor.y += 1;
                    return;
                }
                action_writer.write(ActionIntent {
                    actor: player_entity,
                    action_id: commands::command_action_id(commands::UiCommand::MoveSouth)
                        .unwrap()
                        .into(),
                    direction: Some(Direction::South),
                    target: None,
                });
            }
            (Some(commands::UiCommand::MoveEast), _) => {
                if build_menu.active {
                    return;
                }
                if build_ghost.active {
                    build_ghost.cursor = *player_pos;
                    build_ghost.cursor.x += 1;
                    return;
                }
                action_writer.write(ActionIntent {
                    actor: player_entity,
                    action_id: commands::command_action_id(commands::UiCommand::MoveEast)
                        .unwrap()
                        .into(),
                    direction: Some(Direction::East),
                    target: None,
                });
            }
            (Some(commands::UiCommand::AssignTask), _) => {
                if *mode == bd_core::spatial::GameMode::Outpost {
                    let nearest = input
                        .survivors
                        .iter()
                        .filter(|(_, _, _, scope)| scope_is_active(*scope, *mode))
                        .min_by_key(|(_, sp, _, _)| {
                            ((player_pos.x - sp.x).abs() + (player_pos.y - sp.y).abs()) as u32
                        });
                    if let Some((survivor_entity, _, task, _)) = nearest {
                        let next_action = match task {
                            bd_core::colony::survivors::SurvivorTask::Idle => {
                                "ability.assign_gathering"
                            }
                            bd_core::colony::survivors::SurvivorTask::Gathering => {
                                "ability.assign_defending"
                            }
                            bd_core::colony::survivors::SurvivorTask::Defending => {
                                "ability.assign_resting"
                            }
                            bd_core::colony::survivors::SurvivorTask::Resting => {
                                "ability.assign_idle"
                            }
                            bd_core::colony::survivors::SurvivorTask::AssignedTo(_) => {
                                "ability.assign_idle"
                            }
                        };
                        action_writer.write(ActionIntent {
                            actor: player_entity,
                            action_id: next_action.into(),
                            direction: None,
                            target: Some(survivor_entity),
                        });
                        return;
                    }
                    game_log.push("No survivor is available to assign.", LogLevel::Warn);
                }
            }
            (Some(commands::UiCommand::MoveWest), _) => {
                if build_menu.active {
                    return;
                }
                if build_ghost.active {
                    build_ghost.cursor = *player_pos;
                    build_ghost.cursor.x -= 1;
                    return;
                }
                action_writer.write(ActionIntent {
                    actor: player_entity,
                    action_id: commands::command_action_id(commands::UiCommand::MoveWest)
                        .unwrap()
                        .into(),
                    direction: Some(Direction::West),
                    target: None,
                });
            }
            // Wait
            (Some(commands::UiCommand::Wait), _) => {
                action_writer.write(ActionIntent {
                    actor: player_entity,
                    action_id: commands::command_action_id(commands::UiCommand::Wait)
                        .unwrap()
                        .into(),
                    direction: None,
                    target: None,
                });
            }
            // Attack — target nearest enemy (no-op if none in range)
            (Some(commands::UiCommand::Attack), _) => {
                if let Some(nearest) = find_nearest_enemy(Some(player_pos), &input.enemies, *mode) {
                    action_writer.write(ActionIntent {
                        actor: player_entity,
                        action_id: commands::command_action_id(commands::UiCommand::Attack)
                            .unwrap()
                            .into(),
                        direction: None,
                        target: Some(nearest),
                    });
                } else {
                    game_log.push("No target in range.", LogLevel::Warn);
                }
            }
            // Guard
            (Some(commands::UiCommand::Guard), _) => {
                action_writer.write(ActionIntent {
                    actor: player_entity,
                    action_id: commands::command_action_id(commands::UiCommand::Guard)
                        .unwrap()
                        .into(),
                    direction: None,
                    target: None,
                });
            }
            // Switch to inventory screen
            (Some(commands::UiCommand::Inventory), _) => {
                screen_writer.write(ScreenIntent {
                    screen_id: "inventory".into(),
                });
            }
            // Pick up the item at the player's current position.
            (Some(commands::UiCommand::Pickup), _) => {
                if let Some((item, _, _, _, _)) = input
                    .items
                    .iter()
                    .filter(|(_, _, _, _, scope)| scope_is_active(*scope, *mode))
                    .find(|(_, pos, _, _, _)| pos.is_some_and(|pos| *pos == *player_pos))
                {
                    action_writer.write(ActionIntent {
                        actor: player_entity,
                        action_id: commands::command_action_id(commands::UiCommand::Pickup)
                            .unwrap()
                            .into(),
                        direction: None,
                        target: Some(item),
                    });
                } else {
                    game_log.push("There is nothing to pick up here.", LogLevel::Warn);
                }
            }
            // Use the first carried usable item through the action pipeline.
            (Some(commands::UiCommand::UseItem), _) => {
                if let Some((item, _, Some(_), Some(_), _)) = input
                    .items
                    .iter()
                    .filter(|(_, _, _, _, scope)| scope_is_active(*scope, *mode))
                    .find(|(_, _, usable, contained, _)| usable.is_some() && contained.is_some())
                {
                    action_writer.write(ActionIntent {
                        actor: player_entity,
                        action_id: commands::command_action_id(commands::UiCommand::UseItem)
                            .unwrap()
                            .into(),
                        direction: None,
                        target: Some(item),
                    });
                } else {
                    game_log.push("You have no usable item.", LogLevel::Warn);
                }
            }
            // Switch back to combat screen
            (Some(commands::UiCommand::CombatScreen), _) => {
                screen_writer.write(ScreenIntent {
                    screen_id: if *mode == bd_core::spatial::GameMode::Outpost {
                        "outpost".into()
                    } else {
                        "combat".into()
                    },
                });
            }

            // Toggle help screen
            (Some(commands::UiCommand::Help), _) => {
                if screen_state.current == "help" {
                    screen_writer.write(ScreenIntent {
                        screen_id: if *mode == bd_core::spatial::GameMode::Outpost {
                            "outpost".into()
                        } else {
                            "combat".into()
                        },
                    });
                } else {
                    screen_writer.write(ScreenIntent {
                        screen_id: "help".into(),
                    });
                }
            }
            // Enter the fixed foundation dungeon directly.
            (Some(commands::UiCommand::Travel), _) => {
                if *mode == bd_core::spatial::GameMode::Outpost {
                    transition_writer.write(TransitionIntent {
                        target: bd_core::spatial::GameMode::Tactical,
                        node_id: Some("dungeon.foundation".into()),
                    });
                    screen_writer.write(ScreenIntent {
                        screen_id: "combat".into(),
                    });
                } else {
                    game_log.push("Travel only possible from the outpost.", LogLevel::Warn);
                }
            }
            // Explicitly extract from the dungeon through the action pipeline.
            (Some(commands::UiCommand::Extract), _) => {
                if *mode == bd_core::spatial::GameMode::Tactical {
                    action_writer.write(ActionIntent {
                        actor: player_entity,
                        action_id: commands::command_action_id(commands::UiCommand::Extract)
                            .unwrap()
                            .into(),
                        direction: None,
                        target: None,
                    });
                } else {
                    game_log.push(
                        "Extraction is only possible in the dungeon.",
                        LogLevel::Warn,
                    );
                }
            }
            // Assign survivor to nearest station (outpost mode only)
            (Some(commands::UiCommand::AssignStation), _) => {
                if *mode == bd_core::spatial::GameMode::Outpost {
                    // Find nearest survivor
                    let nearest_survivor = input
                        .survivors
                        .iter()
                        .filter(|(_, _, _, scope)| scope_is_active(*scope, *mode))
                        .min_by_key(|(_, sp, _, _)| {
                            ((player_pos.x - sp.x).abs() + (player_pos.y - sp.y).abs()) as u32
                        });
                    // Find nearest station
                    let nearest_station = input
                        .stations
                        .iter()
                        .filter(|(_, _, scope)| scope_is_active(*scope, *mode))
                        .min_by_key(|(_, sp, _)| {
                            ((player_pos.x - sp.x).abs() + (player_pos.y - sp.y).abs()) as u32
                        });
                    if let (Some((survivor_entity, _, _, _)), Some((_station_entity, _, _))) =
                        (nearest_survivor, nearest_station)
                    {
                        action_writer.write(ActionIntent {
                            actor: player_entity,
                            action_id: commands::command_action_id(
                                commands::UiCommand::AssignStation,
                            )
                            .unwrap()
                            .into(),
                            direction: None,
                            target: Some(survivor_entity),
                        });
                    } else {
                        game_log.push(
                            "No survivor or station nearby to assign.".to_string(),
                            LogLevel::Warn,
                        );
                    }
                }
            }
            // Build mode toggle (outpost mode only)
            (Some(commands::UiCommand::Build), _) => {
                if *mode != bd_core::spatial::GameMode::Outpost {
                    return;
                }
                if build_menu.active {
                    // Cancel menu
                    build_menu.active = false;
                    game_log.push("Build cancelled.".to_string(), LogLevel::Info);
                } else if build_ghost.active {
                    // Cancel ghost placement
                    build_ghost.active = false;
                    build_ghost.station_type = None;
                    game_log.push("Build mode cancelled.".to_string(), LogLevel::Info);
                } else {
                    // Open build menu
                    build_menu.active = true;
                    build_menu.selected = 0;
                    game_log.push(
                        "Select station to build (↑↓ or 1-5, Enter to confirm, b to cancel)"
                            .to_string(),
                        LogLevel::Info,
                    );
                }
            }
            // Build menu navigation: up/down arrows when menu is open
            // Number keys 1-5: select in menu, or select type in ghost mode
            (_, KeyCode::Char(c @ '1'..='5')) => {
                let idx = (c as u8 - b'1') as usize;
                let bps = bd_core::colony::stations::default_station_blueprints();
                if build_menu.active {
                    // Direct selection from menu → confirm and enter ghost mode
                    if let Some(bp) = bps.get(idx) {
                        build_ghost.active = true;
                        build_ghost.cursor = *player_pos;
                        build_ghost.station_type = Some(bp.station_type);
                        build_menu.active = false;
                        game_log.push(
                            format!(
                                "Placing: {:?}. arrows=move Enter=place b=cancel",
                                bp.station_type
                            ),
                            LogLevel::Info,
                        );
                    }
                } else if build_ghost.active {
                    if let Some(bp) = bps.get(idx) {
                        build_ghost.station_type = Some(bp.station_type);
                        game_log.push(
                            format!(
                                "Selected: {:?} ({} Supplies)",
                                bp.station_type, bp.build_cost_supplies
                            ),
                            LogLevel::Info,
                        );
                    }
                }
            }
            // Enter: confirm menu selection → enter ghost mode; or place in ghost mode
            (_, KeyCode::Enter) => {
                if build_menu.active {
                    let bps = bd_core::colony::stations::default_station_blueprints();
                    if let Some(bp) = bps.get(build_menu.selected) {
                        build_ghost.active = true;
                        build_ghost.cursor = *player_pos;
                        build_ghost.station_type = Some(bp.station_type);
                        build_menu.active = false;
                        game_log.push(
                            format!(
                                "Placing: {:?}. arrows=move Enter=place b=cancel",
                                bp.station_type
                            ),
                            LogLevel::Info,
                        );
                    }
                    return;
                }
                if build_ghost.active {
                    if let Some(st) = build_ghost.station_type {
                        pending_station.0 = Some(st);
                        let dx = build_ghost.cursor.x - player_pos.x;
                        let dy = build_ghost.cursor.y - player_pos.y;
                        let dir = if dx != 0 {
                            if dx > 0 {
                                Direction::East
                            } else {
                                Direction::West
                            }
                        } else if dy != 0 {
                            if dy > 0 {
                                Direction::South
                            } else {
                                Direction::North
                            }
                        } else {
                            Direction::North
                        };
                        action_writer.write(ActionIntent {
                            actor: player_entity,
                            action_id: commands::command_action_id(commands::UiCommand::Build)
                                .unwrap()
                                .into(),
                            direction: Some(dir),
                            target: None,
                        });
                        build_ghost.active = false;
                        build_ghost.station_type = None;
                    } else {
                        game_log.push(
                            "Select a station type first (1-5).".to_string(),
                            LogLevel::Warn,
                        );
                    }
                }
            }
            // Debug overlay toggle
            (_, KeyCode::F(1)) => {
                screen_writer.write(ScreenIntent {
                    screen_id: "debug".into(),
                });
            }
            // Save game (sets flag; main loop writes to disk)
            (Some(commands::UiCommand::Save), _) => {
                persistence.save.0 = true;
                game_log.push("Save requested.", LogLevel::Info);
            }
            // Load game (sets flag; main loop reads from disk)
            (Some(commands::UiCommand::Load), _) => {
                persistence.load.0 = true;
                game_log.push("Load requested.", LogLevel::Info);
            }
            // Quit (or cancel build mode)
            (Some(commands::UiCommand::Quit), _) => {
                if build_menu.active {
                    build_menu.active = false;
                    game_log.push("Build cancelled.".to_string(), LogLevel::Info);
                } else if build_ghost.active {
                    build_ghost.active = false;
                    build_ghost.station_type = None;
                    pending_station.0 = None;
                    game_log.push("Build mode cancelled.".to_string(), LogLevel::Info);
                } else {
                    exit_request.0 = true;
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
    enemies: &Query<
        (Entity, &Position, Option<&bd_core::spatial::EntityScope>),
        (With<BlocksMovement>, Without<Player>),
    >,
    mode: bd_core::spatial::GameMode,
) -> Option<Entity> {
    let pp = player_pos?;
    enemies
        .iter()
        .filter(|(_, _, scope)| scope_is_active(*scope, mode))
        .min_by_key(|(_, pos, _)| (pos.x - pp.x).unsigned_abs() + (pos.y - pp.y).unsigned_abs())
        .map(|(entity, _, _)| entity)
}

fn scope_is_active(
    scope: Option<&bd_core::spatial::EntityScope>,
    mode: bd_core::spatial::GameMode,
) -> bool {
    scope.is_none_or(|scope| scope.is_active(mode))
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
    runtime: UiRuntimeState,
) {
    let Some(def) = screen_reg.screens.get(&screen_state.current) else {
        tracing::warn!("Unknown screen: {}", screen_state.current);
        return;
    };

    let _ = ratatui_ctx.draw(|frame| {
        let area = frame.area();

        let layout = commands::terminal_layout(area.width, area.height);
        if layout == commands::TerminalLayout::TooSmall {
            let block = Block::default()
                .title(" Terminal Too Small ")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Red));
            let inner = block.inner(area);
            frame.render_widget(block, area);
            let msg = ratatui::widgets::Paragraph::new(vec![
                Line::from(""),
                Line::styled(
                    format!(
                        "Terminal: {}×{} — minimum {}×{} required.",
                        area.width,
                        area.height,
                        commands::MIN_TERMINAL_WIDTH,
                        commands::MIN_TERMINAL_HEIGHT
                    ),
                    Style::default().fg(Color::Yellow),
                ),
                Line::from(""),
                Line::styled(
                    "Please resize your terminal and restart.",
                    Style::default().fg(Color::Gray),
                ),
            ]);
            frame.render_widget(msg, inner);
            return;
        }

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
        };

        const FOOTER_HEIGHT: u16 = 3;
        let content_area = Rect {
            height: area.height.saturating_sub(FOOTER_HEIGHT),
            ..area
        };
        let panel_rects = compute_panel_rects(def, content_area);

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

        let interaction = if *runtime.mode == bd_core::spatial::GameMode::GameOver {
            commands::InteractionMode::GameOver
        } else if runtime.build_ghost.active || runtime.build_menu.active {
            commands::InteractionMode::Build
        } else {
            commands::InteractionMode::Normal
        };
        let help_text = commands::footer_text(&runtime.bindings, *runtime.mode, interaction);
        render_footer(
            frame,
            area,
            &help_text,
            runtime.game_time.turn,
            runtime.game_time.day,
        );
    });
}

fn render_footer(frame: &mut ratatui::Frame, area: Rect, help: &str, turn: u64, day: u64) {
    let version = env!("CARGO_PKG_VERSION");
    let text = format!("Turn: {turn} | Day: {day} | Broken Divinity Kernel v{version} | {help}");
    let footer_area = Rect {
        y: area.y + area.height.saturating_sub(3),
        height: 3,
        ..area
    };
    let para = Paragraph::new(text)
        .alignment(Alignment::Left)
        .wrap(ratatui::widgets::Wrap { trim: false })
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
        let text =
            format!("Turn: {turn} | Day: {day} | Broken Divinity Kernel v{version} | {help}");
        assert!(text.contains("Turn: 5"), "Footer should show turn counter");
        assert!(text.contains("Day: 0"), "Footer should show day counter");
        assert!(
            text.contains("Broken Divinity Kernel"),
            "Footer should show version"
        );
    }
}
