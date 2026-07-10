# Alpha Risk Register (2026-05-26)

## Purpose
Track alpha-delivery risks with owners, severity, and explicit exit conditions.

## Legend
- Severity: P0 (release blocker), P1 (serious), P2 (moderate), P3 (low)
- Status: Open, Mitigated, Closed

## Risks

### AR-01 Scope Creep Across AXT Tickets
- Severity: P1
- Owner: Engineering
- Status: Open
- Risk: UX tickets blend behavior, wording, and style in one pass and exceed scope.
- Mitigation:
  - Enforce per-ticket allowed/forbidden file lists.
  - Split ticket when touching more than one policy owner.
- Exit condition:
  - No ticket violates allowed file boundaries.

### AR-02 Modal/Esc Regression While Polishing UX
- Severity: P0
- Owner: Engineering
- Status: Open
- Risk: Copy or panel updates accidentally break modal priority and Esc determinism.
- Mitigation:
  - Keep modal/escape regression tests as blockers.
  - Run ux_baseline_red suite after each major step.
- Exit condition:
  - Modal/Esc tests pass in all AXT stages.

### AR-03 Save/Load Recap Drift From Runtime Truth
- Severity: P0
- Owner: Engineering
- Status: Open
- Risk: Recap text diverges from actual game state and misleads player post-load.
- Mitigation:
  - Derive recap from runtime resources only.
  - Add state-matrix tests for colony, overworld, dungeon, return flows.
- Exit condition:
  - Save/load matrix tests pass with no stale-data defects.

### AR-04 Playtest Variance Masks Real UX Delta
- Severity: P1
- Owner: QA
- Status: Open
- Risk: Inconsistent session scripts create noisy score deltas.
- Mitigation:
  - Fixed script and scoring rubric for AXT-07.
  - Record scenario context for each run.
- Exit condition:
  - Three first-session runs captured with consistent protocol.

### AR-05 Diagnostic Noise Obscures Actionable Failures
- Severity: P2
- Owner: Engineering
- Status: Open
- Risk: Startup logging volume hides meaningful warnings/errors during QA.
- Mitigation:
  - Define standard QA log profile before UX-heavy steps.
  - Keep deep diagnostics profile separate.
- Exit condition:
  - QA profile highlights actionable warnings/errors only.

## Review Cadence
- Update this file at the end of each completed AXT ticket.
- Close or downgrade risks with evidence links from test/gate runs.
