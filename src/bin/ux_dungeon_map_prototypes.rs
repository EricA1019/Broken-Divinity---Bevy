//! Dedicated launcher for dungeon-map readability prototypes.
//! Kept isolated from the main UX prototype surface.

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use broken_divinity::ui::ux_dungeon_map_prototypes::DungeonMapPrototypePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
    .add_plugins(EguiPlugin::default())
    .add_plugins(DungeonMapPrototypePlugin)
        .run();
}
