//! bd_test_support — Shared test utilities for the BD Kernel.
//!
//! Provides deterministic RNG, minimal app builders, and snapshot helpers.

pub mod contract_registry;

use bd_core::content::FoundationContent;
use bd_core::{
    BdSet,
    colony::{
        production::ColonyStorage,
        stations::{Station, StationType},
        survivors::{Survivor, SurvivorTask, WorkerActivity},
    },
    components::{Name, Player, Position, ResourceNode, ResourceNodeType, Tile},
    direction::Direction,
    gamelog::GameLog,
    inventory::Item,
    map::SmokeMap,
    pathfinding::{AStarPathfinder, Pathfinder},
    pools::Pools,
    progression::{ActionResolved, SkillProgression},
    relationships::{ContainedIn, FactionMember},
    save::{RunSnapshot, SaveError},
    session::{RunOutcome, RunSession},
    signals::{ActionDenied, ActionIntent, PoolKind},
    spatial::{EntityScope, GameMode, OutpostState, TransitionComplete, TransitionIntent},
    trace::SignalTrace,
};
use bevy_app::{App, Plugin, Update};
use bevy_ecs::{
    entity::Entity,
    error::{BevyError, DefaultErrorHandler, ErrorContext},
    message::{MessageReader, Messages},
    prelude::{IntoScheduleConfigs, ResMut, Resource},
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::{
    collections::{HashMap, HashSet},
    fmt,
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicU64, Ordering},
    },
};

static COMMAND_ERRORS: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

fn record_command_error(error: BevyError, context: ErrorContext) {
    COMMAND_ERRORS
        .get_or_init(|| Mutex::new(Vec::new()))
        .lock()
        .expect("command-error recorder mutex must not be poisoned")
        .push(format!("{context}: {error}"));
}

/// Create a deterministic RNG from a fixed seed for reproducible tests.
pub fn seeded_rng(seed: u64) -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(seed)
}

/// Build a minimal Bevy app with just the core plugin for unit testing.
pub fn minimal_app() -> App {
    foundation_app()
}

/// Build the foundation-only app used by MVP tests.
///
/// This app intentionally excludes terminal rendering and all deferred game
/// systems so simulation tests remain deterministic and headless-safe.
pub fn foundation_app() -> App {
    let mut app = App::new();
    app.add_plugins(bd_core::BdFoundationPlugin);
    let content = foundation_content();
    app.insert_resource(bd_core::colony::stations::StationCatalog::new(
        content.stations.clone(),
    ));
    app.insert_resource(content);
    app
}

/// Load the same foundation bundle used by the application.
pub fn foundation_content() -> FoundationContent {
    let content_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("content");
    bd_data::loader::load_foundation_content(&content_dir)
        .expect("foundation content must validate for headless tests")
}

/// Read-only, entity-ID-independent state used by Foundation assertions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoundationSummary {
    pub mode: GameMode,
    pub session_phase: GameMode,
    pub outcome: RunOutcome,
    pub last_completed_outcome: RunOutcome,
    pub dungeon_id: Option<String>,
    pub day: u64,
    pub turn: u64,
    pub map_size: (i32, i32),
    pub player_position: Option<Position>,
    pub player_health: Option<i32>,
    pub survivors: usize,
    pub assigned_survivors: usize,
    pub stations: usize,
    pub resource_nodes: usize,
    pub hostiles: usize,
    pub loose_items: usize,
    pub carried_items: usize,
    pub storage_items: u32,
    pub extracted_loot: u32,
    pub melee_skill: i32,
    pub medicine_skill: i32,
    pub replay_intents: Vec<bd_core::session::ActionReplayRecord>,
    pub trace_events: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoolFingerprint {
    pub kind: PoolKind,
    pub current: i32,
    pub min: i32,
    pub max: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerFingerprint {
    pub position: Option<Position>,
    pub pools: Vec<PoolFingerprint>,
    pub inventory: Vec<String>,
    pub skills: (i32, i32, i32, i32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurvivorFingerprint {
    pub name: String,
    pub position: Position,
    pub task: String,
    pub activity: String,
    pub logistics: Option<String>,
    pub cargo: Option<(Option<String>, u32)>,
    pub pools: Vec<PoolFingerprint>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StationFingerprint {
    pub content_id: String,
    pub station_type: StationType,
    pub position: Position,
    pub staffed_by: Vec<String>,
    pub effect: String,
    pub construction: Option<(u32, u32)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceNodeFingerprint {
    pub source_id: String,
    pub kind: ResourceNodeType,
    pub position: Position,
    pub depleted: bool,
}

/// Stable, entity-ID-independent state for durable Foundation comparisons.
///
/// Transient TUI interactions, pending actions, logs, and raw Bevy entity
/// identifiers are intentionally excluded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoundationFingerprint {
    pub mode: GameMode,
    pub session_phase: GameMode,
    pub outcome: RunOutcome,
    pub last_completed_outcome: RunOutcome,
    pub dungeon_id: Option<String>,
    pub day: u64,
    pub turn: u64,
    pub map_size: (i32, i32),
    pub player: Option<PlayerFingerprint>,
    pub survivors: Vec<SurvivorFingerprint>,
    pub stations: Vec<StationFingerprint>,
    pub resource_nodes: Vec<ResourceNodeFingerprint>,
    pub colony_pools: Vec<PoolFingerprint>,
    pub colony_storage: Vec<(String, u32)>,
    pub extracted_loot: u32,
}

/// Production save data captured by the Foundation driver.
#[derive(Debug, Clone)]
pub struct FoundationCheckpoint {
    snapshot: RunSnapshot,
}

/// A scenario failure that identifies the unsupported canonical step.
#[derive(Debug)]
pub struct ScenarioError {
    step: String,
    detail: String,
}

impl ScenarioError {
    fn new(step: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            step: step.into(),
            detail: detail.into(),
        }
    }
}

impl fmt::Display for ScenarioError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.step, self.detail)
    }
}

impl std::error::Error for ScenarioError {}

impl From<SaveError> for ScenarioError {
    fn from(error: SaveError) -> Self {
        Self::new("persistence", error.to_string())
    }
}

#[derive(Resource, Default)]
struct ScenarioObservations {
    resolved: Vec<ActionResolved>,
    denied: Vec<ActionDenied>,
    transitions: Vec<TransitionComplete>,
    days: Vec<bd_core::time::DayAdvanced>,
    daily_summaries: Vec<bd_core::colony::production::DailySummary>,
    defeats: Vec<bd_core::signals::EntityDefeated>,
}

fn observe_scenario_results(
    mut resolved: MessageReader<ActionResolved>,
    mut denied: MessageReader<ActionDenied>,
    mut transitions: MessageReader<TransitionComplete>,
    mut days: MessageReader<bd_core::time::DayAdvanced>,
    mut daily_summaries: MessageReader<bd_core::colony::production::DailySummary>,
    mut defeats: MessageReader<bd_core::signals::EntityDefeated>,
    mut observations: ResMut<ScenarioObservations>,
) {
    observations.resolved.extend(resolved.read().cloned());
    observations.denied.extend(denied.read().cloned());
    observations.transitions.extend(transitions.read().cloned());
    observations.days.extend(days.read().copied());
    observations
        .daily_summaries
        .extend(daily_summaries.read().cloned());
    observations.defeats.extend(defeats.read().cloned());
}

fn pool_fingerprint(pools: &Pools) -> Vec<PoolFingerprint> {
    let mut result = pools
        .iter()
        .map(|pool| PoolFingerprint {
            kind: pool.kind,
            current: pool.current,
            min: pool.min,
            max: pool.max,
        })
        .collect::<Vec<_>>();
    result.sort_by_key(|pool| format!("{:?}", pool.kind));
    result
}

fn activity_fingerprint(activity: Option<&WorkerActivity>) -> String {
    match activity {
        Some(WorkerActivity::Idle) => "Idle".into(),
        Some(WorkerActivity::EnRoute {
            target,
            target_position,
            distance,
        }) => {
            format!(
                "EnRoute:{target}@{},{}:{distance}",
                target_position.x, target_position.y
            )
        }
        Some(WorkerActivity::Working {
            target,
            target_position,
        }) => {
            format!(
                "Working:{target}@{},{}",
                target_position.x, target_position.y
            )
        }
        Some(WorkerActivity::Blocked {
            target,
            target_position,
            reason,
        }) => {
            format!("Blocked:{target}:{target_position:?}:{reason:?}")
        }
        Some(WorkerActivity::Resting) => "Resting".into(),
        Some(WorkerActivity::Defending) => "Defending".into(),
        None => "Unresolved".into(),
    }
}

/// Headless production-path driver for the canonical Foundation scenario.
///
/// The driver may submit production intents, advance schedules, invoke the
/// production persistence boundary, and read summaries. It deliberately does
/// not expose mutable world access or add alternate gameplay resolvers.
pub struct FoundationDriver {
    app: App,
}

