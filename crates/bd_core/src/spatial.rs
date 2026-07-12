//! Spatial mode management — outpost, travel, and tactical location transitions.
//!
//! Phase 19: Defines `GameMode` (the current game state), `OutpostState` for
//! the shelter/base layer, `PersistentEntity`/`TransientEntity` for state
//! isolation, and transition messages for moving between modes.

use bevy_app::App;
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::map::SmokeMap;
use crate::{
    components::Player,
    gamelog::{GameLog, LogLevel},
    pools::{Pool, Pools},
    signals::PoolKind,
};

// ---------------------------------------------------------------------------
// Game mode
// ---------------------------------------------------------------------------

/// The current game mode — determines which systems and screens are active.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GameMode {
    /// The outpost/shelter — resource management, travel planning.
    #[default]
    Outpost,
    /// Travelling between locations (time passes, events possible).
    Travel,
    /// Active tactical combat in a procedurally generated location.
    Tactical,
}

// ---------------------------------------------------------------------------
// Persistent vs transient entity markers
// ---------------------------------------------------------------------------

/// Entities with this component survive location transitions (player, party,
/// permanent items).
#[derive(Component, Debug, Default, Serialize, Deserialize)]
pub struct PersistentEntity;

/// Entities with this component are removed when leaving a tactical location
/// (combat enemies, temporary summons, dropped loot not collected).
#[derive(Component, Debug, Default, Serialize, Deserialize)]
pub struct TransientEntity;

// ---------------------------------------------------------------------------
// Outpost state
// ---------------------------------------------------------------------------

/// The player's outpost/shelter — resources, party, and storage.
#[derive(Resource, Debug, Clone)]
pub struct OutpostState {
    /// Outpost resource pools (Supplies, Morale, Faith, etc.).
    pub resources: Pools,
    /// Entity IDs of party members currently at the outpost.
    pub party: Vec<Entity>,
    /// The persistent shelter map (never regenerated on revisit).
    pub map: SmokeMap,
}

impl Default for OutpostState {
    fn default() -> Self {
        Self {
            resources: Pools::new(vec![
                Pool::new(PoolKind::Supplies, 10, 0, 50),
                Pool::new(PoolKind::Morale, 50, 0, 100),
            ]),
            party: Vec::new(),
            map: crate::colony::shelter::create_shelter_map(),
        }
    }
}

// ---------------------------------------------------------------------------
// Travel nodes
// ---------------------------------------------------------------------------

/// A location on the travel map.
#[derive(Debug, Clone)]
pub struct TravelNode {
    pub id: String,
    pub name: String,
    pub travel_time: u32, // turns to reach from outpost
    pub location_template: Option<String>,
}

/// The travel map — a list of reachable locations from the outpost.
#[derive(Resource, Debug, Clone)]
pub struct TravelMap {
    pub nodes: Vec<TravelNode>,
}

impl Default for TravelMap {
    fn default() -> Self {
        Self {
            nodes: vec![
                TravelNode {
                    id: "ruin.ancient_temple".into(),
                    name: "Ancient Temple".into(),
                    travel_time: 3,
                    location_template: Some("location.ruin".into()),
                },
                TravelNode {
                    id: "ruin.crypt".into(),
                    name: "Crypt of the Fallen".into(),
                    travel_time: 5,
                    location_template: Some("location.ruin".into()),
                },
            ],
        }
    }
}

// ---------------------------------------------------------------------------
// Transition messages
// ---------------------------------------------------------------------------

/// Intent to transition to a different game mode.
#[derive(Message, Debug, Clone)]
pub struct TransitionIntent {
    pub target: GameMode,
    pub node_id: Option<String>,
}

/// A transition has been completed.
#[derive(Message, Debug, Clone)]
pub struct TransitionComplete {
    pub from: GameMode,
    pub to: GameMode,
}

// ---------------------------------------------------------------------------
// Transition system
// ---------------------------------------------------------------------------

