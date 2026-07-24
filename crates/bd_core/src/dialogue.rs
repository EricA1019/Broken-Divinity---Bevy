//! Dialogue engine — branching NPC conversations with conditions and effects.

use crate::actions::Effect;
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueNode {
    pub id: String,
    pub speaker: String,
    pub text: String,
    pub choices: Vec<Choice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    pub label: String,
    pub conditions: Vec<Condition>,
    pub effects: Vec<Effect>,
    pub next_node: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Condition {
    HasStatus(String),
    PoolAbove(String, i32),
    FactionReputationAbove(String, i32),
    VirtueAbove(String, i32),
    Always,
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct DialogueLog {
    pub seen_nodes: Vec<String>,
}

impl Default for DialogueLog {
    fn default() -> Self {
        Self {
            seen_nodes: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dialogue_log_starts_empty() {
        let log = DialogueLog::default();
        assert!(log.seen_nodes.is_empty());
    }

    #[test]
    fn condition_always_is_met() {
        assert!(matches!(Condition::Always, Condition::Always));
    }
}
