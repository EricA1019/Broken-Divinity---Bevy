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

/// System set for the console input reducer. Ordered explicitly before
/// gameplay routing within `BdSet::Input` so close keys cannot leak.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConsoleCaptureSet;

/// Bridge: reads `ConsoleCommand` messages emitted by the reducer and
/// queues them into `ConsoleState.pending` for the exclusive Mutation
/// dispatcher. This is the single production bridge from typed command to
/// dispatch queue. Runs in `BdSet::Mutation` before `execute_console_command`.
fn bridge_console_commands(
    mut console: ResMut<ConsoleState>,
    mut commands: bevy_ecs::message::MessageReader<ConsoleCommand>,
) {
    for cmd in commands.read() {
        console.pending.push(cmd.0.clone());
    }
}

/// Production result reader: carries typed core results into console output.
/// Runs after Mutation so the resolver's result is available the same frame.
fn read_debug_results(
    mut console: ResMut<ConsoleState>,
    mut results: bevy_ecs::message::MessageReader<bd_core::debug::DebugMutationResult>,
) {
    for result in results.read() {
        console.output.push(result.message.clone());
    }
}

/// Minimal plugin that registers the console message, systems, and resources.
///
/// Add **after** `BdCorePlugin` (needs `BlueprintCatalog`, `EventRegistry`,
/// `GameTime`, etc.) and **before** `BdTuiPlugin` (the ordering in
/// `BdTuiPlugin` depends on `ConsoleCaptureSet`).
pub struct BdConsolePlugin;

impl Plugin for BdConsolePlugin {
    fn build(&self, app: &mut App) {
        // Messages
        app.add_message::<ConsoleCommand>();
        app.add_message::<bd_core::debug::DebugMutationRequest>();
        app.add_message::<bd_core::debug::DebugMutationResult>();

        // Resources
        app.init_resource::<ConsoleState>();

        // Explicit opt-in: installing the development console grants this app
        // debug-mutation authority. Core runtimes remain disabled.
        if let Some(mut gate) = app
            .world_mut()
            .get_resource_mut::<bd_core::debug::DebugMutationGate>()
        {
            gate.enabled = true;
        }

        // Systems
        // Console input reducer: owns all physical key editing in Input.
        app.add_systems(
            bevy_app::Update,
            (input::capture_console_input
                .in_set(ConsoleCaptureSet)
                .in_set(bd_core::BdSet::Input),),
        );
        // Bridge: carries ConsoleCommand to pending in Mutation, before dispatch.
        app.add_systems(
            bevy_app::Update,
            bridge_console_commands
                .in_set(bd_core::BdSet::Mutation)
                .before(dispatch::execute_console_command),
        );
        // Exclusive dispatcher: parses and emits typed requests in Mutation,
        // explicitly before the named core resolver.
        app.add_systems(
            bevy_app::Update,
            dispatch::execute_console_command
                .in_set(bd_core::BdSet::Mutation)
                .before(bd_core::debug::DebugMutationSet::Resolve),
        );
        // Production result reader: core results become console output.
        app.add_systems(
            bevy_app::Update,
            read_debug_results.in_set(bd_core::BdSet::ResultEmission),
        );
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
            let mut msgs = app
                .world_mut()
                .resource_mut::<bevy_ecs::message::Messages<ConsoleCommand>>();
            msgs.write(ConsoleCommand("test".into()));
        }
        let msgs = app
            .world()
            .resource::<bevy_ecs::message::Messages<ConsoleCommand>>();
        assert!(!msgs.is_empty());
    }

    /// Plugin must build without panic — basic smoke test.
    #[test]
    fn plugin_builds_without_panic() {
        let mut app = App::new();
        app.add_plugins(BdConsolePlugin);
        app.add_message::<bevy_ratatui::event::KeyMessage>();
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

    /// Input→dispatch pipeline: pending commands flow through dispatch to output.
    #[test]
    fn input_to_dispatch_pipeline() {
        let mut world = World::new();
        world.init_resource::<ConsoleState>();
        world.init_resource::<bd_core::time::GameTime>();
        world.init_resource::<bd_core::events::CurrentEvent>();
        world.init_resource::<bd_core::events::EventRegistry>();
        world.init_resource::<bd_core::colony::production::ColonyResources>();
        world.init_resource::<bd_core::factory::BlueprintCatalog>();
        world.insert_resource(bevy_ecs::message::Messages::<
            bd_core::signals::PoolDeltaRequested,
        >::default());
        world.insert_resource(
            bevy_ecs::message::Messages::<bd_core::signals::EventTrigger>::default(),
        );
        world.insert_resource(bevy_ecs::message::Messages::<
            bd_core::signals::EntityDefeated,
        >::default());
        world.insert_resource(bevy_ecs::message::Messages::<
            bd_core::spatial::TransitionIntent,
        >::default());

        world
            .resource_mut::<ConsoleState>()
            .pending
            .push("help".into());
        dispatch::execute_console_command(&mut world);

        let output = &world.resource::<ConsoleState>().output;
        assert!(
            output.iter().any(|l| l.contains("COMMANDS")),
            "dispatch must process 'help'"
        );
        assert!(world.resource::<ConsoleState>().pending.is_empty());
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
            let mut msgs = app
                .world_mut()
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