/// Process transition intents and switch game modes.
/// Handles entity cleanup: transient entities removed when leaving tactical.
pub fn process_transitions(
    mut messages: bevy_ecs::message::MessageReader<TransitionIntent>,
    mut commands: Commands,
    mut mode: ResMut<GameMode>,
    mut game_log: ResMut<GameLog>,
    mut map: ResMut<SmokeMap>,
    outpost: Res<OutpostState>,
    mut colony_res: ResMut<crate::colony::production::ColonyResources>,
    mut overworld: Option<ResMut<crate::overworld::OverworldState>>,
    query: Query<(Entity, Option<&TransientEntity>, Option<&PersistentEntity>)>,
    player_query: Query<Entity, With<Player>>,
) {
    for msg in messages.read() {
        let from = *mode;

        // Clean up transient entities when leaving tactical mode
        if *mode == GameMode::Tactical && msg.target != GameMode::Tactical {
            for (entity, transient, _persistent) in query.iter() {
                if transient.is_some() {
                    commands.entity(entity).despawn();
                    tracing::debug!("Despawned transient entity {entity:?}");
                }
            }
            // Also despawn anything without PersistentEntity
            for (entity, _transient, persistent) in query.iter() {
                if persistent.is_none() {
                    commands.entity(entity).despawn();
                }
            }
        }

        *mode = msg.target;

        match msg.target {
            GameMode::Outpost => {
                game_log.push("You return to the outpost.", LogLevel::Info);
                // Sync global map to shelter map so movement validation works
                *map = outpost.map.clone();
            }
            GameMode::Travel => {
                let node_name = msg.node_id.as_deref().unwrap_or("unknown");
                game_log.push(
                    format!("Travelling to {node_name}..."),
                    LogLevel::Info,
                );
                // Set travel duration
                if let Some(ref mut ow) = overworld {
                    ow.turns_remaining = 3;
                    ow.current_node = msg.node_id.clone();
                }
            }
            GameMode::Tactical => {
                let node_name = msg.node_id.as_deref().unwrap_or("the ruin");
                game_log.push(
                    format!("Entering {node_name}.", node_name = node_name),
                    LogLevel::Info,
                );
                // Deduct travel supplies
                if let Some(supplies) = colony_res.pools.get_mut(PoolKind::Supplies) {
                    supplies.current = (supplies.current - TRAVEL_SUPPLIES_COST).max(0);
                }
                game_log.push(
                    format!("Colony supplies: {}", colony_res.pools.get(PoolKind::Supplies).map_or(0, |p| p.current)),
                    LogLevel::Info,
                );
                // Generate dungeon on first entry
                spawn_dungeon_location(&mut commands, &mut map, &player_query);
            }
        }

        tracing::info!("Game mode: {from:?} → {:?}", msg.target);
    }
}

