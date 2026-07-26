//! Machine-readable Foundation contract metadata and deterministic validation.
//!
//! This module owns test-governance data only. It intentionally contains no
//! gameplay rules and does not infer execution outcomes from Rust source.

use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    path::{Component, Path, PathBuf},
};

const VALID_SCOPES: &[&str] = &[
    "FoundationRequired",
    "FoundationSupport",
    "Regression",
    "FutureProduct",
    "DeferredInfrastructure",
    "Diagnostic",
    "LegacyPendingRetirement",
];

const VALID_EVIDENCE_LAYERS: &[&str] = &[
    "Domain",
    "Schedule",
    "StateDiff",
    "Projection",
    "BufferLayout",
    "InputStateMachine",
    "Workflow",
    "Persistence",
    "PTY",
];

const VALID_PROFILES: &[&str] = &[
    "Foundation",
    "FoundationSupport",
    "Regression",
    "FutureProduct",
    "DeferredInfrastructure",
    "Diagnostic",
    "LegacyPendingRetirement",
    "Baseline80x24",
    "Compact60x20",
    "Repetition",
    "Property",
    "Stress",
];

const VALID_STATUSES: &[&str] = &[
    "NotImplemented",
    "Red",
    "GreenUnreviewed",
    "Accepted",
    "Deferred",
    "Retired",
];

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ContractRegistry {
    pub contracts: Vec<ContractRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ContractRecord {
    pub id: String,
    pub title: String,
    pub scope: String,
    pub authority_references: Vec<String>,
    pub player_outcome: String,
    pub primary_test: Option<String>,
    pub supporting_tests: Vec<String>,
    pub evidence_layers: Vec<String>,
    pub profiles: Vec<String>,
    pub fixture_id: String,
    pub owner_phase: u8,
    pub status: String,
    pub known_failure: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryParseError(String);

impl fmt::Display for RegistryParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for RegistryParseError {}

impl ContractRegistry {
    pub fn parse(source: &str) -> Result<Self, RegistryParseError> {
        ron::from_str(source)
            .map_err(|error| RegistryParseError(format!("invalid contract registry RON: {error}")))
    }

    pub fn load(path: &Path) -> Result<Self, RegistryLoadError> {
        let source = std::fs::read_to_string(path).map_err(|source| RegistryLoadError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        Self::parse(&source).map_err(|source| RegistryLoadError::Parse {
            path: path.to_path_buf(),
            source,
        })
    }

    pub fn validate(&self, context: &RegistryValidationContext) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        let mut contract_ids = BTreeSet::new();
        let mut primary_owners: BTreeMap<&str, &str> = BTreeMap::new();

        for contract in &self.contracts {
            if !contract_ids.insert(contract.id.as_str()) {
                issues.push(ValidationIssue::new(
                    ValidationCode::DuplicateContractId,
                    &contract.id,
                    &contract.id,
                ));
            }

            if !VALID_SCOPES.contains(&contract.scope.as_str()) {
                issues.push(ValidationIssue::new(
                    ValidationCode::UnknownScope,
                    &contract.id,
                    &contract.scope,
                ));
            }
            if !VALID_STATUSES.contains(&contract.status.as_str()) {
                issues.push(ValidationIssue::new(
                    ValidationCode::UnknownStatus,
                    &contract.id,
                    &contract.status,
                ));
            }

            for authority in &contract.authority_references {
                let authority_path = authority
                    .split_once('#')
                    .map_or(authority.as_str(), |(path, _)| path);
                let repository_relative = Path::new(authority_path);
                if repository_relative.is_absolute()
                    || repository_relative
                        .components()
                        .any(|component| matches!(component, Component::ParentDir))
                {
                    issues.push(ValidationIssue::new(
                        ValidationCode::AuthorityOutsideRepository,
                        &contract.id,
                        authority,
                    ));
                    continue;
                }
                if authority_path.is_empty() || !context.project_root.join(authority_path).is_file()
                {
                    issues.push(ValidationIssue::new(
                        ValidationCode::MissingAuthority,
                        &contract.id,
                        authority,
                    ));
                }
            }

            if contract.authority_references.is_empty() {
                issues.push(ValidationIssue::new(
                    ValidationCode::MissingAuthority,
                    &contract.id,
                    "no authority references",
                ));
            }

            let required = contract.scope == "FoundationRequired";
            if required && contract.primary_test.is_none() {
                issues.push(ValidationIssue::new(
                    ValidationCode::MissingPrimaryTest,
                    &contract.id,
                    "FoundationRequired contract has no primary test",
                ));
            }
            if required && contract.status == "Deferred" {
                issues.push(ValidationIssue::new(
                    ValidationCode::DeferredRequiredContract,
                    &contract.id,
                    "FoundationRequired contract cannot be deferred",
                ));
            }
            if required && contract.evidence_layers.is_empty() {
                issues.push(ValidationIssue::new(
                    ValidationCode::MissingEvidenceLayers,
                    &contract.id,
                    "FoundationRequired contract has no evidence layers",
                ));
            }

            if let Some(primary) = contract.primary_test.as_deref() {
                if let Some(owner) = primary_owners.insert(primary, &contract.id) {
                    issues.push(ValidationIssue::new(
                        ValidationCode::DuplicatePrimaryOwner,
                        &contract.id,
                        format!("{primary} is already owned by {owner}"),
                    ));
                }
                if !context.known_tests.contains(primary) {
                    issues.push(ValidationIssue::new(
                        ValidationCode::UnknownPrimaryTest,
                        &contract.id,
                        primary,
                    ));
                }
                if required && context.ignored_tests.contains(primary) {
                    issues.push(ValidationIssue::new(
                        ValidationCode::IgnoredRequiredTest,
                        &contract.id,
                        primary,
                    ));
                }
                if contract.status == "Retired" {
                    issues.push(ValidationIssue::new(
                        ValidationCode::RetiredPrimaryOwner,
                        &contract.id,
                        primary,
                    ));
                }
            }

            for supporting in &contract.supporting_tests {
                if !context.known_tests.contains(supporting) {
                    issues.push(ValidationIssue::new(
                        ValidationCode::UnknownSupportingTest,
                        &contract.id,
                        supporting,
                    ));
                }
            }
            for layer in &contract.evidence_layers {
                if !VALID_EVIDENCE_LAYERS.contains(&layer.as_str()) {
                    issues.push(ValidationIssue::new(
                        ValidationCode::UnknownEvidenceLayer,
                        &contract.id,
                        layer,
                    ));
                }
            }
            for profile in &contract.profiles {
                if !VALID_PROFILES.contains(&profile.as_str()) {
                    issues.push(ValidationIssue::new(
                        ValidationCode::UnknownProfile,
                        &contract.id,
                        profile,
                    ));
                }
            }

            if contract.status == "Red" && contract.known_failure.is_none() {
                issues.push(ValidationIssue::new(
                    ValidationCode::MissingKnownFailure,
                    &contract.id,
                    "Red contract has no known failure",
                ));
            }
            if contract.status == "Accepted"
                && contract.id.starts_with("VISUAL-")
                && !context
                    .visual_evidence
                    .contains(contract.fixture_id.as_str())
            {
                issues.push(ValidationIssue::new(
                    ValidationCode::MissingVisualEvidence,
                    &contract.id,
                    &contract.fixture_id,
                ));
            }
        }

        issues.sort();
        issues
    }
}

#[derive(Debug)]
pub enum RegistryLoadError {
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    Parse {
        path: PathBuf,
        source: RegistryParseError,
    },
}

impl fmt::Display for RegistryLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { path, source } => {
                write!(formatter, "cannot read {}: {source}", path.display())
            }
            Self::Parse { path, source } => {
                write!(formatter, "cannot parse {}: {source}", path.display())
            }
        }
    }
}

