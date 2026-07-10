//! Status effects — DoTs, stuns, and other timed conditions.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::gamelog::{GameLog, LogColor};
use super::stats::{CombatStats, EntityName};
use super::turn::GameTime;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum StatusKind {
    /// DoT: deals (3 + hp_max/10) damage per tick, stacks up to 3.
    Wounded,
    /// Skip next action.
    Stunned,
}

impl StatusKind {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Wounded => "Wounded",
            Self::Stunned => "Stunned",
        }
    }

    pub fn max_stacks(&self) -> u8 {
        match self {
            Self::Wounded => 3,
            Self::Stunned => 1,
        }
    }
}

// ---------------------------------------------------------------------------
// Structs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct StatusEffect {
    pub kind: StatusKind,
    pub stacks: u8,
    pub remaining_turns: u32,
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct StatusEffects {
    pub effects: Vec<StatusEffect>,
}

impl StatusEffects {
    /// Add (or stack) a status effect, refreshing duration if already present.
    pub fn add(&mut self, kind: StatusKind, duration: u32) {
        if let Some(existing) = self.effects.iter_mut().find(|e| e.kind == kind) {
            existing.stacks = (existing.stacks + 1).min(kind.max_stacks());
            existing.remaining_turns = existing.remaining_turns.max(duration);
        } else {
            self.effects.push(StatusEffect {
                kind,
                stacks: 1,
                remaining_turns: duration,
            });
        }
    }

    pub fn has(&self, kind: StatusKind) -> bool {
        self.effects.iter().any(|e| e.kind == kind)
    }

    pub fn remove_expired(&mut self) {
        self.effects.retain(|e| e.remaining_turns > 0);
    }

    /// Decrement remaining_turns on every effect, then drop expired ones.
    pub fn tick(&mut self) {
        for effect in &mut self.effects {
            effect.remaining_turns = effect.remaining_turns.saturating_sub(1);
        }
        self.remove_expired();
    }

    pub fn wound_stacks(&self) -> u8 {
        self.effects
            .iter()
            .find(|e| e.kind == StatusKind::Wounded)
            .map_or(0, |e| e.stacks)
    }

    pub fn is_stunned(&self) -> bool {
        self.has(StatusKind::Stunned)
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Applies wound damage and ticks all status effects during `WorldTick`.
pub fn tick_status_effects(
    mut query: Query<(&mut StatusEffects, &mut CombatStats, Option<&EntityName>)>,
    mut log: ResMut<GameLog>,
    time: Res<GameTime>,
) {
    for (mut statuses, mut stats, name) in &mut query {
        let entity_name = name.map_or("Something", |n| n.name.as_str());

        // --- Wound damage ---
        let wound_stacks = statuses.wound_stacks();
        if wound_stacks > 0 {
            let damage = (3 + stats.hp_max / 10) * wound_stacks as i32;
            stats.hp -= damage;
            log.push(
                format!("{entity_name} takes {damage} bleeding damage"),
                LogColor::EnemyHit,
                time.turn,
            );
        }

        // --- Stun logging ---
        if statuses.is_stunned() {
            log.push(
                format!("{entity_name} is stunned"),
                LogColor::Status,
                time.turn,
            );
        }

        // --- Tick all effects ---
        statuses.tick();
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stacking_wounded() {
        let mut fx = StatusEffects::default();
        fx.add(StatusKind::Wounded, 5);
        fx.add(StatusKind::Wounded, 3);
        fx.add(StatusKind::Wounded, 4);
        // Should be at max 3 stacks, duration refreshed to max seen (5).
        assert_eq!(fx.wound_stacks(), 3);
        assert_eq!(fx.effects[0].remaining_turns, 5);

        // Fourth add should not exceed max stacks.
        fx.add(StatusKind::Wounded, 10);
        assert_eq!(fx.wound_stacks(), 3);
        assert_eq!(fx.effects[0].remaining_turns, 10);
    }

    #[test]
    fn test_tick_removes_expired() {
        let mut fx = StatusEffects::default();
        fx.add(StatusKind::Wounded, 1);
        fx.add(StatusKind::Stunned, 2);

        assert_eq!(fx.effects.len(), 2);

        fx.tick(); // Wounded 1→0 (removed), Stunned 2→1
        assert_eq!(fx.effects.len(), 1);
        assert_eq!(fx.effects[0].kind, StatusKind::Stunned);

        fx.tick(); // Stunned 1→0 (removed)
        assert!(fx.effects.is_empty());
    }

    #[test]
    fn test_stunned_max_one_stack() {
        let mut fx = StatusEffects::default();
        fx.add(StatusKind::Stunned, 3);
        fx.add(StatusKind::Stunned, 2);
        fx.add(StatusKind::Stunned, 1);

        assert!(fx.is_stunned());
        // Should remain at 1 stack regardless of how many times added.
        let stun = fx
            .effects
            .iter()
            .find(|e| e.kind == StatusKind::Stunned)
            .unwrap();
        assert_eq!(stun.stacks, 1);
        // Duration should be the max seen (3).
        assert_eq!(stun.remaining_turns, 3);
    }
}
