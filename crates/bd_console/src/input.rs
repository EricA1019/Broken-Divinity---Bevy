//! Input capture system — the single console key reducer.

use bevy_ecs::prelude::*;
use bevy_ratatui::event::KeyMessage;
use crossterm::event::{KeyCode, KeyEventKind};

use crate::ConsoleCommand;
use crate::state::ConsoleState;

/// The only console editing reducer. Runs in `BdSet::Input`.
///
/// Owns physical key → console-state transitions. Press is actionable;
/// Repeat and Release are inert. Whole-batch capture is recorded before
/// `open` is mutated so that a close/toggle key in the same batch cannot
/// leak into gameplay routing.
///
/// The TUI adapter may order this system before gameplay input and may check
/// `ConsoleState.batch_capture_active`, but must not duplicate editing,
/// history, submission, or completion rules.
pub fn capture_console_input(
    mut state: ResMut<ConsoleState>,
    mut messages: bevy_ecs::message::MessageReader<KeyMessage>,
    mut cmd_writer: bevy_ecs::message::MessageWriter<ConsoleCommand>,
) {
    // Own the batch only when the console was already open or a
    // backtick toggles it — normal gameplay keys pass through freely.
    let has_keys = !messages.is_empty();
    if !has_keys {
        state.batch_capture_active = false;
        return;
    }
    // Start with the current open state; backtick upgrades below.
    state.batch_capture_active = state.open;

    let mut owned = state.open;
    for key_msg in messages.read() {
        if key_msg.0.kind != KeyEventKind::Press {
            continue;
        }
        let code = &key_msg.0.code;

        if matches!(code, KeyCode::Char('`')) {
            state.open = !state.open;
            owned = state.open;
            // Backtick always owns the batch — must not leak into gameplay.
            state.batch_capture_active = true;
            if state.open {
                state.buffer.clear();
                state.cursor = 0;
                state.history_idx = None;
                state.output.push("— DEBUG CONSOLE — Type 'help' for available commands. Press ` or Esc to close.".into());
            }
            continue;
        }

        if !owned {
            continue;
        }

        match code {
            KeyCode::Esc => {
                state.open = false;
                owned = false;
                state.buffer.clear();
                state.cursor = 0;
                state.history_idx = None;
            }
            KeyCode::Enter => {
                let line = state.buffer.clone();
                if !line.is_empty() {
                    state.history.push(line.clone());
                }
                cmd_writer.write(ConsoleCommand(line));
                state.buffer.clear();
                state.cursor = 0;
                state.history_idx = None;
            }
            KeyCode::Backspace => {
                if state.cursor > 0 {
                    let pos = state.cursor - 1;
                    state.buffer.remove(pos);
                    state.cursor = pos;
                }
            }
            KeyCode::Tab => {
                tab_complete(&mut state);
            }
            KeyCode::Up => {
                history_search(&mut state, false);
            }
            KeyCode::Down => {
                history_search(&mut state, true);
            }
            KeyCode::Char(c) if c.is_ascii_graphic() || *c == ' ' => {
                let pos = state.cursor;
                state.buffer.insert(pos, *c);
                state.cursor = pos + 1;
                state.history_idx = None;
            }
            _ => {}
        }
    }
}

const COMMAND_NAMES: &[&str] = &[
    "supplies",
    "materials",
    "faith",
    "plants",
    "day",
    "turn",
    "skip_day",
    "event",
    "end_event",
    "kill_all",
    "heal",
    "god",
    "survivor",
    "task",
    "spawn",
    "goto",
    "shelter",
    "blueprints",
    "events",
    "stats",
    "help",
    "clear",
    "s",
    "m",
    "f",
    "p",
];

/// Tab-complete the current buffer against known command names.
fn tab_complete(state: &mut crate::state::ConsoleState) {
    let prefix = state.buffer.trim();
    if prefix.is_empty() {
        // Show all top-level commands
        state.output.push(format!(
            "Commands: {}",
            COMMAND_NAMES
                .iter()
                .filter(|n| !n.len() == 1 || n.starts_with(|c: char| c.is_alphabetic())) // skip aliases in listing
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        ));
        return;
    }

    let matches: Vec<&&str> = COMMAND_NAMES
        .iter()
        .filter(|n| n.starts_with(prefix) && n.len() > 1) // skip single-char aliases unless exact
        .collect();

    match matches.len() {
        0 => {
            state.output.push(format!("No matches for '{}'", prefix));
        }
        1 => {
            // Complete with a trailing space
            state.buffer = format!("{} ", matches[0]);
            state.cursor = state.buffer.len();
        }
        _ => {
            // Show all matches
            state.output.push(format!(
                "{} matches: {}",
                matches.len(),
                matches.iter().map(|s| **s).collect::<Vec<_>>().join(", ")
            ));
            // Complete the common prefix
            let strs: Vec<&str> = matches.iter().map(|s| **s).collect();
            if let Some(common) = common_prefix(&strs) {
                if common.len() > prefix.len() {
                    state.buffer = common.to_string();
                    state.cursor = state.buffer.len();
                }
            }
        }
    }
}

/// Navigate history filtered by the current buffer prefix.
/// `forward`: false = Up (older), true = Down (newer).
fn history_search(state: &mut crate::state::ConsoleState, forward: bool) {
    let prefix = &state.buffer;
    let matches: Vec<(usize, &String)> = state
        .history
        .iter()
        .enumerate()
        .filter(|(_, h)| h.starts_with(prefix.as_str()))
        .collect();

    if matches.is_empty() {
        return;
    }

    let current_match = state
        .history_idx
        .and_then(|idx| matches.iter().position(|(i, _)| *i == idx));

    let new_pos = match (forward, current_match) {
        // Down (forward): next match, or clear to fresh input
        (true, Some(pos)) if pos + 1 < matches.len() => Some(matches[pos + 1].0),
        (true, _) => {
            // Past last match — clear to fresh input
            state.history_idx = None;
            state.buffer.clear();
            state.cursor = 0;
            return;
        }
        // Up (backward): previous match, or wrap to last
        (false, Some(pos)) if pos > 0 => Some(matches[pos - 1].0),
        (false, _) => {
            // First Up press or at top — go to last match
            matches.last().map(|(i, _)| *i)
        }
    };

    if let Some(idx) = new_pos {
        state.history_idx = Some(idx);
        state.buffer = state.history[idx].clone();
        state.cursor = state.buffer.len();
    }
}

/// Find the longest common prefix among a list of strings.
fn common_prefix<'a>(strings: &[&'a str]) -> Option<&'a str> {
    let first = strings.first()?;
    let mut end = first.len();
    for s in strings.iter().skip(1) {
        end = end.min(
            s.bytes()
                .zip(first.bytes())
                .take_while(|(a, b)| a == b)
                .count(),
        );
    }
    if end == 0 { None } else { Some(&first[..end]) }
}
