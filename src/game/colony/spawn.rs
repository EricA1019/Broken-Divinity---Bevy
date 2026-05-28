#![allow(clippy::too_many_arguments)]

//! Shelter setup and teardown.

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::core::components::{Player, Position, TileKind};
use crate::core::movement::MapTiles;
use crate::core::player::PlayerBundle;
use crate::core::resources::{PlaceholderTileAtlas, ShelterResources, WorldSeed};
use crate::core::save::{self, PendingLoad, PendingStationLoad, PlayerSnapshot};
use crate::game::colony::mapgen::ShelterRoomKind;
use crate::game::colony::stations::{self, Station, StationType};
use crate::game::colony::survivors::Survivor;

/// Resource tracking shelter state.
#[derive(Resource, Debug, Reflect)]
#[reflect(Resource)]
pub struct ShelterState {
    pub seed: u64,
}

/// Marker component for the shelter tilemap entity.
#[derive(Component)]
pub struct ShelterTilemap;

/// Overlay marker that keeps the shelter exit readable on top of the tilemap.
#[derive(Component)]
pub struct ShelterGateMarker;

const DEFAULT_GATE_MARKER_SIZE: f32 = 8.0;
const DEFAULT_GATE_MARKER_COLOR: [f32; 3] = [0.25, 0.7, 1.0];

#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct GateAffordanceConfig {
    pub enabled: bool,
    pub marker_size: f32,
    pub marker_color: [f32; 3],
}

impl Default for GateAffordanceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            marker_size: DEFAULT_GATE_MARKER_SIZE,
            marker_color: DEFAULT_GATE_MARKER_COLOR,
        }
    }
}

/// Setup system: generate shelter, spawn tilemap, place initial stations, spawn player.
///
/// Survivors are spawned by [`super::survivors::spawn_initial_survivors`] which
/// runs as a chained `OnEnter` system.
pub fn setup_shelter(
    mut commands: Commands,
    world_seed: Option<Res<WorldSeed>>,
    pending_load: Option<ResMut<PendingLoad>>,
    pending_station_load: Option<ResMut<PendingStationLoad>>,
    player_snapshot: Option<Res<PlayerSnapshot>>,
    existing_resources: Option<Res<ShelterResources>>,
    gate_affordance: Option<Res<GateAffordanceConfig>>,
    placeholder_tiles: Res<PlaceholderTileAtlas>,
) {
    let gate_affordance = if let Some(config) = gate_affordance {
        config.clone()
    } else {
        let config = GateAffordanceConfig::default();
        commands.insert_resource(config.clone());
        config
    };

    let loaded_save = pending_load
        .and_then(|mut pending| pending.take())
        .filter(|save| matches!(save.app_state, save::SaveAppState::Colony));
    let seed = loaded_save
        .as_ref()
        .and_then(|save| (save.colony.shelter_seed != 0).then_some(save.colony.shelter_seed))
        .or_else(|| world_seed.map(|seed| seed.0.wrapping_add(0xC010_0001)))
        .unwrap_or(12345u64);
    let data = super::mapgen::generate_shelter(seed);
    let saved_stations = loaded_save
        .as_ref()
        .and_then(|save| (!save.colony.stations.is_empty()).then_some(save.colony.stations.clone()))
        .or_else(|| {
            pending_station_load.and_then(|mut pending| {
                let data = std::mem::take(&mut pending.0);
                if data.is_empty() { None } else { Some(data) }
            })
        });

    if let Some(save) = loaded_save.as_ref() {
        save::restore_persistent_run_resources(&mut commands, save);
    }

    // Insert map and state resources
    commands.insert_resource(MapTiles::new(data.tiles.clone()));
    commands.insert_resource(ShelterState { seed });
    if loaded_save.is_none() && existing_resources.is_none() {
        commands.insert_resource(ShelterResources::new_game());
    }

    // --- Spawn tilemap (same pattern as dungeon) ---
    let map_size = TilemapSize {
        x: data.width as u32,
        y: data.height as u32,
    };
    let texture_handle = placeholder_tiles.0.clone();
    let tilemap_entity = commands.spawn((ShelterTilemap, Transform::default())).id();
    let mut tile_storage = TileStorage::empty(map_size);

    for y in 0..data.height {
        for x in 0..data.width {
            let tile_pos = TilePos {
                x: x as u32,
                y: y as u32,
            };
            let texture_index = match data.tiles[y][x] {
                TileKind::Floor => TileTextureIndex(1),
                TileKind::Wall => TileTextureIndex(0),
                TileKind::Door => TileTextureIndex(2),
                TileKind::StairsUp => TileTextureIndex(3),
                _ => TileTextureIndex(0),
            };
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index,
                    ..Default::default()
                })
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size: TilemapGridSize { x: 16.0, y: 16.0 },
        map_type: TilemapType::Square,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size: TilemapTileSize { x: 16.0, y: 16.0 },
        anchor: TilemapAnchor::Center,
        ..Default::default()
    });

    if gate_affordance.enabled
        && let Some((gate_y, gate_x)) = data.tiles.iter().enumerate().find_map(|(y, row)| {
            row.iter()
                .position(|tile| *tile == TileKind::StairsUp)
                .map(|x| (y, x))
        })
    {
        commands.spawn((
            ShelterGateMarker,
            Position::new(gate_x as i32, gate_y as i32),
            Transform::from_xyz(0.0, 0.0, 1.0),
            Sprite {
                color: Color::srgb(
                    gate_affordance.marker_color[0],
                    gate_affordance.marker_color[1],
                    gate_affordance.marker_color[2],
                ),
                custom_size: Some(Vec2::splat(gate_affordance.marker_size)),
                ..Default::default()
            },
        ));
    }

    // --- Spawn player at shelter entrance ---
    let (px, py) = data.spawn_point;
    if let Some(save) = loaded_save.as_ref() {
        save::spawn_player_from_save(&mut commands, &save.player);
    } else if let Some(mut snapshot) = player_snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.0.clone())
    {
        snapshot.sanity.current = 0;
        save::spawn_player_from_save_at(&mut commands, &snapshot, Some((px, py)));
    } else {
        commands.spawn(PlayerBundle::new(px, py));
    }

    // --- Spawn initial stations in appropriate rooms ---
    if let Some(stations) = saved_stations {
        for station in stations {
            stations::spawn_station_with_state(
                &mut commands,
                station.to_runtime(),
                station.x,
                station.y,
            );
        }
    } else {
        for room in &data.rooms {
            let (cx, cy) = room.rect.center();
            match room.kind {
                ShelterRoomKind::Workshop => {
                    stations::spawn_station(&mut commands, StationType::Workbench, cx, cy);
                }
                ShelterRoomKind::Quarters => {
                    stations::spawn_station(&mut commands, StationType::Quarters, cx, cy);
                }
                _ => {}
            }
        }
    }
}

