const DEV_MANUAL_COMMAND: &str = "cargo run --bin broken_divinity --features dev";
const PLAN_VALIDATION_LINE: &str = "Validate against the plan before each phase starts and again at the phase gate.";

#[test]
fn phase_sheet_contains_phase_zero_through_four_contracts() {
    let phase_sheet = include_str!("../PHASE-SHEET-2026-05-29.md");

    for expected in [
        "## Phase 0 - Baseline inventory and policy lock",
        "## Phase 1 - Test scaffolding for the first polish slice",
        "## Phase 2 - Build artifact cleanup and ignore hygiene",
        "## Phase 3 - Launch, menu, and help surfaces",
        "## Phase 4 - Save/load and quit flow",
    ] {
        assert!(
            phase_sheet.contains(expected),
            "phase sheet must include contract heading: {expected}"
        );
    }
}

#[test]
fn phase_sheet_documents_plan_validation_and_dev_walkthrough_command() {
    let phase_sheet = include_str!("../PHASE-SHEET-2026-05-29.md");

    assert!(
        phase_sheet.contains(PLAN_VALIDATION_LINE),
        "phase sheet must require validation against the plan"
    );
    assert!(
        phase_sheet.contains(DEV_MANUAL_COMMAND),
        "phase sheet must name the dev manual walkthrough command"
    );
}

#[test]
fn phase_sheet_preserves_save_baseline_boundary() {
    let phase_sheet = include_str!("../PHASE-SHEET-2026-05-29.md");

    assert!(
        phase_sheet.contains("Preserve `broken_divinity_save.json` as a persistence baseline"),
        "phase sheet must protect the save baseline from cleanup"
    );
}