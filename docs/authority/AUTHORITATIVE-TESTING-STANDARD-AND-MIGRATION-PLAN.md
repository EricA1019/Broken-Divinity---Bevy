# Broken Divinity Authoritative Testing Standard and Migration Plan

**Status:** Owner-approved active testing-governance and suite-migration plan

**Created:** 2026-07-26

**Plan owner:** Project owner

**Execution target:** Coding agents working in small, independently validated
batches

**Scope:** Foundation MVP requirement ownership, automated test architecture,
failure diagnostics, player workflows, visual evidence, metrics, and safe
retirement of weak or duplicate tests

**Coordination authority:** This plan owns how Foundation behavior is tested
and when evidence is sufficient.
[FOUNDATION-TEST-AND-UX-HARDENING-PLAN.md](../active/FOUNDATION-TEST-AND-UX-HARDENING-PLAN.md)
continues to own the behavior being repaired and its implementation order.

**Does not authorize:** Product P2, new gameplay, changed balance, procgen in
the Foundation path, full overworld travel, raids, colony events, sanity,
theology-driven mechanics, faction reputation, final faction canon, or a new
runtime.

---

## 1. Purpose

Broken Divinity currently has a large test suite but weak acceptance
authority. Repeated clean-suite claims were later disproved by real play:

- the player and required targets disappear outside a fixed map crop;
- build placement can trap the player;
- worker movement, occupancy, and production disagree;
- management advances time despite a paused-management contract;
- semantic categories collide visually;
- compact output truncates decisive information;
- catalog existence and substring checks have been cited as visibility,
  discoverability, and readability evidence;
- synthetic stress and count-only persistence tests overclaim what they prove.

The goal is not to maximize test count or rewrite every test. The goal is to
create a smaller set of authoritative contracts supported by useful focused
tests, granular player workflows, high-information failures, and generated
metrics.

Foundation is not accepted because most tests pass. It is accepted only when
every required contract has the correct evidence and every required metric in
this plan passes.

---

## 2. Historical Baseline at Plan Approval

The first execution batch was required to reproduce rather than trust these
then-current observed facts:

- 473 Rust `#[test]` declarations currently exist across 59 files;
- two tests are ignored;
- eight new TUI acceptance tests intentionally fail;
- therefore the expected current result is 463 passed, 8 failed, and 2
  ignored, not “473 passed”;
- approximately 54 UI assertions use substring presence;
- approximately 54 acceptance/support calls used `expect_action`, which
  settled more than one frame;
- stable identity is still violated by `first_survivor`, `first_station`, raw
  entity bits, or ECS iteration order;
- deferred procgen, sanity, overworld, and event tests remain mixed into broad
  workspace runs;
- `testing/README.md` describes an older Python analytics framework but does
  not establish that framework as current Foundation acceptance evidence.

These are historical audit inputs, not permanent magic numbers. Current
generated inventory and migration results live in
`../testing/FOUNDATION-TEST-EVIDENCE.md`.

---

## 3. Required Reading and Authority

Before executing any task, read:

1. [README.md](../README.md);
2. root [GDD.md](../../GDD.md), especially Sections 3, 6, 8, and 9;
3. [DECISIONS-TO-LOCK.md](DECISIONS-TO-LOCK.md), especially D-01 through D-18
   and D-19;
4. root [Kernel.md](../../Kernel.md), especially testing, schedule, view-model,
   persistence, and data-ownership rules;
5. [MVP-SCENARIO.md](MVP-SCENARIO.md);
6. this plan in full;
7. [FOUNDATION-TEST-AND-UX-HARDENING-PLAN.md](../active/FOUNDATION-TEST-AND-UX-HARDENING-PLAN.md);
8. [MIGRATION-AND-DEPRECATION.md](MIGRATION-AND-DEPRECATION.md);
9. `testing/FOUNDATION-TEST-EVIDENCE.md`;
10. `testing/VISUAL-ACCEPTANCE-MATRIX.md`;
11. current tests and production code.

Authority responsibilities:

| Source | Owns |
|---|---|
| GDD | Product intent and player experience |
| Locked decisions | Resolved Foundation behavior and scope |
| Kernel | Technical architecture and ownership boundaries |
| This plan | Test standard, evidence sufficiency, suite migration, metrics |
| UX hardening plan | Behavior implementation sequence |
| Contract registry | Machine-readable mapping, never new product truth |
| Tests | Evidence only |

If this plan and the UX hardening plan conflict about behavior, stop. If they
conflict only about evidence quality, this plan controls the evidence gate.

---

## 4. Scope Classification

Every test must have exactly one classification:

| Classification | Meaning | Foundation gate |
|---|---|---|
| `FoundationRequired` | Primary evidence for a locked Foundation contract | Yes |
| `FoundationSupport` | Focused support for a required contract | Runs, cannot close alone |
| `Regression` | Protects one named repaired defect | Runs, closes only that defect |
| `FutureProduct` | Product P2/P3 behavior | Separate profile |
| `DeferredInfrastructure` | Preserved non-Foundation infrastructure | Separate profile |
| `Diagnostic` | Investigation/reporting aid | Never acceptance |
| `LegacyPendingRetirement` | Old boundary awaiting replacement | Separate profile |

Rules:

- deferred tests may fail without changing Foundation acceptance status, but
  must remain visible in their own maintained profile;
- no ignored test can be `FoundationRequired`;
- no unclassified test may enter the required gate;
- no test may change classification merely to hide a regression;
- moving a test out of Foundation requires authority references and a
  replacement when it covered an active contract.

---

## 5. Contract Registry

### 5.1 Required artifact

Create:

```text
broken-divinity/testing/foundation-contracts.ron
```

Add a parser and validator in `bd_test_support` using the workspace `serde` and
`ron` versions. Do not add a second metadata format.

### 5.2 Contract record

Each record contains:

```text
id
title
scope
authority_references
player_outcome
primary_test
supporting_tests
evidence_layers
profiles
fixture_id
owner_phase
status
known_failure
```

