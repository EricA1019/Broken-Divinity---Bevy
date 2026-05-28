use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum SkillId {
    // Canonical pilot proxies — kept in sync by sync_pilot_combat_skill_proxies.
    Melee,
    Ranged,
    Evasion,
    // Legacy-compat only: retained for serde deserialization of old saves.
    // Toughness → Fortitude virtue + MeleeTraining proficiency (canonical).
    Toughness,
    // Legacy-compat only: retained for serde deserialization of old saves.
    // Stealth → Metis virtue + QuietMovement proficiency (canonical).
    Stealth,
    // Legacy-compat only: retained for serde deserialization of old saves.
    // Awareness → Prudence virtue (canonical).
    Awareness,
    // Legacy-compat only: retained for serde deserialization of old saves.
    Repair,
    // Legacy-compat only: retained for serde deserialization of old saves.
    Leadership,
}

impl SkillId {
    pub fn pilot_proxies() -> &'static [Self] {
        &[Self::Melee, Self::Ranged, Self::Evasion]
    }

    /// Returns all skill variants, including legacy-compat ones.
    /// This is intended for legacy compatibility (e.g., save/load) and should not be used for new-game initialization.
    /// For canonical pilot proxies, use `SkillId::pilot_proxies()` instead.
    pub fn all() -> &'static [Self] {
        &[
            Self::Melee,
            Self::Ranged,
            Self::Evasion,
            Self::Toughness,
            Self::Stealth,
            Self::Awareness,
            Self::Repair,
            Self::Leadership,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Melee => "Melee",
            Self::Ranged => "Ranged",
            Self::Evasion => "Evasion",
            Self::Toughness => "Toughness",
            Self::Stealth => "Stealth",
            Self::Awareness => "Awareness",
            Self::Repair => "Repair",
            Self::Leadership => "Leadership",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum VirtueId {
    Temperance,
    Justice,
    Prudence,
    Fortitude,
    Thumos,
    Metis,
}

impl VirtueId {
    pub fn all() -> &'static [Self] {
        &[
            Self::Temperance,
            Self::Justice,
            Self::Prudence,
            Self::Fortitude,
            Self::Thumos,
            Self::Metis,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Temperance => "Temperance",
            Self::Justice => "Justice",
            Self::Prudence => "Prudence",
            Self::Fortitude => "Fortitude",
            Self::Thumos => "Thumos",
            Self::Metis => "Metis",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum ProficiencyId {
    MeleeTraining,
    RangedTraining,
    QuietMovement,
    Repair,
    Medicine,
    Ritecraft,
}

impl ProficiencyId {
    pub fn all() -> &'static [Self] {
        &[
            Self::MeleeTraining,
            Self::RangedTraining,
            Self::QuietMovement,
            Self::Repair,
            Self::Medicine,
            Self::Ritecraft,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::MeleeTraining => "Melee Training",
            Self::RangedTraining => "Ranged Training",
            Self::QuietMovement => "Quiet Movement",
            Self::Repair => "Repair",
            Self::Medicine => "Medicine",
            Self::Ritecraft => "Ritecraft",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect, Default)]
pub struct VirtueState {
    pub rank: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect, Default)]
pub struct ProficiencyState {
    pub rating: u32,
    pub xp: u32,
    pub level: u32,
}

impl ProficiencyState {
    pub const fn new(rating: u32) -> Self {
        Self {
            rating,
            xp: 0,
            level: 0,
        }
    }

    pub fn effective(&self) -> u32 {
        self.rating + self.level * 2
    }

    pub fn xp_for_next_level(&self) -> u32 {
        50 * (self.level + 1) * (self.level + 1)
    }

    pub fn grant_xp(&mut self, amount: u32) -> Option<u32> {
        if self.level >= 10 {
            return None;
        }
        self.xp += amount;
        if self.xp >= self.xp_for_next_level() {
            self.xp -= self.xp_for_next_level();
            self.level += 1;
            Some(self.level)
        } else {
            None
        }
    }
}

#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct PlayerProgression {
    #[serde(default)]
    pub virtues: HashMap<VirtueId, VirtueState>,
    #[serde(default)]
    pub proficiencies: HashMap<ProficiencyId, ProficiencyState>,
    #[serde(default)]
    pub kleos: u32,
}

impl PlayerProgression {
    pub fn new_game() -> Self {
        let mut progression = Self::default();

        for virtue in VirtueId::all() {
            progression.virtues.insert(*virtue, VirtueState { rank: 1 });
        }
        for (virtue, rank) in [
            (VirtueId::Fortitude, 3),
            (VirtueId::Thumos, 2),
            (VirtueId::Prudence, 2),
        ] {
            progression.virtues.insert(virtue, VirtueState { rank });
        }

        for (proficiency, rating) in [
            (ProficiencyId::MeleeTraining, 12),
            (ProficiencyId::RangedTraining, 12),
            (ProficiencyId::QuietMovement, 6),
            (ProficiencyId::Medicine, 6),
            (ProficiencyId::Repair, 0),
            (ProficiencyId::Ritecraft, 0),
        ] {
            progression
                .proficiencies
                .insert(proficiency, ProficiencyState::new(rating));
        }

        progression
    }

    pub fn from_legacy_skills(skills: &HashMap<SkillId, SkillState>) -> Self {
        let mut progression = Self::new_game();

        progression.set_legacy_proficiency(
            ProficiencyId::MeleeTraining,
            legacy_skill_value(skills, SkillId::Melee),
        );
        progression.set_legacy_proficiency(
            ProficiencyId::RangedTraining,
            legacy_skill_value(skills, SkillId::Ranged),
        );
        progression.set_legacy_proficiency(
            ProficiencyId::QuietMovement,
            [SkillId::Stealth, SkillId::Evasion]
                .into_iter()
                .filter_map(|skill| legacy_skill_value(skills, skill))
                .max(),
        );
        progression.set_legacy_proficiency(
            ProficiencyId::Repair,
            legacy_skill_value(skills, SkillId::Repair),
        );
        progression.set_legacy_proficiency(
            ProficiencyId::Medicine,
            [SkillId::Toughness, SkillId::Awareness]
                .into_iter()
                .filter_map(|skill| legacy_skill_value(skills, skill))
                .max(),
        );

        progression.set_legacy_virtue(
            VirtueId::Fortitude,
            legacy_skill_value(skills, SkillId::Toughness),
        );
        progression.set_legacy_virtue(VirtueId::Thumos, legacy_skill_value(skills, SkillId::Melee));
        progression.set_legacy_virtue(
            VirtueId::Prudence,
            [SkillId::Awareness, SkillId::Ranged]
                .into_iter()
                .filter_map(|skill| legacy_skill_value(skills, skill))
                .max(),
        );
        progression.set_legacy_virtue(
            VirtueId::Metis,
            [SkillId::Stealth, SkillId::Evasion]
                .into_iter()
                .filter_map(|skill| legacy_skill_value(skills, skill))
                .max(),
        );
        progression.set_legacy_virtue(
            VirtueId::Justice,
            legacy_skill_value(skills, SkillId::Leadership),
        );

        progression
    }

    pub fn is_empty(&self) -> bool {
        self.virtues.is_empty() && self.proficiencies.is_empty() && self.kleos == 0
    }

    pub fn ensure_complete(&mut self) {
        let baseline = Self::new_game();
        for virtue in VirtueId::all() {
            self.virtues
                .entry(*virtue)
                .or_insert_with(|| baseline.virtues[virtue].clone());
        }
        for proficiency in ProficiencyId::all() {
            self.proficiencies
                .entry(*proficiency)
                .or_insert_with(|| baseline.proficiencies[proficiency].clone());
        }
    }

    pub fn virtue_rank(&self, virtue: VirtueId) -> u8 {
        self.virtues.get(&virtue).map_or(0, |state| state.rank)
    }

    pub fn proficiency_rating(&self, proficiency: ProficiencyId) -> u32 {
        self.proficiencies
            .get(&proficiency)
            .map_or(0, |state| state.effective())
    }

    pub fn grant_proficiency_xp(
        &mut self,
        proficiency: ProficiencyId,
        amount: u32,
    ) -> Option<(ProficiencyId, u32)> {
        let state = self.proficiencies.get_mut(&proficiency)?;
        state.grant_xp(amount).map(|level| (proficiency, level))
    }

    pub fn action_rating(
        &self,
        virtue: VirtueId,
        proficiency: ProficiencyId,
        gear_bonus: i32,
        perk_bonus: i32,
    ) -> i32 {
        i32::from(self.virtue_rank(virtue)) * 5
            + self.proficiency_rating(proficiency) as i32
            + gear_bonus
            + perk_bonus
    }

    pub fn enemy_attack_dv(&self) -> i32 {
        self.action_rating(VirtueId::Metis, ProficiencyId::QuietMovement, 0, 0)
    }

    pub fn enemy_attack_breakdown(&self, defense_dv: i32) -> String {
        let virtue_bonus = i32::from(self.virtue_rank(VirtueId::Metis)) * 5;
        let quiet_movement = self.proficiency_rating(ProficiencyId::QuietMovement) as i32;
        format!(
            "DV {} = MET {} + QUI {}",
            defense_dv, virtue_bonus, quiet_movement
        )
    }

    pub fn enemy_attack_check_breakdown(&self, attack_rating: u32, defense_dv: i32) -> String {
        format!(
            " [ATK {} vs {}]",
            attack_rating,
            self.enemy_attack_breakdown(defense_dv)
        )
    }

    pub fn enemy_attack_xp_award(&self, evaded: bool, attacker_skill: u32) -> u32 {
        calc_xp_award(
            evaded,
            false,
            attacker_skill,
            self.enemy_attack_dv().max(0) as u32,
        )
    }

    pub fn grant_enemy_attack_xp(
        &mut self,
        evaded: bool,
        attacker_skill: u32,
    ) -> Option<(ProficiencyId, u32)> {
        let xp = self.enemy_attack_xp_award(evaded, attacker_skill);
        if xp == 0 {
            return None;
        }
        self.grant_proficiency_xp(ProficiencyId::QuietMovement, xp)
    }

    pub fn sync_pilot_combat_skill_proxies(&self, skills: &mut HashMap<SkillId, SkillState>) {
        for (skill, base) in [
            (
                SkillId::Melee,
                self.action_rating(VirtueId::Thumos, ProficiencyId::MeleeTraining, 0, 0)
                    .max(0) as u32,
            ),
            (
                SkillId::Ranged,
                self.action_rating(VirtueId::Prudence, ProficiencyId::RangedTraining, 0, 0)
                    .max(0) as u32,
            ),
            (SkillId::Evasion, self.enemy_attack_dv().max(0) as u32),
        ] {
            skills.insert(
                skill,
                SkillState {
                    base,
                    xp: 0,
                    level: 0,
                },
            );
        }
    }

    pub fn virtue_only_rating(&self, virtue: VirtueId, contextual_bonus: i32) -> i32 {
        i32::from(self.virtue_rank(virtue)) * 5 + contextual_bonus
    }

    fn set_legacy_proficiency(&mut self, proficiency: ProficiencyId, legacy_skill: Option<u32>) {
        let Some(legacy_skill) = legacy_skill.filter(|value| *value > 0) else {
            return;
        };

        self.proficiencies.insert(
            proficiency,
            ProficiencyState::new(legacy_skill_to_proficiency(legacy_skill)),
        );
    }

    fn set_legacy_virtue(&mut self, virtue: VirtueId, legacy_skill: Option<u32>) {
        let Some(legacy_skill) = legacy_skill.filter(|value| *value > 0) else {
            return;
        };

        self.virtues.insert(
            virtue,
            VirtueState {
                rank: legacy_skill_to_virtue_rank(legacy_skill),
            },
        );
    }
}

fn legacy_skill_value(skills: &HashMap<SkillId, SkillState>, skill: SkillId) -> Option<u32> {
    skills.get(&skill).map(SkillState::effective)
}

fn legacy_skill_to_proficiency(skill: u32) -> u32 {
    (skill / 3).clamp(0, 24)
}

fn legacy_skill_to_virtue_rank(skill: u32) -> u8 {
    ((skill / 20) + 1).clamp(1, 5) as u8
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct SkillState {
    pub base: u32,
    pub xp: u32,
    pub level: u32,
}

impl SkillState {
    pub fn effective(&self) -> u32 {
        self.base + self.level * 2
    }

    /// Quadratic XP curve: 50 * (level+1)^2.
    pub fn xp_for_next_level(&self) -> u32 {
        50 * (self.level + 1) * (self.level + 1)
    }

    /// Add XP and level up if threshold met (cap level 10).
    /// Returns `Some(new_level)` on level-up, `None` otherwise.
    pub fn grant_xp(&mut self, amount: u32) -> Option<u32> {
        if self.level >= 10 {
            return None;
        }
        self.xp += amount;
        if self.xp >= self.xp_for_next_level() {
            self.xp -= self.xp_for_next_level();
            self.level += 1;
            Some(self.level)
        } else {
            None
        }
    }
}

#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct CombatStats {
    pub hp: i32,
    pub hp_max: i32,
    pub speed: u8,
    pub ar: i32,
    pub md: i32,
    pub skills: HashMap<SkillId, SkillState>,
}

impl CombatStats {
    pub fn skill_level(&self, id: SkillId) -> u32 {
        self.skills.get(&id).map_or(0, |s| s.effective())
    }

    pub fn is_dead(&self) -> bool {
        self.hp <= 0
    }

    /// Grant XP to a specific skill. Returns `Some((skill, new_level))` on level-up.
    pub fn grant_skill_xp(&mut self, skill: SkillId, amount: u32) -> Option<(SkillId, u32)> {
        let state = self.skills.get_mut(&skill)?;
        state.grant_xp(amount).map(|lvl| (skill, lvl))
    }
}

#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct EntityName {
    pub name: String,
}