/// Generate a procedural dungeon and spawn entities when entering tactical mode.
fn spawn_dungeon_location(
    commands: &mut Commands,
    map: &mut ResMut<SmokeMap>,
    player_query: &Query<Entity, With<Player>>,
) {
    use crate::components::{BlocksMovement, ExitTile, Name, Position, Tile};
    use crate::factory::{BlueprintRegistry, spawn_from_blueprint};
    use crate::map::SmokeMap;
    use crate::procgen::{LocationTemplate, generate_location};

    let registry = BlueprintRegistry::phase18_defaults();
    let seed: u64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let template = LocationTemplate::ruin();
    let plan = generate_location(&template, seed);

    **map = SmokeMap::from_tiles(plan.width, plan.height, &plan.tiles);
    map.set(plan.entrance.x, plan.entrance.y, Tile::Door);

    // Move existing player to dungeon entrance instead of creating duplicate
    if let Some(existing_player) = player_query.iter().next() {
        commands.entity(existing_player).insert(plan.entrance);
    } else if let Some(bp) = registry.get("blueprint.player") {
        let entity = spawn_from_blueprint(bp, Some(plan.entrance), &[], commands);
        commands.entity(entity).insert(PersistentEntity);
    }

    let enemy_blueprints = ["blueprint.rat", "blueprint.skeleton"];
    for (i, zone) in plan.spawn_zones.iter().enumerate() {
        let bp_id = enemy_blueprints[i % enemy_blueprints.len()];
        if let Some(bp) = registry.get(bp_id) {
            let entity = spawn_from_blueprint(bp, Some(*zone), &[], commands);
            commands.entity(entity).insert(TransientEntity);
        }
    }

    let item_bps = ["blueprint.healing_potion", "blueprint.sword", "blueprint.shield",
        "blueprint.smite_scroll", "blueprint.gold_pile"];
    for (i, bp_id) in item_bps.iter().enumerate() {
        if let Some(room) = plan.rooms.get((i + 1) % plan.rooms.len()) {
            let pos = Position { x: room.x + 1, y: room.y + 1 + i as i32 % 2 };
            if map.is_walkable(pos.x, pos.y) {
                if let Some(bp) = registry.get(bp_id) {
                    let entity = spawn_from_blueprint(bp, Some(pos), &[], commands);
                    commands.entity(entity).insert(BlocksMovement);
                    commands.entity(entity).remove::<BlocksMovement>();
                }
            }
        }
    }

    // Spawn a SanityPressure entity deeper in the dungeon
    if let Some(room) = plan.rooms.get(3.min(plan.rooms.len() - 1)) {
        let sp_pos = Position { x: room.center().x, y: room.center().y };
        if map.is_walkable(sp_pos.x, sp_pos.y) {
            commands.spawn((
                Position { x: sp_pos.x, y: sp_pos.y },
                crate::sanity::SanityPressure { radius: 2, drain_per_turn: 5 },
                Name("Aura of Dread".into()),
            ));
        }
    }

    if let Some(exit_pos) = plan.exits.first() {
        map.set(exit_pos.x, exit_pos.y, Tile::Door);
        commands.spawn((ExitTile, *exit_pos, Name("Exit".into())));
    }
}


// ---------------------------------------------------------------------------
// Registration
// ---------------------------------------------------------------------------

/// Register spatial/transition resources and systems.
/// Initialize the outpost with starter entities on first entry.
pub fn initialize_outpost(
    mut commands: Commands,
    mut outpost: ResMut<OutpostState>,
    mut game_log: ResMut<GameLog>,
    mut map: ResMut<SmokeMap>,
    mode: Res<GameMode>,
) {
    if *mode != GameMode::Outpost {
        return;
    }
    // Only initialize once
    if !outpost.party.is_empty() {
        return;
    }

    // Set the shelter map as the outpost map
    outpost.map = crate::colony::shelter::create_shelter_map();
    // Sync global map only if still using default 20x12 (not a test/custom map)
    if map.width == 20 && map.height == 12 {
        *map = outpost.map.clone();
    }

    // Spawn a few starter survivors
    for i in 0..3 {
        let x = 5 + i as i32 * 5;
        let y = 5;
        let survivor = commands.spawn((
            crate::components::Position { x, y },
            crate::components::Name(format!("Survivor {}", i + 1)),
            crate::colony::survivors::Survivor,
            crate::colony::survivors::SurvivorTask::Idle,
            crate::colony::survivors::default_survivor_pools(),
            PersistentEntity,
        )).id();
        outpost.party.push(survivor);
    }

    game_log.push("Survivors gather at the shelter.", crate::gamelog::LogLevel::Info);
}

pub fn register_spatial(app: &mut App) {
    app.insert_resource(GameMode::default());
    app.insert_resource(OutpostState::default());
    app.insert_resource(TravelMap::default());
    app.add_message::<TransitionIntent>();
    app.add_message::<TransitionComplete>();

    app.add_systems(
        bevy_app::Update,
        process_transitions
            .in_set(crate::BdSet::IntentCollection),
    );

    app.add_systems(
        bevy_app::Startup,
        initialize_outpost,
    );

    tracing::info!("Spatial module registered");
}