Allowed status values:

```text
NotImplemented
Red
GreenUnreviewed
Accepted
Deferred
Retired
```

### 5.3 ID namespaces

Use:

- `SHELL-*`;
- `INPUT-*`;
- `COLONY-BUILD-*`;
- `COLONY-MGMT-*`;
- `COLONY-WORK-*`;
- `ECON-*`;
- `DUNGEON-*`;
- `PERSIST-*`;
- `PROGRESSION-*`;
- `CONTENT-*`;
- `VISUAL-*`;
- `RESILIENCE-*`.

IDs are stable after acceptance. Renaming a test does not rename its contract.

### 5.4 Registry invariants

Add tests proving:

- every `FoundationRequired` contract has exactly one primary test;
- every primary test maps to one contract;
- supporting tests map to an existing contract;
- every authority path/reference resolves;
- required evidence layers are non-empty;
- required profile names are valid;
- no required contract is ignored or deferred;
- no retired test remains a primary owner;
- every known failing test has status `Red`;
- every `Accepted` visual contract has completed matrix evidence.

The canonical metrics step also rejects the inverse drift: when the observed
workspace has zero failed tests, no `FoundationRequired` contract may remain
`Red`. A red-first worktree with observed failures remains valid TDD evidence;
a passing worktree with stale Red records cannot be reported `VerifiedGreen`.

---

## 6. Test Design Standard

### 6.1 One primary behavior

Each contract test has:

1. one named precondition;
2. one player action or domain trigger;
3. one primary outcome;
4. explicit forbidden mutations.

Do not combine unrelated requirements to reduce setup time.

### 6.2 Given/When/Then record

Every primary test documents:

```text
Contract:
Given:
When:
Then:
Must not change:
Evidence layers:
```

This record may use a structured helper or concise test comments. Failure
output must still identify the contract, case, expected result, and actual
result; a developer must not need the comments merely to identify the failure.

#### Implementation-agent guidance for red tests

A new or strengthened primary test that is intentionally handed off red must
include a concise `Implementation guidance` comment beside its contract record.
The comment must be usable by an implementation agent that has the repository
but does not have the test author's conversation history.

Where applicable, the guidance identifies:

- the reusable production boundary that should own the behavior;
- the composition or integration seam where the final outcome must survive;
- established behavior and neighboring responsibilities that must be
  preserved;
- plausible shortcuts that are not valid ways to make the test green; and
- the final workflow or evidence that must be rerun before status changes.

Use this stable vocabulary when it makes the handoff easier to scan:

```text
Implementation guidance:
- Reusable owner:
- Integration seam:
- Preserve:
- Invalid shortcuts:
- Closing evidence:
```

Omit a line only when it is genuinely inapplicable; do not fill it with
speculative instructions.

Guidance expresses constraints and architectural intent, not a mandatory patch
recipe. Unless an authority document already locks them, it must not require a
private type or function name, source-file placement, exact internal
coordinates, incidental control flow, or a particular algorithm. If the safe
production boundary is genuinely ambiguous, resolve that ambiguity with the
owner instead of encoding a speculative design in the test.

Comments do not replace executable evidence. Every required outcome and
forbidden regression must still be asserted. Conversely, an assertion must not
lock an implementation detail merely because the guidance mentions it. When
reuse or shared behavior is part of the contract, prove it through multiple
representative consumers or through a shared primitive plus final composed
workflow evidence; the word `reusable` in a comment is not proof.

Every completion-critical item listed under `Invalid shortcuts` requires an
executable false-green challenge. Use the least brittle applicable mechanism:

1. a production-path case that cannot be satisfied by injecting the expected
   projection;
2. adversarial seam poisoning that changes legacy prose or flat inputs after
   the authoritative structured projection is built;
3. a typed/layer boundary that removes the forbidden dependency; or
4. causal negative cases proving nearby non-triggers do not produce the edge.

Source-text matching is a last resort for a genuinely source-owned
architecture boundary. It must target the dependency being forbidden rather
than freeze a private function name, file layout, or exact implementation.

Before implementing an adversarial seam, record a field-influence matrix:

```text
authoritative source | poisoned competitor | derived consumer/output |
expected coherent result | mixed-source result to reject | owning assertion
```

Use plausible opposite semantics, not only conspicuous sentinel strings. The
observer must fail if any title, status, body, style, action state, or final
composition decision follows the competitor. A marker appearing and literal
decoys disappearing does not prove single ownership when another derivative
can still contradict the authoritative source. Give transport fidelity and
mixed-source coherence separate independently runnable cases when one early
panic would otherwise hide the other.

Some architectural prohibitions are not fully distinguishable from final
output. For example, formatted prose may be parsed and then reconstructed into
the same visible string. In that case, pair the behavioral coherence test with
a typed boundary or an explicit production-diff audit that names the forbidden
dependency. Do not claim complete executable coverage from a sentinel test
whose assertions do not observe the dependency's effects.

For player-facing work, prove the applicable cause → structured projection →
final composition chain. Directly injected log entries, context actions,
tooltip text, or resource rows are synthetic renderer fixtures. They may be
supporting evidence, but cannot be the sole primary evidence that production
creates, selects, or updates those facts. When a registered primary/supporting
pair divides these layers, the focused handoff must run both members together.

#### Handoff and post-green audit

Before handing an intentional red test to another implementation agent:

1. run the primary, every new completion-critical supporting test, and every
   completion-critical table row/state variant independently, and confirm each
   failure identifies its named missing behavior rather than an incidental
   fixture, an earlier assertion, or copy detail. Split distinct failure modes
   into supporting tests or provide a deterministic case filter when an early
   panic otherwise hides later required evidence;
2. record the known failing cases and any established behavior most likely to
   be sacrificed by a superficial fix; and
3. ensure the implementation guidance and executable forbidden-regression
   assertions agree.

