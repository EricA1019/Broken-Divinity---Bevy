//! Save/load and replay system for the BD Kernel.
//!
//! Phase 17: Serializes world state to RON files, restores from snapshots,
//! and provides fixed-seed intent replay for deterministic testing.
//!
//! Uses a `SaveId(u64)` surrogate for Entity references since Bevy's `Entity`
//! type is not serializable. On save, each entity is assigned a SaveId.
//! On load, entities are spawned in order and a mapping is built.

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    colony::{
        stations::{Station, StationType},
        survivors::{Survivor, SurvivorTask},
    },
    combat::CombatRng,
    components::{
        BlocksMovement, ContentIdentity, ExitTile, Name, Player, Position, ResourceNode, Tile,
    },
    factory::EntityBlueprint,
    gamelog::{GameLog, LogEntry},
    inventory::{Container, EquipmentSlot, Item, SlotKind, Usable, UseEffect},
    map::SmokeMap,
    pools::{Pool, Pools},
    relationships::{ContainedIn, EquippedBy, FactionMember, LocationOwned, OwnedBy, SummonedBy},
    signals::PoolKind,
    spatial::{EntityScope, OutpostState, PersistentEntity, TransientEntity},
    statuses::{StatusInstance, Statuses},
};

// ---------------------------------------------------------------------------
// Versioning
// ---------------------------------------------------------------------------

/// Current save format version. Bump on breaking changes.
pub const SAVE_VERSION: u32 = 7;

/// Content version — corresponds to the content pack hash/date.
pub const CONTENT_VERSION: &str = "foundation-2026-07-24";

/// One-frame integration signal used to clear presentation-only interaction
/// state after durable world state has been restored.
#[derive(Resource, Debug, Clone, Copy)]
pub struct WorldJustRestored;

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
    pub content_id: Option<String>,
    pub position: Option<Position>,
    pub skill_progression: Option<crate::progression::SkillProgression>,
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
    pub scope: Option<EntityScope>,
    pub survivor_task: Option<SurvivorTaskSnapshot>,
    pub station_type: Option<StationType>,
    #[serde(default)]
    pub construction_site: Option<crate::colony::stations::ConstructionSite>,
    pub resource_node: Option<ResourceNode>,
    #[serde(default)]
    pub logistics_job: Option<crate::colony::logistics::LogisticsJob>,
    #[serde(default)]
    pub cargo: Option<crate::colony::logistics::Cargo>,
    #[serde(default)]
    pub direct_gather_progress: Option<crate::colony::resources::DirectGatherProgress>,
    pub exit_tile: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SurvivorTaskSnapshot {
    Idle,
    Gathering(PoolKind),
    Defending,
    AssignedTo(SaveId),
    Resting,
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
    pub session: crate::session::RunSession,
    #[serde(default)]
    pub last_completed_run: crate::session::LastCompletedRun,
    pub map_width: i32,
    pub map_height: i32,
    pub map_tiles: Vec<Tile>,
    pub entities: Vec<EntityData>,
    pub log_entries: Vec<LogEntry>,
    #[serde(default)]
    pub colony_storage: crate::colony::production::ColonyStorage,
    #[serde(default)]
    pub colony_resources: Vec<PoolSnapshot>,
    #[serde(default)]
    pub colony_raw_resources: BTreeMap<String, u32>,
    pub outpost_party: Vec<SaveId>,
    pub combat_rng: CombatRng,
    pub latest_daily_summary: crate::colony::production::LatestDailySummary,
}

pub const MANUAL_SLOT_FILE: &str = "manual-slot.ron";
const MANUAL_SLOT_TEMP_FILE: &str = "manual-slot.ron.tmp";

// ---------------------------------------------------------------------------
// Save/load operations
// ---------------------------------------------------------------------------

/// Default save directory: $XDG_DATA_HOME/broken-divinity or ~/.local/share/broken-divinity.
pub fn default_save_dir() -> PathBuf {
    let base = std::env::var("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
            PathBuf::from(home).join(".local").join("share")
        });
    base.join("broken-divinity")
}

/// Flag set when the TUI requests a save. Read by `bd_app` main loop to call save_world.
#[derive(Resource, Debug, Default)]
pub struct SaveRequest(pub bool);

