# Broken Divinity Alpha Readiness Plan (2026-05-26)

## Mission
Ship a solid Alpha candidate that feels trustworthy in the first 10 minutes, supports the full MVP loop end-to-end, and remains gate-clean under repeat regression runs.

This is a new plan document and does not reopen the closed UX recovery plan in plan.md.

## Why This Plan Exists
Recent UX validation confirms major trust fixes landed, but remaining friction blocks Alpha confidence:
- First-minute instruction hierarchy still competes (objective vs controls vs panel noise)
- Esc behavior semantics are correct but not always self-evident to first-time players
- Primary action emphasis is too weak in high-traffic states
- Some helper copy remains too long under pressure
- Launch/test diagnostics are noisy for QA workflows

Additionally, Alpha requires hard proof of MVP-loop reliability, not just local UX fixes.

## Alpha Definition of Done
Alpha is considered ready only when all of the following are true:
1. New player can complete Menu -> Colony -> Overworld -> Dungeon -> Return loop without external documentation.
2. First-session UX scorecard reaches minimum bar:
   - Onboarding clarity >= 8.0
   - Control discoverability >= 8.0
   - Navigation predictability >= 8.0
   - UI hierarchy/readability >= 7.8
   - Feedback quality >= 8.0
   - Error/edge-case trust >= 8.0
   - Goal clarity/progression cues >= 8.2
   - Overall first-session confidence >= 8.0
3. Save/load continuity works across all gameplay states exercised in Alpha runbook.
4. Full quality gate passes on every ticket completion:
   - debug build
   - tests
   - clippy -D warnings
   - release build
5. No P0 or P1 regressions open in the Alpha backlog.

## Scope Lock
In scope:
- Remaining first-time UX trust issues
- MVP loop validation and guardrails
- Save/load continuity and run-state recap quality
- Deterministic playtest protocol and scorecard update workflow
- Stability/observability cleanup for QA efficiency

Out of scope:
- New content systems beyond MVP baseline
- Phase 2 feature expansion (deeper faction rep, full stealth expansion, etc.)
- Large-scale visual redesign unrelated to clarity/readability

## Working Rules
- TDD is mandatory for every behavior change.
- Each ticket starts with failing tests, then implementation, then full gate.
- No cross-domain mega commits; one ticket per mergeable slice.
- Player-facing message text comes from shared policy/helper paths, not ad-hoc copies.
- Any change to control semantics must update help text and test assertions in same ticket.

## Execution Contract (Mandatory)
Every implementation ticket must include all of the following before coding starts:
1. Allowed file touch-list.
2. Forbidden file list.
3. Red tests to write first (test names or exact behavior assertions).
4. Exit evidence links (test output + gate run + artifact path).

Scope control rules:
- If a ticket touches files outside its allowed list, stop and split into a new ticket.
- If a ticket touches more than one policy owner in one pass, split the ticket.
- No ticket may introduce duplicated UX policy logic across menu, colony, and overworld panels.

Policy ownership constraints:
- Instruction hierarchy policy owner: `src/ui/objective_prompt.rs`.
- Modal/escape priority owner: `src/ui/modal_priority.rs` + `src/core/escape.rs`.
- Feedback severity/cooldown/text owner: `src/core/gamelog.rs`.
- Save/load recap owner: `src/core/save.rs`.

## Operational Metrics (Required for Alpha Claims)
- First objective comprehension rate: >= 90% in scripted runs.
- Time to first valid colony->overworld transition: <= 90 seconds median.
- Failed-action comprehension (player identifies why and next step): >= 85%.
- Hint duplication rate in first 5 minutes: <= 1 repeated non-critical hint per run.

These metrics are mandatory for AXT-07 and AXT-08 signoff.

## Ticket Backlog

### AXT-00 Alpha Baseline Freeze
Goal:
- Freeze current behavior and record benchmark evidence for score deltas.

Tasks:
- Capture baseline scorecard using current playtest template.
- Snapshot key flows (menu, colony objective, overworld transition, dungeon entry, save/load recap).
- Record current gate output and targeted UX test output.
- Open alpha-risk register with owner/severity/exit condition.

