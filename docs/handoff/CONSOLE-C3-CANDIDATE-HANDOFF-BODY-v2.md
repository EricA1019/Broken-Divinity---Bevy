# Console C3 Request Fan-out Recovery Candidate Handoff — v2

Use `$authoritative-test-pipeline`. Execute the complete bounded loop below
until the signed candidate gate reports `STATUS=CandidateGreen`, or stop
immediately with `STATUS=NotComplete` when any stop condition occurs. Do not
merely inspect, plan, explain, or recommend a change.

This v2 seal supersedes C3 v1. V1 produced useful gated typed-boundary work,
but its CandidateGreen claim was rejected because its implementation agent
rewrote tests co-located in an authorized production file. Do not use, edit,
regenerate, restore from, or claim authority from the v1 body, baseline,
manifest, digest, or report. The reviewer has independently adopted the
current typed-request unit-test semantics and protected `dispatch.rs` in full.

The separately supplied v2 manifest digest authenticates this digest-free
body, all test bodies, current useful production outside the one-file write
set, authority/status records, and the dirty-worktree baseline. Do not edit or
regenerate any of them.

## Priority — a later success never waives an earlier rule

1. Protect authority, all tests, the v2 baseline, and pre-existing user work.
2. Obey every stop condition and the exact one-file production write set.
3. Replace destructive request draining with one resolver-local read cursor.
4. Preserve the existing single mutation owner, exact deltas, gate behavior,
   results, traces, target projection, and schedule edge.
5. Preserve read-only commands, C1/C2, and the separately owned C4 behavior.
6. Run every focused, paired, neighboring, inventory, and signed command.
7. Report measured evidence without changing contract or evidence status.

## Read completely before editing

```text
AGENTS.md
GDD.md
Kernel.md
docs/authority/DECISIONS-TO-LOCK.md
docs/authority/AUTHORITATIVE-TESTING-STANDARD-AND-MIGRATION-PLAN.md
docs/active/FOUNDATION-STABILIZATION-AND-CONSOLE-HARDENING-PLAN.md
docs/handoff/CONSOLE-C3-CANDIDATE-HANDOFF-BODY-v2.md
crates/bd_app/tests/console_debug_contract.rs
crates/bd_console/src/commands.rs
crates/bd_console/src/dispatch.rs
crates/bd_console/src/lib.rs
crates/bd_console/src/state.rs
crates/bd_core/src/debug.rs
crates/bd_core/src/lib.rs
crates/bd_core/src/trace.rs
testing/foundation-contracts.ron
testing/FOUNDATION-REQUIREMENT-MAP.md
testing/FOUNDATION-TEST-EVIDENCE.md
testing/CONSOLE-C3-BASELINE-v2.ron
testing/CONSOLE-C3-CANDIDATE-HANDOFF-v2.ron
```

Read the complete authoritative-test skill. Tests, dispatch, console plugin,
parser, state, plan, registry, evidence, requirement map, prior C1/C2 work,
body, baseline, manifest, gate, and policy are reviewer-owned.

## Exact production write set

You may modify only:

```text
crates/bd_core/src/debug.rs
```

No other production, test, documentation, report, evidence, log, baseline, or
manifest path is authorized. Before every edit, state the target path and
confirm it is exactly the path above. After every iteration, compare the
complete Git-visible delta to the sealed v2 baseline. Formatting may change
only this file.

Do not use `git stash`, `git checkout`, `git restore`, `git reset`, or any
revision-based replacement. `HEAD` predates reviewer preparation and useful C3
work; it is not the sealed baseline.

## Locked semantic outcome

`DebugMutationRequest` is a typed Bevy message and must retain normal
independent-reader semantics:

- the existing `DebugMutationSet::Resolve` system remains the only resolver;
- that resolver keeps its own persistent cursor and reads each request once;
- resolving must not call `Messages::drain`, `clear`, or otherwise consume the
  shared collection for other readers;
- one read-only observer scheduled after resolution sees the same request once;
- the resolver still applies the mutation exactly once and produces exactly
  one ordered `DebugMutationResult` and one trace entry per request;
- the disabled gate and enabled rejection paths remain atomic;
- message retention follows Bevy's registered message lifecycle; do not add an
  unbounded archive or duplicate request queue.

A resolver-local `MessageCursor` is compatible with the exclusive world-owned
resolver. The exact private variable names and control flow are not locked.
Do not add a public API, resource, second resolver, second mutation path, or
test-specific branch.

Everything already green in C3 v1 is preservation scope. In particular, do
not change console dispatch, plugin registration, result transport, command
parsing, stable survivor projection, C4 direct routes, pool resolution,
factory behavior, TUI behavior, or status records.

