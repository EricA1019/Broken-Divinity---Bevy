//! Core ECS components for the BD Kernel.

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

/// Position on a 2D grid.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

/// Marks the player entity.
#[derive(Component, Debug, Default, Serialize, Deserialize)]
pub struct Player;

/// Blocks movement — entities with this cannot be walked through.
#[derive(Component, Debug, Default, Serialize, Deserialize)]
pub struct BlocksMovement;

/// Display name for an entity.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Name(pub String);

/// Stable content identity for data-driven entities such as dungeon loot.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct ContentIdentity(pub String);

/// Marks a tile as the exit point of a location.
#[derive(Component, Debug, Default, Serialize, Deserialize)]
pub struct ExitTile;

/// Marks an entity as Gabriel — the mysterious entity who appears in the shelter.
#[derive(Component, Debug, Default, Serialize, Deserialize)]
pub struct Gabriel;

/// Debug marker — entity ignores damage when present. Toggled via console `god on/off`.
#[derive(Component, Debug, Default, Serialize, Deserialize)]
pub struct GodMode;

/// Resource node on the shelter map that survivors can gather from.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct ResourceNode {
    pub source_id: String,
    pub kind: ResourceNodeType,
    pub depleted: bool,
}

/// Types of resource nodes available on the shelter map.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceNodeType {
    Trees,
    WaterSource,
    WildPlants,
}

/// A tile on the smoke map.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tile {
    Floor,
    Wall,
    Door,
    Water,
}

impl Tile {
    pub fn is_walkable(&self) -> bool {
        matches!(self, Tile::Floor | Tile::Door)
    }
}
