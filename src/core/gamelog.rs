//! Game log — ring buffer of combat and game messages.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const DEFAULT_FEEDBACK_COOLDOWN_TICKS: u32 = 3;
const TRAVEL_WARNING_COOLDOWN_TURNS: u32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogSeverity {
    Info,
    Warning,
    Error,
}

/// Single log entry.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct LogEntry {
    pub text: String,
    pub color: LogColor,
    pub turn: u32,
    #[serde(default = "default_log_count")]
    pub count: u32,
}

const fn default_log_count() -> u32 {
    1
}

/// Semantic log colors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum LogColor {
    Default,
    PlayerHit,
    EnemyHit,
    Critical,
    Miss,
    Death,
    Status,
    System,
}

/// Canonical player-facing UX message keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UxMessage {
    ColonyGateEnterHint,
    EscOverworldBackHint,
    EscHelpCloseHint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
enum FeedbackEventKind {
    ColonyGateEnterHint,
    EscOverworldBackHint,
    EscHelpCloseHint,
    TravelNoFood,
    TravelNoWater,
    RaidForecast,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedbackEvent {
    ColonyGateEnterHint,
    EscOverworldBackHint,
    EscHelpCloseHint,
    TravelNoFood,
    TravelNoWater,
    RaidForecast { message: &'static str },
}

impl FeedbackEvent {
    fn kind(self) -> FeedbackEventKind {
        match self {
            Self::ColonyGateEnterHint => FeedbackEventKind::ColonyGateEnterHint,
            Self::EscOverworldBackHint => FeedbackEventKind::EscOverworldBackHint,
            Self::EscHelpCloseHint => FeedbackEventKind::EscHelpCloseHint,
            Self::TravelNoFood => FeedbackEventKind::TravelNoFood,
            Self::TravelNoWater => FeedbackEventKind::TravelNoWater,
            Self::RaidForecast { .. } => FeedbackEventKind::RaidForecast,
        }
    }

    fn text(self) -> &'static str {
        match self {
            Self::ColonyGateEnterHint => {
                "You can only leave for the overworld from the shelter gate (stairs)."
            }
            Self::EscOverworldBackHint => "Esc returns you to shelter from the overworld.",
            Self::EscHelpCloseHint => "Esc closed Help. Press Esc again to back out.",
            Self::TravelNoFood => "No food for travel! Starving.",
            Self::TravelNoWater => "No water for travel! Dehydrating.",
            Self::RaidForecast { message } => message,
        }
    }

    fn severity(self) -> LogColor {
        match self {
            Self::ColonyGateEnterHint | Self::EscOverworldBackHint | Self::EscHelpCloseHint => {
                LogColor::Status
            }
            Self::TravelNoFood | Self::TravelNoWater | Self::RaidForecast { .. } => {
                LogColor::EnemyHit
            }
        }
    }

    fn cooldown_turns(self) -> u32 {
        match self {
            Self::TravelNoFood | Self::TravelNoWater => TRAVEL_WARNING_COOLDOWN_TURNS,
            Self::EscOverworldBackHint | Self::EscHelpCloseHint => u32::MAX,
            Self::ColonyGateEnterHint | Self::RaidForecast { .. } => 0,
        }
    }
}

impl UxMessage {
    fn as_feedback(self) -> FeedbackEvent {
        match self {
            Self::ColonyGateEnterHint => FeedbackEvent::ColonyGateEnterHint,
            Self::EscOverworldBackHint => FeedbackEvent::EscOverworldBackHint,
            Self::EscHelpCloseHint => FeedbackEvent::EscHelpCloseHint,
        }
    }
}

impl LogColor {
    pub fn to_color32(&self) -> bevy::color::Color {
        match self {
            Self::Default => Color::WHITE,
            Self::PlayerHit => Color::srgb(0.2, 0.8, 0.2),
            Self::EnemyHit => Color::srgb(0.9, 0.3, 0.3),
            Self::Critical => Color::srgb(1.0, 0.84, 0.0),
            Self::Miss => Color::srgb(0.6, 0.6, 0.6),
            Self::Death => Color::srgb(0.8, 0.1, 0.1),
            Self::Status => Color::srgb(0.6, 0.4, 0.8),
            Self::System => Color::srgb(0.5, 0.7, 1.0),
        }
    }
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Resource)]
pub struct GameLog {
    entries: Vec<LogEntry>,
    max_entries: usize,
    #[serde(default)]
    last_feedback_turns: HashMap<FeedbackEventKind, u32>,
}

impl Default for GameLog {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            max_entries: 100,
            last_feedback_turns: HashMap::new(),
        }
    }
}

impl GameLog {
    pub fn push(&mut self, text: impl Into<String>, color: LogColor, turn: u32) {
        let text = text.into();
        if let Some(last) = self.entries.last_mut()
            && last.text == text
            && last.color == color
            && last.turn == turn
        {
            last.count = last.count.saturating_add(1);
            return;
        }

        if self.entries.len() >= self.max_entries {
            self.entries.remove(0);
        }
        self.entries.push(LogEntry {
            text,
            color,
            turn,
            count: 1,
        });
    }

    pub fn last_n(&self, n: usize) -> &[LogEntry] {
        let start = self.entries.len().saturating_sub(n);
        &self.entries[start..]
    }

    pub fn entries(&self) -> &[LogEntry] {
        &self.entries
    }

    pub fn push_ux_message(&mut self, message: UxMessage, turn: u32) {
        self.push_feedback(message.as_feedback(), turn);
    }

    pub fn push_feedback(&mut self, event: FeedbackEvent, turn: u32) {
        let kind = event.kind();
        if !self.should_emit_feedback(kind, turn, event.cooldown_turns()) {
            return;
        }

        self.push(event.text(), event.severity(), turn);
        self.last_feedback_turns.insert(kind, turn);
    }

    fn should_emit_feedback(
        &self,
        kind: FeedbackEventKind,
        current_turn: u32,
        cooldown_turns: u32,
    ) -> bool {
        if cooldown_turns == 0 {
            return true;
        }

        let Some(last_turn) = self.last_feedback_turns.get(&kind).copied() else {
            return true;
        };

        current_turn.saturating_sub(last_turn) >= cooldown_turns
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.last_feedback_turns.clear();
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct FeedbackPolicy;

impl FeedbackPolicy {
    pub fn blocked_action(&self, what_failed: &str, why: &str, next: &str) -> String {
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
