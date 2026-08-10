# Console C1 Input Ownership Candidate Handoff — v1

Use `$authoritative-test-pipeline`. Carry out the complete authorized batch in
the loop below until the signed gate reports `STATUS=CandidateGreen`, or stop
with `STATUS=NotComplete` when a stop condition is met. Do not merely inspect,
plan, or report recommendations: implement, run, diagnose, and iterate.

The separately supplied manifest digest authenticates the manifest; it is not
authority by itself. This protected body owns scope, priority, required
behavior, commands, stop conditions, and reporting. Older UI9 handoffs are
historical and must not be reused or edited.

## Priority

1. Preserve the sealed baseline, protected authority/tests, and user work.
2. Obey the exact write set and every stop condition.
3. Make one `bd_console` reducer own physical console input.
4. Capture console-owned batches before closing mutates `ConsoleState.open`.
5. Submit one typed `ConsoleCommand` and dispatch it exactly once.
6. Preserve closed-console gameplay, console editing features, and all neighbor
   behavior.
7. Report measured evidence without promoting CandidateGreen to reviewed
   completion.

A lower-priority green never waives a higher-priority violation.

## Read before editing

Read completely:

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
testing/CONSOLE-C1-BASELINE-v1.ron
testing/CONSOLE-C1-CANDIDATE-HANDOFF-v1.ron
```

Read the complete authoritative-test skill before taking an implementation
action. Do not edit the handoff body, baseline, manifest, tests, fixtures,
plans, authority, gate, registry, evidence, requirement map, or governance
code. Do not regenerate the reviewer-supplied digest.

## Exact production write set

You may modify only:

```text
crates/bd_console/src/input.rs
crates/bd_console/src/lib.rs
crates/bd_console/src/state.rs
crates/bd_tui/src/lib.rs
```

These files contain pre-handoff work recorded by the sealed baseline. Preserve
it and do not claim it as C1 evidence. Before every edit, verify that the path
is one of the four exact paths. After every loop, inspect the complete baseline
delta, including untracked files. A required edit anywhere else is a scope
stop: do not make it, and do not repair a protected observer.

Never reset, revert, delete, or broadly rewrite user work. Formatting is
allowed only when it changes the authorized files; repository-wide formatting
that mutates another path is forbidden.

## Required outcome

### One reusable reducer

- `bd_console` owns the only console editing reducer.
- The registered `BdConsolePlugin` schedules the production physical-input
  adapter in `BdSet::Input`.
- The TUI may adapt capture state or order systems, but it must not retain a
  second character/edit/history/submission match.
- Press is the only mutating key kind. Repeat and Release do nothing.
- Same-batch key order is preserved.
- Existing open/toggle, printable input, cursor, Backspace, Escape, Enter,
  history search, and Tab-completion behavior remains available through the
  shared owner.

### Whole-batch capture

- Input ownership is recorded for the physical batch before a close/toggle key
  changes the final open state.
- When the batch begins with the console open, Escape/backtick and every other
  key in that batch are unavailable to gameplay routing.
- A backtick that opens the console also captures its batch so later keys in
  the same batch reach the console in order.
- Gameplay routing consults batch ownership, not merely the final value of
  `ConsoleState.open`.
- The implementation is mode-agnostic: no Title-only or Outpost-only close-key
  branch.
- When a batch begins and remains console-closed, ordinary keys still reach
  gameplay exactly once.

A small reusable capture resource or system-set boundary in `bd_console` is
appropriate. System ordering must be explicit; plugin registration order is
not accepted as proof.

### One typed submission path

- Enter emits exactly one `ConsoleCommand` from the shared reducer.
- The live TUI path does not push a parallel command independently.
- One bridge may queue the typed command for the existing exclusive dispatch
  system, but only that bridge may write the dispatch queue.
- One physical `help` line creates one history entry, one typed message, and
  one help result; the post-dispatch queue is empty.
- Do not add direct test-only APIs or make tests call dispatch directly.

## Starting baseline

Run each command independently before editing. Use the exact integration-test
harness name printed by `--list`. A zero-test run, unexpected pass, changed failure checkpoint, compile
failure, or environmental failure is a baseline stop.

```text
cargo test --locked -p bd_app --test console_input_contract physical_console_editing_uses_the_registered_production_reducer -- --exact --nocapture
cargo test --locked -p bd_app --test console_input_contract one_physical_line_reaches_dispatch_exactly_once -- --exact --nocapture
cargo test --locked -p bd_app --test console_input_contract escape_close_is_consumed_before_title_routing -- --exact --nocapture
cargo test --locked -p bd_app --test console_input_contract backtick_close_is_consumed_before_title_routing -- --exact --nocapture
cargo test --locked -p bd_app --test console_input_contract escape_close_does_not_quit_or_mutate_outpost -- --exact --nocapture
cargo test --locked -p bd_app --test console_input_contract backtick_close_does_not_reach_a_rebound_outpost_action -- --exact --nocapture
```

Expected Red checkpoints, in the same order:

1. registered reducer leaves `open=false`;
2. typed submission audit is `[]`, expected `["help"]`;
3. Title Escape also requests quit;
4. Title backtick also transitions to Outpost;
5. Outpost Escape also requests quit;
6. rebound Outpost backtick advances turn from 0 to 1.

Then independently prove the preservation row begins green:

```text
cargo test --locked -p bd_app --test console_input_contract closed_console_preserves_normal_gameplay_input -- --exact --nocapture
```

It must execute one test and pass by advancing exactly one turn for one Press
plus Release.

## Mandatory implementation loop

Repeat this complete loop; do not stop after a focused green:

```text
READ
  Re-read the current failing test comments, relevant production path, this
  body, and the latest diff.
