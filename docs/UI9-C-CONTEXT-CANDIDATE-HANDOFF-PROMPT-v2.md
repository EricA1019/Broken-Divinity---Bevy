# UI9-C Context Corrective Candidate Handoff Prompt — v2

> **WITHDRAWN — DO NOT EXECUTE.** Independent review found that the v2
> adversarial observer allowed display-prose parsing and a mixed-source staffed
> title to pass. Its manifest, baseline, and digest are historical only. Wait
> for a reviewer-issued v3 handoff.

Use `$authoritative-test-pipeline`. Carry out the complete authorized UI9-C
production batch, repeatedly testing and correcting it until the signed gate
reports `STATUS=CandidateGreen` or a stop condition makes that result
impossible. Do not stop after analysis, after one green test, or after an
ordinary Cargo command passes.

The prior prompt and manifest and these v2 artifacts are withdrawn historical
evidence. Do not run this implementation handoff.

## Priority order

Higher items always override lower ones:

1. Preserve protected authority, the sealed dirty-worktree baseline, and all
   pre-existing user work.
2. Obey the exhaustive production write set and every stop condition.
3. Fix each red through its production responsibility and the one shared
   structured Context projection.
4. Preserve all currently green UI9-C, proximity, input, Help, and layout
   behavior.
5. Run every required command; an aggregate pass never substitutes for a
   skipped exact case.
6. Report measured candidate evidence without promoting registry status.

A lower-priority success cannot excuse a higher-priority violation. In
particular, a green test or gate does not excuse an out-of-scope edit,
hardcoding, duplicated domain composition, or modification of protected work.

## Read before editing

Read in order:

1. `AGENTS.md` and the complete `$authoritative-test-pipeline` skill;
2. `GDD.md`, `Kernel.md`, and `docs/DECISIONS-TO-LOCK.md`;
3. Sections 17.2 through 17.9 of
   `docs/FOUNDATION-UI-IMPROVEMENT-PLAN.md`;
4. the `VISUAL-CONTEXT-001` record in
   `testing/foundation-contracts.ron` and its latest evidence entry;
5. every registered primary/supporting test; and
6. the baseline and manifest below.

Reviewer-sealed baseline:

```text
testing/UI9-C-CONTEXT-BASELINE-v2.ron
```

Reviewer-owned manifest:

```text
testing/UI9-C-CONTEXT-CANDIDATE-HANDOFF-v2.ron
```

Reviewer-supplied manifest SHA-256:

```text
2fbfdbe8de3a2db3fa94be22c3cd46289b6eb067143d614f080b698686017cc9
```

Do not rewrite either artifact, regenerate the digest, edit a protected file,
or update `VISUAL-CONTEXT-001` from `Red`. If any protected test or observer
appears wrong, reproduce the defect independently and stop with
`STATUS=NotComplete`; this implementation run does not authorize test repair.

## Exact production write set

This is an exhaustive allowlist. You may modify only:

```text
crates/bd_core/src/colony/proximity.rs
crates/bd_tui/src/view_models.rs
crates/bd_tui/src/screens.rs
crates/bd_tui/src/commands.rs
```

These files already contain pre-handoff work. The baseline hashes distinguish
that work from your candidate edits. Preserve it and report only your delta.
All other production files are forbidden. In particular,
`crates/bd_core/src/colony/survivors.rs` is protected pre-existing work: do not
edit, revert, or claim it.

Before every edit, compare the path literally with the allowlist. After every
iteration, inspect all changed paths against the baseline. If a required fix
needs another file, stop and report the failing test, missing responsibility,
required path, and constraint; do not expand scope yourself. Never use reset,
checkout, deletion, or broad formatting to conceal or discard existing work.

## Required outcome

One generic structured target/detail/action model must serve stations, nodes,
colonists, and construction sites. Category adapters may populate typed,
authoritative facts. Downstream view-model and screen code may select, order,
wrap, and style those facts, but may not independently rebuild the same domain
detail or applicability rule from parallel fields.

Complete these currently missing responsibilities:

- a real Refine Water logistics job at `ReadyToRefine 1/2` makes Basic
  Processing project as Staffed with Mara, recipe, and progress;
- those same shared facts survive final Context composition at 80x24 and
  60x20; and