impl std::error::Error for RegistryLoadError {}

#[derive(Debug, Clone)]
pub struct RegistryValidationContext {
    project_root: PathBuf,
    known_tests: BTreeSet<String>,
    ignored_tests: BTreeSet<String>,
    visual_evidence: BTreeSet<String>,
}

impl RegistryValidationContext {
    pub fn new(project_root: impl Into<PathBuf>) -> Self {
        Self {
            project_root: project_root.into(),
            known_tests: BTreeSet::new(),
            ignored_tests: BTreeSet::new(),
            visual_evidence: BTreeSet::new(),
        }
    }

    pub fn with_known_tests(mut self, tests: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.known_tests = tests.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_ignored_tests(
        mut self,
        tests: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.ignored_tests = tests.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_visual_evidence(
        mut self,
        fixtures: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.visual_evidence = fixtures.into_iter().map(Into::into).collect();
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum ValidationCode {
    AuthorityOutsideRepository,
    DeferredRequiredContract,
    DuplicateContractId,
    DuplicatePrimaryOwner,
    IgnoredRequiredTest,
    MissingAuthority,
    MissingEvidenceLayers,
    MissingKnownFailure,
    MissingPrimaryTest,
    MissingVisualEvidence,
    RetiredPrimaryOwner,
    UnknownEvidenceLayer,
    UnknownPrimaryTest,
    UnknownProfile,
    UnknownScope,
    UnknownStatus,
    UnknownSupportingTest,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct ValidationIssue {
    pub code: ValidationCode,
    pub contract_id: String,
    pub detail: String,
}

impl ValidationIssue {
    fn new(
        code: ValidationCode,
        contract_id: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            code,
            contract_id: contract_id.into(),
            detail: detail.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TestResult {
    Passed,
    Failed,
    Ignored,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TestEvidence {
    pub test: String,
    pub result: TestResult,
}

impl TestEvidence {
    pub fn passed(test: impl Into<String>) -> Self {
        Self {
            test: test.into(),
            result: TestResult::Passed,
        }
    }

    pub fn failed(test: impl Into<String>) -> Self {
        Self {
            test: test.into(),
            result: TestResult::Failed,
        }
    }

    pub fn ignored(test: impl Into<String>) -> Self {
        Self {
            test: test.into(),
            result: TestResult::Ignored,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RegistryReport {
    contracts: ContractTotals,
    tests: TestTotals,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ContractTotals {
    listed: usize,
    required: usize,
    accepted: usize,
    green_unreviewed: usize,
    not_implemented: usize,
    red: usize,
    deferred: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct TestTotals {
    listed: usize,
    passed: usize,
    failed: usize,
    ignored: usize,
}

impl RegistryReport {
    pub fn from_registry(registry: &ContractRegistry, evidence: &[TestEvidence]) -> Self {
        Self {
            contracts: ContractTotals {
                listed: registry.contracts.len(),
                required: registry
                    .contracts
                    .iter()
                    .filter(|contract| contract.scope == "FoundationRequired")
                    .count(),
                accepted: registry
                    .contracts
                    .iter()
                    .filter(|contract| contract.status == "Accepted")
                    .count(),
                green_unreviewed: registry
                    .contracts
                    .iter()
                    .filter(|contract| contract.status == "GreenUnreviewed")
                    .count(),
                not_implemented: registry
                    .contracts
                    .iter()
                    .filter(|contract| contract.status == "NotImplemented")
                    .count(),
                red: registry
                    .contracts
                    .iter()
                    .filter(|contract| contract.status == "Red")
                    .count(),
                deferred: registry
                    .contracts
                    .iter()
                    .filter(|contract| contract.status == "Deferred")
                    .count(),
            },
            tests: TestTotals {
                listed: evidence.len(),
                passed: evidence
                    .iter()
                    .filter(|item| item.result == TestResult::Passed)
                    .count(),
                failed: evidence
                    .iter()
                    .filter(|item| item.result == TestResult::Failed)
                    .count(),
                ignored: evidence
                    .iter()
                    .filter(|item| item.result == TestResult::Ignored)
                    .count(),
            },
        }
    }

    pub fn to_text(&self) -> String {
        format!(
            "contracts:\n  listed: {}\n  required: {}\n  accepted: {}\n  green_unreviewed: {}\n  not_implemented: {}\n  red: {}\n  deferred: {}\ntests:\n  listed: {}\n  passed: {}\n  failed: {}\n  ignored: {}\n",
            self.contracts.listed,
            self.contracts.required,
            self.contracts.accepted,
            self.contracts.green_unreviewed,
            self.contracts.not_implemented,
            self.contracts.red,
            self.contracts.deferred,
            self.tests.listed,
            self.tests.passed,
            self.tests.failed,
            self.tests.ignored,
        )
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}
