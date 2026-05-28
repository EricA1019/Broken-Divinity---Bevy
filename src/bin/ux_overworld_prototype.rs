//! Dedicated launcher for overworld map UI prototypes.
//!
//! Run with: `cargo run --bin ux_overworld_prototype`

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use broken_divinity::ui::ux_overworld_prototype::OverworldPrototypePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(OverworldPrototypePlugin)
        .run();
}