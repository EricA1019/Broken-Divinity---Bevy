//! Enemy data definitions and spawning for dungeon encounters.

use std::collections::HashMap;

use bevy::prelude::*;

use crate::core::components::{Enemy, Position};
use crate::core::stats::{CombatStats, EntityName, SkillId, SkillState};
use crate::core::status::StatusEffects;
use crate::core::turn::ActionBudget;
use crate::game::combat::DamageType;
use crate::game::dungeon::theme::DungeonTheme;

/// Optional ranged-attack data baked into an enemy definition.
#[derive(Debug, Clone, Copy)]
pub struct RangedEnemyData {
    pub range: i32,
    pub skill: u32,
    pub damage: i32,
    pub damage_type: DamageType,
}

/// Marker component for enemies that can shoot at range.
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct RangedEnemy {
    pub range: i32,
    pub skill: u32,
    pub damage: i32,
    pub damage_type: DamageType,
}

/// Melee-attack profile attached to spawned enemies.
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct MeleeEnemyAttack {
    pub damage: i32,
    pub damage_type: DamageType,
}

/// Static definition for an enemy type.
#[derive(Debug, Clone)]
pub struct EnemyDef {
    pub name: &'static str,
    pub glyph: char,
    pub hp: i32,
    pub speed: u8,
    pub ar: i32,
    pub md: i32,
    pub melee_skill: u32,
    pub melee_damage: i32,
    pub damage_type: DamageType,
    pub color: Color,
    pub ranged: Option<RangedEnemyData>,
}

/// Returns the enemy spawn table for a given dungeon theme.
pub fn spawn_table(theme: DungeonTheme) -> &'static [EnemyDef] {
    match theme {
        DungeonTheme::UrbanDecay => &URBAN_DECAY_ENEMIES,
        DungeonTheme::Underground => &UNDERGROUND_ENEMIES,
        DungeonTheme::Military => &MILITARY_ENEMIES,
    }
}

static URBAN_DECAY_ENEMIES: [EnemyDef; 3] = [
    EnemyDef {
        name: "Feral Scavenger",
        glyph: 's',
        hp: 15,
        speed: 1,
        ar: 2,
        md: 0,
        melee_skill: 30,
        melee_damage: 5,
        damage_type: DamageType::Slash,
        color: Color::srgb(0.7, 0.4, 0.2),
        ranged: None,
    },
    EnemyDef {
        name: "Rubble Lurker",
        glyph: 'l',
        hp: 25,
        speed: 1,
        ar: 5,
        md: 0,
        melee_skill: 35,
        melee_damage: 8,
        damage_type: DamageType::Blunt,
        color: Color::srgb(0.5, 0.5, 0.3),
        ranged: None,
    },
    EnemyDef {
        name: "Veil-Touched Vagrant",
        glyph: 'v',
        hp: 12,
        speed: 2,
        ar: 1,
        md: 5,
        melee_skill: 25,
        melee_damage: 6,
        damage_type: DamageType::Thaumic,
        color: Color::srgb(0.6, 0.3, 0.8),
        ranged: None,
    },
];

static UNDERGROUND_ENEMIES: [EnemyDef; 3] = [
    EnemyDef {
        name: "Tunnel Crawler",
        glyph: 'c',
        hp: 18,
        speed: 1,
        ar: 4,
        md: 0,
        melee_skill: 30,
        melee_damage: 6,
        damage_type: DamageType::Slash,
        color: Color::srgb(0.3, 0.5, 0.3),
        ranged: None,
    },
    EnemyDef {
        name: "Blighted Mole-Rat",
        glyph: 'r',
        hp: 10,
        speed: 2,
        ar: 1,
        md: 2,
        melee_skill: 20,
        melee_damage: 4,
        damage_type: DamageType::Slash,
        color: Color::srgb(0.4, 0.3, 0.2),
        ranged: None,
    },
    EnemyDef {
        name: "Cave Brute",
        glyph: 'B',
        hp: 35,
        speed: 1,
        ar: 8,
        md: 0,
        melee_skill: 40,
        melee_damage: 12,
        damage_type: DamageType::Blunt,
        color: Color::srgb(0.5, 0.4, 0.4),
        ranged: None,
    },
];

static MILITARY_ENEMIES: [EnemyDef; 3] = [
    EnemyDef {
        name: "Rogue Militiaman",
        glyph: 'm',
        hp: 20,
        speed: 1,
        ar: 6,
        md: 0,
        melee_skill: 35,
        melee_damage: 7,
        damage_type: DamageType::Ballistic,
        color: Color::srgb(0.3, 0.5, 0.3),
        ranged: Some(RangedEnemyData {
            range: 6,
            skill: 30,
            damage: 6,
            damage_type: DamageType::Ballistic,
        }),
    },
    EnemyDef {
        name: "Deserter",
        glyph: 'd',
        hp: 15,
        speed: 1,
        ar: 4,
        md: 0,
        melee_skill: 25,
        melee_damage: 5,
        damage_type: DamageType::Slash,
        color: Color::srgb(0.4, 0.4, 0.3),
        ranged: None,
    },
    EnemyDef {
        name: "Armored Sentinel",
        glyph: 'S',
        hp: 40,
        speed: 1,
        ar: 12,
        md: 2,
        melee_skill: 45,
        melee_damage: 10,
        damage_type: DamageType::Blunt,
        color: Color::srgb(0.6, 0.6, 0.6),
        ranged: None,
    },
];

/// Spawns an enemy entity from a definition at the given position.
pub fn spawn_enemy(commands: &mut Commands, def: &EnemyDef, x: i32, y: i32) -> Entity {
    let mut skills = HashMap::new();
    skills.insert(
        SkillId::Melee,
        SkillState {
            base: def.melee_skill,
            xp: 0,
            level: 0,
        },
    );
    if let Some(r) = &def.ranged {
        skills.insert(
            SkillId::Ranged,
            SkillState {
                base: r.skill,
                xp: 0,
                level: 0,
            },
        );
    }

    let mut cmd = commands.spawn((
        Enemy,
        EntityName {
            name: def.name.to_string(),
        },
        Position::new(x, y),
        CombatStats {
            hp: def.hp,
            hp_max: def.hp,
            speed: def.speed,
            ar: def.ar,
            md: def.md,
            skills,
        },
        ActionBudget::new(def.speed),
        StatusEffects::default(),
        MeleeEnemyAttack {
            damage: def.melee_damage,
            damage_type: def.damage_type,
        },
        Sprite {
            color: def.color,
            custom_size: Some(Vec2::new(16.0, 16.0)),
            ..Default::default()
        },
    ));

    if let Some(r) = &def.ranged {
        cmd.insert(RangedEnemy {
            range: r.range,
            skill: r.skill,
            damage: r.damage,
            damage_type: r.damage_type,
        });
    }

    cmd.id()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::world::CommandQueue;

    #[test]
    fn test_spawn_enemy_inserts_melee_profile() {
        let mut world = World::new();
        let mut queue = CommandQueue::default();
        let entity = {
            let mut commands = Commands::new(&mut queue, &world);
            spawn_enemy(&mut commands, &UNDERGROUND_ENEMIES[1], 4, 7)
        };
        queue.apply(&mut world);

        let attack = world
            .get::<MeleeEnemyAttack>(entity)
            .expect("spawned enemy should carry melee profile");
        assert_eq!(attack.damage, UNDERGROUND_ENEMIES[1].melee_damage);
        assert_eq!(attack.damage_type, UNDERGROUND_ENEMIES[1].damage_type);
    }
}
