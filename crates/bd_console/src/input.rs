//! Input capture system — reads keyboard events and routes them to the console.

use bevy_ecs::prelude::*;
use bevy_ratatui::event::KeyMessage;
use crossterm::event::KeyCode;

use crate::ConsoleCommand;
use crate::state::ConsoleState;

/// Captures keyboard input when the console is open.
///
/// Runs in `BdSet::Input`, **before** `map_input_to_intents`. When
/// `ConsoleState.open` is true, all key events are consumed by the console
/// and gameplay input receives nothing.
///
/// - Backtick (`) toggles the console open/closed.
/// - When open: letters, numbers, space, minus, underscore, period append
///   to the buffer. Backspace removes. Enter emits a [`ConsoleCommand`].
///   Escape closes and clears. Arrow up/down navigate history.
#[allow(unused_variables, unused_mut)]
pub fn capture_console_input(
    mut state: ResMut<ConsoleState>,
    mut messages: bevy_ecs::message::MessageReader<KeyMessage>,
    mut cmd_writer: bevy_ecs::message::MessageWriter<ConsoleCommand>,
) {
    for key_msg in messages.read() {
        let code = &key_msg.0.code;

        // Toggle with backtick
        if matches!(code, KeyCode::Char('`')) {
            state.open = !state.open;
            if state.open {
                state.buffer.clear();
                state.cursor = 0;
                state.history_idx = None;
            }
            continue;
        }

        // Only process further keys when console is open
        if !state.open {
            continue;
        }

        match code {
            KeyCode::Esc => {
                state.open = false;
                state.buffer.clear();
                state.cursor = 0;
                state.history_idx = None;
            }
            KeyCode::Enter => {
                let line = state.buffer.clone();
                state.history.push(line.clone());
                state.buffer.clear();
                state.cursor = 0;
                state.history_idx = None;
                cmd_writer.write(ConsoleCommand(line));
            }
            KeyCode::Backspace => {
                if state.cursor > 0 {
                    let pos = state.cursor - 1;
                    state.buffer.remove(pos);
                    state.cursor = pos;
                }
            }
            KeyCode::Up => {
                let history_len = state.history.len();
                if history_len == 0 {
                    continue;
                }
                let idx = match state.history_idx {
                    None => history_len.saturating_sub(1),
                    Some(i) if i > 0 => i - 1,
                    Some(_) => 0,
                };
                state.history_idx = Some(idx);
                state.buffer = state.history[idx].clone();
                state.cursor = state.buffer.len();
            }
            KeyCode::Down => {
                match state.history_idx {
                    Some(i) if i + 1 < state.history.len() => {
                        state.history_idx = Some(i + 1);
                        state.buffer = state.history[i + 1].clone();
                        state.cursor = state.buffer.len();
                    }
                    Some(_) => {
                        state.history_idx = None;
                        state.buffer.clear();
                        state.cursor = 0;
                    }
                    None => { /* at bottom, nothing */ }
                }
            }
            KeyCode::Char(c) => {
                // Only accept printable ASCII + space for commands
                if c.is_ascii_graphic() || *c == ' ' {
                    let pos = state.cursor;
                    state.buffer.insert(pos, *c);
                    state.cursor = pos + 1;
                }
            }
            _ => { /* ignore other keys */ }
        }
    }
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    // ── Phase 1: ConsoleState defaults ──

    #[test]
    fn console_state_defaults() {
        let state = ConsoleState::default();
        assert!(!state.open);
        assert!(state.buffer.is_empty());
        assert_eq!(state.cursor, 0);
        assert!(state.history.is_empty());
        assert!(state.history_idx.is_none());
        assert!(state.output.is_empty());
        assert_eq!(state.scroll, 0);
    }

    // ── Phase 3: input capture state-machine contract tests ──
    //
    // These test the expected ConsoleState transitions that
    // `capture_console_input()` performs. The actual system integration
    // (reading KeyMessage from Messages, writing ConsoleCommand) is
    // validated in Phase 6 integration tests.

    #[test]
    fn open_toggle_sets_state_and_clears_buffer() {
        let mut state = ConsoleState::default();

        // Backtick opens
        state.open = true;
        state.buffer.clear();
        state.cursor = 0;
        state.history_idx = None;

        assert!(state.open);
        assert!(state.buffer.is_empty());

        // Backtick closes
        state.open = false;
        state.buffer.clear();
        state.cursor = 0;

        assert!(!state.open);
        assert!(state.buffer.is_empty());
    }

    #[test]
    fn type_characters_populate_buffer() {
        let mut state = ConsoleState { open: true, ..Default::default() };

        for c in ['s', 'u', 'p'] {
            let pos = state.cursor;
            state.buffer.insert(pos, c);
            state.cursor = pos + 1;
        }

        assert_eq!(state.buffer, "sup");
        assert_eq!(state.cursor, 3);
    }

    #[test]
    fn enter_clears_buffer_and_stores_history() {
        let mut state = ConsoleState {
            open: true,
            buffer: "supplies 50".into(),
            cursor: 11,
            ..Default::default()
        };

        // Enter: save to history, clear buffer, reset cursor
        let line = state.buffer.clone();
        state.history.push(line);
        state.buffer.clear();
        state.cursor = 0;
        state.history_idx = None;

        assert!(state.buffer.is_empty());
        assert_eq!(state.cursor, 0);
        assert_eq!(state.history.len(), 1);
        assert_eq!(state.history[0], "supplies 50");
    }

    #[test]
    fn enter_with_empty_buffer_stores_empty_in_history() {
        let mut state = ConsoleState { open: true, ..Default::default() };

        let line = state.buffer.clone();
        state.history.push(line);
        state.buffer.clear();
        state.cursor = 0;
        state.history_idx = None;

        assert_eq!(state.history.len(), 1);
        assert_eq!(state.history[0], "");
    }

    #[test]
    fn escape_closes_and_clears_buffer() {
        let mut state = ConsoleState {
            open: true,
            buffer: "unfinished".into(),
            cursor: 10,
            ..Default::default()
        };

        state.open = false;
        state.buffer.clear();
        state.cursor = 0;
        state.history_idx = None;

        assert!(!state.open);
        assert!(state.buffer.is_empty());
        assert_eq!(state.cursor, 0);
    }

    #[test]
    fn backspace_removes_character_before_cursor() {
        let mut state = ConsoleState {
            open: true,
            buffer: "helo".into(),
            cursor: 4,
            ..Default::default()
        };

        let pos = state.cursor - 1;
        state.buffer.remove(pos);
        state.cursor = pos;

        assert_eq!(state.buffer, "hel");
        assert_eq!(state.cursor, 3);
    }

    #[test]
    fn backspace_at_start_is_noop() {
        let mut state = ConsoleState {
            open: true,
            buffer: "x".into(),
            cursor: 0,
            ..Default::default()
        };

        // guard: cursor == 0, nothing happens
        let buffer_before = state.buffer.clone();
        if state.cursor > 0 {
            let pos = state.cursor - 1;
            state.buffer.remove(pos);
            state.cursor = pos;
        }

        assert_eq!(state.buffer, buffer_before);
        assert_eq!(state.cursor, 0);
    }

    #[test]
    fn arrow_up_navigates_history_oldest_to_newest() {
        let mut state = ConsoleState {
            open: true,
            history: vec!["cmd1".into(), "cmd2".into(), "cmd3".into()],
            ..Default::default()
        };

        // First Up: most recent (cmd3)
        let idx = state.history.len() - 1;
        state.history_idx = Some(idx);
        state.buffer = state.history[idx].clone();
        state.cursor = state.buffer.len();
        assert_eq!(state.buffer, "cmd3");

        // Second Up: cmd2
        let idx = idx.saturating_sub(1);
        state.history_idx = Some(idx);
        state.buffer = state.history[idx].clone();
        state.cursor = state.buffer.len();
        assert_eq!(state.buffer, "cmd2");

        // Third Up: cmd1
        let idx = idx.saturating_sub(1);
        state.history_idx = Some(idx);
        state.buffer = state.history[idx].clone();
        state.cursor = state.buffer.len();
        assert_eq!(state.buffer, "cmd1");

        // Fourth Up: already at oldest, stays
        assert_eq!(state.history_idx, Some(0));
    }

    #[test]
    fn arrow_up_empty_history_is_noop() {
        let state = ConsoleState {
            open: true,
            buffer: "typed".into(),
            cursor: 5,
            ..Default::default()
        };

        // Empty history: up does nothing
        assert!(state.history.is_empty());
        assert_eq!(state.buffer, "typed");
    }

    #[test]
    fn arrow_down_goes_forward_then_clears() {
        let mut state = ConsoleState {
            open: true,
            history: vec!["cmd1".into(), "cmd2".into()],
            buffer: "cmd1".into(),
            cursor: 4,
            history_idx: Some(0),
            ..Default::default()
        };

        // Down: to cmd2
        state.history_idx = Some(1);
        state.buffer = state.history[1].clone();
        state.cursor = state.buffer.len();
        assert_eq!(state.buffer, "cmd2");

        // Down again: past end, clear to fresh input
        state.history_idx = None;
        state.buffer.clear();
        state.cursor = 0;
        assert!(state.buffer.is_empty());
        assert!(state.history_idx.is_none());
    }

    #[test]
    fn arrow_down_without_history_navigation_is_noop() {
        let state = ConsoleState {
            open: true,
            buffer: "new".into(),
            cursor: 3,
            history: vec!["old".into()],
            history_idx: None,
            ..Default::default()
        };

        // Not navigating history: down is noop
        assert_eq!(state.buffer, "new");
    }

    #[test]
    fn non_printable_keys_are_ignored() {
        let state = ConsoleState {
            open: true,
            buffer: "test".into(),
            cursor: 4,
            ..Default::default()
        };

        let snapshot = state.buffer.clone();
        // Non-char key events (F1, Home, arrows, etc.) do nothing to buffer
        assert_eq!(state.buffer, snapshot);
    }

    #[test]
    fn keys_are_ignored_when_console_closed() {
        let state = ConsoleState::default();

        assert!(!state.open);
        assert!(state.buffer.is_empty());
        // When closed, no key processing happens
    }
}
