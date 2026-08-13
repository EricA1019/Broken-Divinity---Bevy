use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn temp_root(case: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must follow the Unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "bd-candidate-handoff-{}-{case}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("candidate handoff temp root must be creatable");
    git(&root, &["init", "--quiet"]);
    git(
        &root,
        &["config", "user.email", "candidate-handoff@example.invalid"],
    );
    git(&root, &["config", "user.name", "Candidate Handoff Test"]);
    root
}

fn git(root: &Path, arguments: &[&str]) {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(arguments)
        .output()
        .expect("git must be available for candidate handoff tests");
    assert!(
        output.status.success(),
        "git {arguments:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn sha256(path: &Path) -> String {
    let output = Command::new("sha256sum")
        .arg(path)
        .output()
        .expect("sha256sum must be available for the repository gate");
    assert!(output.status.success());
    String::from_utf8(output.stdout)
        .expect("sha256sum output must be UTF-8")
        .split_whitespace()
        .next()
        .expect("sha256sum must print a digest")
        .to_owned()
}

fn write_manifest(root: &Path, protected: &Path) -> (PathBuf, String) {
    git(root, &["add", "--all"]);
    git(root, &["commit", "--quiet", "-m", "sealed test baseline"]);

    let baseline = root.join("candidate-baseline.ron");
    fs::write(
        &baseline,
        "(\n  version: 1,\n  exact_production_write_set: [\"allowed.txt\"],\n  entries: [],\n)\n",
    )
    .expect("candidate baseline must be writable");
    let protected_digest = sha256(protected);
    let baseline_digest = sha256(&baseline);
    let manifest = root.join("candidate-handoff.ron");
    fs::write(
        &manifest,
        format!(
            "(\n  version: 2,\n  contracts: [\"CONTRACT-A\", \"CONTRACT-B\"],\n  baseline_path: \"candidate-baseline.ron\",\n  exact_production_write_set: [\"allowed.txt\"],\n  protected_files: [\n    (path: \"candidate-baseline.ron\", sha256: \"{baseline_digest}\"),\n    (path: \"protected.txt\", sha256: \"{protected_digest}\"),\n  ],\n)\n"
        ),
    )
    .expect("candidate manifest must be writable");
    let manifest_digest = sha256(&manifest);
    (manifest, manifest_digest)
}

fn write_suffix_manifest(root: &Path, protected: &Path) -> (PathBuf, String) {
    git(root, &["add", "--all"]);
    git(root, &["commit", "--quiet", "-m", "sealed suffix baseline"]);

    let baseline = root.join("candidate-baseline.ron");
    fs::write(
        &baseline,
        "(\n  version: 1,\n  exact_production_write_set: [\"allowed.txt\"],\n  entries: [],\n)\n",
    )
    .expect("candidate baseline must be writable");
    let protected_digest = sha256(protected);
    let baseline_digest = sha256(&baseline);
    let suffix_source = fs::read_to_string(root.join("allowed.txt"))
        .expect("protected suffix source must be readable");
    let suffix_start = suffix_source
        .find("#[cfg(test)]")
        .expect("fixture must contain the protected marker");
    let suffix_fixture = root.join("suffix-digest.txt");
    fs::write(&suffix_fixture, &suffix_source[suffix_start..])
        .expect("suffix digest fixture must be writable");
    let suffix_digest = sha256(&suffix_fixture);
    fs::remove_file(suffix_fixture).expect("suffix digest fixture must be removable");
    let manifest = root.join("candidate-handoff.ron");
    fs::write(
        &manifest,
        format!(
            "(\n  version: 2,\n  contracts: [\"CONTRACT-A\"],\n  baseline_path: \"candidate-baseline.ron\",\n  exact_production_write_set: [\"allowed.txt\"],\n  protected_files: [\n    (path: \"candidate-baseline.ron\", sha256: \"{baseline_digest}\"),\n    (path: \"protected.txt\", sha256: \"{protected_digest}\"),\n  ],\n  protected_suffixes: [\n    (path: \"allowed.txt\", start_marker: \"#[cfg(test)]\", sha256: \"{suffix_digest}\"),\n  ],\n)\n"
        ),
    )
    .expect("candidate suffix manifest must be writable");
    let manifest_digest = sha256(&manifest);
    (manifest, manifest_digest)
}

fn run_guard_with_required(
    root: &Path,
    manifest: &Path,
    manifest_digest: &str,
    required: &[&str],
) -> std::process::Output {
    let guard = std::env::var("CARGO_BIN_EXE_handoff_guard")
        .expect("Cargo must expose the handoff_guard binary to integration tests");
    let mut command = Command::new(guard);
    command.args([
        "--root",
        root.to_str().expect("temp root must be UTF-8"),
        "--manifest",
        manifest.to_str().expect("manifest path must be UTF-8"),
        "--manifest-sha256",
        manifest_digest,
        "--print-contracts",
    ]);
    for path in required {
        command.args(["--require-protected", path]);
    }
    command.output().expect("handoff guard must execute")
}

fn run_guard(root: &Path, manifest: &Path, manifest_digest: &str) -> std::process::Output {
    run_guard_with_required(root, manifest, manifest_digest, &[])
}

#[test]
fn signed_candidate_handoff_accepts_unchanged_protected_files() {
    let root = temp_root("unchanged");
    let protected = root.join("protected.txt");
    fs::write(&protected, "author-owned baseline\n").expect("protected fixture must be writable");
    let (manifest, digest) = write_manifest(&root, &protected);

    let output = run_guard(&root, &manifest, &digest);

    assert!(
        output.status.success(),
        "unchanged signed handoff must pass: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).expect("guard output must be UTF-8"),
        "CONTRACT-A\nCONTRACT-B\n"
    );
}

#[test]
fn signed_candidate_handoff_rejects_a_protected_file_mutation() {
    let root = temp_root("protected-mutation");
    let protected = root.join("protected.txt");
    fs::write(&protected, "author-owned baseline\n").expect("protected fixture must be writable");
    let (manifest, digest) = write_manifest(&root, &protected);
    fs::write(&protected, "implementation-agent rewrite\n")
        .expect("protected mutation fixture must be writable");

    let output = run_guard(&root, &manifest, &digest);

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("protected.txt"),
        "diagnostic must name the mutated protected file: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn signed_candidate_handoff_rejects_manifest_rewriting() {
    let root = temp_root("manifest-mutation");
    let protected = root.join("protected.txt");
    fs::write(&protected, "author-owned baseline\n").expect("protected fixture must be writable");
    let (manifest, digest) = write_manifest(&root, &protected);
    let mut source = fs::read_to_string(&manifest).expect("manifest must remain readable");
    source.push_str("// implementation-agent rewrite\n");
    fs::write(&manifest, source).expect("manifest mutation fixture must be writable");

    let output = run_guard(&root, &manifest, &digest);

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("manifest digest"),
        "diagnostic must identify manifest rewriting: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn signed_candidate_handoff_rejects_an_omitted_required_authority_file() {
    let root = temp_root("omitted-authority");
    let protected = root.join("protected.txt");
    fs::write(&protected, "author-owned baseline\n").expect("protected fixture must be writable");
    let (manifest, digest) = write_manifest(&root, &protected);

    let output = run_guard_with_required(
        &root,
        &manifest,
        &digest,
        &["testing/foundation-contracts.ron"],
    );

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("testing/foundation-contracts.ron"),
        "diagnostic must name the omitted authority file: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn signed_candidate_handoff_rejects_an_out_of_scope_untracked_file() {
    // Governance contract: the sealed dirty-worktree baseline plus exact write
    // set must be executable, not a self-reported checklist. A newly created
    // handoff/report file is still a candidate delta and must be rejected when
    // its exact path was not authorized.
    let root = temp_root("out-of-scope-untracked");
    let protected = root.join("protected.txt");
    fs::write(&protected, "author-owned baseline\n").expect("protected fixture must be writable");
    let (manifest, digest) = write_manifest(&root, &protected);
    fs::write(
        root.join("unauthorized-handoff.md"),
        "candidate-created report\n",
    )
    .expect("out-of-scope fixture must be writable");

    let output = run_guard(&root, &manifest, &digest);

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("unauthorized-handoff.md"),
        "diagnostic must name the unauthorized untracked path: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn signed_candidate_handoff_accepts_an_authorized_write_set_change() {
    let root = temp_root("authorized-write");
    let protected = root.join("protected.txt");
    fs::write(&protected, "author-owned baseline\n").expect("protected fixture must be writable");
    let (manifest, digest) = write_manifest(&root, &protected);
    fs::write(root.join("allowed.txt"), "candidate production change\n")
        .expect("authorized fixture must be writable");

    let output = run_guard(&root, &manifest, &digest);

    assert!(
        output.status.success(),
        "exact write-set change must pass: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn signed_candidate_handoff_rejects_an_out_of_scope_tracked_mutation() {
    let root = temp_root("out-of-scope-tracked");
    let protected = root.join("protected.txt");
    let ordinary = root.join("ordinary.txt");
    fs::write(&protected, "author-owned baseline\n").expect("protected fixture must be writable");
    fs::write(&ordinary, "ordinary baseline\n").expect("ordinary fixture must be writable");
    let (manifest, digest) = write_manifest(&root, &protected);
    fs::write(&ordinary, "candidate mutation\n").expect("mutation fixture must be writable");

    let output = run_guard(&root, &manifest, &digest);

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("ordinary.txt"),
        "diagnostic must name the unauthorized tracked path: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn signed_candidate_handoff_allows_prefix_edits_before_a_protected_test_suffix() {
    let root = temp_root("protected-suffix-prefix");
    let protected = root.join("protected.txt");
    fs::write(&protected, "author-owned baseline\n").expect("protected fixture must be writable");
    fs::write(
        root.join("allowed.txt"),
        "fn production() -> bool { false }\n\n#[cfg(test)]\nmod tests {\n    // reviewer-owned test\n}\n",
    )
    .expect("suffix fixture must be writable");
    let (manifest, digest) = write_suffix_manifest(&root, &protected);
    fs::write(
        root.join("allowed.txt"),
        "fn production() -> bool { true }\n\n#[cfg(test)]\nmod tests {\n    // reviewer-owned test\n}\n",
    )
    .expect("authorized production-prefix mutation must be writable");

    let output = run_guard(&root, &manifest, &digest);

    assert!(
        output.status.success(),
        "production-prefix edit before protected test suffix must pass: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn signed_candidate_handoff_rejects_a_colocated_test_suffix_mutation() {
    let mutations = [
        (
            "body",
            "fn production() -> bool { true }\n\n#[cfg(test)]\nmod tests {\n    // candidate-rewritten test\n}\n",
        ),
        (
            "marker-deleted",
            "fn production() -> bool { true }\n\nmod tests {\n    // reviewer-owned test\n}\n",
        ),
        (
            "marker-duplicated",
            "fn production() -> bool { true }\n\n#[cfg(test)]\nmod tests {\n    // reviewer-owned test\n}\n\n#[cfg(test)]\nmod tests {}\n",
        ),
    ];
    for (case, mutation) in mutations {
        let root = temp_root(&format!("protected-suffix-test-{case}"));
        let protected = root.join("protected.txt");
        fs::write(&protected, "author-owned baseline\n")
            .expect("protected fixture must be writable");
        fs::write(
            root.join("allowed.txt"),
            "fn production() -> bool { false }\n\n#[cfg(test)]\nmod tests {\n    // reviewer-owned test\n}\n",
        )
        .expect("suffix fixture must be writable");
        let (manifest, digest) = write_suffix_manifest(&root, &protected);
        fs::write(root.join("allowed.txt"), mutation)
            .expect("protected suffix mutation fixture must be writable");

        let output = run_guard(&root, &manifest, &digest);

        assert!(!output.status.success(), "case={case}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("allowed.txt"),
            "case={case} diagnostic must identify the co-located protected suffix: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
