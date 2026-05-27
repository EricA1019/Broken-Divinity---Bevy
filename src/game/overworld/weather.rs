use bevy::prelude::*;
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

const WEATHER_CLEAR_MAX: u32 = 54;
const WEATHER_FOG_MAX: u32 = 79;
const WEATHER_ASHFALL_MAX: u32 = 94;
const WEATHER_ROLL_RANGE: u32 = 100;
const WEATHER_DAY_SALT: u64 = 0x9E37_79B9_7F4A_7C15;
const CLEAR_TRAVEL_SPEED: f32 = 1.0;
const FOG_TRAVEL_SPEED: f32 = 0.75;
const ASHFALL_TRAVEL_SPEED: f32 = 0.5;
const STORM_TRAVEL_SPEED: f32 = 0.0;
const ASHFALL_SANITY_PRESSURE: u32 = 1;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, Reflect,
)]
pub enum Weather {
    #[default]
    Clear,
    Fog,
    Ashfall,
    Storm,
}

impl Weather {
    pub fn name(self) -> &'static str {
        match self {
            Self::Clear => "Clear skies",
            Self::Fog => "Fog",
            Self::Ashfall => "Ashfall",
            Self::Storm => "Stormfront",
        }
    }

    pub fn travel_speed(self) -> f32 {
        match self {
            Self::Clear => CLEAR_TRAVEL_SPEED,
            Self::Fog => FOG_TRAVEL_SPEED,
            Self::Ashfall => ASHFALL_TRAVEL_SPEED,
            Self::Storm => STORM_TRAVEL_SPEED,
        }
    }

    pub fn sanity_pressure(self) -> u32 {
        match self {
            Self::Ashfall => ASHFALL_SANITY_PRESSURE,
            _ => 0,
        }
    }

    pub fn blocks_travel(self) -> bool {
        matches!(self, Self::Storm)
    }
}

pub fn roll_weather(world_seed: u64, day: u32) -> Weather {
    let seed = world_seed ^ ((day as u64).wrapping_add(1).wrapping_mul(WEATHER_DAY_SALT));
    let mut rng = ChaCha8Rng::seed_from_u64(seed);

    match rng.random_range(0..WEATHER_ROLL_RANGE) {
        0..=WEATHER_CLEAR_MAX => Weather::Clear,
        55..=WEATHER_FOG_MAX => Weather::Fog,
        80..=WEATHER_ASHFALL_MAX => Weather::Ashfall,
        _ => Weather::Storm,
    }
}