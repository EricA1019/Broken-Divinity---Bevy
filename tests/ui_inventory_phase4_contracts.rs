use broken_divinity::core::inventory::{Equipment, Inventory};
use broken_divinity::core::items::ItemStack;
use broken_divinity::core::state::AppState;
use broken_divinity::core::turn::{PendingAction, PlayerAction, TurnPhase};
use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use broken_divinity::core::components::Player;
use broken_divinity::ui::inventory_panel::{
    InventoryOpen, InventoryUiAction, InventoryUiChoice, InventoryUiStatus, process_inventory_action,
};
use broken_divinity::ui::inventory_rules::{
    EquipOutcome, EquipmentSlot, InventoryRuleError, equip_from_inventory_slot,
    unequip_to_inventory_slot,
};

const IRON_PIPE_ID: &str = "iron_pipe";
const HUNTING_KNIFE_ID: &str = "hunting_knife";
const SCRAP_VEST_ID: &str = "scrap_vest";
const MEDICINE_ID: &str = "medicine";
const ACCESSORY_CHARM_ALPHA_ID: &str = "charm_of_witness";
const ACCESSORY_CHARM_BETA_ID: &str = "charm_of_burden";
const INVENTORY_FULL_COUNT: usize = 20;

fn full_inventory_of_pipes() -> Inventory {
    let mut inventory = Inventory::default();
    for _ in 0..INVENTORY_FULL_COUNT {
        inventory
            .try_add(IRON_PIPE_ID, 1)
            .expect("expected fill inventory slot with single-stack weapon");
    }
    inventory
}

#[test]
fn equip_routes_weapon_item_to_weapon_slot() {
    let mut inventory = Inventory::default();
    let mut equipment = Equipment::default();

    inventory
        .try_add(IRON_PIPE_ID, 1)
        .expect("expected weapon add to inventory");

    let outcome = equip_from_inventory_slot(&mut inventory, &mut equipment, 0)
        .expect("expected weapon equip to succeed");

    assert_eq!(outcome, EquipOutcome::Equipped);
    assert_eq!(equipment.weapon.as_deref(), Some(IRON_PIPE_ID));
    assert!(inventory.slots[0].is_none());
}

#[test]
fn equip_swaps_occupied_weapon_slot_and_returns_old_item_to_inventory() {
    let mut inventory = Inventory::default();
    let mut equipment = Equipment {
        weapon: Some(IRON_PIPE_ID.to_string()),
        ..Equipment::default()
    };
    inventory
        .try_add(HUNTING_KNIFE_ID, 1)
        .expect("expected knife add to inventory");

    let outcome = equip_from_inventory_slot(&mut inventory, &mut equipment, 0)
        .expect("expected swap-first policy for occupied weapon slot");

    assert_eq!(outcome, EquipOutcome::Swapped);
    assert_eq!(equipment.weapon.as_deref(), Some(HUNTING_KNIFE_ID));
    assert_eq!(inventory.count(IRON_PIPE_ID), 1);
}

#[test]
fn equip_blocks_non_equippable_items() {
    let mut inventory = Inventory::default();
    let mut equipment = Equipment::default();

    inventory
        .try_add(MEDICINE_ID, 1)
        .expect("expected consumable add to inventory");

    let error = equip_from_inventory_slot(&mut inventory, &mut equipment, 0)
        .expect_err("expected non-equippable consumable to be rejected");

    assert_eq!(error, InventoryRuleError::NotEquippable);
    assert!(equipment.weapon.is_none());
    assert_eq!(inventory.count(MEDICINE_ID), 1);
}

#[test]
fn accessory_ids_route_to_single_accessory_slot() {
    let mut inventory = Inventory::default();
    let mut equipment = Equipment::default();

    inventory
        .try_add(ACCESSORY_CHARM_ALPHA_ID, 1)
        .expect("expected accessory add to inventory");

    let outcome = equip_from_inventory_slot(&mut inventory, &mut equipment, 0)
        .expect("expected accessory routing to accessory slot");

    assert_eq!(outcome, EquipOutcome::Equipped);
    assert_eq!(equipment.accessory.as_deref(), Some(ACCESSORY_CHARM_ALPHA_ID));
}

#[test]
fn accessory_swap_returns_previous_charm_to_inventory() {
    let mut inventory = Inventory::default();
    let mut equipment = Equipment {
        accessory: Some(ACCESSORY_CHARM_ALPHA_ID.to_string()),
        ..Equipment::default()
    };
    inventory
        .try_add(ACCESSORY_CHARM_BETA_ID, 1)
        .expect("expected second charm add to inventory");

    let outcome = equip_from_inventory_slot(&mut inventory, &mut equipment, 0)
        .expect("expected accessory swap behavior");

    assert_eq!(outcome, EquipOutcome::Swapped);
    assert_eq!(equipment.accessory.as_deref(), Some(ACCESSORY_CHARM_BETA_ID));
    assert_eq!(inventory.count(ACCESSORY_CHARM_ALPHA_ID), 1);
}

