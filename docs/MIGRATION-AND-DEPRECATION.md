# Migration and Deprecation Policy

Broken Divinity is being reorganized around a clearer plan, not restarted from zero.

**Completed Foundation stabilization record:** [FOUNDATION-STABILIZATION-PLAN.md](FOUNDATION-STABILIZATION-PLAN.md)

**Completed Foundation recovery record:** [FOUNDATION-RECOVERY-PLAN.md](FOUNDATION-RECOVERY-PLAN.md)

**Completed Foundation MVP correction record:** [FOUNDATION-MVP-CORRECTION-PLAN.md](FOUNDATION-MVP-CORRECTION-PLAN.md)

**Current documentation inventory:** [DOCUMENT-INVENTORY.md](DOCUMENT-INVENTORY.md)

## Protected project baseline

The current implementation lives in `broken-divinity/`. Treat these as reusable assets:

- `Cargo.toml` and `Cargo.lock`;
- `crates/bd_app` for application wiring;
- `crates/bd_core` for simulation and game rules;
- `crates/bd_tui` for Ratatui presentation and input;
- `crates/bd_data` for content loading and validation;
- `crates/bd_test_support` and existing tests;
- existing content, fixtures, decision records, metrics, and smoke scripts.

## Initial code inventory

The current codebase already contains substantial reusable foundation work:

- **Kernel foundation:** actions, signals, pool deltas, statuses, time, trace guards, relationships, factory/blueprints, inventory, pathfinding, procedural locations, and save/load.
- **Dungeon/combat foundation:** maps, combat actions, enemy AI, spatial state, fixed-dungeon integration points, procedural generation, and Ratatui view models. The first playable dungeon uses the fixed path; procgen remains preserved future infrastructure.
- **Colony foundation:** shelter state, stations, resources, production, and survivors. Raid support remains preserved future infrastructure but is outside the current foundation path.
- **Data foundation:** content IDs, RON loading, registries, and validation.
- **Presentation foundation:** Bevy-Ratatui input, Ratatui screens, render grids, visual/style tokens, themes, and view models.
- **Deferred or optional systems:** sanity, overworld expansion, Gabriel/dialogue, party systems, and deeper faction behavior. These remain in the repository and may be reused later; deferring them does not mean deleting them.

This inventory is an initial classification, not a claim that every module is already correct for the new MVP. The detailed plan must verify behavior and ownership against tests before changing it.

## Preservation rules

1. Do not delete working code, tests, content, or documentation as part of the planning reset.
2. Prefer adapting existing modules behind the intended boundaries over replacing them wholesale.
3. Keep existing tests as regression coverage while ownership moves.
4. Before changing a behavior, identify the current implementation and its tests.
5. If a file is obsolete, mark it deprecated and name its replacement; do not silently remove it.
6. Do not rename or move large directory trees until links and build references have been audited.

## Classification

Every existing artifact should eventually receive one of these labels:

- **Active:** current implementation or canonical documentation.
- **Reuse:** useful implementation to adapt into the new structure.
- **Reconcile:** valuable but conflicting documentation that requires a decision pass.
- **Historical:** retained for context and evidence, not current instructions.
- **Deprecated:** superseded, with a named replacement.

## First migration sequence

1. Capture a recoverable baseline of the current build and test gate.
2. Inventory current modules, systems, tests, content, and documents.
3. Map reusable code to kernel responsibilities and MVP features.
4. Reconcile conflicting documentation against the locked decisions.
5. Execute recovery through the Foundation Recovery Plan.
6. Stabilize any reopened Foundation gates through the Foundation
   Stabilization Plan.
7. Deprecate or archive superseded documents only after replacement links exist.

## Phase 0 migration record — 2026-07-24

### Protected baseline

- Code checkpoint: nested repository commit `1f45ef1`.
- External documentation archive:
  `docs/archive/bd-phase0-docs-baseline-2026-07-24.tar.gz`.
- Hash manifest: `docs/archive/PHASE-0-BASELINE-SHA256.md`.

### Active authority

- Root `GDD.md` is the sole product design authority.
- Root `Kernel.md` is the technical architecture authority.
- `docs/FOUNDATION-STABILIZATION-PLAN.md` was the sole active execution plan
  and is now the completed stabilization record.
- `docs/FOUNDATION-RECOVERY-PLAN.md` is the completed recovery evidence record.
- `docs/MVP-SCENARIO.md` is the canonical Foundation acceptance scenario.
- `docs/DOCUMENT-INVENTORY.md` classifies all other documentation.

### Archived replacements

The following were superseded and preserved under
`docs/archive/foundation-plan-2026-07-24/`:

- historical root UX plan;
- former `ACTIVE-PLAN.md`;
- Phase 3 and Phase 5–11 contracts and remediation plans.

The stale repository-local GDD and repository development plan were moved to
`broken-divinity/docs/archive/` with links to their active replacements.

### Duplicate design material

Project-level `docs/design/` and nested `broken-divinity/docs/` contain both
identical and divergent copies. Identical and divergent pairs are recorded in
`DOCUMENT-INVENTORY.md`.

No unique design material was deleted. Divergent files remain Reconcile until
their unique content is compared with the root GDD. Identical repository copies
are Deprecated but preserved until link-safe consolidation is explicitly
scheduled.

