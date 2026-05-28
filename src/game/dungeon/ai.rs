#![allow(clippy::too_many_arguments, clippy::type_complexity)]

//! Basic enemy AI — pathfind toward player, attack if adjacent. Ranged enemies
//! shoot when the player is in LOS and within range.

use bevy::prelude::*;
use pathfinding::prelude::astar;

use crate::core::components::{Enemy, Player, Position};
use crate::core::escape;
use crate::core::gamelog::{GameLog, LogColor};
use crate::core::movement::MapTiles;
use crate::core::perks::{
    PendingPerkChoices, PlayerPerks, apply_second_wind, queue_progression_lane_perks,
};
use crate::core::sanity::{Hallucination, RaidExposure, SanityEvent, apply_player_sanity_event};
use crate::core::state::AppState;
use crate::core::stats::{CombatStats, EntityName, PlayerProgression, ProficiencyId, VirtueId};
use crate::core::turn::{ActionBudget, GameTime};
use crate::game::combat::{calc_damage, roll_check};
use crate::game::dungeon::enemies::RangedEnemy;

/// Bresenham line-of-sight check. Returns `true` if every tile between `from`
/// and `to` (exclusive of endpoints) is walkable.
fn has_los(from: (i32, i32), to: (i32, i32), map: &MapTiles) -> bool {
    let (mut x0, mut y0) = from;
    let (x1, y1) = to;

    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        // Reached destination — LOS clear
        if x0 == x1 && y0 == y1 {
            return true;
        }

        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }

        // Don't test the destination tile itself
        if x0 == x1 && y0 == y1 {
            return true;
        }

        if !map.is_walkable(x0, y0) {
            return false;
        }
    }
}

fn player_enemy_attack_dv(progression: &PlayerProgression, player_perks: &PlayerPerks) -> i32 {
    progression.enemy_attack_dv() + player_perks.enemy_attack_dv_bonus(true)
}

fn enemy_ranged_check_breakdown(
    skill_level: u32,
    progression: &PlayerProgression,
    player_perks: &PlayerPerks,
    defense_dv: i32,
) -> String {
    let virtue_bonus = i32::from(progression.virtue_rank(VirtueId::Metis)) * 5;
    let quiet_movement = progression.proficiency_rating(ProficiencyId::QuietMovement) as i32;
    let perk_bonus = player_perks.enemy_attack_dv_bonus(true);
    if perk_bonus > 0 {
        format!(
            " [ATK {} vs DV {} = MET {} + QUI {} + PRK {}]",
            skill_level, defense_dv, virtue_bonus, quiet_movement, perk_bonus
        )
    } else {
        format!(
            " [ATK {} vs DV {} = MET {} + QUI {}]",
            skill_level, defense_dv, virtue_bonus, quiet_movement
        )
    }
}