/// Cleanup when leaving Colony state — despawn all shelter entities and remove resources.
pub fn cleanup_shelter(
    mut commands: Commands,
    tilemaps: Query<Entity, With<ShelterTilemap>>,
    players: Query<Entity, With<Player>>,
    survivors_q: Query<Entity, With<Survivor>>,
    stations_q: Query<Entity, With<Station>>,
    gate_markers: Query<Entity, With<ShelterGateMarker>>,
) {
    for e in tilemaps.iter() {
        commands.entity(e).despawn();
    }
    for e in players.iter() {
        commands.entity(e).despawn();
    }
    for e in survivors_q.iter() {
        commands.entity(e).despawn();
    }
    for e in stations_q.iter() {
        commands.entity(e).despawn();
    }
    for e in gate_markers.iter() {
        commands.entity(e).despawn();
    }
    commands.remove_resource::<MapTiles>();
    commands.remove_resource::<ShelterState>();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::save::{PendingStationLoad, SaveStation};
    use crate::core::tilemap::init_placeholder_tile_atlas;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn test_setup_shelter_restores_pending_station_load() {
        let mut world = World::new();
        world.insert_resource(WorldSeed(42));
        world.insert_resource(Assets::<Image>::default());
        let _ = world.run_system_once(init_placeholder_tile_atlas);
        world.insert_resource(PendingStationLoad(vec![SaveStation {
            kind: StationType::Cook,
            tier: 2,
            worker_slots: 1,
            workers_assigned: 1,
            x: 9,
            y: 7,
        }]));

        let _ = world.run_system_once(setup_shelter);

        let mut stations = world.query::<(&Station, &Position)>();
        let restored: Vec<_> = stations.iter(&world).collect();
        assert_eq!(
            restored.len(),
            1,
            "pending load should replace default stations"
        );
        assert_eq!(restored[0].0.kind, StationType::Cook);
        assert_eq!(restored[0].0.tier, 2);
        assert_eq!(restored[0].0.workers_assigned, 1);
        assert_eq!(restored[0].1.to_ivec2(), IVec2::new(9, 7));
    }
}
