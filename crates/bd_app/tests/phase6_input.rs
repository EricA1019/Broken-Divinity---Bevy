use std::collections::HashSet;

use bd_core::{
    colony::{
        production::ColonyResources,
        stations::{BuildInteraction, Station, StationType},
        survivors::{Survivor, SurvivorTask},
    },
    components::{BlocksMovement, Name, Player, Position, ResourceNode, ResourceNodeType, Tile},
    direction::Direction,
    map::SmokeMap,
    pathfinding::{AStarPathfinder, Pathfinder},
    session::RunSession,
    signals::{ActionIntent, PoolKind},
    spatial::{EntityScope, FOUNDATION_DUNGEON_ID, GameMode, OutpostState, TransitionIntent},
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

#[derive(Debug, PartialEq, Eq)]
struct VisibleStatsProjection {
    hp: (i32, i32),
    ap: (i32, i32),
    supplies: i32,
    faith: i32,
    materials: i32,
    wild_plants: i32,
    day: u64,
    run_outcome: String,
    extracted_loot: u32,
    party_names: Vec<String>,
    station_status: Vec<String>,
    latest_daily_summary: Vec<String>,
    stored_items: Vec<(String, u32)>,
    faction_standings: Vec<(String, i32, String)>,
}

#[derive(Debug, PartialEq, Eq)]
struct VisibleProjection {
    mode: GameMode,
    screen: String,
    stats: VisibleStatsProjection,
    map_size: (i32, i32),
    player_position: Option<Position>,
    visuals: Vec<(Position, String, Option<char>)>,
    assigned_targets: Vec<Position>,
    actions: Vec<(String, String, bool, Option<String>)>,
    log: Vec<(String, String)>,
}

fn visible_projection(app: &App) -> VisibleProjection {
    let stats = app
        .world()
        .resource::<bd_tui::view_models::StatsViewModel>();
    let map = app.world().resource::<bd_tui::view_models::MapViewModel>();
    let mut visuals = map
        .visuals
        .iter()
        .map(|visual| (visual.position, format!("{:?}", visual.token), visual.glyph))
        .collect::<Vec<_>>();
    visuals.sort_by(|left, right| {
        (left.0.y, left.0.x, &left.1, left.2).cmp(&(right.0.y, right.0.x, &right.1, right.2))
    });
    let mut assigned_targets = map.assigned_targets.clone();
    assigned_targets.sort_by_key(|position| (position.y, position.x));
    let actions = app
        .world()
        .resource::<bd_tui::view_models::ActionListViewModel>()
        .actions
        .iter()
        .map(|action| {
            (
                action.label.clone(),
                action.key_hint.clone(),
                action.enabled,
                action.denial_reason.clone(),
            )
        })
        .collect();
    let log = app
        .world()
        .resource::<bd_tui::view_models::LogViewModel>()
        .entries
        .iter()
        .map(|entry| (entry.message.clone(), format!("{:?}", entry.level)))
        .collect();

    VisibleProjection {
        mode: *app.world().resource::<GameMode>(),
        screen: app
            .world()
            .resource::<bd_tui::screens::ScreenState>()
            .current
            .clone(),
        stats: VisibleStatsProjection {
            hp: (stats.hp_current, stats.hp_max),
            ap: (stats.ap_current, stats.ap_max),
            supplies: stats.supplies,
            faith: stats.faith,
            materials: stats.materials,
            wild_plants: stats.wild_plants,
            day: stats.day,
            run_outcome: format!("{:?}", stats.run_outcome),
            extracted_loot: stats.extracted_loot,
            party_names: stats.party_names.clone(),
            station_status: stats.station_status.clone(),
            latest_daily_summary: stats.latest_daily_summary.clone(),
            stored_items: stats.stored_items.clone(),
            faction_standings: stats.faction_standings.clone(),
        },
        map_size: (map.width, map.height),
        player_position: map.player_pos,
        visuals,
        assigned_targets,
        actions,
        log,
    }
}

fn round_trip_world(app: &mut App, case_id: &str) {
    let session = app.world().resource::<RunSession>().clone();
    let save_dir = std::env::temp_dir().join(format!(
        "bd-visible-projection-{case_id}-{}",
        std::process::id()
    ));
    let path = bd_core::save::save_world(app.world_mut(), session.seed, session.turn, &save_dir)
        .unwrap_or_else(|error| {
            panic!("contract=PERSIST-PROJECTION-001 case={case_id} checkpoint=save error={error}")
        });
    let snapshot = bd_core::save::load_snapshot(&path).unwrap_or_else(|error| {
        panic!("contract=PERSIST-PROJECTION-001 case={case_id} checkpoint=load error={error}")
    });
    bd_core::save::restore_snapshot_into(app.world_mut(), &snapshot).unwrap_or_else(|error| {
        panic!("contract=PERSIST-PROJECTION-001 case={case_id} checkpoint=restore error={error}")
    });
    app.update();
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_dir(save_dir);
}

fn key_for_step(from: Position, to: Position) -> KeyCode {
    match (to.x - from.x, to.y - from.y) {
        (0, -1) => KeyCode::Char('w'),
        (0, 1) => KeyCode::Char('s'),
        (-1, 0) => KeyCode::Char('a'),
        (1, 0) => KeyCode::Char('d'),
        delta => panic!("workflow path produced non-cardinal step {delta:?}"),
    }
}

fn submit_tactical_key(app: &mut App, key: KeyCode) {
    send_physical_key(app, key);
    app.update();
    app.update();
}

fn hostile_with_position(app: &mut App) -> Option<(Entity, Position)> {
    let mut query = app
        .world_mut()
        .query_filtered::<(Entity, &Position, &bd_core::pools::Pools), (
            With<bd_core::relationships::FactionMember>,
            bevy_ecs::query::Without<Player>,
        )>();
    query
        .iter(app.world())
        .find_map(|(entity, position, pools)| {
            pools
                .get(PoolKind::Health)
                .is_some_and(|health| health.current > health.min)
                .then_some((entity, *position))
        })
}

fn loose_item_with_position(app: &mut App) -> Option<(Entity, Position)> {
    let mut query = app.world_mut().query_filtered::<(Entity, &Position), (
        With<bd_core::inventory::Item>,
        bevy_ecs::query::Without<bd_core::relationships::ContainedIn>,
    )>();
    query
        .iter(app.world())
        .next()
        .map(|(entity, position)| (entity, *position))
}

fn move_player_to_with_physical_keys(
    app: &mut App,
    target: Position,
    case_id: &str,
    avoid_hostile: bool,
) {
    for step in 0..64 {
        let actor = player(app);
        let current = *app
            .world()
            .get::<Position>(actor)
            .expect("workflow player must have a position");
        if current == target {
            return;
        }
        let blocked = if avoid_hostile {
            hostile_with_position(app)
                .map(|(_, position)| HashSet::from([position]))
                .unwrap_or_default()
        } else {
            HashSet::new()
        };
        let path = AStarPathfinder
            .find_path(
                app.world().resource::<SmokeMap>(),
                current,
                target,
                &blocked,
            )
            .unwrap_or_else(|| {
                panic!(
                    "contract=DUNGEON-WORKFLOW-001 case={case_id} step={step} \
                     no physical-key route from {current:?} to {target:?}"
                )
            });
        let next = path.get(1).copied().unwrap_or_else(|| {
            panic!(
                "contract=DUNGEON-WORKFLOW-001 case={case_id} step={step} \
                 route contained no next tile"
            )
        });
        submit_tactical_key(app, key_for_step(current, next));
        assert_eq!(
            *app.world().resource::<GameMode>(),
            GameMode::Tactical,
            "contract=DUNGEON-WORKFLOW-001 case={case_id} step={step} \
             player left Tactical before reaching the checkpoint"
        );
    }
    panic!(
        "contract=DUNGEON-WORKFLOW-001 case={case_id} \
         target {target:?} was not reached within 64 physical-key steps"
    );
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
fn colony_checkpoint_round_trip_preserves_the_visible_projection() {
    // Contract: PERSIST-PROJECTION-001
    // Given: a named colony worker has an active gathering assignment.
    // When: production persistence saves and restores the world.
    // Then: every stable player-visible projection is identical.
    // Must not change: screen, stats, map geometry, actions, and log.
    // Evidence layers: projection, state diff, persistence, workflow.
    let mut app = outpost_runtime();
    let selected_name = "Iven";

    send_physical_key(&mut app, KeyCode::Char('c'));
    app.update();
    let survivor_key = named_survivor_menu_key(&app, selected_name);
    send_physical_key(&mut app, survivor_key);
    app.update();
    let task_key = management_choice_key(&app, "Gather Supplies");
    send_physical_key(&mut app, task_key);
    app.update();
    send_physical_key(&mut app, KeyCode::Enter);
    app.update();
    app.update();

    let before = visible_projection(&app);
    round_trip_world(&mut app, "colony-assigned-worker");
    let after = visible_projection(&app);

    assert_eq!(
        after, before,
        "contract=PERSIST-PROJECTION-001 case=colony-assigned-worker \
         checkpoint=restored visible projection diverged"
    );
}

#[test]
fn tactical_checkpoint_round_trip_preserves_the_visible_projection() {
    let mut app = outpost_runtime();
    send_physical_key(&mut app, KeyCode::Char('t'));
    app.update();
    app.update();
    app.update();
    submit_tactical_key(&mut app, KeyCode::Char('.'));

    let before = visible_projection(&app);
    round_trip_world(&mut app, "tactical-after-enemy-phase");
    let after = visible_projection(&app);

    assert_eq!(
        after, before,
        "contract=PERSIST-PROJECTION-001 case=tactical-after-enemy-phase \
         checkpoint=restored visible projection diverged"
    );
}

#[test]
fn production_keys_complete_the_fixed_dungeon_loop_with_named_checkpoints() {
    // Contract: DUNGEON-WORKFLOW-001
    // Given: a fresh Foundation shelter with the entry cost available.
    // When: only production keys drive entry, loot, combat, exit, and extraction.
    // Then: the run extracts and credits exactly one carried potion.
    // Must not change: fixed checkpoints, paid cost, and one-credit semantics.
    // Evidence layers: input state machine, domain, projection, state diff, workflow.
    let mut app = outpost_runtime();
    let supplies_before = colony_supplies(&app);

    send_physical_key(&mut app, KeyCode::Char('t'));
    app.update();
    app.update();
    app.update();
    assert_eq!(
        *app.world().resource::<GameMode>(),
        GameMode::Tactical,
        "contract=DUNGEON-WORKFLOW-001 checkpoint=paid-entry expected Tactical"
    );
    assert_eq!(
        colony_supplies(&app),
        supplies_before - 2,
        "contract=DUNGEON-WORKFLOW-001 checkpoint=paid-entry wrong colony cost"
    );
    let initial_map = app.world().resource::<bd_tui::view_models::MapViewModel>();
    for token in [
        bd_tui::visual::VisualToken::Player,
        bd_tui::visual::VisualToken::Enemy,
        bd_tui::visual::VisualToken::Item,
        bd_tui::visual::VisualToken::Exit,
    ] {
        assert!(
            initial_map
                .visuals
                .iter()
                .any(|visual| visual.token == token),
            "contract=DUNGEON-WORKFLOW-001 checkpoint=visible-arrival \
             missing {token:?}; visuals={:?}",
            initial_map.visuals
        );
    }

    let (_, item_position) =
        loose_item_with_position(&mut app).expect("fixed dungeon must contain loose loot");
    move_player_to_with_physical_keys(&mut app, item_position, "loot-detour", true);
    submit_tactical_key(&mut app, KeyCode::Char('p'));
    assert!(
        loose_item_with_position(&mut app).is_none(),
        "contract=DUNGEON-WORKFLOW-001 checkpoint=pickup loose item remained on map"
    );

    for combat_action in 0..64 {
        let Some((_, hostile_position)) = hostile_with_position(&mut app) else {
            break;
        };
        let player_position = {
            let actor = player(&mut app);
            *app.world()
                .get::<Position>(actor)
                .expect("workflow player must have a position")
        };
        let distance = (player_position.x - hostile_position.x).abs()
            + (player_position.y - hostile_position.y).abs();
        if distance > 1 {
            let candidates = [
                Position {
                    x: hostile_position.x + 1,
                    y: hostile_position.y,
                },
                Position {
                    x: hostile_position.x - 1,
                    y: hostile_position.y,
                },
                Position {
                    x: hostile_position.x,
                    y: hostile_position.y + 1,
                },
                Position {
                    x: hostile_position.x,
                    y: hostile_position.y - 1,
                },
            ];
            let next = candidates
                .into_iter()
                .filter(|candidate| {
                    app.world()
                        .resource::<SmokeMap>()
                        .is_walkable(candidate.x, candidate.y)
                })
                .filter_map(|candidate| {
                    AStarPathfinder
                        .find_path(
                            app.world().resource::<SmokeMap>(),
                            player_position,
                            candidate,
                            &HashSet::from([hostile_position]),
                        )
                        .and_then(|path| path.get(1).copied().map(|next| (path.len(), next)))
                })
                .min_by_key(|(length, _)| *length)
                .map(|(_, next)| next)
                .unwrap_or_else(|| {
                    panic!(
                        "contract=DUNGEON-WORKFLOW-001 checkpoint=encounter \
                         action={combat_action} no route to hostile"
                    )
                });
            submit_tactical_key(&mut app, key_for_step(player_position, next));
            continue;
        }
        submit_tactical_key(&mut app, KeyCode::Char('f'));
    }
    assert!(
        hostile_with_position(&mut app).is_none(),
        "contract=DUNGEON-WORKFLOW-001 checkpoint=combat hostile survived 64 physical actions"
    );

    let exit = app
        .world_mut()
        .query_filtered::<&Position, With<bd_core::components::ExitTile>>()
        .iter(app.world())
        .copied()
        .next()
        .expect("fixed dungeon must contain an exit");
    move_player_to_with_physical_keys(&mut app, exit, "reach-exit", false);
    submit_tactical_key(&mut app, KeyCode::Char('r'));
    app.update();

    assert_eq!(
        *app.world().resource::<GameMode>(),
        GameMode::Outpost,
        "contract=DUNGEON-WORKFLOW-001 checkpoint=extraction expected Outpost"
    );
    assert_eq!(
        app.world().resource::<RunSession>().outcome,
        bd_core::session::RunOutcome::Extracted,
        "contract=DUNGEON-WORKFLOW-001 checkpoint=extraction outcome was not retained"
    );
    assert_eq!(
        app.world()
            .resource::<bd_core::colony::production::ColonyStorage>()
            .count("item.healing_potion"),
        1,
        "contract=DUNGEON-WORKFLOW-001 checkpoint=colony-result loot was not applied exactly once"
    );
}

#[test]
fn production_keys_complete_defeat_title_and_shelter_restart() {
    // Contract: DUNGEON-WORKFLOW-002
    // Given: a fresh Foundation shelter and fixed dungeon.
    // When: production keys wait for defeat, restart, and begin a new run.
    // Then: one clean shelter player returns with defeat history retained.
    // Must not change: defeat cannot award unextracted loot.
    // Evidence layers: input state machine, domain, state diff, workflow.
    let mut app = outpost_runtime();
    send_physical_key(&mut app, KeyCode::Char('t'));
    app.update();
    app.update();
    app.update();
    assert_eq!(
        *app.world().resource::<GameMode>(),
        GameMode::Tactical,
        "contract=DUNGEON-WORKFLOW-002 checkpoint=entry expected Tactical"
    );

    for turn in 0..64 {
        if *app.world().resource::<GameMode>() == GameMode::GameOver {
            break;
        }
        submit_tactical_key(&mut app, KeyCode::Char('.'));
        assert!(
            matches!(
                *app.world().resource::<GameMode>(),
                GameMode::Tactical | GameMode::GameOver
            ),
            "contract=DUNGEON-WORKFLOW-002 checkpoint=defeat turn={turn} \
             entered an illegal mode"
        );
    }
    assert_eq!(
        *app.world().resource::<GameMode>(),
        GameMode::GameOver,
        "contract=DUNGEON-WORKFLOW-002 checkpoint=defeat player survived 64 waits"
    );
    assert_eq!(
        app.world().resource::<RunSession>().outcome,
        bd_core::session::RunOutcome::Defeated
    );
    assert_eq!(
        app.world()
            .resource::<bd_core::colony::production::ColonyStorage>()
            .count("item.healing_potion"),
        0,
        "contract=DUNGEON-WORKFLOW-002 checkpoint=defeat awarded unextracted loot"
    );

    app.update();
    send_physical_key(&mut app, KeyCode::Char('r'));
    app.update();
    app.update();
    assert_eq!(
        *app.world().resource::<GameMode>(),
        GameMode::Title,
        "contract=DUNGEON-WORKFLOW-002 checkpoint=restart-key expected Title"
    );
    send_physical_key(&mut app, KeyCode::Enter);
    app.update();
    app.update();

    assert_eq!(
        *app.world().resource::<GameMode>(),
        GameMode::Outpost,
        "contract=DUNGEON-WORKFLOW-002 checkpoint=new-run expected Outpost"
    );
    let actor = player(&mut app);
    assert_eq!(
        app.world().get::<Position>(actor),
        Some(&bd_core::colony::shelter::SHELTER_RETURN_SPAWN),
        "contract=DUNGEON-WORKFLOW-002 checkpoint=new-run wrong shelter spawn"
    );
    let player_count = app
        .world_mut()
        .query_filtered::<Entity, With<Player>>()
        .iter(app.world())
        .count();
    assert_eq!(
        player_count, 1,
        "contract=DUNGEON-WORKFLOW-002 checkpoint=new-run expected one player"
    );
    assert_eq!(
        app.world()
            .resource::<bd_core::session::LastCompletedRun>()
            .outcome,
        bd_core::session::RunOutcome::Defeated,
        "contract=DUNGEON-WORKFLOW-002 checkpoint=new-run lost defeat history"
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
    place_named_survivor_at_resource_work_tile(&mut app, "Mara", ResourceNodeType::WaterSource);

    send_physical_key(&mut app, KeyCode::Char('c'));
    app.update();
    let survivor_key = named_survivor_menu_key(&app, "Mara");
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
        .find(|entry| entry.starts_with("Mara"))
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
        .find(|entry| entry.starts_with("Mara"))
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
    let survivor_key = named_survivor_menu_key(&app, "Mara");
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
    place_named_survivor_at_resource_work_tile(&mut app, "Iven", ResourceNodeType::WaterSource);

    send_physical_key(&mut app, KeyCode::Char('c'));
    app.update();
    let survivor_key = named_survivor_menu_key(&app, "Iven");
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
    let survivor = named_survivor(&mut app, "Tala");
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
    let survivor_key = named_survivor_menu_key(&app, "Tala");
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
        .find(|entry| entry.starts_with("Tala"))
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
    let selected_name = "Iven";

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
    assert_eq!(named_survivor_task(&mut app, "Mara"), SurvivorTask::Idle);
    assert_eq!(named_survivor_task(&mut app, "Tala"), SurvivorTask::Idle);
}

#[test]
fn task_confirmation_emits_one_named_target_and_activity_result() {
    // Contract: INPUT-MGMT-007
    // Given: Iven is selected for direct Supplies gathering.
    // When: the player confirms the paused management transaction.
    // Then: one result names survivor, task, Water target, and EnRoute activity.
    // Must not change: confirmation cannot emit duplicate decisive results.
    // Evidence layers: input state machine, projection, state diff, workflow.
    let mut app = outpost_runtime();
    let selected_name = "Iven";
    let log_count_before = app
        .world()
        .resource::<bd_core::gamelog::GameLog>()
        .iter()
        .count();

    send_physical_key(&mut app, KeyCode::Char('c'));
    app.update();
    let survivor_key = named_survivor_menu_key(&app, selected_name);
    send_physical_key(&mut app, survivor_key);
    app.update();
    let task_key = management_choice_key(&app, "Gather Supplies");
    send_physical_key(&mut app, task_key);
    app.update();
    send_physical_key(&mut app, KeyCode::Enter);
    app.update();
    app.update();

    let log = app.world().resource::<bd_core::gamelog::GameLog>();
    let new_messages = log
        .iter()
        .take(log.iter().count().saturating_sub(log_count_before))
        .map(|entry| entry.message.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        new_messages.len(),
        1,
        "contract=INPUT-MGMT-007 case=direct-gather-confirm \
         expected exactly one decisive assignment result; actual={new_messages:?}"
    );
    let result = &new_messages[0];
    for required in [selected_name, "Gather Supplies", "Water", "EnRoute"] {
        assert!(
            result.contains(required),
            "contract=INPUT-MGMT-007 case=direct-gather-confirm \
             missing semantic field `{required}`; result={result:?}"
        );
    }
}

#[test]
fn station_confirmation_emits_one_named_station_and_activity_result() {
    // Contract: INPUT-MGMT-008
    // Given: Iven and the built Stove are selected for staffing.
    // When: the player confirms the paused management transaction.
    // Then: one result names survivor, Stove, and EnRoute activity.
    // Must not change: confirmation cannot emit duplicate decisive results.
    // Evidence layers: input state machine, projection, state diff, workflow.
    let mut app = outpost_runtime();
    build_station(&mut app, StationType::Stove, Direction::East);
    let selected_name = "Iven";
    let log_count_before = app
        .world()
        .resource::<bd_core::gamelog::GameLog>()
        .iter()
        .count();

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

    let log = app.world().resource::<bd_core::gamelog::GameLog>();
    let new_messages = log
        .iter()
        .take(log.iter().count().saturating_sub(log_count_before))
        .map(|entry| entry.message.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        new_messages.len(),
        1,
        "contract=INPUT-MGMT-008 case=station-confirm \
         expected exactly one decisive staffing result; actual={new_messages:?}"
    );
    let result = &new_messages[0];
    for required in [selected_name, "Stove", "EnRoute"] {
        assert!(
            result.contains(required),
            "contract=INPUT-MGMT-008 case=station-confirm \
             missing semantic field `{required}`; result={result:?}"
        );
    }
}

#[test]
fn zero_supplies_overview_exposes_a_reachable_gathering_recovery_path() {
    // Contract: VISUAL-COLONY-STATE-001
    // Given: the colony has zero Supplies.
    // When: the overview and advertised task workflow are inspected.
    // Then: Travel explains its cost and Gather Supplies is reachable.
    // Must not change: unaffordable Travel cannot appear enabled.
    // Evidence layers: projection, input state machine, workflow.
    let mut app = outpost_runtime();
    app.world_mut()
        .resource_mut::<ColonyResources>()
        .pools
        .get_mut(PoolKind::Supplies)
        .expect("Foundation Supplies pool must exist")
        .current = 0;
    app.update();

    let actions = &app
        .world()
        .resource::<bd_tui::view_models::ActionListViewModel>()
        .actions;
    let travel = actions
        .iter()
        .find(|action| action.label == "Travel")
        .expect("zero-Supplies overview must keep dungeon entry discoverable");
    assert!(
        !travel.enabled && travel.denial_reason.as_deref() == Some("Need 2 Supplies"),
        "contract=VISUAL-COLONY-STATE-001 case=zero-supplies \
         expected a truthful travel denial; actual={travel:?}"
    );
    assert!(
        actions
            .iter()
            .any(|action| action.label == "Assign task" && action.enabled),
        "contract=VISUAL-COLONY-STATE-001 case=zero-supplies \
         expected enabled Assign Tasks recovery action; actions={actions:?}"
    );

    send_physical_key(&mut app, KeyCode::Char('c'));
    app.update();
    let management = app
        .world()
        .resource::<bd_tui::view_models::StatsViewModel>()
        .management
        .as_ref()
        .expect("the advertised recovery action must open task management");
    assert!(
        management
            .tasks
            .iter()
            .any(|task| task.contains("Gather Supplies")),
        "contract=VISUAL-COLONY-STATE-001 case=zero-supplies \
         task management did not expose Gather Supplies; tasks={:?}",
        management.tasks
    );
}

#[test]
fn processing_assignment_selects_named_survivor_station_and_recipe_while_paused() {
    let mut app = outpost_runtime();
    let selected_name = "Iven";
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
    let survivor_key = named_survivor_menu_key(&app, "Mara");
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
            entry.contains("Mara")
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

fn approach_target_from_two_tiles_away(
    app: &mut App,
    target: Position,
) -> (Position, Position, KeyCode) {
    let player_entity = player(app);
    let blockers = app
        .world_mut()
        .query_filtered::<(Entity, &Position), With<BlocksMovement>>()
        .iter(app.world())
        .filter_map(|(entity, position)| (entity != player_entity).then_some(*position))
        .collect::<HashSet<_>>();
    let map = &app.world().resource::<OutpostState>().map;
    [
        ((0, -1), KeyCode::Char('s')),
        ((0, 1), KeyCode::Char('w')),
        ((-1, 0), KeyCode::Char('d')),
        ((1, 0), KeyCode::Char('a')),
    ]
    .into_iter()
    .find_map(|((dx, dy), key)| {
        let adjacent = Position {
            x: target.x + dx,
            y: target.y + dy,
        };
        let start = Position {
            x: target.x + 2 * dx,
            y: target.y + 2 * dy,
        };
        (map.is_walkable(adjacent.x, adjacent.y)
            && map.is_walkable(start.x, start.y)
            && !blockers.contains(&adjacent)
            && !blockers.contains(&start))
        .then_some((start, adjacent, key))
    })
    .unwrap_or_else(|| panic!("fixture target at {target:?} needs a clear two-step approach"))
}

fn proximity_target(app: &mut App, case_id: &str) -> (Position, String) {
    if case_id == "water-node" {
        let mut matches = app
            .world_mut()
            .query::<(&Position, &ResourceNode)>()
            .iter(app.world())
            .filter_map(|(position, node)| {
                (node.kind == ResourceNodeType::WaterSource).then_some(*position)
            })
            .collect::<Vec<_>>();
        matches.sort_by_key(|position| (position.y, position.x));
        (
            *matches
                .first()
                .expect("Foundation fixture must contain Water Source"),
            "Water Source".to_owned(),
        )
    } else {
        let mut matches = app
            .world_mut()
            .query_filtered::<(&Position, &Name, &StationType), With<Station>>()
            .iter(app.world())
            .filter_map(|(position, name, station_type)| {
                (*station_type == StationType::Custom(1)).then_some((*position, name.0.clone()))
            })
            .collect::<Vec<_>>();
        matches.sort_by(|left, right| {
            (left.0.y, left.0.x, left.1.as_str()).cmp(&(right.0.y, right.0.x, right.1.as_str()))
        });
        matches
            .first()
            .cloned()
            .expect("Foundation fixture must contain Basic Processing")
    }
}

fn colony_pool_values(app: &App) -> Vec<(PoolKind, i32)> {
    app.world()
        .resource::<ColonyResources>()
        .pools
        .iter()
        .map(|pool| (pool.kind, pool.current))
        .collect()
}

#[test]
fn simultaneous_station_and_node_entry_emits_one_focused_nearby_fact_with_count() {
    // Contract: COLONY-PROXIMITY-001
    // Given: the player is one accepted cardinal movement away from entering the
    // interaction range of a named station or resource node.
    // When: that movement is submitted through the production physical-input path.
    // Then: exactly one new Chronicle fact names the target and makes the semantic
    // Interact action available; later render/projection frames do not duplicate it.
    // Must not change: proximity feedback does not alter colony pools, survivor
    // tasks, or the accepted movement destination.
    // Evidence layers: input state machine, state diff, projection, workflow; final
    // buffer and PTY remain required.
    //
    // Implementation guidance:
    // - Reusable owner: one stable nearby-interactable projection supplies Chronicle
    //   entry detection, passive target detail, and Interact availability.
    // - Integration seam: compare stable proximity before/after an accepted movement;
    //   rendering and view-model refreshes must remain read-only.
    // - Preserve: normal movement timing/cost, worker scheduling, pool values, stable
    //   station/node identity, existing management, and deterministic ordering.
    // - Invalid shortcuts: do not log from the renderer, pollute every movement with
    //   repeated prose, hardcode these coordinates/names, use ECS query order, emit
    //   one fact per simultaneously entered target, or present an unbound Interact
    //   row as executable.
    // - Closing evidence: rerun both target cases, neighboring movement/log/input
    //   tests, the final Chronicle/Context buffers, canonical gate, and PTY movement.
    // False-green challenge: two different targets enter range on the same
    // accepted movement. Historical feedback remains one focused fact with a
    // semantic count, while the current projection retains both targets. This
    // prevents a per-entity logging loop from passing the one-target cases.
    let mut app = outpost_runtime();
    let player_entity = player(&mut app);
    let occupied = app
        .world_mut()
        .query::<(Entity, &Position)>()
        .iter(app.world())
        .filter_map(|(entity, position)| (entity != player_entity).then_some(*position))
        .collect::<HashSet<_>>();
    let (start, destination, station_position, node_position) = {
        let map = &app.world().resource::<OutpostState>().map;
        (2..map.height - 2)
            .flat_map(|y| (2..map.width - 2).map(move |x| Position { x, y }))
            .find_map(|destination| {
                let start = Position {
                    x: destination.x - 1,
                    y: destination.y,
                };
                let station_position = Position {
                    x: destination.x,
                    y: destination.y - 1,
                };
                let node_position = Position {
                    x: destination.x,
                    y: destination.y + 1,
                };
                let clear = [
                    start,
                    destination,
                    station_position,
                    node_position,
                    Position {
                        x: destination.x + 1,
                        y: destination.y,
                    },
                ];
                clear
                    .iter()
                    .all(|position| {
                        map.is_walkable(position.x, position.y) && !occupied.contains(position)
                    })
                    .then_some((start, destination, station_position, node_position))
            })
            .expect("multi-target proximity fixture needs one clear cross")
    };
    app.world_mut().spawn((
        Station,
        StationType::Custom(1),
        Name("Alpha Relay".into()),
        station_position,
        BlocksMovement,
        EntityScope::ColonyPersistent,
    ));
    app.world_mut().spawn((
        ResourceNode {
            source_id: "source.water".into(),
            kind: ResourceNodeType::WaterSource,
            depleted: false,
        },
        Name("Water Source".into()),
        node_position,
        BlocksMovement,
        EntityScope::ColonyPersistent,
    ));
    app.world_mut().entity_mut(player_entity).insert(start);
    app.world_mut()
        .insert_resource(bd_core::colony::proximity::NearbyInteractables::default());
    app.world_mut()
        .insert_resource(bd_core::gamelog::GameLog::default());
    app.update();

    let log_count_before = app
        .world()
        .resource::<bd_core::gamelog::GameLog>()
        .iter()
        .count();
    send_physical_key(&mut app, KeyCode::Char('d'));
    app.update();
    app.update();

    assert_eq!(
        app.world().get::<Position>(player_entity),
        Some(&destination),
        "contract=COLONY-PROXIMITY-001 case=simultaneous-station-node \
             fixture=clear-cross precondition=start_{start:?} input=d frames_advanced=2 \
             expected=accepted_destination_{destination:?} actual={:?}",
        app.world().get::<Position>(player_entity)
    );
    let log = app.world().resource::<bd_core::gamelog::GameLog>();
    let new_entry_count = log.iter().count().saturating_sub(log_count_before);
    let nearby_facts = log
        .iter()
        .take(new_entry_count)
        .filter(|entry| entry.message.to_ascii_uppercase().contains("NEARBY"))
        .map(|entry| entry.message.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        nearby_facts.len(),
        1,
        "contract=COLONY-PROXIMITY-001 case=simultaneous-station-node \
             workflow_step=enter_two_target_range input=d frames_advanced=2 \
             expected=one_focused_NEARBY_fact_with_count actual={nearby_facts:?}"
    );
    let fact = &nearby_facts[0];
    let normalized = fact.to_ascii_lowercase();
    assert!(
        fact.contains("Alpha Relay")
            && (fact.contains("+1")
                || normalized.contains("1 more")
                || normalized.contains("2 nearby")
                || normalized.contains("2 targets")
                || normalized.contains("2 interactable")),
        "contract=COLONY-PROXIMITY-001 case=simultaneous-station-node \
             workflow_step=read_focused_fact expected=deterministic_Alpha_Relay_focus_and_semantic_count \
             actual_fact={fact:?}"
    );
    let targets = &app
        .world()
        .resource::<bd_core::colony::proximity::NearbyInteractables>()
        .targets;
    assert_eq!(
        targets.len(),
        2,
        "contract=COLONY-PROXIMITY-001 case=simultaneous-station-node \
             workflow_step=inspect_current_projection expected=complete_two_target_list \
             actual_targets={targets:?}"
    );
    assert_eq!(targets[0].name, "Alpha Relay");
    assert_eq!(targets[1].name, "Water Source");
}

fn assert_single_target_proximity_entry(case_id: &str) {
    let mut initialization_app = outpost_runtime();
    let (initialization_target, _) = proximity_target(&mut initialization_app, case_id);
    let (_, initialization_adjacent, _) =
        approach_target_from_two_tiles_away(&mut initialization_app, initialization_target);
    let initialization_player = player(&mut initialization_app);

    // False-green challenge: establishing an adjacent fixture and building
    // the initial proximity projection is not an accepted movement. It may
    // populate current tooltip/context state, but it must not write history.
    // This catches position-difference polling that mistakes initialization,
    // restoration, or a fixture relocation for a production movement result.
    initialization_app
        .world_mut()
        .entity_mut(initialization_player)
        .insert(initialization_adjacent);
    initialization_app
        .world_mut()
        .insert_resource(bd_core::colony::proximity::NearbyInteractables::default());
    let nearby_before_initialization = initialization_app
        .world()
        .resource::<bd_core::gamelog::GameLog>()
        .iter()
        .filter(|entry| entry.message.to_ascii_uppercase().contains("NEARBY"))
        .count();
    initialization_app.update();
    let nearby_after_initialization = initialization_app
        .world()
        .resource::<bd_core::gamelog::GameLog>()
        .iter()
        .filter(|entry| entry.message.to_ascii_uppercase().contains("NEARBY"))
        .count();
    assert_eq!(
        nearby_after_initialization, nearby_before_initialization,
        "contract=COLONY-PROXIMITY-001 case={case_id} fixture=adjacent-initialization \
             workflow_step=build_initial_projection input=none frames_advanced=1 \
             expected=no_historical_NEARBY_fact_without_accepted_movement \
             actual_before={nearby_before_initialization} \
             actual_after={nearby_after_initialization}"
    );

    let mut app = outpost_runtime();
    let (target, target_label) = proximity_target(&mut app, case_id);
    let (start, adjacent, movement_key) = approach_target_from_two_tiles_away(&mut app, target);
    let player_entity = player(&mut app);
    app.world_mut().entity_mut(player_entity).insert(start);
    app.update();

    let pools_before = colony_pool_values(&app);
    let tasks_before = survivor_tasks(&mut app);
    let log_count_before = app
        .world()
        .resource::<bd_core::gamelog::GameLog>()
        .iter()
        .count();

    send_physical_key(&mut app, movement_key);
    app.update();
    app.update();

    assert_eq!(
        app.world().get::<Position>(player_entity),
        Some(&adjacent),
        "contract=COLONY-PROXIMITY-001 case={case_id} fixture=two-step-approach \
             precondition=start_{start:?}_target_{target:?} input={movement_key:?} \
             expected=accepted_destination_{adjacent:?} actual={:?}",
        app.world().get::<Position>(player_entity)
    );
    let log = app.world().resource::<bd_core::gamelog::GameLog>();
    let new_entry_count = log.iter().count().saturating_sub(log_count_before);
    let nearby = log
        .iter()
        .take(new_entry_count)
        .filter(|entry| entry.message.to_ascii_uppercase().contains("NEARBY"))
        .map(|entry| entry.message.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        nearby.len(),
        1,
        "contract=COLONY-PROXIMITY-001 case={case_id} fixture=two-step-approach \
             workflow_step=enter_range input={movement_key:?} frames_advanced=2 \
             expected=one_NEARBY_fact actual={nearby:?} trace_tail=n/a replay_tail={:?}",
        action_ids(&app)
    );
    for required in [target_label.as_str(), "Interact"] {
        assert!(
            nearby[0].contains(required),
            "contract=COLONY-PROXIMITY-001 case={case_id} fixture=two-step-approach \
                 workflow_step=enter_range expected_field={required:?} \
                 actual_fact={:?}",
            nearby[0]
        );
    }
    let actions = &app
        .world()
        .resource::<bd_tui::view_models::ActionListViewModel>()
        .actions;
    let configured_interact_key = app
        .world()
        .resource::<bd_tui::commands::CommandBindings>()
        .key_for(bd_tui::commands::UiCommand::Interact)
        .map(bd_tui::commands::config_key_name);
    let interact = actions
        .iter()
        .find(|action| action.label == "Interact")
        .unwrap_or_else(|| {
            panic!(
                "contract=COLONY-PROXIMITY-001 case={case_id} fixture=two-step-approach \
                     workflow_step=inspect_available_actions expected=semantic_Interact_row \
                     actual_actions={actions:?}"
            )
        });
    match configured_interact_key {
        Some(expected_key) => assert!(
            interact.enabled
                && interact.key_hint == expected_key
                && interact.denial_reason.is_none(),
            "contract=COLONY-PROXIMITY-001 case={case_id} fixture=two-step-approach \
                 workflow_step=cross_check_bound_interact expected=enabled_binding_{expected_key:?} \
                 actual_interact={interact:?}"
        ),
        None => assert!(
            !interact.enabled
                && interact.key_hint == "unbound"
                && interact
                    .denial_reason
                    .as_deref()
                    .is_some_and(|reason| !reason.trim().is_empty()),
            "contract=COLONY-PROXIMITY-001 case={case_id} fixture=two-step-approach \
                 workflow_step=cross_check_unbound_interact \
                 expected=visible_but_disabled_Interact_with_truthful_reason \
                 actual_interact={interact:?}"
        ),
    }
    assert_eq!(colony_pool_values(&app), pools_before);
    assert_eq!(survivor_tasks(&mut app), tasks_before);

    let log_count_after_entry = app
        .world()
        .resource::<bd_core::gamelog::GameLog>()
        .iter()
        .count();
    app.update();
    app.update();
    let log_count_after_idle_frames = app
        .world()
        .resource::<bd_core::gamelog::GameLog>()
        .iter()
        .count();
    assert_eq!(
        log_count_after_idle_frames, log_count_after_entry,
        "contract=COLONY-PROXIMITY-001 case={case_id} fixture=two-step-approach \
             workflow_step=idle_projection_frames frames_advanced=2 \
             expected=no_duplicate_NEARBY_fact actual_log_count_before={} \
             actual_log_count_after={log_count_after_idle_frames}",
        log_count_after_entry
    );
}

#[test]
fn entering_adjacent_range_emits_one_deduplicated_nearby_hint() {
    // Primary contract: COLONY-PROXIMITY-001
    // Given/When/Then: one accepted production move enters station range and
    // creates one named Chronicle fact plus a truthful semantic Interact row.
    // Must not change: movement destination, pools, tasks, and idle-frame silence.
    // Implementation guidance: repair the shared nearby resolver/entry-feedback and
    // action-projection seams; do not log from rendering, hardcode the fixture, or
    // enable an unbound command. Close with every registered support, neighbors,
    // final buffers/PTY, and the canonical gate.
    assert_single_target_proximity_entry("processing-station");
}

#[test]
fn entering_water_node_range_emits_one_deduplicated_nearby_hint() {
    assert_single_target_proximity_entry("water-node");
}

#[test]
fn leaving_range_is_silent_and_reentry_emits_exactly_once_again() {
    // Supporting contract: COLONY-PROXIMITY-001
    // Given: an accepted move has already emitted the station's one entry fact.
    // When: production movement leaves cardinal range and later re-enters it.
    // Then: leaving is silent and re-entry emits exactly one fresh named fact.
    // Must not change: idle projection frames stay silent and the final current
    // target list contains the re-entered station once.
    // Evidence layers: production input, edge state, Chronicle, current projection.
    //
    // Implementation guidance:
    // - Reusable owner: the shared proximity edge tracks the current stable set,
    //   not an ever-seen identity set.
    // - Integration seam: accepted player movement rearms entry after a silent exit.
    // - Preserve: movement destination, deterministic identity, and render purity.
    // - Invalid shortcuts: one-shot lifetime deduplication or exit logging is not green.
    // - Closing evidence: run this independently with both single-target entries,
    //   simultaneous aggregation, neighboring input/log tests, gate, and PTY.
    let case_id = "processing-station-reentry";
    let mut app = outpost_runtime();
    let (target, target_label) = proximity_target(&mut app, "processing-station");
    let (start, adjacent, movement_key) = approach_target_from_two_tiles_away(&mut app, target);
    let return_key = match movement_key {
        KeyCode::Char('a') => KeyCode::Char('d'),
        KeyCode::Char('d') => KeyCode::Char('a'),
        KeyCode::Char('w') => KeyCode::Char('s'),
        KeyCode::Char('s') => KeyCode::Char('w'),
        unexpected => panic!(
            "contract=COLONY-PROXIMITY-001 case={case_id} fixture=reentry \
             precondition=cardinal_approach expected=movement_key actual={unexpected:?}"
        ),
    };
    let player_entity = player(&mut app);
    app.world_mut().entity_mut(player_entity).insert(start);
    app.world_mut()
        .insert_resource(bd_core::colony::proximity::NearbyInteractables::default());
    app.world_mut()
        .insert_resource(bd_core::gamelog::GameLog::default());
    app.update();

    send_physical_key(&mut app, movement_key);
    app.update();
    app.update();
    assert_eq!(app.world().get::<Position>(player_entity), Some(&adjacent));
    let after_first_entry = app
        .world()
        .resource::<bd_core::gamelog::GameLog>()
        .iter()
        .filter(|entry| entry.message.to_ascii_uppercase().contains("NEARBY"))
        .count();
    assert_eq!(
        after_first_entry, 1,
        "contract=COLONY-PROXIMITY-001 case={case_id} fixture=reentry \
         workflow_step=first_entry expected=one_NEARBY_fact actual={after_first_entry}"
    );

    send_physical_key(&mut app, return_key);
    app.update();
    app.update();
    assert_eq!(app.world().get::<Position>(player_entity), Some(&start));
    let after_exit = app
        .world()
        .resource::<bd_core::gamelog::GameLog>()
        .iter()
        .filter(|entry| entry.message.to_ascii_uppercase().contains("NEARBY"))
        .count();
    assert_eq!(
        after_exit, after_first_entry,
        "contract=COLONY-PROXIMITY-001 case={case_id} fixture=reentry \
         workflow_step=leave_range expected=silent_exit actual_before={after_first_entry} \
         actual_after={after_exit}"
    );

    send_physical_key(&mut app, movement_key);
    app.update();
    app.update();
    assert_eq!(app.world().get::<Position>(player_entity), Some(&adjacent));
    let facts = app
        .world()
        .resource::<bd_core::gamelog::GameLog>()
        .iter()
        .filter(|entry| entry.message.to_ascii_uppercase().contains("NEARBY"))
        .map(|entry| entry.message.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        facts.len(),
        2,
        "contract=COLONY-PROXIMITY-001 case={case_id} fixture=reentry \
         workflow_step=reenter_range expected=exactly_one_fresh_NEARBY_fact \
         actual_facts={facts:?}"
    );
    assert!(facts.iter().all(|fact| fact.contains(&target_label)));
    let targets = &app
        .world()
        .resource::<bd_core::colony::proximity::NearbyInteractables>()
        .targets;
    assert_eq!(
        targets.len(),
        1,
        "contract=COLONY-PROXIMITY-001 case={case_id} fixture=reentry \
         workflow_step=inspect_current_projection expected=one_reentered_target \
         actual_targets={targets:?}"
    );
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
    let survivor_key = named_survivor_menu_key(&app, "Mara");
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

#[test]
fn assigned_worker_row_names_target_and_numeric_distance() {
    // Contract: VISUAL-COLONY-WORK-006
    // Given: a named survivor receives a direct-gather assignment while away
    // from the matching fixture.
    // When: the production colony projection is rebuilt.
    // Then: the row names the target and its current numeric tile distance.
    // Must not change: assignment remains paused and does not move the worker.
    let mut app = outpost_runtime();
    let survivor = named_survivor(&mut app, "Mara");
    let position_before = *app
        .world()
        .get::<Position>(survivor)
        .expect("named survivor must have a position");
    let water_position = {
        let mut nodes = app.world_mut().query::<(
            &ResourceNode,
            &Position,
            Option<&bd_core::spatial::EntityScope>,
        )>();
        nodes
            .iter(app.world())
            .find_map(|(node, position, scope)| {
                (node.kind == ResourceNodeType::WaterSource
                    && scope.is_some_and(|scope| scope.is_active(GameMode::Outpost)))
                .then_some(*position)
            })
            .expect("Foundation shelter must contain a Water Source")
    };
    let expected_distance = (water_position.x - position_before.x).unsigned_abs()
        + (water_position.y - position_before.y).unsigned_abs();

    send_physical_key(&mut app, KeyCode::Char('c'));
    app.update();
    let survivor_key = named_survivor_menu_key(&app, "Mara");
    send_physical_key(&mut app, survivor_key);
    app.update();
    let gather_key = management_choice_key(&app, "Gather Supplies");
    send_physical_key(&mut app, gather_key);
    app.update();
    send_physical_key(&mut app, KeyCode::Enter);
    app.update();
    app.update();

    assert_eq!(
        app.world()
            .get::<Position>(survivor)
            .copied()
            .expect("assigned survivor must retain a position"),
        position_before,
        "contract=VISUAL-COLONY-WORK-006 assignment moved the survivor during paused UI"
    );
    let row = app
        .world()
        .resource::<bd_tui::view_models::StatsViewModel>()
        .party_names
        .iter()
        .find(|entry| entry.starts_with("Mara"))
        .cloned()
        .expect("assigned survivor must remain visible");
    let distance_text = format!("{expected_distance} tiles");
    assert!(
        row.contains("Water") && row.contains(&distance_text),
        "contract=VISUAL-COLONY-WORK-006 fixture=assigned-water-target \
         expected target `Water` and distance `{distance_text}`; \
         survivor={position_before:?}, target={water_position:?}, actual={row:?}"
    );
}
