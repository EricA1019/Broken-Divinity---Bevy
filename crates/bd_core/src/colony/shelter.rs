//! Shelter map — the persistent outpost map stored in OutpostState.
//!
//! The shelter is a 40×30 grid (SHELTER_WIDTH × SHELTER_HEIGHT) with a wall
//! border and a floor interior. This map persists across transitions and is
//! never regenerated.

use crate::components::Tile;
use crate::map::SmokeMap;

// ── Constants ──

/// Width of the shelter map in tiles.
pub const SHELTER_WIDTH: i32 = 40;

/// Height of the shelter map in tiles.
pub const SHELTER_HEIGHT: i32 = 30;

// ── Map generation ──

/// Generate the default shelter map: wall border, floor interior.
pub fn create_shelter_map() -> SmokeMap {
    let mut map = SmokeMap::new(SHELTER_WIDTH, SHELTER_HEIGHT, Tile::Floor);

    // Top and bottom walls
    for x in 0..SHELTER_WIDTH {
        map.set(x, 0, Tile::Wall);
        map.set(x, SHELTER_HEIGHT - 1, Tile::Wall);
    }

    // Left and right walls
    for y in 0..SHELTER_HEIGHT {
        map.set(0, y, Tile::Wall);
        map.set(SHELTER_WIDTH - 1, y, Tile::Wall);
    }

    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shelter_map_has_correct_dimensions() {
        let map = create_shelter_map();
        assert_eq!(map.width, SHELTER_WIDTH);
        assert_eq!(map.height, SHELTER_HEIGHT);
    }

    #[test]
    fn shelter_map_is_walkable_interior() {
        let map = create_shelter_map();
        // Center should be walkable floor
        assert!(map.is_walkable(SHELTER_WIDTH / 2, SHELTER_HEIGHT / 2));
    }

    #[test]
    fn shelter_map_has_wall_border() {
        let map = create_shelter_map();
        // Top-left corner is wall
        assert!(!map.is_walkable(0, 0));
        // Top-right corner is wall
        assert!(!map.is_walkable(SHELTER_WIDTH - 1, 0));
        // Bottom-left corner is wall
        assert!(!map.is_walkable(0, SHELTER_HEIGHT - 1));
        // Bottom-right corner is wall
        assert!(!map.is_walkable(SHELTER_WIDTH - 1, SHELTER_HEIGHT - 1));
    }

    #[test]
    fn center_of_shelter_is_floor() {
        let map = create_shelter_map();
        let tile = map.get(SHELTER_WIDTH / 2, SHELTER_HEIGHT / 2);
        assert_eq!(tile, Some(Tile::Floor));
    }
}
