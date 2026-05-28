#![allow(clippy::too_many_arguments, clippy::type_complexity)]

pub mod graphgen;
pub mod map;
pub mod travel;
pub mod weather;

use crate::core::gamelog::{GameLog, LogColor};
use crate::core::resources::{self, ShelterResources, TravelDayTimer, WorldSeed};
use crate::core::save::{self, PendingLoad};
use crate::core::state::AppState;
use crate::core::turn;
use bevy::prelude::*;

use self::graphgen::{NodeType, generate_overworld};
use self::map::{PlayerMapPosition, SelectedDestination, WorldMap};
use self::travel::TravelState;

use crate::game::dungeon::spawn::{DungeonState, seed_for_dungeon_site};
use crate::game::dungeon::theme::DungeonTheme;
use crate::game::factions;

pub fn plugin(app: &mut App) {
    // --- Type registration for BRP reflection ---
    app.register_type::<graphgen::NodeType>()
        .register_type::<graphgen::DungeonStoryTag>()
        .register_type::<map::PlayerMapPosition>()
        .register_type::<map::SelectedDestination>()
        .register_type::<travel::TravelState>()
        .register_type::<weather::Weather>()
        .register_type::<crate::game::factions::Factions>()
        .register_type::<crate::game::factions::Faction>()
        .register_type::<crate::game::factions::FactionArchetype>()
        .register_type::<crate::game::factions::FactionDisposition>();

    app.init_resource::<SelectedDestination>()
        .init_resource::<TravelDayTimer>()
        .add_systems(
            OnEnter(AppState::Overworld),
            (resources::reset_travel_day_timer, setup_overworld).chain(),
        )
        .add_systems(OnExit(AppState::Overworld), cleanup_overworld)
        .add_systems(
            Update,
            (
                map::draw_overworld_map,
                start_travel,
                resources::tick_travel_day_timer.run_if(resource_exists::<TravelState>),
                turn::advance_game_time
                    .run_if(resource_exists::<TravelState>)
                    .run_if(resources::travel_day_ready),
                travel::process_travel_day
                    .run_if(resource_exists::<TravelState>)
                    .run_if(resources::travel_day_ready),
                handle_arrival,
            )
                .chain()
                .run_if(in_state(AppState::Overworld)),
        );
}

/// One-shot setup when entering Overworld.
fn setup_overworld(
    mut commands: Commands,
    existing_map: Option<Res<WorldMap>>,
    existing_pos: Option<Res<PlayerMapPosition>>,
    existing_factions: Option<Res<factions::Factions>>,
    world_seed: Option<Res<WorldSeed>>,
    pending_load: Option<ResMut<PendingLoad>>,
) {
    let loaded_save = pending_load
        .and_then(|mut pending| pending.take())
        .filter(|save| matches!(save.app_state, save::SaveAppState::Overworld));

    if let Some(save) = loaded_save.as_ref() {
        save::restore_persistent_run_resources(&mut commands, save);

        let seed = save.seed;
        let graph = if let Some(graph) = save.overworld.graph.clone() {
            graph
        } else {
            let mut graph = generate_overworld(seed);
            graph.nodes[0].discovered = true;
            let neighbors = graph.neighbors(0);
            for &nid in &neighbors {
                if let Some(node) = graph.nodes.get_mut(nid) {
                    node.discovered = true;
                }
            }
            commands.insert_resource(WorldMap(graph.clone()));
            graph
        };

        if save.overworld.factions.is_empty() {
            commands.insert_resource(factions::generate_factions(seed, graph.nodes.len()));
        }

        return;
    }

    // Generate graph once and keep it across re-entries.
    if existing_map.is_none() {
        let seed = world_seed.map(|seed| seed.0).unwrap_or(42u64);
        let mut graph = generate_overworld(seed);
        graph.nodes[0].discovered = true;
        let neighbors = graph.neighbors(0);
        for &nid in &neighbors {
            if let Some(node) = graph.nodes.get_mut(nid) {
                node.discovered = true;
            }
        }
        let node_count = graph.nodes.len();
        commands.insert_resource(WorldMap(graph));

        if existing_factions.is_none() {
            commands.insert_resource(factions::generate_factions(seed, node_count));
        }
    }

    if existing_pos.is_none() {
        commands.insert_resource(PlayerMapPosition { current_node: 0 });
    }
}

