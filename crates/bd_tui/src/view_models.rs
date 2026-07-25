//! View models — plain data structs between ECS and rendering.

use bevy_app::App;
use bevy_ecs::{
    prelude::*,
    query::With,
    system::{Query, Res, ResMut},
};
use serde::{Deserialize, Serialize};

use bd_core::{
    BdSet,
    components::{BlocksMovement, Name, Player, Position, Tile},
    gamelog::{GameLog, LogLevel},
    inventory::Item,
    map::SmokeMap,
    pools::Pools,
    relationships::{ContainedIn, EquippedBy},
    signals::PoolKind,
};

#[derive(Resource, Debug, Clone, Default)]
pub struct StatsViewModel {
    pub hp_current: i32,
    pub hp_max: i32,
    pub ap_current: i32,
    pub ap_max: i32,
    pub supplies: i32,
    pub faith: i32,
    pub materials: i32,
    pub wild_plants: i32,
    pub stored_items: Vec<(String, u32)>,
    pub run_outcome: bd_core::session::RunOutcome,
    pub extracted_loot: u32,
    pub day: u64,
    pub party_names: Vec<String>,
    /// Compact faction standings: (label, value, status_text).
    pub faction_standings: Vec<(String, i32, String)>,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct LogViewModel {
    pub entries: Vec<LogEntryVm>,
}

#[derive(Debug, Clone)]
pub struct LogEntryVm {
    pub message: String,
    pub level: LogLevel,
}

#[derive(Debug, Clone)]
pub struct ActionItemVm {
    pub label: String,
    pub key_hint: String,
    pub enabled: bool,
    pub denial_reason: Option<String>,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct ActionListViewModel {
    pub actions: Vec<ActionItemVm>,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct MapViewModel {
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<Tile>,
    pub player_pos: Option<Position>,
    pub enemy_positions: Vec<Position>,
    /// Per-enemy glyph characters at corresponding indices to enemy_positions.
    /// When empty, all enemies render as default 'E'.
    pub enemy_glyphs: Vec<(Position, char)>,
    /// Survivor positions and glyphs for outpost map rendering.
    pub survivor_glyphs: Vec<(Position, char)>,
    /// Station positions and glyphs for outpost map rendering.
    pub station_glyphs: Vec<(Position, char)>,
    /// Gabriel position and glyph for outpost map rendering. None if not present.
    pub gabriel_glyph: Option<(Position, char)>,
    /// Resource node positions and glyphs for outpost map rendering.
    pub resource_glyphs: Vec<(Position, char)>,
    /// Exit tile positions and glyphs (shelter gate, dungeon exits).
    pub exit_glyphs: Vec<(Position, char)>,
    /// Build ghost cursor position and glyph for outpost map rendering.
    pub build_ghost: Option<(Position, char)>,
    /// Build menu entries with highlight index (None if menu closed).
    pub build_menu: Option<BuildMenuVm>,
}

/// Build menu data for the station selection popup.
#[derive(Debug, Clone)]
pub struct BuildMenuVm {
    pub options: Vec<(String, i32)>, // (label, supply_cost)
    pub selected: usize,
}

#[derive(Resource, Debug, Clone, Default)]
#[allow(dead_code)]
pub struct ActorPanelViewModel {
    pub entity_name: Option<String>,
    pub hp: Option<(i32, i32)>,
}

/// View model for inventory/container display.
#[derive(Resource, Debug, Clone, Default)]
pub struct ContainerViewModel {
    pub items: Vec<ItemEntryVm>,
}

#[derive(Debug, Clone)]
pub struct ItemEntryVm {
    pub name: String,
    pub equipped: bool,
    pub usable: bool,
}

// ── Event view model ──

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct HelpViewModel {
    pub keys: Vec<(String, String)>,
}

impl Default for HelpViewModel {
    fn default() -> Self {
        Self { keys: Vec::new() }
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct EventViewModel {
    pub speaker: String,
    pub text: String,
    pub choices: Vec<String>,
    pub active: bool,
}

pub(crate) fn register_view_models(app: &mut App) {
    app.insert_resource(StatsViewModel::default());
    app.insert_resource(LogViewModel::default());
    app.insert_resource(ActionListViewModel::default());
    app.insert_resource(MapViewModel::default());
    app.insert_resource(ActorPanelViewModel::default());
    app.insert_resource(ContainerViewModel::default());
    app.insert_resource(HelpViewModel::default());
    app.insert_resource(EventViewModel::default());
    app.add_systems(
        bevy_app::Update,
        (
            build_stats_vm,
            build_log_vm,
            build_action_list_vm,
            build_map_vm,
            build_container_vm,
            build_party_vm,
            build_event_vm,
            build_help_vm,
        )
            .in_set(BdSet::ViewModelBuild),
    );
}

fn build_stats_vm(
    player_pools: Query<&Pools, With<Player>>,
    mut vm: ResMut<StatsViewModel>,
    colony_res: Res<bd_core::colony::production::ColonyResources>,
    colony_storage: Res<bd_core::colony::production::ColonyStorage>,
    game_time: Res<bd_core::time::GameTime>,
    session: Res<bd_core::session::RunSession>,
    faction_rep: Option<Res<bd_core::factions::FactionReputation>>,
) {
    if let Ok(pools) = player_pools.single() {
        vm.hp_current = pools.get(PoolKind::Health).map_or(0, |p| p.current);
        vm.hp_max = pools.get(PoolKind::Health).map_or(0, |p| p.max);
        vm.ap_current = pools.get(PoolKind::ActionPoints).map_or(0, |p| p.current);
        vm.ap_max = pools.get(PoolKind::ActionPoints).map_or(0, |p| p.max);
    }
    vm.supplies = colony_res
        .pools
        .get(PoolKind::Supplies)
        .map_or(0, |p| p.current);
    vm.faith = colony_res
        .pools
        .get(PoolKind::Faith)
        .map_or(0, |p| p.current);
    vm.materials = colony_res
        .pools
        .get(PoolKind::Materials)
        .map_or(0, |p| p.current);
    vm.wild_plants = colony_res
        .pools
        .get(PoolKind::WildPlants)
        .map_or(0, |p| p.current);
    vm.stored_items = colony_storage
        .items
        .iter()
        .map(|(id, count)| (id.clone(), *count))
        .collect();
    vm.day = game_time.day;
    vm.run_outcome = session.outcome;
    vm.extracted_loot = session.extracted_loot;

    // P17-D: Faction standings
    vm.faction_standings.clear();
    let Some(faction_rep) = faction_rep else {
        return;
    };
    for faction in bd_core::factions::ALL_FACTIONS {
        let val = faction_rep.get(faction);
        let status = bd_core::factions::faction_status(val);
        let label = match faction {
            PoolKind::RepPuritans => "Puritans",
            PoolKind::RepWanderers => "Wanderers",
            PoolKind::RepBrokenChoir => "BrokenChoir",
            PoolKind::RepDemons => "Demons",
            PoolKind::RepHumanSettlements => "Settlements",
            _ => "???",
        };
        let status_text = match status {
            bd_core::factions::FactionStatus::Hostile => "H",
            bd_core::factions::FactionStatus::Neutral => "N",
            bd_core::factions::FactionStatus::Friendly => "F",
            bd_core::factions::FactionStatus::Allied => "A",
        };
        vm.faction_standings
            .push((label.to_string(), val, status_text.to_string()));
    }
}

fn build_party_vm(
    survivors: Query<
        (&Name, Option<&bd_core::spatial::EntityScope>),
        With<bd_core::colony::survivors::Survivor>,
    >,
    mode: Res<bd_core::spatial::GameMode>,
    mut vm: ResMut<StatsViewModel>,
) {
    vm.party_names = survivors
        .iter()
        .filter(|(_, scope)| scope_active(*scope, *mode))
        .map(|(name, _)| name.0.clone())
        .collect();
}

fn build_log_vm(log: Res<GameLog>, mut vm: ResMut<LogViewModel>) {
    vm.entries = log
        .iter()
        .map(|e| LogEntryVm {
            message: e.message.clone(),
            level: e.level,
        })
        .collect();
}

fn build_action_list_vm(
    player: Query<
        (
            Entity,
            &Position,
            &Pools,
            Option<&bd_core::spatial::EntityScope>,
        ),
        With<Player>,
    >,
    enemies: Query<
        (&Position, Option<&bd_core::spatial::EntityScope>),
        (With<BlocksMovement>, Without<Player>),
    >,
    mode: Res<bd_core::spatial::GameMode>,
    map: Res<SmokeMap>,
    outpost: Res<bd_core::spatial::OutpostState>,
    items: Query<
        (
            Option<&Position>,
            Option<&bd_core::inventory::Usable>,
            Option<&ContainedIn>,
            Option<&bd_core::spatial::EntityScope>,
        ),
        With<Item>,
    >,
    survivors: Query<
        (
            &bd_core::colony::survivors::SurvivorTask,
            Option<&bd_core::spatial::EntityScope>,
        ),
        With<bd_core::colony::survivors::Survivor>,
    >,
    stations: Query<
        Option<&bd_core::spatial::EntityScope>,
        With<bd_core::colony::stations::Station>,
    >,
    colony_resources: Res<bd_core::colony::production::ColonyResources>,
    bindings: Res<crate::commands::CommandBindings>,
    mut vm: ResMut<ActionListViewModel>,
) {
    let Some((player_entity, pp, pools, _)) = player
        .iter()
        .find(|(_, _, _, scope)| scope_active(*scope, *mode))
    else {
        vm.actions.clear();
        return;
    };
    let ap = pools.get(PoolKind::ActionPoints).map_or(0, |p| p.current);
    let has_ap = ap >= 1;
    let enemy_near = enemies
        .iter()
        .filter(|(_, scope)| scope_active(*scope, *mode))
        .any(|(position, _)| {
            (position.x - pp.x).unsigned_abs() + (position.y - pp.y).unsigned_abs() <= 1
        });
    let active_map = if *mode == bd_core::spatial::GameMode::Outpost {
        &outpost.map
    } else {
        &map
    };
    let can_move = [
        (pp.x, pp.y - 1),
        (pp.x, pp.y + 1),
        (pp.x - 1, pp.y),
        (pp.x + 1, pp.y),
    ]
    .into_iter()
    .any(|(x, y)| {
        active_map.is_walkable(x, y)
            && !enemies.iter().any(|(position, scope)| {
                scope_active(scope, *mode) && position.x == x && position.y == y
            })
    });
    let item_here = items.iter().any(|(position, _, _, scope)| {
        scope_active(scope, *mode) && position.is_some_and(|position| *position == *pp)
    });
    let usable_item = items.iter().any(|(_, usable, contained, scope)| {
        scope_active(scope, *mode)
            && usable.is_some()
            && contained.is_some_and(|container| container.0 == player_entity)
    });
    let survivor_available = survivors
        .iter()
        .any(|(_, scope)| scope_active(scope, *mode));
    let idle_survivor_available = survivors.iter().any(|(task, scope)| {
        scope_active(scope, *mode) && matches!(task, bd_core::colony::survivors::SurvivorTask::Idle)
    });
    let station_available = stations.iter().any(|scope| scope_active(scope, *mode));
    let minimum_build_cost = bd_core::colony::stations::default_station_blueprints()
        .iter()
        .map(|blueprint| blueprint.build_cost_supplies)
        .min()
        .unwrap_or_default();
    let supplies = colony_resources
        .pools
        .get(PoolKind::Supplies)
        .map_or(0, |pool| pool.current);
    let availability = crate::commands::ActionAvailability {
        mode: *mode,
        has_ap,
        enemy_in_range: enemy_near,
        can_move,
        item_here,
        usable_item,
        can_build: supplies >= minimum_build_cost,
        survivor_available,
        station_available,
    };
    let mut projections = crate::commands::action_panel(&bindings, availability);
    if let Some(staff) = projections
        .iter_mut()
        .find(|action| action.command == crate::commands::UiCommand::AssignStation)
        && !idle_survivor_available
    {
        staff.enabled = false;
        staff.denial_reason = Some("No idle survivor");
    }

    vm.actions = projections
        .into_iter()
        .map(|action| ActionItemVm {
            label: action.label.into(),
            key_hint: action.key,
            enabled: action.enabled,
            denial_reason: action.denial_reason.map(str::to_owned),
        })
        .collect();
}

fn build_help_vm(
    bindings: Res<crate::commands::CommandBindings>,
    mode: Res<bd_core::spatial::GameMode>,
    build_ghost: Res<bd_core::colony::stations::BuildGhostState>,
    build_menu: Res<bd_core::colony::stations::BuildMenuState>,
    mut vm: ResMut<HelpViewModel>,
) {
    let interaction = if *mode == bd_core::spatial::GameMode::GameOver {
        crate::commands::InteractionMode::GameOver
    } else if build_ghost.active || build_menu.active {
        crate::commands::InteractionMode::Build
    } else {
        crate::commands::InteractionMode::Normal
    };
    vm.keys = crate::commands::help_entries(&bindings, *mode, interaction)
        .into_iter()
        .map(|entry| (entry.key, entry.description.into()))
        .collect();
}

fn build_map_vm(
    map: Res<SmokeMap>,
    player_pos: Query<(&Position, Option<&bd_core::spatial::EntityScope>), With<Player>>,
    enemies: Query<
        (
            &Position,
            Option<&bd_core::components::Name>,
            Option<&bd_core::spatial::EntityScope>,
        ),
        (With<BlocksMovement>, Without<Player>),
    >,
    survivors: Query<
        (
            &Position,
            Option<&bd_core::components::Name>,
            &bd_core::colony::survivors::SurvivorTask,
            Option<&bd_core::spatial::EntityScope>,
        ),
        With<bd_core::colony::survivors::Survivor>,
    >,
    stations: Query<(
        &Position,
        &bd_core::colony::stations::StationType,
        Option<&bd_core::spatial::EntityScope>,
    )>,
    gabriel_q: Query<
        (&Position, Option<&bd_core::spatial::EntityScope>),
        With<bd_core::components::Gabriel>,
    >,
    resource_nodes: Query<(
        &Position,
        &bd_core::components::ResourceNode,
        Option<&bd_core::spatial::EntityScope>,
    )>,
    exit_tiles: Query<
        (&Position, Option<&bd_core::spatial::EntityScope>),
        With<bd_core::components::ExitTile>,
    >,
    build_ghost: Res<bd_core::colony::stations::BuildGhostState>,
    build_menu: Res<bd_core::colony::stations::BuildMenuState>,
    mut vm: ResMut<MapViewModel>,
    mode: Res<bd_core::spatial::GameMode>,
    outpost: Res<bd_core::spatial::OutpostState>,
) {
    // Use shelter map in outpost mode, dungeon map otherwise
    let active_map = if *mode == bd_core::spatial::GameMode::Outpost {
        &outpost.map
    } else {
        &map
    };

    vm.width = active_map.width;
    vm.height = active_map.height;
    vm.tiles.clear();
    for y in 0..active_map.height {
        for x in 0..active_map.width {
            vm.tiles.push(active_map.get(x, y).unwrap_or(Tile::Wall));
        }
    }
    vm.player_pos = player_pos
        .iter()
        .find(|(_, scope)| scope_active(*scope, *mode))
        .map(|(position, _)| *position);
    vm.enemy_positions.clear();
    vm.enemy_glyphs.clear();
    // Only collect enemies in tactical/dungeon mode — shelter has no enemies
    if *mode != bd_core::spatial::GameMode::Outpost {
        for (pos, name, scope) in enemies.iter() {
            if !scope_active(scope, *mode) {
                continue;
            }
            vm.enemy_positions.push(*pos);
            let glyph = name.map_or('E', |n| match n.0.as_str() {
                "Rat" => 'r',
                "Skeleton" => 'S',
                "Boss" => 'B',
                _ => 'E',
            });
            vm.enemy_glyphs.push((*pos, glyph));
        }
    }
    vm.survivor_glyphs.clear();
    for (pos, _name, task, scope) in survivors.iter() {
        if !scope_active(scope, *mode) {
            continue;
        }
        let glyph = match task {
            bd_core::colony::survivors::SurvivorTask::Idle => 'A',
            bd_core::colony::survivors::SurvivorTask::Gathering => 'G',
            bd_core::colony::survivors::SurvivorTask::Defending => 'D',
            bd_core::colony::survivors::SurvivorTask::Resting => 'R',
            _ => 'A',
        };
        vm.survivor_glyphs.push((*pos, glyph));
    }
    vm.station_glyphs.clear();
    for (pos, stype, scope) in stations.iter() {
        if !scope_active(scope, *mode) {
            continue;
        }
        let glyph = match stype {
            bd_core::colony::stations::StationType::Stove => 'F',
            bd_core::colony::stations::StationType::Altar => 'A',
            bd_core::colony::stations::StationType::Workshop => 'W',
            bd_core::colony::stations::StationType::Bed => 'B',
            bd_core::colony::stations::StationType::Storage => 'S',
        };
        vm.station_glyphs.push((*pos, glyph));
    }

    // P15-C: Gabriel glyph on shelter map
    vm.gabriel_glyph = gabriel_q
        .iter()
        .find(|(_, scope)| scope_active(*scope, *mode))
        .map(|(position, _)| (*position, 'G'));

    // P22-D: Resource node glyphs on shelter map
    vm.resource_glyphs.clear();
    for (pos, node, scope) in resource_nodes.iter() {
        if !scope_active(scope, *mode) {
            continue;
        }
        let glyph = match node.kind {
            bd_core::components::ResourceNodeType::Trees => 'T',
            bd_core::components::ResourceNodeType::WaterSource => 'W',
            bd_core::components::ResourceNodeType::WildPlants => 'P',
        };
        vm.resource_glyphs.push((*pos, glyph));
    }

    // P3-A: Exit tile glyphs on the shelter map (gate, dungeon exits)
    vm.exit_glyphs.clear();
    for (pos, scope) in exit_tiles.iter() {
        if !scope_active(scope, *mode) {
            continue;
        }
        vm.exit_glyphs.push((*pos, '>'));
    }

    // P2-C: Build ghost cursor on shelter map
    vm.build_ghost = if build_ghost.active {
        let glyph = match build_ghost.station_type {
            Some(bd_core::colony::stations::StationType::Stove) => 'f',
            Some(bd_core::colony::stations::StationType::Altar) => 'a',
            Some(bd_core::colony::stations::StationType::Workshop) => 'w',
            Some(bd_core::colony::stations::StationType::Bed) => 'b',
            Some(bd_core::colony::stations::StationType::Storage) => 's',
            None => '?',
        };
        Some((build_ghost.cursor, glyph))
    } else {
        None
    };

    // P2: Build menu popup
    vm.build_menu = if build_menu.active {
        let bps = bd_core::colony::stations::default_station_blueprints();
        let options: Vec<(String, i32)> = bps
            .iter()
            .map(|bp| (bp.label.to_string(), bp.build_cost_supplies))
            .collect();
        Some(BuildMenuVm {
            options,
            selected: build_menu.selected,
        })
    } else {
        None
    };
}

/// Build the inventory container view model for the player.
fn build_container_vm(
    player: Query<(Entity, Option<&bd_core::spatial::EntityScope>), With<Player>>,
    items: Query<(
        Entity,
        Option<&Name>,
        Option<&Item>,
        Option<&bd_core::spatial::EntityScope>,
    )>,
    contained_in: Query<&ContainedIn>,
    equipped_by: Query<&EquippedBy>,
    mode: Res<bd_core::spatial::GameMode>,
    mut vm: ResMut<ContainerViewModel>,
) {
    let Some((player_entity, _)) = player.iter().find(|(_, scope)| scope_active(*scope, *mode))
    else {
        vm.items.clear();
        return;
    };

    // Find items in player's inventory (ContainedIn → player)
    let mut entries: Vec<ItemEntryVm> = Vec::new();
    for (entity, name, _item, scope) in items.iter() {
        if !scope_active(scope, *mode) {
            continue;
        }
        // Check if this item belongs to the player
        let is_contained = contained_in
            .get(entity)
            .ok()
            .is_some_and(|c| c.0 == player_entity);
        let is_equipped = equipped_by
            .get(entity)
            .ok()
            .is_some_and(|e| e.0 == player_entity);

        if is_contained || is_equipped {
            entries.push(ItemEntryVm {
                name: name
                    .map(|n| n.0.clone())
                    .unwrap_or_else(|| "Unknown".into()),
                equipped: is_equipped,
                usable: is_contained, // contained items can be used
            });
        }
    }

    vm.items = entries;
}

fn scope_active(
    scope: Option<&bd_core::spatial::EntityScope>,
    mode: bd_core::spatial::GameMode,
) -> bool {
    scope.is_none_or(|scope| scope.is_active(mode))
}

/// Build the event view model from the CurrentEvent resource.
fn build_event_vm(
    current: Option<Res<bd_core::events::CurrentEvent>>,
    registry: Option<Res<bd_core::events::EventRegistry>>,
    mut vm: ResMut<EventViewModel>,
) {
    let (Some(current), Some(registry)) = (current, registry) else {
        vm.active = false;
        return;
    };
    if !current.is_active() {
        vm.active = false;
        return;
    }
    if let Some(event_def) = registry.get(&current.event_id) {
        if let Some(node) = event_def.nodes.get(&current.node_id) {
            vm.speaker = node.speaker.clone();
            vm.text = node.text.clone();
            vm.choices = node.choices.iter().map(|c| c.label.clone()).collect();
            vm.active = true;
            return;
        }
    }
    vm.active = false;
}

#[cfg(test)]
mod tests {
    use super::*;
    use bd_core::pools::Pool;
    use bevy_app::App;

    fn test_app() -> App {
        let mut app = App::new();
        // Minimal plugins needed for schedule execution
        app.add_plugins(bd_core::BdCorePlugin);
        app.insert_resource(bd_core::colony::production::ColonyResources::default());
        app.insert_resource(crate::commands::CommandBindings::default());
        *app.world_mut().resource_mut::<bd_core::spatial::GameMode>() =
            bd_core::spatial::GameMode::Tactical;
        // Insert all view model resources
        app.insert_resource(StatsViewModel::default());
        app.insert_resource(ActionListViewModel::default());
        app.insert_resource(MapViewModel::default());
        app.add_systems(
            bevy_app::Update,
            (build_stats_vm, build_action_list_vm, build_map_vm).in_set(BdSet::ViewModelBuild),
        );
        app
    }

    #[test]
    fn stats_view_model_contains_hp_ap() {
        let mut app = test_app();
        app.world_mut().spawn((
            Player,
            Position { x: 5, y: 5 },
            Pools::new(vec![
                Pool::new(PoolKind::Health, 15, 0, 20),
                Pool::new(PoolKind::ActionPoints, 2, 0, 3),
            ]),
        ));
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        app.update();
        let vm = app.world().resource::<StatsViewModel>();
        assert_eq!(vm.hp_current, 15);
        assert_eq!(vm.hp_max, 20);
        assert_eq!(vm.ap_current, 2);
        assert_eq!(vm.ap_max, 3);
    }

    #[test]
    fn action_list_contains_move_wait_attack_guard() {
        let mut app = test_app();
        app.world_mut().spawn((
            Player,
            Position { x: 5, y: 5 },
            Pools::new(vec![Pool::new(PoolKind::ActionPoints, 3, 0, 3)]),
        ));
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        app.update();
        let labels: Vec<&str> = app
            .world()
            .resource::<ActionListViewModel>()
            .actions
            .iter()
            .map(|a| a.label.as_str())
            .collect();
        assert!(labels.contains(&"Move"));
        assert!(labels.contains(&"Wait"));
        assert!(labels.contains(&"Attack"));
        assert!(labels.contains(&"Guard"));
    }

    #[test]
    fn disabled_action_contains_denial_reason() {
        let mut app = test_app();
        app.world_mut().spawn((
            Player,
            Position { x: 5, y: 5 },
            Pools::new(vec![Pool::new(PoolKind::ActionPoints, 0, 0, 3)]),
        ));
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        app.update();
        let vm = app.world().resource::<ActionListViewModel>();
        let attack = vm.actions.iter().find(|a| a.label == "Attack").unwrap();
        assert!(!attack.enabled);
        assert!(attack.denial_reason.is_some());
    }

    #[test]
    fn map_view_model_contains_tiles() {
        let mut app = test_app();
        app.world_mut().spawn((
            Player,
            Position { x: 5, y: 5 },
            Pools::new(vec![Pool::new(PoolKind::ActionPoints, 3, 0, 3)]),
        ));
        app.world_mut()
            .insert_resource(SmokeMap::default_smoke_map());
        app.update();
        let vm = app.world().resource::<MapViewModel>();
        // Tactical mode projects the active dungeon map.
        assert_eq!(vm.width, 20);
        assert_eq!(vm.player_pos, Some(Position { x: 5, y: 5 }));
    }

    #[test]
    fn widgets_can_render_from_view_models() {
        let mut app = test_app();
        app.world_mut().spawn((
            Player,
            Position { x: 5, y: 5 },
            Pools::new(vec![
                Pool::new(PoolKind::Health, 20, 0, 20),
                Pool::new(PoolKind::ActionPoints, 3, 0, 3),
            ]),
        ));
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        app.update();
        let stats = app.world().resource::<StatsViewModel>();
        assert!(stats.hp_max > 0);
        assert!(stats.ap_max > 0);
        let actions = app.world().resource::<ActionListViewModel>();
        assert!(!actions.actions.is_empty());
        let map = app.world().resource::<MapViewModel>();
        assert!(map.width > 0);
    }

    #[test]
    fn enemy_glyph_maps_by_name() {
        let mut app = test_app();
        // Spawn a Rat enemy at (3,3)
        let _rat = app
            .world_mut()
            .spawn((
                Position { x: 3, y: 3 },
                bd_core::components::BlocksMovement,
                bd_core::components::Name("Rat".into()),
            ))
            .id();
        // Spawn a Skeleton at (5,5)
        let _skeleton = app
            .world_mut()
            .spawn((
                Position { x: 5, y: 5 },
                bd_core::components::BlocksMovement,
                bd_core::components::Name("Skeleton".into()),
            ))
            .id();
        // Spawn an unnamed enemy at (7,7)
        let _unknown = app
            .world_mut()
            .spawn((Position { x: 7, y: 7 }, bd_core::components::BlocksMovement))
            .id();

        // Ensure map resource is set (test_app may have left default)
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));

        app.update();

        let vm = app.world().resource::<MapViewModel>();
        assert_eq!(
            vm.enemy_positions.len(),
            3,
            "Should find 3 enemy positions, got {:?}",
            vm.enemy_positions
        );
        // Find the glyph for the Rat at (3,3)
        let rat_glyph = vm
            .enemy_glyphs
            .iter()
            .find(|(p, _)| p.x == 3 && p.y == 3)
            .map(|(_, g)| *g);
        assert_eq!(rat_glyph, Some('r'), "Rat should map to glyph 'r'");
        // Find the glyph for the Skeleton at (5,5)
        let skel_glyph = vm
            .enemy_glyphs
            .iter()
            .find(|(p, _)| p.x == 5 && p.y == 5)
            .map(|(_, g)| *g);
        assert_eq!(skel_glyph, Some('S'), "Skeleton should map to glyph 'S'");
        // Unknown enemy should be 'E'
        let unknown_glyph = vm
            .enemy_glyphs
            .iter()
            .find(|(p, _)| p.x == 7 && p.y == 7)
            .map(|(_, g)| *g);
        assert_eq!(
            unknown_glyph,
            Some('E'),
            "Unknown enemy should default to 'E'"
        );
    }
}
