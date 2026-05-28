//! Dungeon anomalies — supernatural environmental hazards that drain sanity on proximity.

use bevy::prelude::*;
use rand::RngExt;
use serde::{Deserialize, Serialize};

use crate::core::components::{Player, Position};
use crate::core::gamelog::{GameLog, LogColor};
use crate::core::perks::PlayerPerks;
use crate::core::sanity::{RaidExposure, SanityEvent};
use crate::core::turn::GameTime;

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Marker + data for a dungeon anomaly entity.
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct Anomaly {
    pub kind: AnomalyKind,
    /// Detection radius in manhattan distance (typically 3).
    pub radius: u32,
    /// Whether this anomaly has already been triggered this floor.
    pub triggered: bool,
}

/// Variants of supernatural environmental hazard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum AnomalyKind {
    VisualDistortion,
    Whispers,
    SpatialRift,
}

impl AnomalyKind {
    pub fn name(&self) -> &'static str {
        match self {
            Self::VisualDistortion => "Visual Distortion",
            Self::Whispers => "Whispers",
            Self::SpatialRift => "Spatial Rift",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::VisualDistortion => "Reality bends and shifts around you",
            Self::Whispers => "You hear voices that aren't there",
            Self::SpatialRift => "Space folds in on itself nearby",
        }
    }
}

// ---------------------------------------------------------------------------
// Spawn helper (called from spawn.rs)
// ---------------------------------------------------------------------------

/// Spawn 1–2+ anomalies in random rooms (never room 0 = player spawn).
pub fn spawn_anomalies(
    commands: &mut Commands,
    rooms: &[crate::game::dungeon::bsp::Rect],
    rng: &mut impl rand::Rng,
    floor_number: u32,
) {
    if rooms.len() <= 1 {
        return;
    }

    let base_count = 1 + floor_number.min(2) / 2;
    let count = base_count as usize;

    let kinds = [
        AnomalyKind::VisualDistortion,
        AnomalyKind::Whispers,
        AnomalyKind::SpatialRift,
    ];

    for _ in 0..count {
        // Pick a random room index, excluding room 0 (player spawn).
        let room_idx = rng.random_range(1..rooms.len());
        let room = &rooms[room_idx];

        // Pick a random position inside the room.
        let x = rng.random_range(room.x..room.x + room.w);
        let y = rng.random_range(room.y..room.y + room.h);

        let kind = kinds[rng.random_range(0..kinds.len())];

        commands.spawn((
            Anomaly {
                kind,
                radius: 3,
                triggered: false,
            },
            Position { x, y },
        ));
    }
}

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Checks manhattan distance between each untriggered anomaly and the player.
/// If within radius, triggers the anomaly and applies sanity exposure.
pub fn check_anomaly_proximity(
    mut anomalies: Query<(&mut Anomaly, &Position), Without<Player>>,
    mut player_q: Query<(&Position, &mut RaidExposure, &PlayerPerks), With<Player>>,
    mut log: ResMut<GameLog>,
    time: Res<GameTime>,
) {
    let Ok((player_pos, mut raid_exposure, perks)) = player_q.single_mut() else {
        return;
    };

    for (mut anomaly, pos) in &mut anomalies {
        if anomaly.triggered {
            continue;
        }

        let dx = (pos.x - player_pos.x).unsigned_abs();
        let dy = (pos.y - player_pos.y).unsigned_abs();
        let distance = dx + dy;

        if distance <= anomaly.radius {
            anomaly.triggered = true;
            crate::core::sanity::apply_player_sanity_event(
                &mut raid_exposure,
                perks,
                SanityEvent::AnomalyProximity,
            );
            log.push(anomaly.kind.description(), LogColor::Status, time.turn);
            log.push("Sanity exposure increased.", LogColor::Status, time.turn);
        }
    }
}
