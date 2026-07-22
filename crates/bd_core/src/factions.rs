//! Faction integration — reputation pools for 5 factions.

use std::collections::HashMap;

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::signals::PoolKind;

pub const REPUTATION_MAX: i32 = 100;
pub const REPUTATION_HOSTILE_THRESHOLD: i32 = 25;

/// All faction reputation PoolKind variants.
pub const ALL_FACTIONS: &[PoolKind] = &[
    PoolKind::RepPuritans,
    PoolKind::RepWanderers,
    PoolKind::RepBrokenChoir,
    PoolKind::RepDemons,
    PoolKind::RepHumanSettlements,
];

/// Resource tracking reputation with each faction.
/// Map key: faction PoolKind, value: current reputation.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct FactionReputation {
    pub values: HashMap<PoolKind, i32>,
}

impl Default for FactionReputation {
    fn default() -> Self {
        let mut values = HashMap::new();
        for faction in ALL_FACTIONS {
            values.insert(*faction, 0);
        }
        Self { values }
    }
}

impl FactionReputation {
    pub fn get(&self, faction: &PoolKind) -> i32 {
        self.values.get(faction).copied().unwrap_or(0)
    }

    pub fn modify(&mut self, faction: &PoolKind, delta: i32) {
        if let Some(val) = self.values.get_mut(faction) {
            *val = (*val + delta).clamp(-REPUTATION_MAX, REPUTATION_MAX);
        }
    }
}

/// Adjust faction reputation based on observed events.
/// For now: stub that only accepts direct delta requests.
/// Phase 2: observe dialogue choices, combat outcomes, raid results.
pub fn process_faction_events(
    _rep: ResMut<FactionReputation>,
    _game_log: ResMut<crate::gamelog::GameLog>,
) {
    // Stub: empty. Future: listen for EventSelected messages with faction effects.
}

pub fn register_factions(app: &mut bevy_app::App) {
    app.init_resource::<FactionReputation>();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_factions_are_defined() {
        assert_eq!(ALL_FACTIONS.len(), 5);
    }

    #[test]
    fn reputation_starts_at_zero() {
        let rep = FactionReputation::default();
        for faction in ALL_FACTIONS {
            assert_eq!(rep.get(faction), 0, "{:?} should start at 0", faction);
        }
    }

    #[test]
    fn reputation_can_be_modified() {
        let mut rep = FactionReputation::default();
        rep.modify(&PoolKind::RepPuritans, 10);
        assert_eq!(rep.get(&PoolKind::RepPuritans), 10);
        rep.modify(&PoolKind::RepPuritans, -5);
        assert_eq!(rep.get(&PoolKind::RepPuritans), 5);
    }

    #[test]
    fn reputation_clamps_at_max() {
        let mut rep = FactionReputation::default();
        rep.modify(&PoolKind::RepPuritans, REPUTATION_MAX + 50);
        assert_eq!(rep.get(&PoolKind::RepPuritans), REPUTATION_MAX);
    }

    #[test]
    fn reputation_clamps_at_min() {
        let mut rep = FactionReputation::default();
        rep.modify(&PoolKind::RepPuritans, -(REPUTATION_MAX + 50));
        assert_eq!(rep.get(&PoolKind::RepPuritans), -REPUTATION_MAX);
    }
}