#[test]
fn unequip_blocks_when_inventory_has_no_free_slot() {
    let mut inventory = full_inventory_of_pipes();
    let mut equipment = Equipment {
        armor: Some(SCRAP_VEST_ID.to_string()),
        ..Equipment::default()
    };

    let error = unequip_to_inventory_slot(&mut inventory, &mut equipment, EquipmentSlot::Armor)
        .expect_err("expected unequip to fail when inventory is full");

    assert_eq!(error, InventoryRuleError::InventoryFull);
    assert_eq!(equipment.armor.as_deref(), Some(SCRAP_VEST_ID));
}

#[test]
fn equip_rejects_invalid_inventory_slot() {
    let mut inventory = Inventory::default();
    let mut equipment = Equipment::default();

    let error = equip_from_inventory_slot(&mut inventory, &mut equipment, 99)
        .expect_err("expected invalid slot to be rejected");

    assert_eq!(error, InventoryRuleError::InvalidInventorySlot);
}

#[test]
fn unequip_rejects_empty_equipment_slot() {
    let mut inventory = Inventory::default();
    let mut equipment = Equipment::default();

    let error = unequip_to_inventory_slot(&mut inventory, &mut equipment, EquipmentSlot::Weapon)
        .expect_err("expected empty unequip to fail");

    assert_eq!(error, InventoryRuleError::NothingEquipped);
}

#[test]
fn use_item_action_still_routes_to_pending_action_use_item() {
    let mut world = World::new();

    world.insert_resource(InventoryUiAction(Some(InventoryUiChoice::UseItem(3))));
    world.insert_resource(InventoryOpen(true));
    world.insert_resource(InventoryUiStatus::default());
    world.insert_resource(PlayerAction(None));
    world.insert_resource(State::new(AppState::Dungeon));
    world.insert_resource(State::new(TurnPhase::AwaitingInput));
    world.insert_resource(NextState::<TurnPhase>::default());
    world.spawn((Player, Inventory::default(), Equipment::default()));

    let _ = world.run_system_once(process_inventory_action);

    match world.resource::<PlayerAction>().0.as_ref() {
        Some(PendingAction::UseItem(slot_idx)) => assert_eq!(*slot_idx, 3),
        _ => panic!("expected pending use-item action to remain authoritative"),
    }
}

#[test]
fn use_item_action_does_not_route_outside_awaiting_input() {
    let mut world = World::new();

    world.insert_resource(InventoryUiAction(Some(InventoryUiChoice::UseItem(4))));
    world.insert_resource(InventoryOpen(true));
    world.insert_resource(InventoryUiStatus::default());
    world.insert_resource(PlayerAction(None));
    world.insert_resource(State::new(AppState::Dungeon));
    world.insert_resource(State::new(TurnPhase::EnemyTurn));
    world.insert_resource(NextState::<TurnPhase>::default());
    world.spawn((Player, Inventory::default(), Equipment::default()));

    let _ = world.run_system_once(process_inventory_action);

    assert!(
        world.resource::<PlayerAction>().0.is_none(),
        "expected use-item routing to stay disabled outside AwaitingInput"
    );
}

#[test]
fn equip_rollback_preserves_items_when_swap_cannot_return_replaced_gear() {
    let mut inventory = Inventory::default();
    let mut equipment = Equipment::default();

    inventory.slots[0] = Some(ItemStack {
        item_id: HUNTING_KNIFE_ID.to_string(),
        quantity: 2,
    });
    for slot_index in 1..INVENTORY_FULL_COUNT {
        inventory.slots[slot_index] = Some(ItemStack {
            item_id: IRON_PIPE_ID.to_string(),
            quantity: 1,
        });
    }
    equipment.weapon = Some(IRON_PIPE_ID.to_string());

    let error = equip_from_inventory_slot(&mut inventory, &mut equipment, 0)
        .expect_err("expected swap to fail when return path has no free slot");

    assert_eq!(error, InventoryRuleError::InventoryFull);
    assert_eq!(equipment.weapon.as_deref(), Some(IRON_PIPE_ID));
    assert_eq!(inventory.count(HUNTING_KNIFE_ID), 2);
    let total_iron_pipes = inventory.count(IRON_PIPE_ID)
        + if equipment.weapon.as_deref() == Some(IRON_PIPE_ID) {
            1
        } else {
            0
        };
    assert_eq!(total_iron_pipes, INVENTORY_FULL_COUNT as u8);
}
