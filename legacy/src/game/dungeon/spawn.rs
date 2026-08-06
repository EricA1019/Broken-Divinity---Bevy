//! Dungeon spawn system.
//!
//! On entering `AppState::Dungeon`: generate floor, spawn tilemap, spawn player at entry,
//! spawn enemies in rooms, anomalies, hazards, loot. Supports multi-floor via stairs.

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use rand::RngExt;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::core::components::{Enemy, Player, Position, TileKind};
use crate::core::gamelog::{GameLog, LogColor};
use crate::core::inventory::{Equipment, Inventory, RangedWeaponState};
use crate::core::items::ItemDrop;
use crate::core::movement::MapTiles;
use crate::core::perks::PlayerPerks;
use crate::core::player::PlayerBundle;
use crate::core::resources::{PlaceholderTileAtlas, WorldSeed};
use crate::core::sanity::RaidExposure;
use crate::core::save::{self, PendingLoad, PlayerSnapshot};
use crate::core::stats::{CombatStats, EntityName, PlayerProgression};
use crate::game::dungeon::anomalies::{self, Anomaly};
use crate::game::dungeon::bsp;
use crate::game::dungeon::enemies;
use crate::game::dungeon::gabriel::{self, GabrielState};
use crate::game::dungeon::hazards::{self, HazardTile};
use crate::game::dungeon::loot;
use crate::game::dungeon::theme::DungeonTheme;
use crate::game::overworld::graphgen::DungeonStoryTag;

/// Resource tracking the current dungeon state.
#[derive(Resource, Debug, Reflect)]
#[reflect(Resource)]
pub struct DungeonState {
    pub floor_number: u32,
    pub max_floors: u32,
    pub seed: u64,
    pub theme: DungeonTheme,
    pub tile_texture: Option<Handle<Image>>,
    pub origin_node_id: Option<usize>,
    pub story_tag: Option<DungeonStoryTag>,
}

/// Marker component for tilemap entities so we can clean them up on exit.
#[derive(Component)]
pub struct DungeonTilemap;

/// Pick dungeon theme based on floor number for variety.
fn theme_for_floor(floor: u32) -> DungeonTheme {
    match floor % 3 {
        0 => DungeonTheme::UrbanDecay,
        1 => DungeonTheme::Underground,
        _ => DungeonTheme::Military,
    }
}

pub fn seed_for_dungeon_site(world_seed: u64, node_id: usize) -> u64 {
    world_seed ^ ((node_id as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15))
}

/// Room type for varied content spawning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoomType {
    Empty,
    Loot,
    Enemy,
    Hazard,
    Mixed,
}

/// Assign a room type via weighted random.
fn roll_room_type(rng: &mut impl rand::Rng, floor_number: u32) -> RoomType {
    let roll = rng.random_range(0..100u32);
    if floor_number == 1 {
        match roll {
            0..45 => RoomType::Empty,
            45..70 => RoomType::Loot,
            70..90 => RoomType::Enemy,
            90..95 => RoomType::Hazard,
            _ => RoomType::Mixed,
        }
    } else {
        match roll {
            0..30 => RoomType::Empty,
            30..50 => RoomType::Loot,
            50..75 => RoomType::Enemy,
            75..85 => RoomType::Hazard,
            _ => RoomType::Mixed,
        }
    }
}

fn enemy_pool_len_for_floor(table_len: usize, floor_number: u32) -> usize {
    if floor_number == 1 {
        table_len.saturating_sub(1).max(1)
    } else {
        table_len
    }
}

fn enemy_spawn_count(room_type: RoomType, floor_number: u32, rng: &mut impl rand::Rng) -> u32 {
    match room_type {
        RoomType::Enemy if floor_number == 1 => 1,
        RoomType::Enemy => rng.random_range(1..=2u32),
        RoomType::Mixed => 1,
        RoomType::Empty | RoomType::Loot | RoomType::Hazard => 0,
    }
}

fn max_enemy_spawns_for_floor(floor_number: u32) -> usize {
    if floor_number == 1 { 8 } else { usize::MAX }
}

