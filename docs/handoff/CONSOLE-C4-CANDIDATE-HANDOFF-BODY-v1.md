# Console C4 GodMode and Canonical Spawn Candidate Handoff — v1

Use `$authoritative-test-pipeline`. Execute the complete bounded loop below
until the signed candidate gate reports `STATUS=CandidateGreen`, or stop
immediately with `STATUS=NotComplete` when any stop condition occurs. Do not
merely inspect, plan, explain, recommend changes, or return after one patch.

The separately supplied manifest digest authenticates this digest-free body,
all C4 test bodies and comments, co-located test suffixes, current accepted
behavior, authority/status records, and the dirty-worktree baseline. Do not
edit or regenerate any of those reviewer-owned artifacts.

## Priority — later success never waives an earlier rule

1. Protect authority, tests, sealed suffixes, baseline state, and user work.
2. Obey every stop condition and the exact five-file production write set.
3. Route every remaining mutating command through the existing gated resolver.
4. Keep one signed-delta owner and one scoped blueprint-factory owner.
5. Preserve C1–C3, action spawning, ordinary pools, parsing, and rendering.
6. Run every exact focused, paired, neighboring, integrity, and signed command.
7. Report measured CandidateGreen evidence without changing status records.

## Read completely before editing

```text
AGENTS.md
GDD.md
Kernel.md
docs/authority/DECISIONS-TO-LOCK.md
docs/authority/AUTHORITATIVE-TESTING-STANDARD-AND-MIGRATION-PLAN.md
docs/active/FOUNDATION-STABILIZATION-AND-CONSOLE-HARDENING-PLAN.md
docs/handoff/CONSOLE-C4-CANDIDATE-HANDOFF-BODY-v1.md
crates/bd_app/tests/console_debug_c4_contract.rs
crates/bd_app/tests/console_debug_contract.rs
crates/bd_console/src/commands.rs
crates/bd_console/src/dispatch.rs
crates/bd_console/src/lib.rs
crates/bd_core/src/actions.rs
crates/bd_core/src/components.rs
crates/bd_core/src/debug.rs
crates/bd_core/src/factory.rs
crates/bd_core/src/lib.rs
crates/bd_core/src/pools.rs
crates/bd_core/src/signals.rs
crates/bd_core/src/spatial.rs
testing/foundation-contracts.ron
testing/FOUNDATION-REQUIREMENT-MAP.md
testing/FOUNDATION-TEST-EVIDENCE.md
testing/CONSOLE-C4-BASELINE-v1.ron
testing/CONSOLE-C4-CANDIDATE-HANDOFF-v1.ron
```

Read the complete authoritative-test skill. Tests, fixtures, observers,
co-located test suffixes, plans, registry, evidence, requirement map, body,
baseline, manifest, gate, and testing policy are reviewer-owned.

## Exact production write set

You may modify only:

```text
crates/bd_console/src/dispatch.rs
crates/bd_core/src/actions.rs
crates/bd_core/src/debug.rs
crates/bd_core/src/factory.rs
crates/bd_core/src/pools.rs
```

No other production, test, documentation, report, evidence, log, baseline, or
manifest path is authorized. Before every edit, state the target path and
confirm it is exactly one of these five paths. After every iteration, compare
the complete Git-visible delta to the sealed baseline. Formatting may change
only these five production prefixes.

Four authorized files also contain protected test suffixes. The manifest
hashes every byte from the unique `#[cfg(test)]\nmod tests {` marker through
end of file in `dispatch.rs`, `actions.rs`, `factory.rs`, and `pools.rs`.
Do not edit, delete, duplicate, move, reformat, conditionally disable, or
reconstruct those markers or any suffix byte. `debug.rs` has no co-located
test module and its complete content is candidate-writable.

Do not use `git stash`, `git checkout`, `git restore`, `git reset`, or any
revision-based replacement. `HEAD` is not the sealed baseline and contains
neither all current user work nor all reviewer preparation.

## Locked semantic outcome

### One gated typed owner

`kill_all`, `heal`, `god on|off`, and `spawn <blueprint> <x> <y>` must each
emit exactly one existing `DebugMutationRequest` from console dispatch.
Dispatch parses and emits only; it must not inspect targets, mutate components,
emit gameplay effects, validate catalog membership, or manufacture a result.

The existing core `DebugMutationSet::Resolve` system remains the only resolver.
It owns validation and emits exactly one ordered `DebugMutationResult` and one
`DebugMutation` trace for every request. The disabled gate rejects all four
variants without changing the complete observed state. Do not add a second
resolver, request queue, result path, trace path, or default-on gate.

