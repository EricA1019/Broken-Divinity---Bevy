#![allow(clippy::too_many_arguments, clippy::type_complexity)]

//! Ranged attack, reload, and cover systems for dungeon combat.

use bevy::prelude::*;

use crate::core::abilities::{calc_cover, calc_range_penalty};
use crate::core::components::Enemy;
use crate::core::components::{Player, Position, Viewshed};
use crate::core::gamelog::{GameLog, LogColor};
use crate::core::inventory::{Equipment, Inventory, RangedWeaponState};
use crate::core::items::{ItemKind, find_item};
use crate::core::movement::MapTiles;
use crate::core::perks::{PendingPerkChoices, PerkId, PlayerPerks, queue_progression_lane_perks};
use crate::core::sanity::{Hallucination, RaidExposure, SanityEvent, apply_player_sanity_event};
use crate::core::stats::{
    CombatStats, EntityName, PlayerProgression, ProficiencyId, SkillId, VirtueId, calc_xp_award,
};
use crate::core::turn::{ActionBudget, TurnPhase};
use crate::game::combat::{DamageType, calc_damage, roll_check};
use crate::game::dungeon::gabriel::Gabriel;
use crate::game::dungeon::gabriel::GabrielDialogueState;
use crate::game::dungeon::melee::CombatRng;

use crate::core::turn::GameTime;

// ── Resources ────────────────────────────────────────────────

/// Set by UI or keybind; consumed by the resolve system.
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct ShootTarget(pub Option<Entity>);

fn player_ranged_action_rating(progression: &PlayerProgression) -> u32 {
    progression
        .action_rating(VirtueId::Prudence, ProficiencyId::RangedTraining, 0, 0)
        .max(0) as u32
}

fn player_ranged_action_breakdown(
    progression: &PlayerProgression,
    action_rating: u32,
    total_mod: i32,
) -> String {
    let virtue_bonus = i32::from(progression.virtue_rank(VirtueId::Prudence)) * 5;
    let training = progression.proficiency_rating(ProficiencyId::RangedTraining) as i32;
    format!(
        " [AR {} = PRU {} + TRN {} {:+}]",
        action_rating, virtue_bonus, training, total_mod
    )
}

// ── Systems ──────────────────────────────────────────────────

/// System: listen for `F` key to acquire a ranged target.
///
/// Runs during `TurnPhase::AwaitingInput`.
pub fn handle_shoot_input(
    keys: Res<ButtonInput<KeyCode>>,
    pending_perks: Option<Res<PendingPerkChoices>>,
    dialogue_state: Option<Res<GabrielDialogueState>>,
    player_q: Query<(&Position, &Viewshed, &Equipment, &RangedWeaponState), With<Player>>,
    enemy_q: Query<(Entity, &Position), (With<Enemy>, Without<Gabriel>)>,
    mut shoot_target: ResMut<ShootTarget>,
    mut log: ResMut<GameLog>,
    game_time: Res<GameTime>,
    mut next_state: ResMut<NextState<TurnPhase>>,
) {
    if !keys.just_pressed(KeyCode::KeyF) {
        return;
    }

    if pending_perks.is_some_and(|pending| pending.has_pending()) {
        return;
    }

    if dialogue_state.is_some_and(|dialogue| dialogue.is_active()) {
        return;
    }

    let Ok((player_pos, viewshed, equipment, ranged_state)) = player_q.single() else {
        return;
    };

    // Check for equipped ranged weapon
    let Some(weapon_id) = equipment.weapon.as_ref() else {
        log.push("No weapon equipped.", LogColor::System, game_time.turn);
        return;
    };

    let Some(item_def) = find_item(weapon_id) else {
        log.push("Equipped item not found.", LogColor::System, game_time.turn);
        return;
    };

    if item_def.kind != ItemKind::Weapon {
        log.push(
            "Equipped item is not a weapon.",
            LogColor::System,
            game_time.turn,
        );
        return;
    }

    let Some(ref weapon) = item_def.weapon else {
        return;
    };

    if weapon.range == 0 {
        log.push(
            "Equipped weapon is melee-only.",
            LogColor::System,
            game_time.turn,
        );
        return;
    }

    if !ranged_state.can_fire() {
        log.push(
            "Clip empty — press R to reload.",
            LogColor::System,
            game_time.turn,
        );
        return;
    }

    // Find nearest visible enemy
    let mut best: Option<(Entity, i32)> = None;
    for (entity, enemy_pos) in enemy_q.iter() {
        let target_position = Position::new(enemy_pos.x, enemy_pos.y);
        if !viewshed.contains(&target_position) {
            continue;
        }
        let dist = (player_pos.x - enemy_pos.x).abs() + (player_pos.y - enemy_pos.y).abs();
        if best.is_none_or(|(_, best_dist)| dist < best_dist) {
            best = Some((entity, dist));
        }
    }

    if let Some((entity, _)) = best {
        shoot_target.0 = Some(entity);
        // Advance AwaitingInput → PlayerTurn so resolve_ranged_attack fires.
        next_state.set(TurnPhase::PlayerTurn);
    } else {
        log.push("No visible targets.", LogColor::System, game_time.turn);
    }
}

