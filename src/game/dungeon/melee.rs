#![allow(clippy::too_many_arguments, clippy::type_complexity)]

//! Melee combat resolution — processes bump-to-attack interactions.

use bevy::prelude::*;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::core::components::{Enemy, Player, Position};
use crate::core::escape;
use crate::core::gamelog::{GameLog, LogColor};
use crate::core::inventory::Equipment;
use crate::core::items::find_item;
use crate::core::perks::{
    PendingPerkChoices, PlayerPerks, apply_second_wind, queue_progression_lane_perks,
};
use crate::core::sanity::{Hallucination, RaidExposure, SanityEvent, apply_player_sanity_event};
use crate::core::state::AppState;
use crate::core::stats::calc_xp_award;
use crate::core::stats::{
    CombatStats, EntityName, PlayerProgression, ProficiencyId, SkillId, VirtueId,
};
use crate::core::status::{StatusEffects, StatusKind};
use crate::core::turn::GameTime;
use crate::game::combat::{self, DamageType};
use crate::game::dungeon::enemies::MeleeEnemyAttack;
use crate::game::dungeon::gabriel::GabrielDialogueState;

/// Resource tracking the world RNG for combat rolls.
/// Seeded from `WorldSeed` on dungeon entry to ensure per-run determinism.
#[derive(Resource)]
pub struct CombatRng(pub ChaCha8Rng);

impl Default for CombatRng {
    fn default() -> Self {
        // Seed 0 is a placeholder; reseed_from() is called during setup_dungeon
        // using the run's WorldSeed. Tests that use default() get a fixed seed.
        Self(ChaCha8Rng::seed_from_u64(0))
    }
}

impl CombatRng {
    /// Re-seed the RNG from the world seed for deterministic per-run combat.
    pub fn reseed_from(&mut self, world_seed: u64) {
        self.0 = ChaCha8Rng::seed_from_u64(world_seed.wrapping_mul(0x4A7C_15E9));
    }
}

fn player_melee_profile(equipment: &Equipment) -> (i32, DamageType) {
    let Some(weapon_id) = equipment.weapon.as_deref() else {
        return (4, DamageType::Blunt);
    };
    let Some(item) = find_item(weapon_id) else {
        return (4, DamageType::Blunt);
    };
    let Some(weapon) = item.weapon.as_ref() else {
        return (4, DamageType::Blunt);
    };

    if weapon.range == 0 {
        (weapon.damage, weapon.damage_type)
    } else {
        (4, DamageType::Blunt)
    }
}

fn player_melee_action_rating(progression: &PlayerProgression) -> u32 {
    progression
        .action_rating(VirtueId::Thumos, ProficiencyId::MeleeTraining, 0, 0)
        .max(0) as u32
}

fn player_melee_action_breakdown(
    progression: &PlayerProgression,
    action_rating: u32,
    total_mod: i32,
) -> String {
    let virtue_bonus = i32::from(progression.virtue_rank(VirtueId::Thumos)) * 5;
    let training = progression.proficiency_rating(ProficiencyId::MeleeTraining) as i32;
    format!(
        " [AR {} = THU {} + TRN {} {:+}]",
        action_rating, virtue_bonus, training, total_mod
    )
}

fn player_defense_dv(progression: &PlayerProgression, player_perks: &PlayerPerks) -> i32 {
    progression.enemy_attack_dv() + player_perks.enemy_attack_dv_bonus(false)
}

fn player_defense_breakdown(
    progression: &PlayerProgression,
    player_perks: &PlayerPerks,
    defense_dv: i32,
) -> String {
    let virtue_bonus = i32::from(progression.virtue_rank(VirtueId::Metis)) * 5;
    let quiet_movement = progression.proficiency_rating(ProficiencyId::QuietMovement) as i32;
    let perk_bonus = player_perks.enemy_attack_dv_bonus(false);
    if perk_bonus > 0 {
        format!(
            " [DV {} = MET {} + QUI {} + PRK {}]",
            defense_dv, virtue_bonus, quiet_movement, perk_bonus
        )
    } else {
        format!(
            " [DV {} = MET {} + QUI {}]",
            defense_dv, virtue_bonus, quiet_movement
        )
    }
}