The reviewer scaffold already defines these exact typed variants:

```text
KillAllEnemies
HealPlayer
SetGodMode(bool)
SpawnBlueprint { blueprint_id, position }
```

Replace their deliberate `C4 debug mutation not implemented` rejection arms.
Do not add aliases, parallel variants, string payloads, or new command syntax.

### Canonical combat and healing effects

`KillAllEnemies` retains the established target rule: emit one canonical
`EntityDefeated { kind: Health }` for every entity with `Pools` that is neither
`Player` nor `Survivor`. Reject readably when there are no eligible enemies.
Do not despawn directly or special-case the fixture name.

`HealPlayer` finds the player or rejects readably, then restores each deficient
player pool by emitting its exact positive `PoolDeltaRequested`. It must not
mutate `Pools` directly. The debug resolver must run explicitly before the
canonical pool-delta set so requested and applied counts and final values are
visible in the same update. Full pools require no zero request. Reject a player
without pools readably; do not invent missing pools.

### Narrow GodMode rule

`SetGodMode(true|false)` toggles the marker on the player through the gated
resolver and retains readable already-active/not-active/no-player rejection.

The sole canonical signed-delta resolver suppresses exactly this predicate:

```text
target has Player AND target has GodMode
AND request.kind == Health AND request.amount < 0
```

For that row it emits one normal `PoolDeltaApplied` carrying the original
source, target, kind, tags, and reason, with `before == after` and
`amount_applied == 0`. It must bypass status modifiers, random damage
variance, armor durability loss, Wounded application, and defeat. It does not
suppress positive Health, negative ActionPoints, negative Health on a
non-player even if marked GodMode, or negative Health on an ordinary player.
Do not damage then heal, filter the request at console, or suppress telemetry.

### One scoped blueprint factory

Console spawn validation and mutation live in the core debug resolver. Unknown
IDs cross the typed boundary, preserve the complete observed state, and return
one rejected result and trace whose message contains the requested ID.

Known IDs must use the canonical blueprint factory for the full generic
component bundle: player, blocking, position, name, pools, statuses, and all
catalog marker interpretations. Do not copy the factory into debug or dispatch,
do not branch on either protected fixture ID, and do not special-case a rat.

Mode-derived scope is one DRY rule shared by ordinary action spawning and debug
spawning:

```text
Outpost  -> EntityScope::ColonyPersistent
otherwise -> EntityScope::DungeonTransient
```

Create one reusable scoped-factory operation in `factory.rs` that composes the
existing `spawn_from_blueprint` and authoritative scope assignment. Route both
`Effect::SpawnBlueprintAt` in `actions.rs` and C4 debug spawning through that
operation. Neither path may insert deprecated `PersistentEntity` or
`TransientEntity`. Preserve mutators and the action pipeline's current log and
missing-blueprint behavior. A helper name and private control flow are not
locked; single semantic ownership and both production consumers are locked.

## Authenticate and reproduce the sealed start

Authenticate the v1 manifest with the separately supplied SHA-256 before any
edit. Then run every command below separately and confirm exactly one test
executes.

Intentional Red integration tests:

```text
cargo test --locked -p bd_app --test console_debug_c4_contract remaining_combat_and_spawn_commands_use_one_gated_typed_owner -- --exact --nocapture
cargo test --locked -p bd_app --test console_debug_c4_contract god_mode_blocks_only_negative_player_health_deltas_inside_canonical_resolution -- --exact --nocapture
cargo test --locked -p bd_app --test console_debug_c4_contract console_spawn_matches_canonical_factory_fingerprint_for_unlike_blueprints -- --exact --nocapture
cargo test --locked -p bd_app --test console_debug_c4_contract console_spawn_scope_follows_current_game_mode_without_legacy_markers -- --exact --nocapture
cargo test --locked -p bd_app --test console_debug_c4_contract unknown_console_blueprint_is_atomic_and_reports_one_rejection -- --exact --nocapture
```

Expected baseline signatures, in order:

1. all enabled/disabled rows report no typed request/result/trace; disabled
   rows expose direct mutations; enabled heal reports zero pool messages;
2. only the GodMode-player negative-Health row fails: health reaches zero,
   defeat/despawn occurs, and the application is `12 -> 0` rather than zero;
3. the two rows differ only at their marker fields: RaidEnemy and
   FactionMember are absent from console-spawned entities;
