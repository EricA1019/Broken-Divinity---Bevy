# Phase Sheet: Dev UI Cleanup and Panel Polish

Use the dev unified runtime for manual walkthroughs: `cargo run --bin broken_divinity --features dev`. Treat the non-dev runtime as the rollback boundary only.

## Phase 0 - Baseline inventory and policy lock
- Inventory generated outputs, captures, and transient files.
- Lock the keep/remove boundary.
- Record the current dev unified launch path.
- Exit when the keep list, remove list, ignore-policy status, and launch command are documented.

## Phase 1 - Test scaffolding for the first polish slice
- Add or update slice-specific tests or contracts before touching launch, menu, or help behavior.
- Exit when the first slice has a named regression check that fails before implementation starts.

## Phase 2 - Build artifact cleanup and ignore hygiene
- Add or fix root ignore rules if they are missing or incomplete.
- Remove only disposable local clutter such as `target`, transient logs, and temporary captures.
- Exit when `git status --short` is free of new generated clutter and tracked baselines remain untouched.

## Phase 3 - Launch, menu, and help surfaces
- Align visible labels, banners, and shortcut hints with the real keybindings.
- Centralize repeated copy instead of duplicating it.
- Exit when a new player can tell the active mode and next step from these surfaces alone.

## Phase 4 - Save/load and quit flow
- Add tests first for save/load and quit messaging.
- Tighten confirmations and blocked-action feedback.
- Exit when persistence and exit behavior are explicit, repeatable, and do not fail silently.

## Phase 5 - Colony and overworld panels
- Polish these together only as a narrow shared progression slice.
- Improve resource, travel, and blocked-action feedback.
- Exit when the player can explain the actions from the screen itself.

## Phase 6 - Dossier and inventory/equipment panels
- Polish these together as a tabbing, layout, and slot-readability slice.
- Keep equipment rules and save compatibility stable.
- Exit when the panels read as deliberate UI instead of debug surfaces.

## Phase 7 - Cross-panel coherence pass
- Reconcile labels, shortcut hints, and context feedback across all touched surfaces.
- Remove remaining drift only.
- Exit when a final first-time-player walkthrough shows no contradictory labels, stale hints, or dead-end actions.

## Guardrails
- Keep each slice to 1 or 2 tightly coupled systems.
- Start with tests or contracts, then implement the smallest change that makes them pass.
- Validate against the plan before each phase starts and again at the phase gate.
- Do not add gameplay rebalance, save schema changes, or architecture rewrites unless the phase explicitly calls for them.
- Do not delete tracked docs, handoffs, metrics, or playtest artifacts unless the phase explicitly says they are disposable.
- Preserve `broken_divinity_save.json` as a persistence baseline, not cleanup clutter.

## Done When
- Cleanup boundaries are explicit and enforced.
- Launch, menu, and help explain the active mode clearly.
- Save/load and quit are unambiguous.
- Colony, overworld, dossier, and inventory each communicate their own actions and blockers.
- The dev unified UI feels coherent from launch through the touched panels.
