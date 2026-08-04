//! bd_tui — Terminal UI layer for the BD Kernel.
//!
//! Renders Ratatui widgets from view models. Never queries ECS gameplay
//! internals directly.

pub mod chrome;
pub mod commands;
pub mod render_grid;
mod runtime_control;
pub mod screens;
pub mod theme;
pub mod view_models;
pub mod visual;

#[cfg(test)]
mod ui_development_contract_tests;

use std::{
    collections::hash_map::DefaultHasher,
    fmt::Debug,
    hash::{Hash, Hasher},
};

use bevy_app::{App, Plugin};
use bevy_ecs::{
    entity::Entity,
    message::{MessageReader, MessageWriter},
    query::{With, Without},
    schedule::IntoScheduleConfigs,
    system::{Commands, Query, Res, ResMut, SystemParam},
};
use bevy_ratatui::{RatatuiContext, event::KeyMessage};
use ratatui::{layout::Alignment, layout::Rect, text::Line, widgets::Paragraph};

use bd_core::{
    BdSet,
    components::{BlocksMovement, Player, Position},
    direction::Direction,
    gamelog::{GameLog, LogLevel},
    signals::ActionIntent,
    spatial::{TransitionComplete, TransitionIntent},
};

use chrome::{PanelTone, UiTone, command_ribbon, mode_ribbon, panel, style};
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

use runtime_control::{GameplayInputQueue, RenderInvalidation};

/// TUI plugin — registers input mapping and render systems.
pub struct BdTuiPlugin;

#[derive(Debug, Clone, Copy)]
pub(crate) enum ManagementChoice {
    Action(&'static str),
    Station(bevy_ecs::entity::Entity),
}

#[derive(bevy_ecs::prelude::Resource, Debug, Clone, Copy, Default)]
pub(crate) struct ManagementMenuState {
    pub(crate) active: bool,
    pub(crate) kind: ManagementMenuKind,
    pub(crate) selected_survivor: Option<usize>,
    pub(crate) selected_choice: Option<ManagementChoice>,
    pub(crate) selected_recipe: Option<usize>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum ManagementMenuKind {
    #[default]
    TaskAssignment,
    StationStaffing,
}

impl Plugin for BdTuiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<commands::CommandBindings>();
        app.init_resource::<commands::ApplicationExitRequest>();
        app.init_resource::<RenderInvalidation>();
        app.init_resource::<GameplayInputQueue>();
        app.init_resource::<ManagementMenuState>();
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
        app.add_message::<KeyMessage>();

        view_models::register_view_models(app);

        app.add_systems(
            bevy_app::Update,
            (
                sync_event_screen.in_set(BdSet::IntentCollection),
                sync_transition_screen.in_set(BdSet::IntentCollection),
                reset_transient_ui_after_restore
                    .before(map_input_to_intents)
                    .in_set(BdSet::Input),
                map_input_to_intents.in_set(BdSet::Input),
                screens::process_screen_intents.in_set(BdSet::IntentCollection),
                draw_ui.in_set(BdSet::Render),
            ),
        );

        tracing::info!("BdTuiPlugin initialized");
    }
}

fn reset_transient_ui_after_restore(
    restored: Option<Res<bd_core::save::WorldJustRestored>>,
    mut commands: Commands,
    mut management: ResMut<ManagementMenuState>,
    mut input_queue: ResMut<GameplayInputQueue>,
) {
    if restored.is_none() {
        return;
    }
    *management = ManagementMenuState::default();
    input_queue.clear();
    commands.remove_resource::<bd_core::save::WorldJustRestored>();
}

fn sync_transition_screen(
    mut transitions: MessageReader<TransitionComplete>,
    mut screens: MessageWriter<ScreenIntent>,
    mut input_queue: ResMut<GameplayInputQueue>,
) {
    for transition in transitions.read() {
        input_queue.clear();
        let screen_id = match transition.to {
            bd_core::spatial::GameMode::Title => "title",
            bd_core::spatial::GameMode::Outpost => "outpost",
            bd_core::spatial::GameMode::Tactical => "combat",
            bd_core::spatial::GameMode::GameOver => "game_over",
            bd_core::spatial::GameMode::Travel => continue,
        };
        screens.write(ScreenIntent {
            screen_id: screen_id.into(),
        });
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
#[allow(clippy::type_complexity)] // Input routing keeps each scoped gameplay query explicit.
struct InputQueries<'w, 's> {
    player: Query<
        'w,
        's,
        (
            Entity,
            &'static Position,
            Option<&'static bd_core::spatial::EntityScope>,
            Option<&'static bd_core::time::AwaitingEnemyPhase>,
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
            &'static bd_core::components::Name,
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
            &'static bd_core::components::Name,
            &'static Position,
            Option<&'static bd_core::components::ContentIdentity>,
            Option<&'static bd_core::spatial::EntityScope>,
        ),
        (
            With<bd_core::colony::stations::Station>,
            Without<bd_core::colony::stations::ConstructionSite>,
        ),
    >,
    exits: Query<
        'w,
        's,
        (
            &'static Position,
            Option<&'static bd_core::spatial::EntityScope>,
        ),
        With<bd_core::components::ExitTile>,
    >,
}

#[derive(SystemParam)]
struct PersistenceRequests<'w> {
    save: ResMut<'w, bd_core::save::SaveRequest>,
    load: ResMut<'w, bd_core::save::LoadRequest>,
}

#[derive(SystemParam)]
struct ColonyInteractionState<'w> {
    pending_station_assignment: ResMut<'w, bd_core::colony::stations::PendingStationAssignment>,
    build: ResMut<'w, bd_core::colony::stations::BuildInteraction>,
    management: ResMut<'w, ManagementMenuState>,
    pending_recipe: ResMut<'w, bd_core::colony::logistics::PendingRecipeAssignment>,
    outpost: Res<'w, bd_core::spatial::OutpostState>,
    foundation_content: Option<Res<'w, bd_core::content::FoundationContent>>,
}

#[derive(SystemParam)]
struct UiRuntimeState<'w> {
    game_time: Res<'w, bd_core::time::GameTime>,
    bindings: Res<'w, commands::CommandBindings>,
    mode: Res<'w, bd_core::spatial::GameMode>,
    build: Res<'w, bd_core::colony::stations::BuildInteraction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PredictedBuildInteraction {
    Normal,
    Menu,
    Ghost,
    Awaiting,
}

