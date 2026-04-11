//! Player entity spawning and management.

use std::collections::HashMap;

use bevy::prelude::*;
use crate::core::abilities::SprintCooldown;
use crate::core::components::{Player, Position, Viewshed};
use crate::core::inventory::{Equipment, Inventory, RangedWeaponState};
use crate::core::perks::PlayerPerks;
use crate::core::sanity::RaidExposure;
use crate::core::stats::{CombatStats, EntityName, SkillId, SkillState};
use crate::core::status::StatusEffects;
use crate::core::turn::ActionBudget;

/// Bundle of components required for the player entity.
#[derive(Bundle)]
pub struct PlayerBundle {
    pub player: Player,
    pub position: Position,
    pub viewshed: Viewshed,
    pub name: EntityName,
    pub stats: CombatStats,
    pub budget: ActionBudget,
    pub statuses: StatusEffects,
    pub inventory: Inventory,
    pub equipment: Equipment,
    pub ranged_state: RangedWeaponState,
    pub sanity: RaidExposure,
    pub perks: PlayerPerks,
    pub sprint_cooldown: SprintCooldown,
    pub sprite: Sprite,
}

impl PlayerBundle {
    pub fn new(x: i32, y: i32) -> Self {
        let mut skills = HashMap::new();
        skills.insert(SkillId::Melee, SkillState { base: 40, xp: 0, level: 0 });
        skills.insert(SkillId::Ranged, SkillState { base: 30, xp: 0, level: 0 });
        skills.insert(SkillId::Evasion, SkillState { base: 25, xp: 0, level: 0 });
        skills.insert(SkillId::Toughness, SkillState { base: 30, xp: 0, level: 0 });
        skills.insert(SkillId::Awareness, SkillState { base: 25, xp: 0, level: 0 });

        Self {
            player: Player,
            position: Position::new(x, y),
            viewshed: Viewshed::new(8),
            name: EntityName { name: "Player".to_string() },
            stats: CombatStats {
                hp: 50,
                hp_max: 50,
                speed: 1,
                ar: 5,
                md: 2,
                skills,
            },
            budget: ActionBudget::new(1),
            statuses: StatusEffects::default(),
            inventory: Inventory::default(),
            equipment: Equipment {
                weapon: Some("iron_pipe".to_string()),
                armor: None,
                accessory: None,
            },
            ranged_state: RangedWeaponState {
                clip_current: 0,
                clip_size: 0,
            },
            sanity: RaidExposure::default(),
            perks: PlayerPerks::default(),
            sprint_cooldown: SprintCooldown { remaining: 0 },
            sprite: Sprite {
                color: Color::srgb(0.2, 0.8, 0.2),
                custom_size: Some(Vec2::new(16.0, 16.0)),
                ..Default::default()
            },
        }
    }
}
