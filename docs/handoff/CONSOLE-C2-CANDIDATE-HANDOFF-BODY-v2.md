# Console C2 Closed-State Invalidation Candidate Handoff — v2

Use `$authoritative-test-pipeline`. Execute this entire bounded loop until the
signed candidate gate reports `STATUS=CandidateGreen`, or stop immediately
with `STATUS=NotComplete` when any stop condition occurs. Do not merely inspect
or recommend a change.

This v2 body supersedes C2 v1. V1 produced useful composition work, but its
resize observer was repaired and its protected seal is withdrawn. Do not use,
edit, regenerate, restore from, or derive authority from the v1 baseline,
manifest, or digest.

## Priority

1. Preserve protected authority, the v2 baseline, and current useful work.
2. Obey all stop conditions and the exact one-file production write set.
3. Make absent and closed console states share the ordinary frame fingerprint.
4. Preserve open/buffer/output invalidation and all composition/lifecycle work.
5. Run every exact, neighbor, inventory, and signed command.
6. Report measured candidate evidence without changing contract status.

A later success never waives an earlier rule.

## Read completely before editing

```text
AGENTS.md
GDD.md
Kernel.md
docs/authority/DECISIONS-TO-LOCK.md
docs/authority/AUTHORITATIVE-TESTING-STANDARD-AND-MIGRATION-PLAN.md
docs/active/FOUNDATION-STABILIZATION-AND-CONSOLE-HARDENING-PLAN.md
docs/handoff/CONSOLE-C2-CANDIDATE-HANDOFF-BODY-v2.md
crates/bd_console/src/render.rs
crates/bd_console/src/state.rs
crates/bd_tui/src/lib.rs
crates/bd_tui/src/console_render_contract_tests.rs
testing/foundation-contracts.ron
testing/FOUNDATION-REQUIREMENT-MAP.md
testing/FOUNDATION-TEST-EVIDENCE.md
testing/CONSOLE-C2-BASELINE-v2.ron
testing/CONSOLE-C2-CANDIDATE-HANDOFF-v2.ron
```

Read the complete authoritative-test skill. Tests, reusable renderer, plan,
registry, evidence, requirement map, baseline, manifest, body, and gate are
reviewer-owned. Do not edit them.

## Exact production write set

You may modify only:

```text
crates/bd_tui/src/lib.rs
```

Formatting may change only that file. Do not use `git stash`, `git checkout`,
`git restore`, `git reset`, or any command that replaces current content from
`HEAD`, the index, a stash, or another revision. `HEAD` predates reviewer
preparation and is not the sealed baseline. Do not reconstruct protected or
baseline content manually. If current state appears damaged, stop.

## Locked outcome

The existing `visible_console_fingerprint` extends the ordinary UI fingerprint.
Change only its remaining semantic defect:

- `None` and `Some(closed ConsoleState)` return the same fingerprint because
  both produce the identical clean final frame;
- hidden changes to buffer, output, history, pending dispatch, and capture
  bookkeeping while closed do not change that fingerprint;
- an open transition changes the fingerprint;
- open buffer and output changes each change it;
- open history, pending dispatch, and capture bookkeeping do not change it;
- the exact hash number and internal hashing algorithm are not locked.

Preserve without rewriting:

- `draw_ui` as the one authoritative terminal draw;
- normal UI first, reusable console overlay second;
- no standalone console draw registration;
- open overlay content/border at 80x24 and 60x20;
- closed clean-canvas and style equality;
- open 80x24 → resize backend and terminal to 60x20 → close cleanup;
- the existing render-failure shutdown boundary;
- all C1 input and typed-dispatch behavior.

Do not hash the whole `ConsoleState`, redraw unconditionally, special-case a
fixture/profile/test/hash value, copy renderer behavior, add another draw, or
edit a protected observer.

## Authenticate and reproduce

Authenticate the v2 manifest with the separately supplied digest before any
edit. Then run each command independently.

Intentional Red:

```text
cargo test --locked -p bd_tui --lib console_render_contract_tests::visible_console_state_invalidates_the_authoritative_frame_once -- --exact --nocapture
```

Expected: exactly one test executes and fails only with
`case=no-console-vs-closed expected_equal=true`; open-transition, open-buffer,
open-output, closed-hidden-state, and open-hidden-state are absent from the
violation list. A pass, zero-test run, compile error, or different violation
tuple is a stop condition.

Preservation Green — run independently:

```text
cargo test --locked -p bd_tui --lib console_render_contract_tests::open_console_survives_authoritative_final_composition_at_supported_profiles -- --exact --nocapture
cargo test --locked -p bd_tui --lib console_render_contract_tests::closed_console_matches_clean_canvas_and_styles_at_supported_profiles -- --exact --nocapture
cargo test --locked -p bd_tui --lib console_render_contract_tests::open_resize_close_returns_to_clean_authoritative_output -- --exact --nocapture
cargo test --locked -p bd_tui --lib tests::draw_failure_requests_clean_application_shutdown -- --exact --nocapture
```

Each must execute one test and pass before and after the edit.

## Mandatory bounded loop

