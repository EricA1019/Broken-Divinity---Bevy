# Bug Log

Diagnostic ledger only. This file is not product or test authority, cannot
waive a failing canonical gate, and cannot make work complete. Entries under
ACTIVE are unresolved; fixed entries move to HISTORY with a date and commit
hash.

## ACTIVE

None.

## HISTORY

### UI-001 — Cinder Rite implementation stops at palette and outer perimeter
**Severity:** Player-facing visual contract
**File:** `crates/bd_tui/src/chrome.rs`, `crates/bd_tui/src/screens.rs`, and `crates/bd_tui/src/lib.rs`
**Discovered:** 2026-08-02 during owner build review
**Resolved:** 2026-08-02 in the current uncommitted worktree

The owner confirmed that the palette and outer shell are mostly correct, but
ordinary panels and bars retain the previous presentation. Final Outpost and
reusable-screen buffers still use generic double-box inner panels, plain HP/AP
numbers, and the old status/control footer. `VISUAL-THEME-001` and
`VISUAL-IDENTITY-001` are red until the shared panel, meter, mode-ribbon, and
command-ribbon grammar survives both supported profiles and representative
non-colony composition.

**Resolution:** Shared chrome now owns the muted single-rule panel frame,
responsive ASCII meter, mode ribbon, and command ribbon. Outpost, Combat,
Inventory, title, game-over, build, and management composition consume those
primitives. The initial compact fixture used a shorter `8/10` HP value; a real
60x20 PTY run exposed that live `30/30` removed its track. The test fixture and
named compact layout now retain a track for live-sized values.

### STRUCT-001 — Compact build-selection observer rejects approved double-line chrome
**Severity:** Test observer
**File:** `crates/bd_tui/src/ui_development_contract_tests.rs` (VISUAL-IDENTITY-001) and `crates/bd_tui/src/lib.rs` ~line 2798 (`compact_build_selection_shows_complete_selected_effect`)
**Discovered:** 2026-08-02
**Resolved:** 2026-08-02 in the current uncommitted worktree

`selected_cinder_rite_identity_frames_colony_and_reusable_screens` (contract
VISUAL-IDENTITY-001, Cinder Rite / Ruined Reliquary identity) requires the
continuous double-line terminal perimeter (`║` side rails at x=0 and x=W-1 on
every row y=1..H-2) to survive every build workflow, including the build-menu
modal at 60x20. `compact_build_selection_shows_complete_selected_effect` renders
the same 60x20 outpost and asserts the full effect text
`"Produces two Supplies each day when a survivor is physically working here"`
(73 chars; `"Effect: "` prefix makes the line 81 chars) survives whitespace
re-joining after stripping only `│┌┐└┘─`.

The requirements are compatible: the final 60x20 buffer visibly contains the
complete wrapped effect and the approved continuous outer frame. The observer
rejects that output because its normalization strips only single-line chrome;
double-line `║` cells remain attached to wrapped words and break the substring.
This is a helper limitation, not a product or contract conflict.

**Resolution:** The primary test now locates the titled Build Station panel in
the final buffer, reads only its inner semantic region, and requires the
complete selected label, cost, and effect. Approved single- or double-line
chrome is excluded by semantic region instead of an incomplete glyph strip
list. The canonical gate passes 648/648 tests.