Allowed files:
- `metrics/**`
- `docs/**`
- `PLAN-2026-05-26-ALPHA-READINESS.md`

Forbidden files:
- `src/**` gameplay and UI behavior files

Tests/Gates:
- Run targeted UX suite.
- Run full gate.

Exit:
- Baseline evidence pack exists under metrics/ with date stamp.

---

### AXT-01 First-Minute Instruction Hierarchy
Goal:
- Make immediate next action obvious at all times in the first 5 minutes.

Tasks:
- Define a single ranked instruction policy (Primary, Secondary, Tertiary).
- Ensure colony objective prompt always outranks ambient helper text until first overworld transition succeeds.
- Reduce duplicated instruction surfaces during onboarding.
- Add deterministic suppression rules for non-critical hints when primary objective is active.

Allowed files:
- `src/ui/objective_prompt.rs`
- `src/ui/help_panel.rs`
- `src/ui/colony_panel.rs`
- `src/ui/overworld_panel.rs`
- `src/tests.rs`

Forbidden files:
- `src/core/save.rs`
- `src/core/escape.rs`
- `src/core/gamelog.rs` (except new message keys if explicitly required and isolated)

Tests:
- Objective priority visibility test.
- First-minute clutter suppression test.
- Regression test for objective persistence until transition success.

Exit:
- In scripted first-time runs, >= 90% of sessions identify next action within 5 seconds.

---

### AXT-02 Esc and Control Semantics Reinforcement
Goal:
- Keep behavior unchanged where correct, but reinforce intent contextually.

Tasks:
- Add one-shot contextual reinforcement text when Esc is introduced in each relevant state.
- Ensure reinforcement text is short, state-specific, and non-spammy.
- Validate no conflicts with modal-priority policy.

Allowed files:
- `src/core/escape.rs`
- `src/ui/modal_priority.rs`
- `src/ui/help_panel.rs`
- `src/core/gamelog.rs`
- `src/tests.rs`

Forbidden files:
- `src/ui/menu.rs`
- `src/core/save.rs`

Tests:
- Esc contextual hint appears when needed and auto-suppresses after acknowledgement.
- Existing Esc determinism tests remain green.
- Modal exclusivity tests remain green.

Exit:
- In scripted runs, Esc misinterpretation rate drops to <= 10% with no modal-priority regressions.

---

### AXT-03 Primary Action Emphasis in High-Traffic Screens
Goal:
- Increase visual priority of the most important next action.

Tasks:
- Identify one primary CTA per major state (Menu, Colony, Overworld).
- Apply consistent emphasis rules (label, weight, contrast, position).
- Reduce visual competition from secondary actions.

Allowed files:
- `src/ui/menu.rs`
- `src/ui/colony_panel.rs`
- `src/ui/overworld_panel.rs`
- `src/ui/readability.rs`
- `src/tests.rs`

Forbidden files:
- `src/core/escape.rs`
- `src/core/save.rs`
- `src/core/gamelog.rs`

Tests:
- Readability/contrast assertions remain >= target thresholds.
- UI snapshot/assertion tests for primary marker presence.

Exit:
- Primary action is obvious without reading full panel copy.

---

### AXT-04 Copy Compression and Feedback Tightening
Goal:
- Improve scan speed under pressure without losing meaning.

Tasks:
- Trim non-critical helper text by 15-25% in first-session pathways.
- Convert passive status phrasing into action-oriented phrasing where appropriate.
- Standardize blocked-action feedback pattern: What failed, Why, Next step.

Allowed files:
- `src/core/gamelog.rs`
- `src/core/save.rs`
- `src/ui/help_panel.rs`
- `src/ui/overworld_panel.rs`
- `src/ui/colony_panel.rs`
- `src/tests.rs`

Forbidden files:
- `src/ui/modal_priority.rs`
- `src/core/escape.rs`

Tests:
- Message helper unit tests for concise format and severity mapping.
- Regression tests for blocked-action guidance and throttling behavior.

Exit:
- Failed actions are understandable in one read.

---

### AXT-05 Save/Load Continuity for Alpha Run States
Goal:
- Ensure player can resume with immediate situational understanding.

