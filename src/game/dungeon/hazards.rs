//! Per-theme hazard tiles that apply effects when walked on.

use bevy::prelude::*;
use rand::RngExt;
use serde::{Deserialize, Serialize};

use crate::core::components::{Player, Position};
use crate::core::gamelog::{GameLog, LogColor};
use crate::core::perks::PlayerPerks;
use crate::core::sanity::{RaidExposure, SanityEvent};
use crate::core::stats::CombatStats;
use crate::core::turn::GameTime;
use crate::game::dungeon::theme::DungeonTheme;

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct HazardTile {
    pub kind: HazardKind,
    pub active: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum HazardKind {
    /// UrbanDecay theme. Collapse trap: 5 damage, stun message, makes tile
    /// unwalkable after triggering.
    UnstableFloor,
    /// Underground theme. Slows movement (placeholder: just -1 AP next turn
    /// message).
    DeepWater,
    /// Military theme. 8 damage if active, can be disabled (placeholder).
    SecurityTurret,
}

impl HazardKind {
    pub fn name(&self) -> &'static str {
        match self {
            Self::UnstableFloor => "Unstable Floor",
            Self::DeepWater => "Deep Water",
            Self::SecurityTurret => "Security Turret",
        }
    }

    pub fn damage(&self) -> i32 {
        match self {
            Self::UnstableFloor => 5,
            Self::DeepWater => 0,
            Self::SecurityTurret => 8,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::UnstableFloor => "The floor collapses beneath you! Rubble and dust rain down.",
            Self::DeepWater => "You wade through knee-deep water. Your movement slows to a crawl.",
            Self::SecurityTurret => "A security turret whirs to life and opens fire!",
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Spawns theme-appropriate hazard tiles in dungeon rooms.
///
/// Skips room 0 (spawn room). Each remaining room has a 20% chance to receive
/// one hazard at a random position.
pub fn spawn_hazards(
    commands: &mut Commands,
    rooms: &[crate::game::dungeon::bsp::Rect],
    theme: DungeonTheme,
    rng: &mut impl rand::Rng,
) {
    let kind = match theme {
        DungeonTheme::UrbanDecay => HazardKind::UnstableFloor,
        DungeonTheme::Underground => HazardKind::DeepWater,
        DungeonTheme::Military => HazardKind::SecurityTurret,
    };

    for room in rooms.iter().skip(1) {
        if rng.random_range(0..100) >= 20 {
            continue;
        }

        let x = rng.random_range(room.x..room.x + room.w);
        let y = rng.random_range(room.y..room.y + room.h);

        commands.spawn((HazardTile { kind, active: true }, Position { x, y }));
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Checks whether the player is standing on an active hazard tile and applies
/// the effect. Runs during `TurnPhase::PlayerTurn`.
pub fn check_hazard_tiles(
    mut hazards: Query<(&mut HazardTile, &Position)>,
    mut player_q: Query<
        (&Position, &mut CombatStats, &mut RaidExposure, &PlayerPerks),
        With<Player>,
    >,
    mut log: ResMut<GameLog>,
    time: Res<GameTime>,
) {
    let Ok((player_pos, mut stats, mut exposure, perks)) = player_q.single_mut() else {
        return;
    };

    for (mut hazard, pos) in hazards.iter_mut() {
        if !hazard.active || *pos != *player_pos {
            continue;
        }

        let dmg = hazard.kind.damage();
        if dmg > 0 {
            stats.hp -= dmg;
        }

        crate::core::sanity::apply_player_sanity_event(
            &mut exposure,
            perks,
            SanityEvent::HazardExposure,
        );

        log.push(hazard.kind.description(), LogColor::Status, time.turn);

        hazard.active = false;
    }
}