/// If a destination was selected, begin travel.
fn start_travel(
    mut commands: Commands,
    mut selected: ResMut<SelectedDestination>,
    world_map: Option<Res<WorldMap>>,
    player_pos: Option<Res<PlayerMapPosition>>,
    existing_travel: Option<Res<TravelState>>,
    world_seed: Option<Res<WorldSeed>>,
    resources: Option<Res<ShelterResources>>,
    mut log: ResMut<GameLog>,
    time: Res<turn::GameTime>,
) {
    if existing_travel.is_some() {
        return;
    }
    let Some(dest) = selected.0.take() else {
        return;
    };
    let Some(map) = world_map else {
        return;
    };
    let Some(pos) = player_pos else {
        return;
    };

    let Some(road) = map.0.road_between(pos.current_node, dest) else {
        return;
    };

    let seed = world_seed.map(|seed| seed.0).unwrap_or(42u64);

    if let Some(resources) = resources {
        if resources.food == 0 {
            log.push(
                "Traveling without food is dangerous.",
                LogColor::EnemyHit,
                time.turn,
            );
        }
    }

    let current_weather = weather::roll_weather(seed, 0);
    let _start_theme = map
        .0
        .nodes
        .get(pos.current_node)
        .and_then(|node| node.dungeon_theme)
        .unwrap_or(DungeonTheme::UrbanDecay);
    let _destination_theme = map
        .0
        .nodes
        .get(dest)
        .and_then(|node| node.dungeon_theme)
        .unwrap_or(DungeonTheme::UrbanDecay);

    commands.insert_resource(TravelState {
        from_node: pos.current_node,
        to_node: dest,
        distance_remaining: road.distance.max(1.0),
        day: 0,
        current_weather,
        world_seed: seed,
        encounters_seen: 0,
    });
}

fn handle_arrival(
    mut commands: Commands,
    travel_state: Option<Res<TravelState>>,
    player_pos: Option<ResMut<PlayerMapPosition>>,
    world_map: Option<ResMut<WorldMap>>,
    mut next_state: ResMut<NextState<AppState>>,
    dungeon_state: Option<ResMut<DungeonState>>,
    mut log: ResMut<GameLog>,
) {
    let Some(travel_state) = travel_state else {
        return;
    };

    if travel_state.distance_remaining > 0.0 {
        return;
    }

    let destination = travel_state.to_node;

    if let Some(mut player_pos) = player_pos {
        player_pos.current_node = destination;
    } else {
        commands.insert_resource(PlayerMapPosition {
            current_node: destination,
        });
    }

    if let Some(mut world_map) = world_map {
        let neighbors = world_map.0.neighbors(destination);
        if let Some(node) = world_map.0.nodes.get_mut(destination) {
            node.discovered = true;
        }
        for &neighbor in &neighbors {
            if let Some(node) = world_map.0.nodes.get_mut(neighbor) {
                node.discovered = true;
            }
        }

        if let Some(node) = world_map.0.nodes.get(destination) {
            log.push(
                format!("Arrived at {}.", node.name),
                LogColor::System,
                travel_state.day,
            );

            if matches!(node.node_type, NodeType::Dungeon) {
                let seed = seed_for_dungeon_site(travel_state.world_seed, destination);
                let theme = node.dungeon_theme.unwrap_or(DungeonTheme::UrbanDecay);
                if let Some(mut dungeon_state) = dungeon_state {
                    dungeon_state.seed = seed;
                    dungeon_state.theme = theme;
                    dungeon_state.origin_node_id = Some(destination);
                    dungeon_state.story_tag = node.story_tag;
                } else {
                    commands.insert_resource(DungeonState {
                        floor_number: 1,
                        max_floors: 3,
                        seed,
                        theme,
                        origin_node_id: Some(destination),
                        story_tag: node.story_tag,
                    });
                }
                next_state.set(AppState::Dungeon);
            }
        }
    }

    commands.remove_resource::<TravelState>();
}

fn cleanup_overworld(
    mut commands: Commands,
    selected_destination: Option<Res<SelectedDestination>>,
) {
    if selected_destination.is_some() {
        commands.insert_resource(SelectedDestination::default());
    }
    commands.remove_resource::<TravelState>();
}
