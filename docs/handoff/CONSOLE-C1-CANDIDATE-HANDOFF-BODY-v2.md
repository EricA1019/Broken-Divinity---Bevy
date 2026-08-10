# Console C1 Input Ownership Candidate Handoff — v2

Use `$authoritative-test-pipeline`. Execute this entire bounded loop until the
signed candidate gate reports `STATUS=CandidateGreen`, or stop immediately with
`STATUS=NotComplete` when any stop condition occurs. Do not merely inspect,
plan, or recommend changes.

This v2 body supersedes the rejected v1 candidate. Do not use, edit, regenerate,
or derive authority from the v1 baseline, manifest, digest, or report. The
separately supplied v2 manifest digest authenticates this protected body and
its reviewer-owned observers.

## Priority — later success never waives an earlier rule

1. Preserve protected authority, the v2 baseline, and pre-existing user work.
2. Obey every stop condition and the exact four-file production write set.
3. Make `ConsoleCommand` the causal source of dispatch; remove the reducer's
   parallel pending-queue write.
4. Add an explicit console-capture-before-gameplay schedule edge without
   suppressing Bevy ambiguities.
5. Preserve the six already-green physical input behaviors and editing rules.
6. Run every exact, paired, neighboring, and signed command as written.
7. Report measured candidate evidence without self-promoting contract status.

## Read completely before editing

```text
AGENTS.md
GDD.md
Kernel.md
docs/authority/DECISIONS-TO-LOCK.md
docs/authority/AUTHORITATIVE-TESTING-STANDARD-AND-MIGRATION-PLAN.md
docs/active/FOUNDATION-STABILIZATION-AND-CONSOLE-HARDENING-PLAN.md
crates/bd_app/tests/console_input_contract.rs
crates/bd_console/src/input.rs
crates/bd_console/src/lib.rs
crates/bd_console/src/state.rs
crates/bd_tui/src/lib.rs
testing/foundation-contracts.ron
testing/FOUNDATION-REQUIREMENT-MAP.md
testing/FOUNDATION-TEST-EVIDENCE.md
testing/CONSOLE-C1-BASELINE-v2.ron
testing/CONSOLE-C1-CANDIDATE-HANDOFF-v2.ron
```

Read the complete authoritative-test skill before taking an implementation
action. The plan, tests, fixtures, observers, registry, evidence, requirement
map, policies, gate, handoff body, baseline, and manifest are reviewer-owned.
Do not edit them or regenerate the supplied digest.

## Exact production write set

You may modify only these four repository-relative files:

```text
crates/bd_console/src/input.rs
crates/bd_console/src/lib.rs
crates/bd_console/src/state.rs
crates/bd_tui/src/lib.rs
```

No other production, test, documentation, manifest, evidence, report, or log
path is authorized. Before every edit, name the target and confirm it is in
this list. After every iteration, compare the complete Git-visible delta to the
v2 baseline. Formatting is permitted only when it changes these four files.

## Locked outcome

### One causal typed submission

- Physical Enter emits exactly one `ConsoleCommand` from the shared reducer.
- The reducer must not also write `ConsoleState.pending` or another dispatch
  transport.
- One production bridge may read `ConsoleCommand` and queue it for the existing
  exclusive dispatcher. Only that bridge may write the production pending
  queue.
- The bridge runs after physical Input and the v2 quarantine seam, and before
  `execute_console_command` in Mutation.
- One physical `help` line produces one typed message, one history entry, one
  help result, and an empty post-dispatch queue.
- The solution must work for arbitrary command text. Do not special-case
  `help`, the fixture, the audit resource, or test configuration.

The v2 primary is adversarial by design. It removes any Input-stage pending
content during `BdSet::IntentCollection`. A legitimate typed bridge recreates
the queue later from `ConsoleCommand`; the current parallel-write shortcut is
exposed as `legacy=["help"]` and produces no result.

### Explicit capture ordering

- The registered `bd_console` reducer remains the only editing owner.
- Console capture is explicitly ordered before gameplay routing. Membership in
  the same `BdSet::Input`, resource conflicts, tuple order, plugin insertion
  order, and a no-op guard are not ordering evidence.
- Do not use `ambiguous_with`, `ambiguous_with_all`, ignored scheduling
  ambiguities, Title/Outpost branches, or key-specific leakage exceptions.
- The v2 schedule observer must find exactly one registered reducer, no global
  ambiguity suppression for it, and no unresolved conflicts involving it.
- The paired physical close tests determine direction: ordering gameplay before
  capture remains Red because the close key leaks.

A direct function edge or reusable named system-set edge is valid. Preserve a
cross-project-quality boundary; do not create a test-only production API.

