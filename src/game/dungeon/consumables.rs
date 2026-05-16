//! Consumable item resolution — heals, status removal, etc.
//!
//! Runs during `PlayerTurn` in `AppState::Dungeon`.

use bevy::prelude::*;

use crate::core::components::Player;
use crate::core::gamelog::{GameLog, LogColor};
use crate::core::inventory::Inventory;
use crate::core::items::{ConsumableEffect, find_item};
use crate::core::stats::CombatStats;
use crate::core::turn::{ActionBudget, GameTime, PendingAction, PlayerAction};

/// Resolves `PendingAction::UseItem` — applies consumable effects, removes
/// the item from inventory, and deducts one action point.
pub fn resolve_consumable_use(
    mut player_action: ResMut<PlayerAction>,
    mut query: Query<(&mut Inventory, &mut CombatStats, &mut ActionBudget), With<Player>>,
    mut game_log: ResMut<GameLog>,
    game_time: Res<GameTime>,
) {
    let Some(PendingAction::UseItem(slot_idx)) = player_action.0.as_ref() else {
        return;
    };
    let slot_idx = *slot_idx;

    // Consume the action so it doesn't fire again next frame.
    player_action.0 = None;

    let Ok((mut inventory, mut stats, mut budget)) = query.single_mut() else {
        return;
    };

    // Bounds check
    if slot_idx >= inventory.slots.len() {
        game_log.push("Invalid slot", LogColor::System, game_time.turn);
        return;
    }

    let Some(stack) = &inventory.slots[slot_idx] else {
        game_log.push("No item in that slot", LogColor::System, game_time.turn);
        return;
    };

    let item_id = stack.item_id.clone();
    let Some(def) = find_item(&item_id) else {
        game_log.push("Unknown item", LogColor::System, game_time.turn);
        return;
    };

    let Some(effect) = &def.consumable else {
        game_log.push(
            format!("Can't use {}", def.name),
            LogColor::System,
            game_time.turn,
        );
        return; // Don't consume AP for non-consumable items
    };

    match effect {
        ConsumableEffect::Heal(amount) => {
            let amount = *amount;
            let old_hp = stats.hp;
            stats.hp = (stats.hp + amount).min(stats.hp_max);
            let healed = stats.hp - old_hp;
            game_log.push(
                format!("Used {}: restored {} HP", def.name, healed),
                LogColor::PlayerHit,
                game_time.turn,
            );
        }
        ConsumableEffect::RemoveStatus => {
            game_log.push(
                format!("Used {}: status cleared", def.name),
                LogColor::Status,
                game_time.turn,
            );
        }
    }

    // Remove 1 from the inventory slot
    inventory.remove(slot_idx, 1);

    // Consume 1 action point
    if budget.remaining > 0 {
        budget.remaining -= 1;
    }
}

// ── Tests ────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::inventory::Inventory;
    use crate::core::items::ItemStack;
    use crate::core::stats::{SkillId, SkillState};
    use bevy::ecs::system::RunSystemOnce;
    use std::collections::HashMap;

    fn default_combat_stats(hp: i32, hp_max: i32) -> CombatStats {
        let mut skills = HashMap::new();
        for &skill in SkillId::pilot_proxies() {
            skills.insert(
                skill,
                SkillState {
                    base: 30,
                    xp: 0,
                    level: 0,
                },
            );
        }
        CombatStats {
            hp,
            hp_max,
            speed: 1,
            ar: 0,
            md: 0,
            skills,
        }
    }

    #[test]
    fn default_combat_stats_uses_only_canonical_skill_proxies() {
        let stats = default_combat_stats(10, 20);
        let keys: std::collections::HashSet<_> = stats.skills.keys().copied().collect();
        let expected: std::collections::HashSet<_> = SkillId::pilot_proxies().iter().copied().collect();
        assert_eq!(keys, expected);
    }

    #[test]
    fn heal_consumable_increases_hp_capped_at_max() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<GameLog>();
        app.init_resource::<GameTime>();
        app.insert_resource(PlayerAction(Some(PendingAction::UseItem(0))));

        let mut inventory = Inventory::default();
        inventory.try_add("medicine", 2).unwrap();

        app.world_mut().spawn((
            Player,
            inventory,
            default_combat_stats(20, 50),
            ActionBudget::new(1),
        ));

        app.world_mut().run_system_once(resolve_consumable_use);

        let (inv, stats, budget) = app
            .world_mut()
            .query_filtered::<(&Inventory, &CombatStats, &ActionBudget), With<Player>>()
            .single(app.world())
            .unwrap();

        // Medicine heals 15 → 20 + 15 = 35 (under max 50)
        assert_eq!(stats.hp, 35);
        assert_eq!(inv.count("medicine"), 1);
        assert_eq!(budget.remaining, 0);
    }

    #[test]
    fn heal_consumable_caps_at_max_hp() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<GameLog>();
        app.init_resource::<GameTime>();
        app.insert_resource(PlayerAction(Some(PendingAction::UseItem(0))));

        let mut inventory = Inventory::default();
        inventory.try_add("medicine", 1).unwrap();

        // HP 45/50 — medicine heals 15 but should cap at 50
        app.world_mut().spawn((
            Player,
            inventory,
            default_combat_stats(45, 50),
            ActionBudget::new(1),
        ));

        app.world_mut().run_system_once(resolve_consumable_use);

        let stats = app
            .world_mut()
            .query_filtered::<&CombatStats, With<Player>>()
            .single(app.world())
            .unwrap();

        assert_eq!(stats.hp, 50);
    }

    #[test]
    fn consumable_removed_from_inventory() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<GameLog>();
        app.init_resource::<GameTime>();
        app.insert_resource(PlayerAction(Some(PendingAction::UseItem(0))));

        let mut inventory = Inventory::default();
        inventory.try_add("medicine", 1).unwrap();

        app.world_mut().spawn((
            Player,
            inventory,
            default_combat_stats(20, 50),
            ActionBudget::new(1),
        ));

        app.world_mut().run_system_once(resolve_consumable_use);

        let inv = app
            .world_mut()
            .query_filtered::<&Inventory, With<Player>>()
            .single(app.world())
            .unwrap();

        assert_eq!(inv.count("medicine"), 0);
        assert!(inv.slots[0].is_none());
    }

    #[test]
    fn non_consumable_does_not_consume_ap() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<GameLog>();
        app.init_resource::<GameTime>();
        // Try to "use" an iron_pipe (weapon, no consumable effect)
        app.insert_resource(PlayerAction(Some(PendingAction::UseItem(0))));

        let mut inventory = Inventory::default();
        inventory.try_add("iron_pipe", 1).unwrap();

        app.world_mut().spawn((
            Player,
            inventory,
            default_combat_stats(30, 50),
            ActionBudget::new(1),
        ));

        app.world_mut().run_system_once(resolve_consumable_use);

        let (inv, stats, budget) = app
            .world_mut()
            .query_filtered::<(&Inventory, &CombatStats, &ActionBudget), With<Player>>()
            .single(app.world())
            .unwrap();

        // AP should NOT be consumed
        assert_eq!(budget.remaining, 1);
        // Item should still be in inventory
        assert_eq!(inv.count("iron_pipe"), 1);
        // HP unchanged
        assert_eq!(stats.hp, 30);
    }
}