The author must also prove that directly injected fixture state is used only
for renderer or adapter isolation and is not being mistaken for production
reachability. If the fixture inserts the decisive state and a normal production
update immediately replaces it, do not hand that red test to an implementation
agent. Add a paired production-path case or classify the observer as defective.
Production behavior must never be changed merely to keep synthetic fixture
state alive.

#### Signed candidate handoff and role authority

Red-test authorship, production implementation, and green acceptance are
separate authorities. The author/reviewer owns tests, observers, fixtures,
plans, status ledgers, and the handoff manifest. The implementation agent owns
only the authorized production slice and may produce candidate evidence; it
may not make its own work acceptable by changing what is measured or by
promoting status.

The handoff must state one exact, exhaustive production write set using
repository-relative paths. Do not qualify it with `principally`, `expected`,
`likely`, wildcards that include unrelated owners, or an open-ended phrase.
Before each edit the implementation agent checks the path against this set;
after each loop iteration it inspects the complete changed-file list. Any
required path outside the set is a mandatory stop-and-report condition. The
agent must not edit first and request retroactive approval.

The author also seals a baseline changed-path inventory with enough status and
content identity to distinguish pre-existing user work from candidate edits in
a dirty worktree. Scope validation compares the candidate state with this
baseline rather than assuming `git status` was initially clean. The
implementation agent must preserve, neither claim nor silently repair, and not
revert an out-of-set or pre-existing diff without separate authority. If the
candidate cannot attribute a path reliably, it stops and returns the evidence
to the reviewer for disposition.

Stop conditions have higher precedence than implementation-loop and gate
instructions. A correct scope-conflict response is `STATUS=NotComplete` plus
the blocked test, required path, conflicting constraint, and observed failure.
It is not an incomplete attempt that should be worked around.

For a constrained implementation model, structure the prompt as a low-freedom
procedure. Put this priority ladder before the red-to-green command:

1. preserve protected authority, sealed baseline state, and user work;
2. obey stop conditions and the exact write set;
3. satisfy the semantic contract through the authorized production path;
4. preserve named neighboring behavior;
5. run every focused, paired, neighboring, and role-appropriate gate; and
6. report measured evidence without upgrading its meaning.

State that lower-priority success never waives a higher-priority violation.
Before each patch, require a short decision record naming the current failing
case, missing cause/projection/composition responsibility, reusable owner,
authorized target path, and next paired validation. If any field is unknown,
the model stops instead of guessing. Limit one iteration to one production
responsibility so a passing aggregate cannot hide speculative changes.

End the prompt with a binary shortcut-rejection checklist tailored to the
contract. At minimum ask whether the candidate hardcodes a fixture, case,
profile, coordinate, or content ID; duplicates a shared rule; parses rendered
prose; preserves injected state; changes unrelated simulation/domain behavior;
skips an exact or paired test; satisfies only an isolated layer; claims
pre-existing work; or treats gate output as authority. Any `yes` or `unknown`
forces `STATUS=NotComplete`. Require the final report to show the decision
record and checklist result; unsupported `PASS` is not evidence.

When the contract requires DRY or a shared owner, name the semantic
transformation that is owned once. Identical output assembled independently in
multiple consumers is duplication, not reuse. Downstream layers may perform
layout, wrapping, styling, transport, or serialization, but may not rederive
the same domain facts, wording rule, or applicability decision. Require a
typed boundary, adversarial seam, or alternate-data consumer to prove the
sharing. A named-fixture-only pass cannot support a `No hardcode` claim.

Immediately before handoff, the author creates a version-2 RON manifest with:

```text
(
  version: 2,
  contracts: ["CONTRACT-ID"],
  baseline_path: "testing/<batch>-BASELINE.ron",
  exact_production_write_set: ["repository/relative/production-path"],
  protected_files: [
    (path: "repository/relative/path", sha256: "<64 lowercase hex>"),
  ],
)
```

The manifest names exactly the required `Red` contracts in the batch. Its
protected set includes the active plan, primary and completion-critical
supporting tests, their fixtures and observers, and every repository authority
or status record required by `scripts/test-gate.sh`. The gate always requires
at least `AGENTS.md`, `GDD.md`, `Kernel.md`, locked decisions, the root Cargo
workspace definition, itself, the ignored-test allowlist, contract registry,
requirement map, test evidence, visual acceptance matrix, this standard, and
the candidate verifier, contract reporter, registry implementation, their
crate definition, and their governance/integrity tests. A candidate may not
modify the code that decides whether its own handoff passes. The author adds
any task-specific plan, crate manifest, authority, or test files; satisfying
the minimum set is not permission to omit affected observers.

The manifest must also protect the exact active implementation instructions as
a digest-free handoff body. The manifest digest is supplied separately to
avoid a circular hash. A launcher or user message may pair that protected body
with the digest, but it cannot add or override production scope, priorities,
stop conditions, required commands, or acceptance. Protecting an obsolete
prompt while leaving the active instruction body mutable invalidates the
handoff even when every generic minimum file is sealed.

When a production file in the exact write set also contains reviewer-owned
tests, fixtures, or observers, test-name inventory protection is insufficient:
the candidate could retain every name while weakening its body. Before handoff,
either extract that reviewer-owned code to a fully protected file or add a
manifest `protected_suffixes` entry whose unique `start_marker` begins at the
reviewer-owned suffix and whose SHA-256 covers every byte from that marker to
end of file. The guard must allow edits before the marker, reject marker
deletion or duplication, and reject any suffix-byte change. Do not use a suffix
seal when production code follows the protected region; extract the protected
code instead.

The author computes the manifest SHA-256 after its contents are final and
provides that digest separately in the handoff prompt. The implementation
agent must not regenerate the digest, rewrite the manifest, remove a protected
entry, edit a protected file, change contract status, or reconcile ledgers.
Any needed test repair is reported as an observer defect and separately
authorized.

The implementation agent's closing gate is:

```text
bash scripts/test-gate.sh \
  --candidate-manifest <author-supplied-path> \
  --manifest-sha256 <author-supplied-digest>
```

