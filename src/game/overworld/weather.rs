//! Weather system — rolled per travel day, affects encounters and travel.

use bevy::prelude::*;
use rand::RngExt;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum Weather {
    Clear,
    Overcast,
    Rain,
    HeavyRain,
    Fog,
    DustStorm,
    Ashfall,
    AnomalyStorm,
}

impl Weather {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Clear => "Clear",
            Self::Overcast => "Overcast",
            Self::Rain => "Rain",
            Self::HeavyRain => "Heavy Rain",
            Self::Fog => "Fog",
            Self::DustStorm => "Dust Storm",
            Self::Ashfall => "Ashfall",
            Self::AnomalyStorm => "Anomaly Storm",
        }
    }

    /// Ranged accuracy modifier applied during encounters.
    pub fn ranged_penalty(&self) -> i32 {
        match self {
            Self::Rain => -10,
            Self::HeavyRain => -20,
            Self::DustStorm => -15,
            _ => 0,
        }
    }

    /// Travel speed multiplier (1.0 = normal).
    pub fn travel_speed(&self) -> f32 {
        match self {
            Self::HeavyRain | Self::DustStorm => 0.75,
            Self::AnomalyStorm => 0.0, // travel blocked
            _ => 1.0,
        }
    }

    /// Visibility reduction in tiles.
    pub fn visibility_penalty(&self) -> i32 {
        match self {
            Self::Rain => 1,
            Self::HeavyRain | Self::DustStorm => 2,
            Self::Fog => 3,
            _ => 0,
        }
    }

    /// Sanity pressure per travel day.
    pub fn sanity_pressure(&self) -> u32 {
        match self {
            Self::Ashfall => 1,
            Self::AnomalyStorm => 3,
            _ => 0,
        }
    }

    /// Whether this weather blocks travel entirely.
    pub fn blocks_travel(&self) -> bool {
        matches!(self, Self::AnomalyStorm)
    }
}

/// Roll weather for a given day using deterministic RNG.
pub fn roll_weather(world_seed: u64, day: u32) -> Weather {
    let weather_seed = world_seed.wrapping_add(day as u64 * 7919); // prime multiplier
    let mut rng = ChaCha8Rng::seed_from_u64(weather_seed);

    // Weighted roll: Clear 30, Overcast 20, Rain 15, HeavyRain 5, Fog 10, DustStorm 5, Ashfall 10, AnomalyStorm 5
    let roll = rng.random_range(0..100u32);
    match roll {
        0..30 => Weather::Clear,
        30..50 => Weather::Overcast,
        50..65 => Weather::Rain,
        65..70 => Weather::HeavyRain,
        70..80 => Weather::Fog,
        80..85 => Weather::DustStorm,
        85..95 => Weather::Ashfall,
        _ => Weather::AnomalyStorm,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determinism() {
        let seed = 42u64;
        let day = 7u32;
        let a = roll_weather(seed, day);
        let b = roll_weather(seed, day);
        assert_eq!(a, b, "Same seed+day must produce identical weather");
    }

    #[test]
    fn test_weights() {
        let seed = 12345u64;
        let samples = 10_000u32;
        let mut clear_count = 0u32;
        for day in 0..samples {
            if roll_weather(seed, day) == Weather::Clear {
                clear_count += 1;
            }
        }
        let pct = clear_count as f64 / samples as f64 * 100.0;
        assert!(
            (25.0..=35.0).contains(&pct),
            "Clear should be ~30% but was {pct:.1}%"
        );
    }

    #[test]
    fn test_anomaly_blocks_travel() {
        assert!(Weather::AnomalyStorm.blocks_travel());
        for w in [
            Weather::Clear,
            Weather::Overcast,
            Weather::Rain,
            Weather::HeavyRain,
            Weather::Fog,
            Weather::DustStorm,
            Weather::Ashfall,
        ] {
            assert!(!w.blocks_travel(), "{:?} should not block travel", w);
        }
    }
}
