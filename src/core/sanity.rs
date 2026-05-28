//! Raid Exposure sanity system — a meter that fills during dungeon runs
//! from combat/anomalies and resets on shelter return.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::core::components::{Enemy, Player, Position};
use crate::core::gamelog::{GameLog, LogColor};
use crate::core::movement::MapTiles;
use crate::core::perks::PlayerPerks;
use crate::core::stats::{CombatStats, EntityName};
use crate::core::status::StatusEffects;
use crate::core::turn::GameTime;

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Sanity meter attached to the player entity. Fills from 0 → max during
/// dungeon runs; resets to 0 on shelter return.
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct RaidExposure {
    pub current: u32,
    pub max: u32,
}

#[derive(Component, Debug, Clone, Copy, Default, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct Hallucination;

impl Default for RaidExposure {
    fn default() -> Self {
        Self {
            current: 0,
            max: 100,
        }
    }
}

impl RaidExposure {
    /// Add exposure, clamped to max.
    pub fn add(&mut self, amount: u32) {
        self.current = (self.current + amount).min(self.max);
    }

    /// Reset exposure to zero (e.g. on shelter return).
    pub fn reset(&mut self) {
        self.current = 0;
    }

    /// Current exposure as a 0.0–1.0 fraction.
    pub fn fraction(&self) -> f32 {
        if self.max == 0 {
            return 0.0;
        }
        self.current as f32 / self.max as f32
    }

    /// Current threshold level based on exposure.
    pub fn threshold(&self) -> SanityThreshold {
        match self.current {
            0..50 => SanityThreshold::Normal,
            50..75 => SanityThreshold::Stressed,
            75..90 => SanityThreshold::Shaken,
            _ => SanityThreshold::Breaking,
        }
    }
}

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Discrete sanity bands with increasing penalties.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SanityThreshold {
    /// 0–49: no penalties.
    Normal,
    /// 50–74: perception penalty (−5 to skill checks).
    Stressed,
    /// 75–89: hallucination spawns.
    Shaken,
    /// 90–100: intermittent control loss.
    Breaking,
}

impl SanityThreshold {
    /// Skill-check penalty for this threshold.
    pub fn penalty(&self) -> i32 {
        match self {
            Self::Normal => 0,
            Self::Stressed => -5,
            Self::Shaken => -10,
            Self::Breaking => -15,
        }
    }

    /// Human-readable name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Stressed => "Stressed",
            Self::Shaken => "Shaken",
            Self::Breaking => "Breaking",
        }
    }
}

/// Pure data type describing what caused an exposure increase.
pub enum SanityEvent {
    /// +2 exposure per hit taken.
    CombatHit,
    /// +5 per anomaly encountered.
    AnomalyProximity,
    /// +1 per enemy killed.
    Kill,
    /// +3 per hazard.
    HazardExposure,
}

impl SanityEvent {
    /// How much exposure this event adds.
    pub fn exposure_amount(&self) -> u32 {
        match self {
            Self::CombatHit => 2,
            Self::AnomalyProximity => 5,
            Self::Kill => 1,
            Self::HazardExposure => 3,
        }
    }
}

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

/// Apply a sanity event to the meter. Returns `true` if a new threshold was
/// crossed (useful for triggering one-shot effects on transition).
pub fn apply_sanity_event(exposure: &mut RaidExposure, event: SanityEvent) -> bool {
    let old_threshold = exposure.threshold();
    exposure.add(event.exposure_amount());
    exposure.threshold() != old_threshold
}

pub fn apply_player_sanity_event(
    exposure: &mut RaidExposure,
    perks: &PlayerPerks,
    event: SanityEvent,
) -> bool {
    let old_threshold = exposure.threshold();
    let mut amount = event.exposure_amount();
    if perks.sanity_reduction() {
        amount /= 2;
    }
    exposure.add(amount);
    exposure.threshold() != old_threshold
}

pub fn should_lose_control(turn: u32) -> bool {
    simple_hash(turn, 0xBEEF) % 100 < 20
}

pub fn forced_move_direction(turn: u32) -> (i32, i32) {
    match simple_hash(turn, 0xD1CE) % 4 {
        0 => (0, -1),
        1 => (1, 0),
        2 => (0, 1),
        _ => (-1, 0),
    }
}

// ---------------------------------------------------------------------------
// Systems (placeholder — logging only)
// ---------------------------------------------------------------------------