impl FoundationDriver {
    pub fn new(seed: u64) -> Self {
        Self::from_app(seed, foundation_app())
    }

    /// Build the production Foundation schedule with one additional runtime
    /// plugin. This lets acceptance tests reproduce integration-only ordering
    /// without exposing the ECS world for mutation.
    pub fn new_with_plugin(seed: u64, plugin: impl Plugin) -> Self {
        let mut app = foundation_app();
        app.add_plugins(plugin);
        Self::from_app(seed, app)
    }

    fn from_app(seed: u64, mut app: App) -> Self {
        app.insert_resource(RunSession::new(seed));
        app.init_resource::<ScenarioObservations>();
        app.add_systems(
            Update,
            observe_scenario_results.in_set(BdSet::ViewModelBuild),
        );
        Self { app }
    }

    pub fn fingerprint(&mut self) -> FoundationFingerprint {
        let entity_ids = self.entity_ids();
        let world = self.app.world();
        let session = world.resource::<RunSession>();
        let last_completed = world.resource::<bd_core::session::LastCompletedRun>();
        let mode = *world.resource::<GameMode>();
        let map = world.resource::<SmokeMap>();
        let station_catalog = world.resource::<bd_core::colony::stations::StationCatalog>();

        let mut station_keys = HashMap::new();
        for entity in &entity_ids {
            let Some(station_type) = world.get::<StationType>(*entity).copied() else {
                continue;
            };
            let Some(position) = world.get::<Position>(*entity).copied() else {
                continue;
            };
            let content_id = station_catalog
                .get(station_type)
                .map_or_else(|| format!("{station_type:?}"), |entry| entry.id.clone());
            station_keys.insert(
                entity.to_bits(),
                format!("{content_id}@{},{}", position.x, position.y),
            );
        }

        let mut survivor_names = HashMap::new();
        for entity in &entity_ids {
            if world.entity(*entity).contains::<Survivor>() {
                let name = world
                    .get::<Name>(*entity)
                    .map_or_else(|| "Unnamed survivor".into(), |name| name.0.clone());
                survivor_names.insert(entity.to_bits(), name);
            }
        }

        let player_entity = entity_ids
            .iter()
            .copied()
            .find(|entity| world.entity(*entity).contains::<Player>());
        let player = player_entity.map(|entity| {
            let mut inventory = entity_ids
                .iter()
                .filter_map(|item| {
                    let entity_ref = world.entity(*item);
                    if !entity_ref.contains::<Item>()
                        || !entity_ref
                            .get::<ContainedIn>()
                            .is_some_and(|container| container.0 == entity)
                    {
                        return None;
                    }
                    Some(
                        entity_ref
                            .get::<Name>()
                            .map_or_else(|| "Unnamed item".into(), |name| name.0.clone()),
                    )
                })
                .collect::<Vec<_>>();
            inventory.sort();
            let skills = world
                .get::<SkillProgression>(entity)
                .map_or((0, 0, 0, 0), |skills| {
                    (skills.melee, skills.ranged, skills.repair, skills.medicine)
                });
            PlayerFingerprint {
                position: world.get::<Position>(entity).copied(),
                pools: world
                    .get::<Pools>(entity)
                    .map_or_else(Vec::new, pool_fingerprint),
                inventory,
                skills,
            }
        });

        let mut survivors = entity_ids
            .iter()
            .filter_map(|entity| {
                let entity_ref = world.entity(*entity);
                if !entity_ref.contains::<Survivor>() {
                    return None;
                }
                let name = survivor_names
                    .get(&entity.to_bits())
                    .cloned()
                    .unwrap_or_else(|| "Unnamed survivor".into());
                let position = entity_ref.get::<Position>().copied()?;
                let task = match entity_ref.get::<SurvivorTask>() {
                    Some(SurvivorTask::Idle) | None => "Idle".into(),
                    Some(SurvivorTask::Gathering(kind)) => format!("Gathering:{kind:?}"),
                    Some(SurvivorTask::Defending) => "Defending".into(),
                    Some(SurvivorTask::Resting) => "Resting".into(),
                    Some(SurvivorTask::AssignedTo(bits)) => {
                        let target = station_keys
                            .get(bits)
                            .cloned()
                            .unwrap_or_else(|| "MissingStation".into());
                        format!("AssignedTo:{target}")
                    }
                };
                Some(SurvivorFingerprint {
                    name,
                    position,
                    task,
                    activity: activity_fingerprint(entity_ref.get::<WorkerActivity>()),
                    logistics: entity_ref
                        .get::<bd_core::colony::logistics::LogisticsJob>()
                        .map(|job| {
                            format!(
                                "{}:{:?}:{}:{:?}",
                                job.recipe_id, job.stage, job.work_completed, job.blocked
                            )
                        }),
                    cargo: entity_ref
                        .get::<bd_core::colony::logistics::Cargo>()
                        .map(|cargo| (cargo.resource_id.clone(), cargo.amount)),
                    pools: entity_ref
                        .get::<Pools>()
                        .map_or_else(Vec::new, pool_fingerprint),
                })
            })
            .collect::<Vec<_>>();
        survivors.sort_by(|left, right| left.name.cmp(&right.name));

        let mut stations = entity_ids
            .iter()
            .filter_map(|entity| {
                let entity_ref = world.entity(*entity);
                if !entity_ref.contains::<Station>() {
                    return None;
                }
                let station_type = *entity_ref.get::<StationType>()?;
                let position = *entity_ref.get::<Position>()?;
                let blueprint = station_catalog.get(station_type);
                let content_id =
                    blueprint.map_or_else(|| format!("{station_type:?}"), |entry| entry.id.clone());
                let mut staffed_by = entity_ids
                    .iter()
                    .filter_map(|survivor| {
                        let task = world.get::<SurvivorTask>(*survivor)?;
                        matches!(task, SurvivorTask::AssignedTo(bits) if *bits == entity.to_bits())
                            .then(|| survivor_names.get(&survivor.to_bits()).cloned())
                            .flatten()
                    })
                    .collect::<Vec<_>>();
                staffed_by.sort();
                Some(StationFingerprint {
                    content_id,
                    station_type,
                    position,
                    staffed_by,
                    effect: blueprint.map_or_else(
                        || "MissingCatalogEntry".into(),
                        |entry| entry.effect_label(),
                    ),
                    construction: entity_ref
                        .get::<bd_core::colony::stations::ConstructionSite>()
                        .map(|site| (site.work_completed, site.work_required)),
                })
            })
            .collect::<Vec<_>>();
        stations.sort_by(|left, right| {
            (&left.content_id, left.position.y, left.position.x).cmp(&(
                &right.content_id,
                right.position.y,
                right.position.x,
            ))
        });

        let mut resource_nodes = entity_ids
            .iter()
            .filter_map(|entity| {
                let entity_ref = world.entity(*entity);
                let node = entity_ref.get::<ResourceNode>()?;
                Some(ResourceNodeFingerprint {
                    source_id: node.source_id.clone(),
                    kind: node.kind,
                    position: *entity_ref.get::<Position>()?,
                    depleted: node.depleted,
                })
            })
            .collect::<Vec<_>>();
        resource_nodes.sort_by(|left, right| {
            (&left.source_id, left.position.y, left.position.x).cmp(&(
                &right.source_id,
                right.position.y,
                right.position.x,
            ))
        });

        let colony_pools = world
            .resource::<bd_core::colony::production::ColonyResources>()
            .pools
            .iter()
            .map(|pool| PoolFingerprint {
                kind: pool.kind,
                current: pool.current,
                min: pool.min,
                max: pool.max,
            })
            .collect::<Vec<_>>();
        let mut colony_pools = colony_pools;
        colony_pools.sort_by_key(|pool| format!("{:?}", pool.kind));
        let colony_storage = world
            .resource::<ColonyStorage>()
            .items
            .iter()
            .map(|(id, count)| (id.clone(), *count))
            .collect();

        FoundationFingerprint {
            mode,
            session_phase: session.phase,
            outcome: session.outcome,
            last_completed_outcome: last_completed.outcome,
            dungeon_id: session.dungeon_id.clone(),
            day: session.day,
            turn: session.turn,
            map_size: (map.width, map.height),
            player,
            survivors,
            stations,
            resource_nodes,
            colony_pools,
            colony_storage,
            extracted_loot: session.extracted_loot,
        }
    }

    pub fn from_checkpoint(checkpoint: &FoundationCheckpoint) -> Result<Self, ScenarioError> {
        let mut driver = Self::new(checkpoint.snapshot.seed);
        driver.restore_checkpoint(checkpoint)?;
        Ok(driver)
    }

    pub fn start_colony(&mut self) -> Result<(), ScenarioError> {
        self.request_transition("clean launch → colony", GameMode::Outpost, None)
    }