fn enemy_melee_check_breakdown(
    skill_level: u32,
    progression: &PlayerProgression,
    player_perks: &PlayerPerks,
    defense_dv: i32,
) -> String {
    let defense_breakdown = player_defense_breakdown(progression, player_perks, defense_dv)
        .trim_start_matches(' ')
        .trim_start_matches('[')
        .trim_end_matches(']')
        .to_string();
    format!(" [ATK {} vs {}]", skill_level, defense_breakdown)
}

/// System: resolve player melee attacks against enemies.
///
/// Called when the player bumps into an enemy tile during movement.
/// This is triggered by `bump_attack_check` in `movement.rs`.
pub fn resolve_player_melee(
    mut commands: Commands,
    mut player_q: Query<
        (
            &Position,
            &Equipment,
            &mut RaidExposure,
            &PlayerPerks,
            &mut PlayerProgression,
        ),
        With<Player>,
    >,
    mut enemy_q: Query<
        (
            Entity,
            &Position,
            &mut CombatStats,
            &EntityName,
            &mut StatusEffects,
            Option<&Hallucination>,
        ),
        (With<Enemy>, Without<Player>),
    >,
    mut log: ResMut<GameLog>,
    mut rng: ResMut<CombatRng>,
    game_time: Res<GameTime>,
    mut bump_target: ResMut<BumpAttackTarget>,
    mut pending_perks: ResMut<PendingPerkChoices>,
    dialogue_state: Option<Res<GabrielDialogueState>>,
) {
    if dialogue_state.is_some_and(|dialogue| dialogue.is_active()) {
        bump_target.0 = None;
        return;
    }

    let Some(target_pos) = bump_target.0.take() else {
        return;
    };
    let Ok((_player_pos, equipment, mut raid_exposure, player_perks, mut progression)) =
        player_q.single_mut()
    else {
        return;
    };

    let skill_level = player_melee_action_rating(&progression);
    let total_mod = raid_exposure.threshold().penalty();
    let action_breakdown = player_melee_action_breakdown(&progression, skill_level, total_mod);
    let (weapon_damage, damage_type) = player_melee_profile(equipment);

    for (entity, enemy_pos, mut enemy_stats, enemy_name, _enemy_status, hallucination) in
        enemy_q.iter_mut()
    {
        if enemy_pos.x != target_pos.x || enemy_pos.y != target_pos.y {
            continue;
        }

        let enemy_skill = enemy_stats
            .skill_level(SkillId::Melee)
            .max(enemy_stats.skill_level(SkillId::Ranged));
        let check = combat::roll_check(skill_level, total_mod, 0, &mut rng.0);

        let xp = calc_xp_award(check.success, check.critical, enemy_skill, skill_level);
        if xp > 0 {
            if progression
                .grant_proficiency_xp(ProficiencyId::MeleeTraining, xp)
                .is_some()
            {
                log.push(
                    format!(
                        "{} rises to {}.",
                        ProficiencyId::MeleeTraining.name(),
                        progression.proficiency_rating(ProficiencyId::MeleeTraining)
                    ),
                    LogColor::Status,
                    game_time.turn,
                );
            }

            for perk in queue_progression_lane_perks(
                &mut pending_perks,
                player_perks,
                VirtueId::Thumos,
                &progression,
                ProficiencyId::MeleeTraining,
            ) {
                log.push(
                    format!(
                        "New perk available: {} [{}].",
                        perk.name(),
                        perk.lane_label()
                    ),
                    LogColor::Status,
                    game_time.turn,
                );
            }
        }

        if check.fumble {
            log.push(
                format!(
                    "You fumble your attack on {}!{}",
                    enemy_name.name, action_breakdown
                ),
                LogColor::Miss,
                game_time.turn,
            );
        } else if check.success {
            if hallucination.is_some() {
                log.push(
                    "Your swing tears through empty air and the vision vanishes.",
                    LogColor::Status,
                    game_time.turn,
                );
                commands.entity(entity).despawn();
            } else {
                let damage = combat::calc_damage(
                    weapon_damage + player_perks.melee_damage_bonus(),
                    skill_level,
                    enemy_stats.ar,
                    enemy_stats.md,
                    damage_type,
                    check.critical,
                    &mut rng.0,
                );
                enemy_stats.hp -= damage;

                if check.critical {
                    log.push(
                        format!(
                            "CRITICAL! You strike {} for {} damage!{}",
                            enemy_name.name, damage, action_breakdown
                        ),
                        LogColor::Critical,
                        game_time.turn,
                    );
                } else {
                    log.push(
                        format!(
                            "You hit {} for {} damage.{}",
                            enemy_name.name, damage, action_breakdown
                        ),
                        LogColor::PlayerHit,
                        game_time.turn,
                    );
                }

                if enemy_stats.is_dead() {
                    let crossed_threshold = apply_player_sanity_event(
                        &mut raid_exposure,
                        player_perks,
                        SanityEvent::Kill,
                    );
                    log.push(
                        format!("{} is killed!", enemy_name.name),
                        LogColor::Death,
                        game_time.turn,
                    );
                    if crossed_threshold {
                        log.push(
                            format!("Sanity frays: {}.", raid_exposure.threshold().name()),
                            LogColor::Status,
                            game_time.turn,
                        );
                    }
                    commands.entity(entity).despawn();
                }
            }
        } else {
            log.push(
                format!("You miss {}.{}", enemy_name.name, action_breakdown),
                LogColor::Miss,
                game_time.turn,
            );
        }

        break; // Only attack one enemy per bump
    }
}

