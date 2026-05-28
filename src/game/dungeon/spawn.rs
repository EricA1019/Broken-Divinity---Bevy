#![allow(clippy::too_many_arguments, clippy::type_complexity)]

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
use crate::core::resources::WorldSeed;
use crate::core::sanity::RaidExposure;
use crate::core::save::{self, PendingLoad, PlayerSnapshot};
use crate::core::stats::{CombatStats, EntityName};
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
fn roll_room_type(rng: &mut impl rand::Rng) -> RoomType {
    let roll = rng.random_range(0..100u32);
    match roll {
        0..30 => RoomType::Empty,
        30..50 => RoomType::Loot,
        50..75 => RoomType::Enemy,
        75..85 => RoomType::Hazard,
        _ => RoomType::Mixed,
    }
}

/// Spawn a single floor's worth of entities (tilemap, enemies, loot, hazards, anomalies).
/// Does NOT spawn the player — call site handles that.
fn spawn_floor_entities(
    commands: &mut Commands,
    floor: &bsp::DungeonFloor,
    theme: DungeonTheme,
    floor_number: u32,
    seed: u64,
    story_room: Option<bsp::Rect>,
) {
    // Insert map data resource
    commands.insert_resource(MapTiles::new(floor.tiles.clone()));

    // Spawn tilemap
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
        texture: TilemapTexture::Single(Handle::default()),
        tile_size: TilemapTileSize { x: 16.0, y: 16.0 },
        anchor: TilemapAnchor::Center,
        ..Default::default()
    });

    // RNG for spawning entities on this floor
    let mut rng = ChaCha8Rng::seed_from_u64(seed.wrapping_add(floor_number as u64));
    let table = enemies::spawn_table(theme);
    let content_rooms: Vec<_> = floor
        .rooms
        .iter()
        .copied()
        .filter(|room| Some(*room) != story_room)
        .collect();

    // Assign room types and spawn content (skip room 0 = player room)
    for room in content_rooms.iter().skip(1) {
        let room_type = roll_room_type(&mut rng);

        match room_type {
            RoomType::Empty => {}
            RoomType::Loot => {
                loot::spawn_loot_in_rooms(commands, &[*room], &mut rng, floor_number);
            }
            RoomType::Enemy => {
                let count = rng.random_range(1..=2u32);
                for _ in 0..count {
                    let def = &table[rng.random_range(0..table.len())];
                    let ex = rng.random_range(room.x..(room.x + room.w));
                    let ey = rng.random_range(room.y..(room.y + room.h));
                    enemies::spawn_enemy(commands, def, ex, ey);
                }
            }
            RoomType::Hazard => {
                hazards::spawn_hazards(commands, &[*room], theme, &mut rng);
            }
            RoomType::Mixed => {
                // Enemies + loot
                let def = &table[rng.random_range(0..table.len())];
                let ex = rng.random_range(room.x..(room.x + room.w));
                let ey = rng.random_range(room.y..(room.y + room.h));
                enemies::spawn_enemy(commands, def, ex, ey);
                loot::spawn_loot_in_rooms(commands, &[*room], &mut rng, floor_number);
            }
        }
    }

    // Spawn anomalies (1-2 per floor, scales with depth)
    anomalies::spawn_anomalies(commands, &content_rooms, &mut rng, floor_number);

    // Spawn lore drops (10% chance per room)
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
        origin_node_id,
        story_tag,
    });

    spawn_floor_entities(
        &mut commands,
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
            &crate::core::stats::PlayerProgression,
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
                origin_node_id: dungeon_state.origin_node_id,
                story_tag: dungeon_state.story_tag,
            });

            spawn_floor_entities(
                &mut commands,
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
                origin_node_id: dungeon_state.origin_node_id,
                story_tag: dungeon_state.story_tag,
            });

            spawn_floor_entities(
                &mut commands,
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