    pub fn enter_dungeon(&mut self, dungeon_id: &str) -> Result<(), ScenarioError> {
        if dungeon_id != bd_core::spatial::FOUNDATION_DUNGEON_ID {
            return Err(ScenarioError::new(
                "colony → fixed dungeon",
                format!("unsupported Foundation dungeon `{dungeon_id}`"),
            ));
        }
        let player = self
            .player()
            .ok_or_else(|| ScenarioError::new("colony → fixed dungeon", "player unavailable"))?;
        self.expect_action(
            "colony → fixed dungeon",
            player,
            "ability.enter_foundation_dungeon",
            None,
            None,
        )?;
        if self.summary().mode != GameMode::Tactical {
            return Err(ScenarioError::new(
                "colony → fixed dungeon",
                "accepted entry action did not reach Tactical mode",
            ));
        }
        Ok(())
    }

    pub fn return_to_colony(&mut self, step: &str) -> Result<(), ScenarioError> {
        self.request_transition(step, GameMode::Outpost, None)
    }

    pub fn request_transition(
        &mut self,
        step: &str,
        target: GameMode,
        node_id: Option<&str>,
    ) -> Result<(), ScenarioError> {
        self.clear_observations();
        self.app
            .world_mut()
            .resource_mut::<Messages<TransitionIntent>>()
            .write(TransitionIntent {
                target,
                node_id: node_id.map(str::to_owned),
            });
        self.app.update();

        let actual = *self.app.world().resource::<GameMode>();
        if actual != target {
            return Err(ScenarioError::new(
                step,
                format!("transition rejected; expected {target:?}, found {actual:?}"),
            ));
        }
        Ok(())
    }

    pub fn expect_action(
        &mut self,
        step: &str,
        actor: Entity,
        action_id: &str,
        direction: Option<Direction>,
        target: Option<Entity>,
    ) -> Result<(), ScenarioError> {
        self.clear_observations();
        self.app
            .world_mut()
            .resource_mut::<Messages<ActionIntent>>()
            .write(ActionIntent {
                actor,
                action_id: action_id.to_owned(),
                direction,
                target,
            });
        self.app.update();

        let observation = self.app.world().resource::<ScenarioObservations>();
        if let Some(denial) = observation
            .denied
            .iter()
            .find(|denial| denial.actor == actor && denial.action_id == action_id)
        {
            return Err(ScenarioError::new(
                step,
                format!("action {action_id} denied: {:?}", denial.reason),
            ));
        }
        if !observation
            .resolved
            .iter()
            .any(|result| result.actor == actor && result.action_id == action_id)
        {
            return Err(ScenarioError::new(
                step,
                format!("action {action_id} produced no typed result"),
            ));
        }

        // Resolve the one permitted enemy phase and any next-frame result
        // messages before returning a stable state summary.
        self.app.update();
        Ok(())
    }

    pub fn expect_denied_action(
        &mut self,
        step: &str,
        actor: Entity,
        action_id: &str,
        direction: Option<Direction>,
        target: Option<Entity>,
    ) -> Result<bd_core::signals::DenialReason, ScenarioError> {
        self.clear_observations();
        self.app
            .world_mut()
            .resource_mut::<Messages<ActionIntent>>()
            .write(ActionIntent {
                actor,
                action_id: action_id.to_owned(),
                direction,
                target,
            });
        self.app.update();

        self.app
            .world()
            .resource::<ScenarioObservations>()
            .denied
            .iter()
            .find(|denial| denial.actor == actor && denial.action_id == action_id)
            .map(|denial| denial.reason.clone())
            .ok_or_else(|| ScenarioError::new(step, "action did not emit a typed denial"))
    }

    /// Submit a production action intent without requiring a gameplay result.
    ///
    /// This is used only to verify that already-buffered input targeting an
    /// entity defeated earlier in the schedule is rejected safely.
    pub fn submit_buffered_action(
        &mut self,
        actor: Entity,
        action_id: &str,
        direction: Option<Direction>,
        target: Option<Entity>,
    ) {
        self.app
            .world_mut()
            .resource_mut::<Messages<ActionIntent>>()
            .write(ActionIntent {
                actor,
                action_id: action_id.to_owned(),
                direction,
                target,
            });
        self.app.update();
    }

    pub fn expect_station_assignment_action(
        &mut self,
        step: &str,
        player: Entity,
        survivor: Entity,
        station: Entity,
    ) -> Result<(), ScenarioError> {
        self.expect_action(step, player, "ability.assign_station", None, Some(survivor))?;
        match self.app.world().get::<SurvivorTask>(survivor) {
            Some(SurvivorTask::AssignedTo(station_bits)) if *station_bits == station.to_bits() => {
                Ok(())
            }
            task => Err(ScenarioError::new(
                step,
                format!("action did not assign survivor to station; task={task:?}"),
            )),
        }
    }

    pub fn approach_and_attack_first_hostile(&mut self, step: &str) -> Result<(), ScenarioError> {
        let hostile = self
            .first_hostile()
            .ok_or_else(|| ScenarioError::new(step, "no hostile exists"))?;
        self.approach_hostile(step, hostile)?;
        let player = self
            .player()
            .ok_or_else(|| ScenarioError::new(step, "player was defeated before attacking"))?;
        self.expect_action(step, player, "ability.quick_attack", None, Some(hostile))
    }

    pub fn approach_and_defeat_first_hostile(&mut self, step: &str) -> Result<(), ScenarioError> {
        let hostile = self
            .first_hostile()
            .ok_or_else(|| ScenarioError::new(step, "no hostile exists"))?;
        self.approach_and_defeat(step, hostile)
    }

    pub fn defeat_all_hostiles(&mut self, step: &str) -> Result<(), ScenarioError> {
        for _ in 0..16 {
            let Some(hostile) = self.first_hostile() else {
                return Ok(());
            };
            self.approach_and_defeat(step, hostile)?;
        }
        Err(ScenarioError::new(
            step,
            "hostiles remained after 16 canonical encounters",
        ))
    }

    pub fn approach_and_defeat(
        &mut self,
        step: &str,
        hostile: Entity,
    ) -> Result<(), ScenarioError> {
        for _ in 0..16 {
            if !self.app.world().entities().contains(hostile) {
                return Ok(());
            }
            self.approach_hostile(step, hostile)?;
            let player = self
                .player()
                .ok_or_else(|| ScenarioError::new(step, "player was defeated"))?;
            self.expect_action(step, player, "ability.quick_attack", None, Some(hostile))?;
        }
        Err(ScenarioError::new(
            step,
            "hostile remained after 16 canonical combat actions",
        ))
    }

    fn approach_hostile(&mut self, step: &str, hostile: Entity) -> Result<(), ScenarioError> {
        for _ in 0..32 {
            let player = self
                .player()
                .ok_or_else(|| ScenarioError::new(step, "player was defeated"))?;
            let player_pos = self
                .position(player)
                .ok_or_else(|| ScenarioError::new(step, "player has no position"))?;
            let hostile_pos = self
                .position(hostile)
                .ok_or_else(|| ScenarioError::new(step, "hostile has no position"))?;
            let distance =
                (player_pos.x - hostile_pos.x).abs() + (player_pos.y - hostile_pos.y).abs();
            if distance <= 1 {
                return Ok(());
            }
            let map = self.app.world().resource::<SmokeMap>();
            let candidates = [
                Position {
                    x: hostile_pos.x + 1,
                    y: hostile_pos.y,
                },
                Position {
                    x: hostile_pos.x - 1,
                    y: hostile_pos.y,
                },
                Position {
                    x: hostile_pos.x,
                    y: hostile_pos.y + 1,
                },
                Position {
                    x: hostile_pos.x,
                    y: hostile_pos.y - 1,
                },
            ];
            let next = candidates
                .into_iter()
                .filter(|candidate| map.is_walkable(candidate.x, candidate.y))
                .filter_map(|candidate| {
                    AStarPathfinder
                        .find_path(map, player_pos, candidate, &HashSet::new())
                        .map(|path| (path.len(), path))
                })
                .min_by_key(|(length, _)| *length)
                .and_then(|(_, path)| path.get(1).copied())
                .ok_or_else(|| ScenarioError::new(step, "no route to hostile"))?;
            let direction = direction_between(player_pos, next)
                .ok_or_else(|| ScenarioError::new(step, "hostile route was non-cardinal"))?;
            self.expect_action(step, player, "ability.move", Some(direction), None)?;
        }
        Err(ScenarioError::new(
            step,
            "hostile was not reached within 32 movement actions",
        ))
    }

