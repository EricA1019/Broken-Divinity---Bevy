use bevy::prelude::*;
use rand::RngExt;
use serde::{Deserialize, Serialize};

pub fn plugin(_app: &mut App) {
    // Combat systems are registered in the dungeon plugin (melee, ranged).
    // This space reserved for future combat-wide system registration.
}

// ── Skill Check ──────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct CheckResult {
    pub success: bool,
    pub critical: bool,
    pub fumble: bool,
    pub roll: u32,
    pub target_number: i32,
}

pub fn roll_check(
    skill_level: u32,
    modifiers: i32,
    target_dv: i32,
    rng: &mut impl rand::Rng,
) -> CheckResult {
    let target_number = skill_level as i32 + 25 + modifiers - target_dv;
    let roll: u32 = rng.random_range(1..=100);
    let success = (roll as i32) <= target_number;
    let critical = success && roll <= (skill_level / 5).max(1);
    let fumble = roll == 100;

    CheckResult {
        success,
        critical,
        fumble,
        roll,
        target_number,
    }
}

// ── Damage ───────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum DamageType {
    Ballistic,
    Slash,
    Blunt,
    Celestial,
    Infernal,
    Thaumic,
}

impl DamageType {
    pub fn is_physical(&self) -> bool {
        matches!(self, Self::Ballistic | Self::Slash | Self::Blunt)
    }

    pub fn is_supernatural(&self) -> bool {
        matches!(self, Self::Celestial | Self::Infernal | Self::Thaumic)
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Ballistic => "Ballistic",
            Self::Slash => "Slash",
            Self::Blunt => "Blunt",
            Self::Celestial => "Celestial",
            Self::Infernal => "Infernal",
            Self::Thaumic => "Thaumic",
        }
    }
}

pub fn calc_damage(
    weapon_base: i32,
    skill_level: u32,
    target_ar: i32,
    target_md: i32,
    damage_type: DamageType,
    is_crit: bool,
    rng: &mut impl rand::Rng,
) -> i32 {
    let defense = if damage_type.is_physical() {
        target_ar
    } else {
        target_md / 2
    };
    let skill_damage_bonus = skill_level as i32 / 5;
    let base = (weapon_base + skill_damage_bonus - defense).max(1);
    let variance = rng.random_range(80..=120) as f32 / 100.0;
    let crit_mult = if is_crit { 2.0 } else { 1.0 };
    let final_dmg = (base as f32 * variance * crit_mult).round() as i32;
    final_dmg.max(1)
}

