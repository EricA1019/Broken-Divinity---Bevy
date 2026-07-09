//! Save/load and replay system for the BD Kernel.
//!
//! Phase 17: Serializes world state to RON files, restores from snapshots,
//! and provides fixed-seed intent replay for deterministic testing.
//!
//! Uses a `SaveId(u64)` surrogate for Entity references since Bevy's `Entity`
//! type is not serializable. On save, each entity is assigned a SaveId.
//! On load, entities are spawned in order and a mapping is built.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    components::{BlocksMovement, Name, Player, Position, Tile},
    factory::EntityBlueprint,
    gamelog::GameLog,
    inventory::{Container, EquipmentSlot, Item, SlotKind, Usable, UseEffect},
    map::SmokeMap,
    pools::{Pool, Pools},
    relationships::{
        ContainedIn, EquippedBy, FactionMember, LocationOwned, OwnedBy, SummonedBy,
    },
    signals::PoolKind,
    statuses::{StatusInstance, Statuses},
};

// ---------------------------------------------------------------------------
// Versioning
// ---------------------------------------------------------------------------

/// Current save format version. Bump on breaking changes.
pub const SAVE_VERSION: u32 = 1;

/// Content version — corresponds to the content pack hash/date.
pub const CONTENT_VERSION: &str = "kernel-2026-07-09";

// ---------------------------------------------------------------------------
// Save ID — serializable surrogate for `Entity`
// ---------------------------------------------------------------------------

/// A serializable surrogate for Bevy's `Entity` type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SaveId(pub u64);

impl SaveId {
    pub fn from_entity(entity: Entity) -> Self {
        SaveId(entity.to_bits())
    }

    pub fn to_entity(self) -> Entity {
        Entity::from_raw_u32(self.0 as u32).unwrap_or(Entity::PLACEHOLDER)
    }
}

// ---------------------------------------------------------------------------
// Component snapshots — serializable versions of ECS components
// ---------------------------------------------------------------------------

/// All component data for one entity in a save.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityData {
    pub save_id: SaveId,
    pub blueprint_id: Option<String>,
    pub is_player: bool,
    pub blocks_movement: bool,
    pub name: Option<String>,
    pub position: Option<Position>,
    pub pools: Vec<PoolSnapshot>,
    pub statuses: Vec<StatusSnapshot>,
    pub contains: Vec<SaveId>,
    pub equipped_by: Option<SaveId>,
    pub owned_by: Option<SaveId>,
    pub summoned_by: Option<SaveId>,
    pub location_owned: Option<String>,
    pub faction: Option<String>,
    pub item: bool,
    pub container_capacity: Option<i32>,
    pub equipment_slot: Option<SlotKind>,
    pub usable: bool,
    pub usable_consume: bool,
    pub usable_effects: Vec<UseEffect>,
}

/// Serializable pool data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolSnapshot {
    pub kind: PoolKind,
    pub current: i32,
    pub min: i32,
    pub max: i32,
}

/// Serializable status instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusSnapshot {
    pub status_id: String,
    pub remaining_duration: i32,
    pub stacks: i32,
    pub source_id: Option<SaveId>,
}

// ---------------------------------------------------------------------------
// Save snapshot
// ---------------------------------------------------------------------------

/// The full serializable state of a game run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunSnapshot {
    pub save_version: u32,
    pub content_version: String,
    pub seed: u64,
    pub turn: u64,
    pub map_width: i32,
    pub map_height: i32,
    pub map_tiles: Vec<Tile>,
    pub entities: Vec<EntityData>,
    pub log_entries: Vec<String>,
}

// ---------------------------------------------------------------------------
// Save/load operations
// ---------------------------------------------------------------------------

/// Errors that can occur during save/load.
#[derive(Debug)]
pub enum SaveError {
    Io(std::io::Error),
    Corrupt(String),
    VersionMismatch { expected: u32, found: u32 },
    ContentMismatch { expected: String, found: String },
    MissingBlueprint(String),
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveError::Io(e) => write!(f, "I/O error: {e}"),
            SaveError::Corrupt(msg) => write!(f, "Corrupt save: {msg}"),
            SaveError::VersionMismatch { expected, found } => {
                write!(f, "Save version mismatch: expected {expected}, found {found}")
            }
            SaveError::ContentMismatch { expected, found } => {
                write!(f, "Content version mismatch: expected {expected}, found {found}")
            }
            SaveError::MissingBlueprint(id) => {
                write!(f, "Missing blueprint: {id}")
            }
        }
    }
}

