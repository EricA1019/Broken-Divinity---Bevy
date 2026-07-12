//! View models — plain data structs between ECS and rendering.

use bevy_app::App;
use bevy_ecs::{
    prelude::*,
    query::With,
    system::{Query, Res, ResMut},
};

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
    pub day: u64,
    pub party_names: Vec<String>,
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

#[derive(Resource, Debug, Clone)]
pub struct HelpViewModel {
    pub keys: Vec<(String, String)>,
}

impl Default for HelpViewModel {
    fn default() -> Self {
        Self {
            keys: vec![
                ("WASD".into(), "Move".into()),
                (".".into(), "Wait (restore 1 AP)".into()),
                ("f".into(), "Attack nearest enemy".into()),
                ("g".into(), "Guard (reduce damage)".into()),
                ("t".into(), "Travel to dungeon".into()),
                ("r".into(), "Return to outpost".into()),
                ("i".into(), "Toggle inventory".into()),
                ("b".into(), "Build / cycle station".into()),
                ("a".into(), "Assign survivor task".into()),
                ("?".into(), "Toggle help".into()),
                ("Esc / q".into(), "Cancel build / Quit".into()),
            ],
        }
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
        )
            .in_set(BdSet::ViewModelBuild),
    );
}

fn build_stats_vm(
    player_pools: Query<&Pools, With<Player>>,
    mut vm: ResMut<StatsViewModel>,
    colony_res: Res<bd_core::colony::production::ColonyResources>,
    game_time: Res<bd_core::time::GameTime>,
) {
    if let Ok(pools) = player_pools.single() {
        vm.hp_current = pools.get(PoolKind::Health).map_or(0, |p| p.current);
        vm.hp_max = pools.get(PoolKind::Health).map_or(0, |p| p.max);
        vm.ap_current = pools.get(PoolKind::ActionPoints).map_or(0, |p| p.current);
        vm.ap_max = pools.get(PoolKind::ActionPoints).map_or(0, |p| p.max);
    }
    vm.supplies = colony_res.pools.get(PoolKind::Supplies).map_or(0, |p| p.current);
    vm.faith = colony_res.pools.get(PoolKind::Faith).map_or(0, |p| p.current);
    vm.day = game_time.day;
}

fn build_party_vm(
    survivors: Query<&Name, With<bd_core::colony::survivors::Survivor>>,
    mut vm: ResMut<StatsViewModel>,
) {
    vm.party_names = survivors.iter().map(|n| n.0.clone()).collect();
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
    player: Query<(&Position, &Pools), With<Player>>,
    enemies: Query<&Position, (With<BlocksMovement>, Without<Player>)>,
    map: Res<SmokeMap>,
    mut vm: ResMut<ActionListViewModel>,
) {
    let Ok((pp, pools)) = player.single() else {
        vm.actions.clear();
        return;
    };
    let ap = pools.get(PoolKind::ActionPoints).map_or(0, |p| p.current);
    let has_ap = ap >= 1;
    let enemy_near = enemies
        .iter()
        .any(|ep| (ep.x - pp.x).unsigned_abs() + (ep.y - pp.y).unsigned_abs() <= 1);
    let can_move = map.is_walkable(pp.x + 1, pp.y);

    vm.actions = vec![
        ActionItemVm {
            label: "Move".into(),
            key_hint: "WASD".into(),
            enabled: has_ap && can_move,
            denial_reason: if !has_ap {
                Some("No AP".into())
            } else if !can_move {
                Some("Blocked".into())
            } else {
                None
            },
        },
        ActionItemVm {
            label: "Wait".into(),
            key_hint: ".".into(),
            enabled: true,
            denial_reason: None,
        },
        ActionItemVm {
            label: "Attack".into(),
            key_hint: "f".into(),
            enabled: has_ap && enemy_near,
            denial_reason: if !has_ap {
                Some("No AP".into())
            } else if !enemy_near {
                Some("Range".into())
            } else {
                None
            },
        },
        ActionItemVm {
            label: "Guard".into(),
            key_hint: "g".into(),
            enabled: has_ap,
            denial_reason: if !has_ap { Some("No AP".into()) } else { None },
        },
    ];
}

fn build_map_vm(
    map: Res<SmokeMap>,
    player_pos: Query<&Position, With<Player>>,
    enemies: Query<(&Position, Option<&bd_core::components::Name>), (With<BlocksMovement>, Without<Player>)>,
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
    vm.player_pos = player_pos.single().ok().copied();
    vm.enemy_positions.clear();
    vm.enemy_glyphs.clear();
    for (pos, name) in enemies.iter() {
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

/// Build the inventory container view model for the player.
fn build_container_vm(
    player: Query<Entity, With<Player>>,
    items: Query<(Entity, Option<&Name>, Option<&Item>)>,
    contained_in: Query<&ContainedIn>,
    equipped_by: Query<&EquippedBy>,
    mut vm: ResMut<ContainerViewModel>,
) {
    let Ok(player_entity) = player.single() else {
        vm.items.clear();
        return;
    };

    // Find items in player's inventory (ContainedIn → player)
    let mut entries: Vec<ItemEntryVm> = Vec::new();
    for (entity, name, _item) in items.iter() {
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
                name: name.map(|n| n.0.clone()).unwrap_or_else(|| "Unknown".into()),
                equipped: is_equipped,
                usable: is_contained, // contained items can be used
            });
        }
    }

    vm.items = entries;
}

/// Build the event view model from the CurrentEvent resource.
fn build_event_vm(
    current: Res<bd_core::events::CurrentEvent>,
    registry: Res<bd_core::events::EventRegistry>,
    mut vm: ResMut<EventViewModel>,
) {
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
        // Width is now 40 because startup syncs to shelter map
        assert_eq!(vm.width, 40);
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
        let _rat = app.world_mut().spawn((
            Position { x: 3, y: 3 },
            bd_core::components::BlocksMovement,
            bd_core::components::Name("Rat".into()),
        )).id();
        // Spawn a Skeleton at (5,5)
        let _skeleton = app.world_mut().spawn((
            Position { x: 5, y: 5 },
            bd_core::components::BlocksMovement,
            bd_core::components::Name("Skeleton".into()),
        )).id();
        // Spawn an unnamed enemy at (7,7)
        let _unknown = app.world_mut().spawn((
            Position { x: 7, y: 7 },
            bd_core::components::BlocksMovement,
        )).id();

        // Ensure map resource is set (test_app may have left default)
        app.world_mut().insert_resource(SmokeMap::new(10, 10, Tile::Floor));

        app.update();

        let vm = app.world().resource::<MapViewModel>();
        assert_eq!(vm.enemy_positions.len(), 3,
            "Should find 3 enemy positions, got {:?}", vm.enemy_positions);
        // Find the glyph for the Rat at (3,3)
        let rat_glyph = vm.enemy_glyphs.iter().find(|(p, _)| p.x == 3 && p.y == 3).map(|(_, g)| *g);
        assert_eq!(rat_glyph, Some('r'), "Rat should map to glyph 'r'");
        // Find the glyph for the Skeleton at (5,5)
        let skel_glyph = vm.enemy_glyphs.iter().find(|(p, _)| p.x == 5 && p.y == 5).map(|(_, g)| *g);
        assert_eq!(skel_glyph, Some('S'), "Skeleton should map to glyph 'S'");
        // Unknown enemy should be 'E'
        let unknown_glyph = vm.enemy_glyphs.iter().find(|(p, _)| p.x == 7 && p.y == 7).map(|(_, g)| *g);
        assert_eq!(unknown_glyph, Some('E'), "Unknown enemy should default to 'E'");
    }
}