    pub fn approach_and_pick_up(&mut self, step: &str) -> Result<(), ScenarioError> {
        let item = self
            .first_loose_item()
            .ok_or_else(|| ScenarioError::new(step, "no loose item exists"))?;
        let item_position = self
            .position(item)
            .ok_or_else(|| ScenarioError::new(step, "item has no position"))?;
        self.move_player_to(step, item_position)?;
        let player = self
            .player()
            .ok_or_else(|| ScenarioError::new(step, "player is unavailable"))?;
        self.expect_action(step, player, "ability.pickup", None, Some(item))
    }

    pub fn move_player_to_exit(&mut self, step: &str) -> Result<(), ScenarioError> {
        let exit = self
            .exit_position()
            .ok_or_else(|| ScenarioError::new(step, "dungeon has no exit"))?;
        self.move_player_to(step, exit)
    }

    pub fn move_player_to(
        &mut self,
        step: &str,
        destination: Position,
    ) -> Result<(), ScenarioError> {
        let player = self
            .player()
            .ok_or_else(|| ScenarioError::new(step, "player is unavailable"))?;
        let start = self
            .position(player)
            .ok_or_else(|| ScenarioError::new(step, "player has no position"))?;
        let map = self.app.world().resource::<SmokeMap>();
        let path = AStarPathfinder
            .find_path(map, start, destination, &HashSet::new())
            .ok_or_else(|| {
                ScenarioError::new(
                    step,
                    format!(
                        "destination is unreachable; start={start:?}, destination={destination:?}, start_tile={:?}, destination_tile={:?}",
                        map.get(start.x, start.y),
                        map.get(destination.x, destination.y)
                    ),
                )
            })?;

        for positions in path.windows(2) {
            let direction = direction_between(positions[0], positions[1])
                .ok_or_else(|| ScenarioError::new(step, "path contains a non-cardinal movement"))?;
            self.expect_action(step, player, "ability.move", Some(direction), None)?;
        }
        let actual = self
            .position(player)
            .ok_or_else(|| ScenarioError::new(step, "player disappeared during movement"))?;
        if actual != destination {
            return Err(ScenarioError::new(
                step,
                format!("expected player at {destination:?}, found {actual:?}"),
            ));
        }
        Ok(())
    }

    pub fn extract(&mut self, step: &str) -> Result<(), ScenarioError> {
        let player = self
            .player()
            .ok_or_else(|| ScenarioError::new(step, "player is unavailable"))?;
        self.expect_action(step, player, "ability.extract", None, None)?;
        self.app.update();
        if self.summary().mode != GameMode::Outpost {
            return Err(ScenarioError::new(
                step,
                "accepted extraction did not return to the colony",
            ));
        }
        Ok(())
    }

    pub fn wait_for_player_defeat(&mut self, step: &str) -> Result<(), ScenarioError> {
        for _ in 0..24 {
            if self.summary().mode == GameMode::GameOver {
                return Ok(());
            }
            let player = self.player().ok_or_else(|| {
                ScenarioError::new(step, "player disappeared before defeat result")
            })?;
            self.expect_action(step, player, "ability.wait", None, None)?;
        }
        Err(ScenarioError::new(
            step,
            "player was not defeated within 24 normal combat turns",
        ))
    }

    pub fn checkpoint(&mut self) -> Result<FoundationCheckpoint, ScenarioError> {
        static CHECKPOINT_COUNTER: AtomicU64 = AtomicU64::new(0);
        let sequence = CHECKPOINT_COUNTER.fetch_add(1, Ordering::Relaxed);
        let save_dir = std::env::temp_dir().join(format!(
            "bd-foundation-scenario-{}-{sequence}",
            std::process::id()
        ));
        let session = self.app.world().resource::<RunSession>().clone();
        let path =
            bd_core::save::save_world(self.app.world_mut(), session.seed, session.turn, &save_dir)?;
        let snapshot = bd_core::save::load_snapshot(&path)?;
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_dir(save_dir);
        Ok(FoundationCheckpoint { snapshot })
    }

    pub fn restore_checkpoint(
        &mut self,
        checkpoint: &FoundationCheckpoint,
    ) -> Result<(), ScenarioError> {
        bd_core::save::restore_snapshot_into(
            self.app.world_mut(),
            &checkpoint.snapshot,
            &HashMap::new(),
        )?;
        Ok(())
    }

    pub fn checkpoint_with_missing_relationship(&mut self) -> Option<FoundationCheckpoint> {
        let mut checkpoint = self.checkpoint().ok()?;
        let missing = checkpoint
            .snapshot
            .entities
            .iter()
            .find_map(|entity| entity.contains.first().copied())?;
        checkpoint
            .snapshot
            .entities
            .retain(|entity| entity.save_id != missing);
        Some(checkpoint)
    }

    pub fn save_manual_slot(
        &mut self,
        save_dir: &std::path::Path,
    ) -> Result<std::path::PathBuf, ScenarioError> {
        Ok(bd_core::save::save_manual_slot(
            self.app.world_mut(),
            save_dir,
        )?)
    }

    pub fn load_manual_slot(&mut self, save_dir: &std::path::Path) -> Result<(), ScenarioError> {
        let snapshot = bd_core::save::load_manual_slot(save_dir)?;
        bd_core::save::restore_snapshot_into(self.app.world_mut(), &snapshot, &HashMap::new())?;
        Ok(())
    }

    pub fn outpost_party_references_are_valid(&self) -> bool {
        self.app
            .world()
            .get_resource::<OutpostState>()
            .is_some_and(|outpost| {
                !outpost.party.is_empty()
                    && outpost.party.iter().all(|entity| {
                        self.app.world().entities().contains(*entity)
                            && self.app.world().entity(*entity).contains::<Survivor>()
                    })
            })
    }

    pub fn first_hostile_health(&mut self) -> Option<i32> {
        self.first_hostile()
            .and_then(|hostile| self.app.world().get::<Pools>(hostile))
            .and_then(|pools| pools.get(PoolKind::Health))
            .map(|health| health.current)
    }

    pub fn advance_idle(&mut self) {
        self.app.update();
    }

    /// Replace Bevy's default command error handler with a test-local recorder.
    ///
    /// This is intentionally diagnostic-only: it observes scheduler/command
    /// failures without changing gameplay state.
    pub fn install_command_error_capture(&mut self) {
        COMMAND_ERRORS
            .get_or_init(|| Mutex::new(Vec::new()))
            .lock()
            .expect("command-error recorder mutex must not be poisoned")
            .clear();
        self.app
            .world_mut()
            .insert_resource(DefaultErrorHandler(record_command_error));
    }

    pub fn command_errors(&self) -> Vec<String> {
        COMMAND_ERRORS
            .get_or_init(|| Mutex::new(Vec::new()))
            .lock()
            .expect("command-error recorder mutex must not be poisoned")
            .clone()
    }

    /// Phase 2 fixture adapter for the pre-Phase-4 assignment message.
    ///
    /// This invokes the production assignment system and is intentionally not
    /// used by the canonical acceptance scenario.
    pub fn fixture_assign_station(
        &mut self,
        survivor: Entity,
        station: Entity,
    ) -> Result<(), ScenarioError> {
        let player = self
            .player()
            .ok_or_else(|| ScenarioError::new("scope fixture assignment", "player unavailable"))?;
        self.expect_station_assignment_action(
            "scope fixture assignment",
            player,
            survivor,
            station,
        )?;
        match self.app.world().get::<SurvivorTask>(survivor) {
            Some(SurvivorTask::AssignedTo(station_bits)) if *station_bits == station.to_bits() => {
                Ok(())
            }
            task => Err(ScenarioError::new(
                "scope fixture assignment",
                format!("production assignment did not resolve; task={task:?}"),
            )),
        }
    }

    pub fn fixture_spawn_processing_station(&mut self, position: Position) -> Entity {
        self.app
            .world_mut()
            .spawn((
                bd_core::colony::stations::Station,
                StationType::Custom(1),
                position,
                Name("Basic Processing".into()),
                bd_core::components::ContentIdentity("station.basic_processor".into()),
                bd_core::components::BlocksMovement,
                EntityScope::ColonyPersistent,
                bd_core::spatial::PersistentEntity,
            ))
            .id()
    }

    pub fn fixture_assign_recipe(&mut self, survivor: Entity, recipe_id: &str) {
        self.app.world_mut().entity_mut(survivor).insert((
            bd_core::colony::logistics::LogisticsJob {
                recipe_id: recipe_id.into(),
                stage: bd_core::colony::logistics::JobStage::ToSource,
                work_completed: 0,
                blocked: None,
            },
            bd_core::colony::logistics::Cargo::default(),
        ));
    }

    pub fn fixture_set_logistics_progress(
        &mut self,
        survivor: Entity,
        stage: bd_core::colony::logistics::JobStage,
        work_completed: u32,
    ) {
        let mut job = self
            .app
            .world_mut()
            .get_mut::<bd_core::colony::logistics::LogisticsJob>(survivor)
            .expect("fixture survivor must have a logistics job");
        job.stage = stage;
        job.work_completed = work_completed;
    }

