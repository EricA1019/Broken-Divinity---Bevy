use broken_divinity::alpha_battery::{
    Defect,
    DefectSeverity,
    triage_gate_passes,
};

#[test]
fn triage_gate_fails_with_open_p0() {
    let defects = vec![Defect::open(DefectSeverity::P0)];
    assert!(!triage_gate_passes(&defects));
}

#[test]
fn triage_gate_fails_with_open_p1() {
    let defects = vec![Defect::open(DefectSeverity::P1)];
    assert!(!triage_gate_passes(&defects));
}

#[test]
fn triage_gate_passes_with_only_closed_or_low_severity() {
    let defects = vec![
        Defect::closed(DefectSeverity::P0),
        Defect::open(DefectSeverity::P2),
        Defect::open(DefectSeverity::P3),
    ];

    assert!(triage_gate_passes(&defects));
}