## Authenticate and reproduce every starting case independently

Authenticate the v2 manifest with the separately supplied SHA-256 digest
before any edit. Then run each command separately and confirm exactly one test
executes.

Intentional Red:

```text
cargo test --locked -p bd_app --test console_debug_contract debug_request_channel_remains_observable_after_core_resolution -- --exact --nocapture
```

Expected baseline signature: day resolves to 23, then only
`case=post-resolver-fanout checkpoint=independent-reader` fails with
`expected=[SetDay(23)] actual=[]`. A pass, compile failure, zero-test run,
mutation-delta failure, or different diagnostic is a stop condition.

Preservation Green — run independently:

```text
cargo test --locked -p bd_app --test console_debug_contract console_plugin_explicitly_enables_the_debug_gate -- --exact --nocapture
cargo test --locked -p bd_app --test console_debug_contract disabled_gate_blocks_every_c3_mutation_and_reports_each_denial -- --exact --nocapture
cargo test --locked -p bd_app --test console_debug_contract every_ordinary_mutation_crosses_one_typed_boundary_then_applies_exactly_one_delta -- --exact --nocapture
cargo test --locked -p bd_app --test console_debug_contract enabled_invalid_mutations_are_atomic_and_return_one_rejection_trace -- --exact --nocapture
cargo test --locked -p bd_app --test console_debug_contract debug_dispatch_precedes_exactly_one_named_core_resolver -- --exact --nocapture
cargo test --locked -p bd_app --test console_debug_contract survivor_indices_share_one_visible_stable_order_and_reject_indistinguishable_duplicates -- --exact --nocapture
cargo test --locked -p bd_app --test console_debug_contract core_debug_gate_defaults_disabled_and_denies_direct_requests -- --exact --nocapture
cargo test --locked -p bd_app --test console_debug_contract read_only_and_console_local_commands_emit_no_debug_mutation -- --exact --nocapture
cargo test --locked -p bd_app --test console_debug_contract c4_combat_god_and_blueprint_commands_preserve_their_existing_behavior -- --exact --nocapture
```

Each preservation command must execute one test and pass before and after the
edit. Do not modify a test because its old unit-level expectation appears
inconvenient; every test body is protected in v2.

## Mandatory bounded implementation loop

```text
READ
  Re-read this body, the complete protected fan-out test/comments, current
  debug resolver, baseline, manifest, and first remaining Red.
REPRODUCE
  Run the one Red and all nine preservation Greens independently.
DECIDE
  Fill every decision-record field. Unknown means stop.
IMPLEMENT
  Confirm the target is bd_core/src/debug.rs. Change only request reading.
FOCUSED VALIDATION
  Run the Red, accepted matrix, disabled matrix, rejection matrix, and named
  schedule case independently.
CLASSIFY
  If Red, name the remaining message-reader responsibility and repeat.
  Unexpected output, test change, or scope need means stop.
NEIGHBOR VALIDATION
  Run every required closing command below.
CANDIDATE GATE
  Run the signed v2 candidate gate with the reviewer-supplied digest.
SELF-AUDIT
  Audit the complete baseline delta, one-file diff, request ownership, message
  fan-out, exact deltas, results/traces, protected hashes, and shortcuts.
HANDOFF
  Paste the required in-chat report. Do not create a repository report.
```

Before the edit, record:

```text
Failing contract and exact case:
Observed expected/actual tuple:
Missing production responsibility:
Reusable semantic owner:
Integration seam and schedule stage:
Authorized target file:
Paired cases to rerun:
Behavior that must remain unchanged:
Scope/stop-condition check:
```

Change one responsibility only: replace destructive collection consumption
with resolver-local cursor reading. Do not refactor the mutation match, target
projection, result wording, trace format, registration, or unrelated code.

## Required closing commands

Run all ten focused cases independently first, then:

```text
cargo fmt --all -- --check
cargo test --locked -p bd_core
cargo test --locked -p bd_console --lib
cargo test --locked -p bd_app --test console_debug_contract
cargo test --locked -p bd_app --test console_input_contract
cargo test --locked -p bd_app --tests
cargo test --locked -p bd_tui --lib
cargo test --locked -p bd_test_support --bin handoff_guard
cargo test --locked -p bd_test_support --test candidate_handoff
cargo test --locked -p bd_test_support --test contract_registry
cargo test --locked -p bd_test_support --test repository_governance
cargo clippy --workspace --all-targets --locked -- -D warnings
git diff --check
```

Finally run exactly:

```text
bash scripts/test-gate.sh \
  --candidate-manifest testing/CONSOLE-C3-CANDIDATE-HANDOFF-v2.ron \
  --manifest-sha256 <REVIEWER-SUPPLIED-CONSOLE-C3-V2-DIGEST>
```

