use std::{fs, path::PathBuf};

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate directory has a workspace parent")
        .parent()
        .expect("workspace has a project parent")
        .to_path_buf()
}

fn read(relative: &str) -> String {
    let path = project_root().join(relative);
    fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "required repository file {} is unreadable: {error}",
            path.display()
        )
    })
}

#[test]
fn release_smoke_propagates_launch_failures_and_uses_a_real_pty() {
    let script = read("scripts/release-smoke.sh");

    assert!(
        script.contains("script -qec"),
        "release smoke must launch the TUI through a real pseudo-terminal"
    );
    assert!(
        !script.lines().any(|line| {
            line.contains("target/release/bd")
                && (line.contains("|| true") || line.contains("|| :"))
        }),
        "release smoke must not suppress a failed or timed-out application launch"
    );
}

#[test]
fn development_gate_measures_listed_tests_independently() {
    let script = read("scripts/test-gate.sh");

    assert!(
        script.contains("cargo test --workspace --locked -- --list"),
        "listed-test totals must come from Cargo's list output"
    );
    assert!(
        !script.contains("TEST_LISTED=$((TEST_PASSED + TEST_FAILED + TEST_IGNORED))"),
        "the gate must not relabel executed outcomes as the listed-test count"
    );
}

#[test]
fn development_gate_separates_candidate_and_reviewer_authority() {
    let script = read("scripts/test-gate.sh");

    assert!(
        script.contains("--candidate-manifest")
            && script.contains("--manifest-sha256")
            && script.contains("handoff_guard"),
        "candidate mode must require a signed protected-file handoff manifest"
    );
    assert!(
        script.contains("--candidate-contract"),
        "candidate mode must report the exact contracts that remain author-owned Red"
    );
    for protected in [
        "AGENTS.md",
        "GDD.md",
        "Kernel.md",
        "docs/DECISIONS-TO-LOCK.md",
        "Cargo.toml",
        "scripts/test-gate.sh",
        "testing/allowed-ignored-tests.txt",
        "testing/foundation-contracts.ron",
        "testing/FOUNDATION-TEST-EVIDENCE.md",
        "testing/FOUNDATION-REQUIREMENT-MAP.md",
        "testing/VISUAL-ACCEPTANCE-MATRIX.md",
        "docs/AUTHORITATIVE-TESTING-STANDARD-AND-MIGRATION-PLAN.md",
        "crates/bd_test_support/Cargo.toml",
        "crates/bd_test_support/src/lib.rs",
        "crates/bd_test_support/src/contract_registry.rs",
        "crates/bd_test_support/src/bin/handoff_guard.rs",
        "crates/bd_test_support/src/bin/contract_report.rs",
        "crates/bd_test_support/tests/candidate_handoff.rs",
        "crates/bd_test_support/tests/contract_registry.rs",
        "crates/bd_test_support/tests/repository_governance.rs",
    ] {
        assert!(
            script.contains(&format!("--require-protected {protected}")),
            "candidate mode must require the handoff manifest to protect {protected}"
        );
    }
    assert!(
        script.contains("STATUS=CandidateGreen"),
        "implementation-agent validation must not claim VerifiedGreen"
    );
}

#[test]
fn continuous_integration_runs_the_canonical_gate() {
    let workflow = read(".github/workflows/ci.yml");

    assert!(
        workflow.contains("bash scripts/test-gate.sh"),
        "CI must execute the same canonical gate used locally"
    );
}

#[test]
fn managed_launcher_builds_the_current_workspace_instead_of_running_a_stale_binary() {
    let launcher = read("scripts/bd");
    let installer = read("scripts/install-bd-launcher.sh");
    let update_notice = launcher
        .find("Updating Broken Divinity from the current workspace before launch")
        .expect("the bd launcher must visibly report that it is updating before launch");
    let cargo_run = launcher
        .find("cargo run --quiet -p bd_app")
        .expect("the bd launcher must resolve and build the current workspace application");

    assert!(
        update_notice < cargo_run,
        "the bd launcher must report the workspace update before it builds and launches"
    );
    assert!(
        !launcher.contains("target/debug/bd") && !launcher.contains("target/release/bd"),
        "the managed launcher must not pin a potentially stale build artifact"
    );
    assert!(
        installer.contains("--check"),
        "the installer must retain a non-mutating stale-launcher check"
    );
}

#[test]
fn action_harness_keeps_result_and_enemy_frames_explicit() {
    let support = read("crates/bd_test_support/src/lib.rs");
    let retired_helper = ["pub fn expect_", "action("].concat();
    let retired_calls = [".expect_", "action("].concat();

    assert!(
        !support.contains(&retired_helper) && !support.contains(&retired_calls),
        "the retired action helper must not reintroduce unconditional two-frame settling"
    );
    assert!(
        support.contains("pub fn submit_action_and_advance_result_frame("),
        "accepted actions need an API that names its single result-frame update"
    );
    assert!(
        support.contains("pub fn advance_enemy_phase_frame("),
        "Tactical workflows need a separate, explicit enemy-phase frame"
    );
}

#[test]
fn canonical_document_links_resolve_inside_the_repository() {
    const CANONICAL_DOCUMENTS: &[&str] = &[
        "AGENTS.md",
        "README.md",
        "GDD.md",
        "Kernel.md",
        "Kernel-direction.md",
        "docs/README.md",
        "docs/DECISIONS-TO-LOCK.md",
        "docs/DOCUMENT-INVENTORY.md",
        "docs/MIGRATION-AND-DEPRECATION.md",
        "docs/MVP-SCENARIO.md",
        "docs/AUTHORITATIVE-TESTING-STANDARD-AND-MIGRATION-PLAN.md",
        "docs/FOUNDATION-TEST-AND-UX-HARDENING-PLAN.md",
        "testing/FOUNDATION-REQUIREMENT-MAP.md",
        "testing/FOUNDATION-TEST-EVIDENCE.md",
        "testing/VISUAL-ACCEPTANCE-MATRIX.md",
    ];
    let root = project_root();
    let mut broken = Vec::new();

    for relative in CANONICAL_DOCUMENTS {
        let document = root.join(relative);
        let source = read(relative);
        for target in markdown_link_targets(&source) {
            if target.starts_with('#')
                || target.starts_with("http://")
                || target.starts_with("https://")
                || target.starts_with("mailto:")
            {
                continue;
            }
            let path = target
                .split_once('#')
                .map_or(target.as_str(), |(path, _)| path)
                .trim_matches(['<', '>']);
            if path.is_empty() {
                continue;
            }
            let resolved = document
                .parent()
                .expect("canonical document has a parent")
                .join(path);
            if !resolved.exists() || !resolved.starts_with(&root) {
                broken.push(format!("{relative} -> {target}"));
            }
        }
    }

    assert!(
        broken.is_empty(),
        "canonical documents contain broken or external local links:\n{}",
        broken.join("\n")
    );
}

fn markdown_link_targets(source: &str) -> Vec<String> {
    let mut targets = Vec::new();
    let mut remainder = source;

    while let Some((_, after_open)) = remainder.split_once("](") {
        let Some((target, after_close)) = after_open.split_once(')') else {
            break;
        };
        targets.push(target.to_owned());
        remainder = after_close;
    }
    targets
}