Candidate mode verifies the signed manifest before and after the suite. It
accepts a zero-failure workspace only when the manifest names the exact set of
required contracts still recorded `Red`, the manifest and protected baseline
name the same exact production write set, and every Git-visible delta from that
baseline is inside it. Any self-promotion, unlisted required Red contract,
missing mandatory protected file, protected hash change, manifest digest
change, or unauthorized added/untracked, modified, deleted, or renamed path
fails the gate. Success reports `STATUS=CandidateGreen`, never Verified or
Reviewed green.

Mechanical scope validation is necessary but does not replace the agent's
baseline/diff audit. Ignored build artifacts are outside the production delta,
and the reviewer must still inspect authorized-path changes for erased
pre-existing work, hardcoding, duplicated ownership, or other semantic scope
violations.

Only an independent reviewer may audit the production diff and false-green
challenges, reconcile the registry, requirement map, acceptance matrix, and
evidence in one review change, and then run `bash scripts/test-gate.sh` without
candidate arguments. Canonical success reports `STATUS=VerifiedGreen`; the
manual/visual and owner review requirements for `ReviewedGreen` still apply.

If a candidate run reveals a missing or weak observer, the reviewer strengthens
the smallest deficient test, records the false-green reason, restores all
affected ledger entries to accurate `Red`, creates a new signed manifest and
digest, and starts a new implementation handoff. Reusing an old signature
after test or authority changes is prohibited.

When the implementation agent reports green, review the production diff and
the final player workflow before accepting the result. A passing assertion is
necessary evidence, not automatic proof that the contract was fulfilled.
Specifically check that the implementation did not:

- delete, hide, or recolor neighboring behavior to satisfy the observer;
- special-case only the named fixture, screen, profile, or coordinate;
- duplicate a shared rule in individual consumers;
- alter the test, fixture, observer, registry status, or evidence requirement;
  or
- satisfy isolated output while failing final composition or required PTY
  behavior.

Also perform an explicit false-green challenge: ask whether the evidence would
still pass if production never created the decisive data, a generic panel were
only renamed, formatted prose were parsed to reconstruct structured state, a
placeholder such as `unbound` satisfied a weak string proxy, an enabled action
had no configured reducer route, category detail ignored authoritative state,
duplicate targets collapsed, simultaneous entries emitted one fact per entity,
or only the named fixture/category were implemented. Any yes answer is an
evidence defect that must be repaired before promotion.

If this audit exposes a false green, preserve the production evidence, improve
the smallest deficient observer or forbidden-regression assertion, return the
contract to an accurate red state, and record why the previous test passed.
Do not respond by snapshotting the entire implementation or prescribing the
exact corrective patch. The repaired test must remain strict about the owned
outcome and flexible about unrelated design choices.

#### Completion state protocol

Use exactly these states for test-driven work:

- `CandidateGreen`: the focused primary/supporting tests and the signed
  candidate gate pass while protected authority and named contract records
  remain unchanged and `Red`. This is an implementation handoff state, never
  completion.
- `VerifiedGreen`: focused and neighboring targets pass and the current
  reviewed worktree receives a zero exit from canonical
  `bash scripts/test-gate.sh` with consistent measured totals and no required
  `Red` status drift.
- `ReviewedGreen`: `VerifiedGreen` plus production-diff review, required
  workflow and PTY evidence, authority drift review, and accurate registry,
  matrix, and evidence records.

Only `ReviewedGreen` may be described as complete or done. An implementation
agent may report `CandidateGreen`; an acceptance reviewer or owner performs the
post-green audit and promotes the state. A nonzero canonical gate is
`NotComplete`, regardless of focused-test success.

No bug report, diagnostic label, `pre-existing` classification, claimed
structural conflict, owner-question request, or partial test count waives a
failed canonical gate. Only an explicit owner instruction may waive a required
gate, and the waiver must name the exact failure and evidence limitation. A
diagnostic or bug log is evidence only and cannot add itself to the authority
chain or call itself a source of product/test truth.

Before claiming two tests or contracts conflict:

1. restate both requirements without referring to helper implementation;
2. show that no player-visible or domain result can satisfy both requirements;
3. inspect whether parsing, normalization, fixture geometry, glyph handling, or
   another observer mechanism caused the failure; and
4. treat the problem as an observer defect when the final output satisfies both
   semantic requirements but the helper rejects it.

A helper's inability to recognize approved output is not a contract conflict.
Text-completeness observers should inspect the owned semantic region and
normalize content independently of approved border glyphs or styles.

Every implementation-loop handoff is pasted into the current chat using this
template. It is not permission to create a repository document; a handoff,
bug-report, evidence, or log file may be written only when its exact path is in
the authorized write set.

```text
IMPLEMENTER IN-CHAT HANDOFF
Role: implementation agent
Batch / iteration:
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
Shortcut checklist with evidence-backed answers:
First remaining failure or stop condition:
Next legal action or reviewer decision needed:
```

Write `Not run` or `Unverified` for missing evidence. Never label a zero-test
invocation as passing. The report must distinguish current measured results
from historical claims and must not call the batch done or complete.

After auditing that report and the current worktree, the independent reviewer
pastes a continuation handoff into chat:

```text
REVIEWER IN-CHAT CONTINUATION HANDOFF
Role: independent reviewer
Reviewed batch / implementation iteration:
Reviewed status: CandidateGreen | VerifiedGreen | ReviewedGreen | NotComplete
Evidence independently reproduced:
Implementation claims accepted:
Claims rejected or still unverified:
Production diff / false-green findings:
Baseline, untracked-path, and exact-scope disposition:
Authority, registry, evidence, workflow, and manual / visual / PTY state:
Current blocker or owner decision:
Next authorized batch and exact production write set:
New protected body / baseline / manifest / separate digest, if required:
Exact resume point and first commands:
```

The reviewer must not tell an implementation agent to continue under an old
seal after changing scope, authority, tests, fixtures, observers, or protected
instructions. If any required acceptance field is failing, stale, or open,
the reviewed status remains `NotComplete`.