    pub fn assign_recipe(
        &mut self,
        step: &str,
        survivor: Entity,
        recipe_id: &str,
    ) -> Result<(), ScenarioError> {
        self.app
            .world_mut()
            .resource_mut::<bd_core::colony::logistics::PendingRecipeAssignment>()
            .0 = Some(recipe_id.into());
        let player = self
            .player()
            .ok_or_else(|| ScenarioError::new(step, "player is unavailable"))?;
        self.expect_action(step, player, "ability.assign_recipe", None, Some(survivor))?;
        match self.logistics_job(survivor) {
            Some(job) if job.recipe_id == recipe_id => Ok(()),
            job => Err(ScenarioError::new(
                step,
                format!("recipe assignment did not resolve; job={job:?}"),
            )),
        }
    }

    pub fn logistics_job(
        &self,
        survivor: Entity,
    ) -> Option<bd_core::colony::logistics::LogisticsJob> {
        self.app
            .world()
            .get::<bd_core::colony::logistics::LogisticsJob>(survivor)
            .cloned()
    }

    pub fn worker_cargo(&self, survivor: Entity) -> Option<bd_core::colony::logistics::Cargo> {
        self.app
            .world()
            .get::<bd_core::colony::logistics::Cargo>(survivor)
            .cloned()
    }

    pub fn raw_resource_count(&self, resource_id: &str) -> u32 {
        self.app
            .world()
            .resource::<bd_core::colony::production::ColonyResources>()
            .raw
            .get(resource_id)
            .copied()
            .unwrap_or(0)
    }

    /// Phase 2 fixture adapter for the pre-Phase-4 pickup message.
    pub fn fixture_pick_up(&mut self, item: Entity) -> Result<(), ScenarioError> {
        let position = self
            .position(item)
            .ok_or_else(|| ScenarioError::new("scope fixture pickup", "item has no position"))?;
        self.move_player_to("scope fixture pickup movement", position)?;
        let player = self
            .player()
            .ok_or_else(|| ScenarioError::new("scope fixture pickup", "player is unavailable"))?;
        self.expect_action(
            "scope fixture pickup",
            player,
            "ability.pickup",
            None,
            Some(item),
        )?;
        match self.app.world().get::<ContainedIn>(item) {
            Some(contained) if contained.0 == player => Ok(()),
            _ => Err(ScenarioError::new(
                "scope fixture pickup",
                "production pickup did not contain the item",
            )),
        }
    }

    pub fn entity_exists(&self, entity: Entity) -> bool {
        self.app.world().entities().contains(entity)
    }

    pub fn entity_scope(&self, entity: Entity) -> Option<EntityScope> {
        self.app.world().get::<EntityScope>(entity).copied()
    }

    pub fn scope_count(&mut self, scope: EntityScope) -> usize {
        let entities = self.entity_ids();
        entities
            .into_iter()
            .filter(|entity| self.entity_scope(*entity) == Some(scope))
            .count()
    }

    pub fn resource_nodes(&mut self) -> Vec<Entity> {
        let entities = self.entity_ids();
        entities
            .into_iter()
            .filter(|entity| self.app.world().entity(*entity).contains::<ResourceNode>())
            .collect()
    }

    pub fn resource_node_kinds(&mut self) -> Vec<ResourceNodeType> {
        self.resource_nodes()
            .into_iter()
            .filter_map(|entity| {
                self.app
                    .world()
                    .get::<ResourceNode>(entity)
                    .map(|node| node.kind)
            })
            .collect()
    }

    pub fn resource_node_layout(&mut self) -> Vec<(String, Position)> {
        let mut layout = self
            .resource_nodes()
            .into_iter()
            .filter_map(|entity| {
                let node = self.app.world().get::<ResourceNode>(entity)?;
                let position = self.app.world().get::<Position>(entity)?;
                Some((node.source_id.clone(), *position))
            })
            .collect::<Vec<_>>();
        layout.sort_by(|left, right| {
            (&left.0, left.1.y, left.1.x).cmp(&(&right.0, right.1.y, right.1.x))
        });
        layout
    }

    pub fn resource_nodes_with_state(&mut self) -> Vec<(Entity, ResourceNodeType, Position, bool)> {
        self.resource_nodes()
            .into_iter()
            .filter_map(|entity| {
                let node = self.app.world().get::<ResourceNode>(entity)?;
                let position = self.app.world().get::<Position>(entity)?;
                Some((entity, node.kind, *position, node.depleted))
            })
            .collect()
    }

    pub fn all_resource_nodes_reachable_from_shelter_spawn(&mut self) -> bool {
        let nodes = self.resource_nodes();
        let outpost = self.app.world().resource::<OutpostState>();
        nodes.into_iter().all(|entity| {
            self.app
                .world()
                .get::<Position>(entity)
                .is_some_and(|destination| {
                    AStarPathfinder
                        .find_path(
                            &outpost.map,
                            bd_core::colony::shelter::SHELTER_RETURN_SPAWN,
                            *destination,
                            &HashSet::new(),
                        )
                        .is_some()
                })
        })
    }

    pub fn log_messages(&self) -> Vec<String> {
        self.app
            .world()
            .resource::<GameLog>()
            .iter()
            .map(|entry| entry.message.clone())
            .collect()
    }

    pub fn player(&mut self) -> Option<Entity> {
        let entities = self.entity_ids();
        entities
            .into_iter()
            .find(|entity| self.app.world().entity(*entity).contains::<Player>())
    }

    pub fn player_count(&mut self) -> usize {
        let entities = self.entity_ids();
        entities
            .into_iter()
            .filter(|entity| self.app.world().entity(*entity).contains::<Player>())
            .count()
    }

    pub fn first_survivor(&mut self) -> Option<Entity> {
        let entities = self.entity_ids();
        entities
            .into_iter()
            .find(|entity| self.app.world().entity(*entity).contains::<Survivor>())
    }

    pub fn survivors(&mut self) -> Vec<Entity> {
        let entities = self.entity_ids();
        entities
            .into_iter()
            .filter(|entity| self.app.world().entity(*entity).contains::<Survivor>())
            .collect()
    }

    pub fn survivor_positions(&mut self) -> Vec<Position> {
        self.survivors()
            .into_iter()
            .filter_map(|entity| self.app.world().get::<Position>(entity).copied())
            .collect()
    }

    pub fn survivor_by_name(&mut self, expected_name: &str) -> Option<Entity> {
        let entities = self.entity_ids();
        entities.into_iter().find(|entity| {
            let entity = self.app.world().entity(*entity);
            entity.contains::<Survivor>()
                && entity
                    .get::<Name>()
                    .is_some_and(|name| name.0 == expected_name)
        })
    }

    pub fn survivor_task(&self, survivor: Entity) -> Option<SurvivorTask> {
        self.app.world().get::<SurvivorTask>(survivor).cloned()
    }

    pub fn first_station(&mut self) -> Option<Entity> {
        let entities = self.entity_ids();
        entities
            .into_iter()
            .find(|entity| self.app.world().entity(*entity).contains::<Station>())
    }

    pub fn station_by_type(&mut self, expected: StationType) -> Option<Entity> {
        self.stations()
            .into_iter()
            .find(|entity| self.app.world().get::<StationType>(*entity).copied() == Some(expected))
    }

    pub fn stations(&mut self) -> Vec<Entity> {
        let entities = self.entity_ids();
        entities
            .into_iter()
            .filter(|entity| self.app.world().entity(*entity).contains::<Station>())
            .collect()
    }

    pub fn station_type(&self, station: Entity) -> Option<bd_core::colony::stations::StationType> {
        self.app
            .world()
            .get::<bd_core::colony::stations::StationType>(station)
            .copied()
    }

    pub fn construction_progress(&self, station: Entity) -> Option<(u32, u32)> {
        self.app
            .world()
            .get::<bd_core::colony::stations::ConstructionSite>(station)
            .map(|site| (site.work_completed, site.work_required))
    }

    pub fn station_is_operational(&self, station: Entity) -> bool {
        self.app.world().entity(station).contains::<Station>()
            && !self
                .app
                .world()
                .entity(station)
                .contains::<bd_core::colony::stations::ConstructionSite>()
    }

    /// Fixture setup for contracts whose subject is downstream station
    /// behavior rather than construction scheduling.
    pub fn fixture_complete_construction(&mut self, station: Entity) {
        self.app
            .world_mut()
            .entity_mut(station)
            .remove::<bd_core::colony::stations::ConstructionSite>();
    }

    pub fn update_frames(&mut self, frames: usize) {
        for _ in 0..frames {
            self.app.update();
        }
    }

