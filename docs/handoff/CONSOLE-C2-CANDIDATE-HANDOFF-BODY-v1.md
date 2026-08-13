# Console C2 Final Composition Candidate Handoff — v1

Use `$authoritative-test-pipeline`. Execute this entire bounded loop until the
signed candidate gate reports `STATUS=CandidateGreen`, or stop immediately
with `STATUS=NotComplete` when any stop condition occurs. Do not stop after
inspection, a focused green, or a plausible code change.

The separately supplied manifest digest authenticates this instruction body,
the exact starting worktree, the complete test-name inventory, and all
reviewer-owned observers. Never edit or regenerate the body, baseline,
manifest, digest, tests, plan, registry, evidence, or requirement map.

## Priority — later success never waives an earlier rule

1. Preserve protected authority, reviewer preparation, and pre-existing work.
2. Obey every stop condition and the exact two-file production write set.
3. Keep one authoritative terminal draw; compose the reusable console last.
4. Make visible console state participate in render invalidation exactly once.
5. Preserve clean closed output, resize cleanup, and draw-failure visibility.
6. Run every exact, neighbor, inventory, and signed command as written.
7. Report measured candidate evidence without promoting contract status.

## Read completely before editing

```text
AGENTS.md
GDD.md
Kernel.md
docs/authority/DECISIONS-TO-LOCK.md
docs/authority/AUTHORITATIVE-TESTING-STANDARD-AND-MIGRATION-PLAN.md
docs/active/FOUNDATION-STABILIZATION-AND-CONSOLE-HARDENING-PLAN.md
docs/handoff/CONSOLE-C2-CANDIDATE-HANDOFF-BODY-v1.md
crates/bd_console/src/lib.rs
crates/bd_console/src/render.rs
crates/bd_console/src/state.rs
crates/bd_tui/src/lib.rs
crates/bd_tui/src/console_render_contract_tests.rs
crates/bd_tui/src/ui_development_contract_tests.rs
crates/bd_tui/src/runtime_control.rs
testing/foundation-contracts.ron
testing/FOUNDATION-REQUIREMENT-MAP.md
testing/FOUNDATION-TEST-EVIDENCE.md
testing/CONSOLE-C2-BASELINE-v1.ron
testing/CONSOLE-C2-CANDIDATE-HANDOFF-v1.ron
```

Read the complete authoritative-test skill before implementation. The
reviewer has already extracted `bd_console::render::render_console_overlay`
and `console_overlay_area`, and has installed the TUI's explicit
`render_final_frame` and `visible_console_fingerprint` seams. Those green
preparatory changes are sealed. Do not replace, copy, or move the reusable
overlay widgets.

## Exact production write set

You may modify only:

```text
crates/bd_console/src/lib.rs
crates/bd_tui/src/lib.rs
```

No other production, test, documentation, manifest, evidence, report, or log
path is authorized. Before each edit, name the target and confirm it appears
above. Formatting may change only these two paths. If another path appears
necessary, stop and report it; do not expand scope yourself.

## Locked outcome

### One authoritative final composition

- `draw_ui` remains the sole production owner of `RatatuiContext::draw` for
  the ordinary screen plus console overlay.
- `render_final_frame` draws the normal UI first, then calls the reusable
  `bd_console::render::render_console_overlay` with the same frame when a
  console resource exists.
- Remove the independently registered `render_console` system from
  `BdConsolePlugin`. Do not add an ordering edge between two terminal draws.
- Do not copy console layout, colors, border, output, or prompt rendering into
  `bd_tui`; the protected reusable composer is their single owner.
- An absent or closed console leaves the normal canvas and resolved styles
  byte-for-byte/cell-for-cell unchanged.

### Visible invalidation only

- Extend the existing base fingerprint through
  `visible_console_fingerprint`; do not replace or duplicate the base hash.
- Closed console state hashes exactly like no visible console. Hidden edits
  while closed must not request a draw.
- For an open console, `open`, `buffer`, and `output` affect the fingerprint.
- History, pending dispatch, and batch-capture bookkeeping do not affect the
  fingerprint because the protected renderer does not display them.
- Do not use a constant, test name, fixture string, terminal-size special
  case, or unconditional redraw to satisfy invalidation.

### Preserve the lifecycle and error boundary

