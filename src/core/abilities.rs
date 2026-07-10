//! Ability data types and pure calculation functions.
//!
//! Covers the MVP abilities (Shoot, First Aid, Sprint) plus Cover.
//! Contains no systems — those are wired separately.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::core::components::{Position, TileKind};
use crate::core::movement::MapTiles;

// ── Cover ──────────────────────────────────────────────────────────

/// How much cover a target has relative to an attacker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoverLevel {
    None,
    Half,
    Full,
}

impl CoverLevel {
    /// Accuracy modifier applied to attack rolls against this cover.
    pub fn modifier(&self) -> i32 {
        match self {
            CoverLevel::None => 0,
            CoverLevel::Half => -20,
            CoverLevel::Full => -40,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            CoverLevel::None => "None",
            CoverLevel::Half => "Half",
            CoverLevel::Full => "Full",
        }
    }
}

fn is_wall(map: &MapTiles, x: i32, y: i32) -> bool {
    map.get_tile(x, y) == Some(TileKind::Wall)
}

/// Determine cover level for `target` when attacked from `attacker` position.
///
/// Checks the tile adjacent to the target on the attacker's side (approach tile).
/// - Wall on approach → Full cover
/// - Wall on either flank of the approach → Half cover
/// - Otherwise → No cover
pub fn calc_cover(attacker: Position, target: Position, map: &MapTiles) -> CoverLevel {
    let dx = attacker.x - target.x;
    let dy = attacker.y - target.y;

    if dx == 0 && dy == 0 {
        return CoverLevel::None;
    }

    let sx = dx.signum();
    let sy = dy.signum();

    // Approach tile: one step from target toward attacker
    let ax = target.x + sx;
    let ay = target.y + sy;

    if is_wall(map, ax, ay) {
        return CoverLevel::Full;
    }

    // Flanking tiles depend on whether approach is cardinal or diagonal
    let (f1x, f1y, f2x, f2y) = if sx != 0 && sy != 0 {
        // Diagonal approach — flanks are the two cardinal components
        (target.x + sx, target.y, target.x, target.y + sy)
    } else if sx != 0 {
        // Horizontal approach — flanks are the two diagonals on that side
        (target.x + sx, target.y - 1, target.x + sx, target.y + 1)
    } else {
        // Vertical approach — flanks are the two diagonals on that side
        (target.x - 1, target.y + sy, target.x + 1, target.y + sy)
    };

    if is_wall(map, f1x, f1y) || is_wall(map, f2x, f2y) {
        return CoverLevel::Half;
    }

    CoverLevel::None
}

// ── Range Penalty ──────────────────────────────────────────────────

/// Returns the accuracy penalty for firing beyond optimal range.
///
/// -2 per tile beyond `optimal_range`. Returns 0 if within range.
pub fn calc_range_penalty(distance: i32, optimal_range: u8) -> i32 {
    let over = distance - optimal_range as i32;
    if over > 0 { over * -2 } else { 0 }
}

// ── Sprint ─────────────────────────────────────────────────────────

/// Cooldown tracker for the Sprint ability (3-turn cooldown).
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct SprintCooldown {
    pub remaining: u32,
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn floor_map(w: usize, h: usize) -> MapTiles {
        MapTiles::new(vec![vec![TileKind::Floor; w]; h])
    }

    #[test]
    fn test_cover_none_open_field() {
        let map = floor_map(5, 5);
        let attacker = Position::new(0, 2);
        let target = Position::new(4, 2);
        assert_eq!(calc_cover(attacker, target, &map), CoverLevel::None);
    }

    #[test]
    fn test_cover_half() {
        let mut map = floor_map(5, 5);
        // Attacker at (0,2), target at (4,2). Approach direction: (-1,0).
        // Approach tile (3,2) stays Floor. Flank tile (3,1) becomes Wall.
        map.tiles[1][3] = TileKind::Wall;
        let attacker = Position::new(0, 2);
        let target = Position::new(4, 2);
        assert_eq!(calc_cover(attacker, target, &map), CoverLevel::Half);
    }

    #[test]
    fn test_cover_full() {
        let mut map = floor_map(5, 5);
        // Attacker at (0,2), target at (4,2). Approach tile (3,2) becomes Wall.
        map.tiles[2][3] = TileKind::Wall;
        let attacker = Position::new(0, 2);
        let target = Position::new(4, 2);
        assert_eq!(calc_cover(attacker, target, &map), CoverLevel::Full);
    }

    #[test]
    fn test_range_penalty() {
        // Distance 8, optimal 6 → 2 tiles over → -4
        assert_eq!(calc_range_penalty(8, 6), -4);
    }

    #[test]
    fn test_range_no_penalty() {
        // Distance 3, optimal 6 → within range → 0
        assert_eq!(calc_range_penalty(3, 6), 0);
    }
}
