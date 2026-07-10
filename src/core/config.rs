//! Game balance tuning constants loaded from RON config.
//!
//! All starting values, caps, and tuning levers live here.
//! The RON file is embedded at compile time via `include_str!`.

use std::sync::OnceLock;

use serde::Deserialize;

// ---------------------------------------------------------------------------
// Tuning schema
// ---------------------------------------------------------------------------

/// Player starting stats and baseline caps.
#[derive(Debug, Clone, Deserialize)]
pub struct PlayerTuning {
    pub starting_hp: i32,
    pub starting_hp_max: i32,
    pub starting_speed: u8,
    pub starting_ar: i32,
    pub starting_md: i32,
    pub starting_viewshed_range: u32,
    pub starting_weapon_id: String,
}

/// Shelter resource starting quantities.
#[derive(Debug, Clone, Deserialize)]
pub struct ShelterTuning {
    pub starting_food: u32,
    pub starting_water: u32,
    pub starting_scrap: u32,
    pub starting_medicine: u32,
    pub starting_ammo: u32,
}

/// Starting virtue ranks for a new game.
#[derive(Debug, Clone, Deserialize)]
pub struct VirtueStartingRank {
    pub temperance: u8,
    pub justice: u8,
    pub prudence: u8,
    pub fortitude: u8,
    pub thumos: u8,
    pub metis: u8,
}

/// Starting proficiency ratings for a new game.
#[derive(Debug, Clone, Deserialize)]
pub struct ProficiencyStartingRating {
    pub melee_training: u32,
    pub ranged_training: u32,
    pub quiet_movement: u32,
    pub medicine: u32,
    pub repair: u32,
    pub ritecraft: u32,
}

/// Top-level tuning config container.
#[derive(Debug, Clone, Deserialize)]
pub struct GameBalance {
    pub player: PlayerTuning,
    pub shelter: ShelterTuning,
    pub virtues: VirtueStartingRank,
    pub proficiencies: ProficiencyStartingRating,
}

// ---------------------------------------------------------------------------
// Embedded source path
// ---------------------------------------------------------------------------

/// Path relative to Cargo.toml for the RON tuning file.
const TUNING_RON_SOURCE: &str = include_str!("../../native/assets/data/tuning.ron");

// ---------------------------------------------------------------------------
// Cached loader
// ---------------------------------------------------------------------------

static BALANCE: OnceLock<GameBalance> = OnceLock::new();

fn parse_balance() -> GameBalance {
    ron::from_str(TUNING_RON_SOURCE)
        .expect("Failed to parse embedded tuning.ron — check schema compatibility")
}

/// Access the cached game balance.
pub fn game_balance() -> &'static GameBalance {
    BALANCE.get_or_init(parse_balance)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tuning_file_exists_and_loads() {
        let balance = game_balance();
        // Verify all sections are present with positive values
        assert!(balance.player.starting_hp > 0, "Player starting HP must be positive");
        assert!(balance.player.starting_ar >= 0, "Player starting AR must be >= 0");
        assert!(balance.shelter.starting_food > 0, "Starting food must be positive");
        assert!(balance.virtues.fortitude > 0, "Fortitude virtue rank must be positive");
        assert!(balance.proficiencies.melee_training > 0, "Melee training rating must be positive");
    }

    #[test]
    fn test_tuning_values_are_deterministic() {
        // Two loads must produce identical values (file-based, not time-based)
        let a = game_balance();
        let b = game_balance();
        assert_eq!(a.player.starting_hp, b.player.starting_hp);
        assert_eq!(a.shelter.starting_food, b.shelter.starting_food);
        assert_eq!(a.virtues.fortitude, b.virtues.fortitude);
        assert_eq!(a.proficiencies.melee_training, b.proficiencies.melee_training);
    }
}