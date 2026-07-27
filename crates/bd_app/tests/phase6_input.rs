use std::collections::HashSet;

use bd_core::{
    colony::{
        production::ColonyResources,
        stations::{BuildInteraction, Station, StationType},
        survivors::{Survivor, SurvivorTask},
    },
    components::{Name, Player, Position, ResourceNode, ResourceNodeType, Tile},
    direction::Direction,
    map::SmokeMap,
    session::RunSession,
    signals::{ActionIntent, PoolKind},
    spatial::{FOUNDATION_DUNGEON_ID, GameMode, OutpostState, TransitionIntent},
};
use bd_test_support::foundation_content;
use bevy_app::App;
use bevy_ecs::{entity::Entity, message::Messages, query::With};
use bevy_ratatui::{
    crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    event::KeyMessage,
};

fn outpost_runtime() -> App {
    let mut app = App::new();
    app.add_plugins(bd_core::BdFoundationPlugin);
    let content = foundation_content();
    app.insert_resource(bd_core::colony::stations::StationCatalog::new(
        content.stations.clone(),
    ));
    app.insert_resource(content);
    app.add_plugins(bd_tui::BdTuiPlugin);
    app.world_mut()
        .resource_mut::<Messages<TransitionIntent>>()
        .write(TransitionIntent {
            target: GameMode::Outpost,
            node_id: None,
        });
    app.update();
    app.update();
    app
}

fn survivor_tasks(app: &mut App) -> Vec<bd_core::colony::survivors::SurvivorTask> {
    let mut survivors = app
        .world_mut()
        .query_filtered::<
            (
                &Name,
                &bd_core::colony::survivors::SurvivorTask,
            ),
            bevy_ecs::query::With<bd_core::colony::survivors::Survivor>,
        >()
        .iter(app.world())
        .map(|(name, task)| (name.0.clone(), task.clone()))
        .collect::<Vec<_>>();
    survivors.sort_by(|left, right| left.0.cmp(&right.0));
    survivors.into_iter().map(|(_, task)| task).collect()
}

fn player(app: &mut App) -> Entity {
    let mut query = app
        .world_mut()
        .query_filtered::<Entity, bevy_ecs::query::With<Player>>();
    query
        .iter(app.world())
        .next()
        .expect("Foundation player must exist")
}

fn named_survivor(app: &mut App, expected_name: &str) -> Entity {
    let mut query = app
        .world_mut()
        .query_filtered::<(Entity, &Name), bevy_ecs::query::With<Survivor>>();
    query
        .iter(app.world())
        .find_map(|(entity, name)| (name.0 == expected_name).then_some(entity))
        .unwrap_or_else(|| panic!("Foundation survivor `{expected_name}` must exist"))
}

fn named_survivor_task(app: &mut App, expected_name: &str) -> SurvivorTask {
    let survivor = named_survivor(app, expected_name);
    app.world()
        .get::<SurvivorTask>(survivor)
        .cloned()
        .unwrap_or_else(|| panic!("Foundation survivor `{expected_name}` must have a task"))
}

fn named_survivor_menu_key(app: &App, expected_name: &str) -> KeyCode {
    let menu = app
        .world()
        .resource::<bd_tui::view_models::StatsViewModel>()
        .management
        .as_ref()
        .expect("management view model must be projected");
    let index = menu
        .survivors
        .iter()
        .position(|entry| entry.starts_with(expected_name))
        .unwrap_or_else(|| panic!("management must list survivor `{expected_name}`"));
    KeyCode::Char(char::from(
        b'1' + u8::try_from(index).expect("menu index must fit"),
    ))
}

fn management_choice_key(app: &App, expected_label: &str) -> KeyCode {
    let menu = app
        .world()
        .resource::<bd_tui::view_models::StatsViewModel>()
        .management
        .as_ref()
        .expect("management view model must be projected");
    let index = menu
        .tasks
        .iter()
        .position(|entry| entry.contains(expected_label))
        .unwrap_or_else(|| panic!("management must list choice `{expected_label}`"));
    KeyCode::Char(char::from(
        b'1' + u8::try_from(index).expect("menu index must fit"),
    ))
}

fn build_station(app: &mut App, station_type: StationType, direction: Direction) -> Entity {
    let actor = player(app);
    let player_position = *app
        .world()
        .get::<Position>(actor)
        .expect("player position must exist");
    let (dx, dy) = direction.delta();
    *app.world_mut().resource_mut::<BuildInteraction>() = BuildInteraction::AwaitingResolution {
        selected_station: station_type,
        cursor: Position {
            x: player_position.x + dx,
            y: player_position.y + dy,
        },
    };
    app.world_mut()
        .resource_mut::<Messages<ActionIntent>>()
        .write(ActionIntent {
            actor,
            action_id: "ability.build".into(),
            direction: Some(direction),
            target: None,
        });
    app.update();
    app.update();

    let mut stations = app
        .world_mut()
        .query_filtered::<(Entity, &StationType), bevy_ecs::query::With<Station>>();
    let station = stations
        .iter(app.world())
        .find_map(|(entity, actual)| (*actual == station_type).then_some(entity))
        .unwrap_or_else(|| panic!("{station_type:?} must be built through the production action"));
    app.world_mut()
        .entity_mut(station)
        .remove::<bd_core::colony::stations::ConstructionSite>();
    station
}

fn colony_supplies(app: &App) -> i32 {
    app.world()
        .resource::<ColonyResources>()
        .pools
        .get(PoolKind::Supplies)
        .expect("colony Supplies pool must exist")
        .current
}

fn station_count(app: &mut App) -> usize {
    let mut stations = app
        .world_mut()
        .query_filtered::<Entity, bevy_ecs::query::With<Station>>();
    stations.iter(app.world()).count()
}

fn station_map_glyph(app: &App, station_position: Position) -> char {
    let visual = app
        .world()
        .resource::<bd_tui::view_models::MapViewModel>()
        .visuals
        .iter()
        .find_map(|visual| {
            (visual.position == station_position
                && visual.token == bd_tui::visual::VisualToken::Station)
                .then_some(*visual)
        })
        .unwrap_or_else(|| panic!("station at {station_position:?} must be projected"));
    visual.glyph.unwrap_or_else(|| {
        app.world()
            .resource::<bd_tui::visual::SymbolRegistry>()
            .get(visual.token)
            .expect("station symbol must resolve")
            .glyph
    })
}

