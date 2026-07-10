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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

/// Register travel systems.
pub fn register_travel(app: &mut bevy_app::App) {
    app.add_systems(
        bevy_app::Update,
        process_travel_day.in_set(crate::BdSet::Mutation),
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
