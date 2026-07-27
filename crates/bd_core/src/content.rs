//! Typed foundation content definitions.
//!
//! The loader lives in `bd_data`; these types live in the runtime crate so
//! the application and headless simulation share the same content shape.

use bevy_ecs::prelude::Resource;
use serde::{Deserialize, Serialize};

use crate::{
    components::{Position, ResourceNodeType, Tile},
    factory::EntityBlueprint,
    signals::PoolKind,
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
    pub disposition: FoundationDisposition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FoundationDisposition {
    Hostile,
    Neutral,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColonyResourceDefinition {
    pub id: String,
    pub label: String,
    /// Finished resources map to an authoritative colony pool. Raw resources
    /// remain cargo until a recipe consumes them.
    pub pool_kind: Option<PoolKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColonySourceDefinition {
    pub id: String,
    pub label: String,
    pub node_type: ResourceNodeType,
    pub raw_resource_id: String,
    pub spawn_count: u32,
    pub glyph: char,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColonyRecipeDefinition {
    pub id: String,
    pub label: String,
    pub source_id: String,
    pub input_resource_id: String,
    pub output_resource_id: String,
    pub station_id: String,
    pub input_amount: u32,
    pub output_amount: u32,
    pub gather_work_turns: u32,
    pub refine_work_turns: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColonyPlacementProfile {
    pub id: String,
    pub minimum_manhattan_spacing: u32,
    pub seed_salt: u64,
}

/// Complete content bundle required by the foundation runtime.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct FoundationContent {
    pub dungeons: Vec<DungeonDefinition>,
    pub items: Vec<ItemDefinition>,
    pub skills: Vec<SkillDefinition>,
    pub factions: Vec<FactionDefinition>,
    pub actions: Vec<ActionReference>,
    pub stations: Vec<crate::colony::stations::StationBlueprint>,
    pub blueprints: Vec<EntityBlueprint>,
    pub colony_resources: Vec<ColonyResourceDefinition>,
    pub colony_sources: Vec<ColonySourceDefinition>,
    pub colony_recipes: Vec<ColonyRecipeDefinition>,
    pub colony_placement_profiles: Vec<ColonyPlacementProfile>,
}

impl FoundationContent {
    pub fn dungeon(&self, id: &str) -> Option<&DungeonDefinition> {
        self.dungeons.iter().find(|entry| entry.id == id)
    }
}