- Open overlays fit at both 80x24 and 60x20 with output, prompt, typed buffer,
  title, and complete border inside the terminal.
- The console clear/border/output/prompt remain after normal underlay
  composition; the underlay cannot overwrite them.
- Open at 80x24, resize to 60x20, then close on the same terminal returns the
  exact clean 60x20 canvas and resolved styles.
- The existing terminal-draw error path still records the failure and requests
  clean application shutdown.
- This batch changes no input, dispatch, debug mutation, factory, simulation,
  gameplay, UI9 panel, or product behavior.

## Authenticate and reproduce the sealed start

First run the handoff guard with the separately supplied digest. A hash,
inventory, worktree, compile, or manifest mismatch is a stop condition.

Intentional Red — run independently:

```text
cargo test --locked -p bd_tui --lib console_render_contract_tests::open_console_survives_authoritative_final_composition_at_supported_profiles -- --exact --nocapture
cargo test --locked -p bd_tui --lib console_render_contract_tests::visible_console_state_invalidates_the_authoritative_frame_once -- --exact --nocapture
cargo test --locked -p bd_tui --lib console_render_contract_tests::open_resize_close_returns_to_clean_authoritative_output -- --exact --nocapture
```

Expected checkpoints:

- final composition: `CONSOLE`, `OK: C2-FINAL-OUTPUT`, the typed prompt, and
  the console border are absent at 80x24 before the observer can continue;
- invalidation: `open-transition`, `open-buffer`, and `open-output` all retain
  the same base hash;
- lifecycle: the first 80x24 open checkpoint reports no visible overlay.

Each invocation must compile, execute exactly one test, and fail at its named
checkpoint. A pass, zero-test run, compile error, or different first failure is
`STATUS=NotComplete`.

Preservation Green — run independently:

```text
cargo test --locked -p bd_tui --lib console_render_contract_tests::closed_console_matches_clean_canvas_and_styles_at_supported_profiles -- --exact --nocapture
cargo test --locked -p bd_tui --lib tests::draw_failure_requests_clean_application_shutdown -- --exact --nocapture
```

Each must execute exactly one test and pass before and after implementation.

## Mandatory bounded loop

Repeat until candidate green or a stop condition:

```text
READ
  Re-read this body, all four C2 test comments, both production seams, the
  reusable protected renderer, and the complete baseline delta.
REPRODUCE
  Run the first remaining exact Red and preserve its diagnostic tuple.
DECIDE
  Fill every decision-record field below. Unknown means stop.
IMPLEMENT
  Confirm the target is one of two authorized paths. Change one responsibility:
  final composition/registration OR visible invalidation.
FOCUSED VALIDATION
  Run all three Reds and both preservation Greens independently.
CLASSIFY
  If Red, name the remaining production responsibility and repeat. If a case
  passes for a shortcut or unrelated reason, stop and report observer defect.
NEIGHBOR VALIDATION
  Run every exact neighbor command below; zero-test commands are failures.
CANDIDATE GATE
  Run the signed C2 candidate gate with the reviewer-supplied digest.
SELF-AUDIT
  Inspect the complete baseline delta, production diff, one-draw ownership,
  inventory fingerprint, and every shortcut question.
HANDOFF
  Paste the required report in chat. Do not create a repository report.
```

Before each edit, record:

```text
Failing contract and exact case:
Observed expected/actual tuple:
Missing production responsibility:
Reusable owner:
Integration seam:
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
git diff --check
```

Finally run exactly:

```text
bash scripts/test-gate.sh \
  --candidate-manifest testing/CONSOLE-C2-CANDIDATE-HANDOFF-v1.ron \
  --manifest-sha256 <REVIEWER-SUPPLIED-CONSOLE-C2-V1-DIGEST>
```

The signed inventory must remain the same exact sorted test-name multiset.
Both C2 records remain `Red`; only the independent reviewer may reconcile
them after candidate inspection. Candidate mode may report
`STATUS=CandidateGreen`, never ReviewedGreen.

## Stop conditions — stop outranks every green

Stop immediately with `STATUS=NotComplete` if:

- manifest authentication, a protected hash, test count, or inventory digest
  fails;
- a starting Red passes, does not compile, executes zero tests, or fails at a
  different checkpoint;
