#![allow(clippy::too_many_arguments, clippy::type_complexity)]

//! Save/load system — JSON-based checkpoint on shelter return.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::core::abilities::SprintCooldown;
use crate::core::components::{Player, Position};
use crate::core::gamelog::GameLog;
use crate::core::inventory::{Equipment, Inventory, RangedWeaponState};
use crate::core::movement::MapTiles;
use crate::core::perks::{PendingPerkChoices, PlayerPerks};
use crate::core::player::PlayerBundle;
use crate::core::resources::{ColonyTickTimer, ShelterResources, TravelDayTimer, WorldSeed};
use crate::core::sanity::RaidExposure;
use crate::core::state::AppState;
use crate::core::stats::{CombatStats, EntityName, PlayerProgression, SkillId, SkillState};
use crate::core::turn::GameTime;
use crate::game::colony::raids::{ActiveRaid, PendingRaidReport, RaidChance};
use crate::game::colony::research::CompletedResearch;
use crate::game::colony::spawn::ShelterState;
use crate::game::colony::stations::{Station, StationType};
use crate::game::colony::survivors::{Survivor, SurvivorNeeds, SurvivorTask};
use crate::game::dungeon::gabriel::GabrielState;
use crate::game::dungeon::lore::{LoreFragment, LoreJournal};
use crate::game::dungeon::spawn::DungeonState as CurrentDungeonState;
use crate::game::dungeon::theme::DungeonTheme;
use crate::game::factions::{Faction, Factions};
use crate::game::overworld::graphgen::{DungeonStoryTag, OverworldGraph};
use crate::game::overworld::map::{PlayerMapPosition, SelectedDestination, WorldMap};
use crate::game::overworld::travel::TravelState;

const SAVE_VERSION: u32 = 5;

/// Load-only resource used by future state restoration wiring.
#[derive(Resource, Debug, Clone, Default)]
pub struct PendingLoad(pub Option<SaveGame>);

impl PendingLoad {
    pub fn take(&mut self) -> Option<SaveGame> {
        self.0.take()
    }
}

/// Runtime handoff for player state when no player entity exists in the current state.
#[derive(Resource, Debug, Clone, Default)]
pub struct PlayerSnapshot(pub Option<SavePlayerState>);

