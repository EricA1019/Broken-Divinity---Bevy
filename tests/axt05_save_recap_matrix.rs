use broken_divinity::save_recap::{
    RecapRisk,
    SaveRecapState,
    legacy_recap,
    recap_for_state,
};

#[test]
fn recap_for_colony_has_low_risk_and_next_step() {
    let recap = recap_for_state(SaveRecapState::Colony);

    assert_eq!(recap.risk, RecapRisk::Low);
    assert!(!recap.next_step.is_empty());
}

#[test]
fn recap_for_overworld_has_medium_risk_and_next_step() {
    let recap = recap_for_state(SaveRecapState::Overworld);

    assert_eq!(recap.risk, RecapRisk::Medium);
    assert!(!recap.next_step.is_empty());
}

#[test]
fn recap_for_dungeon_has_high_risk_and_next_step() {
    let recap = recap_for_state(SaveRecapState::Dungeon);

    assert_eq!(recap.risk, RecapRisk::High);
    assert!(!recap.next_step.is_empty());
}

#[test]
fn legacy_save_state_maps_to_stable_default_recap() {
    let recap = legacy_recap("unknown_state_v0");

    assert_eq!(recap.state, SaveRecapState::Colony);
    assert_eq!(recap.risk, RecapRisk::Low);
}