    pub fn station_types(&mut self) -> Vec<bd_core::colony::stations::StationType> {
        let mut station_types = self
            .stations()
            .into_iter()
            .filter_map(|entity| {
                self.app
                    .world()
                    .get::<bd_core::colony::stations::StationType>(entity)
                    .copied()
                    .map(|station_type| (entity.to_bits(), station_type))
            })
            .collect::<Vec<_>>();
        station_types.sort_by_key(|(bits, _)| *bits);
        station_types
            .into_iter()
            .map(|(_, station_type)| station_type)
            .collect()
    }

    pub fn first_hostile(&mut self) -> Option<Entity> {
        let entities = self.entity_ids();
        entities.into_iter().find(|entity| {
            let entity = self.app.world().entity(*entity);
            entity.contains::<FactionMember>()
                && entity
                    .get::<Pools>()
                    .and_then(|pools| pools.get(PoolKind::Health))
                    .is_some_and(|health| health.current > health.min)
        })
    }

    pub fn first_loose_item(&mut self) -> Option<Entity> {
        let entities = self.entity_ids();
        entities.into_iter().find(|entity| {
            let entity = self.app.world().entity(*entity);
            entity.contains::<Item>() && !entity.contains::<ContainedIn>()
        })
    }

    pub fn position(&self, entity: Entity) -> Option<Position> {
        self.app.world().get::<Position>(entity).copied()
    }

    pub fn exit_position(&mut self) -> Option<Position> {
        let entities = self.entity_ids();
        let mode = *self.app.world().resource::<GameMode>();
        entities.into_iter().find_map(|entity| {
            let entity = self.app.world().entity(entity);
            (entity.contains::<bd_core::components::ExitTile>()
                && entity
                    .get::<EntityScope>()
                    .is_some_and(|scope| scope.is_active(mode)))
            .then(|| entity.get::<Position>().copied())
            .flatten()
        })
    }

    pub fn outpost_map(&self) -> SmokeMap {
        self.app.world().resource::<OutpostState>().map.clone()
    }

    pub fn pool_current(&mut self, kind: PoolKind) -> Option<i32> {
        self.player()
            .and_then(|player| self.app.world().get::<Pools>(player))
            .and_then(|pools| pools.get(kind))
            .map(|pool| pool.current)
    }

    pub fn player_pool_kinds(&mut self) -> Vec<PoolKind> {
        self.player()
            .and_then(|player| self.app.world().get::<Pools>(player))
            .map(|pools| pools.iter().map(|pool| pool.kind).collect())
            .unwrap_or_default()
    }

    pub fn entity_pool_current(&self, entity: Entity, kind: PoolKind) -> Option<i32> {
        self.app
            .world()
            .get::<Pools>(entity)
            .and_then(|pools| pools.get(kind))
            .map(|pool| pool.current)
    }

    pub fn resource_current(&self, kind: PoolKind) -> Option<i32> {
        self.app
            .world()
            .resource::<bd_core::colony::production::ColonyResources>()
            .pools
            .get(kind)
            .map(|pool| pool.current)
    }

    pub fn latest_daily_summary(&self) -> Option<bd_core::colony::production::DailySummary> {
        self.app
            .world()
            .resource::<bd_core::colony::production::LatestDailySummary>()
            .0
            .clone()
    }

    pub fn colony_forecast(&mut self) -> bd_core::colony::production::ColonyForecast {
        let survivors = self
            .survivors()
            .into_iter()
            .filter_map(|entity| {
                Some(bd_core::colony::production::SurvivorWorkSnapshot {
                    task: self.app.world().get::<SurvivorTask>(entity)?.clone(),
                    position: *self.app.world().get::<Position>(entity)?,
                })
            })
            .collect::<Vec<_>>();
        let stations = self
            .stations()
            .into_iter()
            .filter_map(|entity| {
                Some(bd_core::colony::production::StationWorkSnapshot {
                    entity_bits: entity.to_bits(),
                    station_type: *self
                        .app
                        .world()
                        .get::<bd_core::colony::stations::StationType>(entity)?,
                    position: *self.app.world().get::<Position>(entity)?,
                })
            })
            .collect::<Vec<_>>();
        let nodes = self
            .resource_nodes_with_state()
            .into_iter()
            .map(|(_, kind, position, depleted)| {
                bd_core::colony::production::ResourceWorkSnapshot {
                    kind,
                    position,
                    depleted,
                }
            })
            .collect::<Vec<_>>();
        bd_core::colony::production::forecast_colony(
            self.app
                .world()
                .resource::<bd_core::colony::production::ColonyResources>(),
            &survivors,
            &stations,
            &nodes,
            self.app
                .world()
                .resource::<bd_core::colony::stations::StationCatalog>(),
        )
    }

    pub fn last_day_advanced_count(&self) -> usize {
        self.app
            .world()
            .resource::<ScenarioObservations>()
            .days
            .len()
    }

    pub fn last_daily_summary_count(&self) -> usize {
        self.app
            .world()
            .resource::<ScenarioObservations>()
            .daily_summaries
            .len()
    }

    pub fn last_resolved_count(&self) -> usize {
        self.app
            .world()
            .resource::<ScenarioObservations>()
            .resolved
            .len()
    }

    pub fn last_denied_count(&self) -> usize {
        self.app
            .world()
            .resource::<ScenarioObservations>()
            .denied
            .len()
    }

    pub fn last_defeat_count(&self) -> usize {
        self.app
            .world()
            .resource::<ScenarioObservations>()
            .defeats
            .len()
    }

    pub fn entity_count(&self) -> usize {
        self.app.world().entities().len() as usize
    }

    /// Phase 6 fixture setup for starvation boundary tests.
    pub fn fixture_set_colony_resource(&mut self, kind: PoolKind, value: i32) {
        if let Some(pool) = self
            .app
            .world_mut()
            .resource_mut::<bd_core::colony::production::ColonyResources>()
            .pools
            .get_mut(kind)
        {
            pool.current = value.clamp(pool.min, pool.max);
        }
    }

    /// Set an entity pool as scenario precondition; gameplay still owns all
    /// behavior under test after setup.
    pub fn fixture_set_entity_pool(&mut self, entity: Entity, kind: PoolKind, value: i32) {
        if let Some(mut pools) = self.app.world_mut().get_mut::<Pools>(entity)
            && let Some(pool) = pools.get_mut(kind)
        {
            pool.current = value.clamp(pool.min, pool.max);
        }
    }

    pub fn fixture_set_position(
        &mut self,
        entity: Entity,
        position: Position,
    ) -> Result<(), ScenarioError> {
        let Some(mut current) = self.app.world_mut().get_mut::<Position>(entity) else {
            return Err(ScenarioError::new(
                "position fixture",
                format!("entity {entity:?} has no Position"),
            ));
        };
        *current = position;
        Ok(())
    }

    pub fn fixture_set_outpost_tile(&mut self, position: Position, tile: Tile) {
        self.app
            .world_mut()
            .resource_mut::<SmokeMap>()
            .set(position.x, position.y, tile);
        self.app
            .world_mut()
            .resource_mut::<OutpostState>()
            .map
            .set(position.x, position.y, tile);
    }

    /// Activate or clear the production build interaction for action-gating tests.
    pub fn fixture_set_build_interaction(&mut self, active: bool) {
        *self
            .app
            .world_mut()
            .resource_mut::<bd_core::colony::stations::BuildInteraction>() = if active {
            bd_core::colony::stations::BuildInteraction::Selecting {
                selected_station: bd_core::colony::stations::StationType::Stove,
            }
        } else {
            bd_core::colony::stations::BuildInteraction::Inactive
        };
    }

    /// Select station content while leaving construction to the production action.
    pub fn fixture_select_station(&mut self, station_type: bd_core::colony::stations::StationType) {
        let mut interaction = self
            .app
            .world_mut()
            .resource_mut::<bd_core::colony::stations::BuildInteraction>();
        *interaction = match &*interaction {
            bd_core::colony::stations::BuildInteraction::Selecting { .. } => {
                bd_core::colony::stations::BuildInteraction::Selecting {
                    selected_station: station_type,
                }
            }
            bd_core::colony::stations::BuildInteraction::Placing {
                cursor, validation, ..
            } => bd_core::colony::stations::BuildInteraction::Placing {
                selected_station: station_type,
                cursor: *cursor,
                validation: validation.clone(),
            },
            bd_core::colony::stations::BuildInteraction::AwaitingResolution { cursor, .. } => {
                bd_core::colony::stations::BuildInteraction::AwaitingResolution {
                    selected_station: station_type,
                    cursor: *cursor,
                }
            }
            bd_core::colony::stations::BuildInteraction::Inactive => {
                // Scenario fixtures submit a direction through the domain action
                // rather than simulating the paused placement UI. Keep the
                // station selection without inventing an absolute cursor.
                bd_core::colony::stations::BuildInteraction::Selecting {
                    selected_station: station_type,
                }
            }
        };
    }

