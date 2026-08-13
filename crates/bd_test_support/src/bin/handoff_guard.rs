use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    io::Write,
    path::{Component, Path, PathBuf},
    process::{self, Command, Stdio},
};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CandidateHandoffManifest {
    version: u8,
    contracts: Vec<String>,
    baseline_path: PathBuf,
    exact_production_write_set: Vec<PathBuf>,
    protected_files: Vec<ProtectedFile>,
    #[serde(default)]
    protected_suffixes: Vec<ProtectedSuffix>,
    #[serde(default)]
    test_inventory: Option<TestInventory>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TestInventory {
    listed: usize,
    sha256: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CandidateBaseline {
    version: u8,
    #[serde(default)]
    snapshot_role: String,
    #[serde(default)]
    comparison_rule: String,
    exact_production_write_set: Vec<PathBuf>,
    entries: Vec<BaselineEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BaselineEntry {
    status: String,
    path: PathBuf,
    sha256: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProtectedFile {
    path: PathBuf,
    sha256: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProtectedSuffix {
    path: PathBuf,
    start_marker: String,
    sha256: String,
}

#[derive(Debug)]
struct Arguments {
    root: PathBuf,
    manifest: PathBuf,
    manifest_sha256: String,
    required_protected: Vec<PathBuf>,
    print_contracts: bool,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("handoff-guard: {error}");
        process::exit(2);
    }
}

fn run() -> Result<(), String> {
    let arguments = parse_arguments(env::args().skip(1))?;
    let actual_manifest_digest = sha256(&arguments.manifest)?;
    if !actual_manifest_digest.eq_ignore_ascii_case(&arguments.manifest_sha256) {
        return Err(format!(
            "manifest digest changed: expected {}, actual {} ({})",
            arguments.manifest_sha256,
            actual_manifest_digest,
            arguments.manifest.display()
        ));
    }

    let source = fs::read_to_string(&arguments.manifest)
        .map_err(|error| format!("cannot read {}: {error}", arguments.manifest.display()))?;
    let manifest: CandidateHandoffManifest = ron::from_str(&source)
        .map_err(|error| format!("invalid candidate handoff manifest: {error}"))?;
    validate_manifest(&manifest)?;
    require_protected_paths(&manifest, &arguments.required_protected)?;

    for protected in &manifest.protected_files {
        let path = arguments.root.join(&protected.path);
        let actual = sha256(&path)?;
        if !actual.eq_ignore_ascii_case(&protected.sha256) {
            return Err(format!(
                "protected file changed: {} (expected {}, actual {})",
                protected.path.display(),
                protected.sha256,
                actual
            ));
        }
    }
    for protected in &manifest.protected_suffixes {
        validate_protected_suffix(&arguments.root, protected)?;
    }

    let baseline_path = arguments.root.join(&manifest.baseline_path);
    let baseline_source = fs::read_to_string(&baseline_path)
        .map_err(|error| format!("cannot read {}: {error}", baseline_path.display()))?;
    let baseline: CandidateBaseline = ron::from_str(&baseline_source)
        .map_err(|error| format!("invalid candidate baseline: {error}"))?;
    validate_baseline(&baseline, &manifest.exact_production_write_set)?;
    validate_worktree_scope(
        &arguments.root,
        &arguments.manifest,
        &manifest.baseline_path,
        &manifest.exact_production_write_set,
        &baseline.entries,
    )?;
    if let Some(inventory) = &manifest.test_inventory {
        validate_test_inventory(&arguments.root, inventory)?;
    }

    if arguments.print_contracts {
        for contract in &manifest.contracts {
            println!("{contract}");
        }
    }
    Ok(())
}

fn require_protected_paths(
    manifest: &CandidateHandoffManifest,
    required: &[PathBuf],
) -> Result<(), String> {
    let protected = manifest
        .protected_files
        .iter()
        .map(|file| file.path.as_path())
        .collect::<BTreeSet<_>>();
    for path in required {
        if !safe_relative_path(path) {
            return Err(format!(
                "required protected path must be repository-relative: {}",
                path.display()
            ));
        }
        if !protected.contains(path.as_path()) {
            return Err(format!(
                "candidate handoff omits required protected file: {}",
                path.display()
            ));
        }
    }
    Ok(())
}

fn validate_manifest(manifest: &CandidateHandoffManifest) -> Result<(), String> {
    if manifest.version != 2 {
        return Err(format!(
            "unsupported candidate handoff manifest version {}",
            manifest.version
        ));
    }
    if manifest.contracts.is_empty() {
        return Err("candidate handoff must name at least one contract".into());
    }
    if manifest.protected_files.is_empty() {
        return Err("candidate handoff must protect at least one file".into());
    }
    if !safe_relative_path(&manifest.baseline_path) {
        return Err("candidate baseline path must be repository-relative".into());
    }

    let mut contracts = BTreeSet::new();
    for contract in &manifest.contracts {
        if contract.trim().is_empty() || !contracts.insert(contract.as_str()) {
            return Err(format!(
                "invalid or duplicate candidate contract {contract:?}"
            ));
        }
    }

    let mut paths = BTreeSet::new();
    for protected in &manifest.protected_files {
        if !safe_relative_path(&protected.path) || !paths.insert(protected.path.as_path()) {
            return Err(format!(
                "protected path must be unique and repository-relative: {}",
                protected.path.display()
            ));
        }
        if protected.sha256.len() != 64
            || !protected
                .sha256
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(format!(
                "protected file has invalid SHA-256 digest: {}",
                protected.path.display()
            ));
        }
    }
    if !paths.contains(manifest.baseline_path.as_path()) {
        return Err(format!(
            "candidate handoff must protect its baseline: {}",
            manifest.baseline_path.display()
        ));
    }

    validate_write_set(&manifest.exact_production_write_set)?;
    let write_set = manifest
        .exact_production_write_set
        .iter()
        .map(PathBuf::as_path)
        .collect::<BTreeSet<_>>();
    let mut suffix_paths = BTreeSet::new();
    for protected in &manifest.protected_suffixes {
        if !safe_relative_path(&protected.path) || !suffix_paths.insert(protected.path.as_path()) {
            return Err(format!(
                "protected suffix path must be unique and repository-relative: {}",
                protected.path.display()
            ));
        }
        if !write_set.contains(protected.path.as_path()) {
            return Err(format!(
                "protected suffix must belong to an authorized production path: {}",
                protected.path.display()
            ));
        }
        if paths.contains(protected.path.as_path()) {
            return Err(format!(
                "protected suffix path cannot also be protected in full: {}",
                protected.path.display()
            ));
        }
        if protected.start_marker.is_empty() {
            return Err(format!(
                "protected suffix must have a non-empty start marker: {}",
                protected.path.display()
            ));
        }
        validate_digest(&protected.sha256, &protected.path, "protected suffix")?;
    }
    if let Some(inventory) = &manifest.test_inventory {
        if inventory.listed == 0 {
            return Err("candidate test inventory must contain at least one test".into());
        }
        validate_digest(
            &inventory.sha256,
            Path::new("workspace test inventory"),
            "candidate",
        )?;
    }
    for path in &manifest.exact_production_write_set {
        if paths.contains(path.as_path()) {
            return Err(format!(
                "authorized production path cannot also be protected: {}",
                path.display()
            ));
        }
    }
    Ok(())
}

fn validate_protected_suffix(root: &Path, protected: &ProtectedSuffix) -> Result<(), String> {
    let source = fs::read(root.join(&protected.path)).map_err(|error| {
        format!(
            "cannot read protected suffix source {}: {error}",
            protected.path.display()
        )
    })?;
    let marker = protected.start_marker.as_bytes();
    let matches = source
        .windows(marker.len())
        .enumerate()
        .filter_map(|(index, window)| (window == marker).then_some(index))
        .collect::<Vec<_>>();
    let [start] = matches.as_slice() else {
        return Err(format!(
            "protected suffix start marker must occur exactly once in {} (found {})",
            protected.path.display(),
            matches.len()
        ));
    };
    let actual = sha256_bytes(&source[*start..])?;
    if !actual.eq_ignore_ascii_case(&protected.sha256) {
        return Err(format!(
            "protected suffix changed: {} from marker {:?} (expected {}, actual {})",
            protected.path.display(),
            protected.start_marker,
            protected.sha256,
            actual
        ));
    }
    Ok(())
}

fn normalized_test_inventory(output: &str) -> (usize, String) {
    let mut names = output
        .lines()
        .map(str::trim)
        .filter(|line| line.ends_with(": test"))
        .map(str::to_owned)
        .collect::<Vec<_>>();
    names.sort();
    let listed = names.len();
    let mut normalized = names.join("\n");
    if !normalized.is_empty() {
        normalized.push('\n');
    }
    (listed, normalized)
}

fn validate_test_inventory(root: &Path, expected: &TestInventory) -> Result<(), String> {
    let output = Command::new("cargo")
        .args(["test", "--workspace", "--locked", "--", "--list"])
        .current_dir(root)
        .output()
        .map_err(|error| format!("cannot list candidate workspace tests: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "cannot list candidate workspace tests: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let source = String::from_utf8(output.stdout)
        .map_err(|error| format!("candidate test inventory was not UTF-8: {error}"))?;
    let (listed, normalized) = normalized_test_inventory(&source);
    let actual_digest = sha256_bytes(normalized.as_bytes())?;
    if listed != expected.listed || !actual_digest.eq_ignore_ascii_case(&expected.sha256) {
        return Err(format!(
            "candidate test inventory changed: expected {} tests / {}, actual {} tests / {}",
            expected.listed, expected.sha256, listed, actual_digest
        ));
    }
    Ok(())
}

fn sha256_bytes(bytes: &[u8]) -> Result<String, String> {
    let mut child = Command::new("sha256sum")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|error| format!("cannot run sha256sum for test inventory: {error}"))?;
    child
        .stdin
        .take()
        .ok_or("cannot open sha256sum stdin for test inventory")?
        .write_all(bytes)
        .map_err(|error| format!("cannot hash candidate test inventory: {error}"))?;
    let output = child
        .wait_with_output()
        .map_err(|error| format!("cannot read test inventory digest: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "cannot hash candidate test inventory: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    String::from_utf8(output.stdout)
        .map_err(|error| format!("sha256sum returned non-UTF-8 output: {error}"))?
        .split_whitespace()
        .next()
        .map(str::to_owned)
        .ok_or_else(|| "sha256sum returned no test inventory digest".into())
}

fn validate_baseline(baseline: &CandidateBaseline, write_set: &[PathBuf]) -> Result<(), String> {
    if baseline.version != 1 {
        return Err(format!(
            "unsupported candidate baseline version {}",
            baseline.version
        ));
    }
    let _ = (&baseline.snapshot_role, &baseline.comparison_rule);
    validate_write_set(&baseline.exact_production_write_set)?;
    if baseline.exact_production_write_set != write_set {
        return Err("candidate manifest write set does not match sealed baseline".into());
    }

    let mut paths = BTreeSet::new();
    for entry in &baseline.entries {
        if entry.status.len() != 2 {
            return Err(format!(
                "baseline status must contain the two Git porcelain columns: {:?}",
                entry.status
            ));
        }
        if !safe_relative_path(&entry.path) || !paths.insert(entry.path.as_path()) {
            return Err(format!(
                "baseline path must be unique and repository-relative: {}",
                entry.path.display()
            ));
        }
        validate_digest(&entry.sha256, &entry.path, "baseline")?;
    }
    Ok(())
}

fn validate_write_set(write_set: &[PathBuf]) -> Result<(), String> {
    if write_set.is_empty() {
        return Err("candidate handoff must name an exact production write set".into());
    }
    let mut paths = BTreeSet::new();
    for path in write_set {
        if !safe_relative_path(path) || !paths.insert(path.as_path()) {
            return Err(format!(
                "write-set path must be unique and repository-relative: {}",
                path.display()
            ));
        }
    }
    Ok(())
}

fn validate_digest(digest: &str, path: &Path, owner: &str) -> Result<(), String> {
    if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!(
            "{owner} file has invalid SHA-256 digest: {}",
            path.display()
        ));
    }
    Ok(())
}

fn validate_worktree_scope(
    root: &Path,
    manifest_path: &Path,
    baseline_path: &Path,
    write_set: &[PathBuf],
    baseline_entries: &[BaselineEntry],
) -> Result<(), String> {
    let current = git_status(root)?;
    let baseline = baseline_entries
        .iter()
        .map(|entry| (entry.path.as_path(), entry))
        .collect::<BTreeMap<_, _>>();
    let allowed = write_set
        .iter()
        .map(PathBuf::as_path)
        .collect::<BTreeSet<_>>();
    let manifest_relative = repository_relative_path(root, manifest_path)?;
    let implicit = [manifest_relative.as_path(), baseline_path]
        .into_iter()
        .collect::<BTreeSet<_>>();

    for (path, status) in &current {
        if allowed.contains(path.as_path()) || implicit.contains(path.as_path()) {
            continue;
        }
        let Some(sealed) = baseline.get(path.as_path()) else {
            return Err(format!(
                "unauthorized worktree delta outside exact write set: {} ({status})",
                path.display()
            ));
        };
        if sealed.status != *status {
            return Err(format!(
                "unauthorized worktree status change outside exact write set: {} (baseline {:?}, current {:?})",
                path.display(),
                sealed.status,
                status
            ));
        }
        let actual = sha256(&root.join(path))?;
        if !actual.eq_ignore_ascii_case(&sealed.sha256) {
            return Err(format!(
                "unauthorized worktree content change outside exact write set: {}",
                path.display()
            ));
        }
    }

    for sealed in baseline_entries {
        if allowed.contains(sealed.path.as_path()) || current.contains_key(&sealed.path) {
            continue;
        }
        return Err(format!(
            "unauthorized worktree delta removed or reverted baseline path outside exact write set: {}",
            sealed.path.display()
        ));
    }
    Ok(())
}

fn repository_relative_path(root: &Path, path: &Path) -> Result<PathBuf, String> {
    let relative = if path.is_absolute() {
        path.strip_prefix(root).map_err(|_| {
            format!(
                "candidate manifest must be inside repository root: {}",
                path.display()
            )
        })?
    } else {
        path
    };
    if !safe_relative_path(relative) {
        return Err(format!(
            "candidate manifest path must be repository-relative: {}",
            relative.display()
        ));
    }
    Ok(relative.to_path_buf())
}

fn git_status(root: &Path) -> Result<BTreeMap<PathBuf, String>, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args([
            "status",
            "--porcelain=v1",
            "-z",
            "--untracked-files=all",
            "--no-renames",
        ])
        .output()
        .map_err(|error| format!("cannot inspect candidate worktree: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "cannot inspect candidate worktree: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let mut entries = BTreeMap::new();
    for record in output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|record| !record.is_empty())
    {
        if record.len() < 4 || record[2] != b' ' {
            return Err(format!(
                "unexpected Git porcelain record: {:?}",
                String::from_utf8_lossy(record)
            ));
        }
        let status = String::from_utf8(record[..2].to_vec())
            .map_err(|error| format!("Git status columns were not UTF-8: {error}"))?;
        let path = String::from_utf8(record[3..].to_vec())
            .map_err(|error| format!("candidate path was not UTF-8: {error}"))?;
        let path = PathBuf::from(path);
        if !safe_relative_path(&path) || entries.insert(path.clone(), status).is_some() {
            return Err(format!(
                "Git status path must be unique and repository-relative: {}",
                path.display()
            ));
        }
    }
    Ok(entries)
}

fn safe_relative_path(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn sha256(path: &Path) -> Result<String, String> {
    let output = Command::new("sha256sum")
        .arg("--")
        .arg(path)
        .output()
        .map_err(|error| format!("cannot run sha256sum for {}: {error}", path.display()))?;
    if !output.status.success() {
        return Err(format!(
            "cannot hash {}: {}",
            path.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    String::from_utf8(output.stdout)
        .map_err(|error| format!("sha256sum returned non-UTF-8 output: {error}"))?
        .split_whitespace()
        .next()
        .map(str::to_owned)
        .ok_or_else(|| format!("sha256sum returned no digest for {}", path.display()))
}

fn parse_arguments(arguments: impl IntoIterator<Item = String>) -> Result<Arguments, String> {
    let mut root = None;
    let mut manifest = None;
    let mut manifest_sha256 = None;
    let mut required_protected = Vec::new();
    let mut print_contracts = false;
    let mut arguments = arguments.into_iter();

    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--root" => root = Some(next_path(&mut arguments, "--root")?),
            "--manifest" => manifest = Some(next_path(&mut arguments, "--manifest")?),
            "--manifest-sha256" => {
                manifest_sha256 = Some(next_string(&mut arguments, "--manifest-sha256")?)
            }
            "--require-protected" => {
                required_protected.push(next_path(&mut arguments, "--require-protected")?)
            }
            "--print-contracts" => print_contracts = true,
            unknown => return Err(format!("unknown argument: {unknown}")),
        }
    }

    Ok(Arguments {
        root: root.ok_or("missing --root")?,
        manifest: manifest.ok_or("missing --manifest")?,
        manifest_sha256: manifest_sha256.ok_or("missing --manifest-sha256")?,
        required_protected,
        print_contracts,
    })
}

fn next_path(
    arguments: &mut impl Iterator<Item = String>,
    option: &str,
) -> Result<PathBuf, String> {
    arguments
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| format!("missing value for {option}"))
}

fn next_string(
    arguments: &mut impl Iterator<Item = String>,
    option: &str,
) -> Result<String, String> {
    arguments
        .next()
        .ok_or_else(|| format!("missing value for {option}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalized_inventory_is_exact_sorted_and_duplicate_sensitive() {
        let source = "noise\nzeta::case: test\nalpha::case: test\n\
                      alpha::case: test\nignored: benchmark\n";
        let (listed, normalized) = normalized_test_inventory(source);

        assert_eq!(listed, 3);
        assert_eq!(
            normalized,
            "alpha::case: test\nalpha::case: test\nzeta::case: test\n"
        );
        assert_ne!(
            sha256_bytes(normalized.as_bytes()).expect("fixture must hash"),
            sha256_bytes("alpha::case: test\nzeta::case: test\n".as_bytes())
                .expect("deletion fixture must hash"),
            "deleting one duplicate test must change the sealed inventory"
        );
    }
}
