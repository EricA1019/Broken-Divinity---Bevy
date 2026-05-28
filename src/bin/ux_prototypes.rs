//! UX Prototype Viewer — standalone binary for comparing visual variants.
//!
//! Run with: `cargo run --bin ux_prototypes`
//!
//! Controls:
//!   M/C/D   — switch screen (Menu / Colony / Dungeon HUD)
//!   Esc     — quit

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use broken_divinity::ui::ux_prototypes::UxPrototypePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Broken Divinity — UX Prototypes".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .add_plugins(UxPrototypePlugin)
        .run();
}