BASELINE / REPRODUCE
  Run the exact failing row. Confirm its expected diagnostic.
DECIDE
  Fill every decision field below. Unknown means stop.
IMPLEMENT
  Verify the target is in the four-file write set. Make one cohesive change
  in the reusable owner or its integration seam.
FOCUSED VALIDATION
  Run all six Red rows and the preservation row independently.
CLASSIFY
  If any remains Red, name the production cause and begin another loop.
  If a row turns green unexpectedly, investigate false green before proceeding.
NEIGHBOR VALIDATION
  Run every neighbor command below. Diagnose and repair only within scope.
CANDIDATE GATE
  Run the signed candidate gate with the supplied digest.
SELF-AUDIT
  Inspect the complete baseline delta and answer every shortcut question.
IN-CHAT HANDOFF
  Paste the exact report template at the end of this body.
```

Before each edit, record:

```text
Failing contract and exact case:
Observed expected/actual:
Missing production responsibility:
Reusable owner:
Integration seam:
Authorized target file:
Paired cases to rerun:
Behavior that must remain unchanged:
Scope/stop-condition check:
```

Do not choose the shortest edit merely because it makes the first assertion
green. A valid change must satisfy the reusable ownership and whole-batch
semantics that make all cases green together.

## Required validation

After every implementation change, run all seven exact
`console_input_contract` cases independently. Before the signed gate, run:

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

Finally run, replacing only the digest placeholder with the separately
supplied reviewer digest:

```text
bash scripts/test-gate.sh \
  --candidate-manifest testing/CONSOLE-C1-CANDIDATE-HANDOFF-v1.ron \
  --manifest-sha256 <REVIEWER-SUPPLIED-CONSOLE-C1-V1-DIGEST>
