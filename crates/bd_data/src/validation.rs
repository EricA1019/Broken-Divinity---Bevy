//! Validation report for content loading.
//!
//! Collects errors and warnings during content validation.
//! Errors are sorted deterministically for snapshot testing.

use crate::id::ContentId;

/// Severity of a validation issue.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Error,
    Warning,
}

/// A single validation error or warning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub source_file: Option<String>,
    pub content_id: Option<ContentId>,
    pub message: String,
    pub severity: Severity,
}

/// Collects validation results with deterministic ordering.
#[derive(Debug, Clone, Default)]
pub struct ValidationReport {
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<String>,
}

impl ValidationReport {
    pub fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    pub fn add_warning(&mut self, msg: impl Into<String>) {
        self.warnings.push(msg.into());
    }

    /// Sort errors deterministically by (severity, source_file, content_id, message).
    pub fn sort(&mut self) {
        self.errors.sort_by(|a, b| {
            a.severity
                .cmp(&b.severity)
                .then_with(|| a.source_file.cmp(&b.source_file))
                .then_with(|| a.content_id.cmp(&b.content_id))
                .then_with(|| a.message.cmp(&b.message))
        });
    }

    pub fn has_errors(&self) -> bool {
        self.errors.iter().any(|e| e.severity == Severity::Error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_report_sorts_errors_deterministically() {
        let mut report = ValidationReport::default();
        report.add_error(ValidationError {
            source_file: Some("b.ron".into()),
            content_id: Some(ContentId::new("test", "z")),
            message: "second".into(),
            severity: Severity::Error,
        });
        report.add_error(ValidationError {
            source_file: Some("a.ron".into()),
            content_id: Some(ContentId::new("test", "a")),
            message: "first".into(),
            severity: Severity::Error,
        });
        report.sort();

        assert!(report.errors[0].message == "first");
        assert!(report.errors[1].message == "second");
    }

    #[test]
    fn report_detects_errors() {
        let mut report = ValidationReport::default();
        assert!(!report.has_errors());
        report.add_error(ValidationError {
            source_file: None,
            content_id: None,
            message: "test error".into(),
            severity: Severity::Error,
        });
        assert!(report.has_errors());
    }
}
