//! Sanity system — mental health, hallucination thresholds, and breakdown.

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};


pub const SANITY_MAX: i32 = 100;
pub const SANITY_HALLUCINATION_THRESHOLD: i32 = 50;
pub const SANITY_BREAKDOWN_THRESHOLD: i32 = 25;
pub const SANITY_RECOVERY_AT_SHELTER: i32 = 20;

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct SanityPressure {
    pub radius: u32,
    pub drain_per_turn: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanity_constants_are_sane() {
        assert!(SANITY_BREAKDOWN_THRESHOLD < SANITY_HALLUCINATION_THRESHOLD);
        assert!(SANITY_HALLUCINATION_THRESHOLD < SANITY_MAX);
    }
}
