const DEFAULT_FEEDBACK_COOLDOWN_TICKS: u32 = 3;

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogColor {
    System,
    Status,
    EnemyHit,
    PlayerHit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameLogEntry {
    pub text: String,
    pub color: LogColor,
    pub turn: u32,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct GameLog {
    entries: Vec<GameLogEntry>,
}

impl GameLog {
    pub fn push(&mut self, text: impl Into<String>, color: LogColor, turn: u32) {
        self.entries.push(GameLogEntry {
            text: text.into(),
            color,
            turn,
        });
    }

    pub fn entries(&self) -> &[GameLogEntry] {
        &self.entries
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct FeedbackPolicy;

impl FeedbackPolicy {
    pub fn blocked_action(
        &self,
        what_failed: &str,
        why: &str,
        next: &str,
    ) -> String {
        blocked_action_message(what_failed, why, next)
    }

    pub fn blocked_action_severity(&self) -> LogSeverity {
        LogSeverity::default_blocked_action()
    }

    pub fn cooldown_ticks(&self) -> u32 {
        default_feedback_cooldown_ticks()
    }
}

impl LogSeverity {
    pub fn default_blocked_action() -> Self {
        Self::Warning
    }
}

pub fn default_feedback_cooldown_ticks() -> u32 {
    DEFAULT_FEEDBACK_COOLDOWN_TICKS
}

pub fn blocked_action_message(what_failed: &str, why: &str, next: &str) -> String {
    format!(
        "What failed: {what_failed}\nWhy: {why}\nNext: {next}",
        what_failed = what_failed,
        why = why,
        next = next,
    )
}