- final Context consumes the authoritative shared `detail` projection once.
  In the adversarial seam, `Shared Detail Probe` must appear and the forbidden
  parallel worker, recipe, and `99/99` decoys must not.

The shared-detail probe is an observer, not production data. Do not hardcode
its phrase, case, or forbidden values; do not filter decoy strings; and do not
append shared detail beside the duplicated reconstruction. Remove the
independent reconstruction so ordinary data and the probe travel through the
same production seam.

Preserve production-reachable assigned-node and active-colonist detail,
construction/depletion, default category rows, duplicate-name focus,
Chronicle range entry/re-entry, disabled action reasons, map primacy, Supplies,
HP/AP, worker/day information, compact containment, and
`Set Production — Coming later`.

UI9-D is forbidden. Do not add or choose an Interact binding, implement a
Context reducer/menu workflow, reroute `c` or `e`, enable Set Production, or
change simulation, movement, assignments, recipes, resources, or time.

## Baseline facts you must reproduce

Before editing, run these three exact tests independently. Each must fail for
the named production responsibility, not from compilation or fixture setup:

```text
cargo test -p bd_tui --lib ui_development_contract_tests::staffed_station_context_includes_worker_recipe_and_progress -- --exact --nocapture
cargo test -p bd_tui --lib ui_development_contract_tests::staffed_station_recipe_progress_survives_final_composition -- --exact --nocapture
cargo test -p bd_tui --lib ui_development_contract_tests::final_context_consumes_the_shared_detail_projection_once -- --exact --nocapture
```

Expected reviewed baseline: the whole `bd_tui` library has 111 tests, 108
passed, 3 failed, and 0 ignored. The staffed projection is Unstaffed despite a
real production-created logistics job; its final Context is also Unstaffed;
the shared-detail seam renders the parallel decoys instead of the shared probe.
If your baseline differs, stop and report it before editing.

## Mandatory red-to-candidate-green loop

Repeat this loop until the entire batch is candidate green:

```text
READ
  Read one failing assertion, its guidance, the production seam, and its paired
  case. Do not infer the requirement from the test name alone.
REPRODUCE
  Run that exact test and confirm its named expected/actual diagnostic.
DECIDE
  Record all six fields below. Unknown or guessed means stop.
IMPLEMENT
  Change one reusable production responsibility in one allowed file.
FOCUSED VALIDATION
  Rerun the exact case, its pair, and the shared-owner seam when projection or
  composition ownership changed.
CLASSIFY
  If red, identify the next missing responsibility and repeat. If green for an
  unexpected reason, stop and investigate the false green.
PRESERVATION
  Run all exact UI9-C cases, then the neighboring suites below. Repair only
  inside the allowlist; otherwise stop.
CANDIDATE GATE
  Run the signed v2 gate exactly. On failure, classify and loop; never edit
  tests, ledgers, evidence, the manifest, or the baseline to make it pass.
SELF-AUDIT
  Compare every path and allowed-file before/after content against the sealed
  baseline. Answer all shortcut questions with evidence.
HANDOFF
  Report CandidateGreen only if the signed gate prints CandidateGreen and all
  higher-priority checks pass. Otherwise report NotComplete.
```

Before each production edit, write this decision record:

```text
Failing case:
Observed expected/actual diagnostic:
Missing responsibility: production cause | structured projection | composition | action truth
Reusable owner:
Authorized target file:
Paired validation:
Stop-condition check: clear | stop with reason
```

## Exact focused validation

Run every command independently; do not replace them with a name filter.

Projection/action cases:

```text
cargo test -p bd_tui --lib ui_development_contract_tests::staffed_station_context_includes_worker_recipe_and_progress -- --exact --nocapture
cargo test -p bd_tui --lib ui_development_contract_tests::assigned_node_context_includes_worker_and_progress -- --exact --nocapture
cargo test -p bd_tui --lib ui_development_contract_tests::assigned_colonist_context_includes_target_and_progress -- --exact --nocapture
cargo test -p bd_tui --lib ui_development_contract_tests::carrying_colonist_context_includes_target_and_cargo -- --exact --nocapture
cargo test -p bd_tui --lib ui_development_contract_tests::blocked_colonist_context_includes_target_and_reason -- --exact --nocapture
cargo test -p bd_tui --lib ui_development_contract_tests::a_binding_without_a_context_reducer_does_not_enable_interact -- --exact --nocapture
```

