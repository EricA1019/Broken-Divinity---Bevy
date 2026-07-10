//! Combat deepening — cover, ammo, wound thresholds, and tactical actions.

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::actions::{ActionDefinition, Effect, Requirement};
use crate::signals::PoolKind;

pub const COVER_DAMAGE_REDUCTION_PCT: i32 = 50;
pub const AMMO_PER_RANGED_WEAPON: i32 = 6;
pub const WOUND_THRESHOLD_PCT: i32 = 50;

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Cover {
    pub damage_reduction_pct: i32,
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Ammo {
    pub current: i32,
    pub max: i32,
}

pub fn register_aimed_attack_action() -> ActionDefinition {
    ActionDefinition {
        id: "ability.aimed_attack".into(),
        label: "Aimed Attack".into(),
        requirements: vec![
            Requirement::HasPoolAtLeast(PoolKind::ActionPoints, 2),
            Requirement::TargetExists,
            Requirement::TargetHostile,
            Requirement::TargetInRange(2),
        ],
        cost_effects: vec![Effect::PoolDelta {
            kind: PoolKind::ActionPoints,
            amount: -2,
            tags: vec![],
            reason: "aimed attack cost".into(),
        }],
        effects: vec![
            Effect::PoolDelta {
                kind: PoolKind::Health,
                amount: -8,
                tags: vec![],
                reason: "aimed attack".into(),
            },
            Effect::Log("Aimed attack!".into(), crate::gamelog::LogLevel::Combat),
        ],
    }
}

pub fn register_quick_attack_action() -> ActionDefinition {
    ActionDefinition {
        id: "ability.quick_attack".into(),
        label: "Quick Attack".into(),
        requirements: vec![
            Requirement::HasPoolAtLeast(PoolKind::ActionPoints, 1),
            Requirement::TargetExists,
            Requirement::TargetHostile,
            Requirement::TargetInRange(1),
        ],
        cost_effects: vec![Effect::PoolDelta {
            kind: PoolKind::ActionPoints,
            amount: -1,
            tags: vec![],
            reason: "quick attack cost".into(),
        }],
        effects: vec![
            Effect::PoolDelta {
                kind: PoolKind::Health,
                amount: -3,
                tags: vec![],
                reason: "quick attack".into(),
            },
            Effect::Log("Quick attack!".into(), crate::gamelog::LogLevel::Combat),
        ],
    }
}

pub fn register_reload_action() -> ActionDefinition {
    ActionDefinition {
        id: "ability.reload".into(),
        label: "Reload".into(),
        requirements: vec![Requirement::HasPoolAtLeast(PoolKind::ActionPoints, 1)],
        cost_effects: vec![Effect::PoolDelta {
            kind: PoolKind::ActionPoints,
            amount: -1,
            tags: vec![],
            reason: "reload cost".into(),
        }],
        effects: vec![
            Effect::Log("You reload.".into(), crate::gamelog::LogLevel::Info),
        ],
    }
}

pub fn register_take_cover_action() -> ActionDefinition {
    ActionDefinition {
        id: "ability.take_cover".into(),
        label: "Take Cover".into(),
        requirements: vec![Requirement::HasPoolAtLeast(PoolKind::ActionPoints, 1)],
        cost_effects: vec![Effect::PoolDelta {
            kind: PoolKind::ActionPoints,
            amount: -1,
            tags: vec![],
            reason: "take cover cost".into(),
        }],
        effects: vec![
            Effect::ApplyStatus("status.guarded".into()),
            Effect::Log("You take cover.".into(), crate::gamelog::LogLevel::Info),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aimed_attack_costs_two_ap() {
        let def = register_aimed_attack_action();
        assert!(def.cost_effects.iter().any(|e| matches!(e, Effect::PoolDelta { kind: PoolKind::ActionPoints, amount: -2, .. })));
    }
}
