# UI9-C Compact Context Composition Candidate Handoff — v4

Use `$authoritative-test-pipeline`. Implement and internally validate this
complete authorized batch in a loop until the signed gate reports
`STATUS=CandidateGreen` or a stop condition requires `STATUS=NotComplete`.
End every loop with the required in-chat handoff. Do not create a repository
handoff, report, bug log, or evidence file.

The separately supplied v4 manifest digest is not authority. This protected
body owns scope, priority, commands, stop conditions, and acceptance. V1-v3
prompts, bodies, manifests, baselines, digests, and reports are withdrawn
historical evidence and must not be reused or modified.

## Priority

1. Preserve protected authority, the sealed v4 baseline, and user work.
2. Obey every stop condition and the exact production write set.
3. Repair compact Context composition through one reusable presentation owner.
4. Preserve shared-detail ownership and every registered UI9-C behavior.
5. Run every exact, neighboring, and signed command.
6. Report measured evidence in chat without promoting its meaning.

A lower-priority green never waives a higher-priority violation.

## Read before editing

Read `AGENTS.md`, the complete `$authoritative-test-pipeline` skill, `GDD.md`,
`Kernel.md`, `docs/DECISIONS-TO-LOCK.md`, UI plan Sections 17.2-17.9, the latest
`VISUAL-CONTEXT-001` registry/evidence records, every registered UI9-C test,
and:

```text
testing/UI9-C-CONTEXT-BASELINE-v4.ron
testing/UI9-C-CONTEXT-CANDIDATE-HANDOFF-v4.ron
```

Do not edit this body, the baseline, manifest, tests, fixtures, observers,
plans, authority, registry, evidence, policy, gate code, or status. Do not
regenerate the supplied digest. A required change to any such file is an
observer/scope stop, not an implementation-loop step.

## Exact production write set

You may modify only:

```text
crates/bd_tui/src/view_models.rs
crates/bd_tui/src/screens.rs
```

Both files contain pre-handoff work recorded by the sealed v4 baseline.
Preserve it and do not claim it as v4 evidence. Before every edit, verify the
target is one of these two exact paths. After every loop, run the signed guard
or gate and inspect the complete baseline delta, including untracked paths.
Any other added, modified, deleted, or renamed Git-visible path is an immediate
`NotComplete` scope stop. Never reset, revert, delete, or broadly format user
work.

## Outcome

Preserve the v3 corrections:

- `NearbyTarget.detail` reaches `ContextTargetVm.status` with every semantic
  segment intact; separator normalization only;
- Context titles follow authoritative category/status and never parallel
  worker/recipe/progress fields; and
- the adversarial Unstaffed projection remains coherently Unstaffed in the
  view model and final output, with all decoys absent.

Repair the generic Context presentation so default and active station, node,
and colonist content remains complete, legible, contained, and truthful at
both 80x24 and 60x20. The default station must retain `Coming later`; the
staffed station must retain worker/recipe/progress, `Assign Worker`, and every
visible denial reason; the assigned node must retain worker/progress,
`Assign Gatherer`, and every visible denial reason.

The reusable screen owner may arrange, wrap, style, group, or allocate the
structured target/detail/action representation. It may not parse display prose
to recover facts, delete semantic segments, duplicate category/action rules,
hardcode a fixture/category/profile/terminal size, hide required reasons,
flatten multiple targets, sacrifice map primacy, or introduce a second Context
renderer. A shared layout/profile primitive is allowed; a `60x20` special-case
branch is not.

UI9-D remains forbidden: no Interact binding or reducer, Context menu state,
enabled preview actions, input reroute, simulation, assignment, recipe,
movement, resource, or time change.

## Starting red reproduction

Run these independently before editing:

```text
CARGO_INCREMENTAL=0 cargo test -p bd_tui --lib ui_development_contract_tests::station_context_survives_final_composition_at_supported_profiles -- --exact --nocapture
CARGO_INCREMENTAL=0 cargo test -p bd_tui --lib ui_development_contract_tests::staffed_station_recipe_progress_survives_final_composition -- --exact --nocapture
CARGO_INCREMENTAL=0 cargo test -p bd_tui --lib ui_development_contract_tests::assigned_node_worker_progress_survives_final_composition -- --exact --nocapture
```

Expected diagnostics are respectively:

1. `station-60x20` cannot read the `Coming later` reason;
2. `station-staffed-60x20` cannot read `Assign Worker`; and
3. `node-assigned-60x20` cannot read the complete `Assign Gatherer` action.

If a command executes zero tests, passes, or fails for another cause, stop
before editing and report the changed baseline. Do not use a bare function name
with `--exact`; the fully qualified names above are mandatory.

Then independently confirm both v3 preservation rows are green:

```text
CARGO_INCREMENTAL=0 cargo test -p bd_tui --lib ui_development_contract_tests::context_view_model_transports_shared_detail_without_semantic_parsing -- --exact --nocapture
CARGO_INCREMENTAL=0 cargo test -p bd_tui --lib ui_development_contract_tests::final_context_consumes_the_shared_detail_projection_once -- --exact --nocapture
```

