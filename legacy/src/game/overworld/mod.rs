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
        )
        .add_systems(
            Update,
            travel::enter_overworld_from_colony.run_if(in_state(AppState::Colony)),
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

    // Generate graph once and keep it across re-entries
    if existing_map.is_none() {
        let seed = world_seed.map(|seed| seed.0).unwrap_or(42u64);
        let mut graph = generate_overworld(seed);
        // Discover shelter + adjacent nodes
        graph.nodes[0].discovered = true;
        let neighbors = graph.neighbors(0);
        for &nid in &neighbors {
            if let Some(n) = graph.nodes.get_mut(nid) {
                n.discovered = true;
            }
        }
        let node_count = graph.nodes.len();
        commands.insert_resource(WorldMap(graph));

        // Generate factions alongside the map (same seed)
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
    let Some(map) = world_map else { return };
    let Some(pos) = player_pos else { return };

    let Some(road) = map.0.road_between(pos.current_node, dest) else {
        return;
    };

    let seed = world_seed.map(|seed| seed.0).unwrap_or(42u64);

    if let Some(resources) = resources {
        if resources.food == 0 {
            log.push(
                format!(
                    "Traveling without food will cost {} HP and +{} exposure per day.",
                    travel::STARVATION_HP_DAMAGE,
                    travel::STARVATION_EXPOSURE
                ),
                LogColor::EnemyHit,
                time.turn,
            );
        }
        if resources.water == 0 {
            log.push(
                format!(
                    "Traveling without water will cost {} HP and +{} exposure per day.",
                    travel::DEHYDRATION_HP_DAMAGE,
                    travel::DEHYDRATION_EXPOSURE
                ),
                LogColor::EnemyHit,
                time.turn,
            );
        }
    }

    commands.insert_resource(TravelDayTimer::default());
    commands.insert_resource(TravelState {
        from_node: pos.current_node,
        to_node: dest,
        distance_remaining: road.distance,
        day: 1,
        current_weather: weather::roll_weather(seed, 1),
        world_seed: seed,
        encounters_seen: 0,
    });
}

/// Check if travel is complete; update position, discover nodes, transition states.
fn handle_arrival(
    mut commands: Commands,
    travel_state: Option<Res<TravelState>>,
    mut world_map: Option<ResMut<WorldMap>>,
    mut player_pos: Option<ResMut<PlayerMapPosition>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut log: ResMut<GameLog>,
    time: Res<turn::GameTime>,
) {
    let Some(travel) = travel_state else { return };
    if travel.distance_remaining > 0.0 {
        return;
    }

    let arrived_at = travel.to_node;
    commands.remove_resource::<TravelState>();

    // Validate destination node exists
    let node_exists = world_map
        .as_ref()
        .is_some_and(|map| map.0.node(arrived_at).is_some());
    if !node_exists {
        warn!("Travel destination node {} is invalid", arrived_at);
        log.push(
            "Travel destination invalid, returning to shelter",
            LogColor::System,
            time.turn,
        );
        next_state.set(AppState::Colony);
        return;
    }

    // Update player position
    if let Some(ref mut pos) = player_pos {
        pos.current_node = arrived_at;
    }

    // Discover arrived node + neighbors
    if let Some(ref mut map) = world_map {
        if let Some(n) = map.0.nodes.get_mut(arrived_at) {
            n.discovered = true;
        }
        let neighbors = map.0.neighbors(arrived_at);
        for &nid in &neighbors {
            if let Some(n) = map.0.nodes.get_mut(nid) {
                n.discovered = true;
            }
        }

        // Transition based on node type
        if let Some(node) = map.0.node(arrived_at) {
            log.push(
                format!("Arrived at {}.", node.name),
                LogColor::System,
                time.turn,
            );
            match node.node_type {
                NodeType::Dungeon => {
                    commands.insert_resource(DungeonState {
                        floor_number: 1,
                        max_floors: 5,
                        seed: seed_for_dungeon_site(travel.world_seed, node.id),
                        theme: node.dungeon_theme.unwrap_or(DungeonTheme::UrbanDecay),
                        tile_texture: None,
                        origin_node_id: Some(node.id),
                        story_tag: node.story_tag,
                    });
                    next_state.set(AppState::Dungeon);
                }
                NodeType::Shelter => {
                    next_state.set(AppState::Colony);
                }
                _ => {}
            }
        }
    }
}

/// Cleanup transient state when leaving Overworld (keep WorldMap & PlayerMapPosition).
fn cleanup_overworld(mut commands: Commands) {
    commands.remove_resource::<TravelState>();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_travel_warns_but_allows_zero_supplies() {
        let mut app = App::new();
        let graph = generate_overworld(42);
        let destination = graph
            .neighbors(0)
            .first()
            .copied()
            .expect("shelter should have a connected destination");

        app.insert_resource(WorldMap(graph));
        app.insert_resource(PlayerMapPosition { current_node: 0 });
        app.insert_resource(SelectedDestination(Some(destination)));
        app.insert_resource(WorldSeed(42));
        app.insert_resource(ShelterResources {
            food: 0,
            water: 0,
            scrap: 0,
            medicine: 0,
            ammo: 0,
        });
        app.insert_resource(GameLog::default());
        app.insert_resource(turn::GameTime { turn: 9 });
        app.add_systems(Update, start_travel);

        app.update();

        assert!(
            app.world().get_resource::<TravelState>().is_some(),
            "travel should still begin without supplies"
        );

        let log = app.world().resource::<GameLog>();
        assert_eq!(log.entries().len(), 2);
        assert!(log.entries()[0].text.contains("without food"));
        assert!(log.entries()[1].text.contains("without water"));
    }
}
