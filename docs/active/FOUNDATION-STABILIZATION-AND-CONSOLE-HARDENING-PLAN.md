# Foundation Stabilization and Developer Console Hardening Plan

> **Status:** Active — S0 reviewer recovery complete; C1 v1 rejected after
> independent review; C1 v2 causal-submission and explicit-ordering handoff
> sealed Red for implementation
> **Created:** 2026-08-09
> **Purpose:** Restore a trustworthy canonical baseline, then stabilize the
> existing `bd_console` feature through sealed, test-first implementation
> batches suitable for a constrained implementation model.

## 1. Outcome

This plan is complete only when the current Foundation behavior is preserved,
the canonical development gate is green, and the existing developer console
has one reusable input owner, one submission path, deterministic final
composition, an explicit gated debug-mutation path, and production-path
evidence for every behavior it advertises.

The console is developer tooling. It must not become a second gameplay engine,
a second renderer, or a privileged path that silently bypasses the kernel.

The intended final flow is:

```text
physical key
  -> bd_tui input adapter
  -> one bd_console key reducer
  -> one parsed command path
  -> read-only query OR gated DebugIntent/debug effect
  -> bd_core debug resolver / existing canonical resolver
  -> traceable result
  -> ConsoleState output
  -> bd_tui final composition invokes the reusable console overlay last
```

## 2. Authority and scope

Read in this order before authoring tests or implementation:

1. `GDD.md`, especially Sections 8–10;
2. `Kernel.md`, especially mutation ownership, signal discipline, UI boundary,
   debug boundary, TDD, and phase-failure rules;
3. `docs/authority/DECISIONS-TO-LOCK.md`, especially D-02 and D-15–D-19;
4. `docs/authority/AUTHORITATIVE-TESTING-STANDARD-AND-MIGRATION-PLAN.md`;
5. `docs/active/FOUNDATION-UI-IMPROVEMENT-PLAN.md`;
6. `docs/active/FOUNDATION-FACTORY-EVENT-PIPELINE-PLAN.md`;
7. this plan;
8. `testing/foundation-contracts.ron` and the latest evidence ledger;
9. current production code and tests.

This is stabilization of existing code. It does not make raids, events, or the
developer console part of the player-facing Foundation acceptance journey.
Their tests remain visible and must pass, but the reviewer must classify their
registry rows accurately rather than silently treating deferred or developer
infrastructure as a new Foundation product requirement.

## 3. Measured starting state

Independent review at commit
`c0602287a00a84c6a74a2be6c0be8c1dfebdf7cb` established:

| Check | Observed result |
|---|---|
| Worktree | Clean; `HEAD == origin/main` |
| Canonical gate | 5 steps passed, 6 failed; `STATUS=NotComplete` |
| Gate inventory | 822 listed, 678 passed before stop, 1 failed, 0 ignored |
| `bd_core` | 235 passed, 0 failed |
| `bd_app --tests` | 261 passed, 0 failed |
| `bd_console --lib` | 99 passed, 0 failed, but input/render evidence is unsound |
| `bd_tui --lib` | 111 passed, 2 failed |
| Formatting | Failed in 15 Rust files |
| Strict Clippy | Failed |
| Contract registry | Failed because authority paths are stale |
| Content validation | Passed |
| PTY console evidence | Not run / unverified |

The two independently reproduced UI failures are:

- `VISUAL-LAYOUT-001`: at 60x20, map area 198 is smaller than another
  interactive panel area 208;
- `VISUAL-CONTEXT-001`: at 80x24, a staffed-station Context composition clips
  the required `Assign Worker` action.

The console-specific false-green findings are:

- live input is implemented in `bd_tui`, while a second divergent input
  implementation remains unscheduled in `bd_console`;
- input tests manually reproduce state mutations instead of invoking the live
  production path;
- the original failing system-wiring tests were removed or replaced rather
  than satisfied;
- `ConsoleCommand` and `ConsoleState.pending` form competing submission paths;
- independent Bevy message readers allow a close key to be re-read as gameplay
  input after the console becomes closed;