### 6.3 Table-driven cases

Use one table when every row exercises the same rule:

- viewport edges;
- terminal profiles;
- station catalog entries;
- resource thresholds;
- visual category pairs;
- save checkpoints.

Each row requires a stable case ID and case-specific failure output.

### 6.4 Workflow tests

Workflow tests validate seams and player reachability. They do not own:

- formulas;
- catalog values;
- pathfinding internals;
- serialization field details;
- exact visual styling.

Those remain in focused contract tests.

Edge-triggered workflow contracts include representative non-causes as
forbidden regressions. When applicable, initialization, idle projection,
rendering, resize, Help, save/load restoration, target spawning, and unrelated
state mutation must not impersonate the authorized action or domain trigger.
When the edge is rearmable, leaving range/state must be silent and a later
production re-entry must emit exactly once again; an `ever seen` set that never
rearms is a false green.

Advertised-action workflow evidence distinguishes three facts that must not be
collapsed into one boolean or a plausible-looking row:

- **applicable**: the selected target and current domain state permit the
  action in principle;
- **bound/reachable**: the current input state has a configured control and
  reducer route for that semantic action; and
- **enabled/executable**: activating that control now reaches the claimed
  paused or gameplay result.

An action is not executable merely because its label is present, its key hint
is non-empty, or a different command owns the displayed key. Until a reducer
or owner-locked binding exists, a preview action remains visibly disabled with
a truthful reason. Tests for action panels, tooltips, context menus, Help, and
footers must cross-check enabled rows against the current binding and input
state, and must exercise at least one advertised enabled control through the
production input path. `unbound`, invented key hints, and keys that resolve to
normal-world commands are explicit false-green challenges.

Projection evidence is insufficient when the final UI owns action truth. At
every required profile, final composition must preserve focus/selection,
enabled/disabled styling, denial reasons, and agreeing controls. A renderer
that hides the reason, styles an unavailable row as active, or flattens every
target's actions into one focused region fails even when the structured rows
were correct before composition.

Multi-target interaction tests use at least one duplicate-label case. They
prove that stable selectors and player-visible disambiguation do not collapse
to display name, query order, or raw ECS identity; that the focused target's
details and actions do not flatten together with other targets; and that a
single historical notification plus a count does not replace the complete
deterministic current target list. Printing all distinguishable target actions
inside one focused Context region is flattening, not a target picker or focused
action set.

Category and adapter matrices include at least one authoritative state variant
that changes both relevant detail and action applicability. Default
station/node/actor rows are insufficient when fixed category prose could pass.
Use representative operational/construction, renewable/depleted,
staffed/unstaffed, idle/working/blocked, progress, cargo, or target changes as
owned by the contract. The test remains semantic about copy while proving the
projection responds to domain state.

### 6.5 Assertion stability

Use exact assertions for:

- resource and pool deltas;
- time;
- typed states;
- action/message counts;
- occupancy;
- relationships;
- persistence fingerprints;
- viewport coordinates.

Use semantic/structural assertions for:

- text completeness;
- controls;
- selected state;
- panel containment;
- category distinction;
- visibility/discoverability.

Use full snapshots only for approved canonical scenes. Never snapshot raw
entity IDs, host paths, full unbounded logs, or nondeterministic ordering.

### 6.6 Visual structure and observer integrity

Visual assertions must prove the complete semantic claim they name.

- Claims such as `closed`, `continuous`, `contained`, `all`, and `every`
  require a universal assertion over the complete owned structure. Four
  corners, a glyph count, or a matching pair of endpoints does not prove a
  connected frame, divider, path, gauge, or boundary.
- Connected structures must verify their topology: required endpoints or
  corners, every intervening segment, explicitly permitted junctions, and the
  resolved style of the structural cells. Allowed glyph sets must be finite
  and justified by the visual contract.
- Assert the stable semantic boundary, not incidental internal coordinates.
  For example, a terminal outer-frame contract may own the final terminal
  perimeter while leaving panel rectangles, copy, and internal spacing free
  to change.
- When later widgets can overwrite earlier rendering, isolated widget output
  is supporting evidence only. The primary visual proof must inspect the final
  composed buffer; real-PTY evidence remains required where the registry lists
  it.
- Style evidence must be tied to the intended semantic token or region. A
  count of cells sharing the same resolved foreground is insufficient when
  unrelated terrain, entities, or text can use that color.
- Every reusable visual observer or detector must have a negative mutation
  probe at the smallest granularity it claims to detect. At minimum, applicable
  probes independently alter one glyph, one resolved-style property, or one
  geometry cell and prove the observer rejects the result.
- Failures report the stable case/profile, the first broken coordinate or
  semantic element, the expected and actual glyph/style, and a useful visual
  crop. They do not report only `frame missing` or an aggregate count.

These rules make the invariant strict while permitting unrelated copy,
content, and layout changes. If a visual contract cannot identify which
boundary, region, or topology it owns, clarify the contract before writing its
test.

### 6.7 Prohibited patterns in new primary tests

- `first_survivor` or `first_station`;
- raw entity-bit identity;
- ECS query-order identity;
- direct mutable `World` access;
- hidden frame settling;
- conditional required assertions;
- `output.contains(...)` as sole readability/visibility proof;
- corner, endpoint, or glyph-count presence as sole proof of a connected
  visual structure;
- raw resolved-color counts as sole proof that a semantic role rendered;
- isolated widget output as sole proof of a structure that can be overwritten
  during final composition;
- count equality as sole persistence proof;
- sleeps;
- wall-clock timing as functional correctness;
- production logic copied into expected-value helpers;
- test-only gameplay resolvers;
- implementation comments used as a substitute for an executable outcome or
  forbidden-regression assertion;
- directly injected decisive projection data as the sole evidence that a
  player-facing production workflow creates that data;
- output-only assertions for a contract that also owns structured projection
  or causality;
