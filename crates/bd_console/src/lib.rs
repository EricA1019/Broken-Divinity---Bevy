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

    // ── Phase 6 RED: system wiring tests ──
    //
    // These FAIL until capture_console_input, execute_console_command,
    // and render_console are registered in BdConsolePlugin::build().

    /// Backtick must toggle ConsoleState.open when capture system is registered.
    #[test]
    fn backtick_toggles_console_open() {
        let mut app = App::new();
        app.add_plugins(BdConsolePlugin);
        app.add_message::<bevy_ratatui::event::KeyMessage>();

        // Fire a backtick key event
        {
            use crossterm::event::{KeyEvent, KeyEventKind, KeyModifiers};
            let mut msgs = app.world_mut()
                .resource_mut::<bevy_ecs::message::Messages<bevy_ratatui::event::KeyMessage>>();
            msgs.write(bevy_ratatui::event::KeyMessage(KeyEvent::new_with_kind(
                crossterm::event::KeyCode::Char('`'),
                KeyModifiers::NONE,
                KeyEventKind::Press,
            )));
        }

        app.update();

        let state = app.world().resource::<ConsoleState>();
        assert!(state.open, "backtick must open the console");
    }

    /// Typing populates buffer when console is open.
    #[test]
    fn typing_populates_buffer_when_open() {
        let mut app = App::new();
        app.add_plugins(BdConsolePlugin);
        app.add_message::<bevy_ratatui::event::KeyMessage>();

        // Open console with backtick
        {
            use crossterm::event::{KeyEvent, KeyEventKind, KeyModifiers};
            let mut msgs = app.world_mut()
                .resource_mut::<bevy_ecs::message::Messages<bevy_ratatui::event::KeyMessage>>();
            msgs.write(bevy_ratatui::event::KeyMessage(KeyEvent::new_with_kind(
                crossterm::event::KeyCode::Char('`'),
                KeyModifiers::NONE,
                KeyEventKind::Press,
            )));
            msgs.write(bevy_ratatui::event::KeyMessage(KeyEvent::new_with_kind(
                crossterm::event::KeyCode::Char('s'),
                KeyModifiers::NONE,
                KeyEventKind::Press,
            )));
        }

        app.update();

        let state = app.world().resource::<ConsoleState>();
        assert!(state.open, "console must be open after backtick");
        assert_eq!(state.buffer, "s", "typing 's' must populate buffer");
    }

    /// Enter dispatches command to ConsoleState.pending.
    #[test]
    fn enter_dispatches_to_pending() {
        let mut app = App::new();
        app.add_plugins(BdConsolePlugin);
        app.add_message::<bevy_ratatui::event::KeyMessage>();

        // Open console, type command, press Enter
        {
            use crossterm::event::{KeyEvent, KeyEventKind, KeyModifiers};
            let mut msgs = app.world_mut()
                .resource_mut::<bevy_ecs::message::Messages<bevy_ratatui::event::KeyMessage>>();
            msgs.write(bevy_ratatui::event::KeyMessage(KeyEvent::new_with_kind(
                crossterm::event::KeyCode::Char('`'),
                KeyModifiers::NONE,
                KeyEventKind::Press,
            )));
            msgs.write(bevy_ratatui::event::KeyMessage(KeyEvent::new_with_kind(
                crossterm::event::KeyCode::Char('h'),
                KeyModifiers::NONE,
                KeyEventKind::Press,
            )));
            msgs.write(bevy_ratatui::event::KeyMessage(KeyEvent::new_with_kind(
                crossterm::event::KeyCode::Enter,
                KeyModifiers::NONE,
                KeyEventKind::Press,
            )));
        }

        app.update();

        let state = app.world().resource::<ConsoleState>();
        assert!(state.open, "console must be open");
        assert_eq!(state.pending.len(), 1, "Enter must push one command to pending");
        assert_eq!(state.pending[0], "h", "pending must contain typed command");
        assert!(state.buffer.is_empty(), "buffer must clear after Enter");
    }

    /// Escape closes console and clears buffer.
    #[test]
    fn escape_closes_console() {
        let mut app = App::new();
        app.add_plugins(BdConsolePlugin);
        app.add_message::<bevy_ratatui::event::KeyMessage>();

        // Open console, type something, press Escape
        {
            use crossterm::event::{KeyEvent, KeyEventKind, KeyModifiers};
            let mut msgs = app.world_mut()
                .resource_mut::<bevy_ecs::message::Messages<bevy_ratatui::event::KeyMessage>>();
            msgs.write(bevy_ratatui::event::KeyMessage(KeyEvent::new_with_kind(
                crossterm::event::KeyCode::Char('`'),
                KeyModifiers::NONE,
                KeyEventKind::Press,
            )));
            msgs.write(bevy_ratatui::event::KeyMessage(KeyEvent::new_with_kind(
                crossterm::event::KeyCode::Char('x'),
                KeyModifiers::NONE,
                KeyEventKind::Press,
            )));
            msgs.write(bevy_ratatui::event::KeyMessage(KeyEvent::new_with_kind(
                crossterm::event::KeyCode::Esc,
                KeyModifiers::NONE,
                KeyEventKind::Press,
            )));
        }

        app.update();

        let state = app.world().resource::<ConsoleState>();
        assert!(!state.open, "Escape must close console");
        assert!(state.buffer.is_empty(), "buffer must clear on close");
    }
}