Tasks:
- Validate recap quality across colony, overworld, dungeon, and return-to-colony resume states.
- Add load state-specific first action hint when pressure is high.
- Ensure recap data is runtime-derived and consistent.

Allowed files:
- `src/core/save.rs`
- `src/tests.rs`

Forbidden files:
- `src/ui/menu.rs`
- `src/ui/colony_panel.rs`
- `src/ui/overworld_panel.rs`

Tests:
- Save/load recap matrix tests by state.
- Legacy save compatibility checks.

Exit:
- Players can identify risk + next action immediately after load.

---

### AXT-06 QA Observability and Noise Reduction
Goal:
- Keep diagnostics useful without drowning signal.

Tasks:
- Reduce avoidable startup log noise in standard QA workflow.
- Document recommended run profiles for normal playtest vs deep renderer diagnostics.
- Keep error-level diagnostics visible while downgrading non-actionable noise.

Allowed files:
- `src/main.rs`
- `docs/**`
- `scripts/**`

Forbidden files:
- `src/ui/**`
- `src/core/save.rs`
- `src/core/escape.rs`

Tests:
- Smoke launch checks for standard profile.
- Ensure no loss of critical warnings/errors in QA mode.

Exit:
- Playtest logs stay actionable and concise.

---

### AXT-07 End-to-End Alpha Playtest Battery
Goal:
- Validate the complete Alpha claim with reproducible sessions.

Tasks:
- Execute 3 first-time-player script runs (fresh context each run).
- Execute 2 interrupted-run resume scenarios (save/load mid-campaign).
- Execute 1 stress scenario (rapid modal toggles + state transitions).
- Fill standardized scorecards and defect logs.

Defect triage gate (mandatory before AXT-08):
- Every finding has severity, owner, disposition, and linked follow-up ticket if unresolved.
- No open P0/P1 findings.
- Any waived P2/P3 must include explicit rationale and mitigation.

Tests/Gates:
- Full gate after each major fix batch.
- Final full gate before alpha signoff.

Exit:
- Alpha scorecard bars all met and no open P0/P1 defects.

---

### AXT-08 Alpha Signoff and Freeze Prep
Goal:
- Convert implementation status into release-ready Alpha handoff.

Tasks:
- Publish final alpha validation report and score delta vs baseline.
- Update handoff doc with open risks, mitigations, and explicit defer list.
- Tag post-alpha backlog for Beta candidates.

Exit:
- One authoritative Alpha readiness report exists and all acceptance criteria are explicitly checked.

Signoff blockers:
- AXT-07 defect triage gate complete.
- Operational metric targets met.
- Full gate green on final candidate.

## Priority and Sequencing
1. AXT-00 baseline freeze
2. AXT-06 QA observability
3. AXT-01 instruction hierarchy
4. AXT-02 Esc reinforcement
5. AXT-03 primary action emphasis
6. AXT-04 copy compression/feedback tightening
7. AXT-05 save/load continuity matrix
8. AXT-07 end-to-end playtest battery
9. AXT-08 signoff/freeze prep

## Risk Register (Initial)
- R1: Scope creep into non-alpha feature work.
  - Mitigation: strict scope lock and ticket boundaries.
- R2: UI polish changes unintentionally break modal/input behavior.
  - Mitigation: preserve modal priority tests as hard gate blockers.
- R3: Save/load recap diverges from runtime state.
  - Mitigation: derive from runtime resources only; matrix tests.
- R4: Manual playtest variance makes score comparisons noisy.
  - Mitigation: fixed script and fixed score rubric in AXT-07.

## Mandatory Verification Commands
Run from Broken Divinity --Bevy:
- cargo test ux_baseline_red:: -- --nocapture
- cargo test
- ./scripts/test-gate.sh

## Completion Checklist
- [x] AXT-00 complete
- [x] AXT-01 complete
- [x] AXT-02 complete
- [x] AXT-03 complete
- [x] AXT-04 complete
- [x] AXT-05 complete
- [x] AXT-06 complete
- [x] AXT-07 complete
- [x] AXT-08 complete
- [x] Alpha Definition of Done fully met
