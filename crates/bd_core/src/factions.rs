//! Faction integration — reputation pools for 5 factions.

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_factions_are_defined() {
        assert_eq!(ALL_FACTIONS.len(), 5);
    }
}