fn resource_map_glyph(app: &mut App, expected_kind: ResourceNodeType) -> char {
    let mut nodes = app.world_mut().query::<(&Position, &ResourceNode)>();
    let position = nodes
        .iter(app.world())
        .find_map(|(position, node)| (node.kind == expected_kind).then_some(*position))
        .unwrap_or_else(|| panic!("{expected_kind:?} resource node must exist"));
    let token = match expected_kind {
        ResourceNodeType::Trees => bd_tui::visual::VisualToken::Trees,
        ResourceNodeType::WaterSource => bd_tui::visual::VisualToken::WaterSource,
        ResourceNodeType::WildPlants => bd_tui::visual::VisualToken::WildPlants,
    };
    let visual = app
        .world()
        .resource::<bd_tui::view_models::MapViewModel>()
        .visuals
        .iter()
        .find(|visual| visual.position == position && visual.token == token)
        .copied()
        .unwrap_or_else(|| panic!("{expected_kind:?} resource node must be projected"));
    app.world()
        .resource::<bd_tui::visual::SymbolRegistry>()
        .get(visual.token)
        .expect("resource symbol must resolve")
        .glyph
}

fn place_named_survivor_at_resource_work_tile(
    app: &mut App,
    survivor_name: &str,
    expected_kind: ResourceNodeType,
) -> Entity {
    let survivor = named_survivor(app, survivor_name);
    let nodes = app
        .world_mut()
        .query::<(&Position, &ResourceNode)>()
        .iter(app.world())
        .map(|(position, node)| (*position, node.kind))
        .collect::<Vec<_>>();
    let target = nodes
        .iter()
        .find_map(|(position, kind)| (*kind == expected_kind).then_some(*position))
        .unwrap_or_else(|| panic!("fixture requires {expected_kind:?}"));
    let mut occupied = nodes
        .iter()
        .map(|(position, _)| *position)
        .collect::<HashSet<_>>();
    let survivor_positions = app
        .world_mut()
        .query_filtered::<(Entity, &Position), With<Survivor>>()
        .iter(app.world())
        .map(|(entity, position)| (entity, *position))
        .collect::<Vec<_>>();
    occupied.extend(
        survivor_positions
            .iter()
            .filter_map(|(entity, position)| (*entity != survivor).then_some(*position)),
    );
    let station_positions = app
        .world_mut()
        .query_filtered::<&Position, With<Station>>()
        .iter(app.world())
        .copied()
        .collect::<Vec<_>>();
    occupied.extend(station_positions);
    let player_position = {
        let player = player(app);
        *app.world()
            .get::<Position>(player)
            .expect("player must have a position")
    };
    occupied.insert(player_position);
    let map = &app.world().resource::<OutpostState>().map;
    let work_position = [
        Position {
            x: target.x,
            y: target.y - 1,
        },
        Position {
            x: target.x,
            y: target.y + 1,
        },
        Position {
            x: target.x - 1,
            y: target.y,
        },
        Position {
            x: target.x + 1,
            y: target.y,
        },
    ]
    .into_iter()
    .find(|candidate| map.is_walkable(candidate.x, candidate.y) && !occupied.contains(candidate))
    .unwrap_or_else(|| panic!("{expected_kind:?} requires a free cardinal work tile"));
    app.world_mut().entity_mut(survivor).insert(work_position);
    survivor
}

fn send_keys(app: &mut App, keys: &[KeyCode]) {
    let mut messages = app.world_mut().resource_mut::<Messages<KeyMessage>>();
    for key in keys {
        messages.write(KeyMessage(KeyEvent::new(*key, KeyModifiers::NONE)));
    }
}

fn send_physical_key(app: &mut App, key: KeyCode) {
    let mut messages = app.world_mut().resource_mut::<Messages<KeyMessage>>();
    messages.write(KeyMessage(KeyEvent::new_with_kind(
        key,
        KeyModifiers::NONE,
        KeyEventKind::Press,
    )));
    messages.write(KeyMessage(KeyEvent::new_with_kind(
        key,
        KeyModifiers::NONE,
        KeyEventKind::Release,
    )));
}

fn action_ids(app: &App) -> Vec<String> {
    app.world()
        .resource::<RunSession>()
        .replay_intents
        .iter()
        .map(|record| record.action_id.clone())
        .collect()
}

#[test]
fn first_outpost_move_key_moves_once_without_opening_or_creating_build_state() {
    let mut app = outpost_runtime();
    let actor = player(&mut app);
    let before_position = *app
        .world()
        .get::<Position>(actor)
        .expect("Foundation player position must exist");
    let stations_before = station_count(&mut app);
    let turn_before = app.world().resource::<bd_core::time::GameTime>().turn;

    send_physical_key(&mut app, KeyCode::Char('d'));
    app.update();
    app.update();

    assert_eq!(
        app.world().get::<Position>(actor),
        Some(&Position {
            x: before_position.x + 1,
            y: before_position.y,
        }),
        "contract=INPUT-MOVE-001 step=first-outpost-key expected=one eastward move"
    );
    assert_eq!(
        station_count(&mut app),
        stations_before,
        "contract=INPUT-MOVE-001 first movement key created a station"
    );
    assert!(
        matches!(
            app.world().resource::<BuildInteraction>(),
            BuildInteraction::Inactive
        ),
        "contract=INPUT-MOVE-001 first movement key opened Build interaction"
    );
    assert_eq!(
        app.world().resource::<bd_core::time::GameTime>().turn,
        turn_before + 1,
        "contract=INPUT-MOVE-001 first movement key must advance exactly one turn"
    );
}

#[test]
fn buffered_semantic_commands_resolve_in_input_order() {
    let mut app = outpost_runtime();
    let before = action_ids(&app).len();

    send_keys(
        &mut app,
        &[KeyCode::Char('.'), KeyCode::Char('n'), KeyCode::Char('.')],
    );
    for _ in 0..8 {
        app.update();
    }

    assert_eq!(
        &action_ids(&app)[before..],
        [
            "ability.wait",
            "ability.rest_until_next_day",
            "ability.wait"
        ]
    );
    let time = app.world().resource::<bd_core::time::GameTime>();
    assert_eq!((time.day, time.turn), (1, 1));
}

#[test]
fn buffered_input_is_bounded_and_reports_one_overflow_warning() {
    let mut app = outpost_runtime();
    let before = action_ids(&app).len();
    send_keys(&mut app, &[KeyCode::Char('.'); 5]);

    for _ in 0..10 {
        app.update();
    }

    let resolved = action_ids(&app)[before..]
        .iter()
        .filter(|action| action.as_str() == "ability.wait")
        .count();
    assert_eq!(resolved, 4, "only the bounded queue capacity may resolve");
    let warnings = app
        .world()
        .resource::<bd_core::gamelog::GameLog>()
        .iter()
        .filter(|entry| entry.message.contains("Input queue full"))
        .count();
    assert_eq!(warnings, 1, "one overflow episode must emit one warning");
}

#[test]
fn lifecycle_controls_are_not_starved_by_buffered_gameplay() {
    let mut app = outpost_runtime();
    send_keys(
        &mut app,
        &[
            KeyCode::Char('.'),
            KeyCode::Char('.'),
            KeyCode::Char('.'),
            KeyCode::Char('.'),
            KeyCode::Char('q'),
        ],
    );

    app.update();

    assert!(
        app.world()
            .resource::<bd_tui::commands::ApplicationExitRequest>()
            .0
    );
}

