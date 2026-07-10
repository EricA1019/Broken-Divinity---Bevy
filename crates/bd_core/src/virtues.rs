//! Virtue foundation — 6 classical virtues + Kleos (glory) tracker.

use crate::signals::PoolKind;

pub const VIRTUE_MAX: i32 = 100;
pub const FORTITUDE_COMBAT_SURVIVAL_GAIN: i32 = 5;
pub const FORTITUDE_BOSS_SURVIVAL_GAIN: i32 = 15;
pub const PRUDENCE_WISE_CHOICE_GAIN: i32 = 5;
pub const TEMPERANCE_CORRUPTION_RESIST_GAIN: i32 = 5;
pub const THUMOS_DECISIVE_ACTION_GAIN: i32 = 5;
pub const METIS_CUNNING_SOLUTION_GAIN: i32 = 5;
pub const JUSTICE_LAWFUL_RULING_GAIN: i32 = 5;

/// All virtue PoolKind variants.
pub const ALL_VIRTUES: &[PoolKind] = &[
    PoolKind::Temperance,
    PoolKind::Justice,
    PoolKind::Prudence,
    PoolKind::Fortitude,
    PoolKind::Thumos,
    PoolKind::Metis,
    PoolKind::Kleos,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_virtues_are_defined() {
        assert_eq!(ALL_VIRTUES.len(), 7);
    }
}