- console overlay ordering is documented but not scheduled or composed;
- `GodMode` is advertised as invincibility but no damage resolver reads it;
- console blueprint spawning duplicates factory behavior and always installs
  colony-persistent scope;
- mutating commands directly edit gameplay state without the debug gate,
  intent/effect owner, and trace required by `Kernel.md`.

Historical labels such as `pre-existing` do not waive any item above.

## 4. Roles and precedence

### Reviewer/test author

The reviewer owns:

- S0 baseline recovery;
- product/architecture interpretation;
- contract IDs and scope classification;
- tests, fixtures, observers, plans, registry, evidence, and status;
- red validation and failure diagnostics;
- the sealed baseline, candidate manifest, active handoff body, and digest;
- post-candidate diff review, false-green review, PTY review, and promotion.

### Smaller implementation model

The implementation model owns only the production files listed by the current
signed handoff. It must not edit tests, plans, fixtures, the gate, registry,
evidence, manifest, or digest. It may report only `CandidateGreen` or
`NotComplete`.

### Priority order

Every handoff begins with this exact priority order:

1. protect authority, the sealed baseline, and user work;
2. obey stop conditions and the exact production write set;
3. satisfy the semantic contract through the named reusable production owner;
4. preserve neighboring behavior;
5. run every required focused, paired, neighboring, and candidate gate;
6. report measured evidence without promoting its meaning.

A lower-priority success never waives a higher-priority violation.

## 5. Locked outcomes

### S-01 — Trustworthy baseline

- The canonical argument-free gate exits zero before new console red tests are
  sealed.
- Formatting, strict Clippy, registry validation, test inventory, workspace
  tests, content validation, and whitespace all pass.
- Every authority link and candidate protected path points to its current
  repository location.
- Registry, requirement map, evidence, active-plan status, and current test
  results agree.
- Both existing UI failures are resolved through their existing semantic
  contracts, not hidden, deleted, ignored, or reclassified.

### S-02 — One console input owner

- One reusable reducer owns console key semantics.
- The TUI owns adaptation from terminal events and gameplay-input suppression;
  it does not independently reimplement console editing rules.
- Press is actionable; Repeat and Release are inert.
- Backtick, Escape, typing, Backspace, Enter, Tab, Up, and Down exercise the
  production reducer.
- One and only one submission path reaches command dispatch.

### S-03 — No input leakage

- A key captured by the console remains captured for the whole input batch,
  even when that key closes the console.
- Closing by Escape or backtick never begins a run, quits, moves, advances
  time, invokes a context action, or cancels another interaction.
- When the console is closed and no toggle occurred, ordinary gameplay input
  continues normally.
- Do not claim that a `MessageReader` drains an event globally; readers have
  independent cursors.

### S-04 — Final console composition

- The console overlay is composed after the normal screen through one final
  TUI-owned composition path.
- `bd_console` exposes reusable console rendering; it does not create a second
  screen-composition owner.
- The open overlay survives final buffers at 80x24 and 60x20 without corrupting
  required console content.
- Closing the console restores the same canvas and resolved styles as a clean
  normal render.
- Render errors remain observable.

### S-05 — Explicit debug mutation boundary

- Read-only commands may query state without mutation.
- Every mutating command emits an explicit typed debug intent/effect.
- One debug resolver owns mutation and runs in a named schedule stage.
- Mutation requires an explicit debug-mode resource/gate. The core default is
  disabled; deliberately installing `BdConsolePlugin` enables the gate for
  that app. Release packaging policy is outside this stabilization batch.
- Every accepted, rejected, or disabled mutation has a clear console result
  and trace entry.
- Console parsing and UI code never directly mutate gameplay components.

### S-06 — Honest GodMode semantics

While the debug gate is enabled and the player has `GodMode`:

- negative Health deltas targeting that player are rejected atomically;
- healing and zero deltas retain their normal semantics;
- AP, virtues, colony resources, and unrelated pools retain normal semantics;
- enemies, survivors, and players without `GodMode` retain normal damage;
- blocked lethal damage emits no defeat and no Game Over;
- removing `GodMode` restores normal damage.

