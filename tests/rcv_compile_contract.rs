#[test]
fn crate_recovery_marker_is_present() {
    assert_eq!(broken_divinity::recovery::crate_recovery_marker(), "rcv-02");
}