#[test]
fn management_requires_confirmation_and_cancel_is_atomic() {
    let mut app = outpost_runtime();
    let original = survivor_tasks(&mut app);
    let replay_before = action_ids(&app).len();

    for keys in [
        vec![KeyCode::Char('c')],
        vec![KeyCode::Char('2')],
        vec![KeyCode::Char('2')],
    ] {
        send_keys(&mut app, &keys);
        app.update();
    }
    assert_eq!(survivor_tasks(&mut app), original);
    assert_eq!(action_ids(&app).len(), replay_before);

    send_keys(&mut app, &[KeyCode::Esc]);
    app.update();
    assert_eq!(survivor_tasks(&mut app), original);

    for keys in [
        vec![KeyCode::Char('c')],
        vec![KeyCode::Char('2')],
        vec![KeyCode::Char('2')],
        vec![KeyCode::Enter],
    ] {
        send_keys(&mut app, &keys);
        app.update();
    }
    assert!(matches!(
        survivor_tasks(&mut app)[1],
        bd_core::colony::survivors::SurvivorTask::Gathering(bd_core::signals::PoolKind::Supplies)
    ));
    assert!(matches!(
        survivor_tasks(&mut app)[0],
        bd_core::colony::survivors::SurvivorTask::Idle
    ));
}

#[test]
fn buffered_tactical_actions_wait_for_each_enemy_phase() {
    let mut app = outpost_runtime();
    let actor = player(&mut app);
    app.world_mut()
        .resource_mut::<Messages<ActionIntent>>()
        .write(ActionIntent {
            actor,
            action_id: "ability.enter_foundation_dungeon".into(),
            direction: None,
            target: None,
        });
    for _ in 0..4 {
        app.update();
    }
    assert_eq!(*app.world().resource::<GameMode>(), GameMode::Tactical);
    assert_eq!(
        app.world().resource::<RunSession>().dungeon_id.as_deref(),
        Some(FOUNDATION_DUNGEON_ID)
    );
    let before = action_ids(&app).len();

    send_keys(&mut app, &[KeyCode::Char('.'), KeyCode::Char('.')]);
    for _ in 0..12 {
        app.update();
    }

    assert_eq!(
        action_ids(&app)[before..]
            .iter()
            .filter(|action| action.as_str() == "ability.wait")
            .count(),
        2
    );
}

#[test]
fn fixed_dungeon_loot_is_projected_as_a_visible_item() {
    let mut app = outpost_runtime();
    let actor = player(&mut app);
    app.world_mut()
        .resource_mut::<Messages<ActionIntent>>()
        .write(ActionIntent {
            actor,
            action_id: "ability.enter_foundation_dungeon".into(),
            direction: None,
            target: None,
        });
    for _ in 0..4 {
        app.update();
    }

    let map = app.world().resource::<bd_tui::view_models::MapViewModel>();
    assert!(
        map.visuals.iter().any(|visual| {
            visual.position == Position { x: 1, y: 3 }
                && visual.token == bd_tui::visual::VisualToken::Item
        }),
        "the fixed-dungeon potion must be visible before the player steps onto it"
    );
}

#[test]
fn build_controls_remain_immediate_inside_one_input_batch() {
    let mut app = outpost_runtime();
    let stations_before = station_count(&mut app);
    send_keys(
        &mut app,
        &[
            KeyCode::Char('b'),
            KeyCode::Char('1'),
            KeyCode::Enter,
            KeyCode::Char('d'),
            KeyCode::Enter,
        ],
    );
    for _ in 0..4 {
        app.update();
    }

    let mut stations = app
        .world_mut()
        .query_filtered::<Entity, bevy_ecs::query::With<Station>>();
    assert_eq!(
        stations.iter(app.world()).count(),
        stations_before + 1,
        "the complete build batch must place one selected station; log={:?}",
        app.world()
            .resource::<bd_core::gamelog::GameLog>()
            .iter()
            .map(|entry| entry.message.as_str())
            .collect::<Vec<_>>()
    );
}

#[test]
fn build_interaction_is_a_paused_press_only_state_machine() {
    let mut app = outpost_runtime();
    let turn_before = app.world().resource::<RunSession>().turn;
    let replay_before = action_ids(&app).len();

    send_physical_key(&mut app, KeyCode::Char('.'));
    send_physical_key(&mut app, KeyCode::Char('b'));
    app.update();
    assert!(
        matches!(
            app.world().resource::<BuildInteraction>(),
            BuildInteraction::Selecting { .. }
        ),
        "a physical b press/release pair must leave station selection open"
    );
    assert_eq!(
        app.world().resource::<RunSession>().turn,
        turn_before,
        "opening the build modal must discard queued gameplay and pause time"
    );
    assert_eq!(action_ids(&app).len(), replay_before);

    send_physical_key(&mut app, KeyCode::Char('2'));
    app.update();
    assert!(matches!(
        app.world().resource::<BuildInteraction>(),
        BuildInteraction::Selecting {
            selected_station: StationType::Altar
        }
    ));

    send_physical_key(&mut app, KeyCode::Enter);
    app.update();
    assert!(
        matches!(
            app.world().resource::<BuildInteraction>(),
            BuildInteraction::Placing { .. }
        ),
        "one Enter press changes selection into placement without also building"
    );

    send_physical_key(&mut app, KeyCode::Char('.'));
    app.update();
    assert_eq!(app.world().resource::<RunSession>().turn, turn_before);
    assert_eq!(action_ids(&app).len(), replay_before);

    send_physical_key(&mut app, KeyCode::Esc);
    app.update();
    assert!(matches!(
        app.world().resource::<BuildInteraction>(),
        BuildInteraction::Inactive
    ));
    assert_eq!(app.world().resource::<RunSession>().turn, turn_before);
    assert_eq!(action_ids(&app).len(), replay_before);
}

#[test]
fn build_menu_sixth_number_key_selects_the_sixth_data_driven_station() {
    let mut app = outpost_runtime();
    send_physical_key(&mut app, KeyCode::Char('b'));
    app.update();
    send_physical_key(&mut app, KeyCode::Char('6'));
    app.update();

    assert_eq!(
        *app.world().resource::<BuildInteraction>(),
        BuildInteraction::Selecting {
            selected_station: StationType::Custom(1),
        }
    );
}