    pub fn fixture_select_station_assignment(&mut self, station: Entity) {
        self.app
            .world_mut()
            .resource_mut::<bd_core::colony::stations::PendingStationAssignment>()
            .0 = Some(station);
    }

    /// Install an active event interaction without registering deferred event content.
    pub fn fixture_set_event_interaction(&mut self, active: bool) {
        self.app
            .world_mut()
            .insert_resource(bd_core::events::CurrentEvent {
                event_id: "fixture.blocking_event".into(),
                node_id: "start".into(),
                previous_screen: "outpost".into(),
                active,
            });
    }

    pub fn deferred_resources_present(&self) -> Vec<&'static str> {
        let world = self.app.world();
        let mut present = Vec::new();
        if world
            .get_resource::<bd_core::events::EventRegistry>()
            .is_some()
        {
            present.push("events");
        }
        if world
            .get_resource::<bd_core::factions::FactionReputation>()
            .is_some()
        {
            present.push("reputation");
        }
        if world
            .get_resource::<bd_core::overworld::OverworldState>()
            .is_some()
        {
            present.push("overworld");
        }
        if world.get_resource::<bd_core::party::PartyState>().is_some() {
            present.push("party");
        }
        if world
            .get_resource::<bd_core::colony::raids::RaidState>()
            .is_some()
        {
            present.push("raids");
        }
        if world
            .get_resource::<bd_core::gabriel::GabrielState>()
            .is_some()
        {
            present.push("gabriel");
        }
        present
    }

    pub fn summary(&mut self) -> FoundationSummary {
        let entity_ids = self.entity_ids();
        let world = self.app.world();
        let session = world.resource::<RunSession>().clone();
        let last_completed = world.resource::<bd_core::session::LastCompletedRun>();
        let mode = *world.resource::<GameMode>();
        let map = world.resource::<SmokeMap>();
        let player = entity_ids
            .iter()
            .copied()
            .find(|entity| world.entity(*entity).contains::<Player>());

        let mut survivors = 0;
        let mut assigned_survivors = 0;
        let mut stations = 0;
        let mut resource_nodes = 0;
        let mut hostiles = 0;
        let mut loose_items = 0;
        let mut carried_items = 0;
        for entity_id in entity_ids {
            let entity = world.entity(entity_id);
            if entity.contains::<Survivor>() {
                survivors += 1;
                if matches!(
                    entity.get::<SurvivorTask>(),
                    Some(SurvivorTask::AssignedTo(_))
                ) {
                    assigned_survivors += 1;
                }
            }
            stations += usize::from(entity.contains::<Station>());
            resource_nodes += usize::from(entity.contains::<ResourceNode>());
            hostiles += usize::from(entity.contains::<FactionMember>());
            if entity.contains::<Item>() {
                match (player, entity.get::<ContainedIn>()) {
                    (Some(player), Some(container)) if container.0 == player => carried_items += 1,
                    (_, None) => loose_items += 1,
                    _ => {}
                }
            }
        }

        let storage_items = world
            .resource::<ColonyStorage>()
            .items
            .values()
            .copied()
            .sum();
        let melee_skill = player
            .and_then(|player| world.get::<SkillProgression>(player))
            .map_or(0, |progression| progression.melee);
        let medicine_skill = player
            .and_then(|player| world.get::<SkillProgression>(player))
            .map_or(0, |progression| progression.medicine);
        let player_position = player.and_then(|player| world.get::<Position>(player).copied());
        let player_health = player
            .and_then(|player| world.get::<Pools>(player))
            .and_then(|pools| pools.get(PoolKind::Health))
            .map(|health| health.current);
        let trace_events = world
            .resource::<SignalTrace>()
            .entries
            .iter()
            .map(|entry| format!("{}:{}:{}", entry.stage, entry.signal_type, entry.summary))
            .collect();

        FoundationSummary {
            mode,
            session_phase: session.phase,
            outcome: session.outcome,
            last_completed_outcome: last_completed.outcome,
            dungeon_id: session.dungeon_id.clone(),
            day: session.day,
            turn: session.turn,
            map_size: (map.width, map.height),
            player_position,
            player_health,
            survivors,
            assigned_survivors,
            stations,
            resource_nodes,
            hostiles,
            loose_items,
            carried_items,
            storage_items,
            extracted_loot: session.extracted_loot,
            melee_skill,
            medicine_skill,
            replay_intents: session.replay_intents.clone(),
            trace_events,
        }
    }

    fn entity_ids(&mut self) -> Vec<Entity> {
        let mut query = self.app.world_mut().query::<Entity>();
        query.iter(self.app.world()).collect()
    }

    fn clear_observations(&mut self) {
        let mut observations = self.app.world_mut().resource_mut::<ScenarioObservations>();
        observations.resolved.clear();
        observations.denied.clear();
        observations.transitions.clear();
        observations.days.clear();
        observations.daily_summaries.clear();
        observations.defeats.clear();
    }
}