GodMode must be implemented inside the canonical delta/resolution path, not as
a second combat or healing system.

### S-07 — Canonical blueprint spawning

- A successful console blueprint spawn uses `BlueprintCatalog` and
  `spawn_from_blueprint` rather than reproducing its component rules.
- Pools, statuses, markers, blocking state, name, and visual identity match a
  normal factory spawn.
- Tactical mode receives `EntityScope::Tactical` and no colony persistence.
- Outpost mode receives the established colony-persistent ownership.
- An unknown blueprint is atomic and returns a readable error.
- No content-ID-specific branch is introduced.

### S-08 — Stable command targeting

- Index-based commands use the same deterministic, player-visible ordering as
  the list they reference.
- Raw ECS iteration order and raw entity IDs never define command identity.
- Duplicate display names remain distinguishable through stable visible data
  or the command is rejected as ambiguous.

## 6. Explicit non-goals

- No new player-facing feature or Product P2 system.
- No new raid, event, enemy, station, survivor, or blueprint content.
- No command syntax expansion unless a locked contract requires stable
  disambiguation and the owner approves the syntax.
- No changes to combat balance, colony balance, AI, normal action costs,
  persistence semantics, or terminal profiles.
- No second input queue, renderer, factory, damage system, resource owner, or
  debug mutation owner.
- No public API created solely for tests.
- No source-text test that merely searches for function names as primary proof.
- No snapshots of the entire implementation as a substitute for semantic
  evidence.

## 7. Contract set to author and register

The reviewer may refine names before sealing, but each row owns one outcome and
must have exactly one primary test.

| Contract | Primary outcome | Required evidence |
|---|---|---|
| `CONSOLE-INPUT-001` | Physical console editing keys reach one production reducer | InputStateMachine, StateDiff |
| `CONSOLE-INPUT-002` | Console-owned close/toggle keys never leak into gameplay | InputStateMachine, StateDiff, Workflow |
| `CONSOLE-COMMAND-001` | One physically submitted line reaches dispatch exactly once through one submission path | InputStateMachine, Schedule, StateDiff, Workflow |
| `CONSOLE-RENDER-001` | Open console survives final composition at both profiles | Projection, BufferLayout, PTY |
| `CONSOLE-RENDER-002` | Closing restores the unchanged underlying final canvas | BufferLayout, StateDiff, PTY |
| `DEBUG-GATE-001` | Mutating commands require the explicit debug gate and produce traceable denial/acceptance | Domain, Schedule, StateDiff, Workflow |
| `DEBUG-INTENT-001` | Every mutating console command reaches one typed resolver rather than direct UI/dispatch mutation | Schedule, StateDiff, Workflow |
| `DEBUG-GOD-001` | GodMode blocks only negative player Health deltas and prevents defeat | Domain, StateDiff, Workflow |
| `DEBUG-SPAWN-001` | Console spawning preserves the canonical factory component fingerprint | Domain, StateDiff, Workflow |
| `DEBUG-SPAWN-002` | Console spawning assigns scope from the current game mode | Domain, StateDiff, Workflow |
| `DEBUG-SPAWN-003` | An unknown console blueprint request is atomic and reports a readable error | Domain, StateDiff, Workflow |
| `DEBUG-TARGET-001` | Command selection is deterministic and not ECS-query-order based | Domain, StateDiff, Workflow |

The reviewer decides whether each console/debug row is `Regression`,
`FoundationSupport`, or `DeferredInfrastructure` according to the testing
standard. A row must not be labeled `FoundationRequired` merely because it
runs in the workspace gate.

## 8. Phase S0A — Reviewer-only canonical recovery

The smaller model must not execute this phase.

### Tasks

1. Repair current authority references in `testing/foundation-contracts.ron`.
2. Repair candidate protected paths in `scripts/test-gate.sh`.
3. Add the missing active-plan navigation and reconcile active-plan status.
4. Run every current registry/governance test independently.
5. Reconcile `VISUAL-LAYOUT-001` and `VISUAL-CONTEXT-001` to accurate Red
   evidence, preserving their current failures.
