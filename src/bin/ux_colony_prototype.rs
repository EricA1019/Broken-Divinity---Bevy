//! Dedicated launcher for colony layout prototypes.

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use broken_divinity::ui::ux_colony_prototype::ColonyPrototypePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(ColonyPrototypePlugin)
        .run();
}
