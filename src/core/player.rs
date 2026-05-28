//! Player entity spawning and management.

use std::collections::HashMap;

use crate::core::abilities::SprintCooldown;
use crate::core::components::{Player, Position, Viewshed};
use crate::core::inventory::{Equipment, Inventory, RangedWeaponState};
use crate::core::perks::PlayerPerks;
use crate::core::sanity::RaidExposure;
use crate::core::stats::{CombatStats, EntityName, PlayerProgression};
use crate::core::status::StatusEffects;
use crate::core::turn::ActionBudget;
use bevy::prelude::*;

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
    pub progression: PlayerProgression,
    pub sprint_cooldown: SprintCooldown,
    pub sprite: Sprite,
}

impl PlayerBundle {
    pub fn new(x: i32, y: i32) -> Self {
        let progression = PlayerProgression::new_game();
        let mut skills = HashMap::new();
        progression.sync_pilot_combat_skill_proxies(&mut skills);
        // Only the three canonical pilot proxies are inserted here.
        // Toughness and Awareness are retired; their meaning now lives in
        // Fortitude virtue and the canonical proficiency system.

        Self {
            player: Player,
            position: Position::new(x, y),
            viewshed: Viewshed::new(8),
            name: EntityName {
                name: "Player".to_string(),
            },
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
            progression,
            sprint_cooldown: SprintCooldown { remaining: 0 },
            sprite: Sprite {
                color: Color::srgb(0.2, 0.8, 0.2),
                custom_size: Some(Vec2::new(16.0, 16.0)),
                ..Default::default()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::stats::{ProficiencyId, SkillId, VirtueId};

    #[test]
    fn test_player_bundle_new_does_not_insert_retired_skill_proxies() {
        // Toughness and Awareness are retired skills whose meaning is now
        // carried by Fortitude virtue and canonical proficiencies.
        // New games must not populate these slots — they have no production
        // reader and would silently litter the skills map.
        let bundle = PlayerBundle::new(0, 0);
        assert!(
            !bundle.stats.skills.contains_key(&SkillId::Toughness),
            "SkillId::Toughness must not be inserted into a new player's skills"
        );
        assert!(
            !bundle.stats.skills.contains_key(&SkillId::Awareness),
            "SkillId::Awareness must not be inserted into a new player's skills"
        );
    }

    #[test]
    fn test_player_bundle_new_aligns_pilot_skills_with_progression() {
        let bundle = PlayerBundle::new(3, 7);

        assert_eq!(
            bundle.stats.skill_level(SkillId::Melee),
            bundle
                .progression
                .action_rating(VirtueId::Thumos, ProficiencyId::MeleeTraining, 0, 0)
                as u32
        );
        assert_eq!(
            bundle.stats.skill_level(SkillId::Ranged),
            bundle.progression.action_rating(
                VirtueId::Prudence,
                ProficiencyId::RangedTraining,
                0,
                0
            ) as u32
        );
        assert_eq!(
            bundle.stats.skill_level(SkillId::Evasion),
            bundle.progression.enemy_attack_dv() as u32
        );
    }
}
