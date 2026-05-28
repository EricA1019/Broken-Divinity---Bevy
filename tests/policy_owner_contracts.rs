use broken_divinity::escape::{EscapeAction, EscapeContext, resolve_escape_action};
use broken_divinity::gamelog::{
    LogSeverity, blocked_action_message, default_feedback_cooldown_ticks,
};
use broken_divinity::modal_priority::{ModalKind, ModalPriorityDecision, resolve_modal_priority};
use broken_divinity::objective_prompt::{
    InstructionPriority, ObjectivePromptPolicy, select_primary_instruction,
};

#[test]
fn objective_prompt_policy_prioritizes_primary() {
    let policy = ObjectivePromptPolicy::new();
    let selected = select_primary_instruction(&policy, true, true);

    assert_eq!(selected, InstructionPriority::Primary);
}

#[test]
fn objective_prompt_policy_falls_back_when_objective_inactive() {
    let policy = ObjectivePromptPolicy::new();
    let selected = select_primary_instruction(&policy, false, true);

    assert_eq!(selected, InstructionPriority::Secondary);
}

#[test]
fn modal_priority_prefers_modal_over_gameplay() {
    let decision = resolve_modal_priority(true, ModalKind::Inventory);

    assert_eq!(decision, ModalPriorityDecision::ModalFirst);
}

#[test]
fn escape_policy_uses_modal_close_when_modal_is_open() {
    let action = resolve_escape_action(EscapeContext {
        modal_open: true,
        can_pause: true,
    });

    assert_eq!(action, EscapeAction::CloseModal);
}

#[test]
fn blocked_action_message_has_required_structure() {
    let message = blocked_action_message("Out of range", "Target too far", "Move closer");

    assert!(message.contains("What failed:"));
    assert!(message.contains("Why:"));
    assert!(message.contains("Next:"));
}

#[test]
fn feedback_cooldown_is_positive() {
    let cooldown_ticks = default_feedback_cooldown_ticks();

    assert!(cooldown_ticks > 0);
}

#[test]
fn blocked_action_severity_defaults_to_warning() {
    let severity = LogSeverity::default_blocked_action();

    assert_eq!(severity, LogSeverity::Warning);
}
