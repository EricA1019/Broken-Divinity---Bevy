# Phase 0 Baseline - Full Scope Tightening (2026-05-29)

## Purpose

Establish a verifiable baseline on branch `feature/dev-ui-full-scope-tighten` before any implementation for continuity, character interaction micro-beats, and thematic pass.

## Branch Cut

- Source branch: `dev`
- New branch: `feature/dev-ui-full-scope-tighten`
- Status at cut: clean working tree

## Baseline Commands and Results

1. Branch creation
- Command: `git checkout -b feature/dev-ui-full-scope-tighten`
- Result: success

2. Unified dev launch check
- Command: `cargo run --bin broken_divinity --features dev`
- Result: success
- Evidence:
  - Launch banner reports: `Mode: Unified UI default (dev tooling enabled)`
  - Window title reports unified path: `Broken Divinity [Unified UI]`
  - Observed warning: `RemoteHttpPlugin is already added` and BRP extras port config ignored

3. Full regression baseline
- Command: `cargo test -j1`
- Result: success (all reported suites passing; no failing test in tail summary)

## Target Runtime Surfaces in Scope

Primary touch surfaces for upcoming phases:
- Unified flow composition in [src/ui/ux_unified_prototype.rs](src/ui/ux_unified_prototype.rs)
- Shared hint tokens in [src/ui/input_hints.rs](src/ui/input_hints.rs)
- Objective continuity policy in [src/ui/objective_prompt.rs](src/ui/objective_prompt.rs)
- Feedback channel in [src/ui/gamelog_panel.rs](src/ui/gamelog_panel.rs)
- Existing character interaction surfaces in [src/ui/gabriel_dialogue_panel.rs](src/ui/gabriel_dialogue_panel.rs) and [src/game/dungeon/gabriel.rs](src/game/dungeon/gabriel.rs)
- Flow contracts in [tests/runtime_flow_contracts.rs](tests/runtime_flow_contracts.rs), [tests/menu_runtime_contracts.rs](tests/menu_runtime_contracts.rs), and [tests/ui_phase5_runtime_contracts.rs](tests/ui_phase5_runtime_contracts.rs)

## Baseline Friction Inventory (Pre-Implementation)

Confirmed from current code/layout:

1. Mode-to-mode continuity is weak in unified prototype path.
- Current top header is a key legend plus current screen label, but not a persistent context model (no previous context breadcrumb, no explicit next action recommendation).

2. Screen switching is direct-key driven and abrupt.
- Unified screen transitions are immediate toggles (`M`, `D`, `C`, `O`, `P`, `I`) without transition framing/carry-forward guidance.

3. Control hints can drift across surfaces.
- Shared hint infrastructure exists in runtime surfaces, but unified prototype header still carries a hardcoded long legend that can diverge from canonical hint tokens.

4. Character interaction exists but is localized.
- Gabriel interaction is present and robust in dungeon lifecycle, but continuity to non-dungeon unified surfaces is not represented.

5. Thematic framing exists in content/docs but is not consistently surfaced as concise microcopy in unified transitions.

## Baseline Criteria Snapshot

| Criterion | Baseline Status | Notes |
| --- | --- | --- |
| Continuity coverage (context + next action on each target screen) | Partial | Context label exists, explicit next action cue is not consistently present. |
| Transition clarity (non-contradictory key/action wording) | Partial | Unified header legend is dense and local; risk of drift with canonical hint tokens. |
| Interaction restraint (event-driven, non-blocking) | Mostly pass | Gabriel interactions are event-gated and non-global. |
| Thematic quality (tone-safe, concise) | Partial | Tone guide exists; no centralized thematic microcopy registry in unified transitions yet. |
| Regression safety (contracts green) | Pass | Full suite baseline run passes on branch cut. |

## Constraints Reaffirmed for Phase 1+

- Test-first for each behavior slice.
- No save schema changes.
- No combat/economy rebalance.
- No new global dialogue framework.
- Shared copy/state ownership required to avoid DRY and coherence drift.

## Phase 0 Exit Criteria

- [x] New branch created from current `dev`.
- [x] Unified launch baseline captured.
- [x] Full regression baseline captured.
- [x] Runtime surfaces and friction inventory documented.
- [x] Criteria snapshot recorded for comparison after implementation.
