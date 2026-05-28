//! Perk system — passive modifiers unlocked at skill level thresholds.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::core::stats::CombatStats;
use crate::core::stats::PlayerProgression;
use crate::core::stats::ProficiencyId;
use crate::core::stats::SkillId;
use crate::core::stats::VirtueId;

/// All available perks in the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum PerkId {
    // Melee perks
    HeavySwing,   // T1 (level 3): +2 melee damage
    CleaveStrike, // T2 (level 6): chance to hit adjacent enemy
    Unstoppable,  // T3 (level 9): immune to stun during melee

    // Ranged perks
    SteadyAim,   // T1 (level 3): +10 ranged accuracy
    QuickReload, // T2 (level 6): reload costs 0 AP
    Deadeye,     // T3 (level 9): crits deal triple damage

    // Quiet movement perks
    GhostStep,        // T1 (level 3): +5 defense DV against enemy attacks
    FalseTrail,       // T2 (level 6): +10 defense DV against ranged enemy attacks
    AmbushExtraction, // T3 (level 9): enemy misses grant +1 extra Quiet Movement XP

    // Toughness perks
    ThickSkin,  // T1 (level 3): +2 AR
    SecondWind, // T2 (level 6): auto-heal 5 HP when below 20%
    IronWill,   // T3 (level 9): sanity exposure reduced by 50%
}

impl PerkId {
    pub fn name(&self) -> &'static str {
        match self {
            Self::HeavySwing => "Heavy Swing",
            Self::CleaveStrike => "Cleave Strike",
            Self::Unstoppable => "Unstoppable",
            Self::SteadyAim => "Steady Aim",
            Self::QuickReload => "Quick Reload",
            Self::Deadeye => "Deadeye",
            Self::GhostStep => "Ghost Step",
            Self::FalseTrail => "False Trail",
            Self::AmbushExtraction => "Ambush Extraction",
            Self::ThickSkin => "Thick Skin",
            Self::SecondWind => "Second Wind",
            Self::IronWill => "Iron Will",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::HeavySwing => "+2 melee damage",
            Self::CleaveStrike => "Chance to hit adjacent enemy on melee kill",
            Self::Unstoppable => "Immune to stun during melee attacks",
            Self::SteadyAim => "+10 ranged accuracy",
            Self::QuickReload => "Reload costs 0 AP",
            Self::Deadeye => "Critical hits deal triple damage",
            Self::GhostStep => "+5 DEF against enemy attacks",
            Self::FalseTrail => "+10 DEF against ranged enemy attacks",
            Self::AmbushExtraction => "Enemy misses grant +1 extra Quiet Movement XP",
            Self::ThickSkin => "+2 base armor rating",
            Self::SecondWind => "Auto-heal 5 HP when below 20%",
            Self::IronWill => "Sanity exposure reduced by 50%",
        }
    }

    pub fn skill(&self) -> SkillId {
        match self {
            Self::HeavySwing | Self::CleaveStrike | Self::Unstoppable => SkillId::Melee,
            Self::SteadyAim | Self::QuickReload | Self::Deadeye => SkillId::Ranged,
            Self::GhostStep | Self::FalseTrail | Self::AmbushExtraction => SkillId::Stealth,
            // Canonical lane: Fortitude + MeleeTraining. Return Melee as the
            // proficiency partner so callers see a consistent skill category.
            Self::ThickSkin | Self::SecondWind | Self::IronWill => SkillId::Melee,
        }
    }

    /// Required skill level to unlock this perk.
    pub fn required_level(&self) -> u32 {
        match self {
            Self::HeavySwing | Self::SteadyAim | Self::GhostStep | Self::ThickSkin => 3,
            Self::CleaveStrike | Self::QuickReload | Self::FalseTrail | Self::SecondWind => 6,
            Self::Unstoppable | Self::Deadeye | Self::AmbushExtraction | Self::IronWill => 9,
        }
    }

    /// Tier of this perk (1-3).
    pub fn tier(&self) -> u8 {
        match self.required_level() {
            3 => 1,
            6 => 2,
            _ => 3,
        }
    }

    pub fn lane_label(&self) -> &'static str {
        match self {
            Self::HeavySwing | Self::CleaveStrike | Self::Unstoppable => "Thumos + Melee Training",
            Self::SteadyAim | Self::QuickReload | Self::Deadeye => "Prudence + Ranged Training",
            Self::GhostStep | Self::FalseTrail | Self::AmbushExtraction => "Metis + Quiet Movement",
            Self::ThickSkin | Self::SecondWind | Self::IronWill => "Fortitude + Melee Training",
        }
    }
}

