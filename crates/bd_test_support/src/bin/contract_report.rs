use bd_test_support::contract_registry::{ContractRegistry, RegistryReport, TestEvidence};
use std::{env, path::PathBuf, process};

#[derive(Debug)]
struct Arguments {
    registry: PathBuf,
    listed: usize,
    passed: usize,
    failed: usize,
    ignored: usize,
    json: bool,
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

fn parse_arguments(arguments: impl IntoIterator<Item = String>) -> Result<Arguments, String> {
    let mut registry = None;
    let mut listed = None;
    let mut passed = None;
    let mut failed = None;
    let mut ignored = None;
    let mut json = false;
    let mut arguments = arguments.into_iter();

    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--registry" => registry = Some(next_path(&mut arguments, "--registry")?),
            "--listed" => listed = Some(next_count(&mut arguments, "--listed")?),
            "--passed" => passed = Some(next_count(&mut arguments, "--passed")?),
            "--failed" => failed = Some(next_count(&mut arguments, "--failed")?),
            "--ignored" => ignored = Some(next_count(&mut arguments, "--ignored")?),
            "--json" => json = true,
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

fn next_count(arguments: &mut impl Iterator<Item = String>, option: &str) -> Result<usize, String> {
    let value = arguments
        .next()
        .ok_or_else(|| format!("missing value for {option}"))?;
    value
        .parse()
        .map_err(|error| format!("invalid value for {option}: {error}"))
}
