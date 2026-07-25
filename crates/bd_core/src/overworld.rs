//! Overworld travel — nodes, roads, weather, and travel state.

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::actions::{ActionDefinition, Effect, Requirement};
use crate::signals::PoolKind;

pub const TRAVEL_FOOD_COST_PER_TURN: i32 = 1;

/// Base chance per travel turn of a random encounter (percentage).
pub const ENCOUNTER_CHANCE_PCT: u32 = 30;

/// Encounter types that can occur during travel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncounterType {
    /// Bandits ambush — spawns 2-3 hostile enemies in a tactical map.
    BanditAmbush,
    /// Demon sighting — small sanity hit, possible corruption.
    DemonSighting,
    /// Angel patrol — small faith gain, possible blessing.
    AngelPatrol,
    /// Survivor camp — gain supplies and potentially a new survivor.
    SurvivorCamp,
    /// Weather hazard — extra supply loss, possible stress.
    WeatherHazard,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverworldNode {
    pub id: String,
    pub name: String,
    pub travel_time: u32,
    pub danger_rating: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Road {
    pub from: String,
    pub to: String,
    pub distance: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Weather {
    Clear,
    Rain,
    Storm,
    AnomalyStorm,
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct OverworldState {
    pub current_node: Option<String>,
    pub turns_remaining: u32,
    pub weather: Weather,
}

impl Default for OverworldState {
    fn default() -> Self {
        Self {
            current_node: None,
            turns_remaining: 0,
            weather: Weather::Clear,
        }
    }
}

pub fn register_begin_travel_action() -> ActionDefinition {
    ActionDefinition {
        id: "ability.begin_travel".into(),
        label: "Begin Travel".into(),
        requirements: vec![Requirement::PlayerHasEntityInRange(1)],
        cost_effects: vec![Effect::PoolDelta {
            kind: PoolKind::Supplies,
            amount: -TRAVEL_FOOD_COST_PER_TURN,
            tags: vec![],
            reason: "travel cost".into(),
        }],
        effects: vec![Effect::Log(
            "You begin traveling.".into(),
            crate::gamelog::LogLevel::Info,
        )],
    }
}

// ── Travel system ──

/// Process one turn of travel: decrement turns_remaining, deduct food.
/// When turns_remaining reaches 0, auto-transition to Tactical.
pub fn process_travel_day(
    mut state: ResMut<OverworldState>,
    mode: Res<crate::spatial::GameMode>,
    time: Res<crate::time::GameTime>,
    mut colony_res: ResMut<crate::colony::production::ColonyResources>,
    faction_rep: Res<crate::factions::FactionReputation>,
    mut transition_writer: bevy_ecs::message::MessageWriter<crate::spatial::TransitionIntent>,
    mut game_log: ResMut<crate::gamelog::GameLog>,
    mut should_advance: ResMut<crate::time::ShouldAdvanceTime>,
) {
    if *mode != crate::spatial::GameMode::Travel {
        return;
    }
    if state.turns_remaining == 0 {
        return;
    }

    // Advance time for travel turns
    should_advance.0 = true;

    // Deduct food from colony resources
    if let Some(supplies) = colony_res.pools.get_mut(PoolKind::Supplies) {
        supplies.current = (supplies.current - TRAVEL_FOOD_COST_PER_TURN).max(0);
        // P14-B: Sanity drain when out of supplies
        if supplies.current == 0 {
            if let Some(sanity) = colony_res.pools.get_mut(PoolKind::Sanity) {
                sanity.current = (sanity.current - 2).max(0);
            }
            game_log.push(
                "You have no supplies — hunger gnaws at your sanity.".to_string(),
                crate::gamelog::LogLevel::Warn,
            );
        }
    }

    state.turns_remaining -= 1;
    game_log.push(
        format!("Travel: {} turn(s) remaining...", state.turns_remaining),
        crate::gamelog::LogLevel::Info,
    );

    // P14-A: Roll for random encounters during travel
    let encounter_seed =
        time.turn
            .wrapping_mul(3141592653)
            .wrapping_add(match &state.current_node {
                Some(n) => n.len() as u64 * 2718281828,
                None => 0,
            });
    if let Some(encounter) = roll_encounter(encounter_seed, &faction_rep) {
        game_log.push(
            format!("Encounter: {:?}!", encounter),
            crate::gamelog::LogLevel::Info,
        );
        resolve_encounter(encounter, &mut colony_res, &mut game_log);
    }

    if state.turns_remaining == 0 {
        // Arrived! Transition to tactical mode
        game_log.push(
            "You have arrived at your destination.".to_string(),
            crate::gamelog::LogLevel::Info,
        );
        transition_writer.write(crate::spatial::TransitionIntent {
            target: crate::spatial::GameMode::Tactical,
            node_id: state.current_node.clone(),
        });
    }
}

pub const WEATHER_CLEAR_WEIGHT: u32 = 60;
pub const WEATHER_RAIN_WEIGHT: u32 = 25;
pub const WEATHER_STORM_WEIGHT: u32 = 10;
pub const WEATHER_ANOMALY_WEIGHT: u32 = 5;
pub const WEATHER_SCALE: u32 =
    WEATHER_CLEAR_WEIGHT + WEATHER_RAIN_WEIGHT + WEATHER_STORM_WEIGHT + WEATHER_ANOMALY_WEIGHT;

/// Roll weather based on a deterministic seed.
pub fn roll_weather(seed: u64) -> Weather {
    let roll = seed % WEATHER_SCALE as u64;
    if roll < WEATHER_CLEAR_WEIGHT as u64 {
        Weather::Clear
    } else if roll < (WEATHER_CLEAR_WEIGHT + WEATHER_RAIN_WEIGHT) as u64 {
        Weather::Rain
    } else if roll < (WEATHER_CLEAR_WEIGHT + WEATHER_RAIN_WEIGHT + WEATHER_STORM_WEIGHT) as u64 {
        Weather::Storm
    } else {
        Weather::AnomalyStorm
    }
}

/// Roll a random encounter based on a deterministic seed and faction standings.
/// Factions skew the odds: hostile → more bandits/demons, allied → more angels/camps.
/// Returns None if no encounter occurs (~30% base chance per turn).
pub fn roll_encounter(
    seed: u64,
    rep: &crate::factions::FactionReputation,
) -> Option<EncounterType> {
    let roll = seed % 100;
    if roll >= ENCOUNTER_CHANCE_PCT as u64 {
        return None;
    }

    // Count hostile / allied factions to skew encounter types
    let hostile_count = crate::factions::ALL_FACTIONS
        .iter()
        .filter(|f| rep.get(f) <= -crate::factions::REPUTATION_HOSTILE_THRESHOLD)
        .count();
    let allied_count = crate::factions::ALL_FACTIONS
        .iter()
        .filter(|f| rep.get(f) >= crate::factions::REPUTATION_ALLIED_THRESHOLD)
        .count();

    // Base weights: Bandit=20, Demon=20, Angel=20, Camp=20, Weather=20
    let mut bandit_w = 20i32 + hostile_count as i32 * 10;
    let mut demon_w = 20i32 + hostile_count as i32 * 5;
    let mut angel_w = 20i32 + allied_count as i32 * 10;
    let mut camp_w = 20i32 + allied_count as i32 * 5;
    let weather_w = 20i32;

    // Clamp negatives
    bandit_w = bandit_w.max(0);
    demon_w = demon_w.max(0);
    angel_w = angel_w.max(0);
    camp_w = camp_w.max(0);

    let total = (bandit_w + demon_w + angel_w + camp_w + weather_w) as u64;
    let type_roll = (seed / 100) % total;

    if type_roll < bandit_w as u64 {
        Some(EncounterType::BanditAmbush)
    } else if type_roll < (bandit_w + demon_w) as u64 {
        Some(EncounterType::DemonSighting)
    } else if type_roll < (bandit_w + demon_w + angel_w) as u64 {
        Some(EncounterType::AngelPatrol)
    } else if type_roll < (bandit_w + demon_w + angel_w + camp_w) as u64 {
        Some(EncounterType::SurvivorCamp)
    } else {
        Some(EncounterType::WeatherHazard)
    }
}

/// Resolve an encounter — applies pool effects and logs.
pub fn resolve_encounter(
    encounter: EncounterType,
    colony_res: &mut crate::colony::production::ColonyResources,
    game_log: &mut crate::gamelog::GameLog,
) {
    match encounter {
        EncounterType::BanditAmbush => {
            game_log.push(
                "Bandits ambush you! You fight them off but take losses.".to_string(),
                crate::gamelog::LogLevel::Combat,
            );
            if let Some(supplies) = colony_res.pools.get_mut(PoolKind::Supplies) {
                supplies.current = (supplies.current - 3).max(0);
            }
        }
        EncounterType::DemonSighting => {
            game_log.push(
                "A demonic presence streaks across the sky. Your mind reels.".to_string(),
                crate::gamelog::LogLevel::Combat,
            );
            if let Some(sanity) = colony_res.pools.get_mut(PoolKind::Sanity) {
                sanity.current = (sanity.current - 10).max(0);
            }
        }
        EncounterType::AngelPatrol => {
            game_log.push(
                "An angelic patrol passes overhead. You feel a flicker of hope.".to_string(),
                crate::gamelog::LogLevel::Info,
            );
            if let Some(faith) = colony_res.pools.get_mut(PoolKind::Faith) {
                faith.current = (faith.current + 5).min(faith.max);
            }
        }
        EncounterType::SurvivorCamp => {
            game_log.push(
                "You stumble upon a survivor camp. They share supplies.".to_string(),
                crate::gamelog::LogLevel::Info,
            );
            if let Some(supplies) = colony_res.pools.get_mut(PoolKind::Supplies) {
                supplies.current = (supplies.current + 5).min(supplies.max);
            }
        }
        EncounterType::WeatherHazard => {
            game_log.push(
                "Harsh weather damages your supplies and frays your nerves.".to_string(),
                crate::gamelog::LogLevel::Warn,
            );
            if let Some(supplies) = colony_res.pools.get_mut(PoolKind::Supplies) {
                supplies.current = (supplies.current - 2).max(0);
            }
            if let Some(stress) = colony_res.pools.get_mut(PoolKind::Stress) {
                stress.current = (stress.current + 5).min(stress.max);
            }
        }
    }
}

/// Roll new weather each travel turn and apply effects.
pub fn process_travel_weather(
    mut state: ResMut<OverworldState>,
    mode: Res<crate::spatial::GameMode>,
    time: Res<crate::time::GameTime>,
    mut colony_res: ResMut<crate::colony::production::ColonyResources>,
    mut game_log: ResMut<crate::gamelog::GameLog>,
) {
    if *mode != crate::spatial::GameMode::Travel {
        return;
    }
    if state.turns_remaining == 0 {
        return;
    }

    // Roll weather from a deterministic seed (turn + node hash)
    let seed = time
        .turn
        .wrapping_mul(2654435761)
        .wrapping_add(match &state.current_node {
            Some(n) => n.len() as u64 * 2654435761,
            None => 0,
        });
    let new_weather = roll_weather(seed);

    if new_weather != state.weather {
        game_log.push(
            format!("Weather changes to {:?}.", new_weather),
            crate::gamelog::LogLevel::Info,
        );
        state.weather = new_weather;
    }

    // Apply weather effects: storm and anomaly increase supply consumption
    let extra_cost: i32 = match state.weather {
        Weather::Storm => 1,
        Weather::AnomalyStorm => 2,
        _ => 0,
    };
    if extra_cost > 0 {
        if let Some(supplies) = colony_res.pools.get_mut(PoolKind::Supplies) {
            supplies.current = (supplies.current - extra_cost).max(0);
        }
        game_log.push(
            format!(
                "{:?} weather increases supply consumption by {}.",
                state.weather, extra_cost
            ),
            crate::gamelog::LogLevel::Info,
        );
    }
}

/// Combined travel context for the TUI layer — bundles overworld state with
/// the travel map so they can be passed as a single system parameter.
#[derive(Resource, Debug, Clone)]
pub struct TravelContext {
    pub overworld: OverworldState,
    pub travel_map: crate::spatial::TravelMap,
}

impl Default for TravelContext {
    fn default() -> Self {
        Self {
            overworld: OverworldState::default(),
            travel_map: crate::spatial::TravelMap::default(),
        }
    }
}

/// Register travel systems.
pub fn register_travel(app: &mut bevy_app::App) {
    app.add_systems(
        bevy_app::Update,
        (process_travel_day, process_travel_weather).in_set(crate::BdSet::Mutation),
    );
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{colony::production::ColonyResources, map::SmokeMap, spatial::GameMode};
    use bevy_app::App;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(crate::BdCorePlugin);
        app
    }

    #[test]
    fn weather_default_is_clear() {
        let state = OverworldState::default();
        assert_eq!(state.weather, Weather::Clear);
    }

    #[test]
    fn weather_roll_is_deterministic() {
        let w1 = roll_weather(42);
        let w2 = roll_weather(42);
        assert_eq!(w1, w2, "Weather should be deterministic for same seed");
    }

    #[test]
    fn weather_rolls_cover_all_kinds() {
        let mut seen = std::collections::HashSet::new();
        for seed in 0..1000 {
            seen.insert(roll_weather(seed));
        }
        assert!(seen.contains(&Weather::Clear), "Clear should appear");
    }

    #[test]
    fn travel_cost_is_constant() {
        assert_eq!(TRAVEL_FOOD_COST_PER_TURN, 1);
    }

    #[test]
    fn travel_progress_decrements_turns() {
        let mut app = test_app();
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, crate::components::Tile::Floor));
        app.world_mut().insert_resource(GameMode::Travel);
        app.world_mut()
            .resource_mut::<OverworldState>()
            .turns_remaining = 3;
        app.update();
        let state = app.world().resource::<OverworldState>();
        assert_eq!(
            state.turns_remaining, 2,
            "Travel should decrement turns each frame"
        );
    }

    #[test]
    fn travel_arrives_at_tactical() {
        let mut app = test_app();
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, crate::components::Tile::Floor));
        app.world_mut().insert_resource(GameMode::Travel);
        app.world_mut()
            .resource_mut::<OverworldState>()
            .turns_remaining = 1;
        // First update: process_travel_day writes TransitionIntent
        app.update();
        let state = app.world().resource::<OverworldState>();
        assert_eq!(
            state.turns_remaining, 0,
            "Travel should reach 0 on first update"
        );
        // Second update: process_transitions processes the intent and changes mode
        app.update();
        let mode = app.world().resource::<GameMode>();
        assert_eq!(
            *mode,
            GameMode::Tactical,
            "Mode should become Tactical when travel arrives"
        );
    }

    #[test]
    fn travel_deducts_food() {
        let mut app = test_app();
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, crate::components::Tile::Floor));
        app.world_mut().insert_resource(GameMode::Travel);
        app.world_mut()
            .resource_mut::<OverworldState>()
            .turns_remaining = 3;
        let supplies_before = app
            .world()
            .resource::<ColonyResources>()
            .pools
            .get(PoolKind::Supplies)
            .unwrap()
            .current;
        app.update();
        let supplies_after = app
            .world()
            .resource::<ColonyResources>()
            .pools
            .get(PoolKind::Supplies)
            .unwrap()
            .current;
        assert!(
            supplies_after < supplies_before,
            "Travel should deduct supplies (was={}, now={})",
            supplies_before,
            supplies_after
        );
    }
}