/// Spawn the tilemap layer from floor data into the world.
fn spawn_tilemap_layer(
    commands: &mut Commands,
    texture_handle: Handle<Image>,
    floor: &bsp::DungeonFloor,
    theme: DungeonTheme,
) {
    let map_size = TilemapSize {
        x: floor.width as u32,
        y: floor.height as u32,
    };

    let tilemap_entity = commands.spawn((DungeonTilemap, Transform::default())).id();
    let mut tile_storage = TileStorage::empty(map_size);

    for y in 0..floor.height {
        for x in 0..floor.width {
            let tile_pos = TilePos {
                x: x as u32,
                y: y as u32,
            };
            let tile_kind = floor.tiles[y][x];
            let atlas_index = theme.atlas_index(tile_kind);

            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index: TileTextureIndex(atlas_index),
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
}

/// Spawn enemies, loot, and hazards into rooms based on room type.
fn spawn_room_content(
    commands: &mut Commands,
    floor: &bsp::DungeonFloor,
    theme: DungeonTheme,
    floor_number: u32,
    seed: u64,
    story_room: Option<bsp::Rect>,
) {
    let mut rng = ChaCha8Rng::seed_from_u64(seed.wrapping_add(floor_number as u64));
    let table = enemies::spawn_table(theme);
    let enemy_pool_len = enemy_pool_len_for_floor(table.len(), floor_number);
    let max_enemy_spawns = max_enemy_spawns_for_floor(floor_number);
    let mut enemies_spawned = 0usize;
    let content_rooms: Vec<_> = floor
        .rooms
        .iter()
        .copied()
        .filter(|room| Some(*room) != story_room)
        .collect();

    for room in content_rooms.iter().skip(1) {
        let room_type = roll_room_type(&mut rng, floor_number);

        match room_type {
            RoomType::Empty => {}
            RoomType::Loot => {
                loot::spawn_loot_in_rooms(commands, &[*room], &mut rng, floor_number);
            }
            RoomType::Enemy => {
                let available = max_enemy_spawns.saturating_sub(enemies_spawned);
                if available == 0 {
                    continue;
                }
                let count =
                    enemy_spawn_count(room_type, floor_number, &mut rng).min(available as u32);
                for _ in 0..count {
                    let def = &table[rng.random_range(0..enemy_pool_len)];
                    let ex = rng.random_range(room.x..(room.x + room.w));
                    let ey = rng.random_range(room.y..(room.y + room.h));
                    enemies::spawn_enemy(commands, def, ex, ey);
                    enemies_spawned += 1;
                }
            }
            RoomType::Hazard => {
                hazards::spawn_hazards(commands, &[*room], theme, &mut rng);
            }
            RoomType::Mixed => {
                let available = max_enemy_spawns.saturating_sub(enemies_spawned);
                let count =
                    enemy_spawn_count(room_type, floor_number, &mut rng).min(available as u32);
                for _ in 0..count {
                    let def = &table[rng.random_range(0..enemy_pool_len)];
                    let ex = rng.random_range(room.x..(room.x + room.w));
                    let ey = rng.random_range(room.y..(room.y + room.h));
                    enemies::spawn_enemy(commands, def, ex, ey);
                    enemies_spawned += 1;
                }
                loot::spawn_loot_in_rooms(commands, &[*room], &mut rng, floor_number);
            }
        }
    }
}

/// Spawn a single floor's worth of entities (tilemap, enemies, loot, hazards, anomalies).
/// Does NOT spawn the player — call site handles that.
fn spawn_floor_entities(
    commands: &mut Commands,
    texture_handle: Handle<Image>,
    floor: &bsp::DungeonFloor,
    theme: DungeonTheme,
    floor_number: u32,
    seed: u64,
    story_room: Option<bsp::Rect>,
) {
    commands.insert_resource(MapTiles::new(floor.tiles.clone()));
    spawn_tilemap_layer(commands, texture_handle, floor, theme);
    spawn_room_content(commands, floor, theme, floor_number, seed, story_room);

    let content_rooms: Vec<_> = floor
        .rooms
        .iter()
        .copied()
        .filter(|room| Some(*room) != story_room)
        .collect();

    let mut rng = ChaCha8Rng::seed_from_u64(seed.wrapping_add(floor_number as u64));
    anomalies::spawn_anomalies(commands, &content_rooms, &mut rng, floor_number);
    super::lore::spawn_lore_drops(commands, &content_rooms, &mut rng);
}

fn configure_story_entities(
    commands: &mut Commands,
    floor: &bsp::DungeonFloor,
    floor_number: u32,
    story_tag: Option<DungeonStoryTag>,
    gabriel_state: &GabrielState,
    player_anchor: (i32, i32),
) -> Option<bsp::Rect> {
    commands.remove_resource::<gabriel::GabrielEncounter>();
    commands.insert_resource(gabriel::GabrielDialogueState::default());

    if gabriel_state.should_stage_intro(floor_number, story_tag) {
        let story_room = gabriel::select_intro_room(floor)?;
        commands.insert_resource(gabriel::GabrielEncounter::new(story_room));
        gabriel::spawn_gabriel_entity(
            commands,
            gabriel::companion_spawn_on_floor(floor, story_room.center()),
            false,
        );
        return Some(story_room);
    }

    if gabriel_state.joined {
        gabriel::spawn_gabriel_entity(
            commands,
            gabriel::companion_spawn_on_floor(floor, player_anchor),
            true,
        );
    }

    None
}

/// One-shot setup when entering the Dungeon state.
pub fn setup_dungeon(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    existing_tilemaps: Query<Entity, With<DungeonTilemap>>,
    existing_players: Query<Entity, With<Player>>,
    world_seed: Option<Res<WorldSeed>>,
    existing_dungeon_state: Option<Res<DungeonState>>,
    pending_load: Option<ResMut<PendingLoad>>,
    player_snapshot: Option<Res<PlayerSnapshot>>,
    gabriel_state: Res<GabrielState>,
    mut log: ResMut<GameLog>,
    placeholder_tiles: Res<PlaceholderTileAtlas>,
    mut combat_rng: ResMut<super::melee::CombatRng>,
) {
    // Clean up any previous dungeon entities
    for entity in existing_tilemaps.iter() {
        commands.entity(entity).despawn();
    }
    for entity in existing_players.iter() {
        commands.entity(entity).despawn();
    }

    let loaded_save = pending_load
        .and_then(|mut pending| pending.take())
        .filter(|save| {
            matches!(
                save.app_state,
                save::SaveAppState::Dungeon | save::SaveAppState::Combat
            )
        });

    if let Some(save) = loaded_save.as_ref() {
        save::restore_persistent_run_resources(&mut commands, save);
    }

    let pending_dungeon_state = existing_dungeon_state.as_deref();
    let active_gabriel_state = loaded_save
        .as_ref()
        .map(|save| &save.gabriel)
        .unwrap_or(&gabriel_state);

    let seed = loaded_save
        .as_ref()
        .and_then(|save| (save.dungeon.seed != 0).then_some(save.dungeon.seed))
        .or_else(|| pending_dungeon_state.map(|state| state.seed))
        .or_else(|| world_seed.map(|seed| seed.0))
        .unwrap_or(42u64);

    // Seed combat RNG deterministically from the dungeon seed
    combat_rng.reseed_from(seed);

    let floor_number = loaded_save.as_ref().map_or_else(
        || pending_dungeon_state.map_or(1u32, |state| state.floor_number.max(1)),
        |save| save.dungeon.floor_number.max(1),
    );
    let max_floors = loaded_save.as_ref().map_or_else(
        || pending_dungeon_state.map_or(5u32, |state| state.max_floors.max(floor_number)),
        |save| save.dungeon.max_floors.max(floor_number),
    );
    let theme = loaded_save
        .as_ref()
        .and_then(|save| save.dungeon.theme)
        .or_else(|| pending_dungeon_state.map(|state| state.theme))
        .unwrap_or_else(|| theme_for_floor(floor_number));
    let tile_texture = pending_dungeon_state
        .and_then(|state| state.tile_texture.clone())
        .unwrap_or_else(|| placeholder_tiles.0.clone());
    let origin_node_id = loaded_save
        .as_ref()
        .and_then(|save| save.dungeon.origin_node_id)
        .or_else(|| pending_dungeon_state.and_then(|state| state.origin_node_id));
    let story_tag = loaded_save
        .as_ref()
        .and_then(|save| save.dungeon.story_tag)
        .or_else(|| pending_dungeon_state.and_then(|state| state.story_tag));
    let floor_seed = if floor_number <= 1 {
        seed
    } else {
        seed.wrapping_add(floor_number as u64 * 1000)
    };
    let floor = bsp::generate_floor(80, 60, floor_seed);
    let story_room = if active_gabriel_state.should_stage_intro(floor_number, story_tag) {
        gabriel::select_intro_room(&floor)
    } else {
        None
    };

    commands.insert_resource(DungeonState {
        floor_number,
        max_floors,
        seed,
        theme,
        tile_texture: Some(tile_texture.clone()),
        origin_node_id,
        story_tag,
    });

    spawn_floor_entities(
        &mut commands,
        tile_texture,
        &floor,
        theme,
        floor_number,
        floor_seed,
        story_room,
    );

    // Spawn player — validate spawn point is walkable
    let (px, py, spawn_adjusted) = floor.validated_spawn_point();
    if spawn_adjusted {
        warn!(
            "Spawn point {:?} was invalid, adjusted to ({}, {})",
            floor.spawn_point, px, py
        );
        log.push("Warning: spawn point adjusted", LogColor::System, 0);
    }
    if let Some(save) = loaded_save.as_ref() {
        save::spawn_player_from_save(&mut commands, &save.player);
    } else if let Some(snapshot) = player_snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.0.clone())
    {
        save::spawn_player_from_save_at(&mut commands, &snapshot, Some((px, py)));
        // Consume snapshot so it's not reused on unexpected re-entry
        commands.insert_resource(save::PlayerSnapshot::default());
    } else {
        commands.spawn(PlayerBundle::new(px, py));
    }

    let player_anchor = loaded_save
        .as_ref()
        .map(|save| (save.player.position.x, save.player.position.y))
        .unwrap_or((px, py));
    configure_story_entities(
        &mut commands,
        &floor,
        floor_number,
        story_tag,
        active_gabriel_state,
        player_anchor,
    );
}

/// System: detect player stepping on stairs and transition floors.
pub fn handle_stairs(
    mut commands: Commands,
    player_q: Query<
        (
            &Position,
            &CombatStats,
            &Inventory,
            &Equipment,
            &RangedWeaponState,
            &RaidExposure,
            &PlayerPerks,
            &PlayerProgression,
            Option<&EntityName>,
            &crate::core::abilities::SprintCooldown,
        ),
        With<Player>,
    >,
    map: Res<MapTiles>,
    dungeon_state: Option<Res<DungeonState>>,
    keys: Res<ButtonInput<KeyCode>>,
    dialogue_state: Option<Res<gabriel::GabrielDialogueState>>,
    // Entities to despawn on floor change
    tilemaps: Query<Entity, With<DungeonTilemap>>,
    enemies_q: Query<Entity, With<Enemy>>,
    gabriel_q: Query<Entity, With<gabriel::Gabriel>>,
    items_q: Query<Entity, With<ItemDrop>>,
    anomalies_q: Query<Entity, With<Anomaly>>,
    hazards_q: Query<Entity, With<HazardTile>>,
    mut log: ResMut<crate::core::gamelog::GameLog>,
    time: Res<crate::core::turn::GameTime>,
    mut next_app_state: ResMut<NextState<crate::core::state::AppState>>,
    gabriel_state: Res<GabrielState>,
) {
    let Some(dungeon_state) = dungeon_state else {
        return;
    };
    let Ok((
        pos,
        stats,
        inventory,
        equipment,
        ranged_state,
        sanity,
        perks,
        progression,
        name,
        sprint_cd,
    )) = player_q.single()
    else {
        return;
    };

    if dialogue_state.is_some_and(|dialogue| dialogue.is_active()) {
        return;
    }

    // Must press '>' (Period) to take stairs
    if !keys.just_pressed(KeyCode::Period) {
        return;
    }

    let tile = map.get_tile(pos.x, pos.y);

    match tile {
        Some(TileKind::StairsDown) => {
            if dungeon_state.floor_number >= dungeon_state.max_floors {
                log.push(
                    "These stairs lead nowhere — this is the deepest floor.",
                    crate::core::gamelog::LogColor::System,
                    time.turn,
                );
                return;
            }

            let new_floor = dungeon_state.floor_number + 1;
            let new_theme = dungeon_state.theme;
            let new_seed = dungeon_state.seed.wrapping_add(new_floor as u64 * 1000);
            let floor_data = bsp::generate_floor(80, 60, new_seed);
            let story_room = if gabriel_state.should_stage_intro(new_floor, dungeon_state.story_tag)
            {
                gabriel::select_intro_room(&floor_data)
            } else {
                None
            };

            // Despawn current floor entities (NOT the player)
            for e in tilemaps.iter() {
                commands.entity(e).despawn();
            }
            for e in enemies_q.iter() {
                commands.entity(e).despawn();
            }
            for e in gabriel_q.iter() {
                commands.entity(e).despawn();
            }
            for e in items_q.iter() {
                commands.entity(e).despawn();
            }
            for e in anomalies_q.iter() {
                commands.entity(e).despawn();
            }
            for e in hazards_q.iter() {
                commands.entity(e).despawn();
            }
            commands.remove_resource::<MapTiles>();

            commands.insert_resource(DungeonState {
                floor_number: new_floor,
                max_floors: dungeon_state.max_floors,
                seed: dungeon_state.seed,
                theme: new_theme,
                tile_texture: dungeon_state.tile_texture.clone(),
                origin_node_id: dungeon_state.origin_node_id,
                story_tag: dungeon_state.story_tag,
            });

            spawn_floor_entities(
                &mut commands,
                dungeon_state
                    .tile_texture
                    .clone()
                    .unwrap_or_else(Handle::default),
                &floor_data,
                new_theme,
                new_floor,
                new_seed,
                story_room,
            );

            // Move player to StairsUp on new floor
            let (px, py) = floor_data.spawn_point;
            configure_story_entities(
                &mut commands,
                &floor_data,
                new_floor,
                dungeon_state.story_tag,
                &gabriel_state,
                (px, py),
            );
            // We'll set player position via a command; the actual position update
            // happens because we have a mutable reference coming next frame
            commands.queue(move |world: &mut World| {
                let mut q = world.query_filtered::<&mut Position, With<Player>>();
                for mut p in q.iter_mut(world) {
                    p.x = px;
                    p.y = py;
                }
            });

            log.push(
                format!("You descend to floor {}. ({})", new_floor, new_theme.name()),
                crate::core::gamelog::LogColor::System,
                time.turn,
            );
        }
        Some(TileKind::StairsUp) => {
            if dungeon_state.floor_number <= 1 {
                // Extract from dungeon → return to overworld
                commands.insert_resource(save::PlayerSnapshot(Some(save::snapshot_player_state(
                    pos,
                    stats,
                    inventory,
                    equipment,
                    ranged_state,
                    sanity,
                    perks,
                    progression,
                    name,
                    sprint_cd.remaining,
                ))));
                log.push(
                    "You climb back to the surface.",
                    crate::core::gamelog::LogColor::System,
                    time.turn,
                );
                next_app_state.set(crate::core::state::AppState::Overworld);
                return;
            }

            let new_floor = dungeon_state.floor_number - 1;
            let new_theme = dungeon_state.theme;
            let new_seed = dungeon_state.seed.wrapping_add(new_floor as u64 * 1000);
            let floor_data = bsp::generate_floor(80, 60, new_seed);
            let story_room = if gabriel_state.should_stage_intro(new_floor, dungeon_state.story_tag)
            {
                gabriel::select_intro_room(&floor_data)
            } else {
                None
            };

            for e in tilemaps.iter() {
                commands.entity(e).despawn();
            }
            for e in enemies_q.iter() {
                commands.entity(e).despawn();
            }
            for e in gabriel_q.iter() {
                commands.entity(e).despawn();
            }
            for e in items_q.iter() {
                commands.entity(e).despawn();
            }
            for e in anomalies_q.iter() {
                commands.entity(e).despawn();
            }
            for e in hazards_q.iter() {
                commands.entity(e).despawn();
            }
            commands.remove_resource::<MapTiles>();

            commands.insert_resource(DungeonState {
                floor_number: new_floor,
                max_floors: dungeon_state.max_floors,
                seed: dungeon_state.seed,
                theme: new_theme,
                tile_texture: dungeon_state.tile_texture.clone(),
                origin_node_id: dungeon_state.origin_node_id,
                story_tag: dungeon_state.story_tag,
            });

            spawn_floor_entities(
                &mut commands,
                dungeon_state
                    .tile_texture
                    .clone()
                    .unwrap_or_else(Handle::default),
                &floor_data,
                new_theme,
                new_floor,
                new_seed,
                story_room,
            );

            // Find StairsDown on previous floor — spawn player there
            let mut stair_pos = floor_data.spawn_point;
            for y in 0..floor_data.height {
                for x in 0..floor_data.width {
                    if floor_data.tiles[y][x] == TileKind::StairsDown {
                        stair_pos = (x as i32, y as i32);
                    }
                }
            }
            let (px, py) = stair_pos;
            configure_story_entities(
                &mut commands,
                &floor_data,
                new_floor,
                dungeon_state.story_tag,
                &gabriel_state,
                (px, py),
            );
            commands.queue(move |world: &mut World| {
                let mut q = world.query_filtered::<&mut Position, With<Player>>();
                for mut p in q.iter_mut(world) {
                    p.x = px;
                    p.y = py;
                }
            });

            log.push(
                format!("You ascend to floor {}. ({})", new_floor, new_theme.name()),
                crate::core::gamelog::LogColor::System,
                time.turn,
            );
        }
        _ => {}
    }
}

/// Cleanup when leaving the Dungeon state.
pub fn cleanup_dungeon(
    mut commands: Commands,
    tilemaps: Query<Entity, With<DungeonTilemap>>,
    players: Query<Entity, With<Player>>,
    gabriel_q: Query<Entity, With<gabriel::Gabriel>>,
    enemies_q: Query<Entity, With<Enemy>>,
    items_q: Query<Entity, With<ItemDrop>>,
    anomalies_q: Query<Entity, With<Anomaly>>,
    hazards_q: Query<Entity, With<HazardTile>>,
) {
    for entity in tilemaps.iter() {
        commands.entity(entity).despawn();
    }
    for entity in players.iter() {
        commands.entity(entity).despawn();
    }
    for entity in gabriel_q.iter() {
        commands.entity(entity).despawn();
    }
    for entity in enemies_q.iter() {
        commands.entity(entity).despawn();
    }
    for entity in items_q.iter() {
        commands.entity(entity).despawn();
    }
    for entity in anomalies_q.iter() {
        commands.entity(entity).despawn();
    }
    for entity in hazards_q.iter() {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<MapTiles>();
    commands.remove_resource::<gabriel::GabrielEncounter>();
    commands.insert_resource(gabriel::GabrielDialogueState::default());
    commands.remove_resource::<DungeonState>();
    // Reset combat resources to prevent stale entity refs on dungeon re-entry
    commands.insert_resource(super::melee::BumpAttackTarget::default());
    commands.insert_resource(super::melee::CombatRng::default());
    commands.insert_resource(super::ranged::ShootTarget::default());
    // Reset PlayerSnapshot so stale snapshots don't persist across state transitions
    commands.insert_resource(save::PlayerSnapshot::default());
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use crate::core::components::Player;
    use crate::core::components::TileKind;
    use crate::core::movement::MapTiles;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_cleanup_dungeon_resets_combat_resources() {
        use super::super::melee::BumpAttackTarget;
        use super::super::ranged::ShootTarget;

        let mut world = World::new();

        // Spawn a dummy entity for valid Entity ID
        let dummy = world.spawn_empty().id();

        // Seed combat resources with non-default state
        world.insert_resource(BumpAttackTarget(Some(Position::new(5, 5))));
        world.insert_resource(ShootTarget(Some(dummy)));
        world.insert_resource(MapTiles::new(vec![vec![TileKind::Floor; 5]; 5]));

        // Player entity so cleanup has something to despawn
        world.spawn((Player,));

        let _ = world.run_system_once(cleanup_dungeon);

        // Verify combat resources reset
        let bump = world.get_resource::<BumpAttackTarget>();
        assert!(bump.is_some(), "BumpAttackTarget should still exist");
        assert!(bump.unwrap().0.is_none(), "BumpAttackTarget should be None after cleanup");

        let shoot = world.get_resource::<ShootTarget>();
        assert!(shoot.is_some(), "ShootTarget should still exist");
        assert!(shoot.unwrap().0.is_none(), "ShootTarget should be None after cleanup");
    }

    #[test]
    fn test_floor_one_enemy_pool_excludes_elite_slot() {
        assert_eq!(enemy_pool_len_for_floor(3, 1), 2);
        assert_eq!(enemy_pool_len_for_floor(1, 1), 1);
        assert_eq!(enemy_pool_len_for_floor(3, 2), 3);
    }

    #[test]
    fn test_floor_one_enemy_rooms_spawn_single_enemy() {
        let mut rng = ChaCha8Rng::seed_from_u64(7);
        assert_eq!(enemy_spawn_count(RoomType::Enemy, 1, &mut rng), 1);
        assert_eq!(enemy_spawn_count(RoomType::Mixed, 1, &mut rng), 1);
    }

    #[test]
    fn test_floor_one_enemy_cap_is_readable() {
        assert_eq!(max_enemy_spawns_for_floor(1), 8);
        assert_eq!(max_enemy_spawns_for_floor(2), usize::MAX);
    }
}