/// During `PlayerTurn`: if Shaken or worse, 15 % chance per turn to hint at
/// hallucinations. Actual hallucination entity spawning is deferred.
pub fn check_hallucinations(
    mut commands: Commands,
    query: Query<(&Position, &RaidExposure), With<Player>>,
    existing: Query<Entity, With<Hallucination>>,
    occupied: Query<&Position, With<Enemy>>,
    map: Option<Res<MapTiles>>,
    mut log: ResMut<GameLog>,
    time: Res<GameTime>,
) {
    let Ok((player_pos, exposure)) = query.single() else {
        return;
    };
    if exposure.threshold() < SanityThreshold::Shaken {
        return;
    }

    if !existing.is_empty() {
        return;
    }

    let roll = simple_hash(time.turn, 0xCAFE) % 100;
    if roll >= 15 {
        return;
    }

    let Some(map) = map else {
        return;
    };

    let occupied_tiles: Vec<(i32, i32)> = occupied.iter().map(|pos| (pos.x, pos.y)).collect();
    let directions = [
        (0, -1),
        (1, 0),
        (0, 1),
        (-1, 0),
        (1, -1),
        (1, 1),
        (-1, 1),
        (-1, -1),
    ];
    let start_idx = (simple_hash(time.turn, 0xFACE) as usize) % directions.len();

    for offset in 0..directions.len() {
        let (dx, dy) = directions[(start_idx + offset) % directions.len()];
        let pos = Position::new(player_pos.x + dx, player_pos.y + dy);
        if !map.is_walkable(pos.x, pos.y) {
            continue;
        }
        if occupied_tiles
            .iter()
            .any(|&(x, y)| x == pos.x && y == pos.y)
        {
            continue;
        }
        commands.spawn((
            Enemy,
            Hallucination,
            EntityName {
                name: "Hallucination".to_string(),
            },
            pos,
            CombatStats {
                hp: 1,
                hp_max: 1,
                speed: 0,
                ar: 0,
                md: 0,
                skills: Default::default(),
            },
            StatusEffects::default(),
        ));
        log.push(
            "A shape lurches at the edge of your vision.",
            LogColor::Status,
            time.turn,
        );
        return;
    }

    log.push("The shadows move...", LogColor::Status, time.turn);
}

/// During `AwaitingInput`: if Breaking, 20 % chance to log a control-loss
/// warning. Actual input override is deferred.
pub fn check_control_loss(
    query: Query<&RaidExposure, With<Player>>,
    mut log: ResMut<GameLog>,
    time: Res<GameTime>,
) {
    let Ok(exposure) = query.single() else { return };
    if exposure.threshold() != SanityThreshold::Breaking {
        return;
    }
    let roll = simple_hash(time.turn, 0xBEEF) % 100;
    if roll < 20 {
        log.push(
            "Your hands tremble uncontrollably.",
            LogColor::Status,
            time.turn,
        );
    }
}

/// Cheap deterministic hash for turn-based random checks.
fn simple_hash(turn: u32, salt: u32) -> u32 {
    let mut h = turn.wrapping_mul(2654435761).wrapping_add(salt);
    h ^= h >> 16;
    h = h.wrapping_mul(0x45d9f3b);
    h ^= h >> 16;
    h
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exposure_clamps_to_max() {
        let mut exp = RaidExposure::default();
        exp.add(200);
        assert_eq!(exp.current, 100);
    }

    #[test]
    fn test_threshold_boundaries() {
        let mut exp = RaidExposure {
            current: 0,
            ..Default::default()
        };

        assert_eq!(exp.threshold(), SanityThreshold::Normal);

        exp.current = 50;
        assert_eq!(exp.threshold(), SanityThreshold::Stressed);

        exp.current = 75;
        assert_eq!(exp.threshold(), SanityThreshold::Shaken);

        exp.current = 90;
        assert_eq!(exp.threshold(), SanityThreshold::Breaking);
    }

    #[test]
    fn test_reset() {
        let mut exp = RaidExposure::default();
        exp.add(60);
        assert!(exp.current > 0);
        exp.reset();
        assert_eq!(exp.current, 0);
    }

    #[test]
    fn test_event_amounts() {
        assert_eq!(SanityEvent::CombatHit.exposure_amount(), 2);
        assert_eq!(SanityEvent::AnomalyProximity.exposure_amount(), 5);
        assert_eq!(SanityEvent::Kill.exposure_amount(), 1);
        assert_eq!(SanityEvent::HazardExposure.exposure_amount(), 3);
    }

    #[test]
    fn test_threshold_penalty() {
        assert_eq!(SanityThreshold::Normal.penalty(), 0);
        assert_eq!(SanityThreshold::Stressed.penalty(), -5);
        assert_eq!(SanityThreshold::Shaken.penalty(), -10);
        assert_eq!(SanityThreshold::Breaking.penalty(), -15);
    }

    #[test]
    fn test_iron_will_reduces_exposure() {
        let mut perks = PlayerPerks::default();
        perks.unlock(crate::core::perks::PerkId::IronWill);
        let mut exp = RaidExposure::default();

        apply_player_sanity_event(&mut exp, &perks, SanityEvent::CombatHit);

        assert_eq!(exp.current, 1);
    }

    #[test]
    fn test_control_loss_helpers_are_deterministic() {
        assert_eq!(should_lose_control(12), should_lose_control(12));
        assert_eq!(forced_move_direction(34), forced_move_direction(34));
    }
}
