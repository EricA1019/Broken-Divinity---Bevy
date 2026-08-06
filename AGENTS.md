# Broken Divinity Development Contract

These instructions apply to every change in this repository.

## Authority

Before changing behavior, read the relevant sections of:

1. [`GDD.md`](GDD.md) — product intent and player experience;
2. [`Kernel.md`](Kernel.md) — technical architecture;
3. [`docs/authority/DECISIONS-TO-LOCK.md`](docs/authority/DECISIONS-TO-LOCK.md) — locked scope and behavior;
4. [`docs/authority/AUTHORITATIVE-TESTING-STANDARD-AND-MIGRATION-PLAN.md`](docs/authority/AUTHORITATIVE-TESTING-STANDARD-AND-MIGRATION-PLAN.md) — test design, evidence, metrics, and execution protocol;
5. [`testing/foundation-contracts.ron`](testing/foundation-contracts.ron) — machine-readable contract ownership;
6. the current implementation and tests.

For test planning, writing, strengthening, review, or red-to-green handoff,
also load and follow the project-agnostic global skill
`$authoritative-test-pipeline`. This repository's authority documents supply
the Broken Divinity-specific requirements; the skill supplies the reusable
plan → red test → bounded implementation loop → candidate-green review
pipeline.

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
6. Run the role-appropriate development gate. An implementation agent with a
   signed handoff runs candidate mode; an independent reviewer runs the
   canonical gate.
7. Compare the result to the GDD and update contract/evidence records when
   their status changed.

Do not write production behavior first and backfill tests afterward. Refactors
must begin with green characterization tests unless existing authoritative
tests already cover the boundary.

### Role-separated red-to-green handoff

Before delegating intentional-red implementation, the test author or reviewer
creates a versioned candidate-handoff manifest, hashes it, and gives the
implementation agent both the manifest path and the SHA-256 digest. The
manifest names the exact contracts owned by the batch, the protected sealed
baseline, and the exact production write set, and hashes every read-only file,
including the active plan, primary/supporting tests, fixtures, observers, and
all core authority/status files required by the gate.

The exact active implementation instructions must be a digest-free protected
file in that manifest. Supply the manifest digest separately so the protected
instructions do not contain their own hash. A small launcher may carry the
digest, but it may not redefine scope, priority, stop conditions, tests, or
acceptance. Protecting an obsolete prompt while leaving the active instruction
body mutable invalidates the handoff.

The implementation agent must not rewrite the manifest, regenerate its digest,
edit a protected file, or promote registry, matrix, requirement-map, or
evidence status. It runs:

```text
bash scripts/test-gate.sh \
  --candidate-manifest <author-supplied-path> \
  --manifest-sha256 <author-supplied-digest>
```

Candidate mode succeeds only when all automated gates pass, every protected
hash is unchanged, the complete Git-visible delta from the sealed baseline is
inside the exact write set, the manifest names exactly the remaining required
`Red` contracts, and those contracts remain `Red`. It reports only
`STATUS=CandidateGreen`. A changed or omitted protected file, manifest rewrite,
unauthorized added/untracked, modified, deleted, or renamed path,
self-promotion, unlisted required Red contract, or failed test is
`STATUS=NotComplete`.

Every delegated handoff must also name an **exact production write set**. Terms
such as `principally`, `expected scope`, `likely files`, or `and related files`
do not define authority and are forbidden in a write set. Before every edit,
the implementation agent checks the target path against that set; after every
iteration, it audits the complete changed-file list. A required edit outside
the set is a scope-expansion stop condition, not permission to make the edit.
The agent reports `STATUS=NotComplete` with the required path and reason and
waits for a newly authorized handoff.

Because the worktree may already be dirty, the author must seal a baseline
changed-path inventory sufficient to distinguish pre-handoff state from the
candidate's edits. Scope validation compares against that baseline, not against
an assumed clean checkout. The implementation agent preserves pre-existing
user work, does not claim it as candidate evidence, and does not modify or
revert an unauthorized path merely to make the audit pass. If attribution is
uncertain, stop and return the diff to the reviewer for disposition.

The stop conditions outrank the red-to-green loop and every gate result. An
implementation agent may report `CandidateGreen` only when the signed gate
passes **and** every production change is inside the exact write set. If a gate
prints `STATUS=CandidateGreen` after an unauthorized change, the agent must
still report `STATUS=NotComplete`; a test command cannot grant write authority.
Stopping on a scope conflict, observer defect, missing product decision, or
forbidden behavior change is correct adherence, not failure to carry out the
task.