## Required loop

Repeat:

```text
READ
  Re-read one failing row, its guidance, paired rows, and current generic
  Context render path.
BASELINE
  Run that exact row and preserve its named 60x20 diagnostic.
DECIDE
  Record every required decision field below. Unknown means stop.
IMPLEMENT
  Verify the target is in the two-file write set. Change one reusable
  presentation/composition responsibility.
FOCUSED VALIDATION
  Run all three composition reds and both shared-owner preservation rows
  independently after every change.
CLASSIFY
  If red, name the remaining cause and repeat. Unexpected green requires a
  false-green investigation before continuing.
PRESERVATION
  Run every registered UI9-C exact row and all neighboring targets below.
CANDIDATE GATE
  Run the signed v4 command with the separately supplied digest. On failure,
  classify and loop; never edit protected files or create a report file.
SELF-AUDIT
  Inspect the complete baseline delta first, then answer every shortcut.
IN-CHAT HANDOFF
  Paste the required implementer template into chat. Report CandidateGreen
  only when the signed gate and higher-priority proofs pass.
```

Before each edit record:

```text
Failing case:
Observed expected/actual:
Missing composition responsibility:
Authoritative structured source:
Reusable presentation owner:
Authorized target file:
Paired validation:
Behavior that must remain visible:
Stop-condition check:
```

## Required validation

Run the three starting reds and two v3 preservation rows independently after
each iteration. Before the signed gate, run every registered UI9-C row
independently, including category projection, construction/depletion, every
active state, passive action truth, bound-without-reducer, all final-composition
rows, duplicate-name focus, and shared-owner transport/coherence. No aggregate
command substitutes for an exact row.

Then run:

```text
cargo fmt --check
CARGO_INCREMENTAL=0 cargo test -p bd_tui --lib
CARGO_INCREMENTAL=0 cargo test -p bd_app --test phase6_input
CARGO_INCREMENTAL=0 cargo test -p bd_tui --test input_help
CARGO_INCREMENTAL=0 cargo test -p bd_test_support --test contract_registry
CARGO_INCREMENTAL=0 cargo test -p bd_test_support --test candidate_handoff
CARGO_INCREMENTAL=0 cargo test -p bd_test_support --test repository_governance
```

Finally run, replacing only the digest placeholder:

```text
CARGO_INCREMENTAL=0 bash scripts/test-gate.sh \
  --candidate-manifest testing/UI9-C-CONTEXT-CANDIDATE-HANDOFF-v4.ron \
  --manifest-sha256 <REVIEWER-SUPPLIED-V4-DIGEST>
```

## Stop conditions

Stop with `STATUS=NotComplete` for a changed starting diagnostic, protected
change, out-of-scope path, test/fixture repair, semantic segment deletion,
display-prose parsing, mixed-source output, hardcoding/filtering, duplicated
category/action logic, hidden action/reason, map or panel regression, gameplay
or UI9-D change, zero-test or skipped/substituted command, untraceable baseline
delta, or a green gate with any unresolved higher-priority violation.

## Shortcut checklist

Answer each `No` with evidence. Any `Yes`, `Unknown`, omission, or unsupported
answer means `NotComplete`.

1. Any fixture/case/profile/coordinate/content/probe/decoy hardcode or filter?
2. Any semantic detail/action/reason removed, hidden, truncated, or recovered elsewhere?
3. Any category, staffing, progress, or applicability fact rederived downstream?
4. Any formatted display prose parsed to recover structured meaning?
5. Any duplicated Context domain/action rule or category-specific renderer?
6. Any map-primacy, containment, focus, Chronicle, or neighboring-panel regression?
7. Any production simulation, input, assignment, recipe, resource, or time change?
8. Any exact, paired, neighboring, or signed command skipped, substituted, or zero-test?
9. Any pre-baseline work claimed, reverted, or changed outside the exact write set?
10. Any repository handoff/report/log/evidence file created by the implementation agent?
11. Any isolated projection/widget green while final composition remains incomplete?
12. Any CandidateGreen claim based on gate output while another rule is unproven?

## Required implementer in-chat handoff

Paste this at the end of every loop. Do not write it to the repository.

```text
IMPLEMENTER IN-CHAT HANDOFF
Role: implementation agent
Batch / iteration: UI9-C compact Context composition v4 / <number>
Status: CandidateGreen | NotComplete
Authority body / manifest / supplied digest:
Iteration objective and decision record:
Changes made this iteration:
Complete delta from sealed baseline, including untracked paths:
Focused and paired commands with measured outcomes:
Registered exact rows with measured outcomes:
Neighboring and workflow evidence:
Signed candidate gate and measured totals:
Protected-file / manifest integrity:
Shortcut checklist 1-12 with evidence-backed answers:
First remaining failure or stop condition:
Next legal action or reviewer decision needed:
```

Write `Not run` or `Unverified` for missing evidence. A zero-test invocation is
not a pass. Never report done, complete, VerifiedGreen, or ReviewedGreen.
