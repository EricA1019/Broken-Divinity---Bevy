//! bd_test_support — Shared test utilities for the BD Kernel.
//!
//! Provides deterministic RNG, minimal app builders, and snapshot helpers.

use bd_core::content::FoundationContent;
use bd_core::{
    BdSet,
    colony::{
        production::ColonyStorage,
        stations::Station,
        survivors::{Survivor, SurvivorTask},
    },
    components::{Player, Position, ResourceNode},
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
use bevy_app::{App, Update};
use bevy_ecs::{
    entity::Entity,
    message::{MessageReader, Messages},
    prelude::{IntoScheduleConfigs, ResMut, Resource},
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::{
    collections::{HashMap, HashSet},
    fmt,
    sync::atomic::{AtomicU64, Ordering},
};

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
    app.insert_resource(foundation_content());
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
}

fn observe_scenario_results(
    mut resolved: MessageReader<ActionResolved>,
    mut denied: MessageReader<ActionDenied>,
    mut transitions: MessageReader<TransitionComplete>,
    mut days: MessageReader<bd_core::time::DayAdvanced>,
    mut daily_summaries: MessageReader<bd_core::colony::production::DailySummary>,
    mut observations: ResMut<ScenarioObservations>,
) {
    observations.resolved.extend(resolved.read().cloned());
    observations.denied.extend(denied.read().cloned());
    observations.transitions.extend(transitions.read().cloned());
    observations.days.extend(days.read().copied());
    observations
        .daily_summaries
        .extend(daily_summaries.read().cloned());
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
        let mut app = foundation_app();
        app.insert_resource(RunSession::new(seed));
        app.init_resource::<ScenarioObservations>();
        app.add_systems(
            Update,
            observe_scenario_results.in_set(BdSet::ViewModelBuild),
        );
        Self { app }
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
        self.request_transition(
            "colony → fixed dungeon",
            GameMode::Tactical,
            Some(dungeon_id),
        )
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
        for _ in 0..12 {
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
            self.expect_action(step, player, "ability.wait", None, None)?;
        }
        Err(ScenarioError::new(
            step,
            "hostile did not approach within 12 wait actions",
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

    pub fn first_survivor(&mut self) -> Option<Entity> {
        let entities = self.entity_ids();
        entities
            .into_iter()
            .find(|entity| self.app.world().entity(*entity).contains::<Survivor>())
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

    pub fn pool_current(&mut self, kind: PoolKind) -> Option<i32> {
        self.player()
            .and_then(|player| self.app.world().get::<Pools>(player))
            .and_then(|pools| pools.get(kind))
            .map(|pool| pool.current)
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

    pub fn last_day_advanced_count(&self) -> usize {
        self.app
            .world()
            .resource::<ScenarioObservations>()
            .days
            .len()
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
        assert_eq!(app.world().resource::<bd_core::map::SmokeMap>().width, 8);
        assert_eq!(app.world().resource::<bd_core::map::SmokeMap>().height, 6);
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
            .insert(Position { x: 6, y: 4 });

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
        assert_eq!(
            app.world().resource::<RunSession>().extraction_applied,
            true
        );
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