Handoffs intended for constrained or smaller implementation models must put the
priority order and shortcut checks before the implementation loop. Use this
order: protect authority and user work; obey stop conditions and exact scope;
satisfy the semantic contract through its authorized production path; preserve
neighboring behavior; run every required gate; report evidence. A later item
never overrides an earlier one.

Before each edit, the implementation agent records the failing case, missing
cause/projection/composition responsibility, reusable owner, authorized target
file, and paired validation it will run. If it cannot name all five without
guessing, it stops. Change one production responsibility per iteration.

Before claiming candidate green, answer every handoff-specific shortcut
challenge. Any `yes` or `unknown` is `STATUS=NotComplete`, including: a
fixture/case/profile hardcode; duplicated shared rules; parsing display prose;
preserving injected state; changing unrelated simulation/domain behavior;
skipping a required exact or paired test; satisfying only an isolated layer;
claiming pre-existing work; or relying on gate output to waive scope.

At the end of every implementation loop, paste a compact implementation
handoff into the current chat. Do not create or modify a repository handoff,
bug-report, evidence, or log file unless its exact path is in the authorized
write set. The in-chat handoff must state the role and batch, exact status,
iteration objective, production changes attributable to the iteration,
complete delta from the sealed baseline including untracked paths, commands
actually run with measured outcomes, first remaining failure, shortcut answers,
stop-condition result, and the next legal action or reviewer decision needed.
Use `Not run` or `Unverified` instead of implying missing evidence passed; a
zero-test invocation is not a pass.

The independent reviewer replies with a matching in-chat continuation handoff
that states what evidence was reproduced, what was accepted or rejected,
scope/baseline disposition, current authoritative status, unresolved blocker
or false-green finding, and the exact next authorized batch. If scope,
authority, tests, or protected instructions change, the reviewer must issue a
new protected body, baseline, manifest, and separately supplied digest rather
than telling the implementation agent to resume under the old seal.

For a DRY/shared-owner claim, define the owned semantic transformation. Shared
does not mean that multiple consumers independently produce identical text or
branches. Domain facts and applicability are derived once; downstream layers
may arrange, wrap, style, or serialize that representation but may not rederive
the same rule. A hardcode-rejection answer needs alternate-data, adversarial,
or typed-boundary evidence; the absence of an obvious fixture literal in the
diff is not sufficient.

After auditing the production diff, false-green challenges, final workflow,
and required visual/PTY evidence, the independent reviewer updates all status
ledgers together and runs `bash scripts/test-gate.sh` without candidate
arguments. Only that canonical mode may report `STATUS=VerifiedGreen`; it is
still not owner acceptance or `ReviewedGreen`.

## Test quality rules

- One primary test owns one contract. Avoid broad tests with overlapping
  responsibility.
- Use stable names, content IDs, positions, and normalized fingerprints. Never
  rely on raw entity IDs or ECS iteration order.
- Assert exact state deltas and forbidden mutations for domain behavior.
- Test player workflows through production inputs and projections; direct
  world mutation is fixture setup only.
- Never change production behavior merely to preserve state directly injected
  by a fixture. If the next normal production update replaces that state, the
  fixture has not proved production reachability. Stop the implementation
  handoff and report an observer defect unless a separate production-path test
  creates the same decisive state through an authorized cause.
- For player-facing contracts, prove the chain from production cause to the
  decisive structured projection to final composition. A fixture that injects
  expected log, action, tooltip, or resource rows may support renderer
  coverage, but cannot be the sole evidence for production reachability.
- Turn every completion-critical `Invalid shortcuts` warning into an
  executable guard. Prefer production-path cases, adversarial seam poisoning,
  typed boundaries, or causal negative cases over source-text matching.
- Before writing an adversarial seam, map the authoritative source, every
  poisoned competitor, and every derived output that can consume either one.
  Poison competitors with plausible opposite semantics and assert that title,
  status, body, style/action state, and final composition all follow the same
  source. `expected marker present` plus `decoy literals absent` is not enough
  when an unasserted derivative can still follow the decoy. Split transport
  fidelity and mixed-source coherence into independently runnable tests when
  either failure could hide the other.
- If a forbidden technique such as display-prose parsing cannot be
  distinguished reliably through final behavior alone, require a typed
  boundary or a named production-diff audit in addition to the behavioral
  observer. Never describe sentinel poisoning as proof of an architectural
  prohibition it cannot actually observe.