fn management_choice_for_index(
    kind: ManagementMenuKind,
    index: usize,
    stations: &[Entity],
) -> Option<ManagementChoice> {
    match kind {
        ManagementMenuKind::TaskAssignment => match index {
            0 => Some(ManagementChoice::Action("ability.assign_idle")),
            1 => Some(ManagementChoice::Action("ability.gather_supplies")),
            2 => Some(ManagementChoice::Action("ability.gather_materials")),
            3 => Some(ManagementChoice::Action("ability.gather_plants")),
            4 => Some(ManagementChoice::Action("ability.assign_resting")),
            _ => None,
        },
        ManagementMenuKind::StationStaffing => {
            stations.get(index).copied().map(ManagementChoice::Station)
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn route_management_key(
    key: crossterm::event::KeyCode,
    management: &mut ManagementMenuState,
    survivors: &[Entity],
    stations: &[Entity],
    processor_station: Option<Entity>,
    recipe_ids: &[String],
    player: Entity,
    pending_station_assignment: &mut bd_core::colony::stations::PendingStationAssignment,
    pending_recipe: &mut bd_core::colony::logistics::PendingRecipeAssignment,
    action_writer: &mut MessageWriter<ActionIntent>,
    game_log: &mut GameLog,
) {
    use crossterm::event::KeyCode;

    let cancel_key = match management.kind {
        ManagementMenuKind::TaskAssignment => KeyCode::Char('c'),
        ManagementMenuKind::StationStaffing => KeyCode::Char('e'),
    };
    match key {
        key if key == KeyCode::Esc || key == cancel_key => {
            management.active = false;
            management.selected_survivor = None;
            management.selected_choice = None;
            management.selected_recipe = None;
            game_log.push("Management cancelled.", LogLevel::Info);
        }
        KeyCode::Enter => {
            let (Some(survivor_index), Some(choice)) =
                (management.selected_survivor, management.selected_choice)
            else {
                return;
            };
            let Some(survivor) = survivors.get(survivor_index).copied() else {
                return;
            };
            let action_id = match choice {
                ManagementChoice::Action(action_id) => action_id,
                ManagementChoice::Station(station) => {
                    if Some(station) == processor_station {
                        let Some(recipe_index) = management.selected_recipe else {
                            return;
                        };
                        let Some(recipe_id) = recipe_ids.get(recipe_index) else {
                            return;
                        };
                        pending_recipe.0 = Some(recipe_id.clone());
                        "ability.assign_recipe"
                    } else {
                        pending_station_assignment.0 = Some(station);
                        "ability.assign_station"
                    }
                }
            };
            action_writer.write(ActionIntent {
                actor: player,
                action_id: action_id.into(),
                direction: None,
                target: Some(survivor),
            });
            management.active = false;
            management.selected_survivor = None;
            management.selected_choice = None;
            management.selected_recipe = None;
        }
        KeyCode::Char(choice @ '1'..='9') => {
            let index = (choice as u8 - b'1') as usize;
            if management.selected_survivor.is_none() {
                if index < survivors.len() {
                    management.selected_survivor = Some(index);
                }
                return;
            }
            if matches!(
                management.selected_choice,
                Some(ManagementChoice::Station(station)) if Some(station) == processor_station
            ) {
                if index < recipe_ids.len() {
                    management.selected_recipe = Some(index);
                }
            } else {
                management.selected_choice =
                    management_choice_for_index(management.kind, index, stations);
            }
        }
        _ => {}
    }
}

impl PredictedBuildInteraction {
    fn from_state(build: &bd_core::colony::stations::BuildInteraction) -> Self {
        match build {
            bd_core::colony::stations::BuildInteraction::Inactive => Self::Normal,
            bd_core::colony::stations::BuildInteraction::Selecting { .. } => Self::Menu,
            bd_core::colony::stations::BuildInteraction::Placing { .. } => Self::Ghost,
            bd_core::colony::stations::BuildInteraction::AwaitingResolution { .. } => {
                Self::Awaiting
            }
        }
    }

    fn command_context(self) -> commands::InteractionMode {
        match self {
            Self::Normal => commands::InteractionMode::Normal,
            Self::Menu | Self::Ghost | Self::Awaiting => commands::InteractionMode::Build,
        }
    }

    /// Predict modal state changes while classifying one terminal input batch.
    ///
    /// The real state changes later in the same order. Prediction only keeps
    /// build controls immediate instead of incorrectly placing them in the
    /// gameplay queue.
    fn observe(
        &mut self,
        command: Option<commands::UiCommand>,
        key: &crossterm::event::KeyCode,
    ) -> bool {
        use crossterm::event::KeyCode;

        match (command, *key, *self) {
            (Some(commands::UiCommand::Build), _, Self::Normal) => *self = Self::Menu,
            (Some(commands::UiCommand::Build | commands::UiCommand::Quit), _, _) => {
                *self = Self::Normal;
            }
            (_, KeyCode::Enter, Self::Menu) => *self = Self::Ghost,
            (_, KeyCode::Enter, Self::Ghost) => {
                *self = Self::Awaiting;
                return true;
            }
            _ => {}
        }
        false
    }
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
    colony_interaction: ColonyInteractionState,
    mut persistence: PersistenceRequests,
    current_event: Option<Res<bd_core::events::CurrentEvent>>,
    mut event_writer: MessageWriter<bd_core::signals::EventSelected>,
    bindings: Res<commands::CommandBindings>,
    station_catalog: Res<bd_core::colony::stations::StationCatalog>,
    mut exit_request: ResMut<commands::ApplicationExitRequest>,
    mut input_queue: ResMut<GameplayInputQueue>,
) {
    use crossterm::event::{KeyCode, KeyEventKind};
    let ColonyInteractionState {
        mut pending_station_assignment,
        mut build,
        mut management,
        mut pending_recipe,
        outpost,
        foundation_content,
    } = colony_interaction;

    // Game over preserves the terminal outcome until the player explicitly
    // restarts, loads a save, or quits.
    if *mode == bd_core::spatial::GameMode::GameOver {
        input_queue.clear();
        if screen_state.current != "game_over" {
            screen_writer.write(ScreenIntent {
                screen_id: "game_over".into(),
            });
        } else if let Some(key) = messages.read().find(|key| key.kind == KeyEventKind::Press) {
            match commands::game_over_input(&bindings, &key.code) {
                Some(commands::GameOverInput::Restart) => {
                    transition_writer.write(TransitionIntent {
                        target: bd_core::spatial::GameMode::Title,
                        node_id: None,
                    });
                    screen_writer.write(ScreenIntent {
                        screen_id: "title".into(),
                    });
                    game_log.push("Restarting the run.", LogLevel::Info);
                }
                Some(commands::GameOverInput::Save) => {
                    persistence.save.0 = true;
                    game_log.push("Save requested.", LogLevel::Info);
                }
                Some(commands::GameOverInput::Load) => {
                    persistence.load.0 = true;
                    game_log.push("Load requested.", LogLevel::Info);
                }
                Some(commands::GameOverInput::Quit) => {
                    exit_request.0 = true;
                }
                None => {}
            }
        }
        return;
    }

    // Title accepts explicit load/quit controls; every other key begins a run.
    if *mode == bd_core::spatial::GameMode::Title {
        input_queue.clear();
        for key in messages
            .read()
            .filter(|key| key.kind == KeyEventKind::Press)
        {
            match commands::title_input(&bindings, &key.code) {
                commands::TitleInput::Load => {
                    persistence.load.0 = true;
                    game_log.push("Load requested.", LogLevel::Info);
                }
                commands::TitleInput::Quit => {
                    exit_request.0 = true;
                }
                commands::TitleInput::Begin => {
                    transition_writer.write(TransitionIntent {
                        target: bd_core::spatial::GameMode::Outpost,
                        node_id: None,
                    });
                    screen_writer.write(ScreenIntent {
                        screen_id: "outpost".into(),
                    });
                    // Preserve the build shortcut across the startup frame.
                    if bindings
                        .key_for(commands::UiCommand::Build)
                        .is_some_and(|binding| *binding == key.code)
                    {
                        let selected_station = station_catalog
                            .entries()
                            .first()
                            .map_or(bd_core::colony::stations::StationType::Stove, |entry| {
                                entry.station_type
                            });
                        *build = bd_core::colony::stations::BuildInteraction::Selecting {
                            selected_station,
                        };
                    }
                }
            }
        }
        return;
    }

    let Some((player_entity, player_pos, _, awaiting_enemy_phase)) = input
        .player
        .iter()
        .find(|(_, _, scope, _)| scope_is_active(*scope, *mode))
    else {
        // Player not spawned yet (spawn_outpost_player runs after Input set).
        return;
    };

    if management.active {
        input_queue.clear();
        let mut survivors: Vec<_> = input
            .survivors
            .iter()
            .filter(|(_, _, _, _, scope)| scope_is_active(*scope, *mode))
            .map(|(entity, name, _, _, _)| (name.0.clone(), entity))
            .collect();
        survivors.sort_by(|left, right| left.0.cmp(&right.0));
        let survivors = survivors
            .into_iter()
            .map(|(_, entity)| entity)
            .collect::<Vec<_>>();
        let mut stations: Vec<_> = input
            .stations
            .iter()
            .filter(|(_, _, _, _, scope)| scope_is_active(*scope, *mode))
            .map(|(entity, name, position, _, _)| (name.0.clone(), position.y, position.x, entity))
            .collect();
        stations
            .sort_by(|left, right| (&left.0, left.1, left.2).cmp(&(&right.0, right.1, right.2)));
        let stations = stations
            .into_iter()
            .map(|(_, _, _, entity)| entity)
            .collect::<Vec<_>>();
        let processor_station = input
            .stations
            .iter()
            .find(|(_, _, _, identity, scope)| {
                scope_is_active(*scope, *mode)
                    && identity.is_some_and(|identity| identity.0 == "station.basic_processor")
            })
            .map(|(entity, _, _, _, _)| entity);
        let recipe_ids = foundation_content
            .as_deref()
            .map(|content| {
                content
                    .colony_recipes
                    .iter()
                    .map(|recipe| recipe.id.clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        for key in messages
            .read()
            .filter(|key| key.kind == KeyEventKind::Press)
        {
            route_management_key(
                key.code,
                &mut management,
                &survivors,
                &stations,
                processor_station,
                &recipe_ids,
                player_entity,
                &mut pending_station_assignment,
                &mut pending_recipe,
                &mut action_writer,
                &mut game_log,
            );
        }
        return;
    }

    // If an event is active, only number keys for choices are handled
    if current_event
        .as_ref()
        .is_some_and(|event| event.is_active())
    {
        input_queue.clear();
        for key in messages
            .read()
            .filter(|key| key.kind == KeyEventKind::Press)
        {
            if let KeyCode::Char(c @ '1'..='9') = key.code {
                let idx = (c as u8 - b'1') as usize;
                event_writer.write(bd_core::signals::EventSelected {
                    actor: player_entity,
                    choice_index: idx,
                });
            }
        }
        return;
    }

    let action_locked = awaiting_enemy_phase.is_some();
    let mut routed_input = Vec::new();
    let mut predicted_interaction = PredictedBuildInteraction::from_state(&build);
    let mut predicted_management = false;
    let mut immediate_gameplay_submitted = false;
    for key in messages
        .read()
        .filter(|key| key.kind == KeyEventKind::Press)
    {
        let interaction = predicted_interaction.command_context();
        let command = bindings.command_for_key_in(&key.code, *mode, interaction);
        let opens_management = command.is_some_and(|command| {
            matches!(
                command,
                commands::UiCommand::AssignTask | commands::UiCommand::AssignStation
            )
        });
        if opens_management {
            routed_input.push((command, key.code));
        } else if !predicted_management
            && interaction == commands::InteractionMode::Normal
            && command.is_some_and(commands::is_buffered_gameplay)
        {
            input_queue.enqueue(command.unwrap(), action_locked);
        } else {
            routed_input.push((command, key.code));
        }
        if opens_management {
            predicted_management = true;
        }
        immediate_gameplay_submitted |= predicted_interaction.observe(command, &key.code);
    }
    if predicted_interaction != PredictedBuildInteraction::Normal || predicted_management {
        input_queue.clear();
    } else if !action_locked && !immediate_gameplay_submitted {
        if let Some(command) = input_queue.pop_front() {
            let key = bindings.key_for(command).cloned().unwrap_or(KeyCode::Null);
            routed_input.push((Some(command), key));
        }
    }
    if input_queue.take_overflow_warning() {
        game_log.push(
            "Input queue full; additional gameplay commands were dropped.",
            LogLevel::Warn,
        );
    }

    let mut stable_survivors = input
        .survivors
        .iter()
        .filter(|(_, _, _, _, scope)| scope_is_active(*scope, *mode))
        .map(|(entity, name, _, _, _)| (name.0.clone(), entity))
        .collect::<Vec<_>>();
    stable_survivors.sort_by(|left, right| left.0.cmp(&right.0));
    let stable_survivors = stable_survivors
        .into_iter()
        .map(|(_, entity)| entity)
        .collect::<Vec<_>>();
    let mut stable_stations = input
        .stations
        .iter()
        .filter(|(_, _, _, _, scope)| scope_is_active(*scope, *mode))
        .map(|(entity, name, position, _, _)| (name.0.clone(), position.y, position.x, entity))
        .collect::<Vec<_>>();
    stable_stations
        .sort_by(|left, right| (&left.0, left.1, left.2).cmp(&(&right.0, right.1, right.2)));
    let stable_stations = stable_stations
        .into_iter()
        .map(|(_, _, _, entity)| entity)
        .collect::<Vec<_>>();
    let processor_station = input
        .stations
        .iter()
        .find(|(_, _, _, identity, scope)| {
            scope_is_active(*scope, *mode)
                && identity.is_some_and(|identity| identity.0 == "station.basic_processor")
        })
        .map(|(entity, _, _, _, _)| entity);
    let recipe_ids = foundation_content
        .as_deref()
        .map(|content| {
            content
                .colony_recipes
                .iter()
                .map(|recipe| recipe.id.clone())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let validate_build_candidate = |candidate: Position| {
        let gate = input
            .exits
            .iter()
            .filter(|(_, scope)| scope_is_active(*scope, *mode))
            .map(|(position, _)| *position)
            .next()
            .ok_or(
                bd_core::colony::stations::BuildInteractionDenial::Placement(
                    bd_core::colony::stations::StationPlacementDenial::WouldBlockShelterEgress,
                ),
            )?;
        let permanent_blockers = input
            .stations
            .iter()
            .filter(|(_, _, _, _, scope)| scope_is_active(*scope, *mode))
            .map(|(_, _, position, _, _)| *position)
            .collect();
        bd_core::colony::stations::validate_station_placement(
            &outpost.map,
            *player_pos,
            gate,
            &permanent_blockers,
            candidate,
        )
        .map_err(bd_core::colony::stations::BuildInteractionDenial::Placement)
    };

    for (command, key_code) in routed_input {
        if management.active {
            route_management_key(
                key_code,
                &mut management,
                &stable_survivors,
                &stable_stations,
                processor_station,
                &recipe_ids,
                player_entity,
                &mut pending_station_assignment,
                &mut pending_recipe,
                &mut action_writer,
                &mut game_log,
            );
            continue;
        }
        match (command, key_code) {
            // Movement — ghost cursor in build mode, normal movement otherwise
            (Some(commands::UiCommand::MoveNorth), _) => {
                match &mut *build {
                    bd_core::colony::stations::BuildInteraction::Selecting { selected_station } => {
                        let entries = station_catalog.entries();
                        let index = entries
                            .iter()
                            .position(|entry| entry.station_type == *selected_station)
                            .unwrap_or(0)
                            .saturating_sub(1);
                        if let Some(entry) = entries.get(index) {
                            *selected_station = entry.station_type;
                        }
                        continue;
                    }
                    bd_core::colony::stations::BuildInteraction::Placing {
                        cursor,
                        validation,
                        ..
                    } => {
                        *cursor = Position {
                            x: cursor.x,
                            y: (cursor.y - 1).max(0),
                        };
                        *validation = validate_build_candidate(*cursor);
                        continue;
                    }
                    bd_core::colony::stations::BuildInteraction::AwaitingResolution { .. } => {
                        continue;
                    }
                    bd_core::colony::stations::BuildInteraction::Inactive => {}
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
                match &mut *build {
                    bd_core::colony::stations::BuildInteraction::Selecting { selected_station } => {
                        let entries = station_catalog.entries();
                        let current = entries
                            .iter()
                            .position(|entry| entry.station_type == *selected_station)
                            .unwrap_or(0);
                        if let Some(entry) = entries.get(current + 1) {
                            *selected_station = entry.station_type;
                        }
                        continue;
                    }
                    bd_core::colony::stations::BuildInteraction::Placing {
                        cursor,
                        validation,
                        ..
                    } => {
                        *cursor = Position {
                            x: cursor.x,
                            y: (cursor.y + 1).min(outpost.map.height - 1),
                        };
                        *validation = validate_build_candidate(*cursor);
                        continue;
                    }
                    bd_core::colony::stations::BuildInteraction::AwaitingResolution { .. } => {
                        continue;
                    }
                    bd_core::colony::stations::BuildInteraction::Inactive => {}
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
                match &mut *build {
                    bd_core::colony::stations::BuildInteraction::Selecting { .. }
                    | bd_core::colony::stations::BuildInteraction::AwaitingResolution { .. } => {
                        continue;
                    }
                    bd_core::colony::stations::BuildInteraction::Placing {
                        cursor,
                        validation,
                        ..
                    } => {
                        *cursor = Position {
                            x: (cursor.x + 1).min(outpost.map.width - 1),
                            y: cursor.y,
                        };
                        *validation = validate_build_candidate(*cursor);
                        continue;
                    }
                    bd_core::colony::stations::BuildInteraction::Inactive => {}
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
                    input_queue.clear();
                    management.active = true;
                    management.kind = ManagementMenuKind::TaskAssignment;
                    management.selected_survivor = None;
                    management.selected_choice = None;
                    management.selected_recipe = None;
                }
            }
            (Some(commands::UiCommand::MoveWest), _) => {
                match &mut *build {
                    bd_core::colony::stations::BuildInteraction::Selecting { .. }
                    | bd_core::colony::stations::BuildInteraction::AwaitingResolution { .. } => {
                        continue;
                    }
                    bd_core::colony::stations::BuildInteraction::Placing {
                        cursor,
                        validation,
                        ..
                    } => {
                        *cursor = Position {
                            x: (cursor.x - 1).max(0),
                            y: cursor.y,
                        };
                        *validation = validate_build_candidate(*cursor);
                        continue;
                    }
                    bd_core::colony::stations::BuildInteraction::Inactive => {}
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
            (Some(commands::UiCommand::RestUntilNextDay), _) => {
                action_writer.write(ActionIntent {
                    actor: player_entity,
                    action_id: commands::command_action_id(commands::UiCommand::RestUntilNextDay)
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
                    screen_id: commands::inventory_toggle_destination(&screen_state.current, *mode)
                        .into(),
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
            // Enter the fixed Foundation dungeon through the action pipeline.
            (Some(commands::UiCommand::Travel), _) => {
                action_writer.write(ActionIntent {
                    actor: player_entity,
                    action_id: "ability.enter_foundation_dungeon".into(),
                    direction: None,
                    target: None,
                });
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
                    input_queue.clear();
                    management.active = true;
                    management.kind = ManagementMenuKind::StationStaffing;
                    management.selected_survivor = None;
                    management.selected_choice = None;
                    management.selected_recipe = None;
                }
            }
            // Build mode toggle (outpost mode only)
            (Some(commands::UiCommand::Build), _) => {
                if *mode != bd_core::spatial::GameMode::Outpost {
                    continue;
                }
                if build.is_active() {
                    *build = bd_core::colony::stations::BuildInteraction::Inactive;
                    game_log.push("Build mode cancelled.".to_string(), LogLevel::Info);
                } else {
                    let selected_station = station_catalog
                        .entries()
                        .first()
                        .map_or(bd_core::colony::stations::StationType::Stove, |entry| {
                            entry.station_type
                        });
                    *build =
                        bd_core::colony::stations::BuildInteraction::Selecting { selected_station };
                    let numeric_choices = station_catalog.entries().len().min(9);
                    game_log.push(
                        format!(
                            "Select station to build (↑↓ or 1-{numeric_choices}, Enter to confirm, b to cancel)"
                        ),
                        LogLevel::Info,
                    );
                }
            }
            // Build menu navigation: up/down arrows when menu is open
            // Number keys 1-9 select the corresponding data-driven entry.
            (_, KeyCode::Char(c @ '1'..='9')) => {
                let idx = (c as u8 - b'1') as usize;
                let bps = station_catalog.entries();
                if let bd_core::colony::stations::BuildInteraction::Selecting { selected_station } =
                    &mut *build
                    && let Some(entry) = bps.get(idx)
                {
                    *selected_station = entry.station_type;
                }
            }
            // Enter: confirm menu selection → enter ghost mode; or place in ghost mode
            (_, KeyCode::Enter) => {
                if let bd_core::colony::stations::BuildInteraction::Selecting { selected_station } =
                    *build
                {
                    if let Some(bp) = station_catalog.get(selected_station) {
                        if !bp.buildable {
                            game_log.push(
                                bp.unavailable_reason
                                    .clone()
                                    .unwrap_or_else(|| "Station is unavailable.".into()),
                                LogLevel::Warn,
                            );
                            continue;
                        }
                        let cursor = [
                            Position {
                                x: player_pos.x + 1,
                                y: player_pos.y,
                            },
                            Position {
                                x: player_pos.x,
                                y: player_pos.y + 1,
                            },
                            Position {
                                x: player_pos.x - 1,
                                y: player_pos.y,
                            },
                            Position {
                                x: player_pos.x,
                                y: player_pos.y - 1,
                            },
                        ]
                        .into_iter()
                        .find(|candidate| {
                            candidate.x >= 0
                                && candidate.y >= 0
                                && candidate.x < outpost.map.width
                                && candidate.y < outpost.map.height
                        })
                        .unwrap_or(*player_pos);
                        let validation = validate_build_candidate(cursor);
                        *build = bd_core::colony::stations::BuildInteraction::Placing {
                            selected_station,
                            cursor,
                            validation,
                        };
                        game_log.push(
                            format!(
                                "Placing: {:?}. arrows=move Enter=place b=cancel",
                                bp.station_type
                            ),
                            LogLevel::Info,
                        );
                    }
                    continue;
                }
                if let bd_core::colony::stations::BuildInteraction::Placing {
                    selected_station,
                    cursor,
                    validation,
                } = &*build
                {
                    if let Err(reason) = validation {
                        game_log.push(reason.to_string(), LogLevel::Warn);
                        continue;
                    }
                    let selected_station = *selected_station;
                    let cursor = *cursor;
                    let dx = cursor.x - player_pos.x;
                    let dy = cursor.y - player_pos.y;
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
                    *build = bd_core::colony::stations::BuildInteraction::AwaitingResolution {
                        selected_station,
                        cursor,
                    };
                    action_writer.write(ActionIntent {
                        actor: player_entity,
                        action_id: commands::command_action_id(commands::UiCommand::Build)
                            .unwrap()
                            .into(),
                        direction: Some(dir),
                        target: None,
                    });
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
                if build.is_active() {
                    *build = bd_core::colony::stations::BuildInteraction::Inactive;
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

fn frame_interaction(
    mode: bd_core::spatial::GameMode,
    build_active: bool,
    management: Option<view_models::ManagementMenuKind>,
) -> commands::InteractionMode {
    if mode == bd_core::spatial::GameMode::GameOver {
        commands::InteractionMode::GameOver
    } else if let Some(kind) = management {
        match kind {
            view_models::ManagementMenuKind::TaskAssignment => {
                commands::InteractionMode::TaskManagement
            }
            view_models::ManagementMenuKind::StationStaffing => {
                commands::InteractionMode::StationStaffing
            }
        }
    } else if build_active {
        commands::InteractionMode::Build
    } else {
        commands::InteractionMode::Normal
    }
}

/// Draw the full TUI layout driven by the current screen definition.
#[allow(clippy::too_many_arguments)]
fn draw_ui(
    ratatui_ctx: Option<ResMut<RatatuiContext>>,
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
    mut invalidation: ResMut<RenderInvalidation>,
    mut exit_request: ResMut<commands::ApplicationExitRequest>,
) {
    let Some(mut ratatui_ctx) = ratatui_ctx else {
        return;
    };
    let Some(def) = screen_reg.screens.get(&screen_state.current) else {
        tracing::warn!("Unknown screen: {}", screen_state.current);
        return;
    };
    let terminal_size = match ratatui_ctx.size() {
        Ok(size) => size,
        Err(error) => {
            let message = format!("Unable to read terminal size: {error}");
            complete_draw_attempt(&mut invalidation, &mut exit_request, Err(message));
            return;
        }
    };

    let interaction = frame_interaction(
        *runtime.mode,
        runtime.build.is_active(),
        stats_vm.management.as_ref().map(|menu| menu.kind),
    );
    let frame_data = UiFrameData {
        definition: def,
        widgets: &widget_reg,
        map: &map_vm,
        stats: &stats_vm,
        log: &log_vm,
        actions: &action_vm,
        container: &container_vm,
        event: &event_vm,
        help: &help_vm,
        symbols: &symbols,
        theme: &theme,
        bindings: &runtime.bindings,
        mode: *runtime.mode,
        interaction,
        turn: runtime.game_time.turn,
        day: runtime.game_time.day,
    };

    let fingerprint = visible_frame_fingerprint(
        terminal_size.width,
        terminal_size.height,
        &screen_state.current,
        def,
        &map_vm,
        &stats_vm,
        &log_vm,
        &action_vm,
        &container_vm,
        &event_vm,
        &help_vm,
        &symbols,
        &theme,
        &runtime.bindings,
        *runtime.mode,
        &runtime.build,
        runtime.game_time.turn,
        runtime.game_time.day,
    );
    if !invalidation.needs_draw(fingerprint) {
        return;
    }

    match ratatui_ctx
        .draw(|frame| render_ui_frame(frame, &frame_data))
        .map(|_| ())
    {
        Ok(()) => complete_draw_attempt(&mut invalidation, &mut exit_request, Ok(())),
        Err(error) => {
            let message = format!("Terminal draw failed: {error}");
            complete_draw_attempt(&mut invalidation, &mut exit_request, Err(message));
        }
    }
}

fn complete_draw_attempt(
    invalidation: &mut RenderInvalidation,
    exit_request: &mut commands::ApplicationExitRequest,
    result: Result<(), String>,
) {
    if let Err(error) = &result {
        tracing::error!("{error}");
        exit_request.0 = true;
    }
    invalidation.record_draw_result(result);
    tracing::trace!(
        draw_count = invalidation.draw_count(),
        "terminal draw attempt"
    );
}

#[allow(clippy::too_many_arguments)]
fn visible_frame_fingerprint(
    width: u16,
    height: u16,
    screen_id: &str,
    definition: &screens::ScreenDefinition,
    map: &MapViewModel,
    stats: &StatsViewModel,
    log: &LogViewModel,
    actions: &ActionListViewModel,
    container: &ContainerViewModel,
    event: &EventViewModel,
    help: &HelpViewModel,
    symbols: &SymbolRegistry,
    theme: &ThemeRegistry,
    bindings: &commands::CommandBindings,
    mode: bd_core::spatial::GameMode,
    build: &bd_core::colony::stations::BuildInteraction,
    turn: u64,
    day: u64,
) -> u64 {
    fn hash_debug(hasher: &mut DefaultHasher, value: &impl Debug) {
        format!("{value:?}").hash(hasher);
    }

    let mut hasher = DefaultHasher::new();
    width.hash(&mut hasher);
    height.hash(&mut hasher);
    screen_id.hash(&mut hasher);
    hash_debug(&mut hasher, definition);
    hash_debug(&mut hasher, map);
    hash_debug(&mut hasher, stats);
    hash_debug(&mut hasher, log);
    hash_debug(&mut hasher, actions);
    hash_debug(&mut hasher, container);
    hash_debug(&mut hasher, event);
    hash_debug(&mut hasher, help);
    hash_debug(&mut hasher, symbols);
    hash_debug(&mut hasher, theme);
    hash_debug(&mut hasher, bindings);
    hash_debug(&mut hasher, &mode);
    hash_debug(&mut hasher, build);
    turn.hash(&mut hasher);
    day.hash(&mut hasher);
    hasher.finish()
}

struct UiFrameData<'a> {
    definition: &'a screens::ScreenDefinition,
    widgets: &'a WidgetRegistry,
    map: &'a MapViewModel,
    stats: &'a StatsViewModel,
    log: &'a LogViewModel,
    actions: &'a ActionListViewModel,
    container: &'a ContainerViewModel,
    event: &'a EventViewModel,
    help: &'a HelpViewModel,
    symbols: &'a SymbolRegistry,
    theme: &'a ThemeRegistry,
    bindings: &'a commands::CommandBindings,
    mode: bd_core::spatial::GameMode,
    interaction: commands::InteractionMode,
    turn: u64,
    day: u64,
}

fn render_ui_frame(frame: &mut ratatui::Frame, data: &UiFrameData<'_>) {
    let area = frame.area();

    let layout = commands::terminal_layout(area.width, area.height);
    if layout == commands::TerminalLayout::TooSmall {
        let block = panel(data.theme, "Terminal Too Small", PanelTone::Danger);
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
                style(data.theme, UiTone::Warning),
            ),
            Line::from(""),
            Line::styled(
                "Please resize your terminal and restart.",
                style(data.theme, UiTone::Muted),
            ),
        ]);
        frame.render_widget(msg, inner);
        return;
    }

    let compact_definition;
    let definition = if layout == commands::TerminalLayout::Compact {
        compact_definition = screens::compact_screen_definition(data.definition);
        &compact_definition
    } else {
        data.definition
    };

    let wctx = WidgetRenderContext {
        map: data.map,
        stats: data.stats,
        log: data.log,
        actions: data.actions,
        container: data.container,
        event: data.event,
        help: data.help,
        symbols: data.symbols,
        theme: data.theme,
        mode: data.mode,
        screen_id: definition.id.as_str(),
    };

    const FOOTER_HEIGHT: u16 = 3;
    let content_area = Rect {
        height: area.height.saturating_sub(FOOTER_HEIGHT),
        ..area
    };
    let panel_rects = compute_panel_rects(definition, content_area);

    for (panel_id, rect) in &panel_rects {
        if let Some(binding) = data.widgets.bindings.get(panel_id.as_str()) {
            (binding.render)(frame, *rect, &wctx);
        } else {
            let block = panel(
                data.theme,
                format!("Unknown widget: {panel_id}"),
                PanelTone::Danger,
            );
            frame.render_widget(block, *rect);
        }
    }

    render_footer(
        frame,
        area,
        definition.id.as_str(),
        data.bindings,
        data.mode,
        data.interaction,
        data.turn,
        data.day,
        data.theme,
    );

    // Structural Ruined Reliquary frame: painted after the panels and footer so
    // the continuous double-line perimeter is never overwritten by content.
    // Splash screens stay frame-less to keep their centered layout intact.
    if !matches!(definition.id.as_str(), "title" | "game_over") {
        chrome::render_outer_frame(frame, data.theme);
    }

    // Build and management overlays render last but stay inside the shell-owned
    // content rect (above the footer) so they never cover the structural
    // perimeter cells or the status footer.
    let overlay_area = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(4),
    };
    screens::render_build_overlay(frame, overlay_area, &wctx);
}

#[allow(clippy::too_many_arguments)]
fn render_footer(
    frame: &mut ratatui::Frame,
    area: Rect,
    screen_id: &str,
    bindings: &commands::CommandBindings,
    mode: bd_core::spatial::GameMode,
    interaction: commands::InteractionMode,
    turn: u64,
    day: u64,
    theme: &ThemeRegistry,
) {
    // The footer lives inside the Ruined Reliquary frame. Shared chrome turns
    // semantic state and command projections into one mode ribbon and one command
    // ribbon; individual screens do not invent footer variants.
    let version = env!("CARGO_PKG_VERSION");
    let footer_area = Rect {
        x: area.x + 1,
        y: area.y + area.height.saturating_sub(3),
        width: area.width.saturating_sub(2),
        height: 2,
    };
    let controls =
        commands::footer_control_lines(bindings, mode, interaction, screen_id, footer_area.width);
    let mode_label = match interaction {
        commands::InteractionMode::Build => "BUILD".to_owned(),
        commands::InteractionMode::TaskManagement => "TASK MANAGEMENT".to_owned(),
        commands::InteractionMode::StationStaffing => "STATION STAFFING".to_owned(),
        commands::InteractionMode::GameOver => "GAME OVER".to_owned(),
        commands::InteractionMode::Normal => screen_id.replace('_', " ").to_ascii_uppercase(),
    };
    let para = Paragraph::new(vec![
        mode_ribbon(theme, mode_label, day, turn, version),
        command_ribbon(
            theme,
            &[controls.global.as_str(), controls.contextual.as_str()],
            footer_area.width,
        ),
    ])
    .alignment(Alignment::Left);
    frame.render_widget(para, footer_area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::visual::VisualToken;
    use bd_core::{
        colony::stations::{BuildInteraction, StationType, default_station_blueprints},
        components::{Position, Tile},
        gamelog::LogLevel,
        spatial::GameMode,
    };
    use ratatui::{Terminal, backend::TestBackend, buffer::Buffer};

    #[allow(clippy::too_many_arguments)] // Test helper mirrors the independent frame inputs.
    fn test_visible_fingerprint(
        width: u16,
        height: u16,
        screen: &str,
        mode: GameMode,
        map: &MapViewModel,
        log: &LogViewModel,
        container: &ContainerViewModel,
        build: &BuildInteraction,
    ) -> u64 {
        let definition = screens::ScreenDefinition {
            id: screen.into(),
            panels: Vec::new(),
        };
        visible_frame_fingerprint(
            width,
            height,
            screen,
            &definition,
            map,
            &StatsViewModel::default(),
            log,
            &ActionListViewModel::default(),
            container,
            &EventViewModel::default(),
            &HelpViewModel::default(),
            &SymbolRegistry::default(),
            &ThemeRegistry::default(),
            &commands::CommandBindings::default(),
            mode,
            build,
            0,
            0,
        )
    }

    #[test]
    fn visible_changes_and_resize_change_the_render_fingerprint() {
        let map = MapViewModel::default();
        let log = LogViewModel::default();
        let container = ContainerViewModel::default();
        let build = BuildInteraction::default();
        let baseline = test_visible_fingerprint(
            80,
            24,
            "outpost",
            GameMode::Outpost,
            &map,
            &log,
            &container,
            &build,
        );

        let mut moved_map = map.clone();
        moved_map.player_pos = Some(Position { x: 2, y: 1 });
        assert_ne!(
            test_visible_fingerprint(
                80,
                24,
                "outpost",
                GameMode::Outpost,
                &moved_map,
                &log,
                &container,
                &build,
            ),
            baseline,
            "movement must invalidate the frame"
        );

        let changed_log = LogViewModel {
            entries: vec![view_models::LogEntryVm {
                message: "Visible result".into(),
                level: LogLevel::Info,
            }],
        };
        assert_ne!(
            test_visible_fingerprint(
                80,
                24,
                "outpost",
                GameMode::Outpost,
                &map,
                &changed_log,
                &container,
                &build,
            ),
            baseline,
            "log changes must invalidate the frame"
        );

        let changed_inventory = ContainerViewModel {
            items: vec![view_models::ItemEntryVm {
                name: "Field Dressing".into(),
                equipped: false,
                usable: true,
            }],
        };
        assert_ne!(
            test_visible_fingerprint(
                80,
                24,
                "outpost",
                GameMode::Outpost,
                &map,
                &log,
                &changed_inventory,
                &build,
            ),
            baseline,
            "inventory changes must invalidate the frame"
        );

        let changed_build = BuildInteraction::Selecting {
            selected_station: StationType::Altar,
        };
        for changed in [
            test_visible_fingerprint(
                80,
                24,
                "inventory",
                GameMode::Outpost,
                &map,
                &log,
                &container,
                &build,
            ),
            test_visible_fingerprint(
                80,
                24,
                "outpost",
                GameMode::Tactical,
                &map,
                &log,
                &container,
                &build,
            ),
            test_visible_fingerprint(
                80,
                24,
                "outpost",
                GameMode::Outpost,
                &map,
                &log,
                &container,
                &changed_build,
            ),
            test_visible_fingerprint(
                60,
                20,
                "outpost",
                GameMode::Outpost,
                &map,
                &log,
                &container,
                &build,
            ),
        ] {
            assert_ne!(changed, baseline);
        }
    }

    #[test]
    fn draw_failure_requests_clean_application_shutdown() {
        let mut invalidation = RenderInvalidation::default();
        let mut exit_request = commands::ApplicationExitRequest::default();
        assert!(invalidation.needs_draw(7));

        complete_draw_attempt(
            &mut invalidation,
            &mut exit_request,
            Err("terminal disconnected".into()),
        );

        assert_eq!(invalidation.error(), Some("terminal disconnected"));
        assert!(exit_request.0);
        assert!(
            invalidation.needs_draw(7),
            "a failed frame must remain dirty until shutdown"
        );
    }

    fn render_screen(
        screen: &str,
        width: u16,
        height: u16,
        mode: GameMode,
        map: &MapViewModel,
        container: &ContainerViewModel,
    ) -> String {
        buffer_text(&render_buffer(screen, width, height, mode, map, container))
    }

    fn render_buffer(
        screen: &str,
        width: u16,
        height: u16,
        mode: GameMode,
        map: &MapViewModel,
        container: &ContainerViewModel,
    ) -> Buffer {
        render_buffer_with_state(
            screen,
            width,
            height,
            mode,
            map,
            container,
            StatsViewModel {
                hp_current: 10,
                hp_max: 10,
                ap_current: 3,
                ap_max: 3,
                supplies: 10,
                day: 1,
                party_names: vec!["Mara".into(), "Iven".into()],
                ..Default::default()
            },
            LogViewModel::default(),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn render_screen_with_state(
        screen: &str,
        width: u16,
        height: u16,
        mode: GameMode,
        map: &MapViewModel,
        container: &ContainerViewModel,
        stats: StatsViewModel,
        log: LogViewModel,
    ) -> String {
        let buffer =
            render_buffer_with_state(screen, width, height, mode, map, container, stats, log);
        buffer_text(&buffer)
    }

    fn buffer_text(buffer: &Buffer) -> String {
        let area = buffer.area;
        (area.y..area.y + area.height)
            .map(|y| {
                (area.x..area.x + area.width)
                    .map(|x| {
                        buffer
                            .cell((x, y))
                            .expect("cell must be inside test buffer")
                            .symbol()
                    })
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn titled_panel_inner(buffer: &Buffer, title: &str) -> Option<Rect> {
        let area = buffer.area;
        let title = title.chars().collect::<Vec<_>>();
        let right = area.x + area.width;
        let bottom = area.y + area.height;

        for y in area.y..bottom {
            for title_x in area.x..right.saturating_sub(title.len() as u16 - 1) {
                let title_matches = title.iter().enumerate().all(|(offset, expected)| {
                    buffer
                        .cell((title_x + offset as u16, y))
                        .is_some_and(|cell| cell.symbol() == expected.to_string())
                });
                if !title_matches {
                    continue;
                }

                let Some(left_x) = (area.x..title_x).rev().find(|x| {
                    buffer
                        .cell((*x, y))
                        .is_some_and(|cell| matches!(cell.symbol(), "┌" | "╔"))
                }) else {
                    continue;
                };
                let Some(right_x) = (title_x + title.len() as u16..right).find(|x| {
                    buffer
                        .cell((*x, y))
                        .is_some_and(|cell| matches!(cell.symbol(), "┐" | "╗"))
                }) else {
                    continue;
                };
                let Some(bottom_y) = (y + 1..bottom).find(|candidate_y| {
                    buffer
                        .cell((left_x, *candidate_y))
                        .is_some_and(|cell| matches!(cell.symbol(), "└" | "╚"))
                        && buffer
                            .cell((right_x, *candidate_y))
                            .is_some_and(|cell| matches!(cell.symbol(), "┘" | "╝"))
                }) else {
                    continue;
                };

                return Some(Rect::new(
                    left_x + 1,
                    y + 1,
                    right_x.saturating_sub(left_x + 1),
                    bottom_y.saturating_sub(y + 1),
                ));
            }
        }
        None
    }

    fn normalized_semantic_text(buffer: &Buffer, area: Rect) -> String {
        let mut text = String::new();
        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                let symbol = buffer
                    .cell((x, y))
                    .expect("semantic region must remain inside the buffer")
                    .symbol();
                if symbol.chars().all(|character| {
                    character.is_whitespace() || ('\u{2500}'..='\u{257f}').contains(&character)
                }) {
                    text.push(' ');
                } else {
                    text.push_str(symbol);
                }
            }
            text.push(' ');
        }
        text.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    #[allow(clippy::too_many_arguments)]
    fn render_buffer_with_state(
        screen: &str,
        width: u16,
        height: u16,
        mode: GameMode,
        map: &MapViewModel,
        container: &ContainerViewModel,
        stats: StatsViewModel,
        log: LogViewModel,
    ) -> Buffer {
        render_buffer_with_state_and_help(
            screen,
            width,
            height,
            mode,
            map,
            container,
            stats,
            log,
            HelpViewModel::default(),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn render_buffer_with_state_and_help(
        screen: &str,
        width: u16,
        height: u16,
        mode: GameMode,
        map: &MapViewModel,
        container: &ContainerViewModel,
        stats: StatsViewModel,
        log: LogViewModel,
        help: HelpViewModel,
    ) -> Buffer {
        let screens = default_screen_registry();
        let widgets = default_widget_registry();
        let action_projections = match mode {
            GameMode::Outpost => commands::action_panel(
                &commands::CommandBindings::default(),
                commands::ActionAvailability::outpost(true, true, true, true, true),
            ),
            GameMode::Tactical => commands::action_panel(
                &commands::CommandBindings::default(),
                commands::ActionAvailability::dungeon(true, true, true, true, true).at_exit(true),
            ),
            _ => Vec::new(),
        };
        let actions = ActionListViewModel {
            actions: action_projections
                .into_iter()
                .map(|action| view_models::ActionItemVm {
                    label: action.label,
                    key_hint: action.key,
                    enabled: action.enabled,
                    denial_reason: action.denial_reason,
                })
                .collect(),
        };
        let event = EventViewModel::default();
        let symbols = SymbolRegistry::phase5_defaults();
        let theme = ThemeRegistry::phase5_defaults();
        let bindings = commands::CommandBindings::default();
        let definition = screens.get(screen).expect("screen fixture must exist");
        let interaction = frame_interaction(
            mode,
            map.build_menu.is_some() || map.build_ghost.is_some(),
            stats.management.as_ref().map(|menu| menu.kind),
        );
        let data = UiFrameData {
            definition,
            widgets: &widgets,
            map,
            stats: &stats,
            log: &log,
            actions: &actions,
            container,
            event: &event,
            help: &help,
            symbols: &symbols,
            theme: &theme,
            bindings: &bindings,
            mode,
            interaction,
            turn: 0,
            day: 1,
        };
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).expect("test terminal must initialize");
        terminal
            .draw(|frame| render_ui_frame(frame, &data))
            .expect("test frame must render");
        terminal.backend().buffer().clone()
    }

    fn shelter_map() -> MapViewModel {
        let width = 24;
        let height = 16;
        MapViewModel {
            width,
            height,
            tiles: vec![Tile::Floor; (width * height) as usize],
            ..Default::default()
        }
    }

    #[test]
    fn colony_worker_recipe_stage_target_and_cargo_are_visible_at_supported_profiles() {
        for (width, height) in [(80, 24), (60, 20)] {
            let output = render_screen_with_state(
                "outpost",
                width,
                height,
                GameMode::Outpost,
                &shelter_map(),
                &ContainerViewModel::default(),
                StatsViewModel {
                    hp_current: 10,
                    hp_max: 10,
                    ap_current: 3,
                    ap_max: 3,
                    party_names: vec![
                        "Mara — Refine Timber: To Station | EnRoute Basic Processing | cargo 1 Raw Timber".into(),
                    ],
                    ..Default::default()
                },
                LogViewModel::default(),
            );
            let normalized = output.split_whitespace().collect::<Vec<_>>().join(" ");
            for required in [
                "Refine",
                "Timber",
                "To",
                "Station",
                "Basic",
                "Processing",
                "cargo",
                "1",
                "Raw",
            ] {
                assert!(
                    normalized.contains(required),
                    "{width}x{height} hides worker detail `{required}`:\n{output}"
                );
            }
        }
    }

    #[test]
    fn direct_gather_progress_raw_stockpile_and_split_forecast_fit_supported_profiles() {
        for (width, height) in [(80, 24), (60, 20)] {
            let output = render_screen_with_state(
                "outpost",
                width,
                height,
                GameMode::Outpost,
                &shelter_map(),
                &ContainerViewModel::default(),
                StatsViewModel {
                    hp_current: 10,
                    hp_max: 10,
                    ap_current: 3,
                    ap_max: 3,
                    party_names: vec!["Mara — Gather Supplies: Water 2/3 | Working Water".into()],
                    station_status: vec!["Raw stockpile — Raw Timber 2".into()],
                    next_day_forecast:
                        "Next worker: Supplies +1 in 1 turn | Next day: upkeep -3 → 5".into(),
                    ..Default::default()
                },
                LogViewModel::default(),
            );
            let words = output
                .split(|character: char| {
                    character.is_whitespace()
                        || matches!(character, '│' | '┌' | '┐' | '└' | '┘' | '─')
                })
                .filter(|word| !word.is_empty())
                .collect::<Vec<_>>();
            for required in [
                "Gather",
                "Supplies:",
                "Water",
                "2/3",
                "Raw",
                "Timber",
                "Next",
                "worker:",
                "day:",
            ] {
                assert!(
                    words.contains(&required),
                    "{width}x{height} hides colony work token `{required}`:\n{output}"
                );
            }
        }
    }

    #[test]
    fn management_information_and_screen_controls_are_complete_at_supported_profiles() {
        for (width, height) in [(80, 24), (60, 20)] {
            let stats = StatsViewModel {
                hp_current: 10,
                hp_max: 10,
                ap_current: 3,
                ap_max: 3,
                supplies: 4,
                party_names: vec![
                    "Mara — Idle (Mood 50)".into(),
                    "Ivo — Gather Supplies (Mood 45)".into(),
                    "Sela — Rest (Mood 40)".into(),
                ],
                management: Some(view_models::ManagementMenuVm {
                    kind: view_models::ManagementMenuKind::TaskAssignment,
                    survivors: vec![
                        "Mara — Idle (Mood 50)".into(),
                        "Ivo — Gather Supplies: Water 2/3 (Mood 45)".into(),
                        "Sela — Rest (Mood 40)".into(),
                    ],
                    tasks: vec![
                        "1. Idle".into(),
                        "2. Gather Supplies".into(),
                        "3. Gather Materials".into(),
                        "4. Gather Plants".into(),
                        "5. Rest".into(),
                        "6. Altar — +2 Faith/day when staffed — Unstaffed".into(),
                        "Defend — unavailable: no Foundation effect".into(),
                    ],
                    selected_survivor: Some(1),
                    selected_task: Some(1),
                    resources: "Sup 10  Mat 0  Plant 0  Faith 2".into(),
                    forecast: "Next worker: Supplies +1 in 1 turn | Next day: upkeep -3 → 3".into(),
                }),
                ..Default::default()
            };
            let buffer = render_buffer_with_state(
                "outpost",
                width,
                height,
                GameMode::Outpost,
                &shelter_map(),
                &ContainerViewModel::default(),
                stats,
                LogViewModel::default(),
            );
            let output = buffer_text(&buffer);
            let modal = titled_panel_inner(&buffer, "Task Management")
                .expect("Task Management modal must own one semantic region");
            let modal_text = normalized_semantic_text(&buffer, modal);
            for required in [
                "Mara",
                "Ivo",
                "Sela",
                "Gather Supplies",
                "Faith/day",
                "Water 2/3",
                "Next worker:",
                "Next day:",
                "upkeep",
            ] {
                assert!(
                    modal_text.contains(required),
                    "{width}x{height} management modal hid `{required}`; \
                     modal_semantic_text={modal_text:?}:\n{output}"
                );
            }
            let footer = output
                .lines()
                .skip(height.saturating_sub(3) as usize)
                .collect::<Vec<_>>()
                .join(" ");
            for required in ["[Enter] confirm", "[c/Esc] cancel"] {
                assert!(
                    footer.contains(required),
                    "{width}x{height} management screen hid `{required}`; footer={footer:?}"
                );
            }
        }
    }

    #[test]
    fn title_and_game_over_fit_both_supported_profiles() {
        for (width, height) in [(80, 24), (60, 20)] {
            let title = render_screen(
                "title",
                width,
                height,
                GameMode::Title,
                &MapViewModel::default(),
                &ContainerViewModel::default(),
            );
            for required in [
                "BROKEN DIVINITY",
                "FOUNDATION BUILD",
                "Press any key to begin",
                "[F9] Load",
                "[q] Quit",
            ] {
                assert!(
                    title.contains(required),
                    "{width}x{height} title hid `{required}`:\n{title}"
                );
            }
            assert!(!title.contains("[wasd/arrows] Move"));

            let game_over = render_screen_with_state(
                "game_over",
                width,
                height,
                GameMode::GameOver,
                &MapViewModel::default(),
                &ContainerViewModel::default(),
                StatsViewModel {
                    extracted_loot: 2,
                    ..Default::default()
                },
                LogViewModel::default(),
            );
            for required in [
                "You have died.",
                "[r] Restart",
                "[F5] Save",
                "[F9] Load",
                "[q] Quit",
            ] {
                assert!(
                    game_over.contains(required),
                    "{width}x{height} Game Over hid `{required}`:\n{game_over}"
                );
            }
        }
    }

    #[test]
    fn title_wordmark_is_complete_and_centered_at_both_supported_profiles() {
        const WORDMARK: &str = "BROKEN DIVINITY";

        for (width, height) in [(80, 24), (60, 20)] {
            let output = render_screen(
                "title",
                width,
                height,
                GameMode::Title,
                &MapViewModel::default(),
                &ContainerViewModel::default(),
            );
            let matching_rows = output
                .lines()
                .filter(|line| line.contains(WORDMARK))
                .collect::<Vec<_>>();

            assert_eq!(
                matching_rows.len(),
                1,
                "{width}x{height} must contain one complete wordmark:\n{output}"
            );
            let title_row = matching_rows[0];
            assert_eq!(
                title_row.trim(),
                WORDMARK,
                "{width}x{height} wordmark row contains malformed title fragments"
            );
            assert_eq!(
                title_row.find(WORDMARK),
                Some((width as usize - WORDMARK.len()).div_ceil(2)),
                "{width}x{height} wordmark is not horizontally centered"
            );
        }
    }

    #[test]
    fn title_wordmark_has_distinct_accent_style_at_both_supported_profiles() {
        const WORDMARK: &str = "BROKEN DIVINITY";

        for (width, height) in [(80, 24), (60, 20)] {
            let output = render_screen(
                "title",
                width,
                height,
                GameMode::Title,
                &MapViewModel::default(),
                &ContainerViewModel::default(),
            );
            let y = output
                .lines()
                .position(|line| line.contains(WORDMARK))
                .expect("the complete wordmark must be rendered") as u16;
            let x = (width as usize - WORDMARK.len()).div_ceil(2) as u16;
            let buffer = render_buffer_with_state(
                "title",
                width,
                height,
                GameMode::Title,
                &MapViewModel::default(),
                &ContainerViewModel::default(),
                StatsViewModel::default(),
                LogViewModel::default(),
            );
            let first_title_cell = buffer
                .cell((x, y))
                .expect("wordmark coordinate must be inside the terminal");

            assert_eq!(first_title_cell.fg, ratatui::style::Color::Cyan);
            assert!(
                first_title_cell
                    .modifier
                    .contains(ratatui::style::Modifier::BOLD),
                "{width}x{height} wordmark must retain the title emphasis"
            );
        }
    }

    #[test]
    fn title_displays_persistence_failures_at_both_supported_profiles() {
        for (width, height) in [(80, 24), (60, 20)] {
            let output = render_screen_with_state(
                "title",
                width,
                height,
                GameMode::Title,
                &MapViewModel::default(),
                &ContainerViewModel::default(),
                StatsViewModel::default(),
                LogViewModel {
                    entries: vec![view_models::LogEntryVm {
                        message: "Load failed: manual save does not exist.".into(),
                        level: bd_core::gamelog::LogLevel::Warn,
                    }],
                },
            );

            assert!(
                output.contains("Load failed: manual save does not exist."),
                "{width}x{height} title hid the load failure:\n{output}"
            );
        }
    }

    #[test]
    fn combat_and_inventory_fit_both_supported_profiles() {
        let inventory = ContainerViewModel {
            items: vec![view_models::ItemEntryVm {
                name: "Field Dressing".into(),
                equipped: false,
                usable: true,
            }],
        };
        for (width, height) in [(80, 24), (60, 20)] {
            let combat = render_screen(
                "combat",
                width,
                height,
                GameMode::Tactical,
                &shelter_map(),
                &ContainerViewModel::default(),
            );
            let compact_combat = combat.split_whitespace().collect::<String>();
            for required in [
                "Map", "Stats", "HP10/10[", "AP3/3[", "Attack", "Extract", "[F5]Save", "[F9]Load",
                "[q]Quit",
            ] {
                assert!(
                    compact_combat.contains(required),
                    "{width}x{height} combat hid `{required}`:\n{combat}"
                );
            }

            let inventory_output = render_screen(
                "inventory",
                width,
                height,
                GameMode::Outpost,
                &shelter_map(),
                &inventory,
            );
            for required in [
                "Field Dressing",
                "usable",
                "[u] Use",
                "[i] Back",
                "[F5] Save",
                "[F9] Load",
                "[q] Quit",
            ] {
                assert!(
                    inventory_output.contains(required),
                    "{width}x{height} inventory hid `{required}`:\n{inventory_output}"
                );
            }
        }
    }

    #[test]
    fn build_selection_and_placement_fit_both_supported_profiles() {
        for (width, height) in [(80, 24), (60, 20)] {
            let mut selection = shelter_map();
            let mut options = default_station_blueprints()
                .into_iter()
                .map(|blueprint| {
                    (
                        blueprint.label.to_string(),
                        blueprint.build_cost_supplies,
                        blueprint.effect_label(),
                    )
                })
                .collect::<Vec<_>>();
            options.push((
                "Basic Processing".into(),
                2,
                "Refines assigned colony recipes".into(),
            ));
            selection.build_menu = Some(view_models::BuildMenuVm {
                options,
                selected: 0,
                available_supplies: 10,
            });
            let selection_output = render_screen(
                "outpost",
                width,
                height,
                GameMode::Outpost,
                &selection,
                &ContainerViewModel::default(),
            );
            for required in [
                "Available: 10 Supplies",
                "Stove",
                "Altar",
                "Workshop",
                "Bed",
                "Storage",
                "Basic Processing",
                "1-6:highlight",
                "Enter:placement",
                "b/Esc:cancel",
            ] {
                assert!(
                    selection_output.contains(required),
                    "{width}x{height} build menu hid `{required}`:\n{selection_output}"
                );
            }

            let mut placement = shelter_map();
            placement.build_ghost = Some((Position { x: 2, y: 2 }, 'f'));
            let placement_output = render_screen(
                "outpost",
                width,
                height,
                GameMode::Outpost,
                &placement,
                &ContainerViewModel::default(),
            );
            for required in [
                "Build Placement",
                "Tile: wasd/arrows",
                "Enter:build",
                "b/Esc:cancel",
            ] {
                assert!(
                    placement_output.contains(required),
                    "{width}x{height} build placement hid `{required}`:\n{placement_output}"
                );
            }
        }
    }

    #[test]
    fn long_feedback_is_deliberately_truncated_and_recent_logs_stay_causal() {
        let log = LogViewModel {
            entries: vec![
                view_models::LogEntryVm {
                    message: "obsolete message one".into(),
                    level: LogLevel::Info,
                },
                view_models::LogEntryVm {
                    message: "obsolete message two".into(),
                    level: LogLevel::Info,
                },
                view_models::LogEntryVm {
                    message: "Player attacks.".into(),
                    level: LogLevel::Combat,
                },
                view_models::LogEntryVm {
                    message: "Rat takes 3 damage.".into(),
                    level: LogLevel::Combat,
                },
                view_models::LogEntryVm {
                    message: "Game saved to /tmp/a/very/long/profile/path/that/cannot/fit/manual-slot.ron."
                        .into(),
                    level: LogLevel::Info,
                },
            ],
        };
        let output = render_screen_with_state(
            "combat",
            60,
            20,
            GameMode::Tactical,
            &shelter_map(),
            &ContainerViewModel::default(),
            StatsViewModel {
                hp_current: 10,
                hp_max: 10,
                ap_current: 3,
                ap_max: 3,
                ..Default::default()
            },
            log,
        );

        assert!(
            output.contains('…'),
            "long feedback needs an ellipsis:\n{output}"
        );
        assert!(
            output.contains("manual-slot.ron"),
            "path truncation must preserve the useful filename:\n{output}"
        );
        let attack = output
            .find("Player attacks.")
            .expect("attack must be visible");
        let damage = output
            .find("Rat takes 3 damage.")
            .expect("damage result must be visible");
        assert!(
            attack < damage,
            "cause must render before result:\n{output}"
        );
        assert!(!output.contains("obsolete message one"));
    }

    #[test]
    fn supported_panel_screens_stay_inside_content_and_out_of_footer() {
        for (screen, mode) in [
            ("outpost", GameMode::Outpost),
            ("combat", GameMode::Tactical),
            ("inventory", GameMode::Outpost),
        ] {
            for (width, height) in [(80_u16, 24_u16), (60, 20)] {
                let output = render_screen(
                    screen,
                    width,
                    height,
                    mode,
                    &shelter_map(),
                    &ContainerViewModel::default(),
                );
                let lines = output.lines().collect::<Vec<_>>();
                assert_eq!(lines.len(), height as usize);
                assert!(
                    lines
                        .iter()
                        .all(|line| line.chars().count() == width as usize),
                    "{screen} {width}x{height} wrote malformed rows:\n{output}"
                );
                for footer_line in &lines[height as usize - 3..] {
                    assert!(
                        !footer_line.chars().any(|character| {
                            matches!(character, '│' | '┌' | '┐' | '└' | '┘')
                        }),
                        "{screen} panel wrote into footer at {width}x{height}:\n{output}"
                    );
                }
                assert!(
                    lines[height as usize - 4]
                        .chars()
                        .any(|character| matches!(character, '└' | '┘')),
                    "{screen} borders did not close before footer at {width}x{height}:\n{output}"
                );
            }
        }
    }

    #[test]
    fn outpost_80x24_viewport_keeps_player_visible_at_far_shelter_edge() {
        let mut map = MapViewModel {
            width: 40,
            height: 30,
            tiles: vec![Tile::Floor; 40 * 30],
            player_pos: Some(Position { x: 38, y: 28 }),
            ..Default::default()
        };
        map.visuals.extend([
            view_models::MapVisualVm {
                position: Position { x: 38, y: 28 },
                token: VisualToken::Player,
                glyph: None,
            },
            view_models::MapVisualVm {
                position: Position { x: 39, y: 28 },
                token: VisualToken::Exit,
                glyph: None,
            },
        ]);

        let output = render_screen(
            "outpost",
            80,
            24,
            GameMode::Outpost,
            &map,
            &ContainerViewModel::default(),
        );

        assert_eq!(
            output.matches('@').count(),
            1,
            "the player disappeared when reaching the far shelter edge:\n{output}"
        );
    }

    #[test]
    fn outpost_60x20_viewport_keeps_player_visible_at_far_shelter_edge() {
        let map = MapViewModel {
            width: 40,
            height: 30,
            tiles: vec![Tile::Floor; 40 * 30],
            player_pos: Some(Position { x: 38, y: 28 }),
            visuals: vec![view_models::MapVisualVm {
                position: Position { x: 38, y: 28 },
                token: VisualToken::Player,
                glyph: None,
            }],
            ..Default::default()
        };

        let output = render_screen(
            "outpost",
            60,
            20,
            GameMode::Outpost,
            &map,
            &ContainerViewModel::default(),
        );

        assert_eq!(
            output.matches('@').count(),
            1,
            "the compact viewport lost the player at the far shelter edge:\n{output}"
        );
    }

    #[test]
    fn distant_build_preview_drives_the_viewport_at_both_supported_profiles() {
        for (width, height) in [(80, 24), (60, 20)] {
            let map = MapViewModel {
                width: 40,
                height: 30,
                tiles: vec![Tile::Floor; 40 * 30],
                player_pos: Some(Position { x: 1, y: 1 }),
                visuals: vec![view_models::MapVisualVm {
                    position: Position { x: 1, y: 1 },
                    token: VisualToken::Player,
                    glyph: None,
                }],
                build_ghost: Some((Position { x: 38, y: 28 }, 'X')),
                build_placement: Some(view_models::BuildPlacementVm {
                    label: "Workshop".into(),
                    supply_cost: 2,
                    effect: "+2 Materials/day when staffed".into(),
                }),
                ..Default::default()
            };

            let output = render_screen(
                "outpost",
                width,
                height,
                GameMode::Outpost,
                &map,
                &ContainerViewModel::default(),
            );

            assert_eq!(
                output.matches('X').count(),
                1,
                "contract=VISUAL-BUILD-004 case={width}x{height} \
                 fixture=colony_build_distant_valid expected the distant build \
                 preview to remain visible while placement owns viewport focus:\n{output}"
            );
            for required in ["Workshop", "2 Supplies", "+2 Materials/day"] {
                assert!(
                    output.contains(required),
                    "contract=VISUAL-BUILD-004 case={width}x{height} \
                     missing selected detail `{required}`:\n{output}"
                );
            }
        }
    }

    #[test]
    fn compact_viewport_projects_resource_next_to_far_edge_player() {
        let map = MapViewModel {
            width: 40,
            height: 30,
            tiles: vec![Tile::Floor; 40 * 30],
            player_pos: Some(Position { x: 38, y: 28 }),
            visuals: vec![
                view_models::MapVisualVm {
                    position: Position { x: 38, y: 28 },
                    token: VisualToken::Player,
                    glyph: None,
                },
                view_models::MapVisualVm {
                    position: Position { x: 39, y: 28 },
                    token: VisualToken::ResourceNode,
                    glyph: Some('Ω'),
                },
            ],
            ..Default::default()
        };

        let output = render_screen(
            "outpost",
            60,
            20,
            GameMode::Outpost,
            &map,
            &ContainerViewModel::default(),
        );

        assert!(
            output.contains('Ω'),
            "a resource adjacent to the player was clipped by a different projection:\n{output}"
        );
    }

    #[test]
    fn assigned_offscreen_target_has_a_directional_edge_indicator() {
        let map = MapViewModel {
            width: 40,
            height: 30,
            tiles: vec![Tile::Floor; 40 * 30],
            player_pos: Some(Position { x: 1, y: 1 }),
            assigned_targets: vec![Position { x: 1, y: 29 }],
            ..Default::default()
        };

        for (width, height) in [(80, 24), (60, 20)] {
            let output = render_screen(
                "outpost",
                width,
                height,
                GameMode::Outpost,
                &map,
                &ContainerViewModel::default(),
            );
            assert_eq!(
                output.chars().filter(|character| *character == '↓').count(),
                1,
                "{width}x{height} must keep the assigned southward target discoverable:\n{output}"
            );
        }
    }

    #[test]
    fn dungeon_loot_item_reaches_the_rendered_map_buffer() {
        let map = MapViewModel {
            width: 12,
            height: 8,
            tiles: vec![Tile::Floor; 12 * 8],
            player_pos: Some(Position { x: 1, y: 1 }),
            visuals: vec![
                view_models::MapVisualVm {
                    position: Position { x: 1, y: 1 },
                    token: VisualToken::Player,
                    glyph: None,
                },
                view_models::MapVisualVm {
                    position: Position { x: 1, y: 3 },
                    token: VisualToken::Item,
                    glyph: None,
                },
            ],
            ..Default::default()
        };

        let output = render_screen(
            "combat",
            60,
            20,
            GameMode::Tactical,
            &map,
            &ContainerViewModel::default(),
        );

        assert_eq!(
            output.chars().filter(|character| *character == '!').count(),
            1,
            "one loose dungeon item must resolve to one visible map glyph:\n{output}"
        );
    }

    #[test]
    fn station_and_resource_cells_have_distinct_resolved_styles() {
        let map = MapViewModel {
            width: 24,
            height: 16,
            tiles: vec![Tile::Floor; 24 * 16],
            player_pos: Some(Position { x: 1, y: 1 }),
            visuals: vec![
                view_models::MapVisualVm {
                    position: Position { x: 1, y: 1 },
                    token: VisualToken::Player,
                    glyph: None,
                },
                view_models::MapVisualVm {
                    position: Position { x: 2, y: 2 },
                    token: VisualToken::Station,
                    glyph: Some('Ω'),
                },
                view_models::MapVisualVm {
                    position: Position { x: 3, y: 2 },
                    token: VisualToken::ResourceNode,
                    glyph: Some('Ω'),
                },
            ],
            ..Default::default()
        };
        let buffer = render_buffer_with_state(
            "outpost",
            80,
            24,
            GameMode::Outpost,
            &map,
            &ContainerViewModel::default(),
            StatsViewModel::default(),
            LogViewModel::default(),
        );
        let cells = buffer
            .content()
            .iter()
            .filter(|cell| cell.symbol() == "Ω")
            .collect::<Vec<_>>();

        assert_eq!(cells.len(), 2, "both semantic categories must render");
        assert_ne!(
            (cells[0].fg, cells[0].bg, cells[0].modifier),
            (cells[1].fg, cells[1].bg, cells[1].modifier),
            "stations and resource nodes currently resolve to the same visual style"
        );
    }

    #[test]
    fn compact_build_selection_shows_complete_selected_effect() {
        // Contract: VISUAL-BUILD-001
        // Given: a selected Workshop with a long decisive effect at compact 60x20.
        // When: the final Build Selection buffer is rendered.
        // Then: the titled modal contains the complete selected label, cost, and effect.
        // Must not change: approved chrome may change without becoming semantic text;
        // truncation or moving the decisive detail outside the modal remains forbidden.
        // Evidence layers: projection, buffer layout; input-state and PTY remain required.
        let required_effect =
            "Produces two Supplies each day when a survivor is physically working here";
        let required_selection = "▶ 1. Workshop — 4 Supplies";
        let mut map = shelter_map();
        map.build_menu = Some(view_models::BuildMenuVm {
            options: vec![("Workshop".into(), 4, required_effect.into())],
            selected: 0,
            available_supplies: 10,
        });

        let buffer = render_buffer(
            "outpost",
            60,
            20,
            GameMode::Outpost,
            &map,
            &ContainerViewModel::default(),
        );
        let output = buffer_text(&buffer);
        let modal_inner = titled_panel_inner(&buffer, "Build Station").unwrap_or_else(|| {
            panic!(
                "contract=VISUAL-BUILD-001 case=build-selection-60x20 \
                 precondition=selected_workshop_with_long_effect \
                 action=locate_titled_build_modal_in_final_buffer \
                 expected=one_bordered_Build_Station_modal actual=missing visual=\n{output}"
            )
        });
        assert!(
            titled_panel_inner(&buffer, "Missing Station").is_none(),
            "contract=VISUAL-BUILD-001 case=observer-negative-title \
             precondition=final_build_selection_buffer action=locate_absent_modal_title \
             expected=no_semantic_region actual=observer_false_positive"
        );
        let semantic_text = normalized_semantic_text(&buffer, modal_inner);
        assert!(
            semantic_text.contains(required_selection)
                && semantic_text.contains(&format!("Effect: {required_effect}")),
            "contract=VISUAL-BUILD-001 case=build-selection-60x20 \
             precondition=selected_workshop_with_long_effect \
             action=read_semantic_content_inside_final_build_modal \
             expected_selection={required_selection:?} expected_effect={required_effect:?} \
             must_not_change=complete_selected_label_cost_effect \
             actual_semantic_text={semantic_text:?} modal_inner={modal_inner:?} visual=\n{output}"
        );
    }

    #[test]
    fn invalid_build_preview_explains_egress_rejection() {
        let mut map = shelter_map();
        map.build_ghost = Some((Position { x: 2, y: 2 }, 'X'));
        map.build_ghost_denial = Some("Would block shelter egress".into());

        let output = render_screen(
            "outpost",
            80,
            24,
            GameMode::Outpost,
            &map,
            &ContainerViewModel::default(),
        );

        assert!(
            output.contains("Would block shelter egress"),
            "an invalid placement has no player-facing rejection reason:\n{output}"
        );
    }

    #[test]
    fn station_staffing_uses_a_distinct_modal_title() {
        let stats = StatsViewModel {
            management: Some(view_models::ManagementMenuVm {
                kind: view_models::ManagementMenuKind::StationStaffing,
                survivors: vec!["Mara — Idle".into()],
                tasks: vec!["1. Stove — Unstaffed".into()],
                selected_survivor: Some(0),
                selected_task: Some(0),
                resources: "Sup 10".into(),
                forecast: "Next Sup: 7".into(),
            }),
            ..Default::default()
        };
        let output = render_screen_with_state(
            "outpost",
            80,
            24,
            GameMode::Outpost,
            &shelter_map(),
            &ContainerViewModel::default(),
            stats,
            LogViewModel::default(),
        );

        assert!(
            output.contains("Station Staffing"),
            "station staffing is visually indistinguishable from task management:\n{output}"
        );
        assert!(
            output.contains("e/Esc:cancel"),
            "station staffing advertises a cancel key that the reducer does not own:\n{output}"
        );
        assert!(
            !output.contains("c/Esc:cancel"),
            "task-management cancellation leaked into station staffing:\n{output}"
        );
    }

    #[test]
    fn compact_station_staffing_keeps_each_wrapped_station_status_inside_the_modal() {
        let stats = StatsViewModel {
            management: Some(view_models::ManagementMenuVm {
                kind: view_models::ManagementMenuKind::StationStaffing,
                survivors: vec!["Mara — Idle".into()],
                tasks: vec![
                    "1. Basic Processing — Refines assigned colony recipes — Unstaffed".into(),
                    "2. Basic Processing — Refines assigned colony recipes — Unstaffed".into(),
                ],
                selected_survivor: Some(0),
                selected_task: None,
                resources: "Sup 8  Mat 0  Plant 0  Faith 0".into(),
                forecast: "Next Sup: -3food +0stn +0gath=-3→5 M+0 P+0 F+0".into(),
            }),
            ..Default::default()
        };
        let output = render_screen_with_state(
            "outpost",
            60,
            20,
            GameMode::Outpost,
            &shelter_map(),
            &ContainerViewModel::default(),
            stats,
            LogViewModel::default(),
        );

        assert_eq!(
            output.matches("Unstaffed").count(),
            2,
            "compact staffing clipped a wrapped station status:\n{output}"
        );
        assert!(output.contains("e/Esc:cancel"), "{output}");
    }

    #[test]
    fn management_modal_and_footer_controls_agree_at_supported_profiles() {
        // Contract: VISUAL-MGMT-002
        // Given: either paused management modal at a supported terminal profile.
        // When: the complete screen buffer is rendered.
        // Then: modal and footer advertise confirm/cancel as one active state.
        // Must not change: unavailable normal-gameplay controls remain hidden.
        // Evidence layers: projection, buffer layout, input state machine, PTY.
        for (width, height) in [(80, 24), (60, 20)] {
            for (kind, cancel_key) in [
                (view_models::ManagementMenuKind::TaskAssignment, "c"),
                (view_models::ManagementMenuKind::StationStaffing, "e"),
            ] {
                let stats = StatsViewModel {
                    management: Some(view_models::ManagementMenuVm {
                        kind,
                        survivors: vec!["Mara — Idle".into()],
                        tasks: vec!["1. Gather Supplies".into()],
                        selected_survivor: Some(0),
                        selected_task: Some(0),
                        resources: "Sup 0".into(),
                        forecast: "Next worker: Supplies +1 in 3 turns".into(),
                    }),
                    ..Default::default()
                };
                let output = render_screen_with_state(
                    "outpost",
                    width,
                    height,
                    GameMode::Outpost,
                    &shelter_map(),
                    &ContainerViewModel::default(),
                    stats,
                    LogViewModel::default(),
                );
                let lines = output.lines().collect::<Vec<_>>();
                let footer = lines[lines.len().saturating_sub(3)..].join(" ");
                for required in [
                    "[Enter] confirm".to_owned(),
                    format!("[{cancel_key}/Esc] cancel"),
                ] {
                    assert!(
                        footer.contains(&required),
                        "contract=VISUAL-MGMT-002 case={kind:?}-{width}x{height} \
                         footer omits modal control `{required}`; footer={footer:?}\n{output}"
                    );
                }
                for forbidden in ["[t] Travel", "[b] Build", "[wasd/arrows] Move"] {
                    assert!(
                        !footer.contains(forbidden),
                        "contract=VISUAL-MGMT-002 case={kind:?}-{width}x{height} \
                         footer advertises unavailable gameplay control `{forbidden}`; \
                         footer={footer:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn blocked_worker_target_and_reason_fit_supported_profiles() {
        for (width, height) in [(80, 24), (60, 20)] {
            let output = render_screen_with_state(
                "outpost",
                width,
                height,
                GameMode::Outpost,
                &shelter_map(),
                &ContainerViewModel::default(),
                StatsViewModel {
                    party_names: vec![
                        "Mara — Gather Supplies | Blocked Water: No route | 0/3 → 1 Supplies"
                            .into(),
                    ],
                    ..Default::default()
                },
                LogViewModel::default(),
            );
            let semantic_text = output
                .chars()
                .map(|character| {
                    if matches!(character, '│' | '┌' | '┐' | '└' | '┘' | '─') {
                        ' '
                    } else {
                        character
                    }
                })
                .collect::<String>()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            for required in [
                "Mara", "Gather", "Supplies", "Blocked", "Water:", "No", "route", "0/3",
            ] {
                assert!(
                    semantic_text.contains(required),
                    "contract=VISUAL-COLONY-WORK-005 case={width}x{height} \
                     missing blocked-worker detail `{required}`:\n{output}"
                );
            }
        }
    }

    #[test]
    fn day_summary_keeps_authoritative_deltas_visible_at_supported_profiles() {
        // Contract: VISUAL-COLONY-STATE-002
        // Given: one completed-day result containing every Foundation delta.
        // When: the Outpost buffer renders at either supported profile.
        // Then: Supplies, Materials, Plants, Faith, and Food remain visible.
        // Must not change: truncation cannot remove a decisive resource result.
        // Evidence layers: projection, buffer layout, PTY.
        let summary = [
            "Day 2:",
            "Supplies 5→2 (-3);",
            "Materials 4→5 (+1);",
            "Plants 2→2 (+0);",
            "Faith 0→0 (+0);",
            "Food -3.",
        ];
        for (width, height) in [(80, 24), (60, 20)] {
            let output = render_screen_with_state(
                "outpost",
                width,
                height,
                GameMode::Outpost,
                &shelter_map(),
                &ContainerViewModel::default(),
                StatsViewModel {
                    latest_daily_summary: summary.iter().map(|line| (*line).into()).collect(),
                    ..Default::default()
                },
                LogViewModel::default(),
            );
            let semantic_text = output.split_whitespace().collect::<Vec<_>>().join(" ");
            for required in [
                "Day 2:",
                "Supplies",
                "5→2",
                "(-3);",
                "Materials",
                "4→5",
                "(+1);",
                "Plants",
                "2→2",
                "(+0);",
                "Faith",
                "0→0",
                "Food -3.",
            ] {
                assert!(
                    semantic_text.contains(required),
                    "contract=VISUAL-COLONY-STATE-002 case={width}x{height} \
                     missing daily delta `{required}`:\n{output}"
                );
            }
        }
    }

    #[test]
    fn outpost_help_explains_visible_resource_glyphs() {
        let entries = commands::help_entries_with_legend(
            &commands::CommandBindings::default(),
            GameMode::Outpost,
            commands::InteractionMode::Normal,
            &SymbolRegistry::phase5_defaults(),
            &bd_core::colony::stations::StationCatalog::new(default_station_blueprints()),
        );
        let help = entries
            .iter()
            .map(|entry| entry.description.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        for required in ["Trees", "Water Source", "Wild Plants"] {
            assert!(
                help.contains(required),
                "Outpost Help does not explain the visible `{required}` resource glyph"
            );
        }
    }

    #[test]
    fn rendered_outpost_help_contains_every_foundation_legend_at_supported_profiles() {
        let mut station_blueprints = default_station_blueprints();
        let mut processor = station_blueprints[0].clone();
        processor.id = "station.basic_processor".into();
        processor.station_type = bd_core::colony::stations::StationType::Custom(1);
        processor.label = "Basic Processing".into();
        processor.glyph = 'p';
        processor.staffed_glyph = 'Q';
        station_blueprints.push(processor);
        let help = HelpViewModel {
            keys: commands::help_entries_with_legend(
                &commands::CommandBindings::default(),
                GameMode::Outpost,
                commands::InteractionMode::Normal,
                &SymbolRegistry::phase5_defaults(),
                &bd_core::colony::stations::StationCatalog::new(station_blueprints),
            )
            .into_iter()
            .map(|entry| (entry.key, entry.description))
            .collect(),
        };

        for (width, height) in [(80, 24), (60, 20)] {
            let buffer = render_buffer_with_state_and_help(
                "help",
                width,
                height,
                GameMode::Outpost,
                &shelter_map(),
                &ContainerViewModel::default(),
                StatsViewModel::default(),
                LogViewModel::default(),
                help.clone(),
            );
            let output = (0..height)
                .map(|y| {
                    (0..width)
                        .map(|x| {
                            buffer
                                .cell((x, y))
                                .expect("cell must be inside Help buffer")
                                .symbol()
                        })
                        .collect::<String>()
                })
                .collect::<Vec<_>>()
                .join("\n");

            for required in ["Trees", "Water Source", "Wild Plants"] {
                assert!(
                    output.contains(required),
                    "{width}x{height} rendered Help clips `{required}` even though it exists in the semantic Help model:\n{output}"
                );
            }
            assert!(
                !output.contains('…'),
                "{width}x{height} Help truncates required controls or legend text:\n{output}"
            );
            for (_, description) in &help.keys {
                assert!(
                    output.contains(description),
                    "{width}x{height} Help omits the complete entry `{description}`:\n{output}"
                );
            }
        }
    }

    #[test]
    fn build_placement_exposes_selected_station_name_cost_and_effect() {
        let mut map = shelter_map();
        map.build_ghost = Some((Position { x: 2, y: 1 }, 'f'));
        let selected = default_station_blueprints()
            .into_iter()
            .find(|blueprint| {
                blueprint.station_type == bd_core::colony::stations::StationType::Stove
            })
            .expect("Foundation fixture must define Stove");
        map.build_placement = Some(view_models::BuildPlacementVm {
            label: selected.label.clone(),
            supply_cost: selected.build_cost_supplies,
            effect: selected.effect_label(),
        });
        let required_details = [
            selected.label.clone(),
            format!("{} Supplies", selected.build_cost_supplies),
            selected.effect_label(),
        ];

        for (width, height) in [(80, 24), (60, 20)] {
            let output = render_screen(
                "outpost",
                width,
                height,
                GameMode::Outpost,
                &map,
                &ContainerViewModel::default(),
            );
            for required in &required_details {
                assert!(
                    output.contains(required.as_str()),
                    "{width}x{height} placement preview omits selected-station detail `{required}`:\n{output}"
                );
            }
        }
    }
}