6. Resolve those two existing UI failures without weakening their observers.
7. Apply formatting and resolve every strict-Clippy finding; do not use broad
   warning suppressions to hide defects.
8. Register and reconcile the already-implemented factory/action/event/raid
   contracts with accurate non-Foundation classification where required.
9. Run the argument-free canonical gate until it exits zero with consistent
   listed/observed totals.
10. Record the new clean commit and exact measured baseline.

### Exit gate

- clean worktree or an explicitly sealed pre-existing delta;
- zero current UI failures;
- zero formatting or strict-Clippy failures;
- registry and contract metrics pass;
- `bash scripts/test-gate.sh` reports `STATUS=VerifiedGreen`;
- required existing 80x24 and 60x20 UI evidence is current.

If S0A is not complete, stop. Do not author a console candidate handoff on top
of an untrusted baseline.

## 9. Phase S0B — Reviewer-only red authoring and sealing

The smaller model must not execute this phase.

### Test-authoring requirements

For each batch below, the reviewer:

1. writes the smallest primary/supporting tests;
2. includes Given/When/Then/Must-not-change/Evidence-layers comments;
3. includes implementation guidance naming reusable owner, integration seam,
   preserved behavior, invalid shortcuts, and closing evidence;
4. runs every primary and completion-critical support independently;
5. proves each failure is caused by missing production behavior;
6. confirms no later case is hidden behind an earlier panic;
7. records the red diagnostics;
8. seals a clean baseline, active handoff body, exact production write set,
   protected paths, manifest, and separate digest.

The manifest must protect this plan, all affected tests and observers, current
authority, the gate, registry, requirement map, evidence, and status files.

## 10. Candidate Batch C1 — Input reducer and whole-batch capture

### Owned contracts

- `CONSOLE-INPUT-001`
- `CONSOLE-INPUT-002`
- `CONSOLE-COMMAND-001`

### Required behavior

Implement one reusable console key reducer and one submission path. The TUI
adapter must record that the console owned the batch before it mutates open
state, so closing keys cannot fall through to gameplay.

Required production-path cases:

- open, type, edit, submit, and dispatch a non-mutating command;
- Repeat and Release do nothing;
- Escape close at Title does not quit or begin;
- backtick close at Title does not begin;
- Escape and backtick close in Outpost do not quit, move, advance time, or
  mutate an active interaction;
- a normal key with the console closed still reaches gameplay;
- multiple keys in one batch remain ordered and captured.

### Proposed exact production write set

The reviewer must finalize this set in the signed manifest:

```text
crates/bd_console/src/input.rs
crates/bd_console/src/lib.rs
crates/bd_console/src/state.rs
crates/bd_tui/src/lib.rs
```

If the implementation needs any other file, report `STATUS=NotComplete` and
name the path and reason. Do not edit tests or add a handoff file.

### Neighbor gates

```text
cargo test -p bd_console --lib
cargo test -p bd_app --test press_repeat_release_policy
cargo test -p bd_app --test phase6_input
cargo test -p bd_tui --lib
signed candidate gate
```

## 11. Candidate Batch C2 — Final overlay composition

### Owned contracts

- `CONSOLE-RENDER-001`
- `CONSOLE-RENDER-002`

### Required behavior

Expose reusable console overlay composition from `bd_console` and invoke it
from the TUI's authoritative final draw path after the normal screen. Avoid a
cross-crate dependency cycle and do not rely on plugin registration order as
render order.

Required cases:

- final open overlay at 80x24 and 60x20;
- command output, prompt, typed buffer, border, and available region remain
  inside the terminal;
- underlying content cannot overwrite the overlay;
- closed console produces the identical clean canvas and styles;
- open -> resize -> close returns to clean authoritative output;
- a render failure remains visible to the existing shutdown/error boundary.

### Proposed exact production write set