```

The candidate must leave all three contract records Red. CandidateGreen means
the signed implementation gate passed while authority/status remains
reviewer-owned; it does not mean VerifiedGreen, ReviewedGreen, or completion.

## Stop conditions

Stop immediately with `STATUS=NotComplete` when any of these occurs:

- a starting Red passes or fails at a different checkpoint before editing;
- the preservation row begins Red;
- a required change is outside the exact write set;
- any protected file, test, fixture, observer, manifest, baseline, or body
  changes;
- a second editing reducer, second physical submission, or second dispatch
  path remains;
- capture is inferred only from final `ConsoleState.open`;
- Title/Outpost-specific leakage exceptions replace batch ownership;
- Repeat or Release mutates state;
- a console-open batch reaches gameplay, or a console-closed batch is swallowed;
- an existing editing/history/completion behavior is removed to gain space or
  simplify the reducer;
- a direct queue mutation, direct dispatch call, source-text assertion, or
  test-only production API substitutes for the physical workflow;
- any exact or neighbor command is skipped, substituted, runs zero tests, or
  has unclassified output;
- the signed guard reports a scope/protected/digest failure;
- a green gate conflicts with any higher-priority rule;
- an untracked report, handoff, evidence, or cleanup file is created.

When stopped, do not improvise around the boundary. Paste the report template
with the exact blocker and required reviewer decision.

## Shortcut checklist

Answer every item with evidence-backed `No`. Any `Yes`, `Unknown`, omission,
or unsupported answer requires `STATUS=NotComplete`.

1. Any test, fixture, plan, registry, evidence, policy, gate, manifest,
   baseline, or handoff body changed?
2. Any Git-visible path outside the four-file write set added, modified,
   deleted, renamed, formatted, or reverted?
3. Any second console editing reducer or copied key match remains in the TUI?
4. Any physical Enter writes more than one command path or bypasses
   `ConsoleCommand`?
5. Any dispatch result can be produced twice from one physical line?
6. Any close/toggle isolation depends only on the console's final open state?
7. Any Title-, Outpost-, key-, command-, fixture-, or test-specific leakage
   special case used instead of generic batch ownership?
8. Any Repeat/Release mutation, batch reordering, or closed-console
   over-capture introduced?
9. Any history, Tab-completion, cursor, Backspace, Escape, or printable-input
   behavior removed or silently weakened?
10. Any direct state/queue/dispatch manipulation used as primary proof instead
    of the registered physical production path?
11. Any exact, neighbor, formatting, Clippy, whitespace, or signed command
    skipped, substituted, filtered incorrectly, or run with zero tests?
12. Any pre-baseline work claimed, reverted, or altered without distinguishing
    it from the authorized candidate delta?
13. Any repository report/handoff/evidence/log file created by the candidate?
14. Any CandidateGreen claim made while a stop condition or higher-priority
    proof remains unresolved?

## Required implementer in-chat handoff

Paste this report into chat at the end of every loop. Do not write it to the
repository.

```text
IMPLEMENTER IN-CHAT HANDOFF
Role: implementation agent
Batch / iteration: Console C1 input ownership v1 / <number>
Status: CandidateGreen | NotComplete
Authority body: docs/handoff/CONSOLE-C1-CANDIDATE-HANDOFF-BODY-v1.md
Manifest: testing/CONSOLE-C1-CANDIDATE-HANDOFF-v1.ron
Reviewer-supplied digest used: <digest>

Decision record before edit:
- Failing contract and exact case:
- Observed expected/actual:
- Missing production responsibility:
- Reusable owner:
- Integration seam:
- Authorized target file:
- Paired cases rerun:
- Behavior preserved:
- Scope/stop-condition check:

Changes this iteration:
- <path>: <behavioral change and why this is the reusable owner/seam>

Exact Red-to-green evidence:
- CONSOLE-INPUT-001 primary: <command, exit, tests, result>
- CONSOLE-COMMAND-001 primary: <command, exit, tests, result>
- CONSOLE-INPUT-002 Title Escape primary: <command, result>
- Title backtick support: <command, result>
- Outpost Escape support: <command, result>
- rebound Outpost backtick support: <command, result>
- closed-console preservation: <command, result>

Neighbor evidence:
- bd_console --lib:
- console_input_contract aggregate:
- press_repeat_release_policy:
- phase6_input:
- bd_tui --lib:
- contract_report unit tests:
- contract_registry:
- candidate_handoff:
- repository_governance:
- fmt / diff-check:

Signed candidate gate:
- Exact command:
- Exit code:
- Gate steps:
- Tests listed / passed / failed / ignored:
- Reported status:

Baseline delta audit:
- Pre-existing sealed paths preserved:
- Authorized files changed:
- Unauthorized added/modified/deleted/renamed paths: none | <exact list>

Shortcut checklist 1-14:
1. No — <evidence>
2. No — <evidence>
3. No — <evidence>
4. No — <evidence>
5. No — <evidence>
6. No — <evidence>
7. No — <evidence>
8. No — <evidence>
9. No — <evidence>
10. No — <evidence>
11. No — <evidence>
12. No — <evidence>
13. No — <evidence>
14. No — <evidence>

Remaining failures or stop condition: none | <exact blocker>
Reviewer-only follow-up: independently inspect diff, reproduce exact rows and
signed gate, review architecture/DRY, then decide registry/evidence promotion.
```
