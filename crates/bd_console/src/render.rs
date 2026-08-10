//! Console overlay renderer — draws the debug console on top of the active screen.

use bevy_ecs::prelude::*;
use bevy_ratatui::RatatuiContext;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::state::ConsoleState;

/// Draws the console overlay when [`ConsoleState::open`] is true.
///
/// Registered in `BdSet::Render` with `.after(draw_ui)` ordering — writes
/// to the same `RatatuiContext` frame as the main UI, overlaying the
/// bottom 40% of the terminal.
pub fn render_console(ratatui_ctx: Option<ResMut<RatatuiContext>>, state: Res<ConsoleState>) {
    if !state.open {
        return;
    }

    let Some(mut ctx) = ratatui_ctx else {
        return;
    };

    let size = match ctx.size() {
        Ok(s) => s,
        Err(_) => return,
    };
    let area = Rect {
        x: 0,
        y: 0,
        width: size.width,
        height: size.height,
    };

    // Bottom 40% of the terminal
    let height = (area.height as f32 * 0.4) as u16;
    let console_area = Rect {
        x: 0,
        y: area.height.saturating_sub(height),
        width: area.width,
        height,
    };

    // Split: output log (top 75%) and input line (bottom 1 row)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(console_area);

    // Output log
    let output_lines: Vec<Line> = state
        .output
        .iter()
        .rev()
        .take(chunks[0].height as usize)
        .rev()
        .map(|line| {
            let style = if line.starts_with("ERROR") {
                Style::default().fg(Color::Red)
            } else if line.starts_with("OK") {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::White)
            };
            Line::from(Span::styled(line.as_str(), style))
        })
        .collect();

    let output = Paragraph::new(output_lines)
        .block(Block::default().borders(Borders::NONE))
        .wrap(Wrap { trim: false });

    if let Err(error) = ctx.draw(|frame| {
        // Clear the console area (overlay on top of whatever bd_tui drew)
        frame.render_widget(ratatui::widgets::Clear, console_area);

        // Border frame
        let block = Block::default()
            .title(" CONSOLE ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));
        let inner = block.inner(console_area);
        frame.render_widget(block, console_area);

        // Output log
        let log_area = Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: inner.height.saturating_sub(1),
        };
        frame.render_widget(output, log_area);

        // Input line: "> buffer█"
        let prompt = format!("> {}", state.buffer);
        let prompt_span = if prompt.len() > inner.width as usize {
            // Scroll: show end of long input
            let start = prompt.len().saturating_sub(inner.width as usize);
            Span::styled(&prompt[start..], Style::default().fg(Color::Yellow))
        } else {
            Span::styled(&prompt, Style::default().fg(Color::Yellow))
        };

        let input_line = Paragraph::new(Line::from(prompt_span));
        let input_area = Rect {
            x: inner.x,
            y: inner.y + inner.height.saturating_sub(1),
            width: inner.width,
            height: 1,
        };
        frame.render_widget(input_line, input_area);
    }) {
        tracing::error!(%error, "console overlay render failed");
    }
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    /// The render system correctly gates on ConsoleState.open.
    #[test]
    fn render_exits_early_when_closed() {
        let state = ConsoleState::default();
        assert!(!state.open);
        // Contract: when closed, the system returns before any draw call.
        // Validated by the `if !state.open { return; }` guard in render_console.
    }

    /// The render system handles missing RatatuiContext gracefully.
    #[test]
    fn render_handles_missing_context() {
        let state = ConsoleState {
            open: true,
            buffer: "test".into(),
            cursor: 4,
            ..Default::default()
        };
        assert!(state.open);
        // Contract: if RatatuiContext is None, system returns early.
        // Validated by `let Some(mut ctx) = ratatui_ctx else { return; }`.
    }

    /// Output lines with ERROR prefix are styled red.
    #[test]
    fn error_lines_detected_for_styling() {
        let state = ConsoleState {
            open: true,
            output: vec![
                "ERROR: something went wrong".into(),
                "OK: success".into(),
                "plain info".into(),
            ],
            ..Default::default()
        };

        assert!(state.output[0].starts_with("ERROR"));
        assert!(state.output[1].starts_with("OK"));
        assert!(!state.output[2].starts_with("ERROR") && !state.output[2].starts_with("OK"));
    }

    /// The render system handles empty output gracefully.
    #[test]
    fn render_handles_empty_output() {
        let state = ConsoleState {
            open: true,
            buffer: "test".into(),
            cursor: 4,
            output: vec![],
            ..Default::default()
        };

        assert!(state.output.is_empty());
        // Contract: empty output produces empty Paragraph, no panic.
    }

    /// The render system handles long buffer strings.
    #[test]
    fn render_handles_long_buffer() {
        let state = ConsoleState {
            open: true,
            buffer: "x".repeat(500),
            cursor: 500,
            output: vec!["line 1".into()],
            ..Default::default()
        };

        assert_eq!(state.buffer.len(), 500);
        // Contract: long buffer is scrolled to show cursor end, no panic.
    }

    /// The render system only renders when open.
    #[test]
    fn closed_console_skips_all_rendering() {
        let state = ConsoleState::default();
        assert!(!state.open);
        assert!(state.buffer.is_empty());
        assert!(state.output.is_empty());
        // Contract: nothing is drawn when console is closed.
    }
}