impl std::error::Error for SaveError {}

/// Serialize the current world into a `RunSnapshot`.
pub fn save_world(
    world: &mut World,
    seed: u64,
    turn: u64,
    save_dir: &PathBuf,
) -> Result<PathBuf, SaveError> {
    let snapshot = build_snapshot(world, seed, turn);
    let path = save_dir.join(format!("save-turn-{turn}.ron"));

    fs::create_dir_all(save_dir).map_err(SaveError::Io)?;
    let content = ron::ser::to_string_pretty(&snapshot, ron::ser::PrettyConfig::default())
        .map_err(|e| SaveError::Corrupt(format!("Serialization: {e}")))?;
    fs::write(&path, content).map_err(SaveError::Io)?;

    tracing::info!("Saved {} entities to {}", snapshot.entities.len(), path.display());
    Ok(path)
}

/// Deserialize a save file and restore the world.
/// Returns the restored World and the seed.
pub fn load_world(
    path: &PathBuf,
    _blueprints: &HashMap<String, EntityBlueprint>,
) -> Result<(World, u64), SaveError> {
    let content = fs::read_to_string(path).map_err(SaveError::Io)?;
    let snapshot: RunSnapshot = ron::de::from_str(&content)
        .map_err(|e| SaveError::Corrupt(format!("Deserialization: {e}")))?;

    // Validate versions
    if snapshot.save_version != SAVE_VERSION {
        return Err(SaveError::VersionMismatch {
            expected: SAVE_VERSION,
            found: snapshot.save_version,
        });
    }
    if snapshot.content_version != CONTENT_VERSION {
        return Err(SaveError::ContentMismatch {
            expected: CONTENT_VERSION.into(),
            found: snapshot.content_version,
        });
    }

    let (world, _) = restore_world(&snapshot, _blueprints)?;
    Ok((world, snapshot.seed))
}

/// Extract the map tiles as a flat Vec<Tile>.
fn tiles_from_map(map: &SmokeMap) -> Vec<Tile> {
    let mut tiles = Vec::with_capacity((map.width * map.height) as usize);
    for y in 0..map.height {
        for x in 0..map.width {
            tiles.push(map.get(x, y).unwrap_or(Tile::Wall));
        }
    }
    tiles
}

/// Build a snapshot from the current world state.
#[allow(clippy::explicit_counter_loop)]
fn build_snapshot(world: &mut World, seed: u64, turn: u64) -> RunSnapshot {
    let mut save_id_counter: u64 = 0;
    let mut entity_to_save_id: HashMap<Entity, SaveId> = HashMap::new();

    // Collect all entity IDs via query
    let mut query = world.query::<Entity>();
    let all_entities: Vec<Entity> = query.iter(world).collect();

    // First pass: assign SaveIds
    for &entity in &all_entities {
        let sid = SaveId(save_id_counter);
        save_id_counter += 1;
        entity_to_save_id.insert(entity, sid);
    }

    // Second pass: serialize each entity
    let mut entities_data: Vec<EntityData> = Vec::new();
    for &entity in &all_entities {
        let sid = entity_to_save_id[&entity];
        let mut data = EntityData {
            save_id: sid,
            blueprint_id: None,
            is_player: world.entity(entity).contains::<Player>(),
            blocks_movement: world.entity(entity).contains::<BlocksMovement>(),
            name: world.entity(entity).get::<Name>().map(|n| n.0.clone()),
            position: world.entity(entity).get::<Position>().copied(),
            pools: Vec::new(),
            statuses: Vec::new(),
            contains: Vec::new(),
            equipped_by: None,
            owned_by: None,
            summoned_by: None,
            location_owned: None,
            faction: None,
            item: world.entity(entity).contains::<Item>(),
            container_capacity: world.entity(entity).get::<Container>().map(|c| c.capacity.unwrap_or(-1)),
            equipment_slot: world.entity(entity).get::<EquipmentSlot>().map(|s| s.kind),
            usable: world.entity(entity).contains::<Usable>(),
            usable_consume: world.entity(entity).get::<Usable>().map(|u| u.consume_on_use).unwrap_or(false),
            usable_effects: world.entity(entity).get::<Usable>().map(|u| u.effects.clone()).unwrap_or_default(),
        };

        // Pools
        if let Some(pools) = world.entity(entity).get::<Pools>() {
            for pool in pools.iter() {
                data.pools.push(PoolSnapshot {
                    kind: pool.kind,
                    current: pool.current,
                    min: pool.min,
                    max: pool.max,
                });
            }
        }

        // Statuses
        if let Some(statuses) = world.entity(entity).get::<Statuses>() {
            for inst in &statuses.instances {
                let source_id = inst.source.and_then(|e| entity_to_save_id.get(&e).copied());
                data.statuses.push(StatusSnapshot {
                    status_id: inst.status_id.clone(),
                    remaining_duration: inst.remaining_duration,
                    stacks: inst.stacks,
                    source_id,
                });
            }
        }

        // Relationships
        if let Some(contained_in) = world.entity(entity).get::<ContainedIn>() {
            if let Some(&container_id) = entity_to_save_id.get(&contained_in.0) {
                data.contains.push(container_id);
            }
        }
        if let Some(equipped_by) = world.entity(entity).get::<EquippedBy>() {
            data.equipped_by = entity_to_save_id.get(&equipped_by.0).copied();
        }
        if let Some(owned_by) = world.entity(entity).get::<OwnedBy>() {
            data.owned_by = entity_to_save_id.get(&owned_by.0).copied();
        }
        if let Some(summoned_by) = world.entity(entity).get::<SummonedBy>() {
            data.summoned_by = entity_to_save_id.get(&summoned_by.0).copied();
        }
        if let Some(loc_owned) = world.entity(entity).get::<LocationOwned>() {
            data.location_owned = Some(loc_owned.0.clone());
        }
        if let Some(faction) = world.entity(entity).get::<FactionMember>() {
            data.faction = Some(faction.0.clone());
        }

        entities_data.push(data);
    }

    // Map
    let map = world.resource::<SmokeMap>();

    // Log
    let log = world.resource::<GameLog>();
    let log_entries: Vec<String> = log.iter().map(|e| format!("{:?}: {}", e.level, e.message)).collect();

    RunSnapshot {
        save_version: SAVE_VERSION,
        content_version: CONTENT_VERSION.into(),
        seed,
        turn,
        map_width: map.width,
        map_height: map.height,
        map_tiles: tiles_from_map(map),
        entities: entities_data,
        log_entries,
    }
}

