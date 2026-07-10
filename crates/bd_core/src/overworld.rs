//! Overworld travel — nodes, roads, weather, and travel state.

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::actions::{ActionDefinition, Effect, Requirement};
use crate::signals::PoolKind;

pub const TRAVEL_FOOD_COST_PER_TURN: i32 = 1;

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
        requirements: vec![
            Requirement::PlayerHasEntityInRange(1),
        ],
        cost_effects: vec![Effect::PoolDelta {
            kind: PoolKind::Supplies,
            amount: -TRAVEL_FOOD_COST_PER_TURN,
            tags: vec![],
            reason: "travel cost".into(),
        }],
        effects: vec![
            Effect::Log("You begin traveling.".into(), crate::gamelog::LogLevel::Info),
        ],
    }
}



// ── Travel system ──

/// Process one turn of travel: decrement turns_remaining, deduct food.
/// When turns_remaining reaches 0, auto-transition to Tactical.
pub fn process_travel_day(
    mut state: ResMut<OverworldState>,
    mode: Res<crate::spatial::GameMode>,
    mut colony_res: ResMut<crate::colony::production::ColonyResources>,
    mut transition_writer: bevy_ecs::message::MessageWriter<crate::spatial::TransitionIntent>,
    mut game_log: ResMut<crate::gamelog::GameLog>,
) {
    if *mode != crate::spatial::GameMode::Travel {
        return;
    }
    if state.turns_remaining == 0 {
        return;
    }

    // Deduct food from colony resources
    if let Some(supplies) = colony_res.pools.get_mut(PoolKind::Supplies) {
        supplies.current = (supplies.current - TRAVEL_FOOD_COST_PER_TURN).max(0);
    }

    state.turns_remaining -= 1;
    game_log.push(
        format!("Travel: {} turn(s) remaining...", state.turns_remaining),
        crate::gamelog::LogLevel::Info,
    );

    if state.turns_remaining == 0 {
        // Arrived! Transition to tactical mode
        game_log.push("You have arrived at your destination.".to_string(), crate::gamelog::LogLevel::Info);
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
    let seed = time.turn.wrapping_mul(2654435761).wrapping_add(match &state.current_node {
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
            format!("{:?} weather increases supply consumption by {}.", state.weather, extra_cost),
            crate::gamelog::LogLevel::Info,
        );
    }
}

/// Register travel systems.
pub fn register_travel(app: &mut bevy_app::App) {
    app.add_systems(
        bevy_app::Update,
        (
            process_travel_day,
            process_travel_weather,
        ).in_set(crate::BdSet::Mutation),
    );
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        gamelog::GameLog,
        spatial::{GameMode, TransitionIntent},
        colony::production::ColonyResources,
        map::SmokeMap,
    };
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
        app.world_mut().insert_resource(SmokeMap::new(10, 10, crate::components::Tile::Floor));
        app.world_mut().insert_resource(GameMode::Travel);
        app.world_mut().resource_mut::<OverworldState>().turns_remaining = 3;
        app.update();
        let state = app.world().resource::<OverworldState>();
        assert_eq!(state.turns_remaining, 2, "Travel should decrement turns each frame");
    }

    #[test]
    fn travel_arrives_at_tactical() {
        let mut app = test_app();
        app.world_mut().insert_resource(SmokeMap::new(10, 10, crate::components::Tile::Floor));
        app.world_mut().insert_resource(GameMode::Travel);
        app.world_mut().resource_mut::<OverworldState>().turns_remaining = 1;
        // First update: process_travel_day writes TransitionIntent
        app.update();
        let state = app.world().resource::<OverworldState>();
        assert_eq!(state.turns_remaining, 0, "Travel should reach 0 on first update");
        // Second update: process_transitions processes the intent and changes mode
        app.update();
        let mode = app.world().resource::<GameMode>();
        assert_eq!(*mode, GameMode::Tactical, "Mode should become Tactical when travel arrives");
    }

    #[test]
    fn travel_deducts_food() {
        let mut app = test_app();
        app.world_mut().insert_resource(SmokeMap::new(10, 10, crate::components::Tile::Floor));
        app.world_mut().insert_resource(GameMode::Travel);
        app.world_mut().resource_mut::<OverworldState>().turns_remaining = 3;
        let supplies_before = app.world().resource::<ColonyResources>().pools.get(PoolKind::Supplies).unwrap().current;
        app.update();
        let supplies_after = app.world().resource::<ColonyResources>().pools.get(PoolKind::Supplies).unwrap().current;
        assert!(supplies_after < supplies_before, "Travel should deduct supplies (was={}, now={})", supplies_before, supplies_after);
    }
}