#[test]
fn build_workflow_uses_one_authoritative_transaction_resource() {
    use bd_core::colony::stations::BuildInteraction;

    let mut app = outpost_runtime();
    assert!(matches!(
        app.world().resource::<BuildInteraction>(),
        BuildInteraction::Inactive
    ));

    send_physical_key(&mut app, KeyCode::Char('b'));
    app.update();
    assert!(matches!(
        app.world().resource::<BuildInteraction>(),
        BuildInteraction::Selecting { .. }
    ));

    send_physical_key(&mut app, KeyCode::Enter);
    app.update();
    assert!(matches!(
        app.world().resource::<BuildInteraction>(),
        BuildInteraction::Placing { .. }
    ));
}

#[test]
fn c_opens_paused_task_management_with_task_identity() {
    let mut app = outpost_runtime();
    let turn_before = app.world().resource::<RunSession>().turn;
    let replay_before = action_ids(&app).len();

    send_physical_key(&mut app, KeyCode::Char('c'));
    app.update();

    let management = app
        .world()
        .resource::<bd_tui::view_models::StatsViewModel>()
        .management
        .as_ref()
        .expect("c must open a projected management mode");
    assert_eq!(
        management.kind,
        bd_tui::view_models::ManagementMenuKind::TaskAssignment
    );
    assert_eq!(app.world().resource::<RunSession>().turn, turn_before);
    assert_eq!(action_ids(&app).len(), replay_before);
}

#[test]
fn world_restore_closes_transient_management_before_gameplay_resumes() {
    let mut app = outpost_runtime();
    send_physical_key(&mut app, KeyCode::Char('c'));
    app.update();
    assert!(
        app.world()
            .resource::<bd_tui::view_models::StatsViewModel>()
            .management
            .is_some(),
        "fixture must begin with task management open"
    );

    app.world_mut()
        .insert_resource(bd_core::save::WorldJustRestored);
    app.update();
    app.update();
    assert!(
        app.world()
            .resource::<bd_tui::view_models::StatsViewModel>()
            .management
            .is_none(),
        "restoration must close transient management state"
    );

    let turn_before = app.world().resource::<RunSession>().turn;
    send_physical_key(&mut app, KeyCode::Char('.'));
    app.update();
    app.update();
    assert_eq!(
        app.world().resource::<RunSession>().turn,
        turn_before + 1,
        "normal gameplay input must resume after transient state is cleared"
    );
}

#[test]
fn e_opens_paused_station_staffing_with_station_identity() {
    let mut app = outpost_runtime();
    build_station(&mut app, StationType::Stove, Direction::East);
    let turn_before = app.world().resource::<RunSession>().turn;
    let replay_before = action_ids(&app).len();

    send_physical_key(&mut app, KeyCode::Char('e'));
    app.update();

    let management = app
        .world()
        .resource::<bd_tui::view_models::StatsViewModel>()
        .management
        .as_ref()
        .expect("e must open a projected management mode");
    assert_eq!(
        management.kind,
        bd_tui::view_models::ManagementMenuKind::StationStaffing
    );
    assert_eq!(app.world().resource::<RunSession>().turn, turn_before);
    assert_eq!(action_ids(&app).len(), replay_before);
}

#[test]
fn station_staffing_lists_station_assignments_not_gathering_tasks() {
    let mut app = outpost_runtime();
    build_station(&mut app, StationType::Stove, Direction::East);

    send_physical_key(&mut app, KeyCode::Char('e'));
    app.update();

    let choices = &app
        .world()
        .resource::<bd_tui::view_models::StatsViewModel>()
        .management
        .as_ref()
        .expect("station staffing must be projected")
        .tasks;
    assert!(
        choices.iter().any(|choice| choice.contains("Stove")),
        "station staffing must expose the built station; choices={choices:?}"
    );
    assert!(
        choices.iter().all(|choice| !choice.contains("Gather")),
        "station staffing currently exposes task-assignment choices; choices={choices:?}"
    );
}

#[test]
fn task_management_lists_survivor_tasks_not_station_staffing_choices() {
    let mut app = outpost_runtime();
    build_station(&mut app, StationType::Stove, Direction::East);

    send_physical_key(&mut app, KeyCode::Char('c'));
    app.update();

    let choices = &app
        .world()
        .resource::<bd_tui::view_models::StatsViewModel>()
        .management
        .as_ref()
        .expect("task management must be projected")
        .tasks;
    assert!(
        choices
            .iter()
            .any(|choice| choice.contains("Gather Supplies")),
        "task management must expose gathering work; choices={choices:?}"
    );
    assert!(
        choices.iter().all(|choice| !choice.contains("Stove")),
        "task management currently exposes station-staffing choices; choices={choices:?}"
    );
}

#[test]
fn direct_gather_assignment_projects_source_and_three_tick_progress() {
    let mut app = outpost_runtime();
    place_named_survivor_at_resource_work_tile(
        &mut app,
        "Survivor 1",
        ResourceNodeType::WaterSource,
    );

    send_physical_key(&mut app, KeyCode::Char('c'));
    app.update();
    let survivor_key = named_survivor_menu_key(&app, "Survivor 1");
    send_physical_key(&mut app, survivor_key);
    app.update();
    let gather_key = management_choice_key(&app, "Gather Supplies");
    send_physical_key(&mut app, gather_key);
    app.update();
    send_physical_key(&mut app, KeyCode::Enter);
    app.update();
    app.update();

    let assigned = app
        .world()
        .resource::<bd_tui::view_models::StatsViewModel>()
        .party_names
        .iter()
        .find(|entry| entry.starts_with("Survivor 1"))
        .cloned()
        .expect("assigned survivor must remain projected");
    assert!(
        assigned.contains("Gather Supplies")
            && assigned.contains("Water")
            && assigned.contains("0/3"),
        "contract=INPUT-MGMT-006 step=confirmed expected direct-gather source \
         and progress; actual={assigned:?}"
    );

    send_physical_key(&mut app, KeyCode::Char('.'));
    app.update();
    app.update();
    let progressed = app
        .world()
        .resource::<bd_tui::view_models::StatsViewModel>()
        .party_names
        .iter()
        .find(|entry| entry.starts_with("Survivor 1"))
        .cloned()
        .expect("working survivor must remain projected");
    assert!(
        progressed.contains("Gather Supplies")
            && progressed.contains("Water")
            && progressed.contains("1/3"),
        "contract=INPUT-MGMT-006 step=first-work-tick expected=1/3; \
         actual={progressed:?}"
    );
}

#[test]
fn recipe_management_uses_human_resource_labels_not_content_ids() {
    let mut app = outpost_runtime();

    send_physical_key(&mut app, KeyCode::Char('e'));
    app.update();
    let survivor_key = named_survivor_menu_key(&app, "Survivor 1");
    send_physical_key(&mut app, survivor_key);
    app.update();
    let station_key = management_choice_key(&app, "Basic Processing");
    send_physical_key(&mut app, station_key);
    app.update();

    let choices = &app
        .world()
        .resource::<bd_tui::view_models::StatsViewModel>()
        .management
        .as_ref()
        .expect("recipe selection must be projected")
        .tasks;
    assert!(
        choices.iter().any(|choice| {
            choice.contains("Refine Timber")
                && choice.contains("Raw Timber")
                && choice.contains("Refined Materials")
        }),
        "contract=VISUAL-COLONY-WORK-002 case=recipe-choices expected human \
         source/result labels; actual={choices:?}"
    );
    assert!(
        choices
            .iter()
            .all(|choice| !choice.contains("recipe.") && !choice.contains("resource.")),
        "content IDs leaked into player-facing recipe choices: {choices:?}"
    );
}