/// Component tracking which perks a player has unlocked.
#[derive(Component, Debug, Clone, Default, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct PlayerPerks {
    pub unlocked: Vec<PerkId>,
}

#[derive(Resource, Debug, Clone, Default, Reflect)]
#[reflect(Resource)]
pub struct PendingPerkChoices {
    pub pending: Vec<PerkId>,
}

impl PendingPerkChoices {
    pub fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }

    pub fn push_unique(&mut self, perk: PerkId) {
        if !self.pending.contains(&perk) {
            self.pending.push(perk);
        }
    }

    pub fn pop_next(&mut self) -> Option<PerkId> {
        if self.pending.is_empty() {
            None
        } else {
            Some(self.pending.remove(0))
        }
    }
}

impl PlayerPerks {
    pub fn has(&self, perk: PerkId) -> bool {
        self.unlocked.contains(&perk)
    }

    pub fn unlock(&mut self, perk: PerkId) {
        if !self.has(perk) {
            self.unlocked.push(perk);
        }
    }

    /// Get the total melee damage bonus from perks.
    pub fn melee_damage_bonus(&self) -> i32 {
        if self.has(PerkId::HeavySwing) { 2 } else { 0 }
    }

    /// Get the total ranged accuracy bonus from perks.
    pub fn ranged_accuracy_bonus(&self) -> i32 {
        if self.has(PerkId::SteadyAim) { 10 } else { 0 }
    }

    pub fn enemy_attack_dv_bonus(&self, is_ranged: bool) -> i32 {
        let mut bonus = 0;
        if self.has(PerkId::GhostStep) {
            bonus += 5;
        }
        if is_ranged && self.has(PerkId::FalseTrail) {
            bonus += 10;
        }
        bonus
    }

    pub fn enemy_attack_xp_bonus(&self, evaded: bool) -> u32 {
        if evaded && self.has(PerkId::AmbushExtraction) {
            1
        } else {
            0
        }
    }

    /// Get the total AR bonus from perks.
    pub fn ar_bonus(&self) -> i32 {
        if self.has(PerkId::ThickSkin) { 2 } else { 0 }
    }

    /// Whether sanity exposure is halved.
    pub fn sanity_reduction(&self) -> bool {
        self.has(PerkId::IronWill)
    }
}