4. Tactical and Outpost both show `ColonyPersistent + PersistentEntity`;
5. state/output are atomic/readable, but request/result/trace are absent.

Intentional Red co-located dispatch tests:

```text
cargo test --locked -p bd_console --lib dispatch::tests::kill -- --exact --nocapture
cargo test --locked -p bd_console --lib dispatch::tests::kill_skips -- --exact --nocapture
cargo test --locked -p bd_console --lib dispatch::tests::heal_ok -- --exact --nocapture
cargo test --locked -p bd_console --lib dispatch::tests::spawn_ok -- --exact --nocapture
cargo test --locked -p bd_console --lib dispatch::tests::spawn_miss -- --exact --nocapture
```

Each executes once and currently fails because emitted C4 requests are `[]`.
A pass, compile failure, zero-test run, different first responsibility, or any
changed protected suffix is a stop condition.

Preservation Green before and after implementation:

```text
cargo test --locked -p bd_app --test console_debug_contract
cargo test --locked -p bd_core actions::tests::spawn_blueprint_at_sets_entity_scope_by_mode -- --exact
cargo test --locked -p bd_core factory::tests::spawn_multiple_markers_with_data -- --exact
cargo test --locked -p bd_core pools::tests::health_negative_delta_reduces_health -- --exact
cargo test --locked -p bd_core pools::tests::player_defeat_triggers_game_over -- --exact
cargo test --locked -p bd_test_support --test candidate_handoff
```

## Mandatory bounded implementation loop

```text
READ
  Re-read this body, first remaining Red, relevant protected comments,
  current five production prefixes, baseline, manifest, and stop conditions.
REPRODUCE
  Run that Red independently and its completion-critical paired cases.
DECIDE
  Fill every decision-record field below. Unknown means stop.
IMPLEMENT
  Confirm the path is in the five-file write set and before any sealed suffix.
  Change exactly one production responsibility.
FOCUSED VALIDATION
  Run the exact Red, its paired matrix, and affected preservation cases.
CLASSIFY
  If still Red, identify the next missing responsibility and repeat. If green
  for a shortcut or unexpected reason, stop; never edit a protected test.
NEIGHBOR VALIDATION
  Run every closing command. Repair only inside an allowed production prefix.
CANDIDATE GATE
  Run the exact signed v1 gate with the reviewer-supplied digest.
SELF-AUDIT
  Audit the full baseline delta, production diff, suffix hashes, DRY owners,
  direct mutation, fixture branches, command counts, and shortcut checklist.
HANDOFF
  Paste the required report in chat. Do not create a repository report file.
```

Before each patch, record:

```text
Failing contract and exact case:
Observed expected/actual tuple:
Missing production responsibility:
Reusable semantic owner:
Integration seam and schedule stage:
Authorized target file and pre-suffix location:
Paired cases to rerun:
Behavior that must remain unchanged:
Scope/stop-condition check:
```

Recommended responsibility order is dispatch emission, core debug variants and
effects/scheduling, narrow pool rule, then shared scoped factory and both
consumers. This is guidance, not permission to skip reproducing the current
first failure or to make speculative multi-owner changes.

## Required closing commands

Run all ten exact Red cases above independently first. Each must execute one
test and pass. Then run:

```text
cargo fmt --all -- --check
cargo test --locked -p bd_core
cargo test --locked -p bd_console --lib
cargo test --locked -p bd_app --test console_debug_contract
cargo test --locked -p bd_app --test console_debug_c4_contract
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
  --candidate-manifest testing/CONSOLE-C4-CANDIDATE-HANDOFF-v1.ron \
  --manifest-sha256 <REVIEWER-SUPPLIED-CONSOLE-C4-V1-DIGEST>
```

All four C4 records remain `Red`; only an independent reviewer may reconcile
registry, requirement map, evidence, plan, or acceptance status. Candidate
mode may report only `CandidateGreen` or `NotComplete`.

## Stop conditions — stop outranks every green

Stop immediately with `STATUS=NotComplete` if:

- manifest authentication, protected hash/suffix, baseline scope, or the
  839-test sealed inventory differs;
- a named Red or preservation Green differs from the signatures above;
- any edit is needed outside the five production paths or inside a suffix;
- any test, fixture, observer, parser, console state/plugin, authority, plan,
  registry, evidence, map, body, baseline, manifest, gate, or policy changes;
- dispatch still mutates, validates catalog membership, emits effects, or
  produces C4 output directly;
- a second debug resolver, request/result/trace channel, pool mutator, factory,
  marker interpreter, or mode-to-scope mapping is introduced;