#[test]
fn colony_projection_separates_next_worker_result_from_next_day_upkeep() {
    let mut app = outpost_runtime();
    place_named_survivor_at_resource_work_tile(
        &mut app,
        "Survivor 2",
        ResourceNodeType::WaterSource,
    );

    send_physical_key(&mut app, KeyCode::Char('c'));
    app.update();
    let survivor_key = named_survivor_menu_key(&app, "Survivor 2");
    send_physical_key(&mut app, survivor_key);
    app.update();
    let gather_key = management_choice_key(&app, "Gather Supplies");
    send_physical_key(&mut app, gather_key);
    app.update();
    send_physical_key(&mut app, KeyCode::Enter);
    app.update();
    app.update();

    let forecast = &app
        .world()
        .resource::<bd_tui::view_models::StatsViewModel>()
        .next_day_forecast;
    assert!(
        forecast.contains("Next worker") && forecast.contains("Next day"),
        "contract=VISUAL-COLONY-WORK-003 expected separately named worker \
         completion and day upkeep; actual={forecast:?}"
    );
}

#[test]
fn nonzero_raw_stockpile_is_projected_with_a_human_label() {
    let mut app = outpost_runtime();
    app.world_mut()
        .resource_mut::<ColonyResources>()
        .raw
        .insert("resource.raw_timber".into(), 2);
    app.update();

    let stats = app
        .world()
        .resource::<bd_tui::view_models::StatsViewModel>();
    let visible_colony_text = stats
        .party_names
        .iter()
        .chain(stats.station_status.iter())
        .cloned()
        .chain(std::iter::once(stats.next_day_forecast.clone()))
        .collect::<Vec<_>>()
        .join(" | ");
    assert!(
        visible_colony_text.contains("Raw Timber")
            && visible_colony_text.contains('2')
            && !visible_colony_text.contains("resource.raw_timber"),
        "contract=VISUAL-COLONY-WORK-004 expected visible human-labelled raw \
         stockpile; actual={visible_colony_text:?}"
    );
}

#[test]
fn blocked_direct_gatherer_projects_target_and_actionable_reason() {
    let mut app = outpost_runtime();
    let survivor = named_survivor(&mut app, "Survivor 3");
    let blocked_position = Position { x: 8, y: 8 };
    app.world_mut()
        .entity_mut(survivor)
        .insert(blocked_position);
    for blocker in [
        Position {
            x: blocked_position.x,
            y: blocked_position.y - 1,
        },
        Position {
            x: blocked_position.x,
            y: blocked_position.y + 1,
        },
        Position {
            x: blocked_position.x - 1,
            y: blocked_position.y,
        },
        Position {
            x: blocked_position.x + 1,
            y: blocked_position.y,
        },
    ] {
        app.world_mut()
            .resource_mut::<SmokeMap>()
            .set(blocker.x, blocker.y, Tile::Wall);
        app.world_mut()
            .resource_mut::<OutpostState>()
            .map
            .set(blocker.x, blocker.y, Tile::Wall);
    }

    send_physical_key(&mut app, KeyCode::Char('c'));
    app.update();
    let survivor_key = named_survivor_menu_key(&app, "Survivor 3");
    send_physical_key(&mut app, survivor_key);
    app.update();
    let gather_key = management_choice_key(&app, "Gather Supplies");
    send_physical_key(&mut app, gather_key);
    app.update();
    send_physical_key(&mut app, KeyCode::Enter);
    app.update();
    app.update();
    send_physical_key(&mut app, KeyCode::Char('.'));
    app.update();
    app.update();

    let row = app
        .world()
        .resource::<bd_tui::view_models::StatsViewModel>()
        .party_names
        .iter()
        .find(|entry| entry.starts_with("Survivor 3"))
        .cloned()
        .expect("blocked survivor must remain projected");
    assert!(
        row.contains("Blocked")
            && row.contains("Water")
            && (row.contains("No route") || row.contains("unreachable")),
        "contract=VISUAL-COLONY-WORK-005 expected blocked direct gather \
         target and actionable reason; actual={row:?}"
    );
}

#[test]
fn explicit_gather_assignment_overrides_pending_automatic_construction() {
    let mut app = outpost_runtime();

    send_physical_key(&mut app, KeyCode::Char('b'));
    app.update();
    send_physical_key(&mut app, KeyCode::Enter);
    app.update();
    send_physical_key(&mut app, KeyCode::Enter);
    app.update();
    app.update();
    send_physical_key(&mut app, KeyCode::Char('.'));
    app.update();
    app.update();

    let (survivor, survivor_name) = {
        let mut workers = app
            .world_mut()
            .query_filtered::<(Entity, &Name), With<bd_core::colony::stations::AutoConstructing>>();
        workers
            .iter(app.world())
            .next()
            .map(|(entity, name)| (entity, name.0.clone()))
            .expect("queued construction must route at least one idle survivor")
    };

    send_physical_key(&mut app, KeyCode::Char('c'));
    app.update();
    let survivor_key = named_survivor_menu_key(&app, &survivor_name);
    send_physical_key(&mut app, survivor_key);
    app.update();

    let choices = &app
        .world()
        .resource::<bd_tui::view_models::StatsViewModel>()
        .management
        .as_ref()
        .expect("task management must remain visible over queued construction")
        .tasks;
    assert!(
        choices
            .iter()
            .any(|choice| choice.contains("Gather Supplies")),
        "queued construction replaced the survivor task choices; choices={choices:?}"
    );

    let gather_key = management_choice_key(&app, "Gather Supplies");
    send_physical_key(&mut app, gather_key);
    app.update();
    send_physical_key(&mut app, KeyCode::Enter);
    app.update();
    app.update();

    assert_eq!(
        app.world().get::<SurvivorTask>(survivor),
        Some(&SurvivorTask::Gathering(PoolKind::Supplies)),
        "an explicit survivor assignment must override automatic construction"
    );
    assert!(
        !app.world()
            .entity(survivor)
            .contains::<bd_core::colony::stations::AutoConstructing>(),
        "the gatherer retained automatic-construction ownership"
    );
    let activity = app
        .world()
        .get::<bd_core::colony::survivors::WorkerActivity>(survivor)
        .expect("the reassigned survivor must project current gathering activity");
    assert!(
        !matches!(
            activity,
            bd_core::colony::survivors::WorkerActivity::EnRoute { target, .. }
                | bd_core::colony::survivors::WorkerActivity::Working { target, .. }
                | bd_core::colony::survivors::WorkerActivity::Blocked { target, .. }
                if target.contains("construction")
        ),
        "the gatherer still shows stale construction activity: {activity:?}"
    );
}

