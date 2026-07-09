//! Grid map resource for the BD Kernel.

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::components::Tile;

/// The current map — a 2D grid of tiles.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct SmokeMap {
    pub width: i32,
    pub height: i32,
    tiles: Vec<Tile>,
}

impl SmokeMap {
    /// Create a new map filled with the given tile.
    pub fn new(width: i32, height: i32, fill: Tile) -> Self {
        let tiles = vec![fill; (width * height) as usize];
        Self {
            width,
            height,
            tiles,
        }
    }

    /// Get the tile at a position. Returns None if out of bounds.
    pub fn get(&self, x: i32, y: i32) -> Option<Tile> {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return None;
        }
        Some(self.tiles[(y * self.width + x) as usize])
    }

    /// Set the tile at a position. No-op if out of bounds.
    pub fn set(&mut self, x: i32, y: i32, tile: Tile) {
        if x >= 0 && x < self.width && y >= 0 && y < self.height {
            self.tiles[(y * self.width + x) as usize] = tile;
        }
    }

    /// Check if a position is within bounds and walkable.
    pub fn is_walkable(&self, x: i32, y: i32) -> bool {
        self.get(x, y).is_some_and(|t| t.is_walkable())
    }

    /// Create the default Phase 1 smoke map: a bordered room.
    pub fn default_smoke_map() -> Self {
        const W: i32 = 20;
        const H: i32 = 12;
        let mut map = Self::new(W, H, Tile::Floor);

        // Walls around the border
        for x in 0..W {
            map.set(x, 0, Tile::Wall);
            map.set(x, H - 1, Tile::Wall);
        }
        for y in 0..H {
            map.set(0, y, Tile::Wall);
            map.set(W - 1, y, Tile::Wall);
        }

        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_smoke_map_has_wall_border() {
        let map = SmokeMap::default_smoke_map();
        assert_eq!(map.get(0, 0), Some(Tile::Wall));
        assert_eq!(map.get(19, 11), Some(Tile::Wall));
        assert_eq!(map.get(10, 6), Some(Tile::Floor));
    }

    #[test]
    fn get_out_of_bounds_returns_none() {
        let map = SmokeMap::default_smoke_map();
        assert_eq!(map.get(-1, 0), None);
        assert_eq!(map.get(20, 0), None);
    }

    #[test]
    fn is_walkable_checks_bounds_and_tile() {
        let map = SmokeMap::default_smoke_map();
        assert!(!map.is_walkable(0, 0)); // wall
        assert!(map.is_walkable(5, 5)); // floor
        assert!(!map.is_walkable(-1, 0)); // oob
    }
}