- red-test guidance that prescribes unlocked private names, file placement,
  coordinates, control flow, or algorithms as the only acceptable solution.

---

## 7. Evidence and Diagnostic Model

### 7.1 Evidence layers

| Layer | Proves |
|---|---|
| Domain | Rule or calculation |
| Schedule | Order and exact-once execution |
| State diff | Authorized and forbidden mutations |
| Projection | Semantic player-visible state |
| Buffer/layout | Glyph, style, geometry, clipping |
| Input state machine | Physical controls and modal behavior |
| Workflow | Complete player journey |
| Persistence | State and continuation equivalence |
| PTY | Real terminal behavior |

A registry record states which layers are mandatory.

### 7.2 Standard failure report

Every primary helper reports:

```text
contract_id
case_id
fixture_id
seed
profile
workflow_step
input
frames_advanced
expected
actual
state_diff
trace_tail
replay_tail
visual_crop
```

Fields not applicable are marked `n/a`; they are not silently omitted.

### 7.3 Normalized state diff

The diff includes stable:

- mode/session phase;
- day/turn;
- player identity, position, pools, inventory;
- survivor name/ID, position, task, target, activity;
- station content ID, position, staffing, effect;
- resource node identity/type, position, depletion;
- colony resources/storage;
- active/completed run state;
- progression and virtues;
- entity counts by scope;
- RNG/replay origin where relevant.

Collections are sorted by stable identity. Raw ECS IDs are excluded.

### 7.4 Workflow transcript

Each player workflow records:

- physical key;
- semantic command;
- interaction state before/after;
- accepted/denied result;
- time change;
- mode change;
- decisive visible feedback.

On failure, print the last successful step and the next expected step.

### 7.5 Visual diagnostics

Visual failures print:

- terminal size;
- panel rectangles;
- viewport world bounds;
- semantic projections;
- relevant buffer crop with coordinates;
- glyph/style/layer/priority;
- allowed and unexpected changed regions.

---

## 8. Foundation Contract Families

### 8.1 Shell and lifecycle

Create primary contracts for:

- New Game reaches Outpost exactly once;
- title Load does not start a new run;
- missing/corrupt save feedback is recoverable;
- quit requests one clean shutdown;
- terminal alternate screen and cursor restore;
- stable idle state does not redraw;
- resize redraws once without stale cells.

### 8.2 Input and controls

- every advertised control executes in its mode;
- unadvertised controls do not mutate gameplay;
- Press/Repeat/Release behavior is explicit;
- queue order is preserved;
- queue overflow is bounded and visible;
- modal input never leaks;
- footer, Help, action panel, configuration, and runtime agree.

### 8.3 Build workflow

Workflow:

```text
B → select → navigate → Enter → place → move preview
→ Enter build or Escape cancel
```

Atomic contracts:

- opening, selection, navigation, and cancellation are paused;
- selection exposes complete cost/effect/availability/staffing;
- placement does not move the player;
- valid and invalid preview states differ semantically;
- invalid preview exposes typed reason;
- rejection changes no payment/time/entity state;
- acceptance spends and advances exactly once;
- accepted placement preserves gate reachability.

### 8.4 Task management and station staffing

Create separate workflows for `c` and `e`.

Contracts:

- distinct mode/title;
- named stable survivor selection;
- named stable station selection;
- paused open/navigation/confirmation/cancellation;
- confirmation changes only intended relationship;
- cancellation is atomic;
- assignment does not immediately move workers;
- feedback identifies survivor, target, and activity.

### 8.5 Viewport and visual language

- player visible at every shelter position for each profile;
- viewport clamps at each edge;
- every layer uses one transform;
- assigned off-screen targets remain discoverable;
- every active semantic category has symbol/style/legend;
- station/resource tokens are distinct;
- staffed/unstaffed and worker activities are distinct;
- player/survivor cannot be hidden by lower gameplay layers;
- ASCII fallback does not depend on color;
- compact decisive text is complete.

### 8.6 Worker movement and production

- Idle does not move;
- assignment becomes EnRoute without movement;
- one Outpost turn permits at most one cardinal step;
- Tactical turns do not move colony workers;
- blockers and reservations are respected;
- survivors do not stack or occupy target tiles;
- valid arrival becomes Working;
- no route becomes Blocked with reason;
- EnRoute/Blocked produce zero;
- correctly Working produces once;
- wrong resource type produces zero;
- Rest equals equivalent individual turns.

### 8.7 Economy

- every day boundary runs one transaction;
- Tactical/Outpost boundaries agree;
- food, station output, gathering, mood, and summary occur once;
- summary equals authoritative delta;
- forecast equals execution;
- zero-Supplies recovery remains reachable and discoverable;
- Storage rejects before payment;
- catalog owns station facts.

### 8.8 Fixed dungeon

Workflow:

```text
Outpost → paid entry → move/explore → encounter → combat
→ loot → exit → extraction → Outpost
```

Contracts:

- entry costs exactly two Supplies;
- denial is atomic;
- fixed content loads without procgen;
- entrance/enemy/loot/exit are reachable;
- movement visibly changes state;
- default enemy survives one default attack;
- enemy phase occurs once;
- invalid attacks are atomic;
- pickup is explicit;
- extraction requires exit and explicit action;
- loot applies once;
- colony state survives;
- defeat grants no loot;
- restart uses shelter return spawn.

### 8.9 Persistence

Required checkpoint cases:

- clean Outpost;
- built station;
- assigned worker;
- EnRoute;
- Working;
- before day boundary;
- after day boundary;
- active dungeon;
- carrying loot;
- extracted;
- Game Over.

For each applicable case:

- fingerprint equality;
- relationship validity;
- projection equality;
- deterministic next action;
- no duplicate costs/results;
- failed load atomicity.

### 8.10 Progression, factions, and content