/// Restore a world from a snapshot.
fn restore_world(
    snapshot: &RunSnapshot,
    _blueprints: &HashMap<String, EntityBlueprint>,
) -> Result<(World, HashMap<SaveId, Entity>), SaveError> {
    let mut world = World::new();

    // Restore map
    let map = SmokeMap::from_tiles(snapshot.map_width, snapshot.map_height, &snapshot.map_tiles);
    world.insert_resource(map);

    // Restore log
    let mut log = GameLog::default();
    for entry in &snapshot.log_entries {
        log.push(entry.clone(), crate::gamelog::LogLevel::Info);
    }
    world.insert_resource(log);

    // First pass: spawn all entities (empty)
    let mut save_id_to_entity: HashMap<SaveId, Entity> = HashMap::new();
    for ed in &snapshot.entities {
        let entity = world.spawn_empty();
        let id = entity.id();
        save_id_to_entity.insert(ed.save_id, id);
    }

    // Second pass: add components
    for ed in &snapshot.entities {
        let entity = save_id_to_entity[&ed.save_id];

        // Basic components
        if ed.is_player {
            world.entity_mut(entity).insert(Player);
        }
        if ed.blocks_movement {
            world.entity_mut(entity).insert(BlocksMovement);
        }
        if let Some(ref name) = ed.name {
            world.entity_mut(entity).insert(Name(name.clone()));
        }
        if let Some(pos) = ed.position {
            world.entity_mut(entity).insert(pos);
        }

        // Pools
        if !ed.pools.is_empty() {
            let pools: Vec<Pool> = ed
                .pools
                .iter()
                .map(|ps| Pool::new(ps.kind, ps.current, ps.min, ps.max))
                .collect();
            world.entity_mut(entity).insert(Pools::new(pools));
        }

        // Statuses
        if !ed.statuses.is_empty() {
            let instances: Vec<StatusInstance> = ed
                .statuses
                .iter()
                .map(|ss| StatusInstance {
                    status_id: ss.status_id.clone(),
                    remaining_duration: ss.remaining_duration,
                    stacks: ss.stacks,
                    source: ss.source_id.and_then(|sid| save_id_to_entity.get(&sid)).copied(),
                })
                .collect();
            world.entity_mut(entity).insert(Statuses { instances });
        }

        // Item
        if ed.item {
            world.entity_mut(entity).insert(Item);
        }

        // Container
        if let Some(cap) = ed.container_capacity {
            world.entity_mut(entity).insert(Container {
                capacity: if cap >= 0 { Some(cap) } else { None },
                allowed_tags: Vec::new(),
            });
        }

        // Equipment slot
        if let Some(kind) = ed.equipment_slot {
            world.entity_mut(entity).insert(EquipmentSlot {
                kind,
                accepted_tags: Vec::new(),
            });
        }

        // Usable
        if ed.usable {
            world.entity_mut(entity).insert(Usable {
                consume_on_use: ed.usable_consume,
                effects: ed.usable_effects.clone(),
            });
        }

        // Relationships
        for &container_sid in &ed.contains {
            if let Some(&container_entity) = save_id_to_entity.get(&container_sid) {
                world.entity_mut(entity).insert(ContainedIn(container_entity));
            }
        }
        if let Some(sid) = ed.equipped_by {
            if let Some(&e) = save_id_to_entity.get(&sid) {
                world.entity_mut(entity).insert(EquippedBy(e));
            }
        }
        if let Some(sid) = ed.owned_by {
            if let Some(&e) = save_id_to_entity.get(&sid) {
                world.entity_mut(entity).insert(OwnedBy(e));
            }
        }
        if let Some(sid) = ed.summoned_by {
            if let Some(&e) = save_id_to_entity.get(&sid) {
                world.entity_mut(entity).insert(SummonedBy(e));
            }
        }
        if let Some(ref loc) = ed.location_owned {
            world.entity_mut(entity).insert(LocationOwned(loc.clone()));
        }
        if let Some(ref faction) = ed.faction {
            world.entity_mut(entity).insert(FactionMember(faction.clone()));
        }
    }

    Ok((world, save_id_to_entity))
}