/// System: resolve enemy melee attacks against the player.
///
/// Runs during `TurnPhase::EnemyTurn`.
pub fn resolve_enemy_melee(
    mut commands: Commands,
    mut player_q: Query<
        (
            &Position,
            &mut CombatStats,
            &mut StatusEffects,
            &mut RaidExposure,
            &PlayerPerks,
            &mut PlayerProgression,
        ),
        With<Player>,
    >,
    enemy_q: Query<
        (&Position, &CombatStats, &EntityName, &MeleeEnemyAttack),
        (With<Enemy>, Without<Player>, Without<Hallucination>),
    >,
    mut log: ResMut<GameLog>,
    mut rng: ResMut<CombatRng>,
    game_time: Res<GameTime>,
    mut pending_perks: ResMut<PendingPerkChoices>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    let Ok((
        player_pos,
        mut player_stats,
        mut player_status,
        mut raid_exposure,
        player_perks,
        mut progression,
    )) = player_q.single_mut()
    else {
        return;
    };
    if player_stats.is_dead() {
        return;
    }

    for (enemy_pos, enemy_stats, enemy_name, melee_attack) in enemy_q.iter() {
        if player_stats.is_dead() {
            return;
        }
        let dx = (player_pos.x - enemy_pos.x).abs();
        let dy = (player_pos.y - enemy_pos.y).abs();

        // Only attack if adjacent
        if dx > 1 || dy > 1 || (dx == 0 && dy == 0) {
            continue;
        }

        let skill_level = enemy_stats.skill_level(SkillId::Melee);
        let defense_dv = player_defense_dv(&progression, player_perks);
        let check_breakdown =
            enemy_melee_check_breakdown(skill_level, &progression, player_perks, defense_dv);
        let check = combat::roll_check(skill_level, 0, defense_dv, &mut rng.0);

        if check.success {
            let previous_hp = player_stats.hp;
            let damage = combat::calc_damage(
                melee_attack.damage,
                skill_level,
                player_stats.ar + player_perks.ar_bonus(),
                player_stats.md,
                melee_attack.damage_type,
                check.critical,
                &mut rng.0,
            );
            player_stats.hp -= damage;
            let crossed_threshold =
                apply_player_sanity_event(&mut raid_exposure, player_perks, SanityEvent::CombatHit);

            if check.critical {
                log.push(
                    format!(
                        "CRITICAL! {} strikes you for {} damage!{}",
                        enemy_name.name, damage, check_breakdown
                    ),
                    LogColor::Critical,
                    game_time.turn,
                );
            } else {
                log.push(
                    format!(
                        "{} hits you for {} damage.{}",
                        enemy_name.name, damage, check_breakdown
                    ),
                    LogColor::EnemyHit,
                    game_time.turn,
                );
            }

            if crossed_threshold {
                log.push(
                    format!("Sanity frays: {}.", raid_exposure.threshold().name()),
                    LogColor::Status,
                    game_time.turn,
                );
            }

            if apply_second_wind(&mut player_stats, player_perks, previous_hp) {
                log.push(
                    "Second Wind keeps you standing.",
                    LogColor::Status,
                    game_time.turn,
                );
            }
            if player_stats.is_dead() {
                escape::queue_game_over(
                    &mut commands,
                    next_app_state.as_mut(),
                    &mut log,
                    game_time.turn,
                );
                return;
            }

            // Chance to apply Wounded (20% per hit)
            if rng.0.random_range(1..=100u32) <= 20 {
                player_status.add(StatusKind::Wounded, 3);
                log.push(
                    format!("{} inflicts a wound!", enemy_name.name),
                    LogColor::Status,
                    game_time.turn,
                );
            }
        } else {
            log.push(
                format!("{} misses you.{}", enemy_name.name, check_breakdown),
                LogColor::Miss,
                game_time.turn,
            );
        }

        let defense_xp = progression.enemy_attack_xp_award(!check.success, skill_level)
            + player_perks.enemy_attack_xp_bonus(!check.success);
        if defense_xp > 0
            && progression
                .grant_proficiency_xp(ProficiencyId::QuietMovement, defense_xp)
                .is_some()
        {
            log.push(
                format!(
                    "{} rises to {}.",
                    ProficiencyId::QuietMovement.name(),
                    progression.proficiency_rating(ProficiencyId::QuietMovement)
                ),
                LogColor::Status,
                game_time.turn,
            );

            for perk in queue_progression_lane_perks(
                &mut pending_perks,
                player_perks,
                VirtueId::Metis,
                &progression,
                ProficiencyId::QuietMovement,
            ) {
                log.push(
                    format!(
                        "New perk available: {} [{}].",
                        perk.name(),
                        perk.lane_label()
                    ),
                    LogColor::Status,
                    game_time.turn,
                );
            }
        }
    }
}