#[test]
fn management_cancel_is_atomic_and_discards_modal_gameplay_input() {
    for open_key in [KeyCode::Char('c'), KeyCode::Char('e')] {
        let mut app = outpost_runtime();
        if open_key == KeyCode::Char('e') {
            build_station(&mut app, StationType::Stove, Direction::East);
        }
        let turn_before = app.world().resource::<RunSession>().turn;
        let replay_before = action_ids(&app).len();
        let tasks_before = survivor_tasks(&mut app);

        send_keys(&mut app, &[open_key, KeyCode::Char('.'), KeyCode::Esc]);
        app.update();
        app.update();

        assert!(
            app.world()
                .resource::<bd_tui::view_models::StatsViewModel>()
                .management
                .is_none(),
            "{open_key:?} management must close on Esc"
        );
        assert_eq!(app.world().resource::<RunSession>().turn, turn_before);
        assert_eq!(action_ids(&app).len(), replay_before);
        assert_eq!(survivor_tasks(&mut app), tasks_before);
    }
}

#[test]
fn station_staffing_confirmation_changes_only_the_named_survivor_relationship() {
    let mut app = outpost_runtime();
    let station = build_station(&mut app, StationType::Stove, Direction::East);
    let selected_name = "Survivor 2";

    send_physical_key(&mut app, KeyCode::Char('e'));
    app.update();
    let survivor_key = named_survivor_menu_key(&app, selected_name);
    send_physical_key(&mut app, survivor_key);
    app.update();
    let station_key = management_choice_key(&app, "Stove");
    send_physical_key(&mut app, station_key);
    app.update();
    send_physical_key(&mut app, KeyCode::Enter);
    app.update();
    app.update();

    assert_eq!(
        named_survivor_task(&mut app, selected_name),
        SurvivorTask::AssignedTo(station.to_bits())
    );
    assert_eq!(
        named_survivor_task(&mut app, "Survivor 1"),
        SurvivorTask::Idle
    );
    assert_eq!(
        named_survivor_task(&mut app, "Survivor 3"),
        SurvivorTask::Idle
    );
}

#[test]
fn processing_assignment_selects_named_survivor_station_and_recipe_while_paused() {
    let mut app = outpost_runtime();
    let selected_name = "Survivor 2";
    let turn_before = app.world().resource::<RunSession>().turn;

    send_physical_key(&mut app, KeyCode::Char('e'));
    app.update();
    let survivor_key = named_survivor_menu_key(&app, selected_name);
    send_physical_key(&mut app, survivor_key);
    app.update();
    let station_key = management_choice_key(&app, "Basic Processing");
    send_physical_key(&mut app, station_key);
    app.update();
    let recipe_key = management_choice_key(&app, "Refine Timber");
    send_physical_key(&mut app, recipe_key);
    app.update();
    send_physical_key(&mut app, KeyCode::Enter);
    app.update();
    app.update();

    let survivor = named_survivor(&mut app, selected_name);
    let job = app
        .world()
        .get::<bd_core::colony::logistics::LogisticsJob>(survivor)
        .expect("confirmed processing assignment must create a durable job");
    assert_eq!(job.recipe_id, "recipe.refine_timber");
    assert_eq!(
        app.world()
            .get::<bd_core::colony::logistics::Cargo>(survivor)
            .expect("confirmed processing assignment must create cargo")
            .amount,
        0
    );
    assert_eq!(app.world().resource::<RunSession>().turn, turn_before);
}

#[test]
fn production_key_workflow_assigns_travels_gathers_refines_and_reports() {
    let mut app = outpost_runtime();
    let materials_before = app
        .world()
        .resource::<ColonyResources>()
        .pools
        .get(PoolKind::Materials)
        .unwrap()
        .current;

    send_physical_key(&mut app, KeyCode::Char('e'));
    app.update();
    let survivor_key = named_survivor_menu_key(&app, "Survivor 1");
    send_physical_key(&mut app, survivor_key);
    app.update();
    let station_key = management_choice_key(&app, "Basic Processing");
    send_physical_key(&mut app, station_key);
    app.update();
    let recipe_key = management_choice_key(&app, "Refine Timber");
    send_physical_key(&mut app, recipe_key);
    app.update();
    send_physical_key(&mut app, KeyCode::Enter);
    app.update();
    app.update();

    for _ in 0..160 {
        send_physical_key(&mut app, KeyCode::Char('.'));
        app.update();
        app.update();
        let materials = app
            .world()
            .resource::<ColonyResources>()
            .pools
            .get(PoolKind::Materials)
            .unwrap()
            .current;
        if materials > materials_before {
            break;
        }
    }

    assert_eq!(
        app.world()
            .resource::<ColonyResources>()
            .pools
            .get(PoolKind::Materials)
            .unwrap()
            .current,
        materials_before + 1
    );
    let party = &app
        .world()
        .resource::<bd_tui::view_models::StatsViewModel>()
        .party_names;
    assert!(
        party.iter().any(|entry| {
            entry.contains("Survivor 1")
                && entry.contains("cargo")
                && (entry.contains("ToStation")
                    || entry.contains("ReadyToRefine")
                    || entry.contains("ToSource"))
        }),
        "worker projection must retain recipe, stage/activity, and cargo; party={party:?}"
    );
}

#[test]
fn deterministic_production_key_fuzz_preserves_colony_invariants() {
    const FUZZ_SEED: u64 = 0x0BDC_0107;
    const STEPS: usize = 256;
    let keys = [
        KeyCode::Char('e'),
        KeyCode::Esc,
        KeyCode::Char('1'),
        KeyCode::Char('2'),
        KeyCode::Char('3'),
        KeyCode::Enter,
        KeyCode::Up,
        KeyCode::Down,
        KeyCode::Char('.'),
    ];
    let mut state = FUZZ_SEED;
    let mut app = outpost_runtime();

    for step in 0..STEPS {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        let key = keys[(state as usize) % keys.len()];
        send_physical_key(&mut app, key);
        app.update();
        app.update();

        let mut positions = app
            .world_mut()
            .query_filtered::<&Position, With<Survivor>>()
            .iter(app.world())
            .copied()
            .collect::<Vec<_>>();
        let survivor_count = positions.len();
        positions.sort_by_key(|position| (position.y, position.x));
        positions.dedup();
        assert_eq!(
            positions.len(),
            survivor_count,
            "seed={FUZZ_SEED} step={step} key={key:?}: survivors stacked"
        );
        for pool in app.world().resource::<ColonyResources>().pools.iter() {
            assert!(
                (pool.min..=pool.max).contains(&pool.current),
                "seed={FUZZ_SEED} step={step} key={key:?}: pool={:?} value={} bounds={}..={}",
                pool.kind,
                pool.current,
                pool.min,
                pool.max
            );
        }
        let mut workers = app.world_mut().query_filtered::<(
            Option<&bd_core::colony::logistics::LogisticsJob>,
            Option<&bd_core::colony::logistics::Cargo>,
        ), With<Survivor>>();
        for (job, cargo) in workers.iter(app.world()) {
            assert_eq!(
                job.is_some(),
                cargo.is_some(),
                "seed={FUZZ_SEED} step={step} key={key:?}: job/cargo ownership diverged"
            );
        }
    }
}

