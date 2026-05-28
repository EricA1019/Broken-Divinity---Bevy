use broken_divinity::alpha_signoff::{SignoffDecision, SignoffInput, evaluate_signoff};

#[test]
fn signoff_rejects_when_triage_gate_incomplete() {
    let input = SignoffInput {
        triage_gate_complete: false,
        metrics_met: true,
        full_gate_green: true,
    };

    assert_eq!(evaluate_signoff(input), SignoffDecision::Rejected);
}

#[test]
fn signoff_rejects_when_metrics_missing() {
    let input = SignoffInput {
        triage_gate_complete: true,
        metrics_met: false,
        full_gate_green: true,
    };

    assert_eq!(evaluate_signoff(input), SignoffDecision::Rejected);
}

#[test]
fn signoff_accepts_when_all_blockers_are_green() {
    let input = SignoffInput {
        triage_gate_complete: true,
        metrics_met: true,
        full_gate_green: true,
    };

    assert_eq!(evaluate_signoff(input), SignoffDecision::Accepted);
}
