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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weather_default_is_clear() {
        let state = OverworldState::default();
        assert_eq!(state.weather, Weather::Clear);
    }

    #[test]
    fn travel_cost_is_constant() {
        assert_eq!(TRAVEL_FOOD_COST_PER_TURN, 1);
    }
}
