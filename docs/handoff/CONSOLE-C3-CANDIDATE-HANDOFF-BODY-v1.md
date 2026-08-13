# Console C3 Debug Mutation Boundary Candidate Handoff — v1

Use `$authoritative-test-pipeline`. Execute the complete bounded loop below
until the signed candidate gate reports `STATUS=CandidateGreen`, or stop
immediately with `STATUS=NotComplete` when any stop condition occurs. Do not
merely inspect, plan, explain, or recommend changes.

The separately supplied manifest digest authenticates this digest-free body,
the protected C3 observer, authority/status records, and the dirty-worktree
baseline. Do not edit or regenerate any of them.

## Priority — a later success never waives an earlier rule

1. Protect authority, the sealed baseline, tests, and pre-existing user work.
2. Obey every stop condition and the exact three-file production write set.
3. Replace direct ordinary console mutation with one typed core-owned boundary.
4. Enforce the disabled-by-default gate, exact deltas, deterministic targets,
   one resolver, clear results, and one trace per request.
5. Preserve read-only commands, all C1/C2 behavior, and the separately owned C4
   combat, GodMode, and blueprint-spawn commands.
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
docs/handoff/CONSOLE-C3-CANDIDATE-HANDOFF-BODY-v1.md
crates/bd_app/tests/console_debug_contract.rs
crates/bd_console/src/commands.rs
crates/bd_console/src/dispatch.rs
crates/bd_console/src/lib.rs
crates/bd_console/src/state.rs
crates/bd_core/src/debug.rs
crates/bd_core/src/lib.rs
crates/bd_core/src/trace.rs
crates/bd_core/src/events.rs
crates/bd_core/src/spatial.rs
crates/bd_core/src/colony/survivors.rs
crates/bd_core/src/colony/production.rs
testing/foundation-contracts.ron
testing/FOUNDATION-REQUIREMENT-MAP.md
testing/FOUNDATION-TEST-EVIDENCE.md
testing/CONSOLE-C3-BASELINE-v1.ron
testing/CONSOLE-C3-CANDIDATE-HANDOFF-v1.ron
```

Read the complete authoritative-test skill. The tests, no-op interface
scaffold registration, plan, registry, evidence, requirement map, prior C1/C2
work, body, baseline, manifest, gate, and policy are reviewer-owned.

## Exact production write set

You may modify only:

```text
crates/bd_console/src/dispatch.rs
crates/bd_console/src/lib.rs
crates/bd_core/src/debug.rs
```

No other production, test, documentation, report, evidence, log, baseline, or
manifest path is authorized. `crates/bd_core/src/lib.rs` already registers the
disabled scaffold and is protected. `commands.rs` and `state.rs` are not
authorized. Before every edit, name the target and confirm it is listed above.
After every iteration, compare the complete Git-visible delta to the sealed
baseline. Formatting may change only these three files.

Do not use `git stash`, `git checkout`, `git restore`, `git reset`, or any
revision-based replacement. `HEAD` is not the sealed baseline and predates
reviewer preparation.

## Locked semantic outcome

### Gate and ownership

- `DebugMutationGate` remains core-owned and defaults to disabled.
- Deliberately installing `BdConsolePlugin` explicitly enables the gate for
  that app; the core default must not become enabled.
- The existing denial-only scaffold becomes the single real resolver. Do not
  leave it in place and add a second resolver.
- The resolver remains in the named `DebugMutationSet::Resolve` stage inside
  `BdSet::Mutation`.
- `execute_console_command` has an explicit production dependency before that
  named resolver. Insertion order, resource conflicts, tuple order, and
  ambiguity suppression are not dependency edges.
- Dispatch parses commands, emits `DebugMutationRequest`, and reports typed
  `DebugMutationResult`; it does not edit gameplay components/resources or emit
  canonical gameplay effects for C3 commands.
- The core resolver alone validates and either applies the mutation or emits
  the existing canonical event/transition effect.

The registered production path is:

```text
ConsoleCommand
  -> existing Mutation-stage command bridge
  -> parse in console dispatch
  -> exactly one DebugMutationRequest
  -> exactly one DebugMutationSet::Resolve system
  -> accepted mutation OR atomic rejection/disabled denial
  -> exactly one DebugMutationResult and SignalTrace entry
  -> ConsoleState output through a production result reader