- either preservation Green starts or becomes Red;
- any edit is needed outside the exact two-file production write set;
- a protected test, renderer, observer, authority, plan, registry, evidence,
  baseline, manifest, body, skill, policy, gate, or digest would need change;
- a second terminal draw remains registered for the console;
- render order relies on plugin insertion, tuple order, set membership,
  resource conflict, ambiguity suppression, or `.after(draw_ui)` across
  crates;
- console widgets or layout are copied into `bd_tui`;
- the final compositor renders console before the ordinary screen;
- invalidation is unconditional or includes hidden history/pending/capture
  state merely to hash the whole resource;
- a fixture-, test-, command-, profile-, or string-specific branch appears;
- a test is deleted, renamed, ignored, filtered away, or otherwise removed
  from the signed inventory;
- an unauthorized or untracked report/log/cleanup file appears;
- any required command is skipped, substituted, or left unclassified;
- the signed gate conflicts with a higher-priority rule.

## Binary shortcut checklist

Every answer must be evidence-backed `No`. Any `Yes`, `Unknown`, missing, or
unsupported answer means `STATUS=NotComplete`.

1. Any protected or reviewer-owned file changed?
2. Any path outside the exact two-file write set changed after baseline?
3. Any test deleted, renamed, ignored, filtered, or removed from inventory?
4. More than one production terminal draw owns the normal UI plus console?
5. Any render ordering based on plugin/tuple/set/conflict behavior?
6. Any console widget/layout/color/prompt logic copied into `bd_tui`?
7. Any console drawn before the normal UI or outside `render_final_frame`?
8. Any unconditional redraw used instead of visible-state invalidation?
9. Any hidden history/pending/capture state hashed as if player-visible?
10. Any fixture-, test-, string-, command-, or profile-specific shortcut?
11. Any C1 input/dispatch, gameplay, simulation, or UI9 behavior changed?
12. Any required command skipped, substituted, zero-test, or unclassified?
13. Any contract status, evidence, or acceptance claim self-promoted?

## Required in-chat handoff report

Paste this template exactly, filling every field with measured evidence. Do
not create a repository file.

```text
# Console C2 Candidate Handoff Report

STATUS=CandidateGreen | STATUS=NotComplete

## Scope
- Manifest path:
- Reviewer-supplied digest used:
- Guard authentication result:
- Authorized files changed:
- Unexpected Git-visible paths:
- Final test inventory count/digest result:

## Implementation ownership
- Sole terminal draw owner:
- Normal-underlay then overlay call path:
- Standalone console registration removed (evidence):
- Reusable overlay owner used:
- Visible console fields hashed:
- Hidden console fields excluded:

## Exact focused cases
- open final composition 80x24 + 60x20:
- visible-state invalidation matrix:
- closed clean-canvas/style equality:
- open -> resize -> close lifecycle:
- render-failure shutdown boundary:

## Neighbor results
- bd_console --lib:
- bd_tui --lib:
- console_input_contract:
- phase6_input:
- handoff_guard unit target:
- candidate_handoff:
- contract_registry:
- repository_governance:
- cargo fmt --check:
- git diff --check:

## Signed candidate gate
- Gate steps:
- Tests listed/passed/failed/ignored:
- Contract metrics:
- Final status line:

## Complete baseline delta
- Per-file production summary:
- Protected-file verification:
- Inventory verification:
- `git status --short`:

## Shortcut checklist
1. Protected/reviewer file changed? No — evidence:
2. Outside-write-set delta? No — evidence:
3. Test removed/renamed/ignored/filtered? No — evidence:
4. Multiple terminal draw owners? No — evidence:
5. Implicit render ordering? No — evidence:
6. Copied console renderer? No — evidence:
7. Wrong final composition order/path? No — evidence:
8. Unconditional redraw? No — evidence:
9. Hidden console state hashed? No — evidence:
10. Fixture/test/string/profile shortcut? No — evidence:
11. C1/gameplay/UI9 behavior changed? No — evidence:
12. Required command skipped/zero/unclassified? No — evidence:
13. Status self-promoted? No — evidence:

## Remaining independent review
- Production diff risks:
- Real 80x24 PTY pending:
- Real 60x20 PTY pending:
- Contract status remains:
```
