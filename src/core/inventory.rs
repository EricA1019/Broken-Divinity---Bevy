use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::core::items::*;

// ── Inventory ────────────────────────────────────────────────

#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct Inventory {
    pub slots: [Option<ItemStack>; 20],
}

impl Default for Inventory {
    fn default() -> Self {
        const NONE: Option<ItemStack> = None;
        Self { slots: [NONE; 20] }
    }
}

impl Inventory {
    /// Add items to inventory. Stacks onto existing matching slots first,
    /// then fills empty slots. Returns Err if no room.
    pub fn try_add(&mut self, item_id: &str, qty: u8) -> Result<(), &'static str> {
        let stack_max = find_item(item_id).map(|d| d.stack_max).unwrap_or(1);
        let mut remaining = qty;

        // First pass: top up existing stacks of the same item
        for slot in self.slots.iter_mut() {
            if remaining == 0 {
                break;
            }
            if let Some(stack) = slot
                && stack.item_id == item_id
                && stack.quantity < stack_max
            {
                let space = stack_max - stack.quantity;
                let add = remaining.min(space);
                stack.quantity += add;
                remaining -= add;
            }
        }

        // Second pass: fill empty slots
        for slot in self.slots.iter_mut() {
            if remaining == 0 {
                break;
            }
            if slot.is_none() {
                let add = remaining.min(stack_max);
                *slot = Some(ItemStack {
                    item_id: item_id.to_string(),
                    quantity: add,
                });
                remaining -= add;
            }
        }

        if remaining > 0 {
            Err("Inventory full")
        } else {
            Ok(())
        }
    }

    /// Remove `qty` items from a slot. Returns the removed stack.
    /// Clears the slot if quantity reaches 0.
    pub fn remove(&mut self, slot_idx: usize, qty: u8) -> Option<ItemStack> {
        let slot = self.slots.get_mut(slot_idx)?;
        let stack = slot.as_mut()?;

        let taken = qty.min(stack.quantity);
        stack.quantity -= taken;

        let removed = ItemStack {
            item_id: stack.item_id.clone(),
            quantity: taken,
        };

        if stack.quantity == 0 {
            *slot = None;
        }

        Some(removed)
    }

    /// True when all 20 slots are occupied.
    pub fn is_full(&self) -> bool {
        self.slots.iter().all(|s| s.is_some())
    }

    /// Total quantity of an item across all slots.
    pub fn count(&self, item_id: &str) -> u8 {
        self.slots
            .iter()
            .filter_map(|s| s.as_ref())
            .filter(|s| s.item_id == item_id)
            .map(|s| s.quantity)
            .fold(0u8, |acc, q| acc.saturating_add(q))
    }

    /// First slot index containing this item.
    pub fn find_slot(&self, item_id: &str) -> Option<usize> {
        self.slots
            .iter()
            .position(|s| s.as_ref().is_some_and(|st| st.item_id == item_id))
    }
}

// ── Equipment ────────────────────────────────────────────────

#[derive(Component, Debug, Clone, Serialize, Deserialize, Default, Reflect)]
#[reflect(Component)]
pub struct Equipment {
    pub weapon: Option<String>,
    pub armor: Option<String>,
    pub accessory: Option<String>,
}

impl Equipment {
    pub fn equip_weapon(&mut self, item_id: String) -> Option<String> {
        self.weapon.replace(item_id)
    }

    pub fn equip_armor(&mut self, item_id: String) -> Option<String> {
        self.armor.replace(item_id)
    }

    pub fn unequip_weapon(&mut self) -> Option<String> {
        self.weapon.take()
    }

    pub fn unequip_armor(&mut self) -> Option<String> {
        self.armor.take()
    }
}

// ── Armor Durability ─────────────────────────────────────────

#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct ArmorDurability {
    pub current: i32,
    pub max: i32,
    pub broken: bool,
}

impl ArmorDurability {
    pub fn take_damage(&mut self, amount: i32) {
        self.current -= amount;
        if self.current <= 0 {
            self.current = 0;
            self.broken = true;
        }
    }

    pub fn repair(&mut self) {
        self.current = self.max;
        self.broken = false;
    }
}

// ── Ranged Weapon State ──────────────────────────────────────

#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct RangedWeaponState {
    pub clip_current: u8,
    pub clip_size: u8,
}

impl RangedWeaponState {
    pub fn can_fire(&self) -> bool {
        self.clip_current > 0
    }

    pub fn fire(&mut self) {
        if self.clip_current > 0 {
            self.clip_current -= 1;
        }
    }

    /// Load up to clip_size from available ammo. Returns ammo consumed.
    pub fn reload(&mut self, available_ammo: u8) -> u8 {
        let need = self.clip_size - self.clip_current;
        let consumed = need.min(available_ammo);
        self.clip_current += consumed;
        consumed
    }
}

// ── Tests ────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inventory_add_and_count() {
        let mut inv = Inventory::default();
        inv.try_add("scrap", 10).unwrap();
        assert_eq!(inv.count("scrap"), 10);
        inv.try_add("scrap", 5).unwrap();
        assert_eq!(inv.count("scrap"), 15);
        inv.try_add("ammo", 3).unwrap();
        assert_eq!(inv.count("ammo"), 3);
        assert_eq!(inv.count("scrap"), 15);
    }

    #[test]
    fn test_inventory_full() {
        let mut inv = Inventory::default();
        // iron_pipe has stack_max=1, so each add fills one slot
        for i in 0..20 {
            inv.try_add("iron_pipe", 1)
                .unwrap_or_else(|_| panic!("should fit slot {i}"));
        }
        assert!(inv.is_full());
        assert!(inv.try_add("iron_pipe", 1).is_err());
    }

    #[test]
    fn test_inventory_remove() {
        let mut inv = Inventory::default();
        inv.try_add("scrap", 10).unwrap();
        let slot = inv.find_slot("scrap").unwrap();
        let removed = inv.remove(slot, 3).unwrap();
        assert_eq!(removed.quantity, 3);
        assert_eq!(inv.count("scrap"), 7);
    }

    #[test]
    fn test_equip_swap() {
        let mut eq = Equipment::default();
        let prev = eq.equip_weapon("iron_pipe".into());
        assert!(prev.is_none());
        let prev = eq.equip_weapon("hunting_knife".into());
        assert_eq!(prev.unwrap(), "iron_pipe");
        assert_eq!(eq.weapon.as_deref(), Some("hunting_knife"));
    }

    #[test]
    fn test_armor_durability_break() {
        let mut dur = ArmorDurability {
            current: 20,
            max: 20,
            broken: false,
        };
        dur.take_damage(15);
        assert!(!dur.broken);
        assert_eq!(dur.current, 5);
        dur.take_damage(10);
        assert!(dur.broken);
        assert_eq!(dur.current, 0);
    }

    #[test]
    fn test_ranged_weapon_reload() {
        let mut rw = RangedWeaponState {
            clip_current: 6,
            clip_size: 6,
        };
        assert!(rw.can_fire());
        rw.fire();
        rw.fire();
        assert_eq!(rw.clip_current, 4);
        let consumed = rw.reload(10);
        assert_eq!(consumed, 2);
        assert_eq!(rw.clip_current, 6);
    }
}
