use bd_test_support::contract_registry::{ContractRegistry, RegistryReport, TestEvidence};
use std::{collections::BTreeSet, env, path::PathBuf, process};

#[derive(Debug)]
struct Arguments {
    registry: PathBuf,
    listed: usize,
    passed: usize,
    failed: usize,
    ignored: usize,
    json: bool,
    candidate_contracts: Vec<String>,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("contract-report: {error}");
        process::exit(2);
    }
}

fn run() -> Result<(), String> {
    let arguments = parse_arguments(env::args().skip(1))?;
    let observed = arguments.passed + arguments.failed + arguments.ignored;
    if observed != arguments.listed {
        return Err(format!(
            "listed total {} does not equal {} observed outcomes",
            arguments.listed, observed
        ));
    }

    let registry =
        ContractRegistry::load(&arguments.registry).map_err(|error| error.to_string())?;
    let status_error = if arguments.candidate_contracts.is_empty() {
        required_red_status_drift(&registry, arguments.failed)
    } else {
        candidate_status_drift(&registry, &arguments.candidate_contracts)
    };
    if let Some(error) = status_error {
        return Err(error);
    }
    let mut evidence = Vec::with_capacity(arguments.listed);
    evidence.extend(
        (0..arguments.passed).map(|index| TestEvidence::passed(format!("passed::{index}"))),
    );
    evidence.extend(
        (0..arguments.failed).map(|index| TestEvidence::failed(format!("failed::{index}"))),
    );
    evidence.extend(
        (0..arguments.ignored).map(|index| TestEvidence::ignored(format!("ignored::{index}"))),
    );
    let report = RegistryReport::from_registry(&registry, &evidence);

    if arguments.json {
        println!(
            "{}",
            report
                .to_json()
                .map_err(|error| format!("cannot serialize report: {error}"))?
        );
    } else {
        print!("{}", report.to_text());
    }
    Ok(())
}

fn candidate_status_drift(
    registry: &ContractRegistry,
    candidate_contracts: &[String],
) -> Option<String> {
    let declared = candidate_contracts
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if declared.len() != candidate_contracts.len() {
        return Some("candidate handoff contains duplicate contract IDs".into());
    }

    for contract_id in &declared {
        let Some(contract) = registry
            .contracts
            .iter()
            .find(|contract| contract.id == *contract_id)
        else {
            return Some(format!(
                "candidate handoff names unknown contract {contract_id}"
            ));
        };
        if contract.scope != "FoundationRequired" {
            return Some(format!(
                "candidate contract {contract_id} is {}, not FoundationRequired",
                contract.scope
            ));
        }
        if contract.status != "Red" {
            return Some(format!(
                "candidate contract {contract_id} must remain Red until independent review; actual status is {}",
                contract.status
            ));
        }
    }

    let actual_red = registry
        .contracts
        .iter()
        .filter(|contract| contract.scope == "FoundationRequired" && contract.status == "Red")
        .map(|contract| contract.id.as_str())
        .collect::<BTreeSet<_>>();
    let unlisted = actual_red
        .difference(&declared)
        .copied()
        .collect::<Vec<_>>();
    if !unlisted.is_empty() {
        return Some(format!(
            "required Red contracts not declared by the candidate handoff: {}",
            unlisted.join(", ")
        ));
    }
    None
}

fn required_red_status_drift(registry: &ContractRegistry, failed: usize) -> Option<String> {
    if failed != 0 {
        return None;
    }
    let mut stale = registry
        .contracts
        .iter()
        .filter(|contract| contract.scope == "FoundationRequired" && contract.status == "Red")
        .map(|contract| contract.id.as_str())
        .collect::<Vec<_>>();
    stale.sort_unstable();
    (!stale.is_empty()).then(|| {
        format!(
            "all observed tests passed but required contracts remain Red: {}; \
             restore a reproducible failure or update reviewed registry/evidence status",
            stale.join(", ")
        )
    })
}

