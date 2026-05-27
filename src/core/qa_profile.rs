#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticsProfile {
    Standard,
    Deep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QaProfile {
    pub min_visible_level: LogLevel,
    pub enable_debug_trace: bool,
}

impl QaProfile {
    pub fn for_mode(mode: DiagnosticsProfile) -> Self {
        match mode {
            DiagnosticsProfile::Standard => Self {
                min_visible_level: LogLevel::Warning,
                enable_debug_trace: false,
            },
            DiagnosticsProfile::Deep => Self {
                min_visible_level: LogLevel::Info,
                enable_debug_trace: true,
            },
        }
    }

    pub fn is_visible(&self, level: LogLevel) -> bool {
        level >= self.min_visible_level || level == LogLevel::Error
    }
}