pub fn queue_progression_lane_perks(
    pending: &mut PendingPerkChoices,
    perks: &PlayerPerks,
    virtue: VirtueId,
    progression: &PlayerProgression,
    proficiency: ProficiencyId,
) -> Vec<PerkId> {
    let thresholds: &[(PerkId, u8, u32)] = match (virtue, proficiency) {
        (VirtueId::Thumos, ProficiencyId::MeleeTraining) => &[
            (PerkId::HeavySwing, 2, 10),
            (PerkId::CleaveStrike, 3, 18),
            (PerkId::Unstoppable, 4, 24),
        ],
        (VirtueId::Prudence, ProficiencyId::RangedTraining) => &[
            (PerkId::SteadyAim, 2, 10),
            (PerkId::QuickReload, 3, 18),
            (PerkId::Deadeye, 4, 24),
        ],
        (VirtueId::Metis, ProficiencyId::QuietMovement) => &[
            (PerkId::GhostStep, 2, 10),
            (PerkId::FalseTrail, 3, 18),
            (PerkId::AmbushExtraction, 4, 24),
        ],
        // Endurance/survival lane: pain tolerance meets practical close-quarters training.
        (VirtueId::Fortitude, ProficiencyId::MeleeTraining) => &[
            (PerkId::ThickSkin, 2, 10),
            (PerkId::SecondWind, 3, 18),
            (PerkId::IronWill, 4, 24),
        ],
        _ => &[],
    };

    let virtue_rank = progression.virtue_rank(virtue);
    let proficiency_rating = progression.proficiency_rating(proficiency);
    let unlocked: Vec<_> = thresholds
        .iter()
        .filter_map(|(perk, required_virtue, required_rating)| {
            (virtue_rank >= *required_virtue
                && proficiency_rating >= *required_rating
                && !perks.has(*perk)
                && !pending.pending.contains(perk))
            .then_some(*perk)
        })
        .collect();

    for perk in &unlocked {
        pending.push_unique(*perk);
    }

    unlocked
}

