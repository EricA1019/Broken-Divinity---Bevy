//! Game log — a ring buffer of log entries.

use bevy_ecs::prelude::*;

/// Severity level for log entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Warn,
    Combat,
}

/// A single log entry.
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub message: String,
    pub level: LogLevel,
}

/// Ring buffer of recent log messages, newest first.
#[derive(Resource, Debug, Clone)]
pub struct GameLog {
    entries: Vec<LogEntry>,
    max_entries: usize,
}

impl Default for GameLog {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            max_entries: 50,
        }
    }
}

impl GameLog {
    /// Push a new entry. Truncates to `max_entries`.
    pub fn push(&mut self, message: impl Into<String>, level: LogLevel) {
        self.entries.insert(
            0,
            LogEntry {
                message: message.into(),
                level,
            },
        );
        self.entries.truncate(self.max_entries);
    }

    /// Iterator over entries, newest first.
    pub fn iter(&self) -> impl Iterator<Item = &LogEntry> {
        self.entries.iter()
    }
}