fn parse_arguments(arguments: impl IntoIterator<Item = String>) -> Result<Arguments, String> {
    let mut registry = None;
    let mut listed = None;
    let mut passed = None;
    let mut failed = None;
    let mut ignored = None;
    let mut json = false;
    let mut candidate_contracts = Vec::new();
    let mut arguments = arguments.into_iter();

    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--registry" => registry = Some(next_path(&mut arguments, "--registry")?),
            "--listed" => listed = Some(next_count(&mut arguments, "--listed")?),
            "--passed" => passed = Some(next_count(&mut arguments, "--passed")?),
            "--failed" => failed = Some(next_count(&mut arguments, "--failed")?),
            "--ignored" => ignored = Some(next_count(&mut arguments, "--ignored")?),
            "--json" => json = true,
            "--candidate-contract" => {
                candidate_contracts.push(next_string(&mut arguments, "--candidate-contract")?)
            }
            unknown => return Err(format!("unknown argument: {unknown}")),
        }
    }

    Ok(Arguments {
        registry: registry.ok_or("missing --registry")?,
        listed: listed.ok_or("missing --listed")?,
        passed: passed.ok_or("missing --passed")?,
        failed: failed.ok_or("missing --failed")?,
        ignored: ignored.ok_or("missing --ignored")?,
        json,
        candidate_contracts,
    })
}

fn next_string(
    arguments: &mut impl Iterator<Item = String>,
    option: &str,
) -> Result<String, String> {
    arguments
        .next()
        .ok_or_else(|| format!("missing value for {option}"))
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

fn next_count(arguments: &mut impl Iterator<Item = String>, option: &str) -> Result<usize, String> {
    let value = arguments
        .next()
        .ok_or_else(|| format!("missing value for {option}"))?;
    value
        .parse()
        .map_err(|error| format!("invalid value for {option}: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bd_test_support::contract_registry::ContractRecord;

    fn registry(scope: &str, status: &str) -> ContractRegistry {
        ContractRegistry {
            contracts: vec![ContractRecord {
                id: "VISUAL-TEST-001".into(),
                title: "Status alignment fixture".into(),
                scope: scope.into(),
                authority_references: vec!["GDD.md#fixture".into()],
                player_outcome: "Fixture".into(),
                primary_test: Some("fixture::test".into()),
                supporting_tests: Vec::new(),
                evidence_layers: vec!["Projection".into()],
                profiles: vec!["Foundation".into()],
                fixture_id: "status_alignment".into(),
                owner_phase: 0,
                status: status.into(),
                known_failure: (status == "Red").then(|| "Known failure".into()),
            }],
        }
    }

    #[test]
    fn passing_suite_rejects_required_red_status_drift() {
        let error = required_red_status_drift(&registry("FoundationRequired", "Red"), 0)
            .expect("passing suite with a required Red contract must be status drift");
        assert!(error.contains("VISUAL-TEST-001"));
    }

    #[test]
    fn failing_suite_keeps_red_status_available_for_tdd() {
        assert!(required_red_status_drift(&registry("FoundationRequired", "Red"), 1).is_none());
    }

    #[test]
    fn passing_suite_accepts_reviewed_non_red_status() {
        assert!(
            required_red_status_drift(&registry("FoundationRequired", "GreenUnreviewed"), 0)
                .is_none()
        );
    }

    #[test]
    fn candidate_mode_accepts_only_the_named_required_red_contracts() {
        let registry = registry("FoundationRequired", "Red");
        assert!(
            candidate_status_drift(&registry, &["VISUAL-TEST-001".into()]).is_none(),
            "candidate green must preserve the author-owned Red record"
        );
    }

    #[test]
    fn candidate_mode_rejects_self_promotion() {
        let registry = registry("FoundationRequired", "GreenUnreviewed");
        let error = candidate_status_drift(&registry, &["VISUAL-TEST-001".into()])
            .expect("candidate contract promotion must be rejected");
        assert!(error.contains("VISUAL-TEST-001"));
        assert!(error.contains("must remain Red"));
    }

    #[test]
    fn candidate_mode_rejects_an_unlisted_required_red_contract() {
        let registry = registry("FoundationRequired", "Red");
        let error = candidate_status_drift(&registry, &[])
            .expect("unlisted Red status must remain canonical drift");
        assert!(error.contains("not declared by the candidate handoff"));
    }

    #[test]
    fn candidate_contract_arguments_are_repeatable() {
        let arguments = parse_arguments(
            [
                "--registry",
                "testing/foundation-contracts.ron",
                "--listed",
                "1",
                "--passed",
                "1",
                "--failed",
                "0",
                "--ignored",
                "0",
                "--candidate-contract",
                "CONTRACT-A",
                "--candidate-contract",
                "CONTRACT-B",
            ]
            .into_iter()
            .map(str::to_owned),
        )
        .expect("candidate arguments must parse");
        assert_eq!(
            arguments.candidate_contracts,
            vec!["CONTRACT-A", "CONTRACT-B"]
        );
    }
}
