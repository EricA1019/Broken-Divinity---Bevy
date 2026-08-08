//! Console state — open/closed, input buffer, history, and output log.

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

/// Live state of the developer console.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConsoleState {
    /// Whether the console overlay is visible and accepting input.
    pub open: bool,
    /// Current command being typed.
    pub buffer: String,
    /// Cursor position within the buffer (byte index).
    pub cursor: usize,
    /// Previously executed commands, most recent last.
    pub history: Vec<String>,
    /// Position in the history being navigated (None = not navigating).
    pub history_idx: Option<usize>,
    /// Output lines shown in the console log (most recent last).
    pub output: Vec<String>,
    /// Scroll offset within the output log.
    pub scroll: usize,
}
