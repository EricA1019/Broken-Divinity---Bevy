# UI9-C Context Corrective Candidate Handoff Prompt

> **WITHDRAWN — DO NOT EXECUTE.** Independent review found observer and scope
> defects. Editing this protected prompt invalidates the prior sealed handoff;
> the manifest and digest below are historical only. A reviewer must repair the
> production-reachability evidence and issue a new manifest and digest before
> another implementation run.

Use `$authoritative-test-pipeline` and carry out the complete corrective UI9-C
batch only after a reviewer replaces this withdrawn handoff. Apply the stop
conditions before entering the bounded red-to-green loop. A stop condition
requires `STATUS=NotComplete` even if a test or gate later prints green.
Do not merely inspect or propose a patch: implement and internally validate the
authorized production changes.

## Authority and sealed handoff

Read, in order: `AGENTS.md`, the global `$authoritative-test-pipeline` skill,
`GDD.md`, `Kernel.md`, `docs/DECISIONS-TO-LOCK.md`, UI plan Sections 17.2-17.9,
the `VISUAL-CONTEXT-001` registry record, its latest evidence entry, and every
registered primary/supporting test.

The withdrawn reviewer-owned manifest was:

```text
testing/UI9-C-CONTEXT-CANDIDATE-HANDOFF.ron
```

Its withdrawn reviewer-supplied SHA-256 digest was:

```text
2692945aba7f8943a681cd7d679698b42ece35273c79fd9cc7d1d85b42ecf330
```

Do not regenerate that digest, rewrite the manifest, or edit any protected
file. Tests, fixtures, observers, plans, authority, the registry, requirement
map, visual matrix, evidence, gate implementation, and status are read-only.
If a protected file appears defective, stop and report the exact independently
reproduced observer defect; do not repair it in this implementation run.

## Authorized production outcome

Complete the existing reusable Context projection and final composition. One
generic structured target/detail/action model must serve stations, resource
nodes, colonists, and construction sites. Domain/category adapters populate
authoritative facts; the renderer only composes those facts. Do not parse
existing display prose, query ECS state from rendering, or create separate
station/node/colonist panels.

`One shared model` means each domain-detail and action-applicability rule has
one semantic owner. Producing identical wording independently in proximity,
view-model, and screen code is still duplication. Downstream code may select,
order, wrap, and style the shared representation; it may not rebuild station,
node, or colonist facts from parallel fields.

The completed behavior must provide:

- staffed station: named worker, active recipe, and current/required progress;
- assigned resource node: named worker and current/required gather progress;
- assigned colonist: authoritative activity, named target, and progress;
- carrying colonist: destination plus human-labeled cargo and amount;
- blocked colonist: blocked activity, target where available, and actionable
  blocker reason;
- the same facts in final Context composition at 80x24 and 60x20; and
- action truth that keeps Interact disabled when a key is configured but no
  active Context reducer route exists.

Preserve the green default category, construction/depletion, unbound-preview,
duplicate-target, Chronicle aggregation/re-entry, Supplies, map-primary, HP/AP,
worker/day, compact-layout, and disabled `Set Production — Coming later`
behavior.

UI9-D is not authorized. Do not choose or ship an Interact binding, add a
Context menu reducer, reroute `e` or `c`, enable Set Production, or change
simulation, assignments, recipes, resources, time, or worker movement.

## Exact production write set

This is an exhaustive allowlist, not guidance. The implementation agent may
modify only these production files:

```text
crates/bd_core/src/colony/proximity.rs
crates/bd_tui/src/view_models.rs
crates/bd_tui/src/screens.rs
crates/bd_tui/src/commands.rs
```

Before every edit, compare the target path with this list. After every loop
iteration, inspect the complete changed-file list. If correctness requires any
other production file, including worker movement or simulation code, do not
edit it. Stop and report `STATUS=NotComplete`, the blocked test, required path,
conflicting constraint, and observed failure. Do not request retroactive scope.

The replacement handoff must include a reviewer-sealed baseline changed-path
inventory because this worktree is dirty. Compare candidate changes against
that baseline. Preserve and do not claim pre-existing work; do not modify or
revert an unauthorized path to make the audit pass. If the author of a diff
cannot be determined, stop and return it to the reviewer.

Never change production behavior to preserve a value directly injected by a
fixture. If a normal production update replaces an injected activity, job,
projection, or other decisive state, stop and report an observer defect unless
a separate production-input test proves that same state is reachable.

## Execution priority — apply in this order

1. Preserve protected authority, the sealed baseline, and user work.
2. Obey every stop condition and the exact production write set.
3. Satisfy `VISUAL-CONTEXT-001` through its real production cause, shared
   structured projection, and final composition.
4. Preserve the named green neighboring behavior.
5. Run every exact, paired, neighboring, and signed candidate command.
6. Report measured evidence without upgrading its meaning.

A lower item never waives a higher item. In particular, passing tests and a
green gate cannot excuse a shortcut, out-of-scope edit, or missing production
path.

Before each edit, establish this decision record internally and include it in
the final report:

```text
Failing case:
Missing responsibility: production cause | structured projection | composition | action truth
Reusable owner:
Authorized target file:
Paired validation to run next:
Stop-condition check: clear | stop with reason
```

If any field is unknown or guessed, stop with `STATUS=NotComplete`. Change one
production responsibility per iteration; do not batch speculative fixes.

## Required loop

Repeat this loop; do not skip directly to the aggregate gate:

```text
READ
  Re-read the next failing case, its guidance, and the current production seam.
BASELINE
  Run that one exact test and preserve its current named failure.
DIAGNOSE
  Identify the missing authoritative query, structured projection fact,
  composition seam, or action-state distinction.
IMPLEMENT
  Before editing, verify the path is in the exact write set. Make the smallest
  reusable production change there.
FOCUSED VALIDATION
  Rerun the exact case and its paired final-composition or action-truth case.
CLASSIFY
  If red, classify the new failure and repeat from DIAGNOSE.
  If green for a shortcut or unexpected reason, stop and report it; protected
  tests may not be changed.
PRESERVATION VALIDATION
  Run the existing category/state/action/focus tests and all four proximity
  workflows. Repair production regressions only inside the exact write set;
  otherwise stop and report the required scope expansion.
CANDIDATE VALIDATION
  Run the signed candidate gate below. If it fails, classify the failure and
  return to the smallest responsible step. Never edit status to satisfy it.
SELF-AUDIT
  Inspect the complete changed-file list and production diff. Any path outside
  the exact write set makes the status NotComplete even if the gate is green.
  Also reject hardcoded fixture data, duplicated category renderers/rules,
  display-prose parsing, enabled no-route actions, flattened targets, or
  unauthorized UI9-D behavior.
SHORTCUT REJECTION
  Answer every binary challenge below from the current diff and test evidence.
  Any Yes or Unknown makes the status NotComplete. Do not continue editing to
  conceal the finding.
HANDOFF
  Report CandidateGreen only when the signed gate says CandidateGreen, the
  changed-file audit passes, and no stop condition occurred.
```

Run each new projection/action case independently:

```text
cargo test -p bd_tui --lib ui_development_contract_tests::staffed_station_context_includes_worker_recipe_and_progress -- --exact --nocapture
cargo test -p bd_tui --lib ui_development_contract_tests::assigned_node_context_includes_worker_and_progress -- --exact --nocapture
cargo test -p bd_tui --lib ui_development_contract_tests::assigned_colonist_context_includes_target_and_progress -- --exact --nocapture
cargo test -p bd_tui --lib ui_development_contract_tests::carrying_colonist_context_includes_target_and_cargo -- --exact --nocapture
cargo test -p bd_tui --lib ui_development_contract_tests::blocked_colonist_context_includes_target_and_reason -- --exact --nocapture
cargo test -p bd_tui --lib ui_development_contract_tests::a_binding_without_a_context_reducer_does_not_enable_interact -- --exact --nocapture
```

Run each paired final-composition case independently:

```text
cargo test -p bd_tui --lib ui_development_contract_tests::staffed_station_recipe_progress_survives_final_composition -- --exact --nocapture
cargo test -p bd_tui --lib ui_development_contract_tests::assigned_node_worker_progress_survives_final_composition -- --exact --nocapture
cargo test -p bd_tui --lib ui_development_contract_tests::assigned_colonist_target_progress_survives_final_composition -- --exact --nocapture
cargo test -p bd_tui --lib ui_development_contract_tests::carrying_colonist_target_cargo_survives_final_composition -- --exact --nocapture
cargo test -p bd_tui --lib ui_development_contract_tests::blocked_colonist_reason_survives_final_composition -- --exact --nocapture
```

After the exact cases pass, run these neighboring targets:

```text
cargo test -p bd_tui --lib
cargo test -p bd_app --test phase6_input
cargo test -p bd_tui --test input_help
```

After a reviewer issues a replacement signature, run that handoff's authorized
closing gate. Do not reuse this withdrawn command:

```text
bash scripts/test-gate.sh \
  --candidate-manifest testing/UI9-C-CONTEXT-CANDIDATE-HANDOFF.ron \
  --manifest-sha256 2692945aba7f8943a681cd7d679698b42ece35273c79fd9cc7d1d85b42ecf330
```

Do not run the argument-free canonical gate as an acceptance substitute. Do
not change `VISUAL-CONTEXT-001` from `Red`; independent review owns promotion.

## Mandatory shortcut-rejection checklist

Answer each item `No` with one concise evidence reference. `Yes`, `Unknown`, a
missing answer, or an unsupported `No` requires `STATUS=NotComplete`.

1. Does the diff hardcode a named fixture, case ID, terminal profile,
   coordinate, worker, station, recipe, or resource to satisfy the tests?
   A valid `No` must cite alternate-data/adversarial evidence or a typed,
   data-driven boundary; merely not seeing the literal is insufficient.
2. Does more than one consumer independently rebuild the same Context domain
   detail or action-applicability rule?
3. Does any layer parse rendered/display prose to recover structured facts?
4. Does production preserve directly injected fixture state that a normal
   update would otherwise replace?
5. Does the diff change simulation, movement, assignments, recipes, resources,
   or time to satisfy a UI projection/composition test?
6. Was any exact case, paired final-composition case, neighboring target, or
   signed gate skipped, substituted, filtered, or treated as optional?
7. Could the projection tests pass while final Context composition receives
   independently reconstructed facts instead of the shared projection?
8. Is any action shown enabled without both a configured binding and an active
   reducer route?
9. Does the candidate claim, repair, or revert pre-handoff work instead of
   comparing against the sealed baseline?
10. Is `CandidateGreen` being claimed solely because the gate printed it while
    any higher-priority rule remains violated or unproven?

## Final response

Report:

```text
Status: CandidateGreen | NotComplete
Plan batch: UI9-C active-state corrective handoff
Focused projection/action cases:
Paired final-composition cases:
Neighboring targets:
Signed candidate gate and measured totals:
Manifest/protected-file integrity:
Production files changed:
Exact write-set audit:
Per-iteration decision records:
Shortcut-rejection checklist (1-10 with evidence):
DRY and false-green self-audit:
Remaining blockers:
```

Do not use “done,” “complete,” `VerifiedGreen`, or `ReviewedGreen`.