- Quick Attack improves Melee once;
- Quick Attack expresses Thumos once;
- combat survival expresses Fortitude once;
- item use improves Medicine and expresses Temperance once;
- rejected actions grant nothing;
- six virtues plus Kleos exist;
- two placeholder factions load from data;
- a third valid faction needs no Rust branch;
- hostility uses disposition;
- invalid content reports path and IDs;
- ambiguous active symbols fail validation.

---

## 9. Metrics

### 9.1 Generated report

Add a report command under `bd_test_support` that reads the contract registry
and test evidence. It outputs human-readable text and deterministic JSON.

The report must not infer pass status from source text alone. Test execution
results and snapshot/PTY evidence are explicit inputs.

### 9.2 Acceptance metrics

Foundation acceptance requires:

- 100% required contracts registered;
- 100% required contracts with exactly one primary owner;
- 100% required evidence layers passing;
- zero orphaned active tests;
- zero duplicate primary owners;
- zero ignored required tests;
- zero acceptance tests using unstable identity;
- zero acceptance helpers with hidden updates;
- zero conditional required assertions;
- zero pending snapshots;
- zero unexplained visual diffs;
- every canonical workflow passing at required profiles;
- every advertised control executable;
- every required failure report containing mandatory diagnostics;
- zero intermittent failures in the repetition profile;
- all selected critical mutation probes killed.

### 9.3 Non-acceptance metrics

Record but do not use alone:

- total test count;
- line/branch coverage;
- wall-clock duration;
- snapshot count;
- assertion count.

Runtime budgets are set only after Phase 0 measures a clean baseline on the
project machine. Hardware-dependent wall-clock thresholds do not become
functional correctness tests.

---

## 10. Current Suite Migration Rules

### Retain

- pure pool/combat/pathfinding/relationship/ID tests;
- strengthened schedule tests;
- loader/registry validation;
- atomic save validation;
- input queue and render invalidation tests.

### Strengthen

- `foundation_actions.rs`;
- `colony_day_cycle.rs`;
- `phase6_input.rs`;
- `persistence.rs`;
- `entity_scope.rs`;
- `progression_factions.rs`;
- `input_help.rs`.

### Consolidate after replacements pass

- `foundation_scenario.rs`;
- `foundation_stabilization.rs`;
- `mvp_correction.rs`.

### Move to separate profiles

- procgen;
- sanity;
- overworld;
- colony events;
- deferred narrative/Gabriel;
- wall-clock procgen timing.

### Retire only after successor acceptance

- direct-mutation legacy combat/pickup tests;
- ignored diagnostic snapshot;
- duplicate count-only persistence tests;
- duplicate summary-only determinism tests;
- superseded substring-only tests;
- superseded synthetic stress tests.

Every retirement record names:

```text
old test
old proof
reason insufficient
replacement contract
replacement test
replacement validation result
date
```

---

## 11. Execution Protocol

For every task:

1. inspect the registry row and authority;
2. identify the current primary/supporting tests;
3. write or strengthen the smallest red test;
4. run only that test;
5. capture the intended failure report;
6. implement only the authorized production or harness change;
7. rerun focused tests;
8. run affected workflow;
9. run phase gate;
10. update registry/report;
11. perform GDD drift check;
12. record completion.

Stop when:

- a requirement is absent from authority;
- a test requires a deferred feature;
- two contracts claim the same primary behavior;
- replacing a test would lose unique coverage;
- a red test passes before implementation and the reason is unknown;
- a public production API would exist only for tests;
- diagnostics require duplicating production logic;
- a snapshot change is unexplained.

---

## 12. Phase 0 — Reproducible Inventory and Authority Map

### Tasks

#### 0.1 Generate the real suite inventory

Record:

- tests listed;
- passed/failed/ignored;
- target and module;
- runtime profile;
- classification;
- use of unstable identity;
- direct fixture mutation;
- hidden settling;
- substring/snapshot behavior;
- authority mapping.

Correct every existing report that confuses listed with passed.

#### 0.2 Audit the legacy Python testing framework

Classify every file under `testing/` as:

- active and reproducible;
- useful diagnostic;
- obsolete;
- unverified;
- generated artifact.

Do not install Python dependencies or cite the old framework as evidence
without a clean reproducible run and explicit owner.

#### 0.3 Seed the contract registry

Translate GDD Foundation scope and D-01–D-18 into atomic records. Mark the
eight current visual failures `Red`.

#### 0.4 Create the migration ledger

Every current test receives one disposition:

- retain;
- strengthen;
- consolidate;
- move;
- retire after replacement.

### Validation

```bash
cargo test --workspace -- --list
rg -n "#\\[ignore|first_survivor|first_station|output\\.contains|expect_action" crates
git diff --check
```

### Completion gate

- counts are generated and arithmetically consistent;
- every current test is inventoried;
- every required contract has an initial status;
- no test is deleted.

---

## 13. Phase 1 — Registry Validation and Metrics Report

### Primary files

- `testing/foundation-contracts.ron`;
- `crates/bd_test_support/Cargo.toml`;
- new contract metadata/report modules in `bd_test_support`;
- new `crates/bd_test_support/tests/contract_registry.rs`;
- evidence ledgers.

### Red tests

- duplicate primary owner is rejected;
- missing authority is rejected;
- missing primary test is rejected;
- ignored required test is rejected;
- deferred required test is rejected;
- unknown evidence layer/profile is rejected;
- accepted visual contract without matrix evidence is rejected.

### Completion gate

- registry parser has no gameplay knowledge;
- report is deterministic;
- text and JSON totals agree;
- invalid fixture tests prove every validator branch.

---

## 14. Phase 2 — Harness Truth and Diagnostic Infrastructure

### Tasks

#### 2.1 Explicit frame control

Replace hidden settling with:

- submit input/action;
- advance exactly one frame;
- settle with named predicate and bounded frame count.

#### 2.2 Stable selectors

Select player, survivors, stations, nodes, and content by stable identity.

#### 2.3 Normalized fingerprint and diff

Implement Section 7.3 without raw ECS IDs.

#### 2.4 Workflow transcript

Record physical input through visible result.

