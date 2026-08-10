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
    /// Commands pending dispatch (pushed by input, consumed by dispatch each frame).
    pub pending: Vec<String>,
    /// True when the current physical batch is owned by the console.
    /// Set before the reducer mutates `open`; checked by gameplay routing.
    /// This prevents close/toggle keys from leaking into gameplay in the
    /// same physical batch after the reducer changes open state.
    #[serde(skip)]
    pub batch_capture_active: bool,
}