// ── Tests ────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_roll_check_success() {
        // skill 50, no mods, dv 0 → target_number = 75
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let result = roll_check(50, 0, 0, &mut rng);
        assert_eq!(result.target_number, 75);
        if result.roll <= 75 {
            assert!(result.success);
        } else {
            assert!(!result.success);
        }
    }

    #[test]
    fn test_fumble_on_100() {
        // Brute-force a seed that produces roll=100.
        for seed in 0u64..100_000 {
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            let result = roll_check(50, 0, 0, &mut rng);
            if result.roll == 100 {
                assert!(result.fumble);
                // roll 100 > target 75 → should also fail
                assert!(!result.success);
                return;
            }
        }
        panic!("Could not find seed producing roll of 100 in 100k attempts");
    }

    #[test]
    fn test_damage_min_1() {
        // weapon 1, skill 0, AR 100 → base clamped to 1, result ≥ 1
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let dmg = calc_damage(1, 0, 100, 0, DamageType::Slash, false, &mut rng);
        assert!(dmg >= 1);
    }

    #[test]
    fn test_crit_doubles_damage() {
        // Same seed → identical variance, crit should be roughly 2× normal, allowing
        // for a 1-point rounding difference when the pre-rounded damage is fractional.
        let mut rng_normal = ChaCha8Rng::seed_from_u64(42);
        let mut rng_crit = ChaCha8Rng::seed_from_u64(42);

        let normal = calc_damage(10, 5, 5, 0, DamageType::Slash, false, &mut rng_normal);
        let crit = calc_damage(10, 5, 5, 0, DamageType::Slash, true, &mut rng_crit);
        assert!(
            (normal * 2 - 1..=normal * 2 + 1).contains(&crit),
            "expected crit {crit} to be roughly double normal {normal}"
        );
    }

    #[test]
    fn test_supernatural_uses_md() {
        // High AR, zero MD → supernatural damage should ignore AR entirely.
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let dmg = calc_damage(10, 5, 100, 0, DamageType::Celestial, false, &mut rng);
        // defense = md/2 = 0, base = 10+5−0 = 15 → result well above 1
        assert!(dmg > 1, "Supernatural should ignore AR; got {dmg}");
    }

    #[test]
    fn test_high_skill_damage_stays_in_sane_range() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let dmg = calc_damage(6, 40, 5, 0, DamageType::Slash, false, &mut rng);
        assert!(
            dmg < 20,
            "starting-skill melee should not spike to floor-1 one-shots"
        );
    }

    // ── Property-based invariants (deterministic brute-force) ──

    /// Verify roll_check range invariants across seeds and skill levels.
    #[test]
    fn invariant_roll_in_range() {
        for seed in 0u64..500 {
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            for skill in [0, 25, 50, 100, 200] {
                for &mods in &[-50, -25, 0, 25, 50] {
                    for &dv in &[-50, -25, 0, 25, 50] {
                        let result = roll_check(skill, mods, dv, &mut rng);
                        assert!(
                            result.roll >= 1 && result.roll <= 100,
                            "roll {} out of range",
                            result.roll
                        );
                    }
                }
            }
        }
    }

    /// Verify fumble is only true when roll == 100.
    #[test]
    fn invariant_fumble_only_on_100() {
        for seed in 0u64..500 {
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            for skill in [0, 25, 50, 100] {
                let result = roll_check(skill, 0, 0, &mut rng);
                assert_eq!(
                    result.fumble,
                    result.roll == 100,
                    "fumble must be true iff roll == 100 at seed={}",
                    seed
                );
            }
        }
    }

    /// Verify critical always implies success.
    #[test]
    fn invariant_critical_implies_success() {
        for seed in 0u64..500 {
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            for skill in [0, 25, 50, 100, 200] {
                let result = roll_check(skill, 0, 0, &mut rng);
                if result.critical {
                    assert!(
                        result.success,
                        "critical must imply success at seed={} skill={}",
                        seed, skill
                    );
                }
            }
        }
    }

    /// Verify target_number formula.
    #[test]
    fn invariant_target_number_formula() {
        for seed in 0u64..500 {
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            for skill in [0, 25, 50, 100] {
                for &mods in &[-30, 0, 30] {
                    for &dv in &[-30, 0, 30] {
                        let result = roll_check(skill, mods, dv, &mut rng);
                        let expected = skill as i32 + 25 + mods - dv;
                        assert_eq!(
                            result.target_number, expected,
                            "formula violated at seed={}",
                            seed
                        );
                    }
                }
            }
        }
    }

    /// Verify damage is always >= 1 across many inputs.
    #[test]
    fn invariant_damage_min_1() {
        for seed in 0u64..500 {
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            for weapon in [1, 5, 25, 49] {
                for skill in [0u32, 10, 50, 99] {
                    for ar in [0, 10, 50] {
                        for &crit in &[false, true] {
                            for &dt in &[
                                DamageType::Blunt,
                                DamageType::Ballistic,
                                DamageType::Celestial,
                            ] {
                                let dmg = calc_damage(weapon, skill, ar, 0, dt, crit, &mut rng);
                                assert!(
                                    dmg >= 1,
                                    "damage must be >= 1, got {} for {:?} seed={}",
                                    dmg,
                                    dt,
                                    seed
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    /// Verify damage determinism: same seed = same result.
    #[test]
    fn invariant_damage_deterministic() {
        for seed in 0u64..500 {
            let mut rng_a = ChaCha8Rng::seed_from_u64(seed);
            let mut rng_b = ChaCha8Rng::seed_from_u64(seed);
            let dmg_a = calc_damage(10, 25, 5, 0, DamageType::Blunt, false, &mut rng_a);
            let dmg_b = calc_damage(10, 25, 5, 0, DamageType::Blunt, false, &mut rng_b);
            assert_eq!(dmg_a, dmg_b, "determinism violated at seed={}", seed);
        }
    }

    /// Verify supernatural damage ignores high AR.
    #[test]
    fn invariant_supernatural_ignores_ar() {
        for seed in 0u64..500 {
            let mut rng_phys = ChaCha8Rng::seed_from_u64(seed);
            let mut rng_super = ChaCha8Rng::seed_from_u64(seed);
            let dmg_phys = calc_damage(10, 15, 50, 0, DamageType::Blunt, false, &mut rng_phys);
            let dmg_super =
                calc_damage(10, 15, 50, 0, DamageType::Celestial, false, &mut rng_super);
            assert!(
                dmg_super >= dmg_phys,
                "supernatural should >= physical vs high AR (phys={}, super={}) at seed={}",
                dmg_phys,
                dmg_super,
                seed
            );
        }
    }
}