#### 2.5 Diagnostic sensitivity

Deliberately alter one field in test fixtures and prove the failure report
identifies that field, contract, and step.

### Completion gate

- no primary test uses hidden updates or query order;
- diagnostics are deterministic;
- no mutable world is exposed;
- no expected-value helper duplicates a game rule.

---

## 15. Phase 3 — Classify and Assign Current Tests

### Tasks

- assign every test to a registry contract/classification;
- identify primary ownership conflicts;
- demote broad tests to supporting evidence where appropriate;
- separate deferred profiles;
- identify exact replacement for every retirement candidate;
- prohibit new uncatalogued tests.

### Completion gate

- zero orphaned tests;
- zero duplicate primary owners;
- no required behavior owned only by legacy/deferred coverage;
- no deletion yet.

---

## 16. Phase 4 — Atomic Domain and Schedule Contracts

Strengthen actions, time, economy, occupancy, progression, transition,
persistence validation, and schedule ordering.

Each contract receives:

- exact input;
- exact result count;
- exact allowed state diff;
- forbidden mutation diff;
- deterministic case ID.

Consolidate only duplicated support tests whose stronger primary replacement is
already green.

---

## 17. Phase 5 — Input and Menu Workflows

Create dedicated production-key targets:

- shell/title lifecycle;
- Build selection/placement;
- task management;
- station staffing;
- Help/inventory;
- save/load feedback.

Each modal tests open, navigate, confirm, cancel, Repeat, Release, and leakage
as discrete cases.

Run at 80x24 and compact-critical 60x20 checkpoints.

---

## 18. Phase 6 — Visual Contract Infrastructure

Implement:

- semantic observations;
- canvas snapshots;
- style-cell snapshots;
- geometry observations;
- transition diffs;
- canonical deterministic fixtures.

Move the eight current red tests out of the broad TUI unit module into named
visual contract targets without weakening them.

No snapshot is accepted until structural and semantic assertions pass.

---

## 19. Phase 7 — Colony Player Paths

Execute the Build, assignment, staffing, worker movement, physical work, Rest,
day transaction, recovery, viewport, egress, and Help contracts.

Validate each atomic contract first, then the canonical colony workflow.

Do not let the workflow own formulas already tested by atomic contracts.

---

## 20. Phase 8 — Dungeon Player Path

Create one canonical production-key workflow with named checkpoints:

1. paid entry;
2. visible arrival;
3. exploration movement;
4. encounter;
5. tactical action;
6. enemy response;
7. defeat hostile;
8. loot detour;
9. pickup;
10. reach exit;
11. explicit extraction;
12. colony result.

Add a separate defeat/restart workflow. Report the last successful checkpoint
on failure.

---

## 21. Phase 9 — Persistence Matrix

Replace scattered count comparisons with Section 8.9 checkpoint cases.

For each case, prove:

- fingerprint equality;
- relationship validity;
- visual equality where applicable;
- deterministic continuation;
- no duplicate side effects.

Retire duplicate persistence tests only after every unique proof is mapped.

---

## 22. Phase 10 — Content, Progression, and Extensibility

Add data-driven matrices for:

- station catalog;
- placeholder factions;
- visual symbol registry;
- fixed dungeon content;
- representative skills/virtues.

Tests must prove extension without adding Rust branches. They must not require
final faction canon or complete virtue balance.

---

## 23. Phase 11 — Adverse, Property, and Stress Profiles

### Required deterministic matrices

- resource minimum/zero/threshold/max;
- every shelter edge;
- every station placement candidate;
- every worker target state;
- every visual category pair;
- every persistence checkpoint boundary.

### Stress

- production colony/dungeon cycles;
- bounded input/message behavior;
- entity scope stability;
- repeated save/load;
- deterministic seeded action sequences.

Wall-clock performance is reported, not used as gameplay correctness.

---

## 24. Phase 12 — Safe Consolidation and Retirement

For every candidate:

1. run old test;
2. run replacement;
3. compare proof;
4. confirm registry ownership;
5. preserve unique regression cases;
6. record retirement;
7. remove or move old test;
8. run full affected profiles.

The suite may become smaller. Contract coverage and diagnostic quality may not
decrease.

---

## 25. Phase 13 — Final Acceptance

### Required automated gates

- registry validation;
- Foundation primary contracts;
- supporting regression suite;
- canonical workflows;
- visual matrix;
- persistence matrix;
- deterministic adverse/property profile;
- content validation;
- formatting, compilation, strict Clippy, and whitespace.

### Required real-terminal gates

At 80x24 and 60x20:

- title/new/load;
- Help;
- Build valid/invalid/cancel;
- task management;
- staffing;
- worker progression;
- Rest/day summary;
- save/load;
- dungeon loop;
- extraction;
- defeat/restart;
- resize and terminal restoration.

### Final GDD review

Review Sections 3, 6, 8, and 9 and every locked decision. Confirm:

- Foundation loop is playable;
- no deferred system became required;
- practical survival remains the focus;
- tests prove player behavior rather than module existence.

### Final completion criteria

This plan is complete only when:

- all Section 9 acceptance metrics pass;
- every required contract is `Accepted`;
- no known visual red test remains;
- no required test is ignored;
- no pending snapshot exists;
- no unexplained test retirement exists;
- all player workflows pass at required profiles;
- real terminal and automated evidence agree;
- documentation, registry, ledgers, and actual test results report the same
  status.

---

## 26. Phase Completion Record

Append:

```text
### Testing Phase N completion — YYYY-MM-DD

Contracts affected:
Tests retained:
Tests strengthened:
Tests added:
Tests moved:
Tests retired:

Red evidence:
Green evidence:
Diagnostic evidence:

Focused commands:
Profile commands:
Result counts:

Metrics before:
Metrics after:

GDD sections reviewed:
Drift:

Residual risks:
Next phase ready: yes/no
```

A phase is not complete when any field is omitted, required tests are red, or
reported metrics do not match actual execution.
