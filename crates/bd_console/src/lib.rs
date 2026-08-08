//! Developer CLI console — quakelike debug overlay for Broken Divinity.
//!
//! Toggled with backtick (`). Provides a command-line interface for spawning
//! entities, manipulating resources, triggering events, and inspecting game
//! state. All commands flow through a single [`ConsoleCommand`] message.

pub mod commands;
pub mod dispatch;
pub mod input;
pub mod render;
pub mod state;

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;

pub use state::ConsoleState;

/// A command entered at the console prompt.
///
/// Fired after the user presses Enter. The raw string is parsed into a
/// [`commands::DebugCommand`] by the dispatch system.
#[derive(Message, Debug, Clone)]
pub struct ConsoleCommand(pub String);

/// Minimal plugin that registers the console message, systems, and resources.
///
/// Add **after** `BdCorePlugin` (needs `BlueprintCatalog`, `EventRegistry`,
/// `GameTime`, etc.) and **before** `BdTuiPlugin` (the input guard in
/// `map_input_to_intents` depends on `ConsoleState`).
pub struct BdConsolePlugin;

impl Plugin for BdConsolePlugin {
    fn build(&self, app: &mut App) {
        // Message
        app.add_message::<ConsoleCommand>();

        // Resources
        app.init_resource::<ConsoleState>();

        // Systems — registered in Phase 3-5.
        // Input capture and dispatch are wired after their tests pass.
    }
}
