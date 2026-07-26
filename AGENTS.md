# Broken Divinity Development Contract

These instructions apply to every change in this repository.

## Authority

Before changing behavior, read the relevant sections of:

1. [`GDD.md`](GDD.md) — product intent and player experience;
2. [`Kernel.md`](Kernel.md) — technical architecture;
3. [`docs/DECISIONS-TO-LOCK.md`](docs/DECISIONS-TO-LOCK.md) — locked scope and behavior;
4. [`docs/AUTHORITATIVE-TESTING-STANDARD-AND-MIGRATION-PLAN.md`](docs/AUTHORITATIVE-TESTING-STANDARD-AND-MIGRATION-PLAN.md) — test design, evidence, metrics, and execution protocol;
5. [`testing/foundation-contracts.ron`](testing/foundation-contracts.ron) — machine-readable contract ownership;
6. the current implementation and tests.

The GDD owns product truth. Tests are evidence, not authority. If the sources
conflict or a requirement is ambiguous, stop and ask the owner instead of
inventing behavior.

## Mandatory TDD workflow

For every behavior change or defect:

1. Identify the authority reference and affected contract.
2. Write or strengthen the smallest deterministic test that expresses one
   player-visible or domain outcome and its forbidden mutations.
3. Run that focused test and observe the expected failure. If it passes before
   implementation, explain why before proceeding.
4. Make the smallest production change needed to satisfy the contract.
5. Rerun the focused test, then neighboring tests, then the affected player
   workflow.
6. Run `bash scripts/test-gate.sh` before declaring the task complete.
7. Compare the result to the GDD and update contract/evidence records when
   their status changed.

Do not write production behavior first and backfill tests afterward. Refactors
must begin with green characterization tests unless existing authoritative
tests already cover the boundary.

## Test quality rules

- One primary test owns one contract. Avoid broad tests with overlapping
  responsibility.
- Use stable names, content IDs, positions, and normalized fingerprints. Never
  rely on raw entity IDs or ECS iteration order.
- Assert exact state deltas and forbidden mutations for domain behavior.
- Test player workflows through production inputs and projections; direct
  world mutation is fixture setup only.
- A substring alone does not prove visibility, readability, selection, layout,
  or discoverability.
- Tests must be deterministic: explicit fixtures, seeds, frame counts, and
  terminal profiles; no sleeps or wall-clock correctness checks.
- Failure output must identify the contract, case, precondition, action,
  expected/actual result, and useful state or visual context.
- Do not add a public production API solely for tests or copy production logic
  into expected-value helpers.
- Do not weaken, ignore, delete, reclassify, or replace a test merely to make a
  gate green. Follow the retirement rules in the authoritative testing
  standard.

## Completion rules

A task is not complete merely because tests pass. Completion requires:

- the focused red-to-green evidence;
- no regression in the affected workflow;
- a clean canonical gate with measured test totals;
- no new warnings, ignored required tests, pending snapshots, or unexplained
  visual diffs;
- GDD and locked-decision drift review;
- required visual or real-terminal evidence for player-facing work;
- accurate contract and evidence status (`GreenUnreviewed` is not
  `Accepted`).

If the gate exposes a pre-existing or unrelated failure, report it explicitly.
Never hide it in a summary or claim the repository is green.
