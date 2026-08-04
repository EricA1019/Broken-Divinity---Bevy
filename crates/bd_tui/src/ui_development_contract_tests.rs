//! Red-first player-facing contracts for the Foundation UI improvement plan.
//!
//! These tests deliberately separate layout, semantic content, resolved
//! style, and transition evidence. A failing test in this module describes a
//! UI development target; it must not be weakened merely to restore a green
//! aggregate test count.

use super::*;
use bd_core::{
    colony::{
        production::ColonyResources,
        stations::{ConstructionSite, Station, StationType},
        survivors::Survivor,
    },
    components::{BlocksMovement, Name, Player, Position, ResourceNode, ResourceNodeType, Tile},
    gamelog::{GameLog, LogLevel},
    signals::PoolKind,
    spatial::{EntityScope, GameMode, TransitionIntent},
};
use bevy_app::App;
use bevy_ecs::{
    entity::Entity,
    message::Messages,
    prelude::{IntoScheduleConfigs, Res, ResMut, Resource},
    query::With,
};
use bevy_ratatui::{
    crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    event::KeyMessage,
};
use ratatui::{
    Terminal,
    backend::TestBackend,
    buffer::Buffer,
    layout::{Rect, Size},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Widget},
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct CellObservation {
    x: u16,
    y: u16,
    symbol: String,
    foreground: Color,
    background: Color,
    modifier: Modifier,
}

type ContextDetailClaim<'a> = (&'a str, &'a [&'a str]);

fn observe(buffer: &Buffer) -> Vec<CellObservation> {
    let area = buffer.area;
    (area.y..area.y + area.height)
        .flat_map(|y| {
            (area.x..area.x + area.width).map(move |x| {
                let cell = buffer
                    .cell((x, y))
                    .expect("observation coordinate must be inside the buffer");
                CellObservation {
                    x,
                    y,
                    symbol: cell.symbol().to_owned(),
                    foreground: cell.fg,
                    background: cell.bg,
                    modifier: cell.modifier,
                }
            })
        })
        .collect()
}

fn frame_cell_violations(
    buffer: &Buffer,
    position: (u16, u16),
    allowed_glyphs: &[&str],
    expected_foreground: Color,
) -> Vec<String> {
    let cell = buffer
        .cell(position)
        .expect("frame coordinate must be inside the buffer");
    let mut violations = Vec::new();

    if !allowed_glyphs.contains(&cell.symbol()) {
        violations.push(format!(
            "position={position:?} expected_glyph={allowed_glyphs:?} actual_glyph={:?}",
            cell.symbol()
        ));
    }
    if cell.fg != expected_foreground {
        violations.push(format!(
            "position={position:?} expected_fg={expected_foreground:?} actual_fg={:?}",
            cell.fg
        ));
    }

    violations
}

fn closed_double_frame_violations(buffer: &Buffer, expected_foreground: Color) -> Vec<String> {
    let area = buffer.area;
    if area.width < 2 || area.height < 2 {
        return vec![format!(
            "expected_terminal_perimeter=at_least_2x2 actual={:?}",
            area.as_size()
        )];
    }

    let left = area.x;
    let right = area.x + area.width - 1;
    let top = area.y;
    let bottom = area.y + area.height - 1;
    let mut violations = Vec::new();

    for (position, glyph) in [
        ((left, top), "╔"),
        ((right, top), "╗"),
        ((left, bottom), "╚"),
        ((right, bottom), "╝"),
    ] {
        violations.extend(frame_cell_violations(
            buffer,
            position,
            &[glyph],
            expected_foreground,
        ));
    }
    for x in left + 1..right {
        violations.extend(frame_cell_violations(
            buffer,
            (x, top),
            &["═"],
            expected_foreground,
        ));
        violations.extend(frame_cell_violations(
            buffer,
            (x, bottom),
            &["═"],
            expected_foreground,
        ));
    }
    for y in top + 1..bottom {
        violations.extend(frame_cell_violations(
            buffer,
            (left, y),
            &["║", "╠"],
            expected_foreground,
        ));
        violations.extend(frame_cell_violations(
            buffer,
            (right, y),
            &["║", "╣"],
            expected_foreground,
        ));
    }

    violations
}

fn ruined_reliquary_panel_violations(
    buffer: &Buffer,
    area: Rect,
    expected_border: Color,
    expected_title: Color,
) -> Vec<String> {
    if area.width < 3 || area.height < 2 {
        return vec![format!(
            "expected_panel=at_least_3x2 actual={:?}",
            area.as_size()
        )];
    }

    let left = area.x;
    let right = area.x + area.width - 1;
    let top = area.y;
    let bottom = area.y + area.height - 1;
    let mut violations = Vec::new();

    for (position, glyph) in [
        ((left, top), "┌"),
        ((right, top), "┐"),
        ((left, bottom), "└"),
        ((right, bottom), "┘"),
    ] {
        violations.extend(frame_cell_violations(
            buffer,
            position,
            &[glyph],
            expected_border,
        ));
    }
    for x in left + 1..right {
        violations.extend(frame_cell_violations(
            buffer,
            (x, bottom),
            &["─"],
            expected_border,
        ));
        let cell = buffer
            .cell((x, top))
            .expect("panel top edge must be inside the buffer");
        let is_rule = cell.symbol() == "─" && cell.fg == expected_border;
        let is_title = cell.fg == expected_title && cell.modifier.contains(Modifier::BOLD);
        if !is_rule && !is_title {
            violations.push(format!(
                "position=({x}, {top}) expected=muted_single_rule_or_bold_title \
                 actual_glyph={:?} actual_fg={:?} actual_modifier={:?}",
                cell.symbol(),
                cell.fg,
                cell.modifier
            ));
        }
    }
    for y in top + 1..bottom {
        violations.extend(frame_cell_violations(
            buffer,
            (left, y),
            &["│"],
            expected_border,
        ));
        violations.extend(frame_cell_violations(
            buffer,
            (right, y),
            &["│"],
            expected_border,
        ));
    }

    let title_markers = (left + 1..right)
        .filter_map(|x| buffer.cell((x, top)))
        .filter(|cell| {
            cell.symbol() == "◆"
                && cell.fg == expected_title
                && cell.modifier.contains(Modifier::BOLD)
        })
        .count();
    if title_markers != 1 {
        violations.push(format!(
            "expected=one_semantic_title_marker actual={title_markers}"
        ));
    }

    violations
}

fn ascii_meter_violations(
    buffer: &Buffer,
    area: Rect,
    label: &str,
    value: &str,
    expected_fill: Color,
    expected_empty: Color,
) -> Vec<String> {
    for y in area.y..area.y + area.height {
        let cells = (area.x..area.x + area.width)
            .map(|x| {
                buffer
                    .cell((x, y))
                    .expect("meter coordinate must be inside the buffer")
            })
            .collect::<Vec<_>>();
        let row = cells.iter().map(|cell| cell.symbol()).collect::<String>();
        if !row.contains(label) || !row.contains(value) {
            continue;
        }
        let Some(open) = cells.iter().position(|cell| cell.symbol() == "[") else {
            return vec![format!(
                "row={y} expected=opening_track_delimiter actual_row={row:?}"
            )];
        };
        let Some(close_offset) = cells[open + 1..]
            .iter()
            .position(|cell| cell.symbol() == "]")
        else {
            return vec![format!(
                "row={y} expected=closing_track_delimiter actual_row={row:?}"
            )];
        };
        let close = open + 1 + close_offset;
        let mut fill_count = 0;
        let mut empty_count = 0;
        let mut violations = Vec::new();
        for offset in open + 1..close {
            let x = area.x + offset as u16;
            let cell = buffer
                .cell((x, y))
                .expect("meter track coordinate must be inside the buffer");
            match cell.symbol() {
                "#" => {
                    fill_count += 1;
                    if cell.fg != expected_fill {
                        violations.push(format!(
                            "position=({x}, {y}) expected_filled_fg={expected_fill:?} \
                             actual_fg={:?}",
                            cell.fg
                        ));
                    }
                }
                "-" => {
                    empty_count += 1;
                    if cell.fg != expected_empty {
                        violations.push(format!(
                            "position=({x}, {y}) expected_empty_fg={expected_empty:?} \
                             actual_fg={:?}",
                            cell.fg
                        ));
                    }
                }
                actual => violations.push(format!(
                    "position=({x}, {y}) expected_track_glyph=['#','-'] actual={actual:?}"
                )),
            }
        }
        if fill_count == 0 || empty_count == 0 {
            violations.push(format!(
                "row={y} expected=partial_track_with_fill_and_remainder \
                 actual_fill={fill_count} actual_empty={empty_count}"
            ));
        }
        return violations;
    }

    vec![format!(
        "expected=meter_row_with_label_{label:?}_and_value_{value:?} actual=missing"
    )]
}

fn footer_key_chip_count(buffer: &Buffer, expected_key_color: Color) -> usize {
    let area = buffer.area;
    let first_footer_y = area.y + area.height.saturating_sub(3);
    (first_footer_y..area.y + area.height)
        .flat_map(|y| (area.x..area.x + area.width).map(move |x| (x, y)))
        .filter(|&(x, y)| {
            let cell = buffer
                .cell((x, y))
                .expect("footer coordinate must be inside the buffer");
            cell.symbol() == "[" && cell.fg == expected_key_color
        })
        .count()
}

fn frame_edge_crop(buffer: &Buffer) -> String {
    let text = buffer_text(buffer);
    let lines = text.lines().collect::<Vec<_>>();
    if lines.len() <= 8 {
        return text;
    }

    lines
        .iter()
        .take(3)
        .copied()
        .chain(std::iter::once("…"))
        .chain(lines.iter().skip(lines.len() - 4).copied())
        .collect::<Vec<_>>()
        .join("\n")
}

fn rect_contains(rect: Rect, x: u16, y: u16) -> bool {
    x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
}

fn shelter_map() -> MapViewModel {
    let width = 40;
    let height = 30;
    MapViewModel {
        width,
        height,
        tiles: vec![Tile::Floor; (width * height) as usize],
        player_pos: Some(Position { x: 1, y: 1 }),
        visuals: vec![view_models::MapVisualVm {
            position: Position { x: 1, y: 1 },
            token: visual::VisualToken::Player,
            glyph: None,
        }],
        ..Default::default()
    }
}