```

Result transport may take the next update; it may not parse trace prose or
inspect mutated state to reconstruct a result. Do not add request/result cursor
state to protected `ConsoleState` or create a parallel pending-command path.

### Completion-critical accepted matrix

The protected primary executes thirteen fresh-app rows covering all ten C3
mutation variants and all four colony resource kinds:

| Command | Typed request | Exact authorized first-frame delta/effect |
|---|---|---|
| `supplies 7` | `AddColonyResource(Supplies, 7)` | Supplies `10 -> 17` only |
| `materials 6` | `AddColonyResource(Materials, 6)` | Materials `0 -> 6` only |
| `faith 5` | `AddColonyResource(Faith, 5)` | Faith `0 -> 5` only |
| `plants 4` | `AddColonyResource(WildPlants, 4)` | WildPlants `0 -> 4` only |
| `day 7` | `SetDay(7)` | day becomes 7 only |
| `turn 9` | `SetTurn(9)` | turn becomes 9 only |
| `skip_day` | `SkipDay` | day increments once only |
| `event test.c3.event` | `TriggerEvent` | one existing `EventTrigger` effect only |
| `end_event` | `EndEvent` | active becomes false; event identity remains |
| `survivor Nia Vale` | `SpawnSurvivor` | one current survivor bundle at `(1,1)` |
| `task 0 defending` | typed task request | only the listed target becomes Defending |
| `goto 8 6` | `TeleportPlayer` | only player position becomes `(8,6)` |
| `shelter` | `TransitionToShelter` | one existing `TransitionIntent` effect only |

The dispatch-boundary observer runs after dispatch and before the resolver. It
must see exactly one matching request while the complete observed snapshot is
still equal to the pre-command snapshot. Emitting a decorative request after a
direct dispatch mutation is Red. Applying the delta both in dispatch and the
resolver is Red.

For event and shelter commands, the resolver emits the existing canonical
effect; it does not directly run event or spatial transition logic. Preserve
resource clamping, existing survivor components/scope/pools, parser aliases,
and current validation semantics.

Each accepted row returns one result containing `OK:` and one new trace entry
with stage `DebugMutation`, signal type `DebugMutationResult`, and an
`accepted` summary. Exact display prose beyond the protected semantic tokens is
not locked.

### Disabled and enabled-rejection behavior

With the gate forced disabled, all thirteen accepted-matrix inputs:

- still cross exactly one typed pre-mutation boundary;
- leave every observed state/effect field unchanged;
- produce one `ERROR` result containing `disabled`; and
- produce one `DebugMutationResult` trace summarized as `denied`.

With the gate enabled, the five protected invalid cases—unknown event,
registered event without a player, inactive `end_event`, out-of-range task
index, and teleport without a player—cross the same typed boundary, remain
atomic, produce one meaningful `ERROR`, and produce one trace summarized as
`rejected`. Validation does not move back into dispatch.

### Stable visible targeting

- Derive the survivor target projection once in core and reuse that semantic
  projection for read-only listing and resolver selection.
- `stats` remains read-only but adds discoverable survivor rows containing
  index, name, and position.
- Ordering is deterministic across opposite spawn orders and does not use raw
  ECS iteration order or raw `Entity` values.
- For the protected fixture, `#0` is `Alex` at `(2,2)` in both insertion orders;
  `task 0 defending` mutates that survivor and not `Alex` at `(6,2)`.
- The task result repeats the chosen visible name and position.
- If two candidates have identical visible name and position, the command is
  rejected atomically with `ERROR`, `ambiguous`, name, and position. Do not use
  task state or raw Entity order as a hidden tie-breaker.

Shared ownership means the ordered target facts are derived once. Independently
sorting/rederiving them in `stats` and the resolver is not DRY and is not green.

### Required preservation and excluded C4 scope

- `help`, `stats`, `blueprints`, `events`, and `clear` emit no debug mutation or
  debug trace and change no gameplay state. `clear` remains ConsoleState-local.
- Existing C1 physical input/submission and C2 composition/invalidation remain.
- C3 does not implement, remove, or semantically redesign `kill_all`, `heal`,
  `god on|off`, or `spawn <blueprint>`.
- The protected C4 preservation case requires those four routes to retain their
  current behavior. C4 later owns their typed migration, GodMode damage rule,
  canonical factory reuse, and mode-derived scope.
- Do not edit pool resolution, factory code, command parsing, TUI code, or
  unrelated gameplay behavior.

## Authenticate and reproduce every starting case independently

Authenticate the manifest with the separately supplied SHA-256 digest before
any edit. Each command below must execute exactly one test.

Intentional Red:

```text
cargo test --locked -p bd_app --test console_debug_contract console_plugin_explicitly_enables_the_debug_gate -- --exact --nocapture
cargo test --locked -p bd_app --test console_debug_contract disabled_gate_blocks_every_c3_mutation_and_reports_each_denial -- --exact --nocapture
cargo test --locked -p bd_app --test console_debug_contract every_ordinary_mutation_crosses_one_typed_boundary_then_applies_exactly_one_delta -- --exact --nocapture
cargo test --locked -p bd_app --test console_debug_contract enabled_invalid_mutations_are_atomic_and_return_one_rejection_trace -- --exact --nocapture
cargo test --locked -p bd_app --test console_debug_contract debug_dispatch_precedes_exactly_one_named_core_resolver -- --exact --nocapture
cargo test --locked -p bd_app --test console_debug_contract survivor_indices_share_one_visible_stable_order_and_reject_indistinguishable_duplicates -- --exact --nocapture
```

Expected baseline signatures:

- console opt-in: `expected=enabled actual=disabled`;
- disabled matrix: all thirteen rows report a missing typed boundary, direct
  state/effect change, missing denial trace, and misleading `OK` result;
- enabled matrix: all thirteen rows report missing typed boundary and missing
  acceptance trace, while the exact authorized deltas themselves are present;
- enabled rejections: all five rows remain atomic and readable but report a
  missing typed boundary and missing rejection trace;
- schedule: `(dispatchers=1,resolvers=1,dispatch-before-resolver=false)`;
- target: missing visible index, spawn-order-dependent selection, results that
  omit visible identity, and non-atomic exact-duplicate selection.

A pass, compile failure, zero-test run, missing completion-critical row, or a
different semantic failure is a stop condition.

Preservation Green:

```text
cargo test --locked -p bd_app --test console_debug_contract core_debug_gate_defaults_disabled_and_denies_direct_requests -- --exact --nocapture
cargo test --locked -p bd_app --test console_debug_contract read_only_and_console_local_commands_emit_no_debug_mutation -- --exact --nocapture
cargo test --locked -p bd_app --test console_debug_contract c4_combat_god_and_blueprint_commands_preserve_their_existing_behavior -- --exact --nocapture
```

Each must execute one test and pass before and after every responsibility.

## Mandatory bounded implementation loop

```text
READ
  Re-read this body, the complete protected C3 test/comments, current three
  authorized files, baseline, manifest, and first remaining Red.
REPRODUCE
  Run all six Reds and three Greens independently; preserve exact signatures.
DECIDE
  Fill every decision-record field. Unknown means stop.
IMPLEMENT
  Confirm the path is authorized. Change one production responsibility only.
FOCUSED VALIDATION
  Run the failing case, its paired gate/target/read-only case, and all three
  preservation Greens independently.
CLASSIFY
  If Red, name the next missing responsibility and repeat. Unexpected output,
  observer change, or scope need means stop.
NEIGHBOR VALIDATION
  Run every required closing command below.
CANDIDATE GATE
  Run the signed v1 candidate gate with the reviewer-supplied digest.
SELF-AUDIT
  Audit the complete baseline delta, three-file diff, request/result ownership,
  exact deltas, traces, stable target projection, test inventory, and shortcuts.
HANDOFF
  Paste the required in-chat report. Do not create a repository report.
```

Before each edit, record:

```text
Failing contract and exact case/row:
Observed expected/actual tuple:
Missing production responsibility:
Reusable semantic owner:
Integration seam and schedule stage:
Authorized target file:
Paired cases to rerun:
Behavior that must remain unchanged:
Scope/stop-condition check:
```

Suggested responsibility order, subject to the decision record:

1. explicit console opt-in plus registered result/request transport;
2. dispatch-to-request mapping and explicit before-resolver edge;
3. gate-aware single resolver with accepted/rejected results and traces;
4. one shared stable survivor projection for stats and task resolution;
5. full focused, neighbor, signed, and shortcut audit.

Do not combine responsibilities merely to reduce iterations.

## Required closing commands

Run all nine focused cases independently first, then:

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
  --candidate-manifest testing/CONSOLE-C3-CANDIDATE-HANDOFF-v1.ron \
  --manifest-sha256 <REVIEWER-SUPPLIED-CONSOLE-C3-V1-DIGEST>
