//! Faction integration — reputation pools for 5 factions.

use std::collections::HashMap;

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::BdSet;
use crate::signals::{PoolDeltaApplied, PoolKind};

pub const REPUTATION_MAX: i32 = 100;
pub const REPUTATION_HOSTILE_THRESHOLD: i32 = 25;
pub const REPUTATION_ALLIED_THRESHOLD: i32 = 75;

/// All faction reputation PoolKind variants.
pub const ALL_FACTIONS: &[PoolKind] = &[
    PoolKind::RepPuritans,
    PoolKind::RepWanderers,
    PoolKind::RepBrokenChoir,
    PoolKind::RepDemons,
    PoolKind::RepHumanSettlements,
];

/// Relationship standing with a faction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FactionStatus {
    Hostile,  // ≤ -REPUTATION_HOSTILE_THRESHOLD
    Neutral,  // between thresholds
    Friendly, // ≥ REPUTATION_HOSTILE_THRESHOLD, < REPUTATION_ALLIED_THRESHOLD
    Allied,   // ≥ REPUTATION_ALLIED_THRESHOLD
}

/// Determine faction status from a reputation value.
pub fn faction_status(value: i32) -> FactionStatus {
    if value <= -REPUTATION_HOSTILE_THRESHOLD {
        FactionStatus::Hostile
    } else if value >= REPUTATION_ALLIED_THRESHOLD {
        FactionStatus::Allied
    } else if value >= REPUTATION_HOSTILE_THRESHOLD {
        FactionStatus::Friendly
    } else {
        FactionStatus::Neutral
    }
}

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

/// Returns true if the PoolKind is a faction reputation pool.
pub fn is_faction_pool(kind: &PoolKind) -> bool {
    matches!(
        kind,
        PoolKind::RepPuritans
            | PoolKind::RepWanderers
            | PoolKind::RepBrokenChoir
            | PoolKind::RepDemons
            | PoolKind::RepHumanSettlements
    )
}

/// Intercepts PoolDeltaApplied messages and reroutes faction pool deltas to
/// the FactionReputation resource instead of player entity pools.
pub fn process_faction_events(
    mut rep: ResMut<FactionReputation>,
    mut applied: bevy_ecs::message::MessageReader<PoolDeltaApplied>,
    mut game_log: ResMut<crate::gamelog::GameLog>,
) {
    for msg in applied.read() {
        if is_faction_pool(&msg.kind) {
            rep.modify(&msg.kind, msg.amount_applied);
            let status = faction_status(rep.get(&msg.kind));
            game_log.push(
                format!(
                    "{:?} reputation: {} ({:?})",
                    msg.kind,
                    rep.get(&msg.kind),
                    status
                ),
                crate::gamelog::LogLevel::Info,
            );
        }
    }
}

pub fn register_factions(app: &mut bevy_app::App) {
    app.init_resource::<FactionReputation>();
    app.add_systems(
        bevy_app::Update,
        process_faction_events.in_set(BdSet::ResultEmission),
    );
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

    #[test]
    fn faction_status_hostile_below_threshold() {
        assert_eq!(
            faction_status(-REPUTATION_HOSTILE_THRESHOLD),
            FactionStatus::Hostile
        );
        assert_eq!(faction_status(-50), FactionStatus::Hostile);
    }

    #[test]
    fn faction_status_allied_above_threshold() {
        assert_eq!(
            faction_status(REPUTATION_ALLIED_THRESHOLD),
            FactionStatus::Allied
        );
        assert_eq!(faction_status(90), FactionStatus::Allied);
    }

    #[test]
    fn faction_status_neutral_in_middle() {
        assert_eq!(faction_status(0), FactionStatus::Neutral);
        assert_eq!(faction_status(10), FactionStatus::Neutral);
        assert_eq!(faction_status(-10), FactionStatus::Neutral);
    }

    #[test]
    fn faction_status_friendly_between_thresholds() {
        assert_eq!(faction_status(25), FactionStatus::Friendly);
        assert_eq!(faction_status(50), FactionStatus::Friendly);
    }

    #[test]
    fn is_faction_pool_identifies_all_five() {
        assert!(is_faction_pool(&PoolKind::RepPuritans));
        assert!(is_faction_pool(&PoolKind::RepWanderers));
        assert!(is_faction_pool(&PoolKind::RepBrokenChoir));
        assert!(is_faction_pool(&PoolKind::RepDemons));
        assert!(is_faction_pool(&PoolKind::RepHumanSettlements));
        assert!(!is_faction_pool(&PoolKind::Health));
        assert!(!is_faction_pool(&PoolKind::ActionPoints));
    }
}