/// Flag set when the TUI requests a load. Read by `bd_app` main loop to call load_world.
#[derive(Resource, Debug, Default)]
pub struct LoadRequest(pub bool);

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
                write!(
                    f,
                    "Save version mismatch: expected {expected}, found {found}"
                )
            }
            SaveError::ContentMismatch { expected, found } => {
                write!(
                    f,
                    "Content version mismatch: expected {expected}, found {found}"
                )
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
    save_dir: &Path,
) -> Result<PathBuf, SaveError> {
    let snapshot = build_snapshot(world, seed, turn);
    write_manual_snapshot(&snapshot, save_dir)
}

pub fn manual_slot_path(save_dir: &Path) -> PathBuf {
    save_dir.join(MANUAL_SLOT_FILE)
}

/// Replace the single Foundation manual slot atomically.
///
/// The temporary file is serialized, parsed, and validated before it replaces
/// the prior slot. A failed write or validation therefore leaves the prior
/// slot available.
pub fn save_manual_slot(world: &mut World, save_dir: &Path) -> Result<PathBuf, SaveError> {
    let session = world
        .get_resource::<crate::session::RunSession>()
        .cloned()
        .ok_or_else(|| SaveError::Corrupt("RunSession resource is missing".into()))?;
    let snapshot = build_snapshot(world, session.seed, session.turn);
    write_manual_snapshot(&snapshot, save_dir)
}

fn write_manual_snapshot(snapshot: &RunSnapshot, save_dir: &Path) -> Result<PathBuf, SaveError> {
    validate_snapshot(snapshot)?;

    fs::create_dir_all(save_dir).map_err(SaveError::Io)?;
    let final_path = manual_slot_path(save_dir);
    let temporary_path = save_dir.join(MANUAL_SLOT_TEMP_FILE);
    let content = ron::ser::to_string_pretty(&snapshot, ron::ser::PrettyConfig::default())
        .map_err(|error| SaveError::Corrupt(format!("Serialization: {error}")))?;
    fs::write(&temporary_path, content).map_err(SaveError::Io)?;

    let staged = load_snapshot(&temporary_path);
    if let Err(error) = staged {
        let _ = fs::remove_file(&temporary_path);
        return Err(error);
    }
    fs::rename(&temporary_path, &final_path).map_err(SaveError::Io)?;
    tracing::info!(
        "Saved {} entities to {}",
        snapshot.entities.len(),
        final_path.display()
    );
    Ok(final_path)
}

pub fn load_manual_slot(save_dir: &Path) -> Result<RunSnapshot, SaveError> {
    load_snapshot(&manual_slot_path(save_dir))
}

/// Deserialize a save file into a validated snapshot.
pub fn load_snapshot(path: &PathBuf) -> Result<RunSnapshot, SaveError> {
    let content = fs::read_to_string(path).map_err(SaveError::Io)?;
    let snapshot: RunSnapshot = ron::de::from_str(&content)
        .map_err(|e| SaveError::Corrupt(format!("Deserialization: {e}")))?;

    validate_snapshot(&snapshot)?;
    Ok(snapshot)
}

fn validate_snapshot(snapshot: &RunSnapshot) -> Result<(), SaveError> {
    if snapshot.save_version != SAVE_VERSION {
        return Err(SaveError::VersionMismatch {
            expected: SAVE_VERSION,
            found: snapshot.save_version,
        });
    }
    if snapshot.content_version != CONTENT_VERSION {
        return Err(SaveError::ContentMismatch {
            expected: CONTENT_VERSION.into(),
            found: snapshot.content_version.clone(),
        });
    }
    if snapshot.map_width <= 0
        || snapshot.map_height <= 0
        || snapshot.map_tiles.len() != (snapshot.map_width * snapshot.map_height) as usize
    {
        return Err(SaveError::Corrupt(
            "invalid map dimensions or tile count".into(),
        ));
    }

    let entities: HashMap<SaveId, &EntityData> = snapshot
        .entities
        .iter()
        .map(|entity| (entity.save_id, entity))
        .collect();
    if entities.len() != snapshot.entities.len() {
        return Err(SaveError::Corrupt("duplicate entity save ID".into()));
    }

    let require = |owner: SaveId, target: SaveId, relationship: &str| {
        if entities.contains_key(&target) {
            Ok(())
        } else {
            Err(SaveError::Corrupt(format!(
                "entity {owner:?} references missing {relationship} {target:?}"
            )))
        }
    };
    for entity in &snapshot.entities {
        for target in &entity.contains {
            require(entity.save_id, *target, "container")?;
        }
        for (target, relationship) in [
            (entity.equipped_by, "equipment owner"),
            (entity.owned_by, "owner"),
            (entity.summoned_by, "summoner"),
        ] {
            if let Some(target) = target {
                require(entity.save_id, target, relationship)?;
            }
        }
        for status in &entity.statuses {
            if let Some(source) = status.source_id {
                require(entity.save_id, source, "status source")?;
            }
        }
        if let Some(SurvivorTaskSnapshot::AssignedTo(station)) = entity.survivor_task {
            require(entity.save_id, station, "assigned station")?;
            if entities
                .get(&station)
                .is_none_or(|station| station.station_type.is_none())
            {
                return Err(SaveError::Corrupt(format!(
                    "survivor {:?} assignment target {:?} is not a station",
                    entity.save_id, station
                )));
            }
        }
    }
    for party_member in &snapshot.outpost_party {
        require(*party_member, *party_member, "outpost party member")?;
        if entities
            .get(party_member)
            .is_none_or(|entity| entity.survivor_task.is_none())
        {
            return Err(SaveError::Corrupt(format!(
                "outpost party reference {party_member:?} is not a survivor"
            )));
        }
    }
    Ok(())
}

