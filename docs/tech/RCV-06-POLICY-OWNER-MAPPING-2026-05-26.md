# RCV-06 Architecture and Policy Owner Mapping (2026-05-26)

## Scope
Maps AXT policy areas to concrete module owners in the current checkout and records recovery subtasks where intended owners are missing.

## Owner Mapping Table

| Policy Area | Intended Owner | Current Owner Decision | Status | Notes |
| --- | --- | --- | --- | --- |
| Instruction hierarchy | `src/ui/objective_prompt.rs` | `src/ui/objective_prompt.rs` | READY | Owner file created and compiled through crate root exports. |
| Modal/escape priority | `src/ui/modal_priority.rs` + `src/core/escape.rs` | `src/ui/modal_priority.rs` + `src/core/escape.rs` | READY | Owner files created and compiled through crate root exports. |
| Feedback severity/cooldown/text | `src/core/gamelog.rs` | `src/core/gamelog.rs` | READY | Owner file created and compiled through crate root exports. |
| Save/load recap | `src/core/save.rs` | `src/core/save.rs` | READY | Owner exists and is aligned with plan intent. |

## Recovery Subtasks Required Before Dependent AXT Work
1. `RCV-06A`: COMPLETE - dedicated instruction hierarchy owner module established.
2. `RCV-06B`: COMPLETE - dedicated modal/escape policy owner module pair established.
3. `RCV-06C`: COMPLETE - dedicated feedback policy owner module established.

## Extension Boundaries
1. Policy behavior must be defined in one owner module per policy area.
2. UI panels may consume policy outputs but must not re-implement policy logic.
3. State-transition systems may expose hooks only; policy decisions remain in owner modules.
4. Save/load recap policy remains centralized in `src/core/save.rs`.

## AXT Gate Implications
1. `AXT-01` owner-file blocker cleared.
2. `AXT-02` owner-file blocker cleared.
3. `AXT-04B` owner-file blocker cleared.
4. `AXT-05` still depends on runtime recap path restoration.

## Exit Check
- Owner mapping table exists.
- Missing-owner recovery subtasks are explicitly named.
- Dependent AXT blockers are explicit and traceable.
