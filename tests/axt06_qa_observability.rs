use broken_divinity::qa_profile::{
    DiagnosticsProfile,
    LogLevel,
    QaProfile,
};

#[test]
fn standard_profile_reduces_noise() {
    let profile = QaProfile::for_mode(DiagnosticsProfile::Standard);

    assert_eq!(profile.enable_debug_trace, false);
    assert_eq!(profile.min_visible_level, LogLevel::Warning);
}

#[test]
fn deep_profile_enables_debug_trace() {
    let profile = QaProfile::for_mode(DiagnosticsProfile::Deep);

    assert_eq!(profile.enable_debug_trace, true);
    assert_eq!(profile.min_visible_level, LogLevel::Info);
}

#[test]
fn error_level_is_always_visible() {
    let standard = QaProfile::for_mode(DiagnosticsProfile::Standard);
    let deep = QaProfile::for_mode(DiagnosticsProfile::Deep);

    assert!(standard.is_visible(LogLevel::Error));
    assert!(deep.is_visible(LogLevel::Error));
}