#[test]
fn entering_build_placement_starts_on_a_visible_adjacent_candidate() {
    let mut app = outpost_runtime();
    let player_entity = player(&mut app);
    let player_position = *app
        .world()
        .get::<Position>(player_entity)
        .expect("player position must exist");

    send_physical_key(&mut app, KeyCode::Char('b'));
    app.update();
    send_physical_key(&mut app, KeyCode::Enter);
    app.update();

    let BuildInteraction::Placing { cursor, .. } = app.world().resource::<BuildInteraction>()
    else {
        panic!("build interaction must enter placement");
    };
    let distance = (cursor.x - player_position.x).abs() + (cursor.y - player_position.y).abs();
    assert_eq!(
        distance, 1,
        "the first highlighted placement must be the same adjacent tile Enter would build; player={player_position:?}, preview={:?}",
        cursor
    );
}

#[test]
fn build_placement_cursor_moves_cumulatively_without_moving_the_player() {
    let mut app = outpost_runtime();
    let player_entity = player(&mut app);
    let player_before = *app
        .world()
        .get::<Position>(player_entity)
        .expect("player position must exist");
    let turn_before = app.world().resource::<RunSession>().turn;
    let replay_before = action_ids(&app);
    let supplies_before = colony_supplies(&app);
    let stations_before = station_count(&mut app);

    send_physical_key(&mut app, KeyCode::Char('b'));
    app.update();
    send_physical_key(&mut app, KeyCode::Enter);
    app.update();
    let BuildInteraction::Placing {
        cursor: cursor_before,
        ..
    } = *app.world().resource::<BuildInteraction>()
    else {
        panic!("contract=INPUT-BUILD-003 step=enter-placement expected=Placing");
    };

    send_physical_key(&mut app, KeyCode::Char('d'));
    app.update();
    send_physical_key(&mut app, KeyCode::Char('d'));
    app.update();

    let BuildInteraction::Placing {
        cursor: cursor_after,
        ..
    } = *app.world().resource::<BuildInteraction>()
    else {
        panic!("contract=INPUT-BUILD-003 step=move-twice expected=Placing");
    };
    assert_eq!(
        cursor_after,
        Position {
            x: cursor_before.x + 2,
            y: cursor_before.y,
        },
        "contract=INPUT-BUILD-003 case=cumulative-east fixture=outpost_clean \
         input=[d,d] expected cursor to move from itself; \
         cursor_before={cursor_before:?} cursor_after={cursor_after:?} player={player_before:?}"
    );
    assert_eq!(
        app.world().get::<Position>(player_entity),
        Some(&player_before)
    );
    assert_eq!(app.world().resource::<RunSession>().turn, turn_before);
    assert_eq!(action_ids(&app), replay_before);
    assert_eq!(colony_supplies(&app), supplies_before);
    assert_eq!(station_count(&mut app), stations_before);
}

#[test]
fn distant_build_confirmation_places_at_the_absolute_preview_coordinate() {
    let mut app = outpost_runtime();
    let stations_before = station_count(&mut app);
    let player_entity = player(&mut app);
    let player_before = *app
        .world()
        .get::<Position>(player_entity)
        .expect("player position must exist");
    let supplies_before = colony_supplies(&app);

    send_physical_key(&mut app, KeyCode::Char('b'));
    app.update();
    send_physical_key(&mut app, KeyCode::Enter);
    app.update();
    send_physical_key(&mut app, KeyCode::Char('d'));
    app.update();
    send_physical_key(&mut app, KeyCode::Char('d'));
    app.update();

    let BuildInteraction::Placing {
        cursor: expected_position,
        validation: Ok(()),
        ..
    } = *app.world().resource::<BuildInteraction>()
    else {
        panic!("contract=INPUT-BUILD-004 step=preview expected=valid distant Placing state");
    };
    let distance = (expected_position.x - player_before.x).abs()
        + (expected_position.y - player_before.y).abs();
    assert!(
        distance > 1,
        "contract=INPUT-BUILD-004 fixture requires a non-adjacent preview; \
         player={player_before:?} preview={expected_position:?}"
    );

    send_physical_key(&mut app, KeyCode::Enter);
    app.update();
    app.update();

    let mut built_positions = app
        .world_mut()
        .query_filtered::<&Position, With<Station>>()
        .iter(app.world())
        .copied()
        .collect::<Vec<_>>();
    built_positions.sort_by_key(|position| (position.y, position.x));
    assert!(
        built_positions.contains(&expected_position)
            && built_positions.len() == stations_before + 1,
        "contract=INPUT-BUILD-004 case=distant-confirm fixture=outpost_clean \
         expected one station at absolute preview; player={player_before:?} \
         preview={expected_position:?} actual={built_positions:?}"
    );
    assert_eq!(
        app.world().get::<Position>(player_entity),
        Some(&player_before)
    );
    assert_eq!(
        colony_supplies(&app),
        supplies_before - bd_core::colony::stations::STATION_BUILD_COST_SUPPLIES
    );
}

#[test]
fn placed_construction_site_has_distinct_map_and_progress_feedback() {
    let mut app = outpost_runtime();
    send_physical_key(&mut app, KeyCode::Char('b'));
    app.update();
    send_physical_key(&mut app, KeyCode::Enter);
    app.update();
    send_physical_key(&mut app, KeyCode::Char('d'));
    app.update();
    send_physical_key(&mut app, KeyCode::Enter);
    app.update();
    app.update();

    let mut sites = app.world_mut().query_filtered::<(
        Entity,
        &Position,
        &bd_core::colony::stations::ConstructionSite,
    ), With<Station>>();
    let (_, site_position, site) = sites
        .iter(app.world())
        .next()
        .expect("accepted placement must create a construction site");
    assert_eq!((site.work_completed, site.work_required), (0, 4));
    let site_position = *site_position;

    let map = app.world().resource::<bd_tui::view_models::MapViewModel>();
    assert!(
        map.visuals.iter().any(|visual| {
            visual.position == site_position
                && visual.token == bd_tui::visual::VisualToken::Station
                && visual.glyph == Some('%')
        }),
        "construction must not look like an operational station: {:?}",
        map.visuals
    );
    let stats = app
        .world()
        .resource::<bd_tui::view_models::StatsViewModel>();
    assert!(
        stats
            .station_status
            .iter()
            .any(|status| status.contains("Stove construction — 0/4 work")),
        "construction progress must be visible: {:?}",
        stats.station_status
    );
}

