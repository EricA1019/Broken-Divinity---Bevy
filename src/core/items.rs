use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

use crate::game::combat::DamageType;

// ── Item Kind ────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum ItemKind {
    Weapon,
    Armor,
    Consumable,
    Resource,
}

// ── Weapon ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponProps {
    pub damage: i32,
    pub damage_type: DamageType,
    /// 0 = melee, >0 = ranged tile range
    pub range: u8,
    pub accuracy_mod: i32,
    /// 0 for melee weapons
    pub clip_size: u8,
    /// % chance to inflict Wounded on hit
    pub status_chance: u8,
}

// ── Armor ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArmorProps {
    pub ar: i32,
    pub durability_max: i32,
}

// ── Consumable ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsumableEffect {
    Heal(i32),
    RemoveStatus,
}

// ── Static Definition ────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ItemDef {
    pub id: &'static str,
    pub name: &'static str,
    pub kind: ItemKind,
    pub stack_max: u8,
    pub weapon: Option<WeaponProps>,
    pub armor: Option<ArmorProps>,
    pub consumable: Option<ConsumableEffect>,
}

// ── ECS Components ───────────────────────────────────────────

#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct ItemStack {
    pub item_id: String,
    pub quantity: u8,
}

#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct ItemDrop {
    pub item_id: String,
    pub quantity: u8,
}

// ── RON-loaded Item Catalog ──────────────────────────────────

/// Schema for RON deserialization (id/name as String, not &str).
#[derive(Debug, Clone, Deserialize)]
pub struct RonItemDef {
    pub id: String,
    pub name: String,
    pub kind: ItemKind,
    pub stack_max: u8,
    pub weapon: Option<WeaponProps>,
    pub armor: Option<ArmorProps>,
    pub consumable: Option<ConsumableEffect>,
}

const ITEMS_RON_SOURCE: &str = include_str!("../../native/assets/data/items.ron");

static ITEMS: OnceLock<Vec<ItemDef>> = OnceLock::new();

pub fn all_items() -> &'static [ItemDef] {
    ITEMS.get_or_init(|| {
        let ron_items: Vec<RonItemDef> = ron::from_str(ITEMS_RON_SOURCE)
            .expect("Failed to parse embedded items.ron — check schema compatibility");
        ron_items
            .into_iter()
            .map(|r| ItemDef {
                id: Box::leak(r.id.into_boxed_str()),
                name: Box::leak(r.name.into_boxed_str()),
                kind: r.kind,
                stack_max: r.stack_max,
                weapon: r.weapon,
                armor: r.armor,
                consumable: r.consumable,
            })
            .collect()
    })
}

pub fn find_item(id: &str) -> Option<&'static ItemDef> {
    all_items().iter().find(|def| def.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_items_loads_and_has_expected_count() {
        let items = all_items();
        // 4 weapons + 3 armor + 1 consumable + 4 resources = 12
        assert!(!items.is_empty(), "Item catalog should not be empty");
        assert!(
            items.len() >= 12,
            "Expected at least 12 items, got {}",
            items.len()
        );
    }

    #[test]
    fn test_find_item_returns_known_id() {
        let item = find_item("iron_pipe").expect("iron_pipe should exist");
        assert_eq!(item.name, "Iron Pipe");
        assert_eq!(item.kind, ItemKind::Weapon);
    }

    #[test]
    fn test_find_item_returns_none_for_unknown() {
        assert!(find_item("nonexistent_item").is_none());
    }

    #[test]
    fn test_all_item_ids_are_unique() {
        let items = all_items();
        let mut seen = std::collections::HashSet::new();
        for item in items {
            assert!(seen.insert(item.id), "Duplicate item ID: {}", item.id);
        }
    }

    #[test]
    fn test_weapon_items_have_weapon_props() {
        let weapons: Vec<&ItemDef> = all_items()
            .iter()
            .filter(|i| i.kind == ItemKind::Weapon)
            .collect();
        assert!(!weapons.is_empty(), "Should have weapons");
        for w in &weapons {
            assert!(
                w.weapon.is_some(),
                "Weapon '{}' should have weapon props",
                w.id
            );
        }
    }

    #[test]
    fn test_armor_items_have_armor_props() {
        let armors: Vec<&ItemDef> = all_items()
            .iter()
            .filter(|i| i.kind == ItemKind::Armor)
            .collect();
        assert!(!armors.is_empty(), "Should have armor items");
        for a in &armors {
            assert!(
                a.armor.is_some(),
                "Armor '{}' should have armor props",
                a.id
            );
        }
    }
}
