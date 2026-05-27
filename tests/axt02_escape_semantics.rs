use broken_divinity::escape::{
    EscapeAction,
    EscapeContext,
    EscapeGuidanceEngine,
    EscapeGuidanceEvent,
    resolve_escape_action,
};

#[test]
fn one_shot_guidance_appears_once_per_context() {
    let mut engine = EscapeGuidanceEngine::new();

    let first = engine.guidance(EscapeContext { modal_open: true, can_pause: true });
    assert_eq!(first, EscapeGuidanceEvent::ShowHint);

    let second = engine.guidance(EscapeContext { modal_open: true, can_pause: true });
    assert_eq!(second, EscapeGuidanceEvent::Suppressed);
}

#[test]
fn acknowledgement_suppresses_future_hints() {
    let mut engine = EscapeGuidanceEngine::new();

    let _ = engine.guidance(EscapeContext { modal_open: false, can_pause: true });
    engine.acknowledge();

    let after_ack = engine.guidance(EscapeContext { modal_open: false, can_pause: true });
    assert_eq!(after_ack, EscapeGuidanceEvent::Suppressed);
}

#[test]
fn modal_exclusivity_still_closes_modal_first() {
    let action = resolve_escape_action(EscapeContext { modal_open: true, can_pause: true });
    assert_eq!(action, EscapeAction::CloseModal);
}
