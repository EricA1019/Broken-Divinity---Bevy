//! DEPRECATED: prototype-only binary. Do not use for production runtime flow.

//! Unified launcher: dungeon style + colony layouts in one surface.

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use broken_divinity::ui::ux_unified_prototype::UnifiedPrototypePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(UnifiedPrototypePlugin)
        .run();
}