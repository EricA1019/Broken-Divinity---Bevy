//! Signal trace — debug-only ordered log of every signal through the pipeline.
//!
//! Enabled in debug builds. Each important system appends a trace entry.
//! Used for debugging and architecture tests.

use bevy_ecs::prelude::*;

/// Ordered log of signals flowing through the schedule.
#[derive(Resource, Debug, Clone, Default)]
pub struct SignalTrace {
    pub entries: Vec<TraceEntry>,
    seq: u64,
}

impl SignalTrace {
    /// Push a new trace entry. Returns the sequence number.
    pub fn push(&mut self, stage: &'static str, signal_type: &'static str, summary: String) {
        let entry = TraceEntry {
            seq: self.seq,
            stage,
            signal_type,
            summary,
        };
        self.entries.push(entry);
        self.seq += 1;
    }
}

/// A single entry in the signal trace.
#[derive(Debug, Clone)]
pub struct TraceEntry {
    pub seq: u64,
    pub stage: &'static str,
    pub signal_type: &'static str,
    pub summary: String,
}

/// Guard against recursive trigger chains.
/// Placeholder — full trigger system arrives in Phase 9.
#[derive(Resource, Debug, Clone)]
pub struct TriggerExecutionGuard {
    pub current_depth: u32,
    pub max_depth: u32,
}

impl Default for TriggerExecutionGuard {
    fn default() -> Self {
        Self {
            current_depth: 0,
            max_depth: 10,
        }
    }
}

impl TriggerExecutionGuard {
    /// Attempt to enter a trigger level. Returns false if max depth exceeded.
    pub fn enter(&mut self) -> bool {
        if self.current_depth >= self.max_depth {
            return false;
        }
        self.current_depth += 1;
        true
    }

    /// Exit a trigger level.
    pub fn exit(&mut self) {
        self.current_depth = self.current_depth.saturating_sub(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trace_entries_are_ordered() {
        let mut trace = SignalTrace::default();
        trace.push("Input", "ActionIntent", "move east".into());
        trace.push("Validation", "ActionValidated", "ok".into());
        trace.push("CostResolution", "PoolDeltaRequested", "AP -1".into());

        assert_eq!(trace.entries[0].seq, 0);
        assert_eq!(trace.entries[1].seq, 1);
        assert_eq!(trace.entries[2].seq, 2);
        assert_eq!(trace.entries[0].stage, "Input");
        assert_eq!(trace.entries[2].stage, "CostResolution");
    }

    #[test]
    fn trigger_depth_guard_rejects_overflow() {
        let mut guard = TriggerExecutionGuard {
            current_depth: 0,
            max_depth: 3,
        };
        assert!(guard.enter()); // depth 1
        assert!(guard.enter()); // depth 2
        assert!(guard.enter()); // depth 3
        assert!(!guard.enter()); // rejected — at max
        guard.exit(); // depth 2
        assert!(guard.enter()); // depth 3 again — ok
    }
}
