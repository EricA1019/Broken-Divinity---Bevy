use broken_divinity::objective_prompt::{
    InstructionEvent, InstructionPriority, ObjectivePromptEngine, ObjectivePromptPolicy,
};

#[test]
fn active_objective_always_selects_primary() {
    let policy = ObjectivePromptPolicy::default();
    let mut engine = ObjectivePromptEngine::new(policy);

    let event = engine.next(true, true, 10);

    assert_eq!(event.priority, InstructionPriority::Primary);
}

#[test]
fn non_critical_duplicates_are_suppressed() {
    let policy = ObjectivePromptPolicy::default();
    let mut engine = ObjectivePromptEngine::new(policy);

    let first = engine.next(false, true, 20);
    assert_eq!(first.kind, InstructionEvent::SecondaryShown);

    let second = engine.next(false, true, 20);
    assert_eq!(second.kind, InstructionEvent::SuppressedDuplicate);
}

#[test]
fn primary_persists_until_transition_success() {
    let policy = ObjectivePromptPolicy::default();
    let mut engine = ObjectivePromptEngine::new(policy);

    let before = engine.next(true, true, 30);
    assert_eq!(before.kind, InstructionEvent::PrimaryShown);

    engine.mark_transition_success();
    let after = engine.next(false, true, 30);

    assert_eq!(after.priority, InstructionPriority::Secondary);
}
