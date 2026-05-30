//! Dedicated launcher for dungeon styling variants.
//! Kept isolated from the dungeon-map readability lab.

//! DEPRECATED: prototype-only binary. Do not use for production runtime flow.

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use broken_divinity::ui::ux_dungeon_style_prototype::DungeonStylePrototypePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(DungeonStylePrototypePlugin)
        .run();
}
