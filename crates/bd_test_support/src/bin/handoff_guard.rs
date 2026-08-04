use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::{Component, Path, PathBuf},
    process::{self, Command},
};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CandidateHandoffManifest {
    version: u8,
    contracts: Vec<String>,
    baseline_path: PathBuf,
    exact_production_write_set: Vec<PathBuf>,
    protected_files: Vec<ProtectedFile>,
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