fn direction_between(from: Position, to: Position) -> Option<Direction> {
    match (to.x - from.x, to.y - from.y) {
        (1, 0) => Some(Direction::East),
        (-1, 0) => Some(Direction::West),
        (0, 1) => Some(Direction::South),
        (0, -1) => Some(Direction::North),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bd_core::{
        components::{Name, Player, Position},
        pools::Pools,
        session::{RunOutcome, RunSession},
        signals::{ActionIntent, PoolKind},
        spatial::{GameMode, PersistentEntity, TransientEntity, TransitionIntent},
    };
    use bevy_ecs::{message::Messages, query::With};

    fn enter_foundation_dungeon(app: &mut App) {
        app.world_mut()
            .resource_mut::<Messages<TransitionIntent>>()
            .write(TransitionIntent {
                target: GameMode::Outpost,
                node_id: None,
            });
        app.update();
        app.world_mut()
            .resource_mut::<Messages<TransitionIntent>>()
            .write(TransitionIntent {
                target: GameMode::Tactical,
                node_id: Some("dungeon.foundation".into()),
            });
        app.update();
    }

    fn player_state(app: &mut App) -> (bevy_ecs::entity::Entity, Position, i32, i32) {
        let mut query = app
            .world_mut()
            .query_filtered::<(bevy_ecs::entity::Entity, &Position, &Pools), With<Player>>();
        let (entity, position, pools) = query
            .iter(app.world())
            .next()
            .expect("foundation dungeon should contain a player");
        let hp = pools
            .get(PoolKind::Health)
            .expect("player should have health")
            .current;
        let ap = pools
            .get(PoolKind::ActionPoints)
            .expect("player should have action points")
            .current;
        (entity, *position, hp, ap)
    }

    fn rat_state(app: &mut App) -> (Position, i32) {
        let mut query = app
            .world_mut()
            .query_filtered::<(&Position, &Pools), With<Name>>();
        query
            .iter(app.world())
            .find(|(_, pools)| pools.get(PoolKind::Health).is_some())
            .map(|(position, pools)| {
                (
                    *position,
                    pools
                        .get(PoolKind::Health)
                        .expect("hostile should have health")
                        .current,
                )
            })
            .expect("foundation dungeon should contain a named hostile")
    }

    #[test]
    fn foundation_app_excludes_deferred_resources() {
        let app = foundation_app();

        assert!(
            app.world()
                .get_resource::<bd_core::events::EventRegistry>()
                .is_none()
        );
        assert!(
            app.world()
                .get_resource::<bd_core::events::CurrentEvent>()
                .is_none()
        );
        assert!(
            app.world()
                .get_resource::<bd_core::factions::FactionReputation>()
                .is_none()
        );
        assert!(
            app.world()
                .get_resource::<bd_core::overworld::OverworldState>()
                .is_none()
        );
        assert!(
            app.world()
                .get_resource::<bd_core::overworld::TravelContext>()
                .is_none()
        );
        assert!(
            app.world()
                .get_resource::<bd_core::party::PartyState>()
                .is_none()
        );
        assert!(
            app.world()
                .get_resource::<bd_core::colony::raids::RaidState>()
                .is_none()
        );
        assert!(
            app.world()
                .get_resource::<bd_core::dialogue::DialogueLog>()
                .is_none()
        );
        assert!(
            app.world()
                .get_resource::<bd_core::gabriel::GabrielState>()
                .is_none()
        );
        assert!(app.world().get_resource::<FoundationContent>().is_some());
    }

    #[test]
    /// Legacy fixture regression only. This test deliberately manufactures
    /// carried loot and exit position, so it is not Foundation acceptance
    /// evidence; `bd_app/tests/foundation_scenario.rs` owns that proof.
    fn legacy_direct_mutation_round_trip_fixture_regression() {
        let mut app = foundation_app();

        app.world_mut()
            .resource_mut::<Messages<TransitionIntent>>()
            .write(TransitionIntent {
                target: GameMode::Outpost,
                node_id: None,
            });
        app.update();

        let survivor_count = app
            .world_mut()
            .query_filtered::<(), With<bd_core::colony::survivors::Survivor>>()
            .iter(app.world())
            .count();
        assert_eq!(survivor_count, 3);
        let supplies_before = app
            .world()
            .resource::<bd_core::colony::production::ColonyResources>()
            .pools
            .get(bd_core::signals::PoolKind::Supplies)
            .unwrap()
            .current;

        app.world_mut()
            .resource_mut::<Messages<TransitionIntent>>()
            .write(TransitionIntent {
                target: GameMode::Tactical,
                node_id: Some("dungeon.foundation".into()),
            });
        app.update();

        assert_eq!(
            app.world().resource::<GameMode>().clone(),
            GameMode::Tactical
        );
        assert_eq!(app.world().resource::<bd_core::map::SmokeMap>().width, 12);
        assert_eq!(app.world().resource::<bd_core::map::SmokeMap>().height, 8);
        let factioned_enemies = app
            .world_mut()
            .query_filtered::<&bd_core::relationships::FactionMember, With<bd_core::relationships::FactionMember>>()
            .iter(app.world())
            .map(|faction| faction.0.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            factioned_enemies,
            vec!["faction.placeholder_a"],
            "fixed encounter should carry content faction identity"
        );
        let player = app
            .world_mut()
            .query_filtered::<(bevy_ecs::entity::Entity, &Position), With<Player>>()
            .iter(app.world())
            .next()
            .map(|(entity, position)| (entity, *position))
            .expect("fixed dungeon should provide a player");
        let potion = app
            .world_mut()
            .query_filtered::<bevy_ecs::entity::Entity, With<bd_core::inventory::Item>>()
            .iter(app.world())
            .next()
            .expect("fixed dungeon should provide loot");
        app.world_mut()
            .entity_mut(potion)
            .insert(bd_core::relationships::ContainedIn(player.0));
        assert_eq!(player.1, Position { x: 1, y: 1 });

        app.world_mut()
            .entity_mut(player.0)
            .insert(Position { x: 10, y: 6 });

        app.world_mut()
            .resource_mut::<Messages<bd_core::signals::ActionIntent>>()
            .write(bd_core::signals::ActionIntent {
                actor: player.0,
                action_id: "ability.extract".into(),
                direction: None,
                target: None,
            });
        app.update();
        app.update();

        assert_eq!(
            app.world().resource::<GameMode>().clone(),
            GameMode::Outpost
        );
        assert_eq!(
            app.world().resource::<RunSession>().outcome,
            RunOutcome::Extracted
        );
        assert!(app.world().resource::<RunSession>().extraction_applied);
        let survivor_count_after = app
            .world_mut()
            .query_filtered::<(), With<bd_core::colony::survivors::Survivor>>()
            .iter(app.world())
            .count();
        assert_eq!(survivor_count_after, 3);
        let supplies_after = app
            .world()
            .resource::<bd_core::colony::production::ColonyResources>()
            .pools
            .get(bd_core::signals::PoolKind::Supplies)
            .unwrap()
            .current;
        assert_eq!(supplies_after, supplies_before);
        assert_eq!(
            app.world()
                .resource::<bd_core::colony::production::ColonyStorage>()
                .count("item.healing_potion"),
            1,
            "extracted carried loot should enter colony storage exactly once",
        );
        let transient_count = app
            .world_mut()
            .query_filtered::<(), With<TransientEntity>>()
            .iter(app.world())
            .count();
        assert_eq!(transient_count, 0);
        assert!(
            app.world_mut()
                .query_filtered::<(), With<PersistentEntity>>()
                .iter(app.world())
                .count()
                >= 4
        );
    }

    #[test]
    fn foundation_turn_idle_does_not_run_enemy_ai() {
        let mut app = foundation_app();
        enter_foundation_dungeon(&mut app);

        let (_, _, hp_before, _) = player_state(&mut app);
        let (rat_position_before, _) = rat_state(&mut app);
        let turn_before = app.world().resource::<bd_core::time::GameTime>().turn;

        for _ in 0..3 {
            app.update();
        }

        let (_, _, hp_after, _) = player_state(&mut app);
        let (rat_position_after, _) = rat_state(&mut app);
        let turn_after = app.world().resource::<bd_core::time::GameTime>().turn;

        assert_eq!(
            hp_after, hp_before,
            "idle frames must not damage the player"
        );
        assert_eq!(
            rat_position_after, rat_position_before,
            "idle frames must not move enemies"
        );
        assert_eq!(turn_after, turn_before, "idle frames must not advance time");
    }

    #[test]
    fn accepted_move_advances_exactly_one_turn() {
        let mut app = foundation_app();
        enter_foundation_dungeon(&mut app);
        let (player, _, _, ap_before) = player_state(&mut app);
        let turn_before = app.world().resource::<bd_core::time::GameTime>().turn;

        app.world_mut()
            .resource_mut::<Messages<ActionIntent>>()
            .write(ActionIntent {
                actor: player,
                action_id: "ability.move".into(),
                direction: Some(bd_core::direction::Direction::East),
                target: None,
            });
        app.update();

        let (_, _, _, ap_after) = player_state(&mut app);
        let turn_after = app.world().resource::<bd_core::time::GameTime>().turn;
        assert_eq!(
            turn_after,
            turn_before + 1,
            "one accepted move must advance exactly one turn"
        );
        assert_eq!(ap_after, ap_before - 1, "one move must spend one AP");
    }

    #[test]
    fn rejected_action_does_not_start_enemy_phase() {
        let mut app = foundation_app();
        enter_foundation_dungeon(&mut app);
        let (player, _, hp_before, ap_before) = player_state(&mut app);
        let (rat_position_before, _) = rat_state(&mut app);

        let rat = app
            .world_mut()
            .query_filtered::<bevy_ecs::entity::Entity, With<Name>>()
            .iter(app.world())
            .next()
            .expect("foundation dungeon should contain a named hostile");
        app.world_mut()
            .resource_mut::<Messages<ActionIntent>>()
            .write(ActionIntent {
                actor: player,
                action_id: "ability.attack".into(),
                direction: None,
                target: Some(rat),
            });
        app.update();

        let (_, _, hp_after, ap_after) = player_state(&mut app);
        let (rat_position_after, _) = rat_state(&mut app);
        assert_eq!(
            hp_after, hp_before,
            "rejected actions must not trigger enemy damage"
        );
        assert_eq!(ap_after, ap_before, "rejected actions must not spend AP");
        assert_eq!(
            rat_position_after, rat_position_before,
            "rejected actions must not trigger enemy movement"
        );
    }

    #[test]
    fn accepted_action_runs_enemy_phase_once() {
        let mut app = foundation_app();
        enter_foundation_dungeon(&mut app);
        let (player, _, _, _) = player_state(&mut app);

        app.world_mut()
            .resource_mut::<Messages<ActionIntent>>()
            .write(ActionIntent {
                actor: player,
                action_id: "ability.move".into(),
                direction: Some(bd_core::direction::Direction::East),
                target: None,
            });
        app.update();

        let (rat_after_player, _) = rat_state(&mut app);
        app.update();
        let (rat_after_enemy, _) = rat_state(&mut app);
        app.update();
        let (rat_after_idle, _) = rat_state(&mut app);

        assert_ne!(
            rat_after_enemy, rat_after_player,
            "accepted player action should permit one enemy phase"
        );
        assert_eq!(
            rat_after_idle, rat_after_enemy,
            "one player action must not permit repeated enemy phases"
        );
    }

    #[test]
    fn player_action_is_locked_during_enemy_phase() {
        let mut app = foundation_app();
        enter_foundation_dungeon(&mut app);
        let (player, _, _, ap_before) = player_state(&mut app);

        app.world_mut()
            .resource_mut::<Messages<ActionIntent>>()
            .write(ActionIntent {
                actor: player,
                action_id: "ability.move".into(),
                direction: Some(bd_core::direction::Direction::East),
                target: None,
            });
        app.update();
        let (_, _, _, ap_after_first) = player_state(&mut app);

        // This intent arrives while the enemy phase is pending and must not
        // become a second player turn in the same resolution window.
        app.world_mut()
            .resource_mut::<Messages<ActionIntent>>()
            .write(ActionIntent {
                actor: player,
                action_id: "ability.move".into(),
                direction: Some(bd_core::direction::Direction::East),
                target: None,
            });
        app.update();

        let (_, player_position, _, ap_after) = player_state(&mut app);
        assert_eq!(player_position, Position { x: 2, y: 1 });
        assert!(
            ap_after > ap_after_first,
            "enemy-phase input must not spend AP; expected regeneration without a second cost (before {}, after {})",
            ap_after_first,
            ap_after
        );
        assert!(ap_before >= ap_after);
    }
}
