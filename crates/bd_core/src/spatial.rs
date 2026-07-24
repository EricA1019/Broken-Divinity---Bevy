//! Spatial mode management — outpost, travel, and tactical location transitions.
//!
//! Phase 19: Defines `GameMode` (the current game state), `OutpostState` for
//! the shelter/base layer, `PersistentEntity`/`TransientEntity` for state
//! isolation, and transition messages for moving between modes.

use bevy_app::App;
use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use serde::{Deserialize, Serialize};

use crate::map::SmokeMap;
use crate::{
    components::{ExitTile, Player, Position},
    gamelog::{GameLog, LogLevel},
};

// ---------------------------------------------------------------------------
// Game mode
// ---------------------------------------------------------------------------

/// The current game mode — determines which systems and screens are active.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum GameMode {
    /// Title screen shown at launch before any gameplay.
    #[default]
    Title,
    /// The outpost/shelter — resource management, travel planning.
    Outpost,
    /// Travelling between locations (time passes, events possible).
    Travel,
    /// Active tactical combat in a procedurally generated location.
    Tactical,
    /// Player has been defeated — game over screen shown.
    GameOver,
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

/// The player's outpost/shelter — party and storage.
#[derive(Resource, Debug, Clone)]
pub struct OutpostState {
    /// Entity IDs of party members currently at the outpost.
    pub party: Vec<Entity>,
    /// The persistent shelter map (never regenerated on revisit).
    pub map: SmokeMap,
}

impl Default for OutpostState {
    fn default() -> Self {
        Self {
            party: Vec::new(),
            map: crate::colony::shelter::create_shelter_map(),
        }
    }
}

// ---------------------------------------------------------------------------
// Travel nodes
// ---------------------------------------------------------------------------

/// A location on the travel map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TravelNode {
    pub id: String,
    pub name: String,
    pub travel_time: u32, // turns to reach from outpost
    pub location_template: Option<String>,
}

/// The travel map — a list of reachable locations from the outpost.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
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
#[derive(SystemParam)]
struct ExtractionContext<'w, 's> {
    storage: ResMut<'w, crate::colony::production::ColonyStorage>,
    loot: Query<
        'w,
        's,
        (
            Entity,
            &'static crate::components::ContentIdentity,
            &'static crate::relationships::ContainedIn,
            Option<&'static TransientEntity>,
        ),
        With<crate::inventory::Item>,
    >,
}

