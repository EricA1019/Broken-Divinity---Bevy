//! Core ECS components for the BD Kernel.

use bevy_ecs::prelude::*;

/// Position on a 2D grid.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

/// Marks the player entity.
#[derive(Component, Debug, Default)]
pub struct Player;

/// Blocks movement — entities with this cannot be walked through.
#[derive(Component, Debug, Default)]
pub struct BlocksMovement;

/// Display name for an entity.
#[derive(Component, Debug, Clone)]
pub struct Name(pub String);

/// A tile on the smoke map.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Floor,
    Wall,
}

impl Tile {
    pub fn is_walkable(&self) -> bool {
        matches!(self, Tile::Floor)
    }

    pub fn glyph(&self) -> char {
        match self {
            Tile::Floor => '.',
            Tile::Wall => '#',
        }
    }
}