/// Deserialize a save file and restore the world.
/// Returns the restored World and the seed.
pub fn load_world(
    path: &PathBuf,
    _blueprints: &HashMap<String, EntityBlueprint>,
) -> Result<(World, u64), SaveError> {
    let snapshot = load_snapshot(path)?;

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
            content_id: world
                .entity(entity)
                .get::<ContentIdentity>()
                .map(|id| id.0.clone()),
            position: world.entity(entity).get::<Position>().copied(),
            skill_progression: world
                .entity(entity)
                .get::<crate::progression::SkillProgression>()
                .cloned(),
            pools: Vec::new(),
            statuses: Vec::new(),
            contains: Vec::new(),
            equipped_by: None,
            owned_by: None,
            summoned_by: None,
            location_owned: None,
            faction: None,
            item: world.entity(entity).contains::<Item>(),
            container_capacity: world
                .entity(entity)
                .get::<Container>()
                .map(|c| c.capacity.unwrap_or(-1)),
            equipment_slot: world.entity(entity).get::<EquipmentSlot>().map(|s| s.kind),
            usable: world.entity(entity).contains::<Usable>(),
            usable_consume: world
                .entity(entity)
                .get::<Usable>()
                .map(|u| u.consume_on_use)
                .unwrap_or(false),
            usable_effects: world
                .entity(entity)
                .get::<Usable>()
                .map(|u| u.effects.clone())
                .unwrap_or_default(),
            scope: world.entity(entity).get::<EntityScope>().copied(),
            survivor_task: world
                .entity(entity)
                .get::<SurvivorTask>()
                .map(|task| match task {
                    SurvivorTask::Idle => SurvivorTaskSnapshot::Idle,
                    SurvivorTask::Gathering(kind) => SurvivorTaskSnapshot::Gathering(*kind),
                    SurvivorTask::Defending => SurvivorTaskSnapshot::Defending,
                    SurvivorTask::AssignedTo(station_bits) => SurvivorTaskSnapshot::AssignedTo(
                        entity_to_save_id
                            .get(&Entity::from_bits(*station_bits))
                            .copied()
                            .unwrap_or(SaveId(u64::MAX)),
                    ),
                    SurvivorTask::Resting => SurvivorTaskSnapshot::Resting,
                }),
            station_type: world.entity(entity).get::<StationType>().copied(),
            construction_site: world
                .entity(entity)
                .get::<crate::colony::stations::ConstructionSite>()
                .cloned(),
            resource_node: world.entity(entity).get::<ResourceNode>().cloned(),
            logistics_job: world
                .entity(entity)
                .get::<crate::colony::logistics::LogisticsJob>()
                .cloned(),
            cargo: world
                .entity(entity)
                .get::<crate::colony::logistics::Cargo>()
                .cloned(),
            direct_gather_progress: world
                .entity(entity)
                .get::<crate::colony::resources::DirectGatherProgress>()
                .cloned(),
            exit_tile: world.entity(entity).contains::<ExitTile>(),
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
    let log_entries: Vec<LogEntry> = log.iter().cloned().collect();
    let colony_storage = world
        .get_resource::<crate::colony::production::ColonyStorage>()
        .cloned()
        .unwrap_or_default();
    let colony_resources = world
        .get_resource::<crate::colony::production::ColonyResources>()
        .map(|resources| {
            resources
                .pools
                .iter()
                .map(|pool| PoolSnapshot {
                    kind: pool.kind,
                    current: pool.current,
                    min: pool.min,
                    max: pool.max,
                })
                .collect()
        })
        .unwrap_or_default();
    let colony_raw_resources = world
        .get_resource::<crate::colony::production::ColonyResources>()
        .map(|resources| resources.raw.clone())
        .unwrap_or_default();

    let mut session = world
        .get_resource::<crate::session::RunSession>()
        .cloned()
        .unwrap_or_else(|| crate::session::RunSession::new(seed));
    // Keep the legacy save API compatible while making the session the
    // serialized authority whenever the runtime provides one.
    session.seed = seed;
    session.turn = turn;
    let outpost_party = world
        .get_resource::<OutpostState>()
        .map(|outpost| {
            outpost
                .party
                .iter()
                .filter_map(|entity| entity_to_save_id.get(entity).copied())
                .collect()
        })
        .unwrap_or_default();
    let combat_rng = world
        .get_resource::<CombatRng>()
        .cloned()
        .unwrap_or_else(|| CombatRng::from_seed(seed));
    let latest_daily_summary = world
        .get_resource::<crate::colony::production::LatestDailySummary>()
        .cloned()
        .unwrap_or_default();
    let last_completed_run = world
        .get_resource::<crate::session::LastCompletedRun>()
        .cloned()
        .unwrap_or_default();

    RunSnapshot {
        save_version: SAVE_VERSION,
        content_version: CONTENT_VERSION.into(),
        seed,
        turn,
        session,
        last_completed_run,
        map_width: map.width,
        map_height: map.height,
        map_tiles: tiles_from_map(map),
        entities: entities_data,
        log_entries,
        colony_storage,
        colony_resources,
        colony_raw_resources,
        outpost_party,
        combat_rng,
        latest_daily_summary,
    }
}