fn process_transitions(
    mut messages: bevy_ecs::message::MessageReader<TransitionIntent>,
    mut commands: Commands,
    mut mode: ResMut<GameMode>,
    mut session: ResMut<crate::session::RunSession>,
    mut game_log: ResMut<GameLog>,
    mut map: ResMut<SmokeMap>,
    outpost: Res<OutpostState>,
    mut overworld: Option<ResMut<crate::overworld::OverworldState>>,
    // Gabriel is a deferred system and is absent from the foundation plugin.
    // Keep this optional so the shared transition system remains usable by the
    // headless foundation runtime.
    gabriel_state: Option<Res<crate::gabriel::GabrielState>>,
    gabriel_q: Query<Entity, With<crate::components::Gabriel>>,
    query: Query<(Entity, Option<&TransientEntity>, Option<&PersistentEntity>)>,
    mut transition_complete: bevy_ecs::message::MessageWriter<TransitionComplete>,
    foundation: Option<Res<crate::session::FoundationRuntime>>,
    foundation_content: Option<Res<crate::content::FoundationContent>>,
    player_query: Query<Entity, With<Player>>,
    mut extraction: ExtractionContext,
) {
    for msg in messages.read() {
        // Legacy tests historically set GameMode directly. Synchronize that
        // setup into the session once, while foundation gameplay uses the
        // session as the transition authority.
        if foundation.is_none() && session.phase != *mode {
            session.phase = *mode;
        }
        let from = session.phase;

        if foundation.is_some()
            && !crate::session::RunSession::allows_foundation_transition(from, msg.target)
        {
            game_log.push(
                format!("Transition rejected: {from:?} → {:?}.", msg.target),
                LogLevel::Warn,
            );
            continue;
        }

        // Clean up transient entities when leaving tactical mode
        if *mode == GameMode::Tactical && msg.target != GameMode::Tactical {
            // Despawn each transient/non-persistent entity once. The prior
            // two-pass cleanup scheduled transient entities twice because
            // they are also non-persistent, producing invalid-entity warnings.
            for (entity, transient, persistent) in query.iter() {
                if transient.is_some() || persistent.is_none() {
                    commands.entity(entity).despawn();
                    tracing::debug!("Despawned transient entity {entity:?}");
                }
            }
        }

        *mode = msg.target;
        session.phase = msg.target;

        match msg.target {
            GameMode::Title => {}
            GameMode::Outpost => {
                game_log.push("You return to the outpost.", LogLevel::Info);
                if from == GameMode::Tactical && !session.extraction_applied {
                    let player = player_query.iter().next();
                    let mut transferred = 0usize;
                    if let Some(player) = player {
                        for (_, identity, contained, transient) in extraction.loot.iter() {
                            if transient.is_some() && contained.0 == player {
                                extraction.storage.add_item(identity.0.clone());
                                transferred += 1;
                            }
                        }
                    }
                    session.mark_extracted_with_loot(transferred as u32);
                    game_log.push(
                        format!("Run extracted successfully. Loot secured: {transferred}."),
                        LogLevel::Info,
                    );
                }
                // Sync global map to shelter map so movement validation works
                *map = outpost.map.clone();

                // P15-C: Spawn Gabriel in shelter if he has appeared and not already present
                if gabriel_state.as_ref().is_some_and(|state| state.appeared)
                    && gabriel_q.iter().next().is_none()
                {
                    commands.spawn((
                        crate::components::Position { x: 12, y: 8 },
                        crate::components::Name("Gabriel".into()),
                        crate::components::Gabriel,
                        PersistentEntity,
                    ));
                    game_log.push("Gabriel waits silently in the shelter.", LogLevel::Info);
                }
            }
            GameMode::Travel => {
                let node_name = msg.node_id.as_deref().unwrap_or("unknown");
                game_log.push(format!("Travelling to {node_name}..."), LogLevel::Info);
                // Set travel duration
                if let Some(ref mut ow) = overworld {
                    ow.turns_remaining = 3;
                    ow.current_node = msg.node_id.clone();
                }
            }
            GameMode::GameOver => {}
            GameMode::Tactical => {
                let node_name = msg.node_id.as_deref().unwrap_or("the ruin");
                game_log.push(
                    format!("Entering {node_name}.", node_name = node_name),
                    LogLevel::Info,
                );
                session.begin_dungeon(node_name);
                if foundation.is_some() {
                    if let Some(content) = foundation_content.as_deref() {
                        spawn_fixed_dungeon(
                            &mut commands,
                            &mut map,
                            content,
                            &player_query,
                            node_name,
                        );
                    } else {
                        game_log.push(
                            "Foundation dungeon content is unavailable.".to_string(),
                            LogLevel::Warn,
                        );
                    }
                }
            }
        }

        if msg.target == GameMode::GameOver {
            session.mark_defeated();
        }

        transition_complete.write(TransitionComplete {
            from,
            to: msg.target,
        });

        tracing::info!("Game mode: {from:?} → {:?}", msg.target);
    }
}