#[test]
fn invalid_build_confirmation_keeps_preview_active_and_is_atomic() {
    let mut app = outpost_runtime();
    let turn_before = app.world().resource::<RunSession>().turn;
    let replay_before = action_ids(&app).len();
    let supplies_before = colony_supplies(&app);
    let stations_before = station_count(&mut app);

    send_physical_key(&mut app, KeyCode::Char('b'));
    app.update();
    send_physical_key(&mut app, KeyCode::Enter);
    app.update();
    send_physical_key(&mut app, KeyCode::Char('a'));
    app.update();
    send_physical_key(&mut app, KeyCode::Char('a'));
    app.update();
    assert!(
        app.world()
            .resource::<bd_tui::view_models::MapViewModel>()
            .build_ghost_denial
            .is_some(),
        "the wall candidate must be visibly invalid before confirmation"
    );

    send_physical_key(&mut app, KeyCode::Enter);
    app.update();
    app.update();

    assert!(
        matches!(
            app.world().resource::<BuildInteraction>(),
            BuildInteraction::Placing { .. }
        ),
        "a rejected placement must remain open so the player can move the preview"
    );
    assert_eq!(app.world().resource::<RunSession>().turn, turn_before);
    assert_eq!(action_ids(&app).len(), replay_before);
    assert_eq!(colony_supplies(&app), supplies_before);
    assert_eq!(station_count(&mut app), stations_before);
}

#[test]
fn denied_build_resolution_returns_to_correctable_placement() {
    let mut app = outpost_runtime();
    let stations_before = station_count(&mut app);
    app.world_mut()
        .resource_mut::<ColonyResources>()
        .pools
        .get_mut(PoolKind::Supplies)
        .expect("Supplies pool must exist")
        .current = 0;
    let turn_before = app.world().resource::<RunSession>().turn;

    send_physical_key(&mut app, KeyCode::Char('b'));
    app.update();
    send_physical_key(&mut app, KeyCode::Enter);
    app.update();
    send_physical_key(&mut app, KeyCode::Enter);
    app.update();
    app.update();

    assert!(
        matches!(
            app.world().resource::<BuildInteraction>(),
            BuildInteraction::Placing {
                validation: Err(
                    bd_core::colony::stations::BuildInteractionDenial::NotEnoughSupplies
                ),
                ..
            }
        ),
        "a core denial must return AwaitingResolution to the same correctable placement"
    );
    assert_eq!(app.world().resource::<RunSession>().turn, turn_before);
    assert_eq!(station_count(&mut app), stations_before);
}

#[test]
fn altar_and_idle_survivor_remain_distinct_without_color() {
    let mut app = outpost_runtime();
    let station = build_station(&mut app, StationType::Altar, Direction::East);
    let station_position = *app
        .world()
        .get::<Position>(station)
        .expect("built Altar must have a position");
    let station_glyph = station_map_glyph(&app, station_position);
    let map = app.world().resource::<bd_tui::view_models::MapViewModel>();
    let symbols = app.world().resource::<bd_tui::visual::SymbolRegistry>();
    let survivor_glyphs = map
        .visuals
        .iter()
        .filter(|visual| {
            matches!(
                visual.token,
                bd_tui::visual::VisualToken::WorkerIdle
                    | bd_tui::visual::VisualToken::WorkerEnRoute
                    | bd_tui::visual::VisualToken::WorkerWorking
                    | bd_tui::visual::VisualToken::WorkerBlocked
                    | bd_tui::visual::VisualToken::WorkerResting
                    | bd_tui::visual::VisualToken::WorkerDefending
            )
        })
        .filter_map(|visual| {
            visual
                .glyph
                .or_else(|| symbols.get(visual.token).map(|symbol| symbol.glyph))
        })
        .collect::<Vec<_>>();

    assert!(
        survivor_glyphs
            .iter()
            .all(|survivor_glyph| *survivor_glyph != station_glyph),
        "Altar `{station_glyph}` collides with an idle survivor in ASCII-only output; survivors={survivor_glyphs:?}"
    );
}

#[test]
fn workshop_and_water_source_remain_distinct_without_color() {
    let mut app = outpost_runtime();
    let station = build_station(&mut app, StationType::Workshop, Direction::East);
    let station_position = *app
        .world()
        .get::<Position>(station)
        .expect("built Workshop must have a position");
    let station_glyph = station_map_glyph(&app, station_position);
    let water_glyph = resource_map_glyph(&mut app, ResourceNodeType::WaterSource);

    assert_ne!(
        station_glyph, water_glyph,
        "Workshop and Water Source both render as `{station_glyph}` without color"
    );
}

#[test]
fn map_projection_uses_one_semantic_visual_list() {
    let app = outpost_runtime();
    let map = app.world().resource::<bd_tui::view_models::MapViewModel>();
    let tokens = map
        .visuals
        .iter()
        .map(|visual| visual.token)
        .collect::<std::collections::HashSet<_>>();

    assert!(tokens.contains(&bd_tui::visual::VisualToken::Player));
    assert!(tokens.contains(&bd_tui::visual::VisualToken::WorkerIdle));
    assert!(
        tokens.contains(&bd_tui::visual::VisualToken::Trees)
            || tokens.contains(&bd_tui::visual::VisualToken::WaterSource)
            || tokens.contains(&bd_tui::visual::VisualToken::WildPlants)
    );
    assert!(tokens.contains(&bd_tui::visual::VisualToken::Exit));
}

#[test]
fn staffed_and_unstaffed_station_have_distinct_ascii_projection() {
    let mut app = outpost_runtime();
    let station = build_station(&mut app, StationType::Stove, Direction::East);
    let station_position = *app
        .world()
        .get::<Position>(station)
        .expect("built Stove must have a position");
    let unstaffed = station_map_glyph(&app, station_position);

    send_physical_key(&mut app, KeyCode::Char('e'));
    app.update();
    let survivor_key = named_survivor_menu_key(&app, "Survivor 1");
    send_physical_key(&mut app, survivor_key);
    app.update();
    let station_key = management_choice_key(&app, "Stove");
    send_physical_key(&mut app, station_key);
    app.update();
    send_physical_key(&mut app, KeyCode::Enter);
    app.update();
    app.update();

    let staffed = station_map_glyph(&app, station_position);
    assert_ne!(
        staffed, unstaffed,
        "staffing the Stove leaves the same `{staffed}` ASCII projection"
    );
}