- For edge-triggered behavior, test both the authorized cause and nearby
  non-causes such as initialization, idle projection, rendering, resize,
  restoration, or spawning when those seams exist. For rearmable edges, prove
  leaving is silent and later re-entry emits exactly once again.
- For advertised actions, distinguish domain applicability, configured/input
  reachability, and immediate executability. Every enabled action must agree
  with its current binding and reducer route and have production-input
  evidence; absent or owner-locked routes remain disabled with a truthful
  reason. A plausible label, `unbound`, invented hint, or key owned by another
  command is not evidence. Final composition must preserve disabled styling,
  denial reason, focus/selection, and controls at every required profile.
- Category/adaptor matrices require an authoritative state variant so fixed
  category prose cannot pass. Multi-target contracts require unlike-category
  aggregation plus a duplicate-display-name case proving deterministic focus,
  complete ordering, and player-visible disambiguation without raw ECS IDs.
- A substring alone does not prove visibility, readability, selection, layout,
  or discoverability.
- Claims such as closed, continuous, contained, all, or every must verify the
  complete owned structure in the final composed output; corners, endpoints,
  or glyph counts alone are insufficient.
- Bind visual style evidence to its semantic token or region, and give reusable
  visual observers a smallest-granularity negative mutation probe so unrelated
  cells cannot create a false green.
- A primary test intentionally handed off red must include concise
  `Implementation guidance` comments that identify the reusable ownership
  boundary, final integration seam, behavior to preserve, invalid shortcuts,
  and required closing evidence. Write constraints, not an unlocked private
  API, file layout, coordinate set, control-flow, or algorithm prescription;
  comments never replace executable assertions.
- After another agent reports green, audit the production diff and final
  workflow for removed neighboring behavior, fixture/profile special cases,
  duplicated shared rules, observer weakening, and final-composition or PTY
  gaps before changing contract status.
- Before accepting green, run a false-green challenge: determine whether the
  test could still pass if production never created the decisive data, a panel
  were merely renamed, prose were parsed to reconstruct state, an enabled row
  had no working input route, category detail were hardcoded, duplicate targets
  collapsed, or only the named fixture/category were implemented. Repair that
  evidence gap before promotion.
- Before a red handoff, run every new primary and completion-critical support
  test independently and record each intended failure. Every
  completion-critical table row/state variant must also be independently
  observable; a later assertion hidden behind an earlier panic is not validated
  red evidence.
- Tests must be deterministic: explicit fixtures, seeds, frame counts, and
  terminal profiles; no sleeps or wall-clock correctness checks.
- Failure output must identify the contract, case, precondition, action,
  expected/actual result, and useful state or visual context.
- Do not add a public production API solely for tests or copy production logic
  into expected-value helpers.
- Do not weaken, ignore, delete, reclassify, or replace a test merely to make a
  gate green. Follow the retirement rules in the authoritative testing
  standard.
- Production regressions discovered during a delegated loop may be repaired
  only inside the exact write set. Otherwise stop and request a new handoff;
  `repair regressions` never expands scope.

## Completion rules

A focused test passing is only focused green. `CandidateGreen` requires the
signed candidate gate when a role-separated handoff is active.
`VerifiedGreen` requires the neighboring target and a zero-exit canonical gate
run after independent review and status reconciliation on the current worktree.
`ReviewedGreen` additionally requires production-diff review, required
workflow/PTY evidence, authority drift review, and accurate registry/evidence
records. Only `ReviewedGreen` may be called complete or done.

A zero-exit test suite with any required contract still recorded `Red` is
status drift, not `VerifiedGreen`. Re-establish a real red failure or update the
record to the accurately reviewed non-Red state before the canonical gate may
verify the worktree.

No bug report, diagnostic label, `pre-existing` classification, conflict claim,
owner-question request, or partial test count waives a failed canonical gate.
Only an explicit owner instruction naming the exact waiver may do so. Before
claiming tests conflict, restate their semantic requirements and prove no output
can satisfy both; a parser, fixture, normalization, or glyph-handling limitation
is an observer defect, not a contract conflict.

Completion requires:

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
Never hide it in a summary or claim the repository is green. Every final test
handoff reports status, focused result, neighboring result, canonical gate,
workflow/PTY evidence, registry/evidence state, and remaining blockers.