/// Restore a world from a snapshot.
fn restore_world(
    snapshot: &RunSnapshot,
    _blueprints: &HashMap<String, EntityBlueprint>,
) -> Result<(World, HashMap<SaveId, Entity>), SaveError> {
    let mut world = World::new();
    let mapping = restore_snapshot_into(&mut world, snapshot, _blueprints)?;
    Ok((world, mapping))
}

/// Restore a validated snapshot into an existing application world.
/// Existing entities are cleared while plugin resources and schedules remain.
pub fn restore_snapshot_into(
    world: &mut World,
    snapshot: &RunSnapshot,
    _blueprints: &HashMap<String, EntityBlueprint>,
) -> Result<HashMap<SaveId, Entity>, SaveError> {
    // Validate every reference before mutating the live application world.
    validate_snapshot(snapshot)?;
    world.clear_entities();

    // Restore map
    let map = SmokeMap::from_tiles(snapshot.map_width, snapshot.map_height, &snapshot.map_tiles);
    world.insert_resource(map);

    // Restore log
    let mut log = GameLog::default();
    for entry in snapshot.log_entries.iter().rev() {
        log.push(entry.message.clone(), entry.level);
    }
    world.insert_resource(log);
    world.insert_resource(snapshot.session.clone());
    world.insert_resource(snapshot.last_completed_run.clone());
    world.insert_resource(snapshot.session.phase);
    world.insert_resource(crate::time::GameTime {
        day: snapshot.session.day,
        turn: snapshot.session.turn,
    });
    let colony_pools = if snapshot.colony_resources.is_empty() {
        crate::colony::production::ColonyResources::default().pools
    } else {
        Pools::new(
            snapshot
                .colony_resources
                .iter()
                .map(|pool| Pool::new(pool.kind, pool.current, pool.min, pool.max))
                .collect(),
        )
    };
    world.insert_resource(crate::colony::production::ColonyResources {
        pools: colony_pools,
        raw: snapshot.colony_raw_resources.clone(),
    });
    world.insert_resource(snapshot.colony_storage.clone());
    world.insert_resource(snapshot.combat_rng.clone());
    world.insert_resource(snapshot.latest_daily_summary.clone());
    world.init_resource::<crate::colony::production::DailyCycleDraft>();

    // First pass: spawn all entities (empty)
    let mut save_id_to_entity: HashMap<SaveId, Entity> = HashMap::new();
    for ed in &snapshot.entities {
        if save_id_to_entity.contains_key(&ed.save_id) {
            return Err(SaveError::Corrupt(format!(
                "duplicate entity save ID {:?}",
                ed.save_id
            )));
        }
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
        if let Some(ref content_id) = ed.content_id {
            world
                .entity_mut(entity)
                .insert(ContentIdentity(content_id.clone()));
        }
        if let Some(pos) = ed.position {
            world.entity_mut(entity).insert(pos);
        }
        if let Some(progression) = ed.skill_progression.clone() {
            world.entity_mut(entity).insert(progression);
        }
        if let Some(scope) = ed.scope {
            world.entity_mut(entity).insert(scope);
            match scope {
                EntityScope::RunPersistent | EntityScope::ColonyPersistent => {
                    world.entity_mut(entity).insert(PersistentEntity);
                }
                EntityScope::DungeonTransient => {
                    world.entity_mut(entity).insert(TransientEntity);
                }
            }
        }
        if let Some(task) = &ed.survivor_task {
            let task = match task {
                SurvivorTaskSnapshot::Idle => SurvivorTask::Idle,
                SurvivorTaskSnapshot::Gathering(kind) => SurvivorTask::Gathering(*kind),
                SurvivorTaskSnapshot::Defending => SurvivorTask::Defending,
                SurvivorTaskSnapshot::AssignedTo(station) => {
                    SurvivorTask::AssignedTo(save_id_to_entity[station].to_bits())
                }
                SurvivorTaskSnapshot::Resting => SurvivorTask::Resting,
            };
            world.entity_mut(entity).insert((Survivor, task));
        }
        if let Some(station_type) = ed.station_type {
            world.entity_mut(entity).insert((Station, station_type));
        }
        if let Some(construction_site) = ed.construction_site.clone() {
            world.entity_mut(entity).insert(construction_site);
        }
        if let Some(resource_node) = ed.resource_node.clone() {
            world.entity_mut(entity).insert(resource_node);
        }
        if let Some(logistics_job) = ed.logistics_job.clone() {
            world.entity_mut(entity).insert(logistics_job);
        }
        if let Some(cargo) = ed.cargo.clone() {
            world.entity_mut(entity).insert(cargo);
        }
        if let Some(progress) = ed.direct_gather_progress.clone() {
            world.entity_mut(entity).insert(progress);
        }
        if ed.exit_tile {
            world.entity_mut(entity).insert(ExitTile);
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
            let mut instances = Vec::with_capacity(ed.statuses.len());
            for ss in &ed.statuses {
                let source = match ss.source_id {
                    Some(source_id) => {
                        Some(*save_id_to_entity.get(&source_id).ok_or_else(|| {
                            SaveError::Corrupt(format!(
                                "status '{}' on {:?} references missing source {:?}",
                                ss.status_id, ed.save_id, source_id
                            ))
                        })?)
                    }
                    None => None,
                };
                instances.push(StatusInstance {
                    status_id: ss.status_id.clone(),
                    remaining_duration: ss.remaining_duration,
                    stacks: ss.stacks,
                    source,
                });
            }
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
            let container_entity = *save_id_to_entity.get(&container_sid).ok_or_else(|| {
                SaveError::Corrupt(format!(
                    "entity {:?} references missing container {:?}",
                    ed.save_id, container_sid
                ))
            })?;
            world
                .entity_mut(entity)
                .insert(ContainedIn(container_entity));
        }
        if let Some(sid) = ed.equipped_by {
            let equipped_by = *save_id_to_entity.get(&sid).ok_or_else(|| {
                SaveError::Corrupt(format!(
                    "entity {:?} references missing equipment owner {:?}",
                    ed.save_id, sid
                ))
            })?;
            world.entity_mut(entity).insert(EquippedBy(equipped_by));
        }
        if let Some(sid) = ed.owned_by {
            let owner = *save_id_to_entity.get(&sid).ok_or_else(|| {
                SaveError::Corrupt(format!(
                    "entity {:?} references missing owner {:?}",
                    ed.save_id, sid
                ))
            })?;
            world.entity_mut(entity).insert(OwnedBy(owner));
        }
        if let Some(sid) = ed.summoned_by {
            let summoner = *save_id_to_entity.get(&sid).ok_or_else(|| {
                SaveError::Corrupt(format!(
                    "entity {:?} references missing summoner {:?}",
                    ed.save_id, sid
                ))
            })?;
            world.entity_mut(entity).insert(SummonedBy(summoner));
        }
        if let Some(ref loc) = ed.location_owned {
            world.entity_mut(entity).insert(LocationOwned(loc.clone()));
        }
        if let Some(ref faction) = ed.faction {
            world
                .entity_mut(entity)
                .insert(FactionMember(faction.clone()));
        }
    }

    let party = snapshot
        .outpost_party
        .iter()
        .map(|save_id| save_id_to_entity[save_id])
        .collect();
    world.insert_resource(OutpostState {
        party,
        map: crate::colony::shelter::create_shelter_map(),
    });
    world.insert_resource(crate::time::ShouldAdvanceTime::default());
    world.insert_resource(crate::time::TimeAdvancePlan::default());
    world.insert_resource(crate::colony::stations::PendingStationAssignment::default());
    world.insert_resource(crate::colony::stations::BuildInteraction::default());

    // Derived activity is deliberately absent from the save format. Rebuild it
    // through the production resolver without granting a movement step or
    // emitting a second Blocked transition log.
    world.insert_resource(crate::colony::survivors::RecomputingWorkerActivity);
    let mut activity_schedule = Schedule::default();
    activity_schedule.add_systems(
        (
            crate::colony::logistics::process_logistics_workers,
            crate::colony::survivors::process_survivor_movement,
        )
            .chain(),
    );
    activity_schedule.run(world);
    world.remove_resource::<crate::colony::survivors::RecomputingWorkerActivity>();
    world.insert_resource(WorldJustRestored);

    Ok(save_id_to_entity)
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
    use bevy_app::App;

    fn test_snapshot() -> RunSnapshot {
        RunSnapshot {
            save_version: SAVE_VERSION,
            content_version: CONTENT_VERSION.into(),
            seed: 42,
            turn: 0,
            session: crate::session::RunSession::new(42),
            last_completed_run: crate::session::LastCompletedRun::default(),
            map_width: 10,
            map_height: 10,
            map_tiles: vec![Tile::Floor; 100],
            entities: vec![],
            log_entries: vec![],
            colony_storage: crate::colony::production::ColonyStorage::default(),
            colony_resources: Vec::new(),
            colony_raw_resources: BTreeMap::new(),
            outpost_party: Vec::new(),
            combat_rng: CombatRng::from_seed(42),
            latest_daily_summary: crate::colony::production::LatestDailySummary::default(),
        }
    }

    #[test]
    fn save_version_recorded() {
        let snap = test_snapshot();
        assert_eq!(snap.save_version, SAVE_VERSION);
    }

    #[test]
    fn log_message_and_level_round_trip_without_text_prefixing() {
        let mut source = App::new();
        source.add_plugins(crate::BdCorePlugin);
        source
            .world_mut()
            .resource_mut::<GameLog>()
            .push("Save requested.", crate::gamelog::LogLevel::Warn);
        source
            .world_mut()
            .resource_mut::<GameLog>()
            .push("Newest result.", crate::gamelog::LogLevel::Combat);
        let snapshot = build_snapshot(source.world_mut(), 42, 0);

        let mut restored = App::new();
        restored.add_plugins(crate::BdCorePlugin);
        restore_snapshot_into(restored.world_mut(), &snapshot, &HashMap::new()).unwrap();
        let entries = restored
            .world()
            .resource::<GameLog>()
            .iter()
            .collect::<Vec<_>>();

        assert_eq!(entries[0].message, "Newest result.");
        assert_eq!(entries[0].level, crate::gamelog::LogLevel::Combat);
        assert_eq!(entries[1].message, "Save requested.");
        assert_eq!(entries[1].level, crate::gamelog::LogLevel::Warn);
    }

    #[test]
    fn older_duplicate_player_supplies_save_is_rejected_readably() {
        let mut snap = test_snapshot();
        snap.save_version = SAVE_VERSION - 1;
        snap.entities.push(EntityData {
            save_id: SaveId(1),
            blueprint_id: Some("blueprint.player".into()),
            is_player: true,
            blocks_movement: false,
            name: Some("Player".into()),
            content_id: None,
            position: Some(Position { x: 1, y: 1 }),
            skill_progression: None,
            pools: vec![PoolSnapshot {
                kind: PoolKind::Supplies,
                current: 10,
                min: 0,
                max: 50,
            }],
            statuses: Vec::new(),
            contains: Vec::new(),
            equipped_by: None,
            owned_by: None,
            summoned_by: None,
            location_owned: None,
            faction: None,
            item: false,
            container_capacity: None,
            equipment_slot: None,
            usable: false,
            usable_consume: false,
            usable_effects: Vec::new(),
            scope: None,
            survivor_task: None,
            station_type: None,
            construction_site: None,
            resource_node: None,
            logistics_job: None,
            cargo: None,
            direct_gather_progress: None,
            exit_tile: false,
        });

        let error = validate_snapshot(&snap).expect_err("older split-ownership save must fail");
        assert!(
            error.to_string().contains("version"),
            "version rejection must be readable: {error}"
        );
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
        let item_data = snap
            .entities
            .iter()
            .find(|e| e.name.as_deref() == Some("Sword"))
            .unwrap();
        assert!(item_data.item);
        // ContainedIn should reference player's SaveId
        assert!(!item_data.contains.is_empty());
    }

    #[test]
    fn faction_identity_survives_save_load() {
        let mut world = World::new();
        world.insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        world.insert_resource(GameLog::default());
        let _enemy = world
            .spawn((
                Name("Placeholder Enemy".into()),
                FactionMember("faction.placeholder_a".into()),
            ))
            .id();

        let snap = build_snapshot(&mut world, 42, 0);
        let blueprints = HashMap::new();
        let (restored, mapping) = restore_world(&snap, &blueprints).unwrap();
        let saved_id = snap
            .entities
            .iter()
            .find(|entity| entity.name.as_deref() == Some("Placeholder Enemy"))
            .unwrap()
            .save_id;
        assert_eq!(
            restored
                .entity(mapping[&saved_id])
                .get::<FactionMember>()
                .unwrap()
                .0,
            "faction.placeholder_a"
        );
    }

    #[test]
    fn invalid_entity_reference_fails_safely() {
        let mut world = World::new();
        world.insert_resource(SmokeMap::new(5, 5, Tile::Floor));
        world.insert_resource(GameLog::default());
        world.spawn((Player, Name("Player".into())));
        world.spawn((Item, Name("Broken Item".into())));

        let mut snap = build_snapshot(&mut world, 42, 0);
        snap.entities
            .iter_mut()
            .find(|entity| entity.name.as_deref() == Some("Broken Item"))
            .unwrap()
            .contains
            .push(SaveId(999_999));

        let error = match restore_world(&snap, &HashMap::new()) {
            Ok(_) => panic!("invalid entity reference should fail safely"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("missing container"));
    }

    #[test]
    fn invalid_map_fails_before_world_restore() {
        let mut snap = test_snapshot();
        snap.map_tiles.pop();
        let ron = ron::ser::to_string(&snap).unwrap();
        let path = std::env::temp_dir().join("test_invalid_map.ron");
        std::fs::write(&path, ron).unwrap();
        let error = load_world(&path, &HashMap::new()).unwrap_err();
        assert!(error.to_string().contains("invalid map"));
        let _ = std::fs::remove_file(path);
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
        let item_data = snap
            .entities
            .iter()
            .find(|e| e.name.as_deref() == Some("Shield"))
            .unwrap();
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
        world.spawn((Name("Temporary".into()), SummonedBy(summoner)));

        let snap = build_snapshot(&mut world, 42, 0);
        let summon = snap
            .entities
            .iter()
            .find(|e| e.name.as_deref() == Some("Temporary"))
            .unwrap();
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
        let item_data = snap
            .entities
            .iter()
            .find(|e| e.name.as_deref() == Some("Ring"))
            .unwrap();
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
        let mut storage = crate::colony::production::ColonyStorage::default();
        storage.add_item("item.healing_potion");
        world.insert_resource(storage);
        world.insert_resource(crate::colony::production::ColonyResources {
            pools: Pools::new(vec![Pool::new(PoolKind::Supplies, 7, 0, 100)]),
            raw: BTreeMap::from([("resource.raw_timber".into(), 2)]),
        });
        let mut session = crate::session::RunSession::new(99);
        session.begin_dungeon("dungeon.foundation");
        session.mark_extracted();
        world.insert_resource(session);

        let player = world
            .spawn((
                Player,
                Position { x: 2, y: 3 },
                crate::progression::SkillProgression {
                    melee: 3,
                    ranged: 1,
                    repair: 0,
                    medicine: 2,
                },
                Pools::new(vec![Pool::new(PoolKind::Health, 10, 0, 20)]),
            ))
            .id();

        let _potion = world
            .spawn((Item, Name("Potion".into()), ContainedIn(player)))
            .id();

        let snap = build_snapshot(&mut world, 99, 0);

        // Serialize to RON string and back
        let ron = ron::ser::to_string(&snap).unwrap();
        let restored: RunSnapshot = ron::de::from_str(&ron).unwrap();

        assert_eq!(restored.seed, 99);
        assert_eq!(restored.entities.len(), 2);

        let player_data = restored.entities.iter().find(|e| e.is_player).unwrap();
        assert_eq!(player_data.position, Some(Position { x: 2, y: 3 }));
        assert_eq!(player_data.pools[0].current, 10);
        assert_eq!(player_data.skill_progression.as_ref().unwrap().melee, 3);
        assert_eq!(restored.colony_storage.count("item.healing_potion"), 1);
        assert_eq!(restored.colony_resources[0].current, 7);
        assert_eq!(restored.colony_raw_resources["resource.raw_timber"], 2);
        assert!(restored.session.extraction_applied);
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

    #[test]
    fn save_roundtrip_preserves_state() {
        let mut app = App::new();
        app.add_plugins(crate::BdCorePlugin);
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));

        let player = app
            .world_mut()
            .spawn((
                Player,
                Position { x: 5, y: 5 },
                Pools::new(vec![Pool::new(PoolKind::Health, 15, 0, 20)]),
            ))
            .id();

        // Save current state
        let save_dir = std::env::temp_dir().join("bd_roundtrip_test");
        let saved_path = save_world(app.world_mut(), 42, 0, &save_dir).unwrap();

        // Modify world state
        app.world_mut()
            .entity_mut(player)
            .insert(Position { x: 10, y: 10 });
        {
            let mut pools_borrow = app.world_mut().get_mut::<Pools>(player).unwrap();
            let hp = pools_borrow.get_mut(PoolKind::Health).unwrap();
            hp.current = 5;
        }

        // Load saved state (empty blueprints — player uses components directly)
        let blueprints = HashMap::new();
        let (mut loaded_world, _loaded_seed) = load_world(&saved_path, &blueprints).unwrap();

        // Verify restored position — the saved world has exactly one entity (player)
        let all_entities: Vec<Entity> =
            loaded_world.query::<Entity>().iter(&loaded_world).collect();
        assert!(
            !all_entities.is_empty(),
            "loaded world should have entities"
        );
        let loaded_player = all_entities[0];
        let loaded_pos = loaded_world.get::<Position>(loaded_player).unwrap();
        assert_eq!(
            *loaded_pos,
            Position { x: 5, y: 5 },
            "player position should be restored to saved state"
        );
        let loaded_hp = loaded_world
            .get::<Pools>(loaded_player)
            .unwrap()
            .get(PoolKind::Health)
            .unwrap()
            .current;
        assert_eq!(
            loaded_hp, 15,
            "player health should be restored to saved state"
        );

        // Cleanup
        let _ = std::fs::remove_dir_all(&save_dir);
    }

    #[test]
    fn snapshot_restores_into_existing_plugin_world() {
        let mut app = App::new();
        app.add_plugins(crate::BdFoundationPlugin);
        let player = app
            .world_mut()
            .spawn((
                Player,
                Position { x: 2, y: 2 },
                Pools::new(vec![Pool::new(PoolKind::Health, 17, 0, 20)]),
            ))
            .id();
        let snapshot = build_snapshot(app.world_mut(), 42, 3);

        app.world_mut()
            .entity_mut(player)
            .insert(Position { x: 9, y: 9 });
        restore_snapshot_into(app.world_mut(), &snapshot, &HashMap::new()).unwrap();

        let restored_position = app
            .world_mut()
            .query::<&Position>()
            .iter(app.world())
            .next()
            .copied();
        assert_eq!(restored_position, Some(Position { x: 2, y: 2 }));
        assert_eq!(
            *app.world().resource::<crate::spatial::GameMode>(),
            snapshot.session.phase
        );
    }
}