fn production_outpost_runtime() -> App {
    let mut app = App::new();
    app.add_plugins(bd_core::BdFoundationPlugin);
    let content = bd_test_support::foundation_content();
    app.insert_resource(bd_core::colony::stations::StationCatalog::new(
        content.stations.clone(),
    ));
    app.insert_resource(content);
    app.add_plugins(BdTuiPlugin);
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

fn send_context_key(app: &mut App, key: KeyCode) {
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

fn advance_context_key(app: &mut App, key: KeyCode) {
    send_context_key(app, key);
    app.update();
    app.update();
}

fn context_named_survivor_key(app: &App, expected_name: &str) -> KeyCode {
    let menu = app
        .world()
        .resource::<StatsViewModel>()
        .management
        .as_ref()
        .expect("context production workflow must project management");
    let index = menu
        .survivors
        .iter()
        .position(|entry| entry.starts_with(expected_name))
        .unwrap_or_else(|| panic!("management must list survivor {expected_name:?}"));
    KeyCode::Char(char::from(
        b'1' + u8::try_from(index).expect("management index must fit a key"),
    ))
}

fn context_management_choice_key(app: &App, expected_label: &str) -> KeyCode {
    let menu = app
        .world()
        .resource::<StatsViewModel>()
        .management
        .as_ref()
        .expect("context production workflow must project management");
    let index = menu
        .tasks
        .iter()
        .position(|entry| entry.contains(expected_label))
        .unwrap_or_else(|| panic!("management must list choice {expected_label:?}"));
    KeyCode::Char(char::from(
        b'1' + u8::try_from(index).expect("management index must fit a key"),
    ))
}

fn context_named_survivor(app: &mut App, expected_name: &str) -> Entity {
    app.world_mut()
        .query_filtered::<(Entity, &Name), With<Survivor>>()
        .iter(app.world())
        .find_map(|(entity, name)| (name.0 == expected_name).then_some(entity))
        .unwrap_or_else(|| panic!("Foundation fixture must contain {expected_name:?}"))
}

fn place_context_survivor_at_water_work_tile(app: &mut App, survivor_name: &str) -> Entity {
    let survivor = context_named_survivor(app, survivor_name);
    let nodes = app
        .world_mut()
        .query::<(&Position, &ResourceNode)>()
        .iter(app.world())
        .map(|(position, node)| (*position, node.kind))
        .collect::<Vec<_>>();
    let target = nodes
        .iter()
        .find_map(|(position, kind)| (*kind == ResourceNodeType::WaterSource).then_some(*position))
        .expect("production gather fixture must contain Water Source");
    let mut occupied = app
        .world_mut()
        .query::<(Entity, &Position)>()
        .iter(app.world())
        .filter_map(|(entity, position)| (entity != survivor).then_some(*position))
        .collect::<Vec<_>>();
    occupied.extend(nodes.iter().map(|(position, _)| *position));
    let map = &app.world().resource::<bd_core::spatial::OutpostState>().map;
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
    .expect("production gather fixture needs a free Water Source work tile");
    app.world_mut().entity_mut(survivor).insert(work_position);
    survivor
}

fn assign_context_gathering_through_input(app: &mut App, survivor_name: &str) -> Entity {
    let survivor = place_context_survivor_at_water_work_tile(app, survivor_name);
    advance_context_key(app, KeyCode::Char('c'));
    let survivor_key = context_named_survivor_key(app, survivor_name);
    advance_context_key(app, survivor_key);
    let task_key = context_management_choice_key(app, "Gather Supplies");
    advance_context_key(app, task_key);
    advance_context_key(app, KeyCode::Enter);
    survivor
}

fn establish_context_gather_progress(app: &mut App, survivor_name: &str) -> Entity {
    let survivor = assign_context_gathering_through_input(app, survivor_name);
    for _ in 0..8 {
        if app
            .world()
            .get::<bd_core::colony::resources::DirectGatherProgress>(survivor)
            .is_some_and(|progress| progress.work_completed == 1)
        {
            return survivor;
        }
        advance_context_key(app, KeyCode::Char('.'));
    }
    panic!("production gather workflow did not reach 1/3 progress")
}

fn assign_context_recipe_through_input(
    app: &mut App,
    survivor_name: &str,
    recipe_label: &str,
) -> Entity {
    let survivor = context_named_survivor(app, survivor_name);
    advance_context_key(app, KeyCode::Char('e'));
    let survivor_key = context_named_survivor_key(app, survivor_name);
    advance_context_key(app, survivor_key);
    let station_key = context_management_choice_key(app, "Basic Processing");
    advance_context_key(app, station_key);
    let recipe_key = context_management_choice_key(app, recipe_label);
    advance_context_key(app, recipe_key);
    advance_context_key(app, KeyCode::Enter);
    assert!(
        app.world()
            .get::<bd_core::colony::logistics::LogisticsJob>(survivor)
            .is_some(),
        "production recipe input must create a durable logistics job"
    );
    survivor
}

fn establish_context_recipe_progress(app: &mut App, survivor_name: &str) -> Entity {
    let survivor = assign_context_recipe_through_input(app, survivor_name, "Refine Water");
    for _ in 0..200 {
        if app
            .world()
            .get::<bd_core::colony::logistics::LogisticsJob>(survivor)
            .is_some_and(|job| {
                job.stage == bd_core::colony::logistics::JobStage::ReadyToRefine
                    && job.work_completed == 1
            })
        {
            return survivor;
        }
        advance_context_key(app, KeyCode::Char('.'));
    }
    panic!("production recipe workflow did not reach Refine Water 1/2")
}

fn establish_context_carrying(app: &mut App, survivor_name: &str) -> Entity {
    let survivor = assign_context_recipe_through_input(app, survivor_name, "Refine Water");
    for _ in 0..200 {
        let carrying = app
            .world()
            .get::<bd_core::colony::logistics::LogisticsJob>(survivor)
            .is_some_and(|job| job.stage == bd_core::colony::logistics::JobStage::ToStation)
            && app
                .world()
                .get::<bd_core::colony::logistics::Cargo>(survivor)
                .is_some_and(|cargo| cargo.amount == 1);
        if carrying {
            return survivor;
        }
        advance_context_key(app, KeyCode::Char('.'));
    }
    panic!("production recipe workflow did not reach carrying-to-station state")
}

fn establish_context_blocked(app: &mut App, survivor_name: &str) -> Entity {
    let survivor = assign_context_recipe_through_input(app, survivor_name, "Refine Water");
    let water_sources = app
        .world_mut()
        .query::<(Entity, &ResourceNode)>()
        .iter(app.world())
        .filter_map(|(entity, node)| (node.source_id == "source.water").then_some(entity))
        .collect::<Vec<_>>();
    assert!(
        !water_sources.is_empty(),
        "blocked production fixture must begin with a Water Source"
    );
    for source in water_sources {
        app.world_mut().entity_mut(source).despawn();
    }
    advance_context_key(app, KeyCode::Char('.'));
    assert!(
        app.world()
            .get::<bd_core::colony::logistics::LogisticsJob>(survivor)
            .is_some_and(|job| {
                job.blocked == Some(bd_core::colony::logistics::LogisticsBlock::MissingSource)
            })
            && matches!(
                app.world()
                    .get::<bd_core::colony::survivors::WorkerActivity>(survivor),
                Some(bd_core::colony::survivors::WorkerActivity::Blocked { .. })
            ),
        "normal logistics update must create the blocked activity"
    );
    survivor
}

#[derive(Resource)]
struct SharedContextDetailProbe;

fn apply_shared_context_detail_probe(
    _probe: Res<SharedContextDetailProbe>,
    mut nearby: ResMut<bd_core::colony::proximity::NearbyInteractables>,
) {
    let Some(target) = nearby
        .targets
        .iter_mut()
        .find(|target| target.name == "Basic Processing")
    else {
        return;
    };
    target.detail = "Station · Unstaffed · Operational · Shared Detail Probe".into();
    target.worker = Some("Forbidden Parallel Worker".into());
    target.recipe = Some("Forbidden Parallel Recipe".into());
    target.progress = Some("99/99".into());
}

fn production_context_fixture(
    case_id: &str,
) -> (
    String,
    String,
    String,
    MapViewModel,
    StatsViewModel,
    LogViewModel,
    ActionListViewModel,
    Vec<bd_core::colony::proximity::NearbyTarget>,
) {
    let mut app = production_outpost_runtime();
    if case_id == "station-shared-detail" {
        app.insert_resource(SharedContextDetailProbe);
        app.add_systems(
            bevy_app::Update,
            apply_shared_context_detail_probe
                .after(bd_core::BdSet::ResultEmission)
                .before(bd_core::BdSet::ViewModelBuild),
        );
    }
    let target = match case_id {
        "station"
        | "station-construction"
        | "station-staffed"
        | "station-bound-interact"
        | "station-shared-detail" => {
            let mut matches = app
                .world_mut()
                .query_filtered::<(Entity, &Name, &Position, &StationType), With<Station>>()
                .iter(app.world())
                .filter_map(|(entity, name, position, station_type)| {
                    (*station_type == StationType::Custom(1)).then_some((
                        name.0.clone(),
                        position.y,
                        position.x,
                        entity,
                    ))
                })
                .collect::<Vec<_>>();
            matches.sort_by(|left, right| {
                (left.0.as_str(), left.1, left.2).cmp(&(right.0.as_str(), right.1, right.2))
            });
            matches
                .first()
                .map(|(_, _, _, entity)| *entity)
                .expect("Foundation fixture must contain Basic Processing")
        }
        "node" | "node-depleted" | "node-assigned" => {
            let mut matches = app
                .world_mut()
                .query::<(Entity, &Position, &ResourceNode)>()
                .iter(app.world())
                .filter_map(|(entity, position, node)| {
                    (node.kind == ResourceNodeType::WaterSource)
                        .then_some((position.y, position.x, entity))
                })
                .collect::<Vec<_>>();
            matches.sort_by_key(|(y, x, _)| (*y, *x));
            matches
                .first()
                .map(|(_, _, entity)| *entity)
                .expect("Foundation fixture must contain Water Source")
        }
        "colonist" | "colonist-assigned" | "colonist-carrying" | "colonist-blocked" => {
            let mut matches = app
                .world_mut()
                .query_filtered::<(Entity, &Name, &Position), With<Survivor>>()
                .iter(app.world())
                .filter_map(|(entity, name, position)| {
                    (name.0 == "Mara").then_some((position.y, position.x, entity))
                })
                .collect::<Vec<_>>();
            matches.sort_by_key(|(y, x, _)| (*y, *x));
            matches
                .first()
                .map(|(_, _, entity)| *entity)
                .expect("Foundation fixture must contain Mara")
        }
        unknown => panic!("unknown context fixture category: {unknown}"),
    };
    match case_id {
        "station-construction" => {
            app.world_mut().entity_mut(target).insert(ConstructionSite {
                work_completed: 1,
                work_required: 4,
            });
        }
        "node-depleted" => {
            app.world_mut()
                .get_mut::<ResourceNode>(target)
                .expect("depleted-node fixture target must be a resource node")
                .depleted = true;
        }
        _ => {}
    }
    if case_id == "station-bound-interact" {
        app.world_mut()
            .resource_mut::<commands::CommandBindings>()
            .bind(commands::UiCommand::Interact, KeyCode::Char('x'));
    }
    let player = app
        .world_mut()
        .query_filtered::<Entity, With<Player>>()
        .iter(app.world())
        .next()
        .expect("Foundation player must exist");
    let occupied = app
        .world_mut()
        .query::<(Entity, &Position)>()
        .iter(app.world())
        .filter_map(|(entity, position)| {
            (entity != target && entity != player).then_some(*position)
        })
        .collect::<Vec<_>>();
    let map = &app.world().resource::<bd_core::spatial::OutpostState>().map;
    let (target_position, adjacent_position, start_position) = (1..map.height - 1)
        .flat_map(|y| (1..map.width - 3).map(move |x| (x, y)))
        .find_map(|(x, y)| {
            let target_position = Position { x, y };
            let adjacent_position = Position { x: x + 1, y };
            let start_position = Position { x: x + 2, y };
            (map.is_walkable(target_position.x, target_position.y)
                && map.is_walkable(adjacent_position.x, adjacent_position.y)
                && map.is_walkable(start_position.x, start_position.y)
                && !occupied.iter().any(|position| {
                    *position == target_position
                        || *position == adjacent_position
                        || *position == start_position
                        || (position.x - adjacent_position.x).unsigned_abs()
                            + (position.y - adjacent_position.y).unsigned_abs()
                            <= 1
                }))
            .then_some((target_position, adjacent_position, start_position))
        })
        .expect("context fixture needs one isolated two-step approach");
    app.world_mut().entity_mut(target).insert(target_position);
    app.world_mut().entity_mut(player).insert(start_position);
    app.world_mut()
        .insert_resource(bd_core::colony::proximity::NearbyInteractables::default());
    app.world_mut().insert_resource(GameLog::default());
    app.update();
    {
        let mut messages = app.world_mut().resource_mut::<Messages<KeyMessage>>();
        messages.write(KeyMessage(KeyEvent::new_with_kind(
            KeyCode::Char('a'),
            KeyModifiers::NONE,
            KeyEventKind::Press,
        )));
        messages.write(KeyMessage(KeyEvent::new_with_kind(
            KeyCode::Char('a'),
            KeyModifiers::NONE,
            KeyEventKind::Release,
        )));
    }
    app.update();
    app.update();
    assert_eq!(
        app.world().get::<Position>(player),
        Some(&adjacent_position),
        "contract=VISUAL-CONTEXT-001 case={case_id} fixture=production-two-step-approach \
         workflow_step=enter_range input=a frames_advanced=2 \
         expected=accepted_destination_{adjacent_position:?} actual={:?}",
        app.world().get::<Position>(player)
    );

    // Active variants are produced through the real paused management inputs
    // and later time-advancing worker ticks. Colonists are repositioned only
    // after the decisive domain state exists so geometry setup cannot stand in
    // for production reachability.
    match case_id {
        "station-staffed" => {
            establish_context_recipe_progress(&mut app, "Mara");
        }
        "node-assigned" => {
            establish_context_gather_progress(&mut app, "Mara");
        }
        "colonist-assigned" => {
            establish_context_gather_progress(&mut app, "Mara");
            app.world_mut().entity_mut(target).insert(target_position);
            app.update();
        }
        "colonist-carrying" => {
            establish_context_carrying(&mut app, "Mara");
            app.world_mut().entity_mut(target).insert(target_position);
            app.update();
        }
        "colonist-blocked" => {
            establish_context_blocked(&mut app, "Mara");
            app.world_mut().entity_mut(target).insert(target_position);
            app.update();
        }
        _ => {}
    }

    let (target_name, category, status) = match case_id {
        "station" | "station-bound-interact" | "station-shared-detail" => {
            ("Basic Processing", "Station", "Unstaffed")
        }
        "station-staffed" => ("Basic Processing", "Station", "Staffed"),
        "station-construction" => ("Basic Processing", "Construction", "1/4"),
        "node" | "node-depleted" | "node-assigned" => ("Water Source", "Resource Node", "Supplies"),
        "colonist" | "colonist-carrying" => ("Mara", "Colonist", "Idle"),
        "colonist-assigned" => ("Mara", "Colonist", "Gathering"),
        "colonist-blocked" => ("Mara", "Colonist", "Blocked"),
        _ => unreachable!(),
    };
    (
        target_name.into(),
        category.into(),
        status.into(),
        app.world().resource::<MapViewModel>().clone(),
        app.world().resource::<StatsViewModel>().clone(),
        app.world().resource::<LogViewModel>().clone(),
        app.world().resource::<ActionListViewModel>().clone(),
        app.world()
            .resource::<bd_core::colony::proximity::NearbyInteractables>()
            .targets
            .clone(),
    )
}

fn assert_context_state_variant(
    case_id: &str,
    required_details: &[&str],
    forbidden_actions: &[&str],
) {
    // Supporting contract: VISUAL-CONTEXT-001
    // Given: the same production adapters observe a construction site and a
    // depleted resource node rather than their ordinary operational/renewable states.
    // When: a real accepted move builds the nearby and Context projections.
    // Then: detail changes with authoritative state and invalid category actions are
    // absent rather than inherited from a hardcoded category menu.
    // Must not change: both targets remain identifiable and Inspect remains present.
    // Evidence layers: production projection and input-state presentation.
    //
    // Implementation guidance:
    // - Reusable owner: category adapters read domain/catalog facts into structured
    //   detail and applicability; renderers do not append stock category prose.
    // - Integration seam: target state must survive through the focused Context
    //   projection and its ordered action set.
    // - Preserve: construction and depletion semantics, stable identity, and the
    //   disabled pre-UI9-D preview policy.
    // - Invalid shortcuts: hardcoding Operational/Renewable by category or showing
    //   Assign/Production actions for an inapplicable target state is not green.
    // - Closing evidence: run this state matrix with the category/profile primary,
    //   action-truth and duplicate-name support tests, canonical gate, and PTY.
    let (target, _, _, _, stats, _, actions, nearby) = production_context_fixture(case_id);
    let projected_target = nearby
        .iter()
        .find(|candidate| candidate.name == target)
        .expect("state-variant target must remain in the nearby projection");
    let details =
        format!("{} {}", projected_target.status, projected_target.detail).to_ascii_lowercase();
    for required in required_details {
        assert!(
            details.contains(required),
            "contract=VISUAL-CONTEXT-001 case={case_id} fixture=state-variant \
                 workflow_step=inspect_authoritative_detail expected={required:?} \
                 actual_target={projected_target:?}"
        );
    }
    assert!(
        actions
            .actions
            .iter()
            .any(|action| action.label.contains("Inspect") && action.label.contains(&target)),
        "contract=VISUAL-CONTEXT-001 case={case_id} fixture=state-variant \
             workflow_step=inspect_context_actions expected=Inspect_{target:?} \
             actual_actions={:?}",
        actions.actions
    );
    for forbidden in forbidden_actions {
        assert!(
            !actions
                .actions
                .iter()
                .any(|action| action.label.contains(forbidden)),
            "contract=VISUAL-CONTEXT-001 case={case_id} fixture=state-variant \
                 workflow_step=inspect_applicability forbidden_action={forbidden:?} \
                 actual_actions={:?}",
            actions.actions
        );
    }
    assert_eq!(
        stats
            .context_target
            .as_ref()
            .map(|target| target.name.as_str()),
        Some(target.as_str())
    );
}

#[test]
fn context_detail_and_actions_follow_authoritative_target_state() {
    assert_context_state_variant(
        "station-construction",
        &["construction", "1/4"],
        &["Assign Worker", "Set Production"],
    );
}

#[test]
fn depleted_node_context_changes_detail_and_action_applicability() {
    assert_context_state_variant("node-depleted", &["depleted"], &["Assign Gatherer"]);
}

#[allow(clippy::too_many_arguments)]
fn render_buffer(
    screen: &str,
    width: u16,
    height: u16,
    mode: GameMode,
    map: &MapViewModel,
    stats: &StatsViewModel,
    log: &LogViewModel,
    actions: &ActionListViewModel,
) -> Buffer {
    let screens = default_screen_registry();
    let widgets = default_widget_registry();
    let container = ContainerViewModel::default();
    let event = EventViewModel::default();
    let help = HelpViewModel::default();
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
        stats,
        log,
        actions,
        container: &container,
        event: &event,
        help: &help,
        symbols: &symbols,
        theme: &theme,
        bindings: &bindings,
        mode,
        interaction,
        turn: 0,
        day: stats.day,
    };
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).expect("test terminal must initialize");
    terminal
        .draw(|frame| render_ui_frame(frame, &data))
        .expect("test frame must render");
    terminal.backend().buffer().clone()
}