### Preserve existing behavior

- Press is the only mutating key kind; Repeat and Release remain inert.
- Same-batch order, backtick open/toggle, Escape, printable ASCII, cursor,
  Backspace, Enter, history search, and Tab completion remain owned once in
  `bd_console`.
- The TUI must not regain a second editing/submission match.
- Console-open batches remain unavailable to gameplay after a close/toggle.
- Console-closed ordinary gameplay input still routes exactly once.
- No command parsing, command output, simulation, debug mutation, renderer,
  factory, GodMode, or product behavior belongs to this batch.

## Sealed starting baseline

First authenticate the v2 manifest with the separately supplied digest. Then
run every case below independently. A compile failure, zero-test invocation,
different failure tuple, or unexpected result is a stop condition.

Intentional Red — exact diagnostics:

```text
cargo test --locked -p bd_app --test console_input_contract one_physical_line_reaches_dispatch_exactly_once -- --exact --nocapture
```

Expected: one test fails with
`typed=["help"], legacy=["help"], history=["help"], help_count=0`.

```text
cargo test --locked -p bd_app --test console_input_contract console_capture_is_explicitly_ordered_before_gameplay_routing -- --exact --nocapture
```

Expected: one test fails with one registered reducer, no global ambiguity
suppression, and exactly one unresolved conflict involving the reducer.

Preservation Green — run all six independently:

```text
cargo test --locked -p bd_app --test console_input_contract physical_console_editing_uses_the_registered_production_reducer -- --exact --nocapture
cargo test --locked -p bd_app --test console_input_contract escape_close_is_consumed_before_title_routing -- --exact --nocapture
cargo test --locked -p bd_app --test console_input_contract backtick_close_is_consumed_before_title_routing -- --exact --nocapture
cargo test --locked -p bd_app --test console_input_contract escape_close_does_not_quit_or_mutate_outpost -- --exact --nocapture
cargo test --locked -p bd_app --test console_input_contract backtick_close_does_not_reach_a_rebound_outpost_action -- --exact --nocapture
cargo test --locked -p bd_app --test console_input_contract closed_console_preserves_normal_gameplay_input -- --exact --nocapture
```

Each must execute exactly one test and pass.

## Mandatory bounded loop

Repeat until candidate green or a stop condition:

```text
READ
  Re-read this body, both failing test comments, the current production seam,
  and the complete baseline delta.
REPRODUCE
  Run the first remaining exact Red and preserve its full tuple.
DECIDE
  Fill every decision-record field below. Unknown means stop.
IMPLEMENT
  Confirm the target is one of the four paths. Change one production
  responsibility: typed bridge OR explicit ordering, not both speculatively.
FOCUSED VALIDATION
  Run both v2 Reds and all six preservation cases independently.
CLASSIFY
  If Red, name the remaining production responsibility and repeat.
  If unexpectedly green for the wrong reason, stop and report observer defect.
NEIGHBOR VALIDATION
  Run every exact neighbor command below.
CANDIDATE GATE
  Run the signed v2 candidate gate with the reviewer-supplied digest.
SELF-AUDIT
  Audit the complete baseline delta, production diff, typed ownership,
  schedule edges, and all shortcut questions.
HANDOFF
  Paste the required in-chat report. Do not create a repository report.
```

Before each edit, record:

```text
Failing contract and exact case:
Observed expected/actual tuple:
Missing production responsibility:
Reusable owner:
Integration seam and schedule stage:
Authorized target file:
Paired cases to rerun:
Behavior that must remain unchanged:
Scope/stop-condition check:
```

## Required closing commands

Run the eight focused cases independently first, then:

```text
cargo fmt --all -- --check
cargo test --locked -p bd_console --lib
cargo test --locked -p bd_app --test console_input_contract
cargo test --locked -p bd_app --test press_repeat_release_policy
cargo test --locked -p bd_app --test phase6_input
cargo test --locked -p bd_tui --lib
cargo test --locked -p bd_test_support --bin contract_report
cargo test --locked -p bd_test_support --test contract_registry
cargo test --locked -p bd_test_support --test candidate_handoff
cargo test --locked -p bd_test_support --test repository_governance
git diff --check
```

Do not substitute `--lib -- contract_report`; that runs the wrong target. A
zero-test or filtered-away invocation is not a pass.

Finally run exactly:

```text
bash scripts/test-gate.sh \
  --candidate-manifest testing/CONSOLE-C1-CANDIDATE-HANDOFF-v2.ron \
  --manifest-sha256 <REVIEWER-SUPPLIED-CONSOLE-C1-V2-DIGEST>
```