All three C3 records remain `Red`; only an independent reviewer may reconcile
registry, map, evidence, or plan status. Candidate mode may report only
`CandidateGreen` or `NotComplete`.

## Stop conditions — stop outranks every green

Stop immediately with `STATUS=NotComplete` if:

- manifest authentication, a protected hash, baseline scope, or the 832-test
  sealed inventory differs;
- the Red or any preservation Green differs from the signatures above;
- any file other than `crates/bd_core/src/debug.rs` would need change;
- any test, dispatch code, console plugin, observer, parser, ConsoleState,
  plan, registry, evidence, map, body, baseline, manifest, skill, policy, gate,
  or prior C1/C2 artifact is edited or would need change;
- the resolver drains, clears, mutates, replaces, or duplicates the shared
  request collection instead of maintaining its own read cursor;
- another resolver, mutation owner, request queue, result reader, or trace path
  is introduced;
- a request is applied or reported more than once, retained without lifecycle
  bounds, or reconstructed from result/trace/display prose;
- gate, exact-delta, rejection, stable-target, read-only, C1/C2, or C4 behavior
  changes;
- a fixture/command/test/content-ID branch or test-only production API appears;
- a test is edited, deleted, renamed, ignored, filtered, production-compiled,
  shadowed, or replaced while retaining its name;
- a required command is skipped, substituted, zero-test, or unclassified;
- an unauthorized report/log/evidence file appears or status is self-promoted;
- the signed gate conflicts with any higher-priority rule.

## Binary shortcut checklist

Every answer must be evidence-backed `No`. Any `Yes`, `Unknown`, missing, or
unsupported answer means `STATUS=NotComplete`.

1. Protected/reviewer-owned file changed?
2. Path outside the exact one-file write set changed after baseline?
3. Stash/checkout/restore/reset or revision replacement used?
4. Test body/name/inventory changed, shadowed, or production-compiled?
5. Shared request collection drained, cleared, replaced, or duplicated?
6. Second resolver/mutator/request queue/result owner introduced?
7. Resolver-local cursor recreated each frame or request processed twice?
8. Result/request reconstructed from prose or post-mutation inspection?
9. Core default enabled, denial mutated state, or validation moved to dispatch?
10. Exact delta, trace, target ordering, or ambiguity behavior changed?
11. C4 or unrelated gameplay/UI behavior changed?
12. Fixture/command/content-ID/test special case added?
13. Required exact/neighbor/signed command skipped or zero-test?
14. Pre-existing work claimed as this one-file iteration?
15. Gate output used to waive a higher-priority violation or status promoted?

## Required in-chat candidate handoff

```text
# Console C3 v2 Candidate Handoff Report

STATUS=CandidateGreen | STATUS=NotComplete

## Scope and integrity
- Manifest/digest used and authentication result:
- Exact authorized file changed:
- Unexpected paths:
- Complete delta from the v2 baseline, including untracked paths:
- Test inventory listed/digest result:
- Revision-replacement commands used:

## Iteration
- Red signature reproduced before edit:
- Missing responsibility and reusable owner:
- Production change attributable to this iteration:
- Paired validation actually run:
- First remaining failure, or None:

## Boundary evidence
- Post-resolver independent-reader observation:
- Accepted/disabled/rejected matrices:
- Resolver count and explicit schedule edge:
- Result and trace cardinality/order:
- Stable targeting, read-only behavior, and C4 preservation:

## Commands and gate
- Ten exact focused commands with counts:
- Neighbor and governance commands with measured outcomes:
- Formatting/clippy/diff check:
- Signed gate steps and test totals:
- Final signed-gate status line:

## Shortcut checklist
1. Protected change? No — evidence:
2. Outside-scope delta? No — evidence:
3. Revision replacement? No — evidence:
4. Test changed/shadowed? No — evidence:
5. Destructive/duplicate request owner? No — evidence:
6. Duplicate resolver/result path? No — evidence:
7. Cursor reset/double processing? No — evidence:
8. Prose/state reconstruction? No — evidence:
9. Gate/validation bypass? No — evidence:
10. Existing C3 semantic drift? No — evidence:
11. C4/unrelated drift? No — evidence:
12. Hardcode/special case? No — evidence:
13. Required command skipped/zero-test? No — evidence:
14. Pre-existing work claimed? No — evidence:
15. Status/gate overclaim? No — evidence:

## Stop-condition result and next action
- Stop condition encountered:
- Exact blocker or first remaining failure:
- Next legal reviewer action:
- C3 contract status remains: Red
```
