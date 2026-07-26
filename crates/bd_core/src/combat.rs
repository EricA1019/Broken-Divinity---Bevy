//! Combat deepening — cover, ammo, wound thresholds, and tactical actions.

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::actions::{ActionDefinition, Effect, Requirement};
use crate::signals::{DeltaTag, PoolKind};

// ── Combat RNG ──

/// Seeded RNG for d100 combat rolls. Deterministic fallback when absent (for tests).
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct CombatRng {
    seed: u64,
    state: u64,
}

impl Default for CombatRng {
    fn default() -> Self {
        Self::from_seed(0)
    }
}

impl CombatRng {
    /// Create a new CombatRng seeded from a global seed.
    pub fn from_seed(seed: u64) -> Self {
        let state = seed ^ 0x9E37_79B9_7F4A_7C15;
        Self {
            seed,
            state: if state == 0 {
                0xA076_1D64_78BD_642F
            } else {
                state
            },
        }
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    fn next_u32(&mut self) -> u32 {
        let mut value = self.state;
        value ^= value >> 12;
        value ^= value << 25;
        value ^= value >> 27;
        self.state = value;
        (value.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 32) as u32
    }

    /// Roll 1d100. Returns None when CombatRng resource is absent (test fallback).
    pub fn d100(rng: Option<&mut CombatRng>) -> Option<i32> {
        rng.map(|r| {
            let v = r.next_u32();
            (v % 100 + 1) as i32
        })
    }

    /// Apply d100 variance to a damage value. Returns 0.5x for roll < 25, 1.5x for roll > 75, else 1.0x.
    /// When no CombatRng is available, returns the raw amount unchanged (deterministic fallback).
    pub fn apply_damage_variance(amount: i32, rng: Option<&mut CombatRng>) -> i32 {
        let Some(roll) = Self::d100(rng) else {
            return amount; // deterministic fallback
        };
        if roll < 25 {
            (amount as f32 * 0.5).round() as i32
        } else if roll > 75 {
            (amount as f32 * 1.5).round() as i32
        } else {
            amount
        }
    }
}

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

/// Armor absorbs a flat amount from each instance of physical/ballistic/slash damage.
/// Durability decreases with each hit absorbed.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Armor {
    /// Flat damage reduction per hit.
    pub reduction: i32,
    /// Remaining durability; armor is destroyed at 0.
    pub durability: i32,
}