/// System: resolve the queued ranged attack.
///
/// Runs during `TurnPhase::PlayerTurn`.
pub fn resolve_ranged_attack(
    mut commands: Commands,
    mut shoot_target: ResMut<ShootTarget>,
    mut player_q: Query<
        (
            &Position,
            &Equipment,
            &mut RangedWeaponState,
            &mut ActionBudget,
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
            Option<&Hallucination>,
        ),
        (With<Enemy>, Without<Player>),
    >,
    map_tiles: Res<MapTiles>,
    mut log: ResMut<GameLog>,
    mut rng: ResMut<CombatRng>,
    game_time: Res<GameTime>,
    mut next_state: ResMut<NextState<TurnPhase>>,
    mut pending_perks: ResMut<PendingPerkChoices>,
) {
    let Some(target_entity) = shoot_target.0.take() else {
        return;
    };

    let Ok((
        player_pos,
        equipment,
        mut ranged_state,
        mut action_budget,
        mut raid_exposure,
        player_perks,
        mut progression,
    )) = player_q.single_mut()
    else {
        return;
    };

    let Ok((target_entity, target_pos, mut target_stats, target_name, hallucination)) =
        enemy_q.get_mut(target_entity)
    else {
        log.push("Target no longer exists.", LogColor::System, game_time.turn);
        return;
    };

    // Get weapon props
    let weapon_id = match equipment.weapon.as_ref() {
        Some(id) => id,
        None => return,
    };
    let item_def = match find_item(weapon_id) {
        Some(def) => def,
        None => return,
    };
    let weapon = match item_def.weapon.as_ref() {
        Some(w) => w,
        None => return,
    };

    // Distance & penalties
    let distance = (player_pos.x - target_pos.x).abs() + (player_pos.y - target_pos.y).abs();
    let range_penalty = calc_range_penalty(distance, weapon.range);
    let cover = calc_cover(*player_pos, *target_pos, &map_tiles);
    let total_mod = range_penalty
        + cover.modifier()
        + weapon.accuracy_mod
        + player_perks.ranged_accuracy_bonus()
        + raid_exposure.threshold().penalty();

    let skill_level = player_ranged_action_rating(&progression);
    let action_breakdown = player_ranged_action_breakdown(&progression, skill_level, total_mod);
    let enemy_skill = target_stats
        .skill_level(SkillId::Melee)
        .max(target_stats.skill_level(SkillId::Ranged));
    let result = roll_check(skill_level, total_mod, 30, &mut rng.0);

    let xp = calc_xp_award(result.success, result.critical, enemy_skill, skill_level);
    if hallucination.is_none() && xp > 0 {
        if progression
            .grant_proficiency_xp(ProficiencyId::RangedTraining, xp)
            .is_some()
        {
            log.push(
                format!(
                    "{} rises to {}.",
                    ProficiencyId::RangedTraining.name(),
                    progression.proficiency_rating(ProficiencyId::RangedTraining)
                ),
                LogColor::Status,
                game_time.turn,
            );
        }

        for perk in queue_progression_lane_perks(
            &mut pending_perks,
            player_perks,
            VirtueId::Prudence,
            &progression,
            ProficiencyId::RangedTraining,
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

    if result.success {
        if hallucination.is_some() {
            log.push(
                "The shot tears through the apparition and it dissolves.",
                LogColor::Status,
                game_time.turn,
            );
            commands.entity(target_entity).despawn();
        } else {
            let mut dmg = calc_damage(
                weapon.damage,
                skill_level,
                target_stats.ar,
                target_stats.md,
                DamageType::Ballistic,
                result.critical,
                &mut rng.0,
            );
            if result.critical && player_perks.has(PerkId::Deadeye) {
                dmg = ((dmg as f32) * 1.5).round() as i32;
            }
            target_stats.hp -= dmg;

            if result.critical {
                log.push(
                    format!(
                        "CRITICAL! You shoot {} for {} damage! (range {}, {} cover){}",
                        target_name.name,
                        dmg,
                        distance,
                        cover.name(),
                        action_breakdown,
                    ),
                    LogColor::Critical,
                    game_time.turn,
                );
            } else {
                log.push(
                    format!(
                        "You shoot {} for {} damage. (range {}, {} cover){}",
                        target_name.name,
                        dmg,
                        distance,
                        cover.name(),
                        action_breakdown,
                    ),
                    LogColor::PlayerHit,
                    game_time.turn,
                );
            }

            if target_stats.is_dead() {
                let crossed_threshold =
                    apply_player_sanity_event(&mut raid_exposure, player_perks, SanityEvent::Kill);
                log.push(
                    format!("{} is killed!", target_name.name),
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
                commands.entity(target_entity).despawn();
            }
        }
    } else if result.fumble {
        log.push(
            format!(
                "You fumble the shot at {}!{}",
                target_name.name, action_breakdown
            ),
            LogColor::Miss,
            game_time.turn,
        );
    } else {
        log.push(
            format!(
                "You miss {}. (range {}, {} cover){}",
                target_name.name,
                distance,
                cover.name(),
                action_breakdown,
            ),
            LogColor::Miss,
            game_time.turn,
        );
    }

    ranged_state.fire();
    action_budget.remaining = action_budget.remaining.saturating_sub(1);
    next_state.set(TurnPhase::EnemyTurn);
}

/// System: listen for `R` key to reload the equipped ranged weapon.
///
/// Runs during `TurnPhase::AwaitingInput`.
pub fn handle_reload_input(
    keys: Res<ButtonInput<KeyCode>>,
    pending_perks: Option<Res<PendingPerkChoices>>,
    dialogue_state: Option<Res<GabrielDialogueState>>,
    mut player_q: Query<
        (
            &Equipment,
            &mut RangedWeaponState,
            &mut Inventory,
            &mut ActionBudget,
            &PlayerPerks,
        ),
        With<Player>,
    >,
    mut log: ResMut<GameLog>,
    game_time: Res<GameTime>,
    mut next_state: ResMut<NextState<TurnPhase>>,
) {
    if !keys.just_pressed(KeyCode::KeyR) {
        return;
    }

    if pending_perks.is_some_and(|pending| pending.has_pending()) {
        return;
    }

    if dialogue_state.is_some_and(|dialogue| dialogue.is_active()) {
        return;
    }

    let Ok((equipment, mut ranged_state, mut inventory, mut action_budget, perks)) =
        player_q.single_mut()
    else {
        return;
    };

    // Check for equipped ranged weapon
    let Some(weapon_id) = equipment.weapon.as_ref() else {
        log.push("No weapon equipped.", LogColor::System, game_time.turn);
        return;
    };

    let Some(item_def) = find_item(weapon_id) else {
        return;
    };

    let Some(ref weapon) = item_def.weapon else {
        return;
    };

    if weapon.range == 0 {
        log.push(
            "Equipped weapon is melee-only.",
            LogColor::System,
            game_time.turn,
        );
        return;
    }

    if ranged_state.clip_current >= ranged_state.clip_size {
        log.push("Clip already full.", LogColor::System, game_time.turn);
        return;
    }

    let Some(ammo_slot) = inventory.find_slot("ammo") else {
        log.push("No ammo in inventory.", LogColor::System, game_time.turn);
        return;
    };

    // Determine how much ammo to consume
    let available = inventory.slots[ammo_slot]
        .as_ref()
        .map(|s| s.quantity)
        .unwrap_or(0);
    let consumed = ranged_state.reload(available);

    if consumed > 0 {
        inventory.remove(ammo_slot, consumed);
    }

    if perks.has(PerkId::QuickReload) {
        log.push(
            format!(
                "Quick Reload snaps {} back into action (+{} rounds).",
                item_def.name, consumed
            ),
            LogColor::Status,
            game_time.turn,
        );
    } else {
        action_budget.remaining = action_budget.remaining.saturating_sub(1);
        log.push(
            format!("Reloaded {} (+{} rounds).", item_def.name, consumed),
            LogColor::System,
            game_time.turn,
        );
        next_state.set(TurnPhase::EnemyTurn);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_ranged_action_rating_prefers_progression() {
        let progression = PlayerProgression::new_game();

        assert_eq!(player_ranged_action_rating(&progression), 22);
    }

    #[test]
    fn test_player_ranged_action_rating_does_not_use_legacy_when_progression_present() {
        let progression = PlayerProgression::default();

        assert_eq!(player_ranged_action_rating(&progression), 0);
    }

    #[test]
    fn test_player_ranged_action_breakdown_formats_expected_terms() {
        let progression = PlayerProgression::new_game();
        let breakdown = player_ranged_action_breakdown(&progression, 22, 4);

        assert!(breakdown.contains("AR 22"));
        assert!(breakdown.contains("PRU 10"));
        assert!(breakdown.contains("TRN 12"));
        assert!(breakdown.contains("+4"));
    }
}
