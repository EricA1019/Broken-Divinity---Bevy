//! Testable runtime boundaries for rendering and buffered semantic input.
//!
//! These resources are the production owners for render invalidation and
//! bounded semantic input buffering. They remain internal to the TUI runtime.

use std::collections::VecDeque;

use bevy_ecs::prelude::Resource;

use super::commands::UiCommand;

pub(crate) const INPUT_QUEUE_CAPACITY: usize = 4;

#[derive(Resource, Debug, Default)]
pub(crate) struct RenderInvalidation {
    last_drawn_fingerprint: Option<u64>,
    pending_fingerprint: Option<u64>,
    draw_count: u64,
    error: Option<String>,
}

impl RenderInvalidation {
    pub(crate) fn needs_draw(&mut self, visible_fingerprint: u64) -> bool {
        if self.last_drawn_fingerprint == Some(visible_fingerprint) {
            self.pending_fingerprint = None;
            return false;
        }
        self.pending_fingerprint = Some(visible_fingerprint);
        true
    }

    pub(crate) fn record_draw_result(&mut self, result: Result<(), String>) {
        self.draw_count += 1;
        match result {
            Ok(()) => {
                self.last_drawn_fingerprint = self.pending_fingerprint.take();
                self.error = None;
            }
            Err(error) => {
                self.pending_fingerprint = None;
                self.error = Some(error);
            }
        }
    }

    pub(crate) fn draw_count(&self) -> u64 {
        self.draw_count
    }

    #[cfg(test)]
    pub(crate) fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }
}

#[derive(Resource, Debug, Default)]
pub(crate) struct GameplayInputQueue {
    pending: VecDeque<UiCommand>,
    overflow_visible: bool,
}

impl GameplayInputQueue {
    pub(crate) fn enqueue(&mut self, command: UiCommand, _action_locked: bool) {
        if self.pending.len() < INPUT_QUEUE_CAPACITY {
            self.pending.push_back(command);
        } else {
            self.overflow_visible = true;
        }
    }

    pub(crate) fn pop_front(&mut self) -> Option<UiCommand> {
        self.pending.pop_front()
    }

    pub(crate) fn clear(&mut self) {
        self.pending.clear();
        self.overflow_visible = false;
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.pending.len()
    }

    #[cfg(test)]
    pub(crate) fn overflow_visible(&self) -> bool {
        self.overflow_visible
    }

    pub(crate) fn take_overflow_warning(&mut self) -> bool {
        std::mem::take(&mut self.overflow_visible)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_unchanged_ui_does_not_draw_again() {
        let mut invalidation = RenderInvalidation::default();

        assert!(invalidation.needs_draw(7), "first frame must draw");
        invalidation.record_draw_result(Ok(()));
        assert!(
            !invalidation.needs_draw(7),
            "an unchanged visible fingerprint must remain idle"
        );
        assert_eq!(invalidation.draw_count(), 1);
    }

    #[test]
    fn each_visible_change_draws_exactly_once() {
        let mut invalidation = RenderInvalidation::default();

        assert!(invalidation.needs_draw(7));
        invalidation.record_draw_result(Ok(()));
        assert!(invalidation.needs_draw(8));
        invalidation.record_draw_result(Ok(()));

        assert!(!invalidation.needs_draw(8));
        assert_eq!(invalidation.draw_count(), 2);
    }

    #[test]
    fn render_failure_is_observable() {
        let mut invalidation = RenderInvalidation::default();

        assert!(invalidation.needs_draw(7));
        invalidation.record_draw_result(Err("terminal disconnected".into()));

        assert_eq!(invalidation.error(), Some("terminal disconnected"));
        assert!(
            invalidation.needs_draw(7),
            "failed draws must not clear the dirty frame"
        );
    }

    #[test]
    fn buffered_gameplay_commands_resolve_in_order() {
        let mut queue = GameplayInputQueue::default();
        for command in [UiCommand::MoveEast, UiCommand::Wait, UiCommand::MoveNorth] {
            queue.enqueue(command, true);
        }

        assert_eq!(queue.pop_front(), Some(UiCommand::MoveEast));
        assert_eq!(queue.pop_front(), Some(UiCommand::Wait));
        assert_eq!(queue.pop_front(), Some(UiCommand::MoveNorth));
    }

    #[test]
    fn buffered_command_overflow_is_visible_and_bounded() {
        let mut queue = GameplayInputQueue::default();
        for _ in 0..=INPUT_QUEUE_CAPACITY {
            queue.enqueue(UiCommand::Wait, true);
        }

        assert_eq!(queue.len(), INPUT_QUEUE_CAPACITY);
        assert!(
            queue.overflow_visible(),
            "the first rejected buffered command must be visible"
        );
        assert!(queue.take_overflow_warning());
        assert!(!queue.take_overflow_warning());
    }
}