/// Calculate XP awarded for a skill check.
/// Diminishing returns when `enemy_skill` is much lower than `player_skill`.
pub fn calc_xp_award(success: bool, critical: bool, enemy_skill: u32, player_skill: u32) -> u32 {
    let base = if critical {
        5
    } else if success {
        3
    } else {
        1
    };

    if player_skill > 0 && enemy_skill < player_skill / 4 {
        0
    } else if player_skill > 0 && enemy_skill < player_skill / 2 {
        base / 2
    } else {
        base
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn skill(level: u32) -> SkillState {
        SkillState {
            base: 5,
            xp: 0,
            level,
        }
    }

    #[test]
    fn test_xp_for_next_level() {
        assert_eq!(skill(0).xp_for_next_level(), 50);
        assert_eq!(skill(1).xp_for_next_level(), 200);
        assert_eq!(skill(5).xp_for_next_level(), 1800);
    }

    #[test]
    fn test_grant_xp_no_levelup() {
        let mut s = skill(0);
        assert_eq!(s.grant_xp(10), None);
        assert_eq!(s.xp, 10);
        assert_eq!(s.level, 0);
    }

    #[test]
    fn test_grant_xp_levelup() {
        let mut s = skill(0);
        assert_eq!(s.grant_xp(60), Some(1));
        assert_eq!(s.level, 1);
        assert_eq!(s.xp, 10);
    }

    #[test]
    fn test_level_cap() {
        let mut s = skill(10);
        assert_eq!(s.grant_xp(9999), None);
        assert_eq!(s.level, 10);
    }

    #[test]
    fn test_diminishing_xp() {
        // enemy_skill 20 is >= 50/4 (12) but < 50/2 (25) → halved: 3/2=1
        assert_eq!(calc_xp_award(true, false, 20, 50), 1);
        // enemy_skill 5 < 50/4 (12) → zero
        assert_eq!(calc_xp_award(true, false, 5, 50), 0);
        assert_eq!(calc_xp_award(true, false, 40, 50), 3);
        assert_eq!(calc_xp_award(false, true, 40, 50), 5);
    }

    #[test]
    fn test_new_game_progression_matches_default_lane() {
        let progression = PlayerProgression::new_game();

        assert_eq!(progression.virtue_rank(VirtueId::Fortitude), 3);
        assert_eq!(progression.virtue_rank(VirtueId::Thumos), 2);
        assert_eq!(
            progression.proficiency_rating(ProficiencyId::MeleeTraining),
            12
        );
        assert_eq!(
            progression.action_rating(VirtueId::Thumos, ProficiencyId::MeleeTraining, 1, 2),
            25
        );
        assert_eq!(progression.enemy_attack_dv(), 11);
        assert_eq!(
            progression.enemy_attack_breakdown(11),
            "DV 11 = MET 5 + QUI 6"
        );
        assert_eq!(
            progression.enemy_attack_check_breakdown(18, 11),
            " [ATK 18 vs DV 11 = MET 5 + QUI 6]"
        );
        assert_eq!(progression.enemy_attack_xp_award(true, 30), 3);
        assert_eq!(progression.enemy_attack_xp_award(false, 30), 1);

        let mut skills = HashMap::new();
        progression.sync_pilot_combat_skill_proxies(&mut skills);
        assert_eq!(skills[&SkillId::Melee].effective(), 22);
        assert_eq!(skills[&SkillId::Ranged].effective(), 22);
        assert_eq!(skills[&SkillId::Evasion].effective(), 11);
    }

    #[test]
    fn test_legacy_progression_infers_melee_lane() {
        let mut skills = HashMap::new();
        skills.insert(
            SkillId::Melee,
            SkillState {
                base: 40,
                xp: 0,
                level: 0,
            },
        );
        skills.insert(
            SkillId::Toughness,
            SkillState {
                base: 30,
                xp: 0,
                level: 0,
            },
        );

        let progression = PlayerProgression::from_legacy_skills(&skills);

        assert_eq!(
            progression.proficiency_rating(ProficiencyId::MeleeTraining),
            13
        );
        assert_eq!(progression.virtue_rank(VirtueId::Thumos), 3);
        assert_eq!(progression.virtue_rank(VirtueId::Fortitude), 2);
    }

    #[test]
    fn pilot_proxies_excludes_legacy_compat_variants() {
        let pilot_keys: std::collections::HashSet<_> =
            SkillId::pilot_proxies().iter().copied().collect();
        let all_keys: std::collections::HashSet<_> = SkillId::all().iter().copied().collect();
        let legacy_keys: std::collections::HashSet<_> =
            all_keys.difference(&pilot_keys).copied().collect();
        assert_eq!(
            pilot_keys,
            [SkillId::Melee, SkillId::Ranged, SkillId::Evasion]
                .into_iter()
                .collect(),
            "pilot_proxies must return exactly the three canonical pilot proxy skills"
        );
        assert_eq!(
            legacy_keys,
            [
                SkillId::Toughness,
                SkillId::Stealth,
                SkillId::Awareness,
                SkillId::Repair,
                SkillId::Leadership
            ]
            .into_iter()
            .collect(),
            "legacy-compat variants must be exactly the non-pilot skills"
        );
    }
}