/// Request resource toggled by UI when the player wants to save and return to the menu.
#[derive(Resource, Debug, Clone, Default)]
pub struct SaveAndQuitRequested(pub bool);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadGameError {
    MissingSave,
    InvalidData,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SaveAppState {
    #[default]
    Menu,
    Overworld,
    Dungeon,
    Colony,
    Combat,
    GameOver,
}

impl From<&AppState> for SaveAppState {
    fn from(value: &AppState) -> Self {
        match value {
            AppState::Menu => Self::Menu,
            AppState::Overworld => Self::Overworld,
            AppState::Dungeon => Self::Dungeon,
            AppState::Colony => Self::Colony,
            AppState::Combat => Self::Combat,
            AppState::GameOver => Self::GameOver,
        }
    }
}

impl SaveAppState {
    pub fn into_runtime_state(self) -> AppState {
        match self {
            SaveAppState::Menu | SaveAppState::Colony | SaveAppState::GameOver => AppState::Colony,
            SaveAppState::Overworld => AppState::Overworld,
            // Combat (shelter raids) isn't saveable mid-fight — restore to
            // the pre-combat context so the player re-enters the dungeon/colony.
            SaveAppState::Dungeon | SaveAppState::Combat => AppState::Dungeon,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SaveGameTime {
    #[serde(default)]
    pub turn: u32,
}

impl SaveGameTime {
    fn from_time(time: Option<&GameTime>) -> Self {
        Self {
            turn: time.map_or(0, |time| time.turn),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SavePosition {
    #[serde(default)]
    pub x: i32,
    #[serde(default)]
    pub y: i32,
}

impl From<&Position> for SavePosition {
    fn from(value: &Position) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SaveCombatStats {
    #[serde(default)]
    pub hp: i32,
    #[serde(default)]
    pub hp_max: i32,
    #[serde(default)]
    pub speed: u8,
    #[serde(default)]
    pub ar: i32,
    #[serde(default)]
    pub md: i32,
    #[serde(default)]
    pub skills: HashMap<SkillId, SkillState>,
}

impl From<&CombatStats> for SaveCombatStats {
    fn from(value: &CombatStats) -> Self {
        Self {
            hp: value.hp,
            hp_max: value.hp_max,
            speed: value.speed,
            ar: value.ar,
            md: value.md,
            skills: value.skills.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SaveRangedState {
    #[serde(default)]
    pub clip_current: u8,
    #[serde(default)]
    pub clip_size: u8,
}

impl From<&RangedWeaponState> for SaveRangedState {
    fn from(value: &RangedWeaponState) -> Self {
        Self {
            clip_current: value.clip_current,
            clip_size: value.clip_size,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SaveSanityState {
    #[serde(default)]
    pub current: u32,
    #[serde(default)]
    pub max: u32,
}

impl From<&RaidExposure> for SaveSanityState {
    fn from(value: &RaidExposure) -> Self {
        Self {
            current: value.current,
            max: value.max,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SavePlayerState {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub position: SavePosition,
    #[serde(default)]
    pub stats: SaveCombatStats,
    #[serde(default)]
    pub inventory: Inventory,
    #[serde(default)]
    pub inventory_slot_count: usize,
    #[serde(default)]
    pub equipment: Equipment,
    #[serde(default)]
    pub ranged_state: SaveRangedState,
    #[serde(default)]
    pub sanity: SaveSanityState,
    #[serde(default)]
    pub perks: PlayerPerks,
    #[serde(default)]
    pub progression: PlayerProgression,
    #[serde(default)]
    pub sprint_cooldown: u32,
}

impl SavePlayerState {
    fn from_snapshot(
        position: &Position,
        stats: &CombatStats,
        inventory: &Inventory,
        equipment: &Equipment,
        ranged_state: &RangedWeaponState,
        sanity: &RaidExposure,
        perks: &PlayerPerks,
        progression: &PlayerProgression,
        name: Option<&EntityName>,
        sprint_cooldown: u32,
    ) -> Self {
        let mut progression = progression.clone();
        progression.ensure_complete();
        let mut canonical_stats = stats.clone();
        progression.sync_pilot_combat_skill_proxies(&mut canonical_stats.skills);

        Self {
            name: name
                .map(|name| name.name.clone())
                .unwrap_or_else(|| "Player".to_string()),
            position: SavePosition::from(position),
            stats: SaveCombatStats::from(&canonical_stats),
            inventory: inventory.clone(),
            inventory_slot_count: inventory.slots.iter().filter(|slot| slot.is_some()).count(),
            equipment: equipment.clone(),
            ranged_state: SaveRangedState::from(ranged_state),
            sanity: SaveSanityState::from(sanity),
            perks: perks.clone(),
            progression,
            sprint_cooldown,
        }
    }
}

pub fn snapshot_player_state(
    position: &Position,
    stats: &CombatStats,
    inventory: &Inventory,
    equipment: &Equipment,
    ranged_state: &RangedWeaponState,
    sanity: &RaidExposure,
    perks: &PlayerPerks,
    progression: &PlayerProgression,
    name: Option<&EntityName>,
    sprint_cooldown: u32,
) -> SavePlayerState {
    SavePlayerState::from_snapshot(
        position,
        stats,
        inventory,
        equipment,
        ranged_state,
        sanity,
        perks,
        progression,
        name,
        sprint_cooldown,
    )
}

pub fn spawn_player_from_save(commands: &mut Commands, player: &SavePlayerState) {
    spawn_player_from_save_at(commands, player, None);
}

pub fn spawn_player_from_save_at(
    commands: &mut Commands,
    player: &SavePlayerState,
    position_override: Option<(i32, i32)>,
) {
    let mut bundle = PlayerBundle::new(player.position.x, player.position.y);
    if let Some((x, y)) = position_override {
        bundle.position = Position::new(x, y);
    }
    bundle.name = EntityName {
        name: if player.name.is_empty() {
            "Player".to_string()
        } else {
            player.name.clone()
        },
    };
    bundle.stats = CombatStats {
        hp: player.stats.hp,
        hp_max: player.stats.hp_max,
        speed: player.stats.speed.max(1),
        ar: player.stats.ar,
        md: player.stats.md,
        skills: player.stats.skills.clone(),
    };
    bundle.budget = crate::core::turn::ActionBudget::new(bundle.stats.speed);
    bundle.inventory = player.inventory.clone();
    bundle.equipment = player.equipment.clone();
    bundle.ranged_state = RangedWeaponState {
        clip_current: player.ranged_state.clip_current,
        clip_size: player.ranged_state.clip_size,
    };
    bundle.sanity = RaidExposure {
        current: player.sanity.current,
        max: player.sanity.max.max(1),
    };
    bundle.perks = player.perks.clone();
    bundle.progression = if player.progression.is_empty() {
        PlayerProgression::from_legacy_skills(&player.stats.skills)
    } else {
        let mut progression = player.progression.clone();
        progression.ensure_complete();
        progression
    };
    bundle.sprint_cooldown = SprintCooldown {
        remaining: player.sprint_cooldown,
    };
    commands.spawn(bundle);
}

pub fn restore_persistent_run_resources(commands: &mut Commands, save: &SaveGame) {
    commands.insert_resource(WorldSeed(save.seed));
    commands.insert_resource(GameTime {
        turn: save.game_time.turn,
    });
    commands.insert_resource(LoreJournal {
        fragments: save.lore_journal.fragments.clone(),
    });
    commands.insert_resource(save.colony.resources.clone());
    commands.insert_resource(save.gabriel.clone());
    commands.insert_resource(PlayerSnapshot(Some(save.player.clone())));
    commands.insert_resource(PendingSurvivorLoad(save.colony.survivors.clone()));
    commands.insert_resource(PendingStationLoad(save.colony.stations.clone()));
    commands.insert_resource(save.colony.research.clone());
    if let Some(report) = save.colony.pending_raid_report.clone() {
        commands.insert_resource(report);
    } else {
        commands.remove_resource::<PendingRaidReport>();
    }
    // Only reset destination if no pending travel — preserve travel context on load
    if save.overworld.travel.is_none() {
        commands.insert_resource(SelectedDestination::default());
    }

    if let Some(mut graph) = save.overworld.graph.clone() {
        graph.ensure_story_tags();
        commands.insert_resource(WorldMap(graph));
    } else {
        commands.remove_resource::<WorldMap>();
    }

    commands.insert_resource(PlayerMapPosition {
        current_node: save.overworld.player_position.current_node,
    });

    if save.overworld.factions.is_empty() {
        commands.remove_resource::<Factions>();
    } else {
        commands.insert_resource(Factions(save.overworld.factions.clone()));
    }

    if let Some(travel) = save.overworld.travel.clone() {
        commands.insert_resource(travel);
    } else {
        commands.remove_resource::<TravelState>();
    }

    let recap_message = format!(
        "Load recap: restored turn {} in {:?}.",
        save.game_time.turn, save.app_state
    );
    let objective_message = load_objective_message(save);
    commands.queue(move |world: &mut World| {
        let turn = world.resource::<GameTime>().turn;
        let Some(mut log) = world.get_resource_mut::<GameLog>() else {
            return;
        };
        log.push(recap_message, crate::core::gamelog::LogColor::Status, turn);
        log.push(
            objective_message,
            crate::core::gamelog::LogColor::Status,
            turn,
        );
    });
}

fn load_objective_message(save: &SaveGame) -> String {
    if save.colony.resources.food == 0 || save.colony.resources.water == 0 {
        return "Objective: secure food and water immediately.".to_string();
    }

    match save.app_state {
        SaveAppState::Overworld => {
            "Objective: choose a travel destination and continue across connected routes."
                .to_string()
        }
        SaveAppState::Dungeon | SaveAppState::Combat => {
            "Objective: continue progression through the dungeon safely.".to_string()
        }
        SaveAppState::Colony | SaveAppState::Menu | SaveAppState::GameOver => {
            "Objective: reach the shelter gate when you are ready to travel.".to_string()
        }
    }
}

/// Reset long-lived run resources when returning to the menu.
pub fn reset_run_state_for_menu(mut commands: Commands) {
    commands.insert_resource(GameTime::default());
    commands.insert_resource(GameLog::default());
    commands.insert_resource(ColonyTickTimer::default());
    commands.insert_resource(TravelDayTimer::default());
    commands.insert_resource(RaidChance::default());
    commands.insert_resource(GabrielState::default());
    commands.insert_resource(LoreJournal::default());
    commands.insert_resource(SelectedDestination::default());
    commands.insert_resource(PendingPerkChoices::default());
    commands.insert_resource(PendingLoad::default());
    commands.insert_resource(SaveAndQuitRequested::default());
    commands.insert_resource(PlayerSnapshot::default());
    commands.insert_resource(PendingSurvivorLoad::default());
    commands.insert_resource(PendingStationLoad::default());
    commands.insert_resource(CompletedResearch::default());

    commands.remove_resource::<MapTiles>();
    commands.remove_resource::<ShelterState>();
    commands.remove_resource::<CurrentDungeonState>();
    commands.remove_resource::<WorldMap>();
    commands.remove_resource::<PlayerMapPosition>();
    commands.remove_resource::<Factions>();
    commands.remove_resource::<ActiveRaid>();
    commands.remove_resource::<PendingRaidReport>();
    commands.remove_resource::<TravelState>();
    commands.remove_resource::<ShelterResources>();
    commands.remove_resource::<WorldSeed>();
}

/// Simplified serializable version of `SurvivorTask`.
/// Transient states (SeekingFood, SeekingWater, Resting, Patrolling) map to Idle on save.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub enum SaveSurvivorTask {
    #[default]
    Idle,
    Working {
        x: i32,
        y: i32,
    },
}

impl From<&SurvivorTask> for SaveSurvivorTask {
    fn from(task: &SurvivorTask) -> Self {
        match task {
            SurvivorTask::Working(pos) => SaveSurvivorTask::Working { x: pos.x, y: pos.y },
            _ => SaveSurvivorTask::Idle,
        }
    }
}

impl SaveSurvivorTask {
    pub fn to_runtime(&self) -> SurvivorTask {
        match self {
            SaveSurvivorTask::Idle => SurvivorTask::Idle,
            SaveSurvivorTask::Working { x, y } => SurvivorTask::Working(IVec2::new(*x, *y)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SaveSurvivor {
    pub name: String,
    #[serde(default)]
    pub x: i32,
    #[serde(default)]
    pub y: i32,
    #[serde(default)]
    pub hunger: u32,
    #[serde(default)]
    pub thirst: u32,
    #[serde(default)]
    pub rest: u32,
    #[serde(default)]
    pub task: SaveSurvivorTask,
}

/// Resource inserted during load to carry saved survivor data into colony OnEnter.
#[derive(Resource, Debug, Clone, Default)]
pub struct PendingSurvivorLoad(pub Vec<SaveSurvivor>);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SaveStation {
    pub kind: StationType,
    #[serde(default)]
    pub tier: u8,
    #[serde(default)]
    pub worker_slots: u8,
    #[serde(default)]
    pub workers_assigned: u8,
    #[serde(default)]
    pub x: i32,
    #[serde(default)]
    pub y: i32,
}

impl SaveStation {
    pub fn to_runtime(&self) -> Station {
        let worker_slots = self.worker_slots.max(self.kind.worker_slots());
        Station {
            kind: self.kind,
            tier: self.tier.max(1),
            worker_slots,
            workers_assigned: self.workers_assigned.min(worker_slots),
        }
    }
}

impl From<(&Station, &Position)> for SaveStation {
    fn from((station, position): (&Station, &Position)) -> Self {
        Self {
            kind: station.kind,
            tier: station.tier,
            worker_slots: station.worker_slots,
            workers_assigned: station.workers_assigned,
            x: position.x,
            y: position.y,
        }
    }
}

/// Resource inserted during load or colony teardown to restore station entities.
#[derive(Resource, Debug, Clone, Default)]
pub struct PendingStationLoad(pub Vec<SaveStation>);

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SaveColonyState {
    #[serde(default)]
    pub shelter_seed: u64,
    #[serde(default)]
    pub resources: ShelterResources,
    #[serde(default)]
    pub survivors: Vec<SaveSurvivor>,
    #[serde(default)]
    pub stations: Vec<SaveStation>,
    #[serde(default)]
    pub research: CompletedResearch,
    #[serde(default)]
    pub pending_raid_report: Option<PendingRaidReport>,
}

impl SaveColonyState {
    fn from_resources(
        shelter_state: Option<&ShelterState>,
        resources: Option<&ShelterResources>,
        survivors: Vec<SaveSurvivor>,
        stations: Vec<SaveStation>,
        research: Option<&CompletedResearch>,
        pending_raid_report: Option<&PendingRaidReport>,
    ) -> Self {
        Self {
            shelter_seed: shelter_state.map_or(0, |state| state.seed),
            resources: resources.cloned().unwrap_or_default(),
            survivors,
            stations,
            research: research.cloned().unwrap_or_default(),
            pending_raid_report: pending_raid_report.cloned(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SaveOverworldPlayerPosition {
    #[serde(default)]
    pub current_node: usize,
}

impl From<&PlayerMapPosition> for SaveOverworldPlayerPosition {
    fn from(value: &PlayerMapPosition) -> Self {
        Self {
            current_node: value.current_node,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SaveOverworldState {
    #[serde(default)]
    pub graph: Option<OverworldGraph>,
    #[serde(default)]
    pub player_position: SaveOverworldPlayerPosition,
    #[serde(default)]
    pub factions: Vec<Faction>,
    #[serde(default)]
    pub travel: Option<TravelState>,
}

impl SaveOverworldState {
    fn from_resources(
        world_map: Option<&WorldMap>,
        player_position: Option<&PlayerMapPosition>,
        factions: Option<&Factions>,
        travel: Option<&TravelState>,
    ) -> Self {
        Self {
            graph: world_map.map(|world_map| world_map.0.clone()),
            player_position: player_position
                .map(SaveOverworldPlayerPosition::from)
                .unwrap_or_default(),
            factions: factions
                .map(|factions| factions.0.clone())
                .unwrap_or_default(),
            travel: travel.cloned(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SaveDungeonState {
    #[serde(default)]
    pub floor_number: u32,
    #[serde(default)]
    pub max_floors: u32,
    #[serde(default)]
    pub seed: u64,
    #[serde(default)]
    pub theme: Option<DungeonTheme>,
    #[serde(default)]
    pub origin_node_id: Option<usize>,
    #[serde(default)]
    pub story_tag: Option<DungeonStoryTag>,
}

impl From<&CurrentDungeonState> for SaveDungeonState {
    fn from(value: &CurrentDungeonState) -> Self {
        Self {
            floor_number: value.floor_number,
            max_floors: value.max_floors,
            seed: value.seed,
            theme: Some(value.theme),
            origin_node_id: value.origin_node_id,
            story_tag: value.story_tag,
        }
    }
}

impl SaveDungeonState {
    fn from_resource(dungeon_state: Option<&CurrentDungeonState>) -> Self {
        dungeon_state.map(Self::from).unwrap_or_default()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SaveLoreJournalState {
    #[serde(default)]
    pub fragments: Vec<LoreFragment>,
    #[serde(default)]
    pub fragment_count: usize,
}

impl From<&LoreJournal> for SaveLoreJournalState {
    fn from(value: &LoreJournal) -> Self {
        Self {
            fragments: value.fragments.clone(),
            fragment_count: value.fragments.len(),
        }
    }
}

impl SaveLoreJournalState {
    fn from_journal(journal: Option<&LoreJournal>) -> Self {
        journal.map(Self::from).unwrap_or_default()
    }
}

/// Aggregated save data.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SaveGame {
    #[serde(default = "default_save_version")]
    pub version: u32,
    #[serde(default)]
    pub seed: u64,
    #[serde(default)]
    pub app_state: SaveAppState,
    #[serde(default)]
    pub game_time: SaveGameTime,
    #[serde(default)]
    pub player: SavePlayerState,
    #[serde(default)]
    pub colony: SaveColonyState,
    #[serde(default)]
    pub overworld: SaveOverworldState,
    #[serde(default)]
    pub dungeon: SaveDungeonState,
    #[serde(default)]
    pub gabriel: GabrielState,
    #[serde(default)]
    pub lore_journal: SaveLoreJournalState,
    #[serde(default, rename = "turn", skip_serializing)]
    legacy_turn: u32,
    #[serde(default, rename = "player_hp", skip_serializing)]
    legacy_player_hp: i32,
    #[serde(default, rename = "player_hp_max", skip_serializing)]
    legacy_player_hp_max: i32,
    #[serde(default, rename = "player_speed", skip_serializing)]
    legacy_player_speed: u8,
    #[serde(default, rename = "player_ar", skip_serializing)]
    legacy_player_ar: i32,
    #[serde(default, rename = "player_md", skip_serializing)]
    legacy_player_md: i32,
    #[serde(default, rename = "player_melee_skill", skip_serializing)]
    legacy_player_melee_skill: u32,
    #[serde(default, rename = "player_ranged_skill", skip_serializing)]
    legacy_player_ranged_skill: u32,
    #[serde(default, rename = "player_toughness_skill", skip_serializing)]
    legacy_player_toughness_skill: u32,
    #[serde(default, rename = "perks", skip_serializing)]
    legacy_perks: PlayerPerks,
    #[serde(default, rename = "resources", skip_serializing)]
    legacy_resources: ShelterResources,
    #[serde(default, rename = "lore_count", skip_serializing)]
    legacy_lore_count: usize,
    #[serde(default, rename = "inventory_count", skip_serializing)]
    legacy_inventory_count: usize,
}

impl Default for SaveGame {
    fn default() -> Self {
        Self {
            version: SAVE_VERSION,
            seed: 0,
            app_state: SaveAppState::default(),
            game_time: SaveGameTime::default(),
            player: SavePlayerState::default(),
            colony: SaveColonyState::default(),
            overworld: SaveOverworldState::default(),
            dungeon: SaveDungeonState::default(),
            gabriel: GabrielState::default(),
            lore_journal: SaveLoreJournalState::default(),
            legacy_turn: 0,
            legacy_player_hp: 0,
            legacy_player_hp_max: 0,
            legacy_player_speed: 0,
            legacy_player_ar: 0,
            legacy_player_md: 0,
            legacy_player_melee_skill: 0,
            legacy_player_ranged_skill: 0,
            legacy_player_toughness_skill: 0,
            legacy_perks: PlayerPerks::default(),
            legacy_resources: ShelterResources::default(),
            legacy_lore_count: 0,
            legacy_inventory_count: 0,
        }
    }
}

impl SaveGame {
    fn normalize_loaded(mut self) -> Self {
        if self.version == 0 {
            self.version = SAVE_VERSION;
        }
        self.apply_legacy_fields();
        if let Some(graph) = self.overworld.graph.as_mut() {
            graph.ensure_story_tags();
        }
        self.sync_derived_fields();
        self
    }

    fn apply_legacy_fields(&mut self) {
        if !self.has_legacy_flat_data() {
            return;
        }

        if self.app_state == SaveAppState::Menu {
            self.app_state = SaveAppState::Colony;
        }
        if self.game_time.turn == 0 {
            self.game_time.turn = self.legacy_turn;
        }
        if self.player.name.is_empty() {
            self.player.name = "Player".to_string();
        }
        if self.player.stats.hp == 0 {
            self.player.stats.hp = self.legacy_player_hp;
        }
        if self.player.stats.hp_max == 0 {
            self.player.stats.hp_max = self.legacy_player_hp_max;
        }
        if self.player.stats.speed == 0 {
            self.player.stats.speed = self.legacy_player_speed;
        }
        if self.player.stats.ar == 0 {
            self.player.stats.ar = self.legacy_player_ar;
        }
        if self.player.stats.md == 0 {
            self.player.stats.md = self.legacy_player_md;
        }

        insert_legacy_skill(
            &mut self.player.stats.skills,
            SkillId::Melee,
            self.legacy_player_melee_skill,
        );
        insert_legacy_skill(
            &mut self.player.stats.skills,
            SkillId::Ranged,
            self.legacy_player_ranged_skill,
        );
        insert_legacy_skill(
            &mut self.player.stats.skills,
            SkillId::Toughness,
            self.legacy_player_toughness_skill,
        );

        if self.player.perks.unlocked.is_empty() && !self.legacy_perks.unlocked.is_empty() {
            self.player.perks = self.legacy_perks.clone();
        }
        if self.colony_is_empty() && legacy_resources_present(&self.legacy_resources) {
            self.colony.resources = self.legacy_resources.clone();
        }

        self.player.inventory_slot_count = self
            .player
            .inventory_slot_count
            .max(self.legacy_inventory_count);
        self.lore_journal.fragment_count =
            self.lore_journal.fragment_count.max(self.legacy_lore_count);
    }

    fn sync_derived_fields(&mut self) {
        if self.player.progression.is_empty() {
            self.player.progression =
                PlayerProgression::from_legacy_skills(&self.player.stats.skills);
        } else {
            self.player.progression.ensure_complete();
        }
        if !self.has_legacy_flat_data() {
            self.player
                .progression
                .sync_pilot_combat_skill_proxies(&mut self.player.stats.skills);
        }
        if self.gabriel.joined {
            self.gabriel.encounter_completed = true;
        }
        let inventory_count = self
            .player
            .inventory
            .slots
            .iter()
            .filter(|slot| slot.is_some())
            .count();
        self.player.inventory_slot_count = self.player.inventory_slot_count.max(inventory_count);
        self.lore_journal.fragment_count = self
            .lore_journal
            .fragment_count
            .max(self.lore_journal.fragments.len());
    }

    fn has_legacy_flat_data(&self) -> bool {
        self.legacy_turn != 0
            || self.legacy_player_hp != 0
            || self.legacy_player_hp_max != 0
            || self.legacy_player_speed != 0
            || self.legacy_player_ar != 0
            || self.legacy_player_md != 0
            || self.legacy_player_melee_skill != 0
            || self.legacy_player_ranged_skill != 0
            || self.legacy_player_toughness_skill != 0
            || !self.legacy_perks.unlocked.is_empty()
            || legacy_resources_present(&self.legacy_resources)
            || self.legacy_lore_count != 0
            || self.legacy_inventory_count != 0
    }

    fn colony_is_empty(&self) -> bool {
        !legacy_resources_present(&self.colony.resources)
    }
}

fn default_save_version() -> u32 {
    SAVE_VERSION
}

fn insert_legacy_skill(skills: &mut HashMap<SkillId, SkillState>, skill: SkillId, base: u32) {
    if base == 0 || skills.contains_key(&skill) {
        return;
    }

    skills.insert(
        skill,
        SkillState {
            base,
            xp: 0,
            level: 0,
        },
    );
}

fn legacy_resources_present(resources: &ShelterResources) -> bool {
    resources.food != 0
        || resources.water != 0
        || resources.scrap != 0
        || resources.medicine != 0
        || resources.ammo != 0
}

fn save_path() -> PathBuf {
    PathBuf::from("./broken_divinity_save.json")
}

/// Check if a save file exists.
pub fn save_exists() -> bool {
    save_path().exists()
}

/// Load and normalize a save from disk if present.
pub fn load_game() -> Option<SaveGame> {
    let raw = std::fs::read_to_string(save_path()).ok()?;
    let save: SaveGame = serde_json::from_str(&raw).ok()?;
    Some(save.normalize_loaded())
}

pub fn load_game_detailed() -> Result<SaveGame, LoadGameError> {
    let raw = std::fs::read_to_string(save_path()).map_err(|_| LoadGameError::MissingSave)?;
    let save: SaveGame = serde_json::from_str(&raw).map_err(|_| LoadGameError::InvalidData)?;
    Ok(save.normalize_loaded())
}

pub fn load_success_message() -> String {
    "Load complete. Recap restored for the active run.".to_string()
}

pub fn load_error_message(error: LoadGameError) -> String {
    match error {
        LoadGameError::MissingSave => {
            "Load failed: no save file was found for this run.".to_string()
        }
        LoadGameError::InvalidData => {
            "Load failed: the save data could not be read safely.".to_string()
        }
    }
}

/// Queue a loaded save for future state restoration.
pub fn queue_loaded_game(commands: &mut Commands, save: SaveGame) {
    commands.insert_resource(PendingLoad(Some(save.normalize_loaded())));
}

fn write_save(save: &SaveGame) -> bool {
    let Ok(json) = serde_json::to_string_pretty(save) else {
        return false;
    };
    std::fs::write(save_path(), json).is_ok()
}

/// Collect all survivor entities into save data.
fn collect_survivors(
    survivors: &Query<(&EntityName, &Position, &SurvivorNeeds, &SurvivorTask), With<Survivor>>,
) -> Vec<SaveSurvivor> {
    survivors
        .iter()
        .map(|(name, pos, needs, task)| SaveSurvivor {
            name: name.name.clone(),
            x: pos.x,
            y: pos.y,
            hunger: needs.hunger,
            thirst: needs.thirst,
            rest: needs.rest,
            task: SaveSurvivorTask::from(task),
        })
        .collect()
}

/// Collect all station entities into save data.
fn collect_stations(stations: &Query<(&Station, &Position)>) -> Vec<SaveStation> {
    stations.iter().map(SaveStation::from).collect()
}

fn capture_colony_survivors_for_save(
    runtime_state: Option<&AppState>,
    survivors: &Query<(&EntityName, &Position, &SurvivorNeeds, &SurvivorTask), With<Survivor>>,
    pending_survivors: Option<&PendingSurvivorLoad>,
) -> Vec<SaveSurvivor> {
    let live = collect_survivors(survivors);
    if matches!(runtime_state, Some(AppState::Colony)) || !live.is_empty() {
        live
    } else {
        pending_survivors.map_or_else(Vec::new, |pending| pending.0.clone())
    }
}

fn capture_colony_stations_for_save(
    runtime_state: Option<&AppState>,
    stations: &Query<(&Station, &Position)>,
    pending_stations: Option<&PendingStationLoad>,
) -> Vec<SaveStation> {
    let live = collect_stations(stations);
    if matches!(runtime_state, Some(AppState::Colony)) || !live.is_empty() {
        live
    } else {
        pending_stations.map_or_else(Vec::new, |pending| pending.0.clone())
    }
}

/// Cache survivor/station state before the colony scene is torn down.
pub fn cache_colony_runtime_state(
    mut commands: Commands,
    survivors: Query<(&EntityName, &Position, &SurvivorNeeds, &SurvivorTask), With<Survivor>>,
    stations: Query<(&Station, &Position)>,
) {
    commands.insert_resource(PendingSurvivorLoad(collect_survivors(&survivors)));
    commands.insert_resource(PendingStationLoad(collect_stations(&stations)));
}

/// System: autosave when entering Colony state.
pub fn autosave(
    app_state: Option<Res<State<AppState>>>,
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
            &SprintCooldown,
        ),
        With<Player>,
    >,
    colony_queries: (
        Query<(&EntityName, &Position, &SurvivorNeeds, &SurvivorTask), With<Survivor>>,
        Query<(&Station, &Position)>,
    ),
    colony_resources: (
        Option<Res<GameTime>>,
        Option<Res<ShelterResources>>,
        Option<Res<ShelterState>>,
        Option<Res<CompletedResearch>>,
    ),
    world_resources: (
        Option<Res<WorldSeed>>,
        Option<Res<GabrielState>>,
        Option<Res<LoreJournal>>,
        Option<Res<WorldMap>>,
        Option<Res<PlayerMapPosition>>,
        Option<Res<Factions>>,
        Option<Res<TravelState>>,
        Option<Res<CurrentDungeonState>>,
    ),
    persistence_resources: (
        Option<Res<PendingSurvivorLoad>>,
        Option<Res<PendingStationLoad>>,
        Option<Res<PendingRaidReport>>,
    ),
) {
    let Ok((
        position,
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
    let (survivor_q, station_q) = colony_queries;
    let (time, resources, shelter_state, research) = colony_resources;
    let (
        seed,
        gabriel_state,
        journal,
        world_map,
        player_map_position,
        factions,
        travel,
        dungeon_state,
    ) = world_resources;
    let (pending_survivor_load, pending_station_load, pending_raid_report) = persistence_resources;
    let runtime_state = app_state.as_ref().map(|state| state.get());

    let mut save = SaveGame {
        version: SAVE_VERSION,
        seed: seed.as_ref().map_or(0, |seed| seed.0),
        app_state: app_state
            .as_ref()
            .map(|state| SaveAppState::from(state.get()))
            .unwrap_or_default(),
        game_time: SaveGameTime::from_time(time.as_deref()),
        player: SavePlayerState::from_snapshot(
            position,
            stats,
            inventory,
            equipment,
            ranged_state,
            sanity,
            perks,
            progression,
            name,
            sprint_cd.remaining,
        ),
        colony: SaveColonyState::from_resources(
            shelter_state.as_deref(),
            resources.as_deref(),
            capture_colony_survivors_for_save(
                runtime_state,
                &survivor_q,
                pending_survivor_load.as_deref(),
            ),
            capture_colony_stations_for_save(
                runtime_state,
                &station_q,
                pending_station_load.as_deref(),
            ),
            research.as_deref(),
            pending_raid_report.as_deref(),
        ),
        overworld: SaveOverworldState::from_resources(
            world_map.as_deref(),
            player_map_position.as_deref(),
            factions.as_deref(),
            travel.as_deref(),
        ),
        dungeon: SaveDungeonState::from_resource(dungeon_state.as_deref()),
        gabriel: gabriel_state
            .as_ref()
            .map(|state| (**state).clone())
            .unwrap_or_default(),
        lore_journal: SaveLoreJournalState::from_journal(journal.as_deref()),
        ..SaveGame::default()
    };

    save.sync_derived_fields();
    let _ = write_save(&save);
}

/// Save the current run and return to the menu when requested by the UI.
pub fn handle_save_and_quit(
    request: Option<ResMut<SaveAndQuitRequested>>,
    mut next_state: ResMut<NextState<AppState>>,
    log: Option<ResMut<GameLog>>,
    app_state: Option<Res<State<AppState>>>,
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
            &SprintCooldown,
        ),
        With<Player>,
    >,
    colony_queries: (
        Query<(&EntityName, &Position, &SurvivorNeeds, &SurvivorTask), With<Survivor>>,
        Query<(&Station, &Position)>,
    ),
    player_snapshot: Option<Res<PlayerSnapshot>>,
    colony_resources: (
        Option<Res<GameTime>>,
        Option<Res<ShelterResources>>,
        Option<Res<ShelterState>>,
        Option<Res<WorldSeed>>,
        Option<Res<CompletedResearch>>,
        Option<Res<PendingSurvivorLoad>>,
        Option<Res<PendingStationLoad>>,
        Option<Res<PendingRaidReport>>,
    ),
    world_resources: (
        Option<Res<GabrielState>>,
        Option<Res<LoreJournal>>,
        Option<Res<WorldMap>>,
        Option<Res<PlayerMapPosition>>,
        Option<Res<Factions>>,
        Option<Res<TravelState>>,
        Option<Res<CurrentDungeonState>>,
    ),
) {
    let Some(mut request) = request else { return };
    if !request.0 {
        return;
    }
    request.0 = false;

    let player = if let Ok((
        position,
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
    {
        snapshot_player_state(
            position,
            stats,
            inventory,
            equipment,
            ranged_state,
            sanity,
            perks,
            progression,
            name,
            sprint_cd.remaining,
        )
    } else if let Some(snapshot) = player_snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.0.clone())
    {
        snapshot
    } else if let Some(existing_save) = load_game() {
        existing_save.player
    } else {
        return;
    };
    let (survivor_q, station_q) = colony_queries;
    let (
        time,
        resources,
        shelter_state,
        seed,
        research,
        pending_survivor_load,
        pending_station_load,
        pending_raid_report,
    ) = colony_resources;
    let (gabriel_state, journal, world_map, player_map_position, factions, travel, dungeon_state) =
        world_resources;
    let runtime_state = app_state.as_ref().map(|state| state.get());

    let mut save = SaveGame {
        version: SAVE_VERSION,
        seed: seed.as_ref().map_or(0, |seed| seed.0),
        app_state: app_state
            .as_ref()
            .map(|state| SaveAppState::from(state.get()))
            .unwrap_or_default(),
        game_time: SaveGameTime::from_time(time.as_deref()),
        player,
        colony: SaveColonyState::from_resources(
            shelter_state.as_deref(),
            resources.as_deref(),
            capture_colony_survivors_for_save(
                runtime_state,
                &survivor_q,
                pending_survivor_load.as_deref(),
            ),
            capture_colony_stations_for_save(
                runtime_state,
                &station_q,
                pending_station_load.as_deref(),
            ),
            research.as_deref(),
            pending_raid_report.as_deref(),
        ),
        overworld: SaveOverworldState::from_resources(
            world_map.as_deref(),
            player_map_position.as_deref(),
            factions.as_deref(),
            travel.as_deref(),
        ),
        dungeon: SaveDungeonState::from_resource(dungeon_state.as_deref()),
        gabriel: gabriel_state
            .as_ref()
            .map(|state| (**state).clone())
            .unwrap_or_default(),
        lore_journal: SaveLoreJournalState::from_journal(journal.as_deref()),
        ..SaveGame::default()
    };

    save.sync_derived_fields();
    if write_save(&save) {
        let turn = time.as_ref().map_or(0, |time| time.turn);
        if let Some(mut log) = log {
            log.push(
                "Save complete. Progress saved; returning to menu.",
                crate::core::gamelog::LogColor::Status,
                turn,
            );
        }
        next_state.set(AppState::Menu);
    }
}

/// Delete the save file (for permadeath).
pub fn delete_save() {
    let _ = std::fs::remove_file(save_path());
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::core::items::ItemStack;
    use crate::core::perks::PerkId;
    use crate::game::colony::raids::{PendingRaidReport, RaidPhase, RaidReport, RaidReportOrigin};
    use crate::game::factions::{FactionArchetype, FactionDisposition};
    use crate::game::overworld::graphgen::{NodeType, OverworldNode, Road};
    use crate::game::overworld::weather::Weather;
    use bevy::ecs::system::RunSystemOnce;
    use std::time::Duration;

    fn sample_player_state() -> SavePlayerState {
        let mut inventory = Inventory::default();
        inventory.slots[0] = Some(ItemStack {
            item_id: "scrap".to_string(),
            quantity: 3,
        });
        inventory.slots[4] = Some(ItemStack {
            item_id: "ammo".to_string(),
            quantity: 6,
        });

        let mut skills = HashMap::new();
        skills.insert(
            SkillId::Melee,
            SkillState {
                base: 40,
                xp: 5,
                level: 1,
            },
        );
        skills.insert(
            SkillId::Ranged,
            SkillState {
                base: 30,
                xp: 15,
                level: 2,
            },
        );

        let mut perks = PlayerPerks::default();
        perks.unlock(PerkId::ThickSkin);

        let mut progression = PlayerProgression::new_game();
        progression.kleos = 18;

        SavePlayerState {
            name: "Player".to_string(),
            position: SavePosition { x: 7, y: 9 },
            stats: SaveCombatStats {
                hp: 46,
                hp_max: 50,
                speed: 1,
                ar: 5,
                md: 2,
                skills,
            },
            inventory,
            inventory_slot_count: 2,
            equipment: Equipment {
                weapon: Some("iron_pipe".to_string()),
                armor: Some("leather_jacket".to_string()),
                accessory: None,
            },
            ranged_state: SaveRangedState {
                clip_current: 4,
                clip_size: 6,
            },
            sanity: SaveSanityState {
                current: 35,
                max: 100,
            },
            perks,
            progression,
            sprint_cooldown: 0,
        }
    }

    fn sample_lore_journal(count: usize) -> SaveLoreJournalState {
        let mut fragments = Vec::new();
        for id in 0..count {
            fragments.push(LoreFragment {
                id,
                title: format!("Fragment {id}"),
                text: format!("Lore entry {id}"),
                category: crate::game::dungeon::lore::LoreCategory::Personal,
            });
        }

        SaveLoreJournalState {
            fragment_count: fragments.len(),
            fragments,
        }
    }

    fn roundtrip(save: SaveGame) -> SaveGame {
        let json = serde_json::to_string_pretty(&save).expect("serialize");
        let loaded: SaveGame = serde_json::from_str(&json).expect("deserialize");
        loaded.normalize_loaded()
    }

    #[test]
    fn test_normalize_loaded_infers_missing_player_progression() {
        let save = SaveGame {
            player: SavePlayerState {
                progression: PlayerProgression::default(),
                ..sample_player_state()
            },
            ..Default::default()
        };

        let loaded = save.normalize_loaded();

        assert_eq!(
            loaded
                .player
                .progression
                .proficiency_rating(crate::core::stats::ProficiencyId::MeleeTraining),
            14
        );
        assert_eq!(
            loaded
                .player
                .progression
                .virtue_rank(crate::core::stats::VirtueId::Thumos),
            3
        );
        assert_eq!(loaded.player.progression.kleos, 0);
    }

    #[test]
    fn test_normalize_loaded_syncs_pilot_skill_proxies_from_progression() {
        let mut save = SaveGame {
            player: SavePlayerState {
                progression: PlayerProgression::new_game(),
                ..sample_player_state()
            },
            ..Default::default()
        };
        save.player.stats.skills.insert(
            SkillId::Melee,
            SkillState {
                base: 99,
                xp: 7,
                level: 4,
            },
        );
        save.player.stats.skills.insert(
            SkillId::Ranged,
            SkillState {
                base: 88,
                xp: 9,
                level: 3,
            },
        );
        save.player.stats.skills.insert(
            SkillId::Evasion,
            SkillState {
                base: 77,
                xp: 11,
                level: 2,
            },
        );

        let loaded = save.normalize_loaded();

        assert_eq!(
            loaded.player.stats.skills[&SkillId::Melee].effective(),
            loaded.player.progression.action_rating(
                crate::core::stats::VirtueId::Thumos,
                crate::core::stats::ProficiencyId::MeleeTraining,
                0,
                0,
            ) as u32
        );
        assert_eq!(
            loaded.player.stats.skills[&SkillId::Ranged].effective(),
            loaded.player.progression.action_rating(
                crate::core::stats::VirtueId::Prudence,
                crate::core::stats::ProficiencyId::RangedTraining,
                0,
                0,
            ) as u32
        );
        assert_eq!(
            loaded.player.stats.skills[&SkillId::Evasion].effective(),
            loaded.player.progression.enemy_attack_dv() as u32
        );
        assert_eq!(loaded.player.stats.skills[&SkillId::Melee].xp, 0);
        assert_eq!(loaded.player.stats.skills[&SkillId::Ranged].xp, 0);
        assert_eq!(loaded.player.stats.skills[&SkillId::Evasion].xp, 0);
    }

    #[test]
    fn test_snapshot_player_state_syncs_pilot_skill_proxies_from_progression() {
        let mut runtime_stats = CombatStats {
            hp: 46,
            hp_max: 50,
            speed: 1,
            ar: 5,
            md: 2,
            skills: HashMap::from([
                (
                    SkillId::Melee,
                    SkillState {
                        base: 99,
                        xp: 7,
                        level: 4,
                    },
                ),
                (
                    SkillId::Ranged,
                    SkillState {
                        base: 88,
                        xp: 9,
                        level: 3,
                    },
                ),
                (
                    SkillId::Evasion,
                    SkillState {
                        base: 77,
                        xp: 11,
                        level: 2,
                    },
                ),
            ]),
        };
        let progression = PlayerProgression::new_game();
        let snapshot = snapshot_player_state(
            &Position::new(7, 9),
            &runtime_stats,
            &Inventory::default(),
            &Equipment::default(),
            &RangedWeaponState {
                clip_current: 0,
                clip_size: 0,
            },
            &RaidExposure::default(),
            &PlayerPerks::default(),
            &progression,
            Some(&EntityName {
                name: "Player".to_string(),
            }),
            0,
        );

        assert_eq!(
            snapshot.stats.skills[&SkillId::Melee].effective(),
            progression.action_rating(
                crate::core::stats::VirtueId::Thumos,
                crate::core::stats::ProficiencyId::MeleeTraining,
                0,
                0,
            ) as u32
        );
        assert_eq!(
            snapshot.stats.skills[&SkillId::Ranged].effective(),
            progression.action_rating(
                crate::core::stats::VirtueId::Prudence,
                crate::core::stats::ProficiencyId::RangedTraining,
                0,
                0,
            ) as u32
        );
        assert_eq!(
            snapshot.stats.skills[&SkillId::Evasion].effective(),
            progression.enemy_attack_dv() as u32
        );

        runtime_stats.skills.insert(
            SkillId::Melee,
            SkillState {
                base: 5,
                xp: 0,
                level: 0,
            },
        );
        assert_eq!(snapshot.stats.skills[&SkillId::Melee].effective(), 22);
    }

    #[test]
    fn test_colony_save_roundtrip() {
        let loaded = roundtrip(SaveGame {
            seed: 42,
            app_state: SaveAppState::Colony,
            game_time: SaveGameTime { turn: 12 },
            player: sample_player_state(),
            colony: SaveColonyState {
                shelter_seed: 0xC010_0001,
                resources: ShelterResources {
                    food: 14,
                    water: 9,
                    scrap: 27,
                    medicine: 4,
                    ammo: 18,
                },
                survivors: Vec::new(),
                stations: vec![SaveStation {
                    kind: StationType::Cook,
                    tier: 2,
                    worker_slots: 1,
                    workers_assigned: 1,
                    x: 4,
                    y: 6,
                }],
                research: CompletedResearch::default(),
                pending_raid_report: Some(PendingRaidReport {
                    origin: RaidReportOrigin::AwayAutoResolve,
                    report: RaidReport {
                        survivors_lost: 1,
                        raiders_killed: 3,
                        resources_stolen: 4,
                        stations_damaged: 0,
                    },
                }),
            },
            lore_journal: sample_lore_journal(1),
            ..SaveGame::default()
        });

        assert_eq!(loaded.seed, 42);
        assert_eq!(loaded.app_state, SaveAppState::Colony);
        assert_eq!(loaded.game_time.turn, 12);
        assert_eq!(loaded.colony.shelter_seed, 0xC010_0001);
        assert_eq!(loaded.colony.resources.scrap, 27);
        assert_eq!(loaded.colony.stations.len(), 1);
        assert_eq!(loaded.colony.stations[0].kind, StationType::Cook);
        assert_eq!(
            loaded
                .colony
                .pending_raid_report
                .as_ref()
                .map(|report| report.origin),
            Some(RaidReportOrigin::AwayAutoResolve)
        );
        assert_eq!(loaded.player.inventory_slot_count, 2);
        assert_eq!(loaded.player.equipment.weapon.as_deref(), Some("iron_pipe"));
        assert_eq!(loaded.lore_journal.fragment_count, 1);
    }

    #[test]
    fn test_overworld_save_roundtrip() {
        let loaded = roundtrip(SaveGame {
            seed: 77,
            app_state: SaveAppState::Overworld,
            player: sample_player_state(),
            overworld: SaveOverworldState {
                graph: Some(OverworldGraph {
                    nodes: vec![
                        OverworldNode {
                            id: 0,
                            node_type: NodeType::Shelter,
                            name: "The Shelter".to_string(),
                            x: 0.0,
                            y: 0.0,
                            discovered: true,
                            dungeon_theme: None,
                            story_tag: None,
                        },
                        OverworldNode {
                            id: 1,
                            node_type: NodeType::Dungeon,
                            name: "Sector 1".to_string(),
                            x: 3.0,
                            y: -2.0,
                            discovered: true,
                            dungeon_theme: Some(DungeonTheme::UrbanDecay),
                            story_tag: Some(DungeonStoryTag::GabrielIntro),
                        },
                    ],
                    roads: vec![Road {
                        from: 0,
                        to: 1,
                        distance: 4.2,
                    }],
                }),
                player_position: SaveOverworldPlayerPosition { current_node: 1 },
                factions: vec![Faction {
                    id: 2,
                    name: "The Collective".to_string(),
                    archetype: FactionArchetype::Commune,
                    disposition: FactionDisposition::Friendly,
                    home_node: Some(1),
                    description: "A cooperative of survivors.".to_string(),
                }],
                travel: Some(TravelState {
                    from_node: 0,
                    to_node: 1,
                    distance_remaining: 1.5,
                    day: 8,
                    current_weather: Weather::Fog,
                    world_seed: 77,
                    encounters_seen: 0,
                }),
            },
            ..SaveGame::default()
        });

        assert_eq!(loaded.app_state, SaveAppState::Overworld);
        assert_eq!(loaded.overworld.player_position.current_node, 1);
        assert_eq!(loaded.overworld.factions.len(), 1);
        assert_eq!(loaded.overworld.factions[0].name, "The Collective");
        assert_eq!(
            loaded.overworld.travel.as_ref().map(|travel| travel.day),
            Some(8)
        );
        assert_eq!(
            loaded
                .overworld
                .graph
                .as_ref()
                .map(|graph| graph.nodes.len()),
            Some(2)
        );
    }

    #[test]
    fn test_dungeon_save_roundtrip() {
        let mut player = sample_player_state();
        player.position = SavePosition { x: 18, y: 11 };
        player.sanity = SaveSanityState {
            current: 82,
            max: 100,
        };

        let loaded = roundtrip(SaveGame {
            seed: 99,
            app_state: SaveAppState::Dungeon,
            game_time: SaveGameTime { turn: 47 },
            player,
            dungeon: SaveDungeonState {
                floor_number: 3,
                max_floors: 5,
                seed: 0xD00D,
                theme: Some(DungeonTheme::Military),
                origin_node_id: Some(2),
                story_tag: Some(DungeonStoryTag::GabrielIntro),
            },
            gabriel: GabrielState {
                encounter_completed: true,
                joined: true,
            },
            lore_journal: sample_lore_journal(2),
            ..SaveGame::default()
        });

        assert_eq!(loaded.app_state, SaveAppState::Dungeon);
        assert_eq!(loaded.game_time.turn, 47);
        assert_eq!(loaded.player.position.x, 18);
        assert_eq!(loaded.player.sanity.current, 82);
        assert_eq!(loaded.dungeon.floor_number, 3);
        assert_eq!(loaded.dungeon.theme, Some(DungeonTheme::Military));
        assert_eq!(loaded.dungeon.origin_node_id, Some(2));
        assert_eq!(
            loaded.dungeon.story_tag,
            Some(DungeonStoryTag::GabrielIntro)
        );
        assert!(loaded.gabriel.joined);
        assert_eq!(loaded.lore_journal.fragment_count, 2);
    }

    #[test]
    fn test_legacy_flat_save_loads_into_nested_schema() {
        let legacy_json = r#"{
            "seed": 42,
            "turn": 10,
            "player_hp": 80,
            "player_hp_max": 100,
            "player_speed": 2,
            "player_ar": 3,
            "player_md": 1,
            "player_melee_skill": 40,
            "player_ranged_skill": 30,
            "player_toughness_skill": 35,
            "perks": { "unlocked": ["ThickSkin"] },
            "resources": {
                "food": 10,
                "water": 12,
                "scrap": 16,
                "medicine": 2,
                "ammo": 9
            },
            "lore_count": 2,
            "inventory_count": 3
        }"#;

        let loaded = serde_json::from_str::<SaveGame>(legacy_json)
            .expect("deserialize legacy save")
            .normalize_loaded();

        assert_eq!(loaded.seed, 42);
        assert_eq!(loaded.app_state, SaveAppState::Colony);
        assert_eq!(loaded.game_time.turn, 10);
        assert_eq!(loaded.player.stats.hp, 80);
        assert_eq!(loaded.player.stats.hp_max, 100);
        assert_eq!(loaded.player.inventory_slot_count, 3);
        assert_eq!(loaded.colony.resources.scrap, 16);
        assert_eq!(loaded.lore_journal.fragment_count, 2);
        assert_eq!(
            loaded
                .player
                .stats
                .skills
                .get(&SkillId::Melee)
                .map(|state| state.base),
            Some(40)
        );
        assert!(loaded.player.perks.has(PerkId::ThickSkin));
    }

    #[test]
    fn test_save_path_returns_pathbuf() {
        let path = save_path();
        assert!(path.to_str().is_some());
        assert!(path.to_str().unwrap().contains("broken_divinity_save"));
    }

    // --- Survivor save/load tests ---

    #[test]
    fn test_save_survivor_roundtrip() {
        let survivor = SaveSurvivor {
            name: "Marcus".to_string(),
            x: 5,
            y: 10,
            hunger: 60,
            thirst: 40,
            rest: 90,
            task: SaveSurvivorTask::Working { x: 3, y: 7 },
        };

        let json = serde_json::to_string(&survivor).unwrap();
        let loaded: SaveSurvivor = serde_json::from_str(&json).unwrap();

        assert_eq!(loaded.name, "Marcus");
        assert_eq!(loaded.x, 5);
        assert_eq!(loaded.y, 10);
        assert_eq!(loaded.hunger, 60);
        assert_eq!(loaded.thirst, 40);
        assert_eq!(loaded.rest, 90);
        assert!(matches!(
            loaded.task,
            SaveSurvivorTask::Working { x: 3, y: 7 }
        ));
    }

    #[test]
    fn test_save_station_roundtrip() {
        let station = SaveStation {
            kind: StationType::MilitiaTraining,
            tier: 2,
            worker_slots: 1,
            workers_assigned: 1,
            x: 11,
            y: 13,
        };

        let json = serde_json::to_string(&station).unwrap();
        let loaded: SaveStation = serde_json::from_str(&json).unwrap();

        assert_eq!(loaded.kind, StationType::MilitiaTraining);
        assert_eq!(loaded.tier, 2);
        assert_eq!(loaded.workers_assigned, 1);
        assert_eq!(loaded.x, 11);
        assert_eq!(loaded.y, 13);
    }

    #[test]
    fn test_transient_task_maps_to_idle() {
        assert!(matches!(
            SaveSurvivorTask::from(&SurvivorTask::Idle),
            SaveSurvivorTask::Idle
        ));
        assert!(matches!(
            SaveSurvivorTask::from(&SurvivorTask::SeekingFood),
            SaveSurvivorTask::Idle
        ));
        assert!(matches!(
            SaveSurvivorTask::from(&SurvivorTask::SeekingWater),
            SaveSurvivorTask::Idle
        ));
        assert!(matches!(
            SaveSurvivorTask::from(&SurvivorTask::Resting),
            SaveSurvivorTask::Idle
        ));
        assert!(matches!(
            SaveSurvivorTask::from(&SurvivorTask::Patrolling),
            SaveSurvivorTask::Idle
        ));

        let working = SaveSurvivorTask::from(&SurvivorTask::Working(IVec2::new(4, 8)));
        assert!(matches!(working, SaveSurvivorTask::Working { x: 4, y: 8 }));
    }

    #[test]
    fn test_save_colony_state_backward_compat() {
        // Old save format without survivors field
        let json = r#"{"shelter_seed": 42, "resources": {"food": 10, "water": 5, "scrap": 0, "medicine": 0, "ammo": 0}}"#;
        let colony: SaveColonyState = serde_json::from_str(json).unwrap();
        assert_eq!(colony.shelter_seed, 42);
        assert!(
            colony.survivors.is_empty(),
            "missing survivors should default to empty vec"
        );
        assert!(
            colony.stations.is_empty(),
            "missing stations should default to empty vec"
        );
        assert!(
            colony.pending_raid_report.is_none(),
            "missing pending raid report should default to none"
        );
    }

    #[test]
    fn test_save_colony_state_with_survivors() {
        let colony = SaveColonyState {
            shelter_seed: 99,
            resources: ShelterResources::default(),
            survivors: vec![
                SaveSurvivor {
                    name: "Elena".to_string(),
                    x: 6,
                    y: 5,
                    hunger: 70,
                    thirst: 55,
                    rest: 80,
                    task: SaveSurvivorTask::Idle,
                },
                SaveSurvivor {
                    name: "Jin".to_string(),
                    x: 3,
                    y: 7,
                    hunger: 30,
                    thirst: 20,
                    rest: 50,
                    task: SaveSurvivorTask::Working { x: 3, y: 7 },
                },
            ],
            stations: vec![SaveStation {
                kind: StationType::Workbench,
                tier: 1,
                worker_slots: 2,
                workers_assigned: 1,
                x: 8,
                y: 4,
            }],
            research: CompletedResearch::default(),
            pending_raid_report: Some(PendingRaidReport {
                origin: RaidReportOrigin::AwayAutoResolve,
                report: RaidReport {
                    survivors_lost: 0,
                    raiders_killed: 2,
                    resources_stolen: 1,
                    stations_damaged: 0,
                },
            }),
        };

        let json = serde_json::to_string(&colony).unwrap();
        let loaded: SaveColonyState = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.survivors.len(), 2);
        assert_eq!(loaded.survivors[0].name, "Elena");
        assert_eq!(loaded.survivors[1].hunger, 30);
        assert_eq!(loaded.stations.len(), 1);
        assert_eq!(loaded.stations[0].kind, StationType::Workbench);
        assert!(loaded.pending_raid_report.is_some());
        assert!(matches!(
            loaded.survivors[1].task,
            SaveSurvivorTask::Working { x: 3, y: 7 }
        ));
    }

    #[test]
    fn test_save_survivor_task_to_runtime() {
        let idle = SaveSurvivorTask::Idle;
        assert!(matches!(idle.to_runtime(), SurvivorTask::Idle));

        let working = SaveSurvivorTask::Working { x: 10, y: 20 };
        let rt = working.to_runtime();
        assert!(matches!(rt, SurvivorTask::Working(pos) if pos == IVec2::new(10, 20)));
    }

    #[test]
    fn test_reset_run_state_for_menu_clears_raid_and_timer_state() {
        let mut app = App::new();

        let mut colony_timer = ColonyTickTimer::default();
        colony_timer.0.tick(Duration::from_secs_f32(0.4));
        let mut travel_timer = TravelDayTimer::default();
        travel_timer.0.tick(Duration::from_secs_f32(0.6));

        app.insert_resource(GameTime { turn: 99 });
        app.insert_resource(GameLog::default());
        app.insert_resource(colony_timer);
        app.insert_resource(travel_timer);
        app.insert_resource(RaidChance {
            accumulated: 2.5,
            base_chance: 0.5,
            ticks_since_last_raid: 17,
        });
        app.insert_resource(ActiveRaid {
            raider_count: 4,
            raider_strength: 60,
            casualties: 1,
            resources_stolen: 3,
            phase: RaidPhase::Planning,
        });
        app.insert_resource(TravelState {
            from_node: 0,
            to_node: 1,
            distance_remaining: 2.0,
            day: 3,
            current_weather: Weather::Clear,
            world_seed: 7,
            encounters_seen: 1,
        });
        app.insert_resource(ShelterResources::new_game());
        app.add_systems(Update, reset_run_state_for_menu);

        app.update();

        assert_eq!(app.world().resource::<GameTime>().turn, 0);
        assert_eq!(
            app.world().resource::<RaidChance>().ticks_since_last_raid,
            0
        );
        assert_eq!(
            app.world().resource::<ColonyTickTimer>().0.elapsed_secs(),
            0.0
        );
        assert_eq!(
            app.world().resource::<TravelDayTimer>().0.elapsed_secs(),
            0.0
        );
        assert!(app.world().get_resource::<ActiveRaid>().is_none());
        assert!(app.world().get_resource::<TravelState>().is_none());
        assert!(app.world().get_resource::<ShelterResources>().is_none());
    }

    #[test]
    fn test_cache_colony_runtime_state_captures_pending_loads() {
        let mut world = World::new();
        world.spawn((
            Survivor,
            EntityName {
                name: "Marcus".to_string(),
            },
            Position::new(3, 4),
            SurvivorNeeds {
                hunger: 70,
                thirst: 65,
                rest: 80,
            },
            SurvivorTask::Working(IVec2::new(9, 7)),
        ));
        world.spawn((
            Station {
                kind: StationType::Cook,
                tier: 2,
                worker_slots: 1,
                workers_assigned: 1,
            },
            Position::new(9, 7),
        ));

        let _ = world.run_system_once(cache_colony_runtime_state);

        let survivors = &world.resource::<PendingSurvivorLoad>().0;
        let stations = &world.resource::<PendingStationLoad>().0;
        assert_eq!(survivors.len(), 1);
        assert_eq!(survivors[0].name, "Marcus");
        assert_eq!(stations.len(), 1);
        assert_eq!(stations[0].kind, StationType::Cook);
        assert_eq!(stations[0].workers_assigned, 1);
    }
}