pub fn apply_second_wind(stats: &mut CombatStats, perks: &PlayerPerks, previous_hp: i32) -> bool {
    if !perks.has(PerkId::SecondWind) || stats.hp <= 0 {
        return false;
    }

    let was_above_threshold = previous_hp.saturating_mul(5) > stats.hp_max;
    let is_below_threshold = stats.hp.saturating_mul(5) <= stats.hp_max;
    if was_above_threshold && is_below_threshold {
        stats.hp = (stats.hp + 5).min(stats.hp_max);
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::stats::{
        CombatStats, PlayerProgression, ProficiencyId, ProficiencyState, SkillState, VirtueId,
        VirtueState,
    };
    use std::collections::HashMap;

    fn make_stats(hp: i32, hp_max: i32) -> CombatStats {
        CombatStats {
            hp,
            hp_max,
            speed: 1,
            ar: 0,
            md: 0,
            skills: HashMap::<SkillId, SkillState>::new(),
        }
    }

    #[test]
    fn test_unlock_perk() {
        let mut perks = PlayerPerks::default();
        assert!(!perks.has(PerkId::HeavySwing));
        perks.unlock(PerkId::HeavySwing);
        assert!(perks.has(PerkId::HeavySwing));
    }

    #[test]
    fn test_no_duplicates() {
        let mut perks = PlayerPerks::default();
        perks.unlock(PerkId::SteadyAim);
        perks.unlock(PerkId::SteadyAim);
        assert_eq!(perks.unlocked.len(), 1);
    }

    #[test]
    fn test_damage_bonus() {
        let mut perks = PlayerPerks::default();
        assert_eq!(perks.melee_damage_bonus(), 0);
        perks.unlock(PerkId::HeavySwing);
        assert_eq!(perks.melee_damage_bonus(), 2);
    }

    #[test]
    fn test_enemy_attack_lane_bonuses() {
        let mut perks = PlayerPerks::default();
        perks.unlock(PerkId::GhostStep);
        perks.unlock(PerkId::FalseTrail);
        perks.unlock(PerkId::AmbushExtraction);

        assert_eq!(perks.enemy_attack_dv_bonus(false), 5);
        assert_eq!(perks.enemy_attack_dv_bonus(true), 15);
        assert_eq!(perks.enemy_attack_xp_bonus(true), 1);
        assert_eq!(perks.enemy_attack_xp_bonus(false), 0);
    }

    #[test]
    fn test_queue_progression_lane_perks_for_thumos_melee() {
        let perks = PlayerPerks::default();
        let mut pending = PendingPerkChoices::default();
        let mut progression = PlayerProgression::new_game();
        progression
            .virtues
            .insert(VirtueId::Thumos, VirtueState { rank: 3 });
        progression
            .proficiencies
            .insert(ProficiencyId::MeleeTraining, ProficiencyState::new(18));

        let unlocked = queue_progression_lane_perks(
            &mut pending,
            &perks,
            VirtueId::Thumos,
            &progression,
            ProficiencyId::MeleeTraining,
        );

        assert_eq!(unlocked, vec![PerkId::HeavySwing, PerkId::CleaveStrike]);
        assert_eq!(
            pending.pending,
            vec![PerkId::HeavySwing, PerkId::CleaveStrike]
        );
    }

    #[test]
    fn test_queue_progression_lane_perks_requires_both_thresholds() {
        const REQUIRED_VIRTUE_RANK: u8 = 2;
        const BELOW_REQUIRED_VIRTUE_RANK: u8 = REQUIRED_VIRTUE_RANK - 1;
        const REQUIRED_PROFICIENCY_RATING: u32 = 10;
        const BELOW_REQUIRED_PROFICIENCY_RATING: u32 = REQUIRED_PROFICIENCY_RATING - 1;

        let perks = PlayerPerks::default();

        let mut insufficient_virtue_progression = PlayerProgression::new_game();
        insufficient_virtue_progression.virtues.insert(
            VirtueId::Thumos,
            VirtueState {
                rank: BELOW_REQUIRED_VIRTUE_RANK,
            },
        );
        insufficient_virtue_progression.proficiencies.insert(
            ProficiencyId::MeleeTraining,
            ProficiencyState::new(REQUIRED_PROFICIENCY_RATING),
        );

        let mut insufficient_virtue_pending = PendingPerkChoices::default();
        let unlocked_with_low_virtue = queue_progression_lane_perks(
            &mut insufficient_virtue_pending,
            &perks,
            VirtueId::Thumos,
            &insufficient_virtue_progression,
            ProficiencyId::MeleeTraining,
        );

        assert!(unlocked_with_low_virtue.is_empty());
        assert!(insufficient_virtue_pending.pending.is_empty());

        let mut insufficient_training_progression = PlayerProgression::new_game();
        insufficient_training_progression.virtues.insert(
            VirtueId::Thumos,
            VirtueState {
                rank: REQUIRED_VIRTUE_RANK,
            },
        );
        insufficient_training_progression.proficiencies.insert(
            ProficiencyId::MeleeTraining,
            ProficiencyState::new(BELOW_REQUIRED_PROFICIENCY_RATING),
        );

        let mut insufficient_training_pending = PendingPerkChoices::default();
        let unlocked_with_low_training = queue_progression_lane_perks(
            &mut insufficient_training_pending,
            &perks,
            VirtueId::Thumos,
            &insufficient_training_progression,
            ProficiencyId::MeleeTraining,
        );

        assert!(unlocked_with_low_training.is_empty());
        assert!(insufficient_training_pending.pending.is_empty());
    }

    #[test]
    fn test_queue_progression_lane_perks_for_prudence_ranged() {
        let perks = PlayerPerks::default();
        let mut pending = PendingPerkChoices::default();
        let mut progression = PlayerProgression::new_game();
        progression
            .virtues
            .insert(VirtueId::Prudence, VirtueState { rank: 4 });
        progression
            .proficiencies
            .insert(ProficiencyId::RangedTraining, ProficiencyState::new(24));

        let unlocked = queue_progression_lane_perks(
            &mut pending,
            &perks,
            VirtueId::Prudence,
            &progression,
            ProficiencyId::RangedTraining,
        );

        assert_eq!(
            unlocked,
            vec![PerkId::SteadyAim, PerkId::QuickReload, PerkId::Deadeye]
        );
        assert_eq!(
            pending.pending,
            vec![PerkId::SteadyAim, PerkId::QuickReload, PerkId::Deadeye]
        );
    }

    #[test]
    fn test_queue_progression_lane_perks_for_metis_quiet_movement() {
        let perks = PlayerPerks::default();
        let mut pending = PendingPerkChoices::default();
        let mut progression = PlayerProgression::new_game();
        progression
            .virtues
            .insert(VirtueId::Metis, VirtueState { rank: 4 });
        progression
            .proficiencies
            .insert(ProficiencyId::QuietMovement, ProficiencyState::new(24));

        let unlocked = queue_progression_lane_perks(
            &mut pending,
            &perks,
            VirtueId::Metis,
            &progression,
            ProficiencyId::QuietMovement,
        );

        assert_eq!(
            unlocked,
            vec![
                PerkId::GhostStep,
                PerkId::FalseTrail,
                PerkId::AmbushExtraction
            ]
        );
        assert_eq!(
            pending.pending,
            vec![
                PerkId::GhostStep,
                PerkId::FalseTrail,
                PerkId::AmbushExtraction
            ]
        );
    }

    #[test]
    fn test_lane_labels_match_current_pilot_mapping() {
        assert_eq!(PerkId::HeavySwing.lane_label(), "Thumos + Melee Training");
        assert_eq!(PerkId::SteadyAim.lane_label(), "Prudence + Ranged Training");
        assert_eq!(PerkId::GhostStep.lane_label(), "Metis + Quiet Movement");
        assert_eq!(PerkId::ThickSkin.lane_label(), "Fortitude + Melee Training");
    }

    #[test]
    fn test_second_wind_triggers_on_threshold_cross() {
        let mut perks = PlayerPerks::default();
        perks.unlock(PerkId::SecondWind);
        let mut stats = make_stats(8, 50);

        assert!(apply_second_wind(&mut stats, &perks, 20));
        assert_eq!(stats.hp, 13);
    }

    #[test]
    fn test_toughness_lane_perks_skill_returns_melee() {
        // Toughness-lane perks are now unlocked via Fortitude+MeleeTraining.
        // skill() must reflect the canonical proficiency partner so the perk
        // choice panel and any future callers show a consistent category.
        assert_eq!(PerkId::ThickSkin.skill(), SkillId::Melee);
        assert_eq!(PerkId::SecondWind.skill(), SkillId::Melee);
        assert_eq!(PerkId::IronWill.skill(), SkillId::Melee);
    }

    // --- Phase 4: Fortitude + Melee Training canonical lane ---

    /// Threshold constants for the Fortitude+MeleeTraining perk lane.
    const FORTITUDE_T1_VIRTUE_RANK: u8 = 2;
    const FORTITUDE_T2_VIRTUE_RANK: u8 = 3;
    const FORTITUDE_T3_VIRTUE_RANK: u8 = 4;
    const FORTITUDE_T1_MELEE_RATING: u32 = 10;
    const FORTITUDE_T2_MELEE_RATING: u32 = 18;
    const FORTITUDE_T3_MELEE_RATING: u32 = 24;

    #[test]
    fn test_fortitude_melee_training_lane_unlocks_all_toughness_perks() {
        let perks = PlayerPerks::default();
        let mut pending = PendingPerkChoices::default();
        let mut progression = PlayerProgression::new_game();
        progression.virtues.insert(
            VirtueId::Fortitude,
            VirtueState {
                rank: FORTITUDE_T3_VIRTUE_RANK,
            },
        );
        progression.proficiencies.insert(
            ProficiencyId::MeleeTraining,
            ProficiencyState::new(FORTITUDE_T3_MELEE_RATING),
        );

        let unlocked = queue_progression_lane_perks(
            &mut pending,
            &perks,
            VirtueId::Fortitude,
            &progression,
            ProficiencyId::MeleeTraining,
        );

        assert_eq!(
            unlocked,
            vec![PerkId::ThickSkin, PerkId::SecondWind, PerkId::IronWill]
        );
        assert_eq!(
            pending.pending,
            vec![PerkId::ThickSkin, PerkId::SecondWind, PerkId::IronWill]
        );
    }

    #[test]
    fn test_fortitude_melee_training_lane_tier_gating() {
        let perks = PlayerPerks::default();

        // T1 threshold only
        let mut pending_t1 = PendingPerkChoices::default();
        let mut progression_t1 = PlayerProgression::new_game();
        progression_t1.virtues.insert(
            VirtueId::Fortitude,
            VirtueState {
                rank: FORTITUDE_T1_VIRTUE_RANK,
            },
        );
        progression_t1.proficiencies.insert(
            ProficiencyId::MeleeTraining,
            ProficiencyState::new(FORTITUDE_T1_MELEE_RATING),
        );
        let unlocked_t1 = queue_progression_lane_perks(
            &mut pending_t1,
            &perks,
            VirtueId::Fortitude,
            &progression_t1,
            ProficiencyId::MeleeTraining,
        );
        assert_eq!(unlocked_t1, vec![PerkId::ThickSkin]);

        // T2 threshold
        let mut pending_t2 = PendingPerkChoices::default();
        let mut progression_t2 = PlayerProgression::new_game();
        progression_t2.virtues.insert(
            VirtueId::Fortitude,
            VirtueState {
                rank: FORTITUDE_T2_VIRTUE_RANK,
            },
        );
        progression_t2.proficiencies.insert(
            ProficiencyId::MeleeTraining,
            ProficiencyState::new(FORTITUDE_T2_MELEE_RATING),
        );
        let unlocked_t2 = queue_progression_lane_perks(
            &mut pending_t2,
            &perks,
            VirtueId::Fortitude,
            &progression_t2,
            ProficiencyId::MeleeTraining,
        );
        assert_eq!(unlocked_t2, vec![PerkId::ThickSkin, PerkId::SecondWind]);
    }

    #[test]
    fn test_fortitude_melee_training_lane_requires_both_thresholds() {
        let perks = PlayerPerks::default();

        // Insufficient virtue rank, sufficient training
        let mut pending1 = PendingPerkChoices::default();
        let mut progression1 = PlayerProgression::new_game();
        progression1.virtues.insert(
            VirtueId::Fortitude,
            VirtueState {
                rank: FORTITUDE_T1_VIRTUE_RANK - 1,
            },
        );
        progression1.proficiencies.insert(
            ProficiencyId::MeleeTraining,
            ProficiencyState::new(FORTITUDE_T1_MELEE_RATING),
        );
        let unlocked1 = queue_progression_lane_perks(
            &mut pending1,
            &perks,
            VirtueId::Fortitude,
            &progression1,
            ProficiencyId::MeleeTraining,
        );
        assert!(
            unlocked1.is_empty(),
            "Should not unlock with insufficient virtue rank"
        );

        // Sufficient virtue rank, insufficient training
        let mut pending2 = PendingPerkChoices::default();
        let mut progression2 = PlayerProgression::new_game();
        progression2.virtues.insert(
            VirtueId::Fortitude,
            VirtueState {
                rank: FORTITUDE_T1_VIRTUE_RANK,
            },
        );
        progression2.proficiencies.insert(
            ProficiencyId::MeleeTraining,
            ProficiencyState::new(FORTITUDE_T1_MELEE_RATING - 1),
        );
        let unlocked2 = queue_progression_lane_perks(
            &mut pending2,
            &perks,
            VirtueId::Fortitude,
            &progression2,
            ProficiencyId::MeleeTraining,
        );
        assert!(
            unlocked2.is_empty(),
            "Should not unlock with insufficient melee training"
        );
    }

    #[test]
    fn test_toughness_perk_lane_label_is_fortitude_melee_training() {
        assert_eq!(PerkId::ThickSkin.lane_label(), "Fortitude + Melee Training");
        assert_eq!(
            PerkId::SecondWind.lane_label(),
            "Fortitude + Melee Training"
        );
        assert_eq!(PerkId::IronWill.lane_label(), "Fortitude + Melee Training");
    }
}