/// System: each enemy with remaining actions pathfinds toward the player and
/// either attacks (if adjacent) or moves one step closer.
///
/// Runs during `TurnPhase::EnemyTurn` in `AppState::Dungeon`.
pub fn enemy_ai_turn(
    mut commands: Commands,
    mut enemies: Query<
        (
            Entity,
            &mut Position,
            &mut ActionBudget,
            &CombatStats,
            &EntityName,
            Option<&RangedEnemy>,
        ),
        (With<Enemy>, Without<Player>, Without<Hallucination>),
    >,
    mut player_query: Query<
        (
            Entity,
            &Position,
            &mut CombatStats,
            &mut RaidExposure,
            &PlayerPerks,
            &mut PlayerProgression,
        ),
        With<Player>,
    >,
    map: Option<Res<MapTiles>>,
    mut log: ResMut<GameLog>,
    time: Res<GameTime>,
    mut pending_perks: ResMut<PendingPerkChoices>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    let Ok((
        _player_entity,
        player_pos,
        mut player_stats,
        mut raid_exposure,
        player_perks,
        mut progression,
    )) = player_query.single_mut()
    else {
        return;
    };
    if player_stats.is_dead() {
        return;
    }
    let Some(map) = map else { return };
    let player_xy = (player_pos.x, player_pos.y);

    // Collect enemy positions to avoid same-tile collisions (immutable read before mutable loop)
    let occupied: Vec<(i32, i32)> = enemies
        .iter()
        .map(|(_, p, _, _, _, _)| (p.x, p.y))
        .collect();

    for (_entity, mut pos, mut budget, _stats, name, ranged) in enemies.iter_mut() {
        if player_stats.is_dead() {
            return;
        }
        if budget.remaining == 0 {
            continue;
        }

        let dx = (player_pos.x - pos.x).abs();
        let dy = (player_pos.y - pos.y).abs();
        let manhattan = dx + dy;

        // ── Ranged attack ────────────────────────────────────
        if let Some(r) = ranged
            && manhattan <= r.range
            && manhattan > 1
            && has_los((pos.x, pos.y), player_xy, &map)
        {
            let mut rng = rand::rng();
            let defense_dv = player_enemy_attack_dv(&progression, player_perks);
            let check_breakdown =
                enemy_ranged_check_breakdown(r.skill, &progression, player_perks, defense_dv);
            let check = roll_check(r.skill, 0, defense_dv, &mut rng);
            if check.success {
                let previous_hp = player_stats.hp;
                let dmg = calc_damage(
                    r.damage,
                    r.skill,
                    player_stats.ar + player_perks.ar_bonus(),
                    player_stats.md,
                    r.damage_type,
                    check.critical,
                    &mut rng,
                );
                player_stats.hp -= dmg;
                let crossed_threshold = apply_player_sanity_event(
                    &mut raid_exposure,
                    player_perks,
                    SanityEvent::CombatHit,
                );
                let color = if check.critical {
                    LogColor::Critical
                } else {
                    LogColor::EnemyHit
                };
                log.push(
                    format!(
                        "{} shoots you for {} {} damage!{}",
                        name.name,
                        dmg,
                        r.damage_type.name(),
                        check_breakdown
                    ),
                    color,
                    time.turn,
                );
                if crossed_threshold {
                    log.push(
                        format!("Sanity frays: {}.", raid_exposure.threshold().name()),
                        LogColor::Status,
                        time.turn,
                    );
                }
                if apply_second_wind(&mut player_stats, player_perks, previous_hp) {
                    log.push(
                        "Second Wind keeps you standing.",
                        LogColor::Status,
                        time.turn,
                    );
                }
                if player_stats.is_dead() {
                    escape::queue_game_over(
                        &mut commands,
                        next_app_state.as_mut(),
                        &mut log,
                        time.turn,
                    );
                    return;
                }
            } else {
                log.push(
                    format!("{} fires and misses!{}", name.name, check_breakdown),
                    LogColor::Miss,
                    time.turn,
                );
            }
            let defense_xp = progression.enemy_attack_xp_award(!check.success, r.skill)
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
                    time.turn,
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
                        time.turn,
                    );
                }
            }
            budget.remaining = budget.remaining.saturating_sub(1);
            continue;
        }

        // ── Adjacent (including diagonals) → melee ───────────
        if dx <= 1 && dy <= 1 && (dx + dy) > 0 {
            budget.remaining = budget.remaining.saturating_sub(1);
            // Combat resolution will be handled by the resolve_melee system.
            continue;
        }

        // ── Pathfind toward player ───────────────────────────
        let start = (pos.x, pos.y);
        let goal = (player_pos.x, player_pos.y);

        let result = astar(
            &start,
            |&(x, y)| {
                let mut neighbors = Vec::new();
                for (ndx, ndy) in &[(0, 1), (0, -1), (1, 0), (-1, 0)] {
                    let nx = x + ndx;
                    let ny = y + ndy;
                    if map.is_walkable(nx, ny) {
                        // Don't walk into other enemies (except the player position)
                        let blocked = occupied
                            .iter()
                            .any(|&(ox, oy)| ox == nx && oy == ny && (nx, ny) != goal);
                        if !blocked {
                            neighbors.push(((nx, ny), 1));
                        }
                    }
                }
                neighbors
            },
            |&(x, y)| ((x - goal.0).abs() + (y - goal.1).abs()) as u32,
            |&p| p == goal,
        );

        if let Some((path, _cost)) = result
            && path.len() > 1
        {
            let next = path[1];
            pos.x = next.0;
            pos.y = next.1;
        }

        budget.remaining = budget.remaining.saturating_sub(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::gamelog::GameLog;
    use crate::core::movement::MapTiles;
    use crate::core::perks::{PendingPerkChoices, PerkId, PlayerPerks};
    use crate::core::sanity::RaidExposure;
    use crate::core::stats::{
        CombatStats, PlayerProgression, ProficiencyId, ProficiencyState, SkillId, SkillState,
        VirtueId, VirtueState,
    };
    use crate::ui::gameover::DeathSummary;
    use bevy::ecs::system::RunSystemOnce;
    use std::collections::HashMap;

    fn skill_map(skill: SkillId, base: u32) -> HashMap<SkillId, SkillState> {
        HashMap::from([(
            skill,
            SkillState {
                base,
                xp: 0,
                level: 0,
            },
        )])
    }

    #[test]
    fn test_enemy_ranged_check_breakdown_formats_expected_terms() {
        let progression = PlayerProgression::new_game();
        let breakdown = enemy_ranged_check_breakdown(18, &progression, &PlayerPerks::default(), 11);

        assert!(breakdown.contains("ATK 18"));
        assert!(breakdown.contains("DV 11"));
        assert!(breakdown.contains("MET 5"));
        assert!(breakdown.contains("QUI 6"));
    }

    #[test]
    fn test_enemy_ranged_check_breakdown_uses_progression_when_component_present() {
        let progression = PlayerProgression::default();
        let breakdown = enemy_ranged_check_breakdown(17, &progression, &PlayerPerks::default(), 0);

        assert!(breakdown.contains("ATK 17"));
        assert!(breakdown.contains("DV 0"));
    }

    #[test]
    fn test_player_enemy_attack_dv_includes_ranged_lane_perk_bonuses() {
        let progression = PlayerProgression::new_game();
        let mut perks = PlayerPerks::default();
        perks.unlock(PerkId::GhostStep);
        perks.unlock(PerkId::FalseTrail);

        assert_eq!(player_enemy_attack_dv(&progression, &perks), 26);
    }

    #[test]
    fn test_enemy_ai_stops_after_fatal_ranged_hit() {
        let mut world = World::new();
        world.insert_resource(MapTiles::new(
            vec![vec![crate::core::components::TileKind::Floor; 8]; 8],
        ));
        world.insert_resource(GameLog::default());
        world.insert_resource(GameTime { turn: 12 });
        world.insert_resource(PendingPerkChoices::default());
        world.insert_resource(NextState::<AppState>::default());

        world.spawn((
            Player,
            Position::new(4, 4),
            CombatStats {
                hp: 5,
                hp_max: 5,
                speed: 1,
                ar: 0,
                md: 0,
                skills: HashMap::new(),
            },
            RaidExposure::default(),
            PlayerPerks::default(),
            PlayerProgression::new_game(),
        ));

        for x in [1, 2] {
            world.spawn((
                Enemy,
                Position::new(x, 4),
                ActionBudget::new(1),
                CombatStats {
                    hp: 10,
                    hp_max: 10,
                    speed: 1,
                    ar: 0,
                    md: 0,
                    skills: skill_map(SkillId::Ranged, 10_000),
                },
                EntityName {
                    name: format!("Shooter {x}"),
                },
                RangedEnemy {
                    range: 6,
                    skill: 10_000,
                    damage: 20,
                    damage_type: crate::game::combat::DamageType::Ballistic,
                },
            ));
        }

        let _ = world.run_system_once(enemy_ai_turn);

        let shot_logs = world
            .resource::<GameLog>()
            .entries()
            .iter()
            .filter(|entry| entry.text.contains("shoots you"))
            .collect::<Vec<_>>();
        assert_eq!(
            shot_logs.len(),
            1,
            "enemy turn should stop after the fatal ranged hit"
        );
        assert!(
            shot_logs[0]
                .text
                .contains("[ATK 10000 vs DV 11 = MET 5 + QUI 6]")
        );
        assert!(world.contains_resource::<DeathSummary>());
    }

    #[test]
    fn test_enemy_ai_ranged_logs_progression_defense_breakdown() {
        let mut world = World::new();
        world.insert_resource(MapTiles::new(
            vec![vec![crate::core::components::TileKind::Floor; 8]; 8],
        ));
        world.insert_resource(GameLog::default());
        world.insert_resource(GameTime { turn: 18 });
        world.insert_resource(PendingPerkChoices::default());
        world.insert_resource(NextState::<AppState>::default());

        world.spawn((
            Player,
            Position::new(4, 4),
            CombatStats {
                hp: 25,
                hp_max: 25,
                speed: 1,
                ar: 0,
                md: 0,
                skills: HashMap::new(),
            },
            RaidExposure::default(),
            PlayerPerks::default(),
            PlayerProgression::new_game(),
        ));

        world.spawn((
            Enemy,
            Position::new(1, 4),
            ActionBudget::new(1),
            CombatStats {
                hp: 10,
                hp_max: 10,
                speed: 1,
                ar: 0,
                md: 0,
                skills: skill_map(SkillId::Ranged, 10_000),
            },
            EntityName {
                name: "Shooter".to_string(),
            },
            RangedEnemy {
                range: 6,
                skill: 10_000,
                damage: 2,
                damage_type: crate::game::combat::DamageType::Ballistic,
            },
        ));

        let _ = world.run_system_once(enemy_ai_turn);

        let shot_log = world
            .resource::<GameLog>()
            .entries()
            .iter()
            .find(|entry| entry.text.contains("shoots you"))
            .expect("enemy should land a ranged attack");
        assert!(
            shot_log
                .text
                .contains("[ATK 10000 vs DV 11 = MET 5 + QUI 6]")
        );
    }

    #[test]
    fn test_enemy_ai_ranged_miss_trains_quiet_movement() {
        let mut world = World::new();
        world.insert_resource(MapTiles::new(
            vec![vec![crate::core::components::TileKind::Floor; 8]; 8],
        ));
        world.insert_resource(GameLog::default());
        world.insert_resource(GameTime { turn: 21 });
        world.insert_resource(PendingPerkChoices::default());
        world.insert_resource(NextState::<AppState>::default());

        let mut progression = PlayerProgression::new_game();
        progression
            .virtues
            .insert(VirtueId::Metis, VirtueState { rank: 4 });
        progression.proficiencies.insert(
            ProficiencyId::QuietMovement,
            ProficiencyState {
                rating: 40,
                xp: 49,
                level: 0,
            },
        );

        world.spawn((
            Player,
            Position::new(4, 4),
            CombatStats {
                hp: 25,
                hp_max: 25,
                speed: 1,
                ar: 0,
                md: 0,
                skills: HashMap::new(),
            },
            RaidExposure::default(),
            PlayerPerks::default(),
            progression,
        ));

        world.spawn((
            Enemy,
            Position::new(1, 4),
            ActionBudget::new(1),
            CombatStats {
                hp: 10,
                hp_max: 10,
                speed: 1,
                ar: 0,
                md: 0,
                skills: skill_map(SkillId::Ranged, 30),
            },
            EntityName {
                name: "Shooter".to_string(),
            },
            RangedEnemy {
                range: 6,
                skill: 30,
                damage: 2,
                damage_type: crate::game::combat::DamageType::Ballistic,
            },
        ));

        let _ = world.run_system_once(enemy_ai_turn);

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
                PerkId::GhostStep,
                PerkId::FalseTrail,
                PerkId::AmbushExtraction
            ]
        );
    }
}
