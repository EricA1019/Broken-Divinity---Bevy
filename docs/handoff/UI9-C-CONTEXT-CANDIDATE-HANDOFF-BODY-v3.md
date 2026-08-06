# UI9-C Shared Context Ownership Candidate Handoff — v3

Use `$authoritative-test-pipeline`. Implement and internally validate the
complete authorized batch in a loop until the signed gate reports
`STATUS=CandidateGreen` or a stop condition requires `STATUS=NotComplete`.
Do not stop after analysis, one focused green, or an ordinary Cargo pass.

The manifest digest is deliberately supplied separately. This protected body
owns all scope, priority, tests, stop conditions, and acceptance rules. A chat
launcher may supply the digest but cannot override this file.

## Priority

1. Preserve protected authority, the sealed baseline, and user work.
2. Obey every stop condition and the exact write set.
3. Fix both reds through one coherent shared Context projection.
4. Preserve all named green production and presentation behavior.
5. Run every exact, neighboring, and signed command.
6. Report measured evidence without promoting status.

A lower-priority green never waives a higher-priority violation.

## Read before editing

Read `AGENTS.md`, the complete `$authoritative-test-pipeline` skill, `GDD.md`,
`Kernel.md`, `docs/DECISIONS-TO-LOCK.md`, UI plan Sections 17.2–17.9, the latest
`VISUAL-CONTEXT-001` registry/evidence records, every registered test, and:

```text
testing/UI9-C-CONTEXT-BASELINE-v3.ron
testing/UI9-C-CONTEXT-CANDIDATE-HANDOFF-v3.ron
```

Do not edit this body, the baseline, manifest, tests, fixtures, observers,
plans, authority, registry, evidence, policy, gate code, or status. Do not
regenerate the separately supplied digest. An observer concern is a stop and
report, not permission to change tests.

## Exact production write set

You may modify only:

```text
crates/bd_tui/src/view_models.rs
```

This file contains pre-handoff work recorded by the baseline. Preserve and do
not claim that work. Any required change to another file is an immediate
`NotComplete` scope stop. Never reset, revert, delete, or broadly format user
work.

## Outcome

Preserve the valid real-logistics staffed-station projection. Repair the
downstream shared-owner seam so:

- `NearbyTarget.detail` reaches the Context view model with every semantic
  segment intact; separator normalization is allowed, semantic prefix parsing
  or segment deletion is not;
- the Context title follows authoritative `category` and `status`, never
  `worker`, `recipe`, `progress`, or another legacy parallel field;
- authoritative Unstaffed detail/status plus a poisoned staffed worker remains
  coherently Unstaffed in structured and final output;
- worker/recipe/progress decoys never enter final Context; and
- the same ordinary station, node, colonist, construction/depletion,
  active-state, action-truth, duplicate-target, 80x24, and 60x20 behavior stays
  green.

Do not parse formatted display prose to recover domain facts. Do not hardcode
probe strings, decoys, Basic Processing, Mara, Refine Water, terminal sizes,
or test case IDs. Do not filter forbidden literals. Do not add a second domain
projection or rebuild category/staffing facts from parallel fields.

UI9-D remains forbidden: no binding, Context reducer/menu, input reroute,
enabled Set Production, simulation, assignment, recipe, movement, resource, or
time changes.

## Starting red reproduction

Run independently before editing:

```text
CARGO_INCREMENTAL=0 cargo test -p bd_tui --lib ui_development_contract_tests::context_view_model_transports_shared_detail_without_semantic_parsing -- --exact --nocapture
CARGO_INCREMENTAL=0 cargo test -p bd_tui --lib ui_development_contract_tests::final_context_consumes_the_shared_detail_projection_once -- --exact --nocapture
```

Expected first diagnostic: complete normalized detail is
`Station Unstaffed Operational Shared Detail Probe`, while actual transport
drops `Station Unstaffed`. Expected second diagnostic: final title is
`Context · Station Unstaffed`, while actual is `Context · Station Staffed`.
If either test compiles differently, passes, or fails for another cause, stop
before editing.

## Required loop

Repeat:

```text
READ
  Read one red, its influence table, paired case, and current consumer path.
REPRODUCE
  Run that exact test and confirm its named expected/actual diagnostic.
DECIDE
  Record all required decision fields below. Unknown means stop.
IMPLEMENT
  Change one reusable responsibility in view_models.rs only.
FOCUSED VALIDATION
  Run both red cases independently after every change.
CLASSIFY
  If red, name the remaining cause and repeat. Unexpected green requires a
  false-green investigation, not immediate continuation.
PRESERVATION
  Run every required exact case and neighboring target below.
CANDIDATE GATE
  Run the signed v3 command with the separately supplied digest. On failure,
  classify and loop; never edit protected files or status.
SELF-AUDIT
  Compare changed paths against the v3 baseline and answer all shortcuts.
HANDOFF
  Report CandidateGreen only when the signed gate says CandidateGreen and
  every higher-priority proof passes; otherwise report NotComplete.
```

Before each edit record:

```text
Failing case:
Observed expected/actual:
Authoritative source:
Poisoned competitors:
Derived outputs:
Mixed-source result to reject:
Missing responsibility:
Reusable owner:
Authorized target file:
Paired validation:
Stop-condition check:
```

## Required validation

Run the two corrective tests independently, then all registered UI9-C tests
independently. At minimum rerun every active projection/action row, the five
active final-composition rows, default station/node/colonist rows,
construction/depletion, passive action truth, duplicate-name focus, and the
two shared-owner rows. No aggregate command substitutes for an exact row.

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

Finally run, replacing only the digest placeholder with the separately supplied
v3 digest:

```text
CARGO_INCREMENTAL=0 bash scripts/test-gate.sh \
  --candidate-manifest testing/UI9-C-CONTEXT-CANDIDATE-HANDOFF-v3.ron \
  --manifest-sha256 <REVIEWER-SUPPLIED-V3-DIGEST>
```

## Stop conditions

Stop with `STATUS=NotComplete` for a changed starting diagnostic, protected
change, out-of-scope path, test/fixture repair, semantic segment deletion,
display-prose parsing, mixed-source output, hardcoding/filtering, gameplay or
UI9-D change, skipped command, untraceable baseline delta, or a green gate with
any unresolved higher-priority violation.

## Shortcut checklist

Answer each `No` with evidence. Any `Yes`, `Unknown`, omission, or unsupported
answer means NotComplete.

1. Any fixture/case/profile/content/probe/decoy hardcode or literal filter?
2. Any semantic segment removed from shared detail and recovered elsewhere?
3. Any category/staffing fact derived from worker/recipe/progress competitors?
4. Any formatted display prose parsed to recover structured meaning?
5. Any duplicated Context domain or action-applicability rule?
6. Any production simulation, input, assignment, recipe, resource, or time change?
7. Any exact, paired, neighboring, or signed command skipped or substituted?
8. Any projection green while final title/body remains mixed-source?
9. Any pre-baseline work claimed, repaired, reverted, or changed out of scope?
10. Any CandidateGreen claim based on gate output while another rule is unproven?

## Report

```text
Status: CandidateGreen | NotComplete
Batch: UI9-C shared Context ownership v3
Starting red reproduction:
Transport result:
Final coherence result:
Registered UI9-C exact results:
Neighbors:
Signed gate and totals:
Manifest/baseline integrity:
Production delta from baseline:
Decision records:
Shortcut checklist 1-10:
DRY/false-green audit:
Remaining blockers:
```

Never report done, complete, VerifiedGreen, or ReviewedGreen.