## Foundation completion record — 2026-07-24

Foundation Recovery completed without replacing the workspace or discarding
the reusable kernel.

### Active and reused

- `bd_core` remains the sole owner of Foundation session transitions, player
  creation, action resolution, colony state, dungeon state, and persistence
  snapshot contents.
- `bd_app` owns process startup, validated content assembly, paths, save/load
  I/O, restored-screen routing, and clean application exit.
- `bd_tui` owns semantic terminal commands, contextual guidance, screen
  selection, view models, and rendering.
- `bd_data` owns Foundation RON loading and cross-record validation.
- `bd_test_support` owns the action-driven canonical scenario harness.

The duplicate application-level player spawner was removed after the final
manual audit proved that it competed with the kernel’s player owner.

### Preserved for later products

Procgen, overworld, raids, events, sanity, Gabriel/dialogue, deeper party
systems, and richer faction behavior remain in the repository as deferred
infrastructure. Their presence is not evidence that they are active in the
Foundation runtime.

### Deprecated or historical

The superseded plans and repository-local design authorities listed in
`DOCUMENT-INVENTORY.md` remain archived. `broken-divinity/KNOWN_ISSUES.md` is a
historical 2026-07-11 snapshot; current Foundation limitations are recorded in
the completed recovery plan.

The clean-session audit performed after this record reopened Foundation gates
for economy ownership, defeat cleanup, terminal behavior, and acceptance
coverage. `FOUNDATION-STABILIZATION-PLAN.md` authorizes only those repairs.
Product P2 remains unauthorized and must start with its own plan and an
explicit decision about which preserved systems to activate.

## Foundation stabilization record — 2026-07-25

Stabilization Phases 0–7 reused the protected workspace and repaired the
reopened Foundation gates without activating deferred Product P2 systems.

- `bd_core` retains authoritative action, economy, transition, time,
  daily-cycle, defeat, and persistence ownership.
- `bd_tui` now uses invalidation-driven rendering, compact exact-size layouts,
  contextual navigation, and one bounded semantic gameplay-input queue.
- `bd_app::KeyBindingConfig::default` derives characters from the built-in
  semantic command bindings. Shipped TOML and README controls are checked
  against that owner.
- Existing procgen, raids, events, sanity, overworld, Gabriel, and deeper
  faction modules remain preserved and deferred.
- Strict workspace Clippy and focused behavior suites pass after
  behavior-neutral maintainability cleanup.

Foundation acceptance passed on 2026-07-25. Phase 8 closed the reopened gates
through the complete automated matrix, clean 80x24 and 60x20 terminal
scenarios, persistence/failure checks, and final GDD reconciliation. Existing
deferred infrastructure remains preserved and inactive. Product P2 remains
unauthorized.

A later 2026-07-25 discovery run reopened acceptance after exposing a skipped
colony transaction at Tactical day boundaries and an unprotected adverse
economy path. `FOUNDATION-MVP-CORRECTION-PLAN.md` is the owner-authorized
response. It does not authorize Product P2.

## Foundation MVP correction record — 2026-07-25

The correction plan completed without replacing the kernel or activating
deferred systems.

- Existing daily-cycle systems were reordered into one mode-independent
  transaction rather than duplicated.
- Existing colony resources, survivors, stations, actions, persistence, and
  TUI boundaries were retained. Station facts moved into one validated RON
  catalog used by simulation and presentation.
- The fixed dungeon remained on the existing content pipeline and was expanded
  as hand-authored content; procgen remains preserved and inactive.
- `LastCompletedRun`, named shelter returns, explicit management, forecast
  projection, and friendly persistence failures extend existing owners rather
  than creating alternate state.
- Storage remains preserved for compatibility but is disabled until a later
  approved product gives it an implemented effect.
- Procgen, overworld expansion, raids, events, sanity, Gabriel/dialogue,
  theology-driven mechanics, reputation, final factions, and deeper narrative
  remain reusable deferred infrastructure.

The full automated gate and isolated 80x24/60x20 play audit pass. Foundation
acceptance is current; no execution plan is active. Product P2 still requires
a new owner-approved plan and an explicit decision about which preserved
systems enter scope.

## Foundation basic colony-loop migration record — 2026-07-27

The owner-approved D-20 pass extended existing owners rather than replacing
the kernel:

- `ColonyResources` still owns finished pools and now also owns canceled raw
  cargo deposits.
- Existing survivor movement/pathfinding and occupancy reservations are reused
  by a separate durable logistics transition.
- Existing station content gained one data-defined Basic Processing entry and
  one guaranteed starter instance; all five prior entries remain preserved.
- Existing fixed shelter/dungeon topology remains unchanged. Only
  deterministic resource-fixture placement on the fixed shelter was added.
- Existing save snapshots gained defaulted job, cargo, raw-resource, source,
  and node identity fields so older compatible snapshots retain defaults.
- Existing Ratatui semantic projection gained recipe/stage/target/cargo
  details and responsive six-entry controls without introducing a second UI
  state owner.

No procgen topology, raids, events, sanity, overworld generation, faction
reputation, Product P2 automation, queues, upgrades, or depletion balance was
activated. C0–C7 implementation and C8 PTY behavior are green. Formal visual
acceptance remains pending owner review under D-19.
