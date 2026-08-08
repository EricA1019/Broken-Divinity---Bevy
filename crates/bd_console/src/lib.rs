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

        // Systems — registered in Phase 6.
        // Input capture, dispatch, and render are wired after integration.
    }
}

// ── Integration tests ──

#[cfg(test)]
mod tests {
    use super::*;

    /// BdConsolePlugin must register ConsoleState as a resource.
    #[test]
    fn plugin_registers_console_state() {
        let mut app = App::new();
        app.add_plugins(BdConsolePlugin);

        let state = app.world().resource::<ConsoleState>();
        assert!(!state.open);
        assert!(state.buffer.is_empty());
        assert!(state.pending.is_empty());
        assert!(state.output.is_empty());
        assert!(state.history.is_empty());
    }

    /// BdConsolePlugin must register ConsoleCommand message type.
    #[test]
    fn plugin_registers_console_command_message() {
        let mut app = App::new();
        app.add_plugins(BdConsolePlugin);

        // Write a message and verify it's stored
        {
            let mut msgs = app.world_mut()
                .resource_mut::<bevy_ecs::message::Messages<ConsoleCommand>>();
            msgs.write(ConsoleCommand("test".into()));
        }
        let msgs = app.world()
            .resource::<bevy_ecs::message::Messages<ConsoleCommand>>();
        assert!(msgs.len() >= 1);
    }

    /// Plugin must build without panic — basic smoke test.
    #[test]
    fn plugin_builds_without_panic() {
        let mut app = App::new();
        app.add_plugins(BdConsolePlugin);
        app.update(); // run one frame

        // ConsoleState should still be present
        assert!(app.world().get_resource::<ConsoleState>().is_some());
    }

    /// ConsoleState starts with console closed.
    #[test]
    fn console_state_starts_closed() {
        let mut app = App::new();
        app.add_plugins(BdConsolePlugin);

        let state = app.world().resource::<ConsoleState>();
        assert!(!state.open, "console must start closed");
    }
}
