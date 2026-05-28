//! Dungeon theme overlay.
//!
//! Maps `TileKind` → tile atlas index per theme. Applied to `DungeonFloor` tiles
//! after generation to give each dungeon a distinct visual identity.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::core::components::TileKind;

/// Available dungeon themes. Only UrbanDecay is in scope for MVP.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum DungeonTheme {
    UrbanDecay,
    Underground,
    Military,
}

impl DungeonTheme {
    pub fn name(&self) -> &'static str {
        match self {
            Self::UrbanDecay => "Urban Decay",
            Self::Underground => "Underground",
            Self::Military => "Military",
        }
    }

    /// Returns the tile atlas index for a given `TileKind`.
    pub fn atlas_index(&self, tile: TileKind) -> u32 {
        match self {
            Self::UrbanDecay => match tile {
                TileKind::Wall => 0,
                TileKind::Floor => 1,
                TileKind::Door => 2,
                TileKind::StairsUp => 3,
                TileKind::StairsDown => 4,
            },
            Self::Underground => match tile {
                TileKind::Wall => 5,
                TileKind::Floor => 6,
                TileKind::Door => 7,
                TileKind::StairsUp => 8,
                TileKind::StairsDown => 9,
            },
            Self::Military => match tile {
                TileKind::Wall => 10,
                TileKind::Floor => 11,
                TileKind::Door => 12,
                TileKind::StairsUp => 13,
                TileKind::StairsDown => 14,
            },
        }
    }
}
