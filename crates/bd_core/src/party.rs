//! Party system — selecting survivors for expeditions.

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::actions::{ActionDefinition, Effect, Requirement};

pub const MAX_PARTY_SIZE: u32 = 4;

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct PartyMember;

#[derive(Resource, Debug, Clone, Default)]
pub struct PartyState {
    pub members: Vec<Entity>,
}


pub fn register_add_to_party_action() -> ActionDefinition {
    ActionDefinition {
        id: "ability.add_to_party".into(),
        label: "Add to Party".into(),
        requirements: vec![
            Requirement::TargetHasComponent("bd_core::colony::survivors::Survivor"),
            Requirement::PlayerHasEntityInRange(2),
        ],
        cost_effects: vec![],
        effects: vec![
            Effect::Log("Added to party.".into(), crate::gamelog::LogLevel::Info),
        ],
    }
}

pub fn register_remove_from_party_action() -> ActionDefinition {
    ActionDefinition {
        id: "ability.remove_from_party".into(),
        label: "Remove from Party".into(),
        requirements: vec![
            Requirement::TargetHasComponent("bd_core::colony::survivors::Survivor"),
            Requirement::PlayerHasEntityInRange(2),
        ],
        cost_effects: vec![],
        effects: vec![
            Effect::Log("Removed from party.".into(), crate::gamelog::LogLevel::Info),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn party_starts_empty() {
        let state = PartyState::default();
        assert!(state.members.is_empty());
    }

    #[test]
    fn max_party_size_is_four() {
        assert_eq!(MAX_PARTY_SIZE, 4);
    }
}