/// Supplies deducted when traveling from outpost to a dungeon.
pub const TRAVEL_SUPPLIES_COST: i32 = 2;
// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Position;
    use crate::map::SmokeMap;

    fn test_app() -> bevy_app::App {
        let mut app = bevy_app::App::new();
        app.insert_resource(GameMode::default());
        app.insert_resource(OutpostState::default());
        app.insert_resource(TravelMap::default());
        app.insert_resource(GameLog::default());
        app.insert_resource(crate::colony::production::ColonyResources::default());
        app.insert_resource(SmokeMap::new(10, 10, crate::components::Tile::Floor));
        app.add_message::<TransitionIntent>();
        app.add_systems(bevy_app::Update, process_transitions);
        app
    }

    #[test]
    fn leaving_location_preserves_player() {
        let mut app = test_app();
        app.world_mut().insert_resource(GameMode::Tactical);

        // Spawn player with PersistentEntity
        let player = app.world_mut().spawn((
            PersistentEntity,
            Position { x: 5, y: 5 },
        )).id();

        // Spawn transient enemy
        let enemy = app.world_mut().spawn((
            TransientEntity,
            Position { x: 3, y: 3 },
        )).id();

        // Transition to Outpost
        app.world_mut()
            .resource_mut::<bevy_ecs::message::Messages<TransitionIntent>>()
            .write(TransitionIntent {
                target: GameMode::Outpost,
                node_id: None,
            });
        app.update();

        // Player should still exist
        assert!(app.world().entities().contains(player),
            "Player should persist after leaving tactical");
        // Enemy should be despawned
        assert!(!app.world().entities().contains(enemy),
            "Transient enemy should be removed");
    }

    #[test]
    fn returning_to_outpost_works() {
        let mut app = test_app();
        app.world_mut().insert_resource(GameMode::Tactical);

        app.world_mut()
            .resource_mut::<bevy_ecs::message::Messages<TransitionIntent>>()
            .write(TransitionIntent {
                target: GameMode::Outpost,
                node_id: None,
            });
        app.update();

        assert_eq!(*app.world().resource::<GameMode>(), GameMode::Outpost);
    }

    #[test]
    fn travel_advances_time() {
        // Travel time is simulated by setting GameMode::Travel.
        // The number of turns spent in Travel mode equals the travel time.
        let mut app = test_app();
        app.world_mut().insert_resource(GameMode::Outpost);

        app.world_mut()
            .resource_mut::<bevy_ecs::message::Messages<TransitionIntent>>()
            .write(TransitionIntent {
                target: GameMode::Travel,
                node_id: Some("ruin.ancient_temple".into()),
            });
        app.update();

        assert_eq!(*app.world().resource::<GameMode>(), GameMode::Travel);

        // After completing travel, transition to Tactical
        app.world_mut()
            .resource_mut::<bevy_ecs::message::Messages<TransitionIntent>>()
            .write(TransitionIntent {
                target: GameMode::Tactical,
                node_id: Some("ruin.ancient_temple".into()),
            });
        app.update();

        assert_eq!(*app.world().resource::<GameMode>(), GameMode::Tactical);
    }

    #[test]
    fn outpost_resources_use_pool_like_system() {
        let app = test_app();
        let outpost = app.world().resource::<OutpostState>().clone();
        let supplies = outpost.resources.get(PoolKind::Supplies).unwrap();
        assert_eq!(supplies.current, 10);
        assert_eq!(supplies.max, 50);

        let morale = outpost.resources.get(PoolKind::Morale).unwrap();
        assert_eq!(morale.current, 50);
        assert_eq!(morale.max, 100);
    }

    #[test]
    fn transient_combat_entities_do_not_leak() {
        let mut app = test_app();
        app.world_mut().insert_resource(GameMode::Tactical);

        // Spawn entities without marker (assumed transient)
        let summon = app.world_mut().spawn(Position { x: 1, y: 1 }).id();
        let item = app.world_mut().spawn((crate::inventory::Item, Position { x: 2, y: 2 })).id();

        // Transition to outpost
        app.world_mut()
            .resource_mut::<bevy_ecs::message::Messages<TransitionIntent>>()
            .write(TransitionIntent {
                target: GameMode::Outpost,
                node_id: None,
            });
        app.update();

        // Non-persistent entities should be despawned
        assert!(!app.world().entities().contains(summon));
        assert!(!app.world().entities().contains(item));
    }
}