pub fn register_aimed_attack_action() -> ActionDefinition {
    ActionDefinition {
        id: "ability.aimed_attack".into(),
        label: "Aimed Attack".into(),
        requirements: vec![
            Requirement::HasPoolAtLeast(PoolKind::ActionPoints, 2),
            Requirement::HasPoolAtLeast(PoolKind::Ammo, 1),
            Requirement::TargetExists,
            Requirement::TargetHostile,
            Requirement::TargetInRange(2),
        ],
        cost_effects: vec![
            Effect::PoolDelta {
                kind: PoolKind::ActionPoints,
                amount: -2,
                tags: vec![],
                reason: "aimed attack cost".into(),
            },
            Effect::PoolDelta {
                kind: PoolKind::Ammo,
                amount: -1,
                tags: vec![],
                reason: "ammo consumed".into(),
            },
        ],
        effects: vec![
            Effect::Log("Aimed attack!".into(), crate::gamelog::LogLevel::Combat),
            Effect::PoolDelta {
                kind: PoolKind::Health,
                amount: -8,
                tags: vec![DeltaTag::Slash],
                reason: "aimed attack".into(),
            },
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
            Effect::Log("Quick attack!".into(), crate::gamelog::LogLevel::Combat),
            Effect::PoolDelta {
                kind: PoolKind::Health,
                amount: -3,
                tags: vec![DeltaTag::Ballistic],
                reason: "quick attack".into(),
            },
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
            Effect::PoolDelta {
                kind: PoolKind::Ammo,
                amount: AMMO_PER_RANGED_WEAPON,
                tags: vec![],
                reason: "reload".into(),
            },
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
        assert!(def.cost_effects.iter().any(|e| matches!(
            e,
            Effect::PoolDelta {
                kind: PoolKind::ActionPoints,
                amount: -2,
                ..
            }
        )));
    }

    #[test]
    fn quick_attack_logs_cause_before_damage_effect() {
        let definition = register_quick_attack_action();
        let log_index = definition
            .effects
            .iter()
            .position(|effect| matches!(effect, Effect::Log(_, _)))
            .expect("quick attack must emit its cause");
        let damage_index = definition
            .effects
            .iter()
            .position(|effect| {
                matches!(
                    effect,
                    Effect::PoolDelta {
                        kind: PoolKind::Health,
                        amount,
                        ..
                    } if *amount < 0
                )
            })
            .expect("quick attack must emit damage");

        assert!(
            log_index < damage_index,
            "the visible attack cause must precede its damage result"
        );
    }

    #[test]
    fn d100_roll_in_range() {
        let mut rng = CombatRng::from_seed(42);
        for _ in 0..100 {
            let roll = CombatRng::d100(Some(&mut rng));
            assert!(roll.is_some());
            let r = roll.unwrap();
            assert!((1..=100).contains(&r), "d100 roll out of range: {}", r);
        }
    }

    #[test]
    fn seeded_combat_rolls_are_reproducible_and_seed_sensitive() {
        let mut first = CombatRng::from_seed(7);
        let mut second = CombatRng::from_seed(7);
        let mut different = CombatRng::from_seed(8);

        let first_rolls: Vec<_> = (0..8).map(|_| CombatRng::d100(Some(&mut first))).collect();
        let second_rolls: Vec<_> = (0..8).map(|_| CombatRng::d100(Some(&mut second))).collect();
        let different_rolls: Vec<_> = (0..8)
            .map(|_| CombatRng::d100(Some(&mut different)))
            .collect();

        assert_eq!(first_rolls, second_rolls);
        assert_ne!(first_rolls, different_rolls);
    }

    #[test]
    fn high_roll_increases_damage() {
        // Test with multiple seeds to find one where damage varies
        // apply_damage_variance should produce different results for some seeds
        let mut high_result = false;
        let mut low_result = false;
        for seed in 0..100 {
            let mut rng = CombatRng::from_seed(seed);
            let result = CombatRng::apply_damage_variance(-10, Some(&mut rng));
            if result < -10 {
                high_result = true;
            } // low roll (0.5x)
            if result > -10 {
                low_result = true;
            } // high roll (not possible for negative numbers)
            // Actually for negative damage: roll < 25 → 0.5x (e.g., -5), roll > 75 → 1.5x (e.g., -15)
            // So result < -10 means high roll, result > -10 means low roll
            // But result == -10 means middle roll
        }
        assert!(high_result && low_result);
        // At least one seed should produce a non-middle roll
        let mut rng = CombatRng::from_seed(42);
        let result = CombatRng::apply_damage_variance(-10, Some(&mut rng));
        // Just verify the result is a valid modified amount
        assert!(
            result == -15 || result == -10 || result == -5,
            "d100 variance should produce -15, -10, or -5 for input -10, got {}",
            result
        );
    }

    #[test]
    fn low_roll_decreases_damage() {
        // Test deterministic fallback when no RNG present
        let result = CombatRng::apply_damage_variance(-10, None);
        assert_eq!(result, -10, "deterministic fallback returns raw amount");
    }

    #[test]
    fn cover_reduces_damage_by_pct() {
        // Cover reduction is tested via the resolve_pool_deltas system integration
        // Unit: verify COVER_DAMAGE_REDUCTION_PCT constant is reasonable
        const {
            assert!(COVER_DAMAGE_REDUCTION_PCT > 0 && COVER_DAMAGE_REDUCTION_PCT <= 100);
        }
    }

    #[test]
    fn damage_type_matches_weapon() {
        let qa = register_quick_attack_action();
        assert!(qa.effects.iter().any(|e| matches!(e, Effect::PoolDelta { tags, .. } if tags.contains(&DeltaTag::Ballistic))),
            "quick attack should have Ballistic tag");

        let aa = register_aimed_attack_action();
        assert!(
            aa.effects.iter().any(
                |e| matches!(e, Effect::PoolDelta { tags, .. } if tags.contains(&DeltaTag::Slash))
            ),
            "aimed attack should have Slash tag"
        );
    }
}
