use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

use broken_divinity::core::inventory::Equipment;
use broken_divinity::ui::help_panel::{HelpOpen, toggle_help};
use broken_divinity::ui::inventory_panel::{InventoryOpen, toggle_inventory};
use broken_divinity::ui::modal_priority::{
    ModalBlockers, ModalPriorityCoordinator, apply_modal_priority_policy,
};
use broken_divinity::ui::overworld_panel::primary_overworld_cta_label;

const KEY_TOGGLE_INVENTORY: KeyCode = KeyCode::KeyI;
const KEY_TOGGLE_TAB: KeyCode = KeyCode::Tab;
const KEY_TOGGLE_HELP: KeyCode = KeyCode::F1;
const EXPECTED_SINGLE_ACCESSORY_SLOTS: usize = 1;

#[test]
fn inventory_toggle_supports_i_key_contract() {
    let mut world = World::new();
    let mut keyboard = ButtonInput::<KeyCode>::default();
    keyboard.press(KEY_TOGGLE_INVENTORY);

    world.insert_resource(keyboard);
    world.insert_resource(InventoryOpen(false));

    let _ = world.run_system_once(toggle_inventory);

    let inventory_open = world.resource::<InventoryOpen>();
    assert!(
        inventory_open.0,
        "expected inventory toggle to open on the I key contract"
    );
}

#[test]
fn inventory_toggle_supports_tab_key_contract() {
    let mut world = World::new();
    let mut keyboard = ButtonInput::<KeyCode>::default();
    keyboard.press(KEY_TOGGLE_TAB);

    world.insert_resource(keyboard);
    world.insert_resource(InventoryOpen(false));

    let _ = world.run_system_once(toggle_inventory);

    let inventory_open = world.resource::<InventoryOpen>();
    assert!(
        inventory_open.0,
        "expected inventory toggle to open on the Tab key contract"
    );
}

#[test]
fn help_toggle_is_blocked_by_critical_modal_policy_contract() {
    let mut world = World::new();
    let mut keyboard = ButtonInput::<KeyCode>::default();
    keyboard.press(KEY_TOGGLE_HELP);

    world.insert_resource(keyboard);
    world.insert_resource(HelpOpen(false));
    world.insert_resource(ModalPriorityCoordinator);
    world.insert_resource(ModalBlockers {
        critical_modal_active: true,
    });

    let _ = world.run_system_once(toggle_help);
    let _ = world.run_system_once(apply_modal_priority_policy);

    let help_open = world.resource::<HelpOpen>();
    assert!(
        !help_open.0,
        "expected critical modal policy to keep help closed"
    );
}

#[test]
fn overworld_cta_label_remains_non_empty_contract() {
    assert!(!primary_overworld_cta_label().is_empty());
}

#[test]
fn equipment_single_accessory_baseline_contract() {
    let mut equipment = Equipment::default();
    assert!(equipment.accessory.is_none());

    equipment.accessory = Some("charm_of_witness".to_string());
    assert_eq!(equipment.accessory.as_deref(), Some("charm_of_witness"));

    let current_slots = usize::from(equipment.accessory.is_some());
    assert_eq!(
        current_slots,
        EXPECTED_SINGLE_ACCESSORY_SLOTS,
        "expected baseline runtime model to expose one accessory slot"
    );
}
