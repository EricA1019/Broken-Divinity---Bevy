//! Shared ECS components will live here as the MVP slices are implemented.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Tile types for dungeon and shelter maps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum TileKind {
    Wall,
    Floor,
    Door,
    StairsUp,
    StairsDown,
}

/// Marker component for the player entity.
#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct Player;

/// Marker component for enemy entities.
#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct Enemy;

/// Grid-based position used for movement, FOV, and pathfinding.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn to_ivec2(self) -> IVec2 {
        IVec2::new(self.x, self.y)
    }
}

impl From<IVec2> for Position {
    fn from(v: IVec2) -> Self {
        Self { x: v.x, y: v.y }
    }
}

/// Field-of-view component. Recalculated when the entity moves.
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct Viewshed {
    pub range: u32,
    pub visible_tiles: Vec<Position>, // Vec instead of HashSet for serde compat
    pub dirty: bool,
}

impl Viewshed {
    pub fn new(range: u32) -> Self {
        Self {
            range,
            visible_tiles: Vec::new(),
            dirty: true,
        }
    }

    pub fn contains(&self, pos: &Position) -> bool {
        self.visible_tiles.contains(pos)
    }
}
