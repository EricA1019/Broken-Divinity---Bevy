//! Console overlay renderer — draws the debug console on top of the active screen.

use bevy_ecs::prelude::*;

use crate::state::ConsoleState;

/// Draws the console overlay when [`ConsoleState::open`] is true.
///
/// Registered in `BdSet::Render` with `.after(draw_ui)` ordering — writes
/// to the same `RatatuiContext` frame as the main UI, overlaying the
/// bottom 40% of the terminal.
#[allow(unused_variables)]
pub fn render_console(
    state: Res<ConsoleState>,
    // ratatui_ctx: Option<ResMut<bevy_ratatui::RatatuiContext>>,
) {
    // Stub — implementation in Phase 5.
    // Uses RatatuiContext to draw a framed overlay with output log and prompt.
}