// ---------------------------------------------------------------------------
// Replay log
// ---------------------------------------------------------------------------

/// A record of intents for deterministic replay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentReplayLog {
    pub seed: u64,
    pub intents: Vec<String>,
}

impl IntentReplayLog {
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            intents: Vec::new(),
        }
    }

    pub fn record(&mut self, intent: String) {
        self.intents.push(intent);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Tile;
    
    fn test_snapshot() -> RunSnapshot {
        RunSnapshot {
            save_version: SAVE_VERSION,
            content_version: CONTENT_VERSION.into(),
            seed: 42,
            turn: 0,
            map_width: 10,
            map_height: 10,
            map_tiles: vec![Tile::Floor; 100],
            entities: vec![],
            log_entries: vec![],
        }
    }

    #[test]
    fn save_version_recorded() {
        let snap = test_snapshot();
        assert_eq!(snap.save_version, SAVE_VERSION);
    }

    #[test]
    fn content_version_mismatch_errors() {
        let mut snap = test_snapshot();
        snap.content_version = "wrong".into();
        let blueprints = HashMap::new();
        let ron = ron::ser::to_string(&snap).unwrap();
        let path = std::env::temp_dir().join("test_mismatch.ron");
        std::fs::write(&path, &ron).unwrap();
        let result = load_world(&path, &blueprints);
        assert!(result.is_err());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn player_position_persists() {
        let mut world = World::new();
        world.insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        world.insert_resource(GameLog::default());

        let _player = world.spawn((Player, Position { x: 5, y: 7 })).id();

        let snap = build_snapshot(&mut world, 42, 0);
        let player_data = snap.entities.iter().find(|e| e.is_player).unwrap();
        assert_eq!(player_data.position, Some(Position { x: 5, y: 7 }));
    }

    #[test]
    fn pools_persist() {
        let mut world = World::new();
        world.insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        world.insert_resource(GameLog::default());

        world.spawn((
            Player,
            Pools::new(vec![Pool::new(PoolKind::Health, 15, 0, 20)]),
        ));

        let snap = build_snapshot(&mut world, 42, 0);
        let pdata = snap.entities.iter().find(|e| e.is_player).unwrap();
        assert_eq!(pdata.pools.len(), 1);
        assert_eq!(pdata.pools[0].kind, PoolKind::Health);
        assert_eq!(pdata.pools[0].current, 15);
    }

    #[test]
    fn inventory_persists() {
        let mut world = World::new();
        world.insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        world.insert_resource(GameLog::default());

        let player = world.spawn(Player).id();
        let item = world.spawn((Item, Name("Sword".into()))).id();
        world.entity_mut(item).insert(ContainedIn(player));

        let snap = build_snapshot(&mut world, 42, 0);
        let item_data = snap.entities.iter().find(|e| e.name.as_deref() == Some("Sword")).unwrap();
        assert!(item_data.item);
        // ContainedIn should reference player's SaveId
        assert!(!item_data.contains.is_empty());
    }

    #[test]
    fn equipment_persists() {
        let mut world = World::new();
        world.insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        world.insert_resource(GameLog::default());

        let player = world.spawn(Player).id();
        let item = world
            .spawn((
                Item,
                Name("Shield".into()),
                EquipmentSlot {
                    kind: SlotKind::Armor,
                    accepted_tags: vec![],
                },
            ))
            .id();
        world.entity_mut(item).insert(EquippedBy(player));

        let snap = build_snapshot(&mut world, 42, 0);
        let item_data = snap.entities.iter().find(|e| e.name.as_deref() == Some("Shield")).unwrap();
        assert_eq!(item_data.equipment_slot, Some(SlotKind::Armor));
        assert!(item_data.equipped_by.is_some());
    }

    #[test]
    fn transient_summon_excluded() {
        // Summoned entities are still saved (they have SummonedBy).
        // But they can be identified and excluded on load if desired.
        let mut world = World::new();
        world.insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        world.insert_resource(GameLog::default());

        let summoner = world.spawn(Player).id();
        world.spawn((
            Name("Temporary".into()),
            SummonedBy(summoner),
        ));

        let snap = build_snapshot(&mut world, 42, 0);
        let summon = snap.entities.iter().find(|e| e.name.as_deref() == Some("Temporary")).unwrap();
        assert!(summon.summoned_by.is_some());
    }

    #[test]
    fn relationships_restore() {
        let mut world = World::new();
        world.insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        world.insert_resource(GameLog::default());

        let player = world.spawn(Player).id();
        let item = world.spawn((Item, Name("Ring".into()))).id();
        world.entity_mut(item).insert(EquippedBy(player));
        world.entity_mut(item).insert(OwnedBy(player));

        let snap = build_snapshot(&mut world, 42, 0);
        let blueprints = HashMap::new();
        let (restored, mapping) = restore_world(&snap, &blueprints).unwrap();

        // Find the restored item
        let item_data = snap.entities.iter().find(|e| e.name.as_deref() == Some("Ring")).unwrap();
        let restored_item = mapping[&item_data.save_id];
        let restored_entity = restored.entity(restored_item);
        assert!(restored_entity.contains::<EquippedBy>());
        assert!(restored_entity.contains::<OwnedBy>());
    }

    #[test]
    fn location_seed_persists() {
        let mut world = World::new();
        world.insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        world.insert_resource(GameLog::default());
        world.spawn(Player);

        let snap = build_snapshot(&mut world, 12345, 0);
        assert_eq!(snap.seed, 12345);
    }

    #[test]
    fn save_roundtrip() {
        let mut world = World::new();
        world.insert_resource(SmokeMap::new(5, 5, Tile::Floor));
        world.insert_resource(GameLog::default());

        let player = world.spawn((
            Player,
            Position { x: 2, y: 3 },
            Pools::new(vec![Pool::new(PoolKind::Health, 10, 0, 20)]),
        )).id();

        let _potion = world.spawn((
            Item,
            Name("Potion".into()),
            ContainedIn(player),
        )).id();

        let snap = build_snapshot(&mut world, 99, 0);

        // Serialize to RON string and back
        let ron = ron::ser::to_string(&snap).unwrap();
        let restored: RunSnapshot = ron::de::from_str(&ron).unwrap();

        assert_eq!(restored.seed, 99);
        assert_eq!(restored.entities.len(), 2);

        let player_data = restored.entities.iter().find(|e| e.is_player).unwrap();
        assert_eq!(player_data.position, Some(Position { x: 2, y: 3 }));
        assert_eq!(player_data.pools[0].current, 10);
    }

    #[test]
    fn fixed_intent_replay_is_deterministic() {
        let mut log = IntentReplayLog::new(42);
        log.record("move_north".into());
        log.record("wait".into());
        log.record("move_east".into());

        assert_eq!(log.seed, 42);
        assert_eq!(log.intents.len(), 3);

        // Serialize roundtrip
        let ron = ron::ser::to_string(&log).unwrap();
        let restored: IntentReplayLog = ron::de::from_str(&ron).unwrap();
        assert_eq!(restored.seed, 42);
        assert_eq!(restored.intents, vec!["move_north", "wait", "move_east"]);
    }
}
