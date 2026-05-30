use crate::core::inventory::{Equipment, Inventory};
use crate::core::items::{ItemKind, ItemStack, find_item};

const ACCESSORY_ID_PREFIX: &str = "charm_";
const EQUIP_STACK_QUANTITY: u8 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentSlot {
    Weapon,
    Armor,
    Accessory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipOutcome {
    Equipped,
    Swapped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventoryRuleError {
    InvalidInventorySlot,
    NotEquippable,
    InventoryFull,
    NothingEquipped,
}

pub fn equip_from_inventory_slot(
    inventory: &mut Inventory,
    equipment: &mut Equipment,
    slot_index: usize,
) -> Result<EquipOutcome, InventoryRuleError> {
    let Some(stack) = inventory.slots.get(slot_index).and_then(Option::as_ref) else {
        return Err(InventoryRuleError::InvalidInventorySlot);
    };

    let incoming_item_id = stack.item_id.clone();
    let target_slot = resolve_equipment_slot(&incoming_item_id)?;

    let removed_stack = inventory
        .remove(slot_index, EQUIP_STACK_QUANTITY)
        .ok_or(InventoryRuleError::InvalidInventorySlot)?;

    let replaced_item = replace_equipment_slot(equipment, target_slot, incoming_item_id.clone());

    let Some(replaced_item) = replaced_item else {
        return Ok(EquipOutcome::Equipped);
    };

    if inventory.try_add(&replaced_item, EQUIP_STACK_QUANTITY).is_ok() {
        return Ok(EquipOutcome::Swapped);
    }

    replace_equipment_slot(equipment, target_slot, replaced_item);
    restore_removed_stack(inventory, slot_index, &removed_stack);
    Err(InventoryRuleError::InventoryFull)
}

pub fn unequip_to_inventory_slot(
    inventory: &mut Inventory,
    equipment: &mut Equipment,
    slot: EquipmentSlot,
) -> Result<(), InventoryRuleError> {
    let item_id = take_equipment_slot(equipment, slot).ok_or(InventoryRuleError::NothingEquipped)?;

    if inventory.try_add(&item_id, EQUIP_STACK_QUANTITY).is_ok() {
        return Ok(());
    }

    replace_equipment_slot(equipment, slot, item_id);
    Err(InventoryRuleError::InventoryFull)
}

pub fn resolve_equipment_slot(item_id: &str) -> Result<EquipmentSlot, InventoryRuleError> {
    if let Some(item) = find_item(item_id) {
        return match item.kind {
            ItemKind::Weapon => Ok(EquipmentSlot::Weapon),
            ItemKind::Armor => Ok(EquipmentSlot::Armor),
            ItemKind::Consumable | ItemKind::Resource => {
                if is_accessory_item(item_id) {
                    Ok(EquipmentSlot::Accessory)
                } else {
                    Err(InventoryRuleError::NotEquippable)
                }
            }
        };
    }

    if is_accessory_item(item_id) {
        Ok(EquipmentSlot::Accessory)
    } else {
        Err(InventoryRuleError::NotEquippable)
    }
}

fn is_accessory_item(item_id: &str) -> bool {
    item_id.starts_with(ACCESSORY_ID_PREFIX)
}

fn replace_equipment_slot(
    equipment: &mut Equipment,
    slot: EquipmentSlot,
    item_id: String,
) -> Option<String> {
    match slot {
        EquipmentSlot::Weapon => equipment.weapon.replace(item_id),
        EquipmentSlot::Armor => equipment.armor.replace(item_id),
        EquipmentSlot::Accessory => equipment.accessory.replace(item_id),
    }
}

fn take_equipment_slot(equipment: &mut Equipment, slot: EquipmentSlot) -> Option<String> {
    match slot {
        EquipmentSlot::Weapon => equipment.weapon.take(),
        EquipmentSlot::Armor => equipment.armor.take(),
        EquipmentSlot::Accessory => equipment.accessory.take(),
    }
}

fn restore_removed_stack(inventory: &mut Inventory, slot_index: usize, removed_stack: &ItemStack) {
    let Some(slot) = inventory.slots.get_mut(slot_index) else {
        let _ = inventory.try_add(&removed_stack.item_id, removed_stack.quantity);
        return;
    };

    match slot {
        Some(existing) if existing.item_id == removed_stack.item_id => {
            existing.quantity = existing.quantity.saturating_add(removed_stack.quantity);
        }
        None => {
            *slot = Some(removed_stack.clone());
        }
        Some(_) => {
            let _ = inventory.try_add(&removed_stack.item_id, removed_stack.quantity);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::items::ItemStack;

    const IRON_PIPE_ID: &str = "iron_pipe";
    const HUNTING_KNIFE_ID: &str = "hunting_knife";
    const MEDICINE_ID: &str = "medicine";
    const CHARM_ALPHA_ID: &str = "charm_of_witness";
    const CHARM_BETA_ID: &str = "charm_of_burden";

    #[test]
    fn equip_swaps_occupied_slot() {
        let mut inventory = Inventory::default();
        let mut equipment = Equipment {
            weapon: Some(IRON_PIPE_ID.to_string()),
            ..Equipment::default()
        };
        inventory
            .try_add(HUNTING_KNIFE_ID, 1)
            .expect("expected knife add");

        let outcome = equip_from_inventory_slot(&mut inventory, &mut equipment, 0)
            .expect("expected swap to succeed");

        assert_eq!(outcome, EquipOutcome::Swapped);
        assert_eq!(equipment.weapon.as_deref(), Some(HUNTING_KNIFE_ID));
        assert_eq!(inventory.count(IRON_PIPE_ID), 1);
    }

    #[test]
    fn equip_rejects_non_equippable_item() {
        let mut inventory = Inventory::default();
        let mut equipment = Equipment::default();

        inventory
            .try_add(MEDICINE_ID, 1)
            .expect("expected medicine add");

        let error = equip_from_inventory_slot(&mut inventory, &mut equipment, 0)
            .expect_err("expected non-equippable rejection");

        assert_eq!(error, InventoryRuleError::NotEquippable);
    }

    #[test]
    fn accessory_prefix_routes_to_accessory_slot() {
        let mut inventory = Inventory::default();
        let mut equipment = Equipment::default();

        inventory
            .try_add(CHARM_ALPHA_ID, 1)
            .expect("expected charm add");

        let outcome = equip_from_inventory_slot(&mut inventory, &mut equipment, 0)
            .expect("expected accessory equip");

        assert_eq!(outcome, EquipOutcome::Equipped);
        assert_eq!(equipment.accessory.as_deref(), Some(CHARM_ALPHA_ID));
    }

    #[test]
    fn rollback_preserves_items_when_swap_return_path_is_full() {
        let mut inventory = Inventory::default();
        let mut equipment = Equipment::default();

        inventory.slots[0] = Some(ItemStack {
            item_id: HUNTING_KNIFE_ID.to_string(),
            quantity: 2,
        });
        for slot in 1..inventory.slots.len() {
            inventory.slots[slot] = Some(ItemStack {
                item_id: IRON_PIPE_ID.to_string(),
                quantity: 1,
            });
        }
        equipment.weapon = Some(IRON_PIPE_ID.to_string());

        let error = equip_from_inventory_slot(&mut inventory, &mut equipment, 0)
            .expect_err("expected full-inventory rollback");

        assert_eq!(error, InventoryRuleError::InventoryFull);
        assert_eq!(equipment.weapon.as_deref(), Some(IRON_PIPE_ID));
        assert_eq!(inventory.count(HUNTING_KNIFE_ID), 2);
    }

    #[test]
    fn accessory_swap_restores_previous_accessory_to_inventory() {
        let mut inventory = Inventory::default();
        let mut equipment = Equipment {
            accessory: Some(CHARM_ALPHA_ID.to_string()),
            ..Equipment::default()
        };
        inventory
            .try_add(CHARM_BETA_ID, 1)
            .expect("expected second charm add");

        let outcome = equip_from_inventory_slot(&mut inventory, &mut equipment, 0)
            .expect("expected accessory swap");

        assert_eq!(outcome, EquipOutcome::Swapped);
        assert_eq!(equipment.accessory.as_deref(), Some(CHARM_BETA_ID));
        assert_eq!(inventory.count(CHARM_ALPHA_ID), 1);
    }
}
