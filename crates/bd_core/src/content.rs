//! Typed foundation content definitions.
//!
//! The loader lives in `bd_data`; these types live in the runtime crate so
//! the application and headless simulation share the same content shape.

use bevy_ecs::prelude::Resource;
use serde::{Deserialize, Serialize};

use crate::{
    components::{Position, Tile},
    factory::EntityBlueprint,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DungeonDefinition {
    pub id: String,
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<Tile>,
    pub entrance: Position,
    pub extraction: Position,
    pub enemy_placements: Vec<Placement>,
    pub item_placements: Vec<Placement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Placement {
    pub content_id: String,
    pub position: Position,
    #[serde(default)]
    pub faction_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemDefinition {
    pub id: String,
    pub label: String,
    pub blueprint_id: String,
    pub usable: bool,
    pub healing_amount: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDefinition {
    pub id: String,
    pub label: String,
    pub action_id: String,
    pub governing_virtue: String,
    pub progression_rate: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionDefinition {
    pub id: String,
    pub label: String,
    pub identity_key: String,
    pub hostility: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionReference {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub skill_id: Option<String>,
    #[serde(default)]
    pub skill_gain: i32,
    #[serde(default)]
    pub virtue_expression: Option<String>,
    #[serde(default)]
    pub virtue_gain: i32,
}

/// Complete content bundle required by the foundation runtime.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct FoundationContent {
    pub dungeons: Vec<DungeonDefinition>,
    pub items: Vec<ItemDefinition>,
    pub skills: Vec<SkillDefinition>,
    pub factions: Vec<FactionDefinition>,
    pub actions: Vec<ActionReference>,
    pub blueprints: Vec<EntityBlueprint>,
}

impl FoundationContent {
    pub fn dungeon(&self, id: &str) -> Option<&DungeonDefinition> {
        self.dungeons.iter().find(|entry| entry.id == id)
    }
}