/// Resource: set by movement system when player bumps into an enemy.
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct BumpAttackTarget(pub Option<Position>);

use rand::RngExt;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::gamelog::GameLog;
    use crate::ui::gameover::DeathSummary;
    use bevy::ecs::system::RunSystemOnce;
    use std::collections::HashMap;

    #[test]
    fn test_player_melee_profile_uses_equipped_melee_weapon() {
        let equipment = Equipment {
            weapon: Some("iron_pipe".to_string()),
            armor: None,
            accessory: None,
        };

        assert_eq!(player_melee_profile(&equipment), (6, DamageType::Blunt));
    }

    #[test]
    fn test_player_melee_profile_falls_back_for_ranged_weapon() {
        let equipment = Equipment {
            weapon: Some("makeshift_pistol".to_string()),
            armor: None,
            accessory: None,
        };

        assert_eq!(player_melee_profile(&equipment), (4, DamageType::Blunt));
    }

    #[test]
    fn test_player_melee_action_rating_prefers_progression() {
        let progression = PlayerProgression::new_game();

        assert_eq!(player_melee_action_rating(&progression), 22);
    }

    #[test]
    fn test_player_melee_action_breakdown_formats_expected_terms() {
        let progression = PlayerProgression::new_game();
        let breakdown = player_melee_action_breakdown(&progression, 22, -3);

        assert!(breakdown.contains("AR 22"));
        assert!(breakdown.contains("THU 10"));
        assert!(breakdown.contains("TRN 12"));
        assert!(breakdown.contains("-3"));
    }

    #[test]
    fn test_player_defense_dv_prefers_progression_lane() {
        let progression = PlayerProgression::new_game();
        let perks = PlayerPerks::default();

        assert_eq!(player_defense_dv(&progression, &perks), 11);
    }

    #[test]
    fn test_player_defense_dv_does_not_use_legacy_when_progression_present() {
        let progression = PlayerProgression::default();
        let perks = PlayerPerks::default();

        assert_eq!(player_defense_dv(&progression, &perks), 0);
    }

    #[test]
    fn test_player_defense_breakdown_formats_expected_terms() {
        let progression = PlayerProgression::new_game();
        let breakdown = player_defense_breakdown(&progression, &PlayerPerks::default(), 11);

        assert!(breakdown.contains("DV 11"));
        assert!(breakdown.contains("MET 5"));
        assert!(breakdown.contains("QUI 6"));
    }

    #[test]
    fn test_player_defense_breakdown_keeps_progression_terms_with_empty_component() {
        let progression = PlayerProgression::default();
        let breakdown = player_defense_breakdown(&progression, &PlayerPerks::default(), 0);

        assert!(breakdown.contains("DV 0"));
        assert!(breakdown.contains("MET 0"));
        assert!(breakdown.contains("QUI 0"));
    }

    #[test]
    fn test_enemy_melee_check_breakdown_formats_expected_terms() {
        let progression = PlayerProgression::new_game();
        let breakdown = enemy_melee_check_breakdown(18, &progression, &PlayerPerks::default(), 11);

        assert!(breakdown.contains("ATK 18"));
        assert!(breakdown.contains("DV 11"));
        assert!(breakdown.contains("MET 5"));
        assert!(breakdown.contains("QUI 6"));
    }

    #[test]
    fn test_enemy_melee_check_breakdown_uses_progression_when_component_present() {
        let progression = PlayerProgression::default();
        let breakdown = enemy_melee_check_breakdown(17, &progression, &PlayerPerks::default(), 0);

        assert!(breakdown.contains("ATK 17"));
        assert!(breakdown.contains("DV 0"));
    }

    #[test]
    fn test_player_defense_dv_includes_ghost_step_bonus() {
        let progression = PlayerProgression::new_game();
        let mut perks = PlayerPerks::default();
        perks.unlock(crate::core::perks::PerkId::GhostStep);

        assert_eq!(player_defense_dv(&progression, &perks), 16);
    }

    #[test]
    fn test_enemy_melee_stops_after_fatal_hit() {
        let mut world = World::new();
        world.insert_resource(GameLog::default());
        world.insert_resource(GameTime { turn: 9 });
        world.insert_resource(CombatRng::default());
        world.insert_resource(PendingPerkChoices::default());
        world.insert_resource(NextState::<AppState>::default());

        world.spawn((
            Player,
            Position::new(5, 5),
            CombatStats {
                hp: 5,
                hp_max: 5,
                speed: 1,
                ar: 0,
                md: 0,
                skills: HashMap::new(),
            },
            StatusEffects::default(),
            RaidExposure::default(),
            PlayerPerks::default(),
            PlayerProgression::new_game(),
        ));

        for x in [4, 6] {
            world.spawn((
                Enemy,
                Position::new(x, 5),
                CombatStats {
                    hp: 10,
                    hp_max: 10,
                    speed: 1,
                    ar: 0,
                    md: 0,
                    skills: HashMap::from([(
                        SkillId::Melee,
                        crate::core::stats::SkillState {
                            base: 10_000,
                            xp: 0,
                            level: 0,
                        },
                    )]),
                },
                EntityName {
                    name: format!("Melee {x}"),
                },
                MeleeEnemyAttack {
                    damage: 20,
                    damage_type: DamageType::Slash,
                },
            ));
        }

        let _ = world.run_system_once(resolve_enemy_melee);

        let hit_entries: Vec<_> = world
            .resource::<GameLog>()
            .entries()
            .iter()
            .filter(|entry| entry.text.contains("hits you") || entry.text.contains("strikes you"))
            .collect();
        assert_eq!(
            hit_entries.len(),
            1,
            "melee resolution should stop after the fatal hit"
        );
        assert!(
            hit_entries[0]
                .text
                .contains("[ATK 10000 vs DV 11 = MET 5 + QUI 6]")
        );
        assert!(world.contains_resource::<DeathSummary>());
    }

    #[test]
    fn test_enemy_melee_logs_progression_defense_breakdown() {
        let mut world = World::new();
        world.insert_resource(GameLog::default());
        world.insert_resource(GameTime { turn: 12 });
        world.insert_resource(CombatRng::default());
        world.insert_resource(PendingPerkChoices::default());
        world.insert_resource(NextState::<AppState>::default());

        world.spawn((
            Player,
            Position::new(5, 5),
            CombatStats {
                hp: 25,
                hp_max: 25,
                speed: 1,
                ar: 0,
                md: 0,
                skills: HashMap::new(),
            },
            StatusEffects::default(),
            RaidExposure::default(),
            PlayerPerks::default(),
            PlayerProgression::new_game(),
        ));

        world.spawn((
            Enemy,
            Position::new(4, 5),
            CombatStats {
                hp: 10,
                hp_max: 10,
                speed: 1,
                ar: 0,
                md: 0,
                skills: HashMap::from([(
                    SkillId::Melee,
                    crate::core::stats::SkillState {
                        base: 10_000,
                        xp: 0,
                        level: 0,
                    },
                )]),
            },
            EntityName {
                name: "Melee Test".to_string(),
            },
            MeleeEnemyAttack {
                damage: 1,
                damage_type: DamageType::Slash,
            },
        ));

        let _ = world.run_system_once(resolve_enemy_melee);

        let hit_entry = world
            .resource::<GameLog>()
            .entries()
            .iter()
            .find(|entry| entry.text.contains("hits you") || entry.text.contains("strikes you"))
            .expect("enemy should land a melee attack");
        assert!(
            hit_entry
                .text
                .contains("[ATK 10000 vs DV 11 = MET 5 + QUI 6]")
        );
    }

    #[test]
    fn test_enemy_melee_miss_trains_quiet_movement() {
        let mut world = World::new();
        world.insert_resource(GameLog::default());
        world.insert_resource(GameTime { turn: 14 });
        world.insert_resource(CombatRng::default());
        world.insert_resource(PendingPerkChoices::default());
        world.insert_resource(NextState::<AppState>::default());

        let mut progression = PlayerProgression::new_game();
        progression
            .virtues
            .insert(VirtueId::Metis, crate::core::stats::VirtueState { rank: 4 });
        progression.proficiencies.insert(
            ProficiencyId::QuietMovement,
            crate::core::stats::ProficiencyState {
                rating: 40,
                xp: 49,
                level: 0,
            },
        );

        world.spawn((
            Player,
            Position::new(5, 5),
            CombatStats {
                hp: 25,
                hp_max: 25,
                speed: 1,
                ar: 0,
                md: 0,
                skills: HashMap::new(),
            },
            StatusEffects::default(),
            RaidExposure::default(),
            PlayerPerks::default(),
            progression,
        ));

        world.spawn((
            Enemy,
            Position::new(4, 5),
            CombatStats {
                hp: 10,
                hp_max: 10,
                speed: 1,
                ar: 0,
                md: 0,
                skills: HashMap::from([(
                    SkillId::Melee,
                    crate::core::stats::SkillState {
                        base: 30,
                        xp: 0,
                        level: 0,
                    },
                )]),
            },
            EntityName {
                name: "Training Dummy".to_string(),
            },
            MeleeEnemyAttack {
                damage: 1,
                damage_type: DamageType::Slash,
            },
        ));

        let _ = world.run_system_once(resolve_enemy_melee);

        let progression = world
            .query::<&PlayerProgression>()
            .single(&world)
            .expect("player progression should remain present");
        assert_eq!(
            progression.proficiency_rating(ProficiencyId::QuietMovement),
            42
        );
        assert!(
            world
                .resource::<GameLog>()
                .entries()
                .iter()
                .any(|entry| entry.text.contains("Quiet Movement rises to 42."))
        );
        assert_eq!(
            world.resource::<PendingPerkChoices>().pending,
            vec![
                crate::core::perks::PerkId::GhostStep,
                crate::core::perks::PerkId::FalseTrail,
                crate::core::perks::PerkId::AmbushExtraction,
            ]
        );
    }

    #[test]
    fn test_combat_rng_default_is_seeded_from_zero() {
        // Baseline: confirm current behavior. CombatRng::default() uses seed 0.
        // This test will be updated when WorldSeed-based seeding is implemented.
        let rng = CombatRng::default();
        let mut clone_a = rng.0.clone();
        let mut clone_b = rng.0.clone();
        let a: u32 = clone_a.random_range(1..=100);
        let b: u32 = clone_b.random_range(1..=100);
        assert_eq!(
            a, b,
            "Two CombatRng defaults should produce identical first draws (seed 0)"
        );
    }
}