```

All three C3 records remain `Red`; only an independent reviewer may reconcile
registry, map, evidence, or plan status. Candidate mode may report only
`CandidateGreen` or `NotComplete`.

## Stop conditions — stop outranks every green

Stop immediately with `STATUS=NotComplete` if:

- manifest authentication, protected hashes, baseline scope, or sealed test
  inventory differs;
- a starting Red/Green differs from the signatures above;
- a required edit is outside the exact three-file write set;
- any protected test, observer, scaffold registration, parser, ConsoleState,
  plan, registry, evidence, requirement map, body, baseline, manifest, skill,
  policy, gate, or prior C1/C2 file would need change;
- another debug resolver, gameplay mutator, command queue, result reconstruction,
  or survivor-order derivation is introduced;
- the core default becomes enabled, gate denial mutates, or dispatch retains a
  C3 gameplay mutation/effect alongside a typed request;
- event/transition logic is duplicated instead of emitting canonical effects;
- survivor selection uses ECS iteration order, raw Entity identity, mutable task
  state as a tie-breaker, or independently rederived list/resolver ordering;
- a C4 command is removed, redesigned, claimed, or pulled into this batch;
- a fixture/command/test/content-ID branch, prose parsing, ambiguity suppression,
  or test-only production API is used;
- a test is edited, deleted, renamed, ignored, filtered, or removed from the
  sealed inventory;
- a required command is skipped, substituted, zero-test, or unclassified;
- an unauthorized report/log/evidence file appears or status is self-promoted;
- the signed gate conflicts with any higher-priority rule.

A genuine required path outside scope or observer defect is a correct
`NotComplete` result and requires a newly sealed handoff.

## Binary shortcut checklist

Every answer must be evidence-backed `No`. Any `Yes`, `Unknown`, missing, or
unsupported answer means `STATUS=NotComplete`.

1. Protected or reviewer-owned file changed?
2. Path outside the exact three-file write set changed after baseline?
3. Stash/checkout/restore/reset or revision replacement used?
4. Test edited, deleted, renamed, ignored, filtered, or production-compiled?
5. Core gate default enabled or gate bypass retained?
6. Direct C3 dispatch mutation/effect remains beside a decorative request?
7. More than one resolver/result owner or command submission path exists?
8. Result reconstructed from display/trace prose or post-mutation inspection?
9. Shared survivor ordering duplicated, ECS-ordered, or raw-ID-backed?
10. Exact duplicate silently selected instead of visibly rejected?
11. Event/transition canonical behavior duplicated or directly applied?
12. Resource kind, command, fixture, content ID, test, or profile hardcoded?
13. C4 or unrelated gameplay/UI behavior changed?
14. Required exact/neighbor/signed command skipped or zero-test?
15. Contract, map, plan, or evidence status self-promoted?

## Required in-chat candidate handoff

```text
# Console C3 v1 Candidate Handoff Report

STATUS=CandidateGreen | STATUS=NotComplete

## Scope and integrity
- Manifest/digest used:
- Authentication result:
- Exact authorized files changed:
- Unexpected paths:
- Complete delta from C3 baseline, including untracked paths:
- Test inventory listed/digest result:
- Revision-replacement commands used:

## Current iteration
- Objective:
- Failing case/row reproduced before edit:
- Missing responsibility and reusable owner:
- Production change attributable to this iteration:
- Paired validation actually run:
- First remaining failure, or None:

## Gate and typed boundary
- Core default / console opt-in:
- Dispatch pre-mutation observations for all 13 rows:
- Disabled atomicity/results/traces:
- Enabled accepted exact deltas/results/traces:
- Enabled rejection atomicity/results/traces:
- Named resolver count and explicit schedule edge:

## Stable targeting and preservation
- Shared target projection owner:
- Forward/reverse visible order and selected identity:
- Exact-duplicate denial:
- Read-only/local commands:
- C4 preservation:
- C1/C2 neighbors:

## Commands actually run
- Nine exact focused commands with pass/fail counts:
- bd_core / bd_console / bd_app / bd_tui outcomes:
- governance / candidate guard / inventory outcomes:
- formatting / clippy / diff check:

## Signed candidate gate
- Steps passed/failed:
- Tests listed/passed/failed/ignored:
- Contract metrics:
- Final status line:

## Shortcut checklist
1. Protected change? No — evidence:
2. Outside-scope delta? No — evidence:
3. Revision replacement? No — evidence:
4. Test changed/removed/filtered? No — evidence:
5. Core default enabled/gate bypass? No — evidence:
6. Direct mutation plus decorative request? No — evidence:
7. Duplicate owner/path? No — evidence:
8. Prose/state reconstruction? No — evidence:
9. Target ordering shortcut? No — evidence:
10. Duplicate silently selected? No — evidence:
11. Canonical effect duplicated? No — evidence:
12. Hardcode/special case? No — evidence:
13. C4/unrelated change? No — evidence:
14. Required command skipped/zero-test? No — evidence:
15. Status self-promoted? No — evidence:

## Stop-condition result and next action
- Stop condition encountered:
- Exact blocker or first remaining failure:
- Next legal action or reviewer decision required:
- C3 contract status remains: Red
```