fn buffer_text(buffer: &Buffer) -> String {
    let area = buffer.area;
    (area.y..area.y + area.height)
        .map(|y| {
            (area.x..area.x + area.width)
                .map(|x| {
                    buffer
                        .cell((x, y))
                        .expect("text coordinate must be inside the buffer")
                        .symbol()
                })
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn default_actions() -> ActionListViewModel {
    ActionListViewModel::default()
}

fn unique_symbol(buffer: &Buffer, symbol: &str) -> CellObservation {
    let matches = observe(buffer)
        .into_iter()
        .filter(|cell| cell.symbol == symbol)
        .collect::<Vec<_>>();
    assert_eq!(
        matches.len(),
        1,
        "fixture expected one `{symbol}` cell; matches={matches:?}\n{}",
        buffer_text(buffer)
    );
    matches.into_iter().next().expect("one match was asserted")
}

fn normalized_text(buffer: &Buffer) -> String {
    buffer_text(buffer)
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
        .join(" ")
}

fn normalized_panel_text(buffer: &Buffer, screen: &str, panel_id: &str) -> String {
    let area = buffer.area;
    let registry = default_screen_registry();
    let canonical = registry.get(screen).expect("screen fixture must exist");
    let definition = if commands::terminal_layout(area.width, area.height)
        == commands::TerminalLayout::Compact
    {
        screens::compact_screen_definition(canonical)
    } else {
        canonical.clone()
    };
    let content = Rect::new(area.x, area.y, area.width, area.height.saturating_sub(3));
    let panel = compute_panel_rects(&definition, content)
        .into_iter()
        .find_map(|(id, rect)| (id == panel_id).then_some(rect))
        .unwrap_or_else(|| panic!("screen `{screen}` has no `{panel_id}` panel"));
    (panel.y..panel.y + panel.height)
        .flat_map(|y| {
            (panel.x..panel.x + panel.width).map(move |x| {
                buffer
                    .cell((x, y))
                    .expect("panel coordinate must be inside the buffer")
                    .symbol()
            })
        })
        .collect::<String>()
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
        .join(" ")
}

#[test]
fn tui_renderers_delegate_terminal_colors_to_the_theme_layer() {
    // Contract: VISUAL-THEME-001
    // Given: the production TUI renderers for screens and frame chrome.
    // When: their source is inspected before the test-only modules.
    // Then: no renderer chooses a raw Ratatui color.
    // Must not change: tests may still use concrete colors to verify resolved output.
    // Evidence layers: architecture, resolved style.
    for (path, source) in [
        ("src/screens.rs", include_str!("screens.rs")),
        ("src/lib.rs", include_str!("lib.rs")),
        ("src/chrome.rs", include_str!("chrome.rs")),
    ] {
        let production = source.split("#[cfg(test)]").next().unwrap_or(source);
        let raw_color_lines = production
            .lines()
            .enumerate()
            .filter(|(_, line)| line.contains("Color::"))
            .map(|(index, line)| format!("{}: {}", index + 1, line.trim()))
            .collect::<Vec<_>>();

        assert!(
            raw_color_lines.is_empty(),
            "contract=VISUAL-THEME-001 case={path} fixture=production-renderers \
             precondition=production_source_before_test_modules \
             action=scan_renderer_color_ownership \
             expected={path} delegates every terminal color to ThemeRegistry \
             must_not_change=concrete_colors_remain_allowed_in_test_modules \
             actual_raw_color_lines={raw_color_lines:#?}"
        );
    }
}

#[test]
fn outpost_panels_render_shared_chrome_at_supported_profiles() {
    // Supporting contract: VISUAL-THEME-001
    // Given: the Outpost at both supported terminal profiles.
    // When: the final composed buffer is inspected at each registered panel boundary.
    // Then: every ordinary panel uses the complete muted single-rule Reliquary
    // structure and exactly one emphasized heading marker without depending on title
    // copy or its exact offset.
    // Must not change: panel IDs and computed areas remain fixture inputs, not outcomes.
    // Evidence layers: resolved style, buffer layout.
    let expected_theme = ThemeRegistry::phase5_defaults();
    let expected_border = expected_theme.resolve(visual::StyleToken::UiPanelBorder);
    let expected_title = expected_theme.resolve(visual::StyleToken::UiPanelTitle);
    for (width, height) in [(80_u16, 24_u16), (60, 20)] {
        let map = shelter_map();
        let buffer = render_buffer(
            "outpost",
            width,
            height,
            GameMode::Outpost,
            &map,
            &StatsViewModel::default(),
            &LogViewModel::default(),
            &default_actions(),
        );
        let canonical = default_screen_registry()
            .get("outpost")
            .expect("outpost screen must exist")
            .clone();
        let definition =
            if commands::terminal_layout(width, height) == commands::TerminalLayout::Compact {
                screens::compact_screen_definition(&canonical)
            } else {
                canonical
            };
        let content = Rect::new(0, 0, width, height - 3);

        for (panel_id, area) in compute_panel_rects(&definition, content) {
            let actual = ruined_reliquary_panel_violations(
                &buffer,
                area,
                expected_border.fg.unwrap_or(Color::Reset),
                expected_title.fg.unwrap_or(Color::Reset),
            );
            assert!(
                actual.is_empty(),
                "contract=VISUAL-THEME-001 case={width}x{height} panel={panel_id} \
                 precondition=registered_outpost_panel action=inspect_complete_final_panel_frame \
                 expected=muted_single_rule_with_one_bold_reliquary_title_marker \
                 must_not_change=title_copy_and_offset_are_not_owned actual={actual:?} \
                 visual_crop=\n{}",
                frame_edge_crop(&buffer)
            );
        }
    }
}

#[test]
fn reliquary_panel_and_meter_observers_reject_single_cell_breaks() {
    // Supporting contract: VISUAL-IDENTITY-001
    // Given: one valid inner panel and one valid partial ASCII meter.
    // When: a single structural glyph or resolved meter style is changed.
    // Then: the applicable reusable observer rejects the exact damaged cell.
    // Must not change: title wording, track width, and fill algorithm are unowned.
    // Evidence layers: observer integrity, resolved style, buffer geometry.
    let theme = ThemeRegistry::phase5_defaults();
    let border = theme
        .resolve(visual::StyleToken::UiPanelBorder)
        .fg
        .unwrap_or(Color::Reset);
    let title = theme
        .resolve(visual::StyleToken::UiPanelTitle)
        .fg
        .unwrap_or(Color::Reset);
    let fill = theme
        .resolve(visual::StyleToken::UiPositive)
        .fg
        .unwrap_or(Color::Reset);
    let empty = theme
        .resolve(visual::StyleToken::UiMuted)
        .fg
        .unwrap_or(Color::Reset);

    let panel_area = Rect::new(0, 0, 18, 4);
    let mut panel_buffer = Buffer::empty(panel_area);
    Block::default()
        .title(ratatui::text::Line::styled(
            " ◆ Stats ",
            theme.resolve(visual::StyleToken::UiPanelTitle),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(theme.resolve(visual::StyleToken::UiPanelBorder))
        .render(panel_area, &mut panel_buffer);
    assert!(
        ruined_reliquary_panel_violations(&panel_buffer, panel_area, border, title).is_empty(),
        "contract=VISUAL-IDENTITY-001 case=panel-observer-valid-control \
         precondition=canonical_single_rule_reliquary_panel \
         action=inspect_complete_panel expected=accepted actual={:?}",
        ruined_reliquary_panel_violations(&panel_buffer, panel_area, border, title)
    );
    let mut broken_panel = panel_buffer.clone();
    broken_panel[(7, 3)].set_symbol("═");
    let panel_actual = ruined_reliquary_panel_violations(&broken_panel, panel_area, border, title);
    assert!(
        panel_actual
            .iter()
            .any(|violation| violation.contains("position=(7, 3)")),
        "contract=VISUAL-IDENTITY-001 case=panel-observer-glyph-mutation \
         precondition=one_bottom_rule_cell_changed action=inspect_complete_panel \
         expected=reject_position_(7,3) actual={panel_actual:?}"
    );

    let meter_area = Rect::new(0, 0, 18, 1);
    let mut meter_buffer = Buffer::empty(meter_area);
    meter_buffer.set_string(0, 0, "HP 8/10 [###-]", Style::default());
    for x in 9..=11 {
        meter_buffer[(x, 0)].set_style(Style::default().fg(fill));
    }
    meter_buffer[(12, 0)].set_style(Style::default().fg(empty));
    assert!(
        ascii_meter_violations(&meter_buffer, meter_area, "HP", "8/10", fill, empty).is_empty(),
        "contract=VISUAL-IDENTITY-001 case=meter-observer-valid-control \
         precondition=canonical_partial_ascii_meter action=inspect_semantic_meter \
         expected=accepted actual={:?}",
        ascii_meter_violations(&meter_buffer, meter_area, "HP", "8/10", fill, empty)
    );
    let mut broken_meter = meter_buffer.clone();
    broken_meter[(10, 0)].set_style(Style::default().fg(Color::Red));
    let meter_actual = ascii_meter_violations(&broken_meter, meter_area, "HP", "8/10", fill, empty);
    assert!(
        meter_actual
            .iter()
            .any(|violation| violation.contains("position=(10, 0)")),
        "contract=VISUAL-IDENTITY-001 case=meter-observer-style-mutation \
         precondition=one_fill_cell_recolored action=inspect_semantic_meter \
         expected=reject_position_(10,0) actual={meter_actual:?}"
    );
}

#[test]
fn closed_double_frame_observer_rejects_single_cell_breaks() {
    // Supporting contract: VISUAL-IDENTITY-001
    // Given: one valid closed frame and independent one-cell mutations.
    // When: the shared perimeter observer inspects glyph, style, and geometry.
    // Then: it accepts the control and rejects each mutation at its exact coordinate.
    // Must not change: internal content and unrelated layout are outside this observer.
    // Evidence layers: observer integrity, resolved style, buffer geometry.
    let area = Rect::new(0, 0, 8, 5);
    let frame_color = Color::Rgb(0xb7, 0x6b, 0x4c);
    let frame_style = Style::default().fg(frame_color);
    let mut baseline = Buffer::empty(area);
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(frame_style)
        .render(area, &mut baseline);
    assert!(
        closed_double_frame_violations(&baseline, frame_color).is_empty(),
        "contract=VISUAL-IDENTITY-001 case=observer-valid-control \
         precondition=canonical_closed_double_frame action=inspect_complete_perimeter \
         expected=canonical_closed_double_frame_is_accepted \
         must_not_change=internal_content_is_not_observed actual={:?}",
        closed_double_frame_violations(&baseline, frame_color)
    );

    let mut glyph_break = baseline.clone();
    glyph_break[(0, 2)].set_symbol("│");

    let mut style_break = baseline.clone();
    style_break[(3, 0)].set_style(Style::default().fg(Color::Red));

    let mut geometry_break = baseline.clone();
    geometry_break[(7, 2)].set_symbol(" ");
    geometry_break[(6, 2)]
        .set_symbol("║")
        .set_style(frame_style);

    for (case, broken, expected_position) in [
        ("glyph", glyph_break, (0, 2)),
        ("resolved-style", style_break, (3, 0)),
        ("geometry", geometry_break, (7, 2)),
    ] {
        let actual = closed_double_frame_violations(&broken, frame_color);
        assert!(
            actual
                .iter()
                .any(|violation| violation.contains(&format!("position={expected_position:?}"))),
            "contract=VISUAL-IDENTITY-001 case=observer-{case}-mutation \
             precondition=one_smallest_granularity_{case}_break \
             action=inspect_complete_perimeter \
             expected=single_cell_break_rejected_at_{expected_position:?} actual={actual:?}"
        );
    }
}

#[test]
fn selected_cinder_rite_identity_frames_colony_and_reusable_screens() {
    // Contract: VISUAL-IDENTITY-001
    // Given: the owner-approved Ruined Reliquary / Cinder Rite acceptance target.
    // When: normal, Build Selection, Build Placement, and representative reusable
    // screens render at both supported profiles.
    // Then: semantic roles resolve the selected palette, every final composed state
    // carries a closed double-line shell, ordinary panels use the muted single-rule
    // Reliquary grammar, major modal chrome uses the selected double rule, shared HP/AP
    // tracks appear in colony and combat, and the footer uses mode/command ribbons.
    // Must not change: Turn, Day, build version, exact HP/AP values, and active controls
    // remain visible even though their presentation is upgraded.
    // Gameplay state, map glyphs, general copy, punctuation, panel count, and exact
    // internal panel rectangles are deliberately not owned by this visual contract.
    // Evidence layers: projection, resolved style, buffer geometry; PTY remains required.
    //
    // Implementation guidance for the renderer agent:
    // - Reusable owner: the Ruined Reliquary shell owns the terminal perimeter and
    //   exposes one inner Rect to ordinary panels, overlays, and the footer. Keep
    //   palette and chrome decisions in shared theme/chrome primitives.
    // - Integration seam: the final composed buffer is authoritative. Build and
    //   management overlays must stay inside the shell-owned inner Rect regardless of
    //   render order; isolated widgets cannot prove that the perimeter survives.
    // - Preserve: the footer still serves status, contextual controls, and global
    //   controls. HP/AP tracks retain exact numeric values. Do not remove information
    //   or recolor terrain/footer controls to manufacture widget evidence.
    // - Invalid shortcuts: do not add Outpost-only coordinates, screen-name branches,
    //   one-off colors, weaken the observer, remove a workflow/profile case, or change
    //   contract status merely to restore an aggregate green.
    // - Closing evidence: rerun this buffer matrix and real PTY workflows at both
    //   supported profiles before changing VISUAL-IDENTITY-001 to GreenUnreviewed.
    let theme = ThemeRegistry::phase5_defaults();
    let palette = [
        (visual::StyleToken::UiText, Color::Rgb(0xdc, 0xc7, 0xb3)),
        (visual::StyleToken::UiMuted, Color::Rgb(0x92, 0x78, 0x6b)),
        (
            visual::StyleToken::UiPanelBorder,
            Color::Rgb(0x71, 0x47, 0x37),
        ),
        (
            visual::StyleToken::UiPanelTitle,
            Color::Rgb(0xdd, 0x8a, 0x50),
        ),
        (visual::StyleToken::UiAccent, Color::Rgb(0xb7, 0x6b, 0x4c)),
        (
            visual::StyleToken::UiModalBorder,
            Color::Rgb(0xb7, 0x6b, 0x4c),
        ),
        (
            visual::StyleToken::UiModalTitle,
            Color::Rgb(0xdd, 0x8a, 0x50),
        ),
        (visual::StyleToken::UiInfo, Color::Rgb(0xa6, 0x8a, 0xb0)),
        (visual::StyleToken::UiKeyHint, Color::Rgb(0xa6, 0x8a, 0xb0)),
        (visual::StyleToken::UiPositive, Color::Rgb(0x8d, 0x9d, 0x62)),
        (visual::StyleToken::UiWarning, Color::Rgb(0xe0, 0xa1, 0x3f)),
        (visual::StyleToken::UiDanger, Color::Rgb(0xd1, 0x53, 0x48)),
    ];
    let selection_foreground = Color::Rgb(0xff, 0xe1, 0xc6);
    let selection_background = Color::Rgb(0x5b, 0x2e, 0x20);
    let primary_frame = Color::Rgb(0xb7, 0x6b, 0x4c);
    let panel_title = Color::Rgb(0xdd, 0x8a, 0x50);
    let panel_border = Color::Rgb(0x71, 0x47, 0x37);
    let body_text = Color::Rgb(0xdc, 0xc7, 0xb3);
    let muted_text = Color::Rgb(0x92, 0x78, 0x6b);
    let positive = Color::Rgb(0x8d, 0x9d, 0x62);
    let info = Color::Rgb(0xa6, 0x8a, 0xb0);
    let key_hint = Color::Rgb(0xa6, 0x8a, 0xb0);
    let mut violations = Vec::new();

    for (token, expected_foreground) in palette {
        let actual = theme.resolve(token);
        if actual.fg != Some(expected_foreground) {
            violations.push(format!(
                "case=theme-role-{token:?} expected_fg={expected_foreground:?} actual={actual:?}"
            ));
        }
    }

    let selection = theme.resolve(visual::StyleToken::Selection);
    if selection.fg != Some(selection_foreground)
        || selection.bg != Some(selection_background)
        || !selection.add_modifier.contains(Modifier::BOLD)
    {
        violations.push(format!(
            "case=theme-role-Selection expected_fg={selection_foreground:?} \
             expected_bg={selection_background:?} expected_modifier=BOLD actual={selection:?}"
        ));
    }

    for title_token in [
        visual::StyleToken::UiPanelTitle,
        visual::StyleToken::UiModalTitle,
    ] {
        let actual = theme.resolve(title_token);
        if !actual.add_modifier.contains(Modifier::BOLD) {
            violations.push(format!(
                "case=theme-role-{title_token:?} expected_modifier=BOLD actual={actual:?}"
            ));
        }
    }

    // Major modal moments are part of the selected global grammar. Test the shared
    // primitive so every present and future caller inherits the same double rule.
    let modal_area = Rect::new(0, 0, 24, 5);
    let mut modal_buffer = Buffer::empty(modal_area);
    panel(&theme, "Build Station", PanelTone::Modal).render(modal_area, &mut modal_buffer);
    let modal_left = modal_area.x;
    let modal_right = modal_area.x + modal_area.width - 1;
    let modal_top = modal_area.y;
    let modal_bottom = modal_area.y + modal_area.height - 1;
    let mut modal_violations = Vec::new();
    for (position, glyph) in [
        ((modal_left, modal_top), "╔"),
        ((modal_right, modal_top), "╗"),
        ((modal_left, modal_bottom), "╚"),
        ((modal_right, modal_bottom), "╝"),
    ] {
        modal_violations.extend(frame_cell_violations(
            &modal_buffer,
            position,
            &[glyph],
            primary_frame,
        ));
    }
    for x in modal_left + 1..modal_right {
        modal_violations.extend(frame_cell_violations(
            &modal_buffer,
            (x, modal_bottom),
            &["═"],
            primary_frame,
        ));
        let top_cell = modal_buffer
            .cell((x, modal_top))
            .expect("modal top edge must be inside the buffer");
        let is_border = top_cell.symbol() == "═" && top_cell.fg == primary_frame;
        let is_title = top_cell.fg == panel_title && top_cell.modifier.contains(Modifier::BOLD);
        if !is_border && !is_title {
            modal_violations.push(format!(
                "position=({x}, {modal_top}) expected=primary_double_rule_or_bold_modal_title \
                 actual_glyph={:?} actual_fg={:?} actual_modifier={:?}",
                top_cell.symbol(),
                top_cell.fg,
                top_cell.modifier
            ));
        }
    }
    for y in modal_top + 1..modal_bottom {
        modal_violations.extend(frame_cell_violations(
            &modal_buffer,
            (modal_left, y),
            &["║"],
            primary_frame,
        ));
        modal_violations.extend(frame_cell_violations(
            &modal_buffer,
            (modal_right, y),
            &["║"],
            primary_frame,
        ));
    }
    if !modal_violations.is_empty() {
        let first_modal_violations = modal_violations
            .iter()
            .take(12)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        violations.push(format!(
            "case=shared-major-modal-chrome expected=closed_double_rule_with_primary_frame_style \
             violation_count={} first_violations=[{first_modal_violations}] visual_crop=\n{}",
            modal_violations.len(),
            buffer_text(&modal_buffer)
        ));
    }

    let map = shelter_map();
    let mut build_selection = map.clone();
    build_selection.build_menu = Some(view_models::BuildMenuVm {
        options: vec![
            ("Stove".into(), 2, "+3 Supplies/day when staffed".into()),
            ("Workshop".into(), 2, "Produces refined Materials".into()),
        ],
        selected: 0,
        available_supplies: 10,
    });
    let mut build_placement = map.clone();
    build_placement.build_ghost = Some((Position { x: 2, y: 1 }, 'f'));
    build_placement.build_placement = Some(view_models::BuildPlacementVm {
        label: "Stove".into(),
        supply_cost: 2,
        effect: "+3 Supplies/day when staffed".into(),
    });
    let stats = StatsViewModel {
        hp_current: 24,
        hp_max: 30,
        ap_current: 2,
        ap_max: 3,
        supplies: 5,
        materials: 2,
        day: 3,
        party_names: vec!["Mara — EnRoute Trees · 6 tiles".into()],
        next_day_forecast: "Next day: Supplies 5 -> 5".into(),
        ..Default::default()
    };
    let cases = [
        ("outpost-normal", "outpost", GameMode::Outpost, &map),
        (
            "outpost-build-selection",
            "outpost",
            GameMode::Outpost,
            &build_selection,
        ),
        (
            "outpost-build-placement",
            "outpost",
            GameMode::Outpost,
            &build_placement,
        ),
        ("combat-normal", "combat", GameMode::Tactical, &map),
        ("inventory-normal", "inventory", GameMode::Outpost, &map),
    ];

    for (case_id, screen, mode, case_map) in cases {
        for (width, height) in [(80_u16, 24_u16), (60, 20)] {
            let buffer = render_buffer(
                screen,
                width,
                height,
                mode,
                case_map,
                &stats,
                &LogViewModel::default(),
                &default_actions(),
            );
            let mut case_violations = Vec::new();
            let frame_violations = closed_double_frame_violations(&buffer, primary_frame);
            if !frame_violations.is_empty() {
                let first_violations = frame_violations
                    .iter()
                    .take(12)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ");
                case_violations.push(format!(
                    "closed_frame expected=continuous_terminal_perimeter_with_primary_frame_style \
                     violation_count={} first_violations=[{first_violations}]",
                    frame_violations.len()
                ));
            }

            let rendered_text = buffer_text(&buffer);
            let footer = rendered_text
                .lines()
                .skip(height.saturating_sub(3) as usize)
                .collect::<Vec<_>>()
                .join(" ");
            let expected_mode = if case_id.contains("build") {
                "BUILD".to_owned()
            } else {
                screen.to_ascii_uppercase()
            };
            for required in [
                expected_mode,
                format!("DAY {:02}", stats.day),
                "TURN 0".to_owned(),
                format!("KERNEL v{}", env!("CARGO_PKG_VERSION")),
            ] {
                if !footer.contains(&required) {
                    case_violations.push(format!(
                        "mode_ribbon expected_visible_token={required:?} actual_footer={footer:?}"
                    ));
                }
            }
            let key_chips = footer_key_chip_count(&buffer, key_hint);
            if key_chips < 2 {
                case_violations.push(format!(
                    "command_ribbon expected_theme_key_chips>=2 actual={key_chips} \
                     actual_footer={footer:?}"
                ));
            }

            let cells = observe(&buffer);
            let title_cells = cells
                .iter()
                .filter(|cell| {
                    !cell.symbol.trim().is_empty()
                        && cell.foreground == panel_title
                        && cell.modifier.contains(Modifier::BOLD)
                })
                .count();
            if title_cells < 3 {
                case_violations.push(format!(
                    "title_hierarchy expected_bold_lit_copper_cells>=3 actual={title_cells}"
                ));
            }
            let canonical = default_screen_registry()
                .get(screen)
                .expect("representative screen fixture must exist")
                .clone();
            let definition =
                if commands::terminal_layout(width, height) == commands::TerminalLayout::Compact {
                    screens::compact_screen_definition(&canonical)
                } else {
                    canonical
                };
            let content_area = Rect::new(0, 0, width, height.saturating_sub(3));
            let panel_rects = compute_panel_rects(&definition, content_area);
            let map_area = panel_rects
                .iter()
                .into_iter()
                .find_map(|(panel_id, area)| (panel_id == "map").then_some(*area));
            let body_cells = cells
                .iter()
                .filter(|cell| {
                    let inside_content = rect_contains(content_area, cell.x, cell.y);
                    let outside_map = map_area
                        .map(|area| !rect_contains(area, cell.x, cell.y))
                        .unwrap_or(true);
                    inside_content
                        && outside_map
                        && !cell.symbol.trim().is_empty()
                        && cell.foreground == body_text
                })
                .count();
            if screen != "inventory" && body_cells == 0 {
                case_violations
                    .push("body_hierarchy expected_non_map_warm_bone_cells>0 actual=0".into());
            }

            if !case_id.contains("build") {
                for (panel_id, area) in &panel_rects {
                    let actual = ruined_reliquary_panel_violations(
                        &buffer,
                        *area,
                        panel_border,
                        panel_title,
                    );
                    if !actual.is_empty() {
                        let first = actual.iter().take(8).cloned().collect::<Vec<_>>();
                        case_violations.push(format!(
                            "panel={panel_id} expected=complete_muted_single_rule_with_title_marker \
                             violation_count={} first_violations={first:?}",
                            actual.len()
                        ));
                    }
                }
            }

            if matches!(screen, "outpost" | "combat") {
                let stats_area = panel_rects
                    .iter()
                    .find_map(|(panel_id, area)| (panel_id == "stats").then_some(*area))
                    .expect("outpost and combat fixtures must retain a Stats panel");
                for (label, value, fill) in [("HP", "24/30", positive), ("AP", "2/3", info)] {
                    let actual =
                        ascii_meter_violations(&buffer, stats_area, label, value, fill, muted_text);
                    if !actual.is_empty() {
                        case_violations.push(format!(
                            "meter={label} expected=responsive_ascii_track_with_exact_value \
                             actual={actual:?}"
                        ));
                    }
                }
            }

            if !case_violations.is_empty() {
                let crop = frame_edge_crop(&buffer);
                violations.push(format!(
                    "case={case_id}-{width}x{height} screen={screen} actual=[{}] \
                     visual_crop=\n{crop}",
                    case_violations.join("; ")
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "contract_id=VISUAL-IDENTITY-001 fixture_id=selected_cinder_rite_reliquary_identity \
         precondition=owner-approved_cinder_rite_target_at_supported_profiles \
         action=render_normal_and_build_workflows_into_final_composed_buffers \
         expected=selected palette plus double shell, single-rule panels, semantic meters, and ribbons \
         must_not_change=turn,day,version,hp-ap-values,active-controls \
         not_owned=panel-count,exact-rectangles,map-glyphs,gameplay-state,general-copy \
         actual_violations=\n{}",
        violations.join("\n\n")
    );
}

#[test]
fn visual_observation_detects_glyph_foreground_modifier_and_geometry_changes() {
    // Contract: VISUAL-OBSERVATION-001
    let mut baseline = Buffer::empty(Rect::new(0, 0, 3, 2));
    baseline.set_string(0, 0, "A", Style::default().fg(Color::White));

    let mut glyph = baseline.clone();
    glyph.set_string(0, 0, "B", Style::default().fg(Color::White));
    assert_ne!(
        observe(&baseline),
        observe(&glyph),
        "glyph-only diff was lost"
    );

    let mut foreground = baseline.clone();
    foreground.set_string(0, 0, "A", Style::default().fg(Color::Red));
    assert_ne!(
        observe(&baseline),
        observe(&foreground),
        "foreground-only diff was lost"
    );

    let mut modifier = baseline.clone();
    modifier.set_string(
        0,
        0,
        "A",
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    );
    assert_ne!(
        observe(&baseline),
        observe(&modifier),
        "modifier-only diff was lost"
    );

    let larger = Buffer::empty(Rect::new(0, 0, 4, 2));
    assert_ne!(
        observe(&baseline),
        observe(&larger),
        "geometry diff was lost"
    );
}

#[test]
fn identical_fixture_has_identical_canvas_and_resolved_styles() {
    // Supporting deterministic evidence for VISUAL-OBSERVATION-001.
    let map = shelter_map();
    let stats = StatsViewModel::default();
    let log = LogViewModel::default();
    let actions = default_actions();
    let first = render_buffer(
        "outpost",
        80,
        24,
        GameMode::Outpost,
        &map,
        &stats,
        &log,
        &actions,
    );
    let second = render_buffer(
        "outpost",
        80,
        24,
        GameMode::Outpost,
        &map,
        &stats,
        &log,
        &actions,
    );

    assert_eq!(
        observe(&first),
        observe(&second),
        "same-state rendering is not deterministic"
    );
}

#[test]
fn supported_outpost_panel_rectangles_never_overlap() {
    // Supporting geometry evidence for VISUAL-LAYOUT-001.
    for (width, height) in [(80_u16, 24_u16), (60, 20)] {
        let registry = default_screen_registry();
        let canonical = registry.get("outpost").expect("outpost must exist");
        let definition =
            if commands::terminal_layout(width, height) == commands::TerminalLayout::Compact {
                screens::compact_screen_definition(canonical)
            } else {
                canonical.clone()
            };
        let content = Rect::new(0, 0, width, height - 3);
        let rectangles = compute_panel_rects(&definition, content);

        for (left_index, (left_id, left)) in rectangles.iter().enumerate() {
            assert!(
                left.width > 0 && left.height > 0,
                "contract=VISUAL-LAYOUT-001 case={width}x{height} panel={left_id} \
                 has unusable geometry {left:?}"
            );
            for (right_id, right) in rectangles.iter().skip(left_index + 1) {
                let horizontal = left.x < right.x + right.width && right.x < left.x + left.width;
                let vertical = left.y < right.y + right.height && right.y < left.y + left.height;
                assert!(
                    !(horizontal && vertical),
                    "contract=VISUAL-LAYOUT-001 case={width}x{height} \
                     panels overlap: {left_id}={left:?}, {right_id}={right:?}"
                );
            }
        }
    }
}

#[test]
fn outpost_map_is_the_largest_interactive_panel_at_supported_profiles() {
    // Contract: VISUAL-LAYOUT-001
    for (width, height) in [(80_u16, 24_u16), (60, 20)] {
        let registry = default_screen_registry();
        let canonical = registry.get("outpost").expect("outpost must exist");
        let definition =
            if commands::terminal_layout(width, height) == commands::TerminalLayout::Compact {
                screens::compact_screen_definition(canonical)
            } else {
                canonical.clone()
            };
        let rectangles = compute_panel_rects(&definition, Rect::new(0, 0, width, height - 3));
        let map_area = rectangles
            .iter()
            .find(|(id, _)| id == "map")
            .map(|(_, rect)| u32::from(rect.width) * u32::from(rect.height))
            .expect("outpost must contain a map");
        let largest_other = rectangles
            .iter()
            .filter(|(id, _)| id != "map")
            .map(|(_, rect)| u32::from(rect.width) * u32::from(rect.height))
            .max()
            .unwrap_or_default();

        assert!(
            map_area > largest_other,
            "contract=VISUAL-LAYOUT-001 case={width}x{height} \
             expected physical shelter map to own the largest interactive area; \
             map_area={map_area}, largest_other={largest_other}, panels={rectangles:?}"
        );
    }
}

#[test]
fn offscreen_assignment_names_target_and_distance_at_supported_profiles() {
    // Contract: VISUAL-VIEWPORT-005
    let mut map = shelter_map();
    map.assigned_targets = vec![Position { x: 30, y: 29 }];
    map.assigned_target_details = vec![view_models::AssignedTargetVm {
        position: Position { x: 30, y: 29 },
        label: "Water Source".into(),
        survivor: "Mara".into(),
    }];
    for (width, height) in [(80, 24), (60, 20)] {
        let buffer = render_buffer(
            "outpost",
            width,
            height,
            GameMode::Outpost,
            &map,
            &StatsViewModel::default(),
            &LogViewModel::default(),
            &default_actions(),
        );
        let text = normalized_text(&buffer);
        for required in ["Water Source", "57 tiles"] {
            assert!(
                text.contains(required),
                "contract=VISUAL-VIEWPORT-005 case={width}x{height} \
                 fixture=offscreen-water-target missing `{required}`; \
                 expected direction, target name, and distance\n{}",
                buffer_text(&buffer)
            );
        }
    }
}

#[test]
fn valid_and_invalid_build_previews_differ_without_color() {
    // Contract: VISUAL-BUILD-005
    let mut valid_map = shelter_map();
    valid_map.build_ghost = Some((Position { x: 2, y: 1 }, '¤'));
    valid_map.build_placement = Some(view_models::BuildPlacementVm {
        label: "Workshop".into(),
        supply_cost: 4,
        effect: "Produces refined Materials".into(),
    });
    let valid = render_buffer(
        "outpost",
        80,
        24,
        GameMode::Outpost,
        &valid_map,
        &StatsViewModel::default(),
        &LogViewModel::default(),
        &default_actions(),
    );

    let mut invalid_map = valid_map.clone();
    invalid_map.build_ghost_denial = Some("Occupied by Basic Processing".into());
    let invalid = render_buffer(
        "outpost",
        80,
        24,
        GameMode::Outpost,
        &invalid_map,
        &StatsViewModel::default(),
        &LogViewModel::default(),
        &default_actions(),
    );
    let valid_cell = unique_symbol(&valid, "¤");
    let invalid_cell = unique_symbol(&invalid, "!");

    assert_ne!(
        (&valid_cell.symbol, valid_cell.modifier),
        (&invalid_cell.symbol, invalid_cell.modifier),
        "contract=VISUAL-BUILD-005 fixture=valid-vs-invalid-preview \
         color-independent preview semantics are identical; \
         valid={valid_cell:?}, invalid={invalid_cell:?}"
    );
}

#[test]
fn unaffordable_build_selection_explains_the_exact_shortage() {
    // Contract: VISUAL-BUILD-006
    let mut map = shelter_map();
    map.build_menu = Some(view_models::BuildMenuVm {
        options: vec![("Workshop".into(), 4, "Produces refined Materials".into())],
        selected: 0,
        available_supplies: 1,
    });
    for (width, height) in [(80, 24), (60, 20)] {
        let buffer = render_buffer(
            "outpost",
            width,
            height,
            GameMode::Outpost,
            &map,
            &StatsViewModel::default(),
            &LogViewModel::default(),
            &default_actions(),
        );
        assert!(
            normalized_text(&buffer).contains("Need 3 more Supplies"),
            "contract=VISUAL-BUILD-006 case={width}x{height} \
             expected exact unavailable reason `Need 3 more Supplies`\n{}",
            buffer_text(&buffer)
        );
    }
}

#[test]
fn task_management_exposes_survivor_task_and_confirm_stages() {
    // Contract: VISUAL-MGMT-003
    let stats = StatsViewModel {
        management: Some(view_models::ManagementMenuVm {
            kind: view_models::ManagementMenuKind::TaskAssignment,
            survivors: vec!["Mara — Idle".into()],
            tasks: vec!["1. Gather Supplies".into()],
            selected_survivor: Some(0),
            selected_task: Some(0),
            resources: "Sup 4  Mat 0  Plant 0  Faith 0".into(),
            forecast: "Next worker: none | Next day: upkeep -3".into(),
        }),
        ..Default::default()
    };
    for (width, height) in [(80, 24), (60, 20)] {
        let buffer = render_buffer(
            "outpost",
            width,
            height,
            GameMode::Outpost,
            &shelter_map(),
            &stats,
            &LogViewModel::default(),
            &default_actions(),
        );
        let text = normalized_text(&buffer);
        for required in ["1 Survivor", "2 Task", "3 Confirm"] {
            assert!(
                text.contains(required),
                "contract=VISUAL-MGMT-003 case={width}x{height} \
                 missing workflow stage `{required}`\n{}",
                buffer_text(&buffer)
            );
        }
    }
}

#[test]
fn station_staffing_exposes_survivor_station_recipe_and_confirm_stages() {
    // Contract: VISUAL-MGMT-004
    let stats = StatsViewModel {
        management: Some(view_models::ManagementMenuVm {
            kind: view_models::ManagementMenuKind::StationStaffing,
            survivors: vec!["Mara — Idle".into()],
            tasks: vec!["1. Refine Timber — Raw Timber → Refined Materials".into()],
            selected_survivor: Some(0),
            selected_task: Some(0),
            resources: "Sup 4  Mat 0  Plant 0  Faith 0".into(),
            forecast: "Next worker: Refined Materials in 2 turns".into(),
        }),
        ..Default::default()
    };
    for (width, height) in [(80, 24), (60, 20)] {
        let buffer = render_buffer(
            "outpost",
            width,
            height,
            GameMode::Outpost,
            &shelter_map(),
            &stats,
            &LogViewModel::default(),
            &default_actions(),
        );
        let text = normalized_text(&buffer);
        for required in ["1 Survivor", "2 Station", "3 Recipe", "4 Confirm"] {
            assert!(
                text.contains(required),
                "contract=VISUAL-MGMT-004 case={width}x{height} \
                 missing workflow stage `{required}`\n{}",
                buffer_text(&buffer)
            );
        }
    }
}

#[test]
fn blocked_worker_has_a_distinct_resolved_style_from_working_worker() {
    // Contract: VISUAL-LANGUAGE-004
    let mut map = shelter_map();
    map.visuals.extend([
        view_models::MapVisualVm {
            position: Position { x: 2, y: 1 },
            token: visual::VisualToken::WorkerWorking,
            glyph: Some('ŵ'),
        },
        view_models::MapVisualVm {
            position: Position { x: 3, y: 1 },
            token: visual::VisualToken::WorkerBlocked,
            glyph: Some('ƀ'),
        },
    ]);
    let buffer = render_buffer(
        "outpost",
        80,
        24,
        GameMode::Outpost,
        &map,
        &StatsViewModel::default(),
        &LogViewModel::default(),
        &default_actions(),
    );
    let working = unique_symbol(&buffer, "ŵ");
    let blocked = unique_symbol(&buffer, "ƀ");

    assert_ne!(
        (working.foreground, working.background, working.modifier),
        (blocked.foreground, blocked.background, blocked.modifier),
        "contract=VISUAL-LANGUAGE-004 fixture=working-vs-blocked \
         resolved styles are identical; working={working:?}, blocked={blocked:?}"
    );
}

#[test]
fn decisive_warning_survives_routine_log_overflow() {
    // Contract: VISUAL-FEEDBACK-001
    let mut entries = vec![view_models::LogEntryVm {
        message: "Cannot build here — shelter egress would be blocked".into(),
        level: LogLevel::Warn,
    }];
    entries.extend((1..=20).map(|turn| view_models::LogEntryVm {
        message: format!("Routine worker movement {turn}"),
        level: LogLevel::Info,
    }));
    let buffer = render_buffer(
        "outpost",
        80,
        24,
        GameMode::Outpost,
        &shelter_map(),
        &StatsViewModel::default(),
        &LogViewModel { entries },
        &default_actions(),
    );

    assert!(
        normalized_text(&buffer).contains("Cannot build here"),
        "contract=VISUAL-FEEDBACK-001 fixture=warning-plus-routine-overflow \
         decisive warning was buried before the player could act\n{}",
        buffer_text(&buffer)
    );
}

#[test]
fn rendered_dungeon_denials_explain_attack_pickup_and_extraction() {
    // Supporting rendered evidence for VISUAL-DUNGEON-001.
    let actions = ActionListViewModel {
        actions: vec![
            view_models::ActionItemVm {
                label: "Attack".into(),
                key_hint: "f".into(),
                enabled: false,
                denial_reason: Some("No adjacent target".into()),
            },
            view_models::ActionItemVm {
                label: "Pickup".into(),
                key_hint: "g".into(),
                enabled: false,
                denial_reason: Some("No loot here".into()),
            },
            view_models::ActionItemVm {
                label: "Extract".into(),
                key_hint: ">".into(),
                enabled: false,
                denial_reason: Some("Reach the exit".into()),
            },
        ],
    };
    for (width, height) in [(80, 24), (60, 20)] {
        let buffer = render_buffer(
            "combat",
            width,
            height,
            GameMode::Tactical,
            &shelter_map(),
            &StatsViewModel::default(),
            &LogViewModel::default(),
            &actions,
        );
        let text = normalized_panel_text(&buffer, "combat", "actions");
        for required in ["No adjacent target", "No loot here", "Reach the exit"] {
            assert!(
                text.contains(required),
                "contract=VISUAL-DUNGEON-001 case={width}x{height} \
                 rendered action panel hid `{required}`\n{}",
                buffer_text(&buffer)
            );
        }
    }
}

#[test]
fn dungeon_status_distinguishes_carried_loot_and_extraction_readiness() {
    // Contract: VISUAL-DUNGEON-001
    let stats = StatsViewModel {
        hp_current: 8,
        hp_max: 10,
        ap_current: 2,
        ap_max: 3,
        carried_loot: 2,
        extraction_ready: true,
        ..Default::default()
    };
    let buffer = render_buffer(
        "combat",
        80,
        24,
        GameMode::Tactical,
        &shelter_map(),
        &stats,
        &LogViewModel::default(),
        &default_actions(),
    );
    let text = normalized_text(&buffer);
    for required in ["Carried loot: 2", "Extraction: Ready"] {
        assert!(
            text.contains(required),
            "contract=VISUAL-DUNGEON-001 fixture=exit-with-two-loot \
             missing dungeon status `{required}`\n{}",
            buffer_text(&buffer)
        );
    }
}

#[test]
fn title_without_save_explains_why_load_is_unavailable() {
    // Contract: VISUAL-SHELL-001
    for (width, height) in [(80, 24), (60, 20)] {
        let buffer = render_buffer(
            "title",
            width,
            height,
            GameMode::Title,
            &MapViewModel::default(),
            &StatsViewModel::default(),
            &LogViewModel::default(),
            &default_actions(),
        );
        let text = normalized_text(&buffer);
        assert!(
            text.contains("Load unavailable") && text.contains("No save"),
            "contract=VISUAL-SHELL-001 case={width}x{height} fixture=title-no-save \
             title advertises Load without explaining its unavailable state\n{}",
            buffer_text(&buffer)
        );
    }
}

#[test]
fn closed_management_modal_leaves_the_same_canvas_as_a_clean_overview() {
    // Contract: VISUAL-RESIZE-001 (modal-close stale-cell portion).
    let width = 80;
    let height = 24;
    let map = shelter_map();
    let actions = default_actions();
    let modal_stats = StatsViewModel {
        management: Some(view_models::ManagementMenuVm {
            kind: view_models::ManagementMenuKind::TaskAssignment,
            survivors: vec!["Mara — Idle".into()],
            tasks: vec!["1. Gather Supplies".into()],
            selected_survivor: Some(0),
            selected_task: None,
            resources: "Sup 4".into(),
            forecast: "Next worker: none".into(),
        }),
        ..Default::default()
    };
    let clean_stats = StatsViewModel::default();
    let screens = default_screen_registry();
    let widgets = default_widget_registry();
    let symbols = SymbolRegistry::phase5_defaults();
    let theme = ThemeRegistry::phase5_defaults();
    let bindings = commands::CommandBindings::default();
    let container = ContainerViewModel::default();
    let event = EventViewModel::default();
    let help = HelpViewModel::default();
    let log = LogViewModel::default();
    let definition = screens.get("outpost").expect("outpost must exist");
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).expect("test terminal must initialize");

    for stats in [&modal_stats, &clean_stats] {
        let data = UiFrameData {
            definition,
            widgets: &widgets,
            map: &map,
            stats,
            log: &log,
            actions: &actions,
            container: &container,
            event: &event,
            help: &help,
            symbols: &symbols,
            theme: &theme,
            bindings: &bindings,
            mode: GameMode::Outpost,
            interaction: frame_interaction(
                GameMode::Outpost,
                false,
                stats.management.as_ref().map(|menu| menu.kind),
            ),
            turn: 0,
            day: 0,
        };
        terminal
            .draw(|frame| render_ui_frame(frame, &data))
            .expect("frame must render");
    }
    let after_close = terminal.backend().buffer().clone();
    let clean = render_buffer(
        "outpost",
        width,
        height,
        GameMode::Outpost,
        &map,
        &clean_stats,
        &log,
        &actions,
    );

    assert_eq!(
        observe(&after_close),
        observe(&clean),
        "contract=VISUAL-RESIZE-001 fixture=management-close \
         closed modal left stale cells"
    );
}

#[test]
fn resize_round_trip_returns_to_the_original_canvas_and_styles() {
    // Contract: VISUAL-RESIZE-001
    let map = shelter_map();
    let stats = StatsViewModel::default();
    let log = LogViewModel::default();
    let actions = default_actions();
    let original = render_buffer(
        "outpost",
        80,
        24,
        GameMode::Outpost,
        &map,
        &stats,
        &log,
        &actions,
    );
    let compact = render_buffer(
        "outpost",
        60,
        20,
        GameMode::Outpost,
        &map,
        &stats,
        &log,
        &actions,
    );
    let returned = render_buffer(
        "outpost",
        80,
        24,
        GameMode::Outpost,
        &map,
        &stats,
        &log,
        &actions,
    );

    assert_eq!(compact.area, Rect::from(Size::new(60, 20)));
    assert_eq!(
        observe(&original),
        observe(&returned),
        "contract=VISUAL-RESIZE-001 fixture=80-60-80 \
         round-trip resize changed the restored frame"
    );
}

fn meter_track_counts(buffer: &Buffer, label: &str, value: &str) -> Option<(usize, usize)> {
    let area = buffer.area;
    for y in area.y..area.y + area.height {
        let cells = (area.x..area.x + area.width)
            .filter_map(|x| buffer.cell((x, y)))
            .collect::<Vec<_>>();
        let row = cells.iter().map(|cell| cell.symbol()).collect::<String>();
        if !row.contains(label) || !row.contains(value) {
            continue;
        }
        let open = cells.iter().position(|cell| cell.symbol() == "[")?;
        let close = cells[open + 1..]
            .iter()
            .position(|cell| cell.symbol() == "]")?
            + open
            + 1;
        let filled = cells[open + 1..close]
            .iter()
            .filter(|cell| cell.symbol() == "#")
            .count();
        let empty = cells[open + 1..close]
            .iter()
            .filter(|cell| cell.symbol() == "-")
            .count();
        return Some((filled, empty));
    }
    None
}

#[test]
fn provisions_show_stock_pressure_and_dawn_outlook_at_supported_profiles() {
    // Contract: VISUAL-ECON-002
    // Given: the same colony at low and stable Supplies pressure.
    // When: the final Outpost frame renders at 80x24 and 60x20.
    // Then: Supplies has one semantic partial gauge, exact stock, a text condition,
    // and the authoritative next-day delta/result; the higher stock fills more of
    // the same responsive visual language.
    // Must not change: HP/AP remain exact shared meters, the map remains present,
    // and the renderer must not hide the current stock or dawn result to gain space.
    // Evidence layers: projection, resolved style, buffer layout; PTY remains required.
    //
    // Implementation guidance:
    // - Reusable owner: project resource amount, bound, condition, delta, and result
    //   as display-ready facts consumed by the shared meter/chip/panel language.
    // - Integration seam: inspect the final composed Outpost buffer at both supported
    //   profiles; an isolated gauge or a forecast string outside the visible resource
    //   region is insufficient.
    // - Preserve: existing HP/AP tracks, day/resource truth, map primacy, theme-owned
    //   colors, and the distinction between next-worker and next-day information.
    // - Invalid shortcuts: do not add a decorative unrelated bar, hardcode this
    //   fixture's values/thresholds, parse forecast prose in the renderer, or delete
    //   neighboring status to make the track fit.
    // - Closing evidence: rerun both pressure cases at both profiles, the complete
    //   bd_tui target, canonical gate, and real 80x24/60x20 Outpost PTY review before
    //   changing VISUAL-ECON-002 from Red.
    let theme = ThemeRegistry::phase5_defaults();
    let empty = theme
        .resolve(visual::StyleToken::UiMuted)
        .fg
        .unwrap_or(Color::Reset);
    let cases = [
        ("low", 10, "LOW", "-3", "7", visual::StyleToken::UiWarning),
        (
            "stable",
            70,
            "STABLE",
            "-3",
            "67",
            visual::StyleToken::UiPositive,
        ),
    ];

    for (width, height) in [(80_u16, 24_u16), (60, 20)] {
        let mut tracks = Vec::new();
        for (case_id, supplies, condition, delta, result, tone) in cases {
            let mut app = production_outpost_runtime();
            app.world_mut()
                .resource_mut::<ColonyResources>()
                .pools
                .get_mut(PoolKind::Supplies)
                .expect("Foundation colony must own Supplies")
                .current = supplies;
            app.update();
            let map = app.world().resource::<MapViewModel>().clone();
            let projected = app.world().resource::<StatsViewModel>().clone();
            let log = app.world().resource::<LogViewModel>().clone();
            let actions = app.world().resource::<ActionListViewModel>().clone();
            assert_eq!(
                projected.supplies, supplies,
                "contract=VISUAL-ECON-002 case={case_id}-{width}x{height} \
                 fixture=production-colony precondition=projected_stock"
            );

            // False-green challenge: poison only the legacy flat/prose inputs after
            // production projection. A compliant renderer consumes the structured
            // resource presentation produced in the same frame, so the final gauge,
            // condition, and dawn facts remain unchanged. Parsing this prose or
            // recomputing pressure from these flat fields must fail visibly.
            let mut stats = projected.clone();
            stats.supplies = -777;
            stats.supplies_max = 1;
            stats.next_day_forecast = "opaque legacy forecast prose".into();
            let buffer = render_buffer(
                "outpost",
                width,
                height,
                GameMode::Outpost,
                &map,
                &stats,
                &log,
                &actions,
            );
            let semantic = normalized_text(&buffer);
            let fill = theme.resolve(tone).fg.unwrap_or(Color::Reset);
            let gauge_violations = ascii_meter_violations(
                &buffer,
                buffer.area,
                "SUP",
                &supplies.to_string(),
                fill,
                empty,
            );
            assert!(
                gauge_violations.is_empty(),
                "contract=VISUAL-ECON-002 case={case_id}-{width}x{height} \
                 fixture=provisions-pressure precondition=supplies_{supplies} \
                 action=inspect_final_semantic_resource_gauge \
                 expected=partial_SUP_gauge_with_exact_stock_and_theme_owned_tone \
                 must_not_change=HP_AP_map_and_day actual={gauge_violations:?} \
                 visual_crop=\n{}",
                buffer_text(&buffer)
            );
            for required in [condition, delta, result] {
                assert!(
                    semantic.contains(required),
                    "contract=VISUAL-ECON-002 case={case_id}-{width}x{height} \
                     fixture=provisions-pressure precondition=supplies_{supplies} \
                     action=read_visible_pressure_and_dawn_outlook \
                     expected_field={required:?} actual_semantic={semantic:?} \
                     visual_crop=\n{}",
                    buffer_text(&buffer)
                );
            }
            for (label, value, tone) in [
                (
                    "HP",
                    format!("{}/{}", projected.hp_current, projected.hp_max),
                    visual::StyleToken::UiPositive,
                ),
                (
                    "AP",
                    format!("{}/{}", projected.ap_current, projected.ap_max),
                    visual::StyleToken::UiInfo,
                ),
            ] {
                let preserved_fill = theme.resolve(tone).fg.unwrap_or(Color::Reset);
                let preserved = ascii_meter_violations(
                    &buffer,
                    buffer.area,
                    label,
                    &value,
                    preserved_fill,
                    empty,
                );
                assert!(
                    preserved.is_empty(),
                    "contract=VISUAL-ECON-002 case={case_id}-{width}x{height} \
                     forbidden_regression={label}_meter_removed actual={preserved:?}"
                );
            }
            tracks.push((
                case_id,
                meter_track_counts(&buffer, "SUP", &supplies.to_string()),
            ));
        }

        let low = tracks[0]
            .1
            .expect("asserted low Supplies gauge must expose a track");
        let stable = tracks[1]
            .1
            .expect("asserted stable Supplies gauge must expose a track");
        assert!(
            stable.0 > low.0 && stable.1 < low.1,
            "contract=VISUAL-ECON-002 case=pressure-comparison-{width}x{height} \
             precondition=low_and_stable_use_same_profile action=compare_track_semantics \
             expected=higher_stock_has_more_fill_and_less_remainder \
             actual_low={low:?} actual_stable={stable:?}"
        );
    }
}

fn assert_nearby_context_presentation(
    case_id: &str,
    required_actions: &[&str],
    required_details: &[ContextDetailClaim<'_>],
) {
    // Contract: VISUAL-CONTEXT-001
    // Given: a station, resource node, or colonist is the active adjacent target.
    // When: the final Outpost frame presents proximity feedback and contextual actions.
    // Then: Context identifies every target/category/status; station/node Chronicle
    // feedback makes Interact discoverable; category-specific operational detail and
    // the applicable action set remain complete at both profiles.
    // Must not change: Set Production is visibly disabled with a reason; preview
    // actions without an owner-approved binding/reducer are not presented as enabled;
    // context cannot flatten unrelated targets or normal-world actions together.
    // Evidence layers: projection, buffer layout, input-state presentation; workflow
    // and PTY remain required.
    //
    // Implementation guidance:
    // - Reusable owner: one generic interaction-target/detail/action projection is
    //   populated by station, node, and colonist adapters.
    // - Integration seam: both the Chronicle region and final Context action region
    //   must survive complete screen composition at 80x24 and 60x20.
    // - Preserve: map primacy, semantic B3 chrome, truthful enabled/disabled reasons,
    //   and existing domain/catalog labels.
    // - Invalid shortcuts: do not infer identity from glyphs, place target data in log
    //   prose only, hardcode these names, create three renderers, treat `unbound` or a
    //   borrowed world key as executable, collapse duplicate names, or enable a no-op
    //   Set Production action.
    // - Closing evidence: rerun this category/profile matrix, production movement and
    //   input workflows, close/resize stale-cell evidence, canonical gate, and PTY.
    let (target, category, status, _, _, _, actions, nearby) = production_context_fixture(case_id);
    let projected_target = nearby
        .iter()
        .find(|candidate| {
            candidate.name == target
                && candidate.detail.contains(&category)
                && candidate.detail.contains(&status)
        })
        .unwrap_or_else(|| {
            panic!(
                "contract=VISUAL-CONTEXT-001 case={case_id} fixture=production-two-step-approach \
                     workflow_step=inspect_nearby_target_projection input=a frames_advanced=2 \
                     expected=target_{target:?}_category_{category:?}_status_{status:?} \
                     actual_targets={nearby:?}"
            )
        });
    let projected_detail =
        format!("{} {}", projected_target.status, projected_target.detail).to_ascii_lowercase();
    for (concept, accepted_phrases) in required_details {
        assert!(
            accepted_phrases
                .iter()
                .any(|phrase| projected_detail.contains(&phrase.to_ascii_lowercase())),
            "contract=VISUAL-CONTEXT-001 case={case_id} fixture=production-two-step-approach \
                 workflow_step=inspect_structured_target_detail expected_concept={concept:?} \
                 accepted_phrases={accepted_phrases:?} actual_target={projected_target:?}"
        );
    }
    let projected = &actions.actions;
    let inspect = projected
        .iter()
        .find(|action| action.label.contains("Inspect") && action.label.contains(&target));
    assert!(
        inspect.is_some(),
        "contract=VISUAL-CONTEXT-001 case={case_id} fixture=production-adjacent-{case_id} \
             workflow_step=inspect_production_context_projection \
             expected=enabled_Inspect_naming_{target:?} actual_actions={projected:?}"
    );
    for required in required_actions {
        assert!(
            projected
                .iter()
                .any(|action| action.label.contains(required)),
            "contract=VISUAL-CONTEXT-001 case={case_id} fixture=production-adjacent-{case_id} \
                 workflow_step=inspect_production_context_projection \
                 expected_action={required:?} actual_actions={projected:?}"
        );
    }
    if case_id == "station" {
        let set_production = projected
            .iter()
            .find(|action| action.label.contains("Set Production"));
        assert!(
            set_production.is_some_and(|action| {
                !action.enabled
                    && action
                        .denial_reason
                        .as_deref()
                        .is_some_and(|reason| reason.contains("Coming later"))
            }),
            "contract=VISUAL-CONTEXT-001 case=station fixture=production-adjacent-station \
                 workflow_step=inspect_disabled_placeholder \
                 expected=disabled_Set_Production_with_Coming_later \
                 actual_actions={projected:?}"
        );
    }
}

fn assert_nearby_context_final_composition(
    case_id: &str,
    required_actions: &[&str],
    required_details: &[ContextDetailClaim<'_>],
) {
    // Supporting contract: VISUAL-CONTEXT-001 final-composition seam.
    // Given: production movement created a category-specific nearby projection.
    // When: the complete Outpost screen is composed at each supported profile.
    // Then: Chronicle and Context retain the target, structured detail, focused
    // action set, disabled reasons, and separation from normal-world actions.
    // Must not change: map primacy, B3 panel ownership, or compact containment.
    //
    // Implementation guidance:
    // - Reusable owner: render the generic target/detail/action model through the
    //   shared Context panel; do not introduce category-specific screen renderers.
    // - Integration seam: inspect the final buffer because later widgets and compact
    //   wrapping can overwrite or clip an otherwise correct projection.
    // - Preserve: Chronicle history, focused selection, disabled reasons, and both
    //   supported profiles.
    // - Invalid shortcuts: isolated-widget green, hidden denial reasons, or flattening
    //   world actions into Context is not final-composition evidence.
    // - Closing evidence: pair with the category projection and action-truth cases,
    //   final profile observers, canonical gate, and PTY.
    let (target, category, status, map, stats, log, actions, _) =
        production_context_fixture(case_id);
    for (width, height) in [(80_u16, 24_u16), (60, 20)] {
        let buffer = render_buffer(
            "outpost",
            width,
            height,
            GameMode::Outpost,
            &map,
            &stats,
            &log,
            &actions,
        );
        let chronicle = normalized_panel_text(&buffer, "outpost", "log");
        let context = normalized_panel_text(&buffer, "outpost", "actions");
        assert!(
            chronicle.contains("Chronicle"),
            "contract=VISUAL-CONTEXT-001 case={case_id}-{width}x{height} \
                 fixture=adjacent_{case_id} action=inspect_final_chronicle_region \
                 expected=Chronicle_panel actual_chronicle={chronicle:?}"
        );
        // The ordinary station/node rows own the walk-by Chronicle contract.
        // Active-state fixtures advance real worker turns after that entry, so
        // their final observation owns current Context composition rather than
        // requiring an old entry message to remain inside the bounded log.
        for required in matches!(case_id, "station" | "node")
            .then_some([
                "NEARBY",
                target.as_str(),
                category.as_str(),
                status.as_str(),
                "Interact",
            ])
            .into_iter()
            .flatten()
        {
            assert!(
                chronicle.contains(required),
                "contract=VISUAL-CONTEXT-001 case={case_id}-{width}x{height} \
                     fixture=adjacent_{case_id} action=inspect_final_chronicle_region \
                     expected_field={required:?} actual_chronicle={chronicle:?} \
                     visual_crop=\n{}",
                buffer_text(&buffer)
            );
        }
        assert!(
            context.contains("Context")
                && context.contains(&target)
                && context.contains(&category)
                && context.contains(&status),
            "contract=VISUAL-CONTEXT-001 case={case_id}-{width}x{height} \
                 fixture=adjacent_{case_id} action=inspect_final_context_region \
                 expected=Context_for_{target:?}_category_{category:?}_status_{status:?} \
                 actual_context={context:?} \
                 visual_crop=\n{}",
            buffer_text(&buffer)
        );
        let normalized_context = context.to_ascii_lowercase();
        for (concept, accepted_phrases) in required_details {
            assert!(
                accepted_phrases
                    .iter()
                    .any(|phrase| { normalized_context.contains(&phrase.to_ascii_lowercase()) }),
                "contract=VISUAL-CONTEXT-001 case={case_id}-{width}x{height} \
                     fixture=adjacent_{case_id} action=read_applicable_target_detail \
                     expected_concept={concept:?} accepted_phrases={accepted_phrases:?} \
                     actual_context={context:?}"
            );
        }
        for required in required_actions {
            assert!(
                context.contains(required),
                "contract=VISUAL-CONTEXT-001 case={case_id}-{width}x{height} \
                     fixture=adjacent_{case_id} action=read_applicable_context_actions \
                     expected_action={required:?} actual_context={context:?}"
            );
        }
        for action in actions
            .actions
            .iter()
            .filter(|action| action.label != "Interact")
        {
            let reason = action.denial_reason.as_deref().unwrap_or_default();
            assert!(
                !reason.trim().is_empty() && context.contains(reason),
                "contract=VISUAL-CONTEXT-001 case={case_id}-{width}x{height} \
                 fixture=adjacent_{case_id} action=read_disabled_reason \
                 expected=visible_truthful_reason_for_{:?} actual_action={action:?} \
                 actual_context={context:?}",
                action.label
            );
        }
        for forbidden in ["Travel", "Build", "Rest to Day"] {
            assert!(
                !context.contains(forbidden),
                "contract=VISUAL-CONTEXT-001 case={case_id}-{width}x{height} \
                     forbidden_regression=normal_world_action_leaked_into_context \
                     unexpected={forbidden:?} actual_context={context:?}"
            );
        }
    }
}

#[test]
fn nearby_station_context_is_complete_at_supported_profiles() {
    // Primary contract: VISUAL-CONTEXT-001
    // Given/When/Then: accepted movement reaches an operational station and one
    // generic production projection carries its identity, operational/staffing
    // detail, and applicable preview actions toward final Context composition.
    // Must not change: map primacy, shared B3 chrome, domain controls, or UI9-D's
    // owner lock. Implementation guidance: extend the shared target/detail/action
    // adapters and final Context seam; do not hardcode this station, invent bindings,
    // or create category-specific renderers. Close with every registered support,
    // both profiles/PTY, neighboring input/UI tests, and the canonical gate.
    assert_nearby_context_presentation(
        "station",
        &["Inspect", "Assign Worker", "Set Production"],
        &[
            ("operational state", &["operational"]),
            ("staffing", &["unstaffed"]),
        ],
    );
}

#[test]
fn nearby_node_context_is_complete_at_supported_profiles() {
    assert_nearby_context_presentation(
        "node",
        &["Inspect", "Assign Gatherer"],
        &[("output", &["supplies"]), ("renewal state", &["renewable"])],
    );
}

#[test]
fn nearby_colonist_context_is_complete_at_supported_profiles() {
    assert_nearby_context_presentation(
        "colonist",
        &["Inspect", "Assign Task"],
        &[
            ("activity", &["idle"]),
            ("target", &["no target", "target none", "unassigned"]),
        ],
    );
}

#[test]
fn staffed_station_context_includes_worker_recipe_and_progress() {
    // Supporting contract: VISUAL-CONTEXT-001 staffed/active station state.
    // Given: paused management assigns Mara to Refine Water and real worker
    // turns reach one of two refinement turns. When: Context focuses the
    // recipe's Basic Processing station.
    // Then: the shared target projection names the worker, active recipe, and
    // progress in addition to operational/staffed state.
    // Implementation guidance:
    // - Reusable owner: enrich the generic station adapter from authoritative
    //   staffing/logistics/catalog facts; do not special-case this recipe or station.
    // - Integration seam: structured nearby detail must survive the same Context
    //   projection and final composition used by every target category.
    // - Preserve: disabled preview truth and Set Production as Coming later.
    // - Invalid shortcuts: the generic word Staffed is not worker, recipe, or
    //   progress evidence; parsing the existing colony summary is not allowed.
    // - Closing evidence: run every UI9-C state and final-profile row together.
    assert_nearby_context_presentation(
        "station-staffed",
        &["Inspect", "Assign Worker", "Set Production"],
        &[
            ("staffed worker", &["mara"]),
            ("active recipe", &["refine water"]),
            ("recipe progress", &["1/2", "1 of 2"]),
        ],
    );
}

#[test]
fn assigned_node_context_includes_worker_and_progress() {
    // Supporting contract: VISUAL-CONTEXT-001 assigned resource-node state.
    // The reusable node adapter must aggregate the authoritative gatherer and
    // direct-work progress without turning the Context renderer into a domain query.
    assert_nearby_context_presentation(
        "node-assigned",
        &["Inspect", "Assign Gatherer"],
        &[
            ("assigned worker", &["mara"]),
            ("gather progress", &["1/3", "1 of 3"]),
        ],
    );
}

#[test]
fn assigned_colonist_context_includes_target_and_progress() {
    // Supporting contract: VISUAL-CONTEXT-001 active colonist state.
    // The colonist adapter must read authoritative activity, named target, and
    // progress; fixed `Gathering` category prose is an explicit false green.
    assert_nearby_context_presentation(
        "colonist-assigned",
        &["Inspect", "Assign Task"],
        &[
            ("named target", &["water source"]),
            ("work progress", &["1/3", "1 of 3"]),
        ],
    );
}

#[test]
fn carrying_colonist_context_includes_target_and_cargo() {
    // Supporting contract: VISUAL-CONTEXT-001 carrying/logistics state.
    // Cargo and its destination are distinct authoritative facts; neither may
    // disappear behind an Idle label or be reconstructed from display prose.
    assert_nearby_context_presentation(
        "colonist-carrying",
        &["Inspect", "Assign Task"],
        &[
            ("destination", &["basic processing"]),
            ("cargo", &["raw water", "cargo 1"]),
        ],
    );
}

#[test]
fn blocked_colonist_context_includes_target_and_reason() {
    // Supporting contract: VISUAL-CONTEXT-001 blocked colonist state.
    // After paused management assigns Refine Water, removing its fixture source
    // and advancing a real worker turn makes logistics own MissingSource and a
    // typed Blocked activity. The shared adapter must expose that blocker.
    assert_nearby_context_presentation(
        "colonist-blocked",
        &["Inspect", "Assign Task"],
        &[
            ("blocked activity", &["blocked"]),
            ("blocker reason", &["target gone", "missing", "no matching"]),
        ],
    );
}

#[test]
fn a_binding_without_a_context_reducer_does_not_enable_interact() {
    // Supporting contract: VISUAL-CONTEXT-001 action truth.
    // Given: a test binding names Interact but the normal Outpost router still
    // has no Interact reducer route. Then: Context must keep Interact disabled
    // with a truthful menu/reducer reason. Binding reachability alone is not
    // executability, and this test must remain green when UI9-D later supplies
    // both facts together.
    let mut bindings = commands::CommandBindings::default();
    bindings.bind(commands::UiCommand::Interact, KeyCode::Char('x'));
    assert_eq!(
        bindings.command_for_key_in(
            &KeyCode::Char('x'),
            GameMode::Outpost,
            commands::InteractionMode::Normal,
        ),
        None,
        "fixture must prove bound-but-unroutable rather than unbound"
    );
    let (_, _, _, _, _, _, actions, _) = production_context_fixture("station-bound-interact");
    let interact = actions
        .actions
        .iter()
        .find(|action| action.label == "Interact")
        .expect("bound seam fixture must project Interact");
    let reason = interact.denial_reason.as_deref().unwrap_or_default();
    assert!(
        !interact.enabled
            && interact.key_hint == "x"
            && ["menu", "reducer", "route"]
                .iter()
                .any(|term| reason.to_ascii_lowercase().contains(term)),
        "contract=VISUAL-CONTEXT-001 case=bound-without-reducer \
         workflow_step=cross_check_applicability_reachability_executability \
         expected=disabled_x_binding_with_route_reason actual={interact:?}"
    );
}

#[test]
fn context_view_model_transports_shared_detail_without_semantic_parsing() {
    // Supporting contract: VISUAL-CONTEXT-001 shared-detail transport seam.
    // Influence table:
    // - Authoritative source: NearbyTarget.detail.
    // - Poisoned competitors: worker, recipe, and progress legacy fields.
    // - Derived output: ContextTargetVm.status.
    // - Mixed-source shortcut: strip category/staffing prose and rebuild those
    //   semantics elsewhere from the poisoned fields.
    // - Independent observer: exact semantic-segment transport below; final
    //   title/body coherence is owned by the paired composition test.
    // Implementation guidance:
    // - Reusable owner: keep domain detail owned by the nearby projection.
    // - Integration seam: transport its complete semantic segments into the
    //   Context view model; separator normalization is presentation-only.
    // - Preserve: category adapters, compact wrapping, and final action rows.
    // - Invalid shortcuts: do not parse category/staffing prefixes from the
    //   display string or recover removed segments from parallel fields.
    // - Closing evidence: pair with the final mixed-source coherence case.
    let (target, _, _, _, stats, _, _, nearby) =
        production_context_fixture("station-shared-detail");
    let projected = nearby
        .iter()
        .find(|candidate| candidate.name == target)
        .expect("shared-detail fixture must retain its nearby station");
    let context_target = stats
        .context_target
        .as_ref()
        .expect("shared-detail fixture must project a Context target");
    let expected = projected.detail.replace(" · ", " ");
    assert_eq!(
        context_target.status, expected,
        "contract=VISUAL-CONTEXT-001 case=shared-detail-transport \
         workflow_step=transport_authoritative_detail_without_semantic_parsing \
         expected_complete_normalized_detail={expected:?} \
         actual_context_target={context_target:?} projected={projected:?}"
    );
}

#[test]
fn final_context_consumes_the_shared_detail_projection_once() {
    // Supporting contract: VISUAL-CONTEXT-001 shared-owner seam.
    // Given: the production proximity projection owns a distinctive station
    // detail while parallel legacy fields carry adversarial decoy values.
    // When: the complete Outpost frame composes Context at both profiles.
    // Then: Context preserves the shared detail and never reconstructs the
    // decoys in a downstream consumer.
    // Implementation guidance:
    // - Reusable owner: one generic nearby target/detail/action projection owns
    //   domain wording and applicability for every category.
    // - Integration seam: the Context view model and screen may select, wrap,
    //   order, and style that representation but may not rederive its facts.
    // - Preserve: ordinary category/state rows and Chronicle detail.
    // - Invalid shortcuts: appending shared detail beside independently rebuilt
    //   station/node/colonist strings, or deriving the title from the poisoned
    //   worker field, remains red even when literal decoys are hidden.
    // - Closing evidence: run all active category rows and paired final buffers.
    let (target, _, _, map, stats, log, actions, nearby) =
        production_context_fixture("station-shared-detail");
    let projected = nearby
        .iter()
        .find(|candidate| candidate.name == target)
        .expect("shared-detail fixture must retain its nearby station");
    assert!(
        projected.detail.contains("Shared Detail Probe")
            && projected.status == "Unstaffed"
            && projected.worker.as_deref() == Some("Forbidden Parallel Worker"),
        "contract=VISUAL-CONTEXT-001 case=shared-detail-seam \
         precondition=adversarial_projection \
         expected=shared_unstaffed_probe_and_opposite_parallel_worker_decoy \
         actual={projected:?}"
    );

    for (width, height) in [(80_u16, 24_u16), (60, 20)] {
        let buffer = render_buffer(
            "outpost",
            width,
            height,
            GameMode::Outpost,
            &map,
            &stats,
            &log,
            &actions,
        );
        let context = normalized_panel_text(&buffer, "outpost", "actions");
        let expected_title = format!("Context · Station {}", projected.status);
        assert!(
            context.contains(&expected_title)
                && !context.contains("Context · Station Staffed")
                && context.contains("Shared Detail Probe")
                && !context.contains("Forbidden Parallel Worker")
                && !context.contains("Forbidden Parallel Recipe")
                && !context.contains("99/99"),
            "contract=VISUAL-CONTEXT-001 case=shared-detail-seam-{width}x{height} \
             workflow_step=compose_one_coherent_shared_projection \
             expected_title={expected_title:?} \
             expected=shared_probe_without_parallel_decoys_or_mixed_staffing \
             actual_context={context:?} \
             visual_crop=\n{}",
            buffer_text(&buffer)
        );
    }
}

#[test]
fn station_context_survives_final_composition_at_supported_profiles() {
    assert_nearby_context_final_composition(
        "station",
        &["Inspect", "Assign Worker", "Set Production"],
        &[
            ("operational state", &["operational"]),
            ("staffing", &["unstaffed"]),
        ],
    );
}

#[test]
fn node_context_survives_final_composition_at_supported_profiles() {
    assert_nearby_context_final_composition(
        "node",
        &["Inspect", "Assign Gatherer"],
        &[("output", &["supplies"]), ("renewal state", &["renewable"])],
    );
}

#[test]
fn colonist_context_survives_final_composition_at_supported_profiles() {
    assert_nearby_context_final_composition(
        "colonist",
        &["Inspect", "Assign Task"],
        &[
            ("activity", &["idle"]),
            ("target", &["no target", "target none", "unassigned"]),
        ],
    );
}

#[test]
fn staffed_station_recipe_progress_survives_final_composition() {
    assert_nearby_context_final_composition(
        "station-staffed",
        &["Inspect", "Assign Worker", "Set Production"],
        &[
            ("staffed worker", &["mara"]),
            ("active recipe", &["refine water"]),
            ("recipe progress", &["1/2", "1 of 2"]),
        ],
    );
}

#[test]
fn assigned_node_worker_progress_survives_final_composition() {
    assert_nearby_context_final_composition(
        "node-assigned",
        &["Inspect", "Assign Gatherer"],
        &[
            ("assigned worker", &["mara"]),
            ("gather progress", &["1/3", "1 of 3"]),
        ],
    );
}

#[test]
fn assigned_colonist_target_progress_survives_final_composition() {
    assert_nearby_context_final_composition(
        "colonist-assigned",
        &["Inspect", "Assign Task"],
        &[
            ("named target", &["water source"]),
            ("work progress", &["1/3", "1 of 3"]),
        ],
    );
}

#[test]
fn carrying_colonist_target_cargo_survives_final_composition() {
    assert_nearby_context_final_composition(
        "colonist-carrying",
        &["Inspect", "Assign Task"],
        &[
            ("destination", &["basic processing"]),
            ("cargo", &["raw water", "cargo 1"]),
        ],
    );
}

#[test]
fn blocked_colonist_reason_survives_final_composition() {
    assert_nearby_context_final_composition(
        "colonist-blocked",
        &["Inspect", "Assign Task"],
        &[
            ("blocked activity", &["blocked"]),
            ("blocker reason", &["target gone", "missing", "no matching"]),
        ],
    );
}

fn assert_passive_context_action_truth(case_id: &str) {
    // Supporting contract: VISUAL-CONTEXT-001
    // Given: UI9-C projects a real nearby station, node, or colonist while the
    // owner-locked Interact binding and Context reducer do not yet exist.
    // When: the production action projection advertises the target's preview actions.
    // Then: every preview is visibly disabled with a truthful reason; no invented
    // key hint is presented as executable.
    // Must not change: `e` remains station staffing, `c` remains task assignment,
    // and Set Production remains unavailable as Coming later.
    // Evidence layers: projection and input-state presentation.
    //
    // Implementation guidance:
    // - Reusable owner: one semantic context-action projection distinguishes domain
    //   applicability from binding reachability and executable state.
    // - Integration seam: action rows, configured bindings, and the active input
    //   reducer must agree before any row is enabled.
    // - Preserve: D-20 controls and the Section 17.2 owner lock.
    // - Invalid shortcuts: `unbound` is not enabled; `a`, `p`, or Enter cannot be
    //   borrowed from unrelated world commands; a non-empty hint is not reachability.
    // - Closing evidence: rerun this category matrix, input/Help/footer neighbors,
    //   the canonical gate, and the eventual UI9-D production workflow.
    let (_, _, _, _, _, _, actions, _) = production_context_fixture(case_id);
    let bindings = commands::CommandBindings::default();
    assert!(
        bindings.key_for(commands::UiCommand::Interact).is_none(),
        "contract=VISUAL-CONTEXT-001 case={case_id} fixture=owner-locked-bindings \
             precondition=UI9_D_not_authorized expected=Interact_unbound"
    );
    for action in &actions.actions {
        let reason = action.denial_reason.as_deref().unwrap_or_default();
        assert!(
            !action.enabled && !reason.trim().is_empty(),
            "contract=VISUAL-CONTEXT-001 case={case_id} fixture=production-adjacent-{case_id} \
                 workflow_step=cross_check_preview_reachability \
                 expected=disabled_preview_with_truthful_reason actual_action={action:?}"
        );
        if action.label == "Interact" {
            assert_eq!(
                action.key_hint, "unbound",
                "contract=VISUAL-CONTEXT-001 case={case_id} fixture=owner-locked-bindings \
                     workflow_step=read_interact_preview expected=unbound_hint \
                     actual_action={action:?}"
            );
        } else if action.label.contains("Set Production") {
            assert!(
                reason.contains("Coming later"),
                "contract=VISUAL-CONTEXT-001 case={case_id} \
                     workflow_step=read_production_placeholder \
                     expected=Coming_later actual_action={action:?}"
            );
        } else {
            let normalized = reason.to_ascii_lowercase();
            assert!(
                normalized.contains("interact")
                    || normalized.contains("menu")
                    || normalized.contains("binding"),
                "contract=VISUAL-CONTEXT-001 case={case_id} \
                     workflow_step=read_preview_denial expected=reason_naming_interact_menu_or_binding \
                     actual_action={action:?}"
            );
        }
    }
}

#[test]
fn passive_context_never_advertises_unroutable_actions_as_enabled() {
    assert_passive_context_action_truth("station");
}

#[test]
fn passive_node_context_never_advertises_unroutable_actions_as_enabled() {
    assert_passive_context_action_truth("node");
}

#[test]
fn passive_colonist_context_never_advertises_unroutable_actions_as_enabled() {
    assert_passive_context_action_truth("colonist");
}

#[test]
fn duplicate_named_nearby_targets_remain_distinguishable_in_context() {
    // Supporting contract: VISUAL-CONTEXT-001
    // Given: two stations with the same player-facing name enter range together.
    // When: production proximity and Context projections are built.
    // Then: both stable targets remain present in deterministic order, the passive
    // Context identifies its focus with player-facing selector data, and only that
    // focused target contributes actions.
    // Must not change: deterministic focus, map primacy, or supported-profile
    // containment.
    // Evidence layers: production projection and final buffer layout.
    //
    // Implementation guidance:
    // - Reusable owner: the nearby projection owns complete stable target selectors;
    //   the focused Context projection owns exactly one target's details/actions.
    // - Integration seam: final Context composition shows focus, total target count,
    //   and player-facing disambiguation without printing raw ECS identity.
    // - Preserve: deterministic ordering, map primacy, and both supported profiles.
    // - Invalid shortcuts: one Inspect/action set per nearby target flattens targets
    //   and is not a picker; display name alone does not distinguish duplicates.
    // - Closing evidence: run this independently with simultaneous Chronicle
    //   aggregation, category/action truth tests, final profiles, gate, and PTY.
    let mut app = production_outpost_runtime();
    let player = app
        .world_mut()
        .query_filtered::<Entity, With<Player>>()
        .iter(app.world())
        .next()
        .expect("Foundation player must exist");
    let occupied = app
        .world_mut()
        .query::<(Entity, &Position)>()
        .iter(app.world())
        .filter_map(|(entity, position)| (entity != player).then_some(*position))
        .collect::<Vec<_>>();
    let (start, destination, first_station, second_station) = {
        let map = &app.world().resource::<bd_core::spatial::OutpostState>().map;
        (2..map.height - 2)
            .flat_map(|y| (2..map.width - 2).map(move |x| Position { x, y }))
            .find_map(|destination| {
                let start = Position {
                    x: destination.x - 1,
                    y: destination.y,
                };
                let first = Position {
                    x: destination.x,
                    y: destination.y - 1,
                };
                let second = Position {
                    x: destination.x,
                    y: destination.y + 1,
                };
                let clear = [
                    start,
                    destination,
                    first,
                    second,
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
                    .then_some((start, destination, first, second))
            })
            .expect("duplicate-name context fixture needs one clear cross")
    };
    for position in [first_station, second_station] {
        app.world_mut().spawn((
            Station,
            StationType::Custom(1),
            Name("Basic Processing".into()),
            position,
            BlocksMovement,
            EntityScope::ColonyPersistent,
        ));
    }
    app.world_mut().entity_mut(player).insert(start);
    app.world_mut()
        .insert_resource(bd_core::colony::proximity::NearbyInteractables::default());
    app.world_mut().insert_resource(GameLog::default());
    app.update();
    {
        let mut messages = app.world_mut().resource_mut::<Messages<KeyMessage>>();
        messages.write(KeyMessage(KeyEvent::new_with_kind(
            KeyCode::Char('d'),
            KeyModifiers::NONE,
            KeyEventKind::Press,
        )));
        messages.write(KeyMessage(KeyEvent::new_with_kind(
            KeyCode::Char('d'),
            KeyModifiers::NONE,
            KeyEventKind::Release,
        )));
    }
    app.update();
    app.update();
    assert_eq!(app.world().get::<Position>(player), Some(&destination));

    let targets = app
        .world()
        .resource::<bd_core::colony::proximity::NearbyInteractables>()
        .targets
        .iter()
        .filter(|target| target.name == "Basic Processing")
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(
        targets.len(),
        2,
        "contract=VISUAL-CONTEXT-001 case=duplicate-station-name \
         workflow_step=inspect_complete_target_list expected=two_stable_targets \
         actual_targets={targets:?}"
    );
    assert_ne!(targets[0].identity(), targets[1].identity());
    assert!(
        (
            targets[0].category,
            targets[0].name.as_str(),
            targets[0].position.y,
            targets[0].position.x,
        ) < (
            targets[1].category,
            targets[1].name.as_str(),
            targets[1].position.y,
            targets[1].position.x,
        ),
        "contract=VISUAL-CONTEXT-001 case=duplicate-station-name \
         workflow_step=inspect_target_order expected=stable_identity_order \
         actual_targets={targets:?}"
    );

    let context_actions = &app.world().resource::<ActionListViewModel>().actions;
    let inspect_labels = context_actions
        .iter()
        .filter(|action| action.label.contains("Inspect Basic Processing"))
        .map(|action| action.label.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        inspect_labels.len(),
        1,
        "contract=VISUAL-CONTEXT-001 case=duplicate-station-name \
         workflow_step=inspect_focused_actions expected=one_focused_Inspect_action \
         actual_labels={inspect_labels:?}"
    );
    for action_label in ["Assign Worker", "Set Production"] {
        assert_eq!(
            context_actions
                .iter()
                .filter(|action| action.label.contains(action_label))
                .count(),
            1,
            "contract=VISUAL-CONTEXT-001 case=duplicate-station-name \
             workflow_step=inspect_focused_actions expected=one_{action_label:?}_for_focus \
             actual_actions={context_actions:?}"
        );
    }
    let focused = app
        .world()
        .resource::<StatsViewModel>()
        .context_target
        .as_ref()
        .expect("duplicate target fixture must expose one focused context target");
    assert_eq!(focused.name, targets[0].name);
    assert_eq!(focused.category, targets[0].category.label());

    let focused_position = targets[0].position;
    let accepted_focus_cues = [
        format!("{},{}", focused_position.x, focused_position.y),
        format!("{}, {}", focused_position.x, focused_position.y),
        format!("{}:{}", focused_position.x, focused_position.y),
        "north".to_string(),
        "above".to_string(),
    ];
    assert!(
        inspect_labels[0]
            .to_ascii_lowercase()
            .contains("basic processing"),
        "contract=VISUAL-CONTEXT-001 case=duplicate-station-name \
         workflow_step=inspect_focus_label expected=focused_target_name \
         actual_labels={inspect_labels:?}"
    );

    let map = app.world().resource::<MapViewModel>().clone();
    let stats = app.world().resource::<StatsViewModel>().clone();
    let log = app.world().resource::<LogViewModel>().clone();
    let actions = app.world().resource::<ActionListViewModel>().clone();
    for (width, height) in [(80, 24), (60, 20)] {
        let buffer = render_buffer(
            "outpost",
            width,
            height,
            GameMode::Outpost,
            &map,
            &stats,
            &log,
            &actions,
        );
        let context = normalized_panel_text(&buffer, "outpost", "actions");
        let normalized = context.to_ascii_lowercase();
        assert!(
            context.contains(&inspect_labels[0])
                && (normalized.contains("1/2")
                    || normalized.contains("1 of 2")
                    || normalized.contains("2 nearby")
                    || normalized.contains("2 targets"))
                && accepted_focus_cues
                    .iter()
                    .any(|cue| normalized.contains(&cue.to_ascii_lowercase())),
            "contract=VISUAL-CONTEXT-001 case=duplicate-station-name-{width}x{height} \
             workflow_step=read_focused_duplicate_selector \
             expected=one_focused_action_plus_target_count_plus_player_visible_location \
             accepted_focus_cues={accepted_focus_cues:?} actual_context={context:?} \
             visual_crop=\n{}",
            buffer_text(&buffer)
        );
    }
}