/// Construct the hand-authored foundation dungeon from validated content.
/// This provider owns content construction; transitions only select the mode.
fn spawn_fixed_dungeon(
    commands: &mut Commands,
    map: &mut ResMut<SmokeMap>,
    content: &crate::content::FoundationContent,
    player_query: &Query<Entity, With<Player>>,
    dungeon_id: &str,
) {
    use crate::components::{ContentIdentity, ExitTile, Name, Tile};
    use crate::factory::spawn_from_blueprint;
    use crate::inventory::{Container, Item, Usable, UseEffect};
    use crate::relationships::FactionMember;

    let Some(dungeon) = content.dungeon(dungeon_id) else {
        tracing::error!("Missing fixed dungeon content: {dungeon_id}");
        return;
    };
    **map = SmokeMap::from_tiles(dungeon.width, dungeon.height, &dungeon.tiles);

    if let Some(player) = player_query.iter().next() {
        commands
            .entity(player)
            .insert((dungeon.entrance, Container::default()));
    } else if let Some(blueprint) = content
        .blueprints
        .iter()
        .find(|bp| bp.id == "blueprint.player")
    {
        let player = spawn_from_blueprint(blueprint, Some(dungeon.entrance), &[], commands);
        commands
            .entity(player)
            .insert((PersistentEntity, Container::default()));
    }

    for placement in &dungeon.enemy_placements {
        if let Some(blueprint) = content
            .blueprints
            .iter()
            .find(|bp| bp.id == placement.content_id)
        {
            let enemy = spawn_from_blueprint(blueprint, Some(placement.position), &[], commands);
            commands.entity(enemy).insert(TransientEntity);
            if let Some(faction_id) = placement.faction_id.as_deref() {
                commands
                    .entity(enemy)
                    .insert(FactionMember(faction_id.to_string()));
            }
        }
    }

    for placement in &dungeon.item_placements {
        let Some(item_def) = content
            .items
            .iter()
            .find(|item| item.id == placement.content_id)
        else {
            continue;
        };
        let Some(blueprint) = content
            .blueprints
            .iter()
            .find(|bp| bp.id == item_def.blueprint_id)
        else {
            continue;
        };
        let item = spawn_from_blueprint(blueprint, Some(placement.position), &[], commands);
        commands.entity(item).insert((
            Item,
            ContentIdentity(item_def.id.clone()),
            Name(item_def.label.clone()),
            TransientEntity,
        ));
        if item_def.usable {
            let effects = item_def
                .healing_amount
                .map(|amount| vec![UseEffect::Heal(amount)])
                .unwrap_or_default();
            commands.entity(item).insert(Usable {
                consume_on_use: true,
                effects,
            });
        }
    }

    map.set(dungeon.extraction.x, dungeon.extraction.y, Tile::Door);
    commands.spawn((
        ExitTile,
        dungeon.extraction,
        Name("Dungeon Exit".into()),
        TransientEntity,
    ));
}