All three contract records must remain `Red`. Candidate mode may report only
`CandidateGreen`; the independent reviewer owns status reconciliation and
canonical promotion.

## Stop conditions — stop outranks every green

Stop immediately with `STATUS=NotComplete` if:

- manifest authentication or a protected hash fails;
- either starting Red passes, compiles differently, or fails at another tuple;
- any of the six preservation cases starts or becomes Red;
- a required edit is outside the four-file write set;
- a protected test, observer, plan, registry, evidence file, requirement map,
  gate, baseline, manifest, body, or digest would need to change;
- the reducer retains a production pending write alongside `ConsoleCommand`;
- no production reader bridges `ConsoleCommand` to dispatch;
- more than one bridge/reducer writes the dispatch queue;
- ordering relies on insertion/tuple/resource-conflict behavior, the no-op TUI
  guard, or ambiguity suppression;
- a Title-, Outpost-, key-, command-, fixture-, audit-, or test-specific branch
  replaces the shared ownership rule;
- a duplicate TUI editing/submission match returns;
- Repeat/Release mutates, a console batch leaks, or closed input is swallowed;
- an exact or neighbor command is skipped, substituted, filtered to zero,
  or has unclassified output;
- an unauthorized/untracked report or cleanup file appears;
- the signed gate conflicts with any higher-priority rule.

## Binary shortcut checklist

Every answer must be evidence-backed `No`. Any `Yes`, `Unknown`, missing, or
unsupported answer means `STATUS=NotComplete`.

1. Any protected or reviewer-owned file changed?
2. Any path outside the four-file production write set changed after baseline?
3. Any v1 body, baseline, manifest, digest, or report reused as authority?
4. Any reducer or TUI path writes both `ConsoleCommand` and pending dispatch?
5. Any production dispatch result bypasses `ConsoleCommand`?
6. Any second queue writer, command emitter, editing reducer, or dispatch path?
7. Any `help`, fixture, audit-resource, test-name, or profile special case?
8. Any ambiguity suppression used instead of an explicit dependency?
9. Any tuple/plugin insertion/resource-conflict/no-op-guard order assumption?
10. Any physical close behavior dependent only on final `ConsoleState.open`?
11. Any Repeat/Release mutation, reordering, leak, or closed-input over-capture?
12. Any history, completion, cursor, Backspace, Escape, or printable behavior
    removed or silently weakened?
13. Any direct reducer/queue/dispatcher call used as primary proof?
14. Any exact or neighbor command skipped, substituted, zero-test, or filtered?
15. Any pre-baseline work claimed, altered, reverted, or formatted outside scope?
16. Any repository report/handoff/evidence/log file created by the candidate?
17. Any gate output used to waive a scope, authority, or architecture failure?
18. Any claim beyond `CandidateGreen` or claim made with an unresolved stop?

## Required implementer in-chat handoff

Paste this into chat at the end of every loop; do not write it to the repository.

```text
IMPLEMENTER IN-CHAT HANDOFF
Role: implementation agent
Batch / iteration: Console C1 input ownership v2 / <number>
Status: CandidateGreen | NotComplete
Authority body: docs/handoff/CONSOLE-C1-CANDIDATE-HANDOFF-BODY-v2.md
Manifest: testing/CONSOLE-C1-CANDIDATE-HANDOFF-v2.ron
Reviewer-supplied digest used: <digest>

Decision record before each edit:
- Failing contract and exact case:
- Observed expected/actual tuple:
- Missing production responsibility:
- Reusable owner:
- Integration seam and schedule stage:
- Authorized target file:
- Paired cases rerun:
- Behavior preserved:
- Scope/stop-condition check:

Changes attributable to this iteration:
- <authorized path>: <one production responsibility and why>

Complete delta from sealed v2 baseline, including untracked paths:
- Authorized production paths changed:
- Unauthorized added/modified/deleted/renamed paths: none | <exact list>

Eight independent focused results:
- physical editing:
- typed causal dispatch:
- Title Escape:
- Title backtick:
- explicit schedule ordering:
- Outpost Escape:
- rebound Outpost backtick:
- closed-console preservation:

Neighbor results with tests passed/failed/ignored:
- fmt:
- bd_console --lib:
- console_input_contract aggregate:
- press_repeat_release_policy:
- phase6_input:
- bd_tui --lib:
- contract_report --bin:
- contract_registry:
- candidate_handoff:
- repository_governance:
- git diff --check:

Signed candidate gate:
- Exact command:
- Exit code:
- Gate steps:
- Tests listed / passed / failed / ignored:
- Reported status:

Protected-file / manifest integrity:
Shortcut checklist 1-18 with evidence-backed answers:
First remaining failure or stop condition:
Next legal reviewer action:
```