```text
crates/bd_console/src/lib.rs
crates/bd_console/src/render.rs
crates/bd_tui/src/lib.rs
```

### Neighbor gates

```text
cargo test -p bd_console --lib
cargo test -p bd_tui --lib
cargo test -p bd_app --test phase6_input
signed candidate gate
```

Candidate evidence is incomplete until the reviewer performs real 80x24 and
60x20 PTY inspection.

## 12. Candidate Batch C3 — Debug gate, intent owner, and ordinary mutations

### Owned contracts

- `DEBUG-GATE-001`
- `DEBUG-INTENT-001`
- `DEBUG-TARGET-001`

### Required behavior

Create one explicit debug mutation boundary owned by the core. Migrate the
existing resource, time, event, transition, teleport, survivor-task, and other
ordinary mutating commands to typed requests resolved only behind the debug
gate. Read-only help/list/stats/clear behavior remains read-only.

The debug resolver must have a named schedule stage, deterministic ordering,
clear denial behavior, and trace entries. Dispatch may parse and report; it may
not directly edit gameplay components.

Table-driven evidence must cover every mutating command in this batch and
assert both its authorized delta and forbidden neighboring mutations. Stable
target selection must not depend on ECS query order.

### Proposed exact production write set

```text
crates/bd_console/src/dispatch.rs
crates/bd_console/src/lib.rs
crates/bd_core/src/debug.rs
crates/bd_core/src/lib.rs
```

The reviewer may replace this proposed set only before sealing. The
implementation model may not expand it.

### Neighbor gates

```text
cargo test -p bd_core
cargo test -p bd_console --lib
cargo test -p bd_app --tests
signed candidate gate
```

## 13. Candidate Batch C4 — GodMode and canonical factory spawn

### Owned contracts

- `DEBUG-GOD-001`
- `DEBUG-SPAWN-001`
- `DEBUG-SPAWN-002`
- `DEBUG-SPAWN-003`

### Required behavior

Implement GodMode as a rule inside the canonical signed-delta resolution path.
Migrate console blueprint spawning to the canonical blueprint factory and
derive scope from the current game mode. Migrate any remaining combat/spawn
mutation commands through the debug intent owner so no direct mutation remains
in console dispatch.

Factory-parity evidence must compare structured components for at least two
blueprints with different markers/statuses and both Tactical and Outpost mode.
It must fail if console dispatch copies only the named rat fixture.

### Proposed exact production write set

```text
crates/bd_console/src/dispatch.rs
crates/bd_core/src/debug.rs
crates/bd_core/src/pools.rs
```

If canonical factory reuse requires a production change outside this set, stop
and request a new sealed handoff rather than copying the factory.

### Neighbor gates

```text
cargo test -p bd_core
cargo test -p bd_console --lib
cargo test -p bd_app --tests
signed candidate gate
```

## 14. Required implementation loop for every candidate batch

The smaller model repeats this loop until `CandidateGreen` or a stop condition:

```text
READ
  Re-read this batch, the protected tests, manifest, digest, and write set.
BASELINE
  Run each named red primary and paired support independently.
DECISION RECORD
  State failing case, missing responsibility, reusable owner, authorized target
  file, and the exact paired validation to run next. If any is unknown, stop.
IMPLEMENT
  Confirm the file is in the write set. Change one production responsibility.
FOCUSED VALIDATION
  Run the exact failing case and every completion-critical pair.
CLASSIFY
  If still red, diagnose and repeat. If green for the wrong reason, stop and
  report the observer defect; do not edit protected tests.
NEIGHBOR VALIDATION
  Run every named affected crate/workflow gate. Repair only inside the write
  set; otherwise stop.
CANDIDATE GATE
  Run the exact signed candidate command with the author-supplied digest.
SELF-AUDIT
  Audit every changed/untracked path, production diff, shortcuts, preserved
  behavior, and manifest integrity.
HANDOFF
  Paste the implementation report in chat. Never create a report file unless
  its exact path was authorized.
```

## 15. Mandatory stop conditions

Report `STATUS=NotComplete` immediately when:

- a required edit falls outside the exact write set;
- a protected file, test, fixture, plan, registry, evidence file, gate,
  manifest, or digest would need to change;
- a named red test passes before implementation for an unexplained reason;
- a test does not invoke the production input, resolver, or final composition
  path it claims;
- satisfying a test would require duplicating input rules, rendering, factory
  logic, damage logic, target ordering, or mutation ownership;
- a Bevy reader is assumed to consume an event for other readers;
- plugin insertion order is assumed to prove render order;
- GodMode semantics are ambiguous beyond S-06;
- a command requires a new syntax or product decision;
- a regression appears outside the authorized batch;
- the signed candidate gate or manifest integrity check fails;
- any shortcut answer below is `Yes`, `Unknown`, missing, or unsupported.

Do not label a stop condition `pre-existing` and continue. Record the first
failure and return control to the reviewer.

## 16. Binary shortcut checklist

Every implementation handoff answers each with evidence-backed `No`:

1. Did I edit or replace a protected test, fixture, observer, plan, registry,
   evidence file, gate, manifest, or digest?
2. Did I delete a failing test or substitute a direct-state test for a
   production-path test?
3. Did I manually reproduce console key transitions in a consumer instead of
   using the shared reducer?
4. Did I keep two command submission paths?
5. Did I assume reading a Bevy message prevents another reader from seeing it?
6. Did I rely on plugin registration order rather than final composition
   evidence?
7. Did I mutate gameplay state directly from `bd_console` parsing/UI code?
8. Did I bypass the debug gate or omit a trace entry?
9. Did I create separate damage/healing behavior outside the signed-delta
   resolver?
10. Did I copy blueprint component rules instead of using the factory?
11. Did I hardcode a fixture, content ID, terminal profile, coordinate, or
    display name?
12. Did I use raw ECS iteration order or raw entity IDs for command targeting?
13. Did I skip an exact primary, paired test, neighbor gate, or signed candidate
    gate?
14. Did I claim unrelated/pre-existing work or use a green command to waive a
    scope violation?

## 17. Candidate in-chat handoff template

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

Missing evidence is written as `Not run` or `Unverified`. A zero-test
invocation is not a pass.

## 18. Independent review and final reconciliation

After each candidate handoff, the reviewer:

1. verifies the baseline, manifest, digest, protected hashes, and complete
   changed/untracked path list;
2. reproduces every primary, paired, neighboring, and candidate gate;
3. audits the diff for copied rules, fixture/profile special cases, direct
   mutation, hidden behavior, weakened tests, and isolated-layer greens;
4. runs the relevant production workflow and final composed buffer;
5. performs 80x24 and 60x20 PTY checks for input and rendering batches;
6. accepts or rejects the candidate without asking it to self-promote;
7. creates a new baseline/body/manifest/digest for the next batch.

After C1–C4 are independently accepted, the reviewer atomically updates the
registry, requirement map, evidence ledger, plan status, documentation, and
changelog, then runs the argument-free canonical gate.

## 19. Final completion gate

This stabilization is `ReviewedGreen` only when:

- every contract in Section 7 has accurate registered ownership and status;
- all current Foundation contracts and workflows pass;
- no direct console gameplay mutation remains outside the explicit debug
  resolver;
- one input reducer, one submission path, one debug mutation owner, one
  factory, one signed-delta resolver, and one final UI composition owner remain;
- formatting, compilation, strict Clippy, content validation, whitespace,
  registry validation, inventory, and workspace tests pass;
- listed, passed, failed, and ignored totals reconcile;
- 80x24 and 60x20 PTY evidence agrees with automated input/render evidence;
- the production diff and GDD/Kernel drift review find no unauthorized feature,
  deferred-system promotion, or duplicated responsibility;
- `bash scripts/test-gate.sh` reports `STATUS=VerifiedGreen` on the reviewed
  worktree;
- the reviewer records `ReviewedGreen` only after the remaining manual and
  evidence checks pass.

Only then is the repository ready to resume further feature development.
