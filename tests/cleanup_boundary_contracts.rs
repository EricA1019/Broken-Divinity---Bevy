const BASELINE_DOC: &str = include_str!("../docs/tech/PHASE-0-CLEANUP-BASELINE-2026-05-29.md");
const ROOT_GITIGNORE: &str = include_str!("../.gitignore");

const DEV_WALKTHROUGH_COMMAND: &str = "cargo run --bin broken_divinity --features dev";
const ROLLBACK_WALKTHROUGH_COMMAND: &str = "cargo run --bin broken_divinity";
const SAVE_BASELINE_MARKER: &str = "broken_divinity_save.json";
const BUILD_OUTPUT_IGNORE: &str = "/target/";
const TRANSIENT_LOG_IGNORE: &str = "*.log";

#[test]
fn baseline_doc_locks_keep_remove_boundary_and_runtime_commands() {
    for expected in [
        "## Keep",
        "## Remove",
        "## Ignore Policy Status",
        DEV_WALKTHROUGH_COMMAND,
        ROLLBACK_WALKTHROUGH_COMMAND,
        SAVE_BASELINE_MARKER,
    ] {
        assert!(
            BASELINE_DOC.contains(expected),
            "baseline doc must include: {expected}"
        );
    }
}

#[test]
fn root_gitignore_covers_disposable_build_and_log_outputs() {
    for expected in [BUILD_OUTPUT_IGNORE, TRANSIENT_LOG_IGNORE] {
        assert!(
            ROOT_GITIGNORE.contains(expected),
            "root .gitignore must include disposable artifact rule: {expected}"
        );
    }
}

#[test]
fn baseline_doc_preserves_tracked_artifacts_and_save_baseline() {
    for expected in [
        "playtest artifacts",
        "tracked docs",
        "prototype binaries",
        SAVE_BASELINE_MARKER,
    ] {
        assert!(
            BASELINE_DOC.contains(expected),
            "baseline doc must preserve tracked boundary: {expected}"
        );
    }
}