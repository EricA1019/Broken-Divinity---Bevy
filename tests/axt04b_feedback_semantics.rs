use broken_divinity::gamelog::{
    FeedbackPolicy,
    LogSeverity,
};

#[test]
fn blocked_action_pattern_contains_required_fields() {
    let policy = FeedbackPolicy::default();
    let message = policy.blocked_action("Door locked", "Missing key", "Find key");

    assert!(message.contains("What failed:"));
    assert!(message.contains("Why:"));
    assert!(message.contains("Next:"));
}

#[test]
fn blocked_action_defaults_to_warning_severity() {
    let policy = FeedbackPolicy::default();
    assert_eq!(policy.blocked_action_severity(), LogSeverity::Warning);
}

#[test]
fn cooldown_is_stable_and_positive() {
    let policy = FeedbackPolicy::default();

    let first = policy.cooldown_ticks();
    let second = policy.cooldown_ticks();

    assert!(first > 0);
    assert_eq!(first, second);
}
