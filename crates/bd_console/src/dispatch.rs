//! Dispatch system — parses [`ConsoleCommand`] messages and executes them.

use bevy_ecs::prelude::*;

/// System that reads [`ConsoleCommand`] messages, parses them, and dispatches
/// to the appropriate game system via signals or direct mutation.
///
/// Registered in `BdSet::Mutation` (runs after validation — debug commands
/// intentionally bypass gameplay validation).
#[allow(unused_variables, unused_mut)]
pub fn execute_console_command(world: &mut World) {
    // Stub — implementation in Phase 4 after dispatch tests pass.
    // Reads MessageReader<ConsoleCommand>, parses, matches DebugCommand variant,
    // dispatches via MessageWriter<EventTrigger>/MessageWriter<PoolDeltaRequested>
    // or direct ResMut<GameTime>/ResMut<CurrentEvent>/query mutations.
    let _ = world;
}