- healing directly edits Pools or is not applied through pool messages in the
  same update;
- GodMode suppresses any row outside the exact predicate, damages then heals,
  consumes armor, adds Wounded/defeat, or hides PoolDeltaApplied telemetry;
- factory/component/scope behavior is copied, ID-special-cased, or differs
  between action and console consumers;
- unknown blueprint validation moves to dispatch or partially spawns;
- established mutators, action spawn logs/errors, C1–C3, normal pool behavior,
  parsing, input, rendering, or gameplay changes;
- a test is edited, renamed, deleted, ignored, filtered, production-compiled,
  shadowed, or retained by name with weakened assertions;
- a required command is skipped, substituted, zero-test, or unclassified;
- an unauthorized report/log/evidence file appears or status is self-promoted;
- the signed gate conflicts with any higher-priority rule.

## Binary shortcut checklist

Every answer must be evidence-backed `No`. Any `Yes`, `Unknown`, missing, or
unsupported answer means `STATUS=NotComplete`.

1. Protected/reviewer-owned byte, suffix, test, or status changed?
2. Path outside the exact five-file write set changed after baseline?
3. Stash/checkout/restore/reset or revision replacement used?
4. Test body/name/inventory changed, shadowed, disabled, or production-compiled?
5. Dispatch directly mutated, validated, emitted effects, or wrote C4 output?
6. Debug gate default/bypass or duplicate request/resolver/result/trace owner?
7. Pools directly edited or heal effects missed same-frame canonical resolution?
8. GodMode broadened, post-healed, or hid applied telemetry/side effects?
9. Factory bundle, marker handling, or scope rule copied into another owner?
10. Fixture name, command, blueprint ID, entity bits, or mode special-cased?
11. Unknown blueprint created partial state or rejected before typed boundary?
12. Action spawning, mutators, normal pools, C1–C3, input, or UI drifted?
13. Required exact, paired, neighbor, integrity, or signed command skipped/zero?
14. Pre-existing work claimed, reverted, reformatted, or silently repaired?
15. Gate output used to waive a higher-priority violation or promote status?

## Required in-chat candidate handoff

```text
# Console C4 v1 Candidate Handoff Report

STATUS=CandidateGreen | STATUS=NotComplete

## Scope and integrity
- Manifest/digest used and authentication result:
- Exact authorized production files changed:
- Unexpected paths:
- Complete delta from sealed baseline, including untracked paths:
- Four protected suffix results:
- Test inventory listed/digest result:
- Revision-replacement commands used:

## Iteration record
- Each decision record in execution order:
- Ten Red signatures reproduced before implementation:
- Production responsibilities changed and reusable owners:
- Paired validation actually run per iteration:
- First remaining failure, or None:

## Boundary and behavior evidence
- Eight enabled/disabled typed-boundary rows:
- Heal request/application/final-value evidence:
- Five-row GodMode signed-delta matrix and forbidden side effects:
- Two factory-derived fingerprints:
- Tactical/Outpost scope and legacy-marker results:
- Unknown-blueprint atomic result/trace/output:
- Action-spawn and C1–C3 preservation:

## Commands and gate
- Ten exact focused commands with counts:
- Neighbor/governance commands with measured outcomes:
- Formatting/clippy/diff check:
- Signed gate steps and test totals:
- Final signed-gate status line:

## Shortcut checklist
1. Protected change? No — evidence:
2. Outside-scope delta? No — evidence:
3. Revision replacement? No — evidence:
4. Test changed/shadowed? No — evidence:
5. Direct dispatch owner? No — evidence:
6. Gate/duplicate owner? No — evidence:
7. Direct/delayed pool mutation? No — evidence:
8. GodMode semantic shortcut? No — evidence:
9. Copied factory/scope rule? No — evidence:
10. Fixture/ID/mode hardcode? No — evidence:
11. Unknown-ID boundary/atomicity shortcut? No — evidence:
12. Neighbor behavior drift? No — evidence:
13. Required command skipped/zero-test? No — evidence:
14. Pre-existing work claimed/reverted? No — evidence:
15. Status/gate overclaim? No — evidence:

## Stop-condition result and next action
- Stop condition encountered:
- Exact blocker or first remaining failure:
- Candidate-only conclusion:
```

Do not claim ReviewedGreen, VerifiedGreen, acceptance, or completion. A clean
handoff ends with the filled in-chat report and either `STATUS=CandidateGreen`
or a precise `STATUS=NotComplete` stop.