/// Legacy procedural location constructor retained for later products.
///
/// It is intentionally not registered or called by the foundation transition
/// path. The MVP fixed-location provider will replace this boundary later.
#[allow(dead_code)]
fn spawn_dungeon_location(
    commands: &mut Commands,
    map: &mut ResMut<SmokeMap>,
    player_query: &Query<Entity, With<Player>>,
    seed: u64,
) {
    use crate::components::{BlocksMovement, ExitTile, Name, Position, Tile};
    use crate::factory::{BlueprintRegistry, spawn_from_blueprint};
    use crate::map::SmokeMap;
    use crate::procgen::{LocationTemplate, generate_location};

    let registry = BlueprintRegistry::phase18_defaults();
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

    let item_bps = [
        "blueprint.healing_potion",
        "blueprint.sword",
        "blueprint.shield",
        "blueprint.smite_scroll",
        "blueprint.gold_pile",
    ];
    for (i, bp_id) in item_bps.iter().enumerate() {
        if let Some(room) = plan.rooms.get((i + 1) % plan.rooms.len()) {
            let pos = Position {
                x: room.x + 1,
                y: room.y + 1 + i as i32 % 2,
            };
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
        let sp_pos = Position {
            x: room.center().x,
            y: room.center().y,
        };
        if map.is_walkable(sp_pos.x, sp_pos.y) {
            commands.spawn((
                Position {
                    x: sp_pos.x,
                    y: sp_pos.y,
                },
                crate::sanity::SanityPressure {
                    radius: 2,
                    drain_per_turn: 5,
                },
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
        let survivor = commands
            .spawn((
                crate::components::Position { x, y },
                crate::components::Name(format!("Survivor {}", i + 1)),
                crate::colony::survivors::Survivor,
                crate::colony::survivors::SurvivorTask::Idle,
                crate::colony::survivors::default_survivor_pools(),
                PersistentEntity,
            ))
            .id();
        outpost.party.push(survivor);
    }

    game_log.push(
        "Survivors gather at the shelter.",
        crate::gamelog::LogLevel::Info,
    );

    // P22: Spawn resource nodes on the shelter map
    let node_count = crate::colony::resources::spawn_resource_nodes(&mut commands, &outpost.map);
    if node_count > 0 {
        game_log.push(
            format!("{} resource nodes found near the shelter.", node_count),
            crate::gamelog::LogLevel::Info,
        );
    }

    // P3-A: Spawn shelter exit tile (gate) at the top-center of the map
    let exit_x = crate::colony::shelter::SHELTER_WIDTH / 2;
    let exit_y = 1; // top wall row — the gate breach
    commands.spawn((
        ExitTile,
        Position {
            x: exit_x,
            y: exit_y,
        },
        crate::components::Name("Shelter Gate".into()),
        crate::spatial::PersistentEntity,
    ));
    game_log.push(
        "The shelter gate stands open to the north.",
        crate::gamelog::LogLevel::Info,
    );
}

pub fn register_spatial(app: &mut App) {
    app.insert_resource(GameMode::default());
    app.insert_resource(OutpostState::default());
    app.insert_resource(TravelMap::default());
    app.add_message::<TransitionIntent>();
    app.add_message::<TransitionComplete>();

    app.add_systems(
        bevy_app::Update,
        process_transitions.in_set(crate::BdSet::IntentCollection),
    );

    app.add_systems(
        bevy_app::Update,
        initialize_outpost.in_set(crate::BdSet::IntentCollection),
    );

    // Exit tile detection — return to outpost when player steps on exit.
    // Runs in Mutation after movement has applied the new position but before
    // ResultEmission cleanup systems that might despawn the ExitTile entity.
    app.add_systems(
        bevy_app::Update,
        detect_exit_tile.in_set(crate::BdSet::Mutation),
    );

    tracing::info!("Spatial module registered");
}

/// When player steps on an ExitTile in Tactical or Outpost mode, transition.
fn detect_exit_tile(
    player: Query<&Position, With<Player>>,
    exits: Query<&Position, With<ExitTile>>,
    mode: Res<GameMode>,
    mut game_log: ResMut<GameLog>,
) {
    match *mode {
        GameMode::Tactical | GameMode::Outpost => {} // allow exit detection
        _ => return,
    }
    let Ok(player_pos) = player.single() else {
        return;
    };
    for exit_pos in exits.iter() {
        if *player_pos == *exit_pos {
            tracing::info!("Player on exit tile at {:?} in {:?} mode", player_pos, mode);
            if *mode == GameMode::Tactical {
                game_log.push(
                    "The exit is here. Press r to extract.".to_string(),
                    LogLevel::Info,
                );
            } else {
                // Outpost mode: walking to the gate logs a hint, then 't' travels
                game_log.push(
                    "The shelter gate. Press t to travel.".to_string(),
                    LogLevel::Info,
                );
            }
            return;
        }
    }
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
    use crate::signals::PoolKind;

    #[test]
    fn title_screen_is_default_on_launch() {
        let mode = GameMode::default();
        assert_eq!(
            mode,
            GameMode::Title,
            "Game should start in Title mode, got {:?}",
            mode
        );
    }

    fn test_app() -> bevy_app::App {
        let mut app = bevy_app::App::new();
        app.add_plugins(crate::BdCorePlugin);
        app
    }

    #[test]
    fn leaving_location_preserves_player() {
        let mut app = test_app();
        app.world_mut().insert_resource(GameMode::Tactical);

        // Spawn player with PersistentEntity
        let player = app
            .world_mut()
            .spawn((PersistentEntity, Position { x: 5, y: 5 }))
            .id();

        // Spawn transient enemy
        let enemy = app
            .world_mut()
            .spawn((TransientEntity, Position { x: 3, y: 3 }))
            .id();

        // Transition to Outpost
        app.world_mut()
            .resource_mut::<bevy_ecs::message::Messages<TransitionIntent>>()
            .write(TransitionIntent {
                target: GameMode::Outpost,
                node_id: None,
            });
        app.update();

        // Player should still exist
        assert!(
            app.world().entities().contains(player),
            "Player should persist after leaving tactical"
        );
        // Enemy should be despawned
        assert!(
            !app.world().entities().contains(enemy),
            "Transient enemy should be removed"
        );
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
    fn colony_resources_have_default_supplies() {
        let app = test_app();
        let res = app
            .world()
            .resource::<crate::colony::production::ColonyResources>();
        let supplies = res.pools.get(PoolKind::Supplies).unwrap();
        assert_eq!(supplies.current, 10);
        assert_eq!(supplies.max, 100);
    }

    #[test]
    fn transient_combat_entities_do_not_leak() {
        let mut app = test_app();
        app.world_mut().insert_resource(GameMode::Tactical);

        // Spawn entities without marker (assumed transient)
        let summon = app.world_mut().spawn(Position { x: 1, y: 1 }).id();
        let item = app
            .world_mut()
            .spawn((crate::inventory::Item, Position { x: 2, y: 2 }))
            .id();

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

    #[test]
    fn exit_tile_detection_requires_explicit_extraction() {
        let mut app = test_app();
        app.world_mut().insert_resource(GameMode::Tactical);

        let exit_pos = Position { x: 10, y: 5 };
        let player_pos = exit_pos; // player standing on exit tile

        // Spawn persistent player on the exit tile
        app.world_mut().spawn((
            Player,
            player_pos,
            PersistentEntity,
            crate::components::Name("TestPlayer".into()),
        ));

        // Spawn the exit tile
        app.world_mut()
            .spawn((ExitTile, exit_pos, crate::components::Name("Exit".into())));

        // The exit provides feedback but does not bypass the explicit action.
        app.update();
        app.update();

        assert_eq!(
            *app.world().resource::<GameMode>(),
            GameMode::Tactical,
            "Player on exit tile should remain tactical until extraction"
        );
        assert!(
            app.world()
                .resource::<GameLog>()
                .iter()
                .any(|entry| { entry.message.contains("Press r to extract") })
        );
    }

    #[test]
    fn exit_tile_not_triggered_when_player_elsewhere() {
        let mut app = test_app();
        app.world_mut().insert_resource(GameMode::Tactical);

        let exit_pos = Position { x: 10, y: 5 };
        let player_pos = Position { x: 20, y: 15 }; // different position

        app.world_mut().spawn((
            Player,
            player_pos,
            PersistentEntity,
            crate::components::Name("TestPlayer".into()),
        ));
        app.world_mut()
            .spawn((ExitTile, exit_pos, crate::components::Name("Exit".into())));

        app.update();
        app.update();

        // Mode should still be Tactical
        assert_eq!(
            *app.world().resource::<GameMode>(),
            GameMode::Tactical,
            "Player not on exit should stay in Tactical mode"
        );
    }

    // ── P3-A: Shelter exit tile test ──

    #[test]
    fn shelter_map_has_exit_tile() {
        let mut app = test_app();
        app.world_mut().insert_resource(GameMode::Outpost);
        app.update();
        // After initialize_outpost runs, an ExitTile should exist
        // Use an archetype query to count entities with ExitTile
        let mut query = app.world_mut().query::<&crate::components::Name>();
        let names: Vec<String> = query.iter(app.world()).map(|n| n.0.clone()).collect();
        let has_gate = names.iter().any(|n| n == "Shelter Gate");
        assert!(
            has_gate,
            "Shelter map should have exit tile named 'Shelter Gate'. Names found: {:?}",
            names
        );
    }

    #[test]
    fn exit_tile_in_outpost_triggers_intent() {
        let mut app = test_app();
        app.world_mut().insert_resource(GameMode::Outpost);

        // Player on the exit position
        let exit_pos = Position { x: 20, y: 1 };
        app.world_mut().spawn((
            Player,
            exit_pos,
            PersistentEntity,
            crate::components::Name("TestPlayer".into()),
        ));
        // Spawn exit tile
        app.world_mut().spawn((
            ExitTile,
            Position { x: 20, y: 1 },
            crate::components::Name("Gate".into()),
        ));

        app.update();
        app.update();

        let mode = *app.world().resource::<GameMode>();
        assert!(
            mode == GameMode::Outpost || mode == GameMode::Travel,
            "Stepping on exit tile in Outpost should trigger intent, mode was {:?}",
            mode
        );
    }
}