Shared-owner seam:

```text
cargo test -p bd_tui --lib ui_development_contract_tests::final_context_consumes_the_shared_detail_projection_once -- --exact --nocapture
```

Paired final-composition cases:

```text
cargo test -p bd_tui --lib ui_development_contract_tests::staffed_station_recipe_progress_survives_final_composition -- --exact --nocapture
cargo test -p bd_tui --lib ui_development_contract_tests::assigned_node_worker_progress_survives_final_composition -- --exact --nocapture
cargo test -p bd_tui --lib ui_development_contract_tests::assigned_colonist_target_progress_survives_final_composition -- --exact --nocapture
cargo test -p bd_tui --lib ui_development_contract_tests::carrying_colonist_target_cargo_survives_final_composition -- --exact --nocapture
cargo test -p bd_tui --lib ui_development_contract_tests::blocked_colonist_reason_survives_final_composition -- --exact --nocapture
```

After all twelve pass, run:

```text
cargo fmt --check
cargo test -p bd_tui --lib
cargo test -p bd_app --test phase6_input
cargo test -p bd_tui --test input_help
cargo test -p bd_test_support --test contract_registry
cargo test -p bd_test_support --test candidate_handoff
cargo test -p bd_test_support --test repository_governance
```

Then run the only authorized closing gate:

```text
CARGO_INCREMENTAL=0 bash scripts/test-gate.sh \
  --candidate-manifest testing/UI9-C-CONTEXT-CANDIDATE-HANDOFF-v2.ron \
  --manifest-sha256 2fbfdbe8de3a2db3fa94be22c3cd46289b6eb067143d614f080b698686017cc9
```

Do not use the withdrawn signature. Do not use the argument-free canonical
gate as a substitute. The signed gate must say `STATUS=CandidateGreen`.

## Stop conditions

Stop with `STATUS=NotComplete` when any of these is true:

- the starting red diagnostic differs materially from the reviewed baseline;
- a protected file or baseline identity changed;
- correctness requires a path outside the exact write set;
- a normal production update cannot reach or preserve the decisive state;
- a test goes green through hardcoding, filtering probe values, deleted detail,
  weakened output, or a skipped integration seam;
- simulation/gameplay/input behavior must change for this UI9-C task;
- any required exact, neighboring, or signed-gate command is skipped;
- the candidate cannot distinguish its edits from pre-existing work; or
- the signed gate is green but any higher-priority rule is violated or unknown.

## Mandatory shortcut-rejection checklist

Answer every item `No` with a concrete diff or test reference. `Yes`,
`Unknown`, a missing answer, or an unsupported `No` means NotComplete.

1. Does the diff hardcode a fixture name, recipe, worker, case ID, coordinate,
   profile, probe phrase, forbidden decoy, or expected progress value?
2. Does more than one consumer rebuild the same Context domain detail or
   action-applicability rule?
3. Does any layer parse display prose to recover structured facts?
4. Does production preserve directly injected fixture state that a normal
   workflow would replace?
5. Does the diff change simulation, movement, assignments, recipes, resources,
   time, or input routing to satisfy a presentation test?
6. Was any exact case, pair, neighboring suite, or signed gate skipped,
   substituted, filtered, or treated as optional?
7. Could projection pass while final Context still receives reconstructed
   parallel facts instead of the shared detail projection?
8. Is any action enabled without both a configured binding and an active
   reducer route?
9. Does the candidate claim, edit, revert, or repair pre-baseline work outside
   its exact allowed-file delta?
10. Is CandidateGreen claimed solely because a command printed green while a
    stop condition or higher-priority proof remains unresolved?

## Final report format

```text
Status: CandidateGreen | NotComplete
Plan batch: UI9-C context corrective v2
Starting red reproduction:
Focused projection/action results:
Shared-owner seam result:
Paired final-composition results:
Neighboring results:
Signed candidate gate and measured totals:
Manifest and baseline integrity:
Production files changed, with baseline-to-candidate attribution:
Exact write-set audit:
Per-iteration decision records:
Shortcut checklist 1-10 with evidence:
DRY/false-green audit:
Remaining blockers:
```

Do not report “done,” “complete,” `VerifiedGreen`, or `ReviewedGreen`.