```text
READ
  Re-read this body, the complete fingerprint test, current fingerprint helper,
  and the v2 baseline delta.
REPRODUCE
  Run the exact Red and preserve its one-row violation tuple.
DECIDE
  Fill the decision record. Unknown means stop.
IMPLEMENT
  Confirm the target is crates/bd_tui/src/lib.rs. Change only the closed/absent
  visible-fingerprint responsibility.
FOCUSED VALIDATION
  Run the Red and all four preservation cases independently.
CLASSIFY
  If Red, name the remaining semantic row and repeat. Unexpected behavior or a
  requested test change means stop.
NEIGHBOR VALIDATION
  Run every required neighbor below.
CANDIDATE GATE
  Run the signed v2 candidate gate.
SELF-AUDIT
  Audit the complete v2 baseline delta, production diff, test inventory, and
  shortcut checklist.
HANDOFF
  Paste the required in-chat report. Do not create a repository report.
```

Decision record:

```text
Failing contract and exact case:
Observed violation tuple:
Missing semantic responsibility:
Reusable owner:
Authorized target file:
Paired cases to rerun:
Behavior that must remain unchanged:
Scope/stop-condition check:
```

## Required closing commands

Run all five focused cases independently, then:

```text
cargo fmt --all -- --check
cargo test --locked -p bd_console --lib
cargo test --locked -p bd_tui --lib
cargo test --locked -p bd_app --test console_input_contract
cargo test --locked -p bd_app --test phase6_input
cargo test --locked -p bd_test_support --bin handoff_guard
cargo test --locked -p bd_test_support --test candidate_handoff
cargo test --locked -p bd_test_support --test contract_registry
cargo test --locked -p bd_test_support --test repository_governance
cargo clippy --workspace --all-targets --locked -- -D warnings
git diff --check
```

Finally run:

```text
bash scripts/test-gate.sh \
  --candidate-manifest testing/CONSOLE-C2-CANDIDATE-HANDOFF-v2.ron \
  --manifest-sha256 <REVIEWER-SUPPLIED-CONSOLE-C2-V2-DIGEST>
```

`CONSOLE-RENDER-001` remains `Red`; only independent review may reconcile it.

## Stop conditions

Stop with `STATUS=NotComplete` if:

- authentication, protected hashes, baseline scope, or the 822-name inventory
  fails;
- the starting Red differs from the one expected violation;
- any preservation case is Red;
- any path outside `crates/bd_tui/src/lib.rs` would need change;
- any protected file, test, observer, renderer, authority, status, evidence,
  body, baseline, manifest, gate, skill, or policy would need change;
- `git stash`, checkout, restore, reset, or revision-based replacement would be
  used;
- test-module conditional compilation is removed or test code enters the
  production build;
- hidden state is hashed, redraw is unconditional, renderer logic is copied,
  or another terminal draw is introduced;
- a fixture-, test-, profile-, hash-, or command-specific branch appears;
- a test is deleted, renamed, ignored, filtered, or removed from inventory;
- a required command is skipped, substituted, zero-test, or unclassified;
- an unauthorized report/log/file appears;
- contract/evidence status is self-promoted.

## Binary shortcut checklist

Every answer must be evidence-backed `No`; otherwise report NotComplete.

1. Protected or reviewer-owned file changed?
2. Path outside the one-file write set changed after baseline?
3. Stash/checkout/restore/reset or revision replacement used?
4. Test deleted, renamed, ignored, filtered, or compiled into production?
5. Whole ConsoleState or hidden state hashed?
6. Unconditional redraw introduced?
7. Renderer/draw ownership duplicated or reordered?
8. Fixture/test/profile/hash/command special case introduced?
9. C1 input/dispatch, gameplay, or unrelated UI behavior changed?
10. Required command skipped, zero-test, or unclassified?
11. Status or evidence self-promoted?

## Required in-chat report

```text
# Console C2 v2 Candidate Handoff Report

STATUS=CandidateGreen | STATUS=NotComplete

## Scope and integrity
- Manifest/digest used:
- Guard result:
- Authorized file changed:
- Unexpected paths:
- Test inventory result:
- Revision-replacement commands used:

## Fingerprint ownership
- Closed/absent equality implementation:
- Open visible fields included:
- Hidden fields excluded:
- Exact focused results:

## Preservation and neighbors
- Open final composition:
- Closed clean canvas/styles:
- Open-resize-close lifecycle:
- Draw-failure boundary:
- bd_console / bd_tui / console_input / phase6:
- governance / clippy / formatting / diff check:

## Signed candidate gate
- Steps:
- Tests listed/passed/failed/ignored:
- Contract metrics:
- Final status line:

## Complete v2 baseline delta
- Production diff summary:
- `git status --short`:
- Protected and inventory verification:

## Shortcut checklist
1. Protected change? No — evidence:
2. Outside-scope delta? No — evidence:
3. Revision replacement? No — evidence:
4. Test removed/renamed/ignored/production-compiled? No — evidence:
5. Whole/hidden state hashed? No — evidence:
6. Unconditional redraw? No — evidence:
7. Draw/renderer duplicated or reordered? No — evidence:
8. Special case? No — evidence:
9. C1/gameplay/unrelated UI changed? No — evidence:
10. Required command skipped? No — evidence:
11. Status self-promoted? No — evidence:

## Remaining independent review
- Production diff risk:
- PTY still pending:
- Contract status remains:
```
