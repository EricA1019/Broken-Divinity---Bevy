//! Dedicated launcher for inventory + equipment prototype.
//!
//! Run with: `cargo run --bin ux_inventory_equipment_prototype`

//! DEPRECATED: prototype-only binary. Do not use for production runtime flow.

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use broken_divinity::ui::ux_inventory_equipment_prototype::InventoryEquipmentPrototypePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(InventoryEquipmentPrototypePlugin)
        .run();
}
