# Broken Divinity Foundation Stabilization Plan

**Status:** Complete owner-authorized Foundation stabilization record

**Effective date:** 2026-07-24

**Product target:** Trustworthy Broken Divinity Foundation MVP

**Current phase:** Phase 8 complete; its acceptance was later reopened and
restored by the completed MVP correction record

**Next authorized work:** None; Product P2 requires a new owner-approved plan

**Extends:** [FOUNDATION-RECOVERY-PLAN.md](FOUNDATION-RECOVERY-PLAN.md)

**Does not authorize:** Product P2, deferred systems, or new content

**Does not supersede:** `GDD.md`, `Kernel.md`, `Kernel-direction.md`, or locked
decisions in `DECISIONS-TO-LOCK.md`

## 1. Purpose

The 2026-07-24 Foundation Recovery established a reusable shell and a playable
dungeon round trip. A later clean-session discovery test proved that the
Foundation acceptance gate was too weak:

- construction validates one resource owner and charges another;
- Foundation dungeon entry bypasses the action and resource-cost path;
- player defeat reaches Game Over while emitting an invalid-entity command
  error;
- exit and gate guidance mutates the log every application frame;
- the TUI redraws continuously while idle;
- the declared 80x24 supported layout clips controls and hides the build menu;
- inventory, daily-cycle, buffered-input, log-order, and binding guidance are
  not coherent enough for a first-time player;
- strict Clippy validation is not green;
- active documentation still claims an unqualified final pass.

This plan repairs those regressions without restarting the project or
expanding the product. It reuses the recovery architecture, test harness,
content, and implementation wherever they are sound.

The target player journey remains:

```text
Persistent shelter
  → visible preparation with real resource consequences
  → fixed dungeon
  → combat, loot, and explicit extraction or defeat
  → exact return result
  → persistent shelter and save state
```

The plan is complete only when headless tests, exact-size terminal tests,
manual play, persistence, code quality, and canonical documentation all
describe the same behavior.

## 2. Authority and Reading Order

Implementation agents must read:

1. `docs/README.md`;
2. `GDD.md`;
3. `docs/DECISIONS-TO-LOCK.md`;
4. relevant sections of `Kernel.md`;
5. relevant sections of `Kernel-direction.md`;
6. `docs/MIGRATION-AND-DEPRECATION.md`;
7. `docs/MVP-SCENARIO.md`;
8. this plan;
9. `docs/FOUNDATION-RECOVERY-PLAN.md` as completed recovery evidence;
10. current code, tests, and content.

Authority rules from the recovery plan remain binding:

- the GDD owns player experience and product scope;
- locked decisions own Foundation exclusions;
- `Kernel.md` owns technical boundaries;
- during execution, this document is the sole active execution plan;
- the completed recovery plan remains evidence, not current task authority;
- code and passing tests do not override a failed player-facing gate.

## 3. Scope

### 3.1 Required stabilization outcomes

- one authoritative owner for colony resources;
- station construction validates and charges that same owner exactly once;
- the Foundation dungeon-entry command uses the shared action pipeline;
- the existing Foundation entry cost is visibly charged to colony supplies;
- insufficient supplies reject construction and dungeon entry without partial
  mutation;
- no Foundation player entity carries a second authoritative colony-supplies
  pool;
- defeat produces one result, one cleanup transaction, one Game Over
  transition, and no command error;
- location guidance is contextual UI state, not a per-frame game-log event;
- idle application updates do not mutate gameplay state or the game log;
- terminal rendering occurs only when visible state, terminal size, or
  terminal lifecycle requires it;
- 80x24 and 60x20 layouts have deterministic, tested behavior;
- the build menu always shows station names, costs, selection, and controls;
- inventory has a visible and consistent return/toggle path;
- rendered log entries read in causal chronological order;
- one player-visible shelter command can safely advance to the next daily
  boundary through authoritative time logic;
- buffered gameplay input is retained in a bounded semantic queue or rejected
  visibly; it is never silently lost;
- built-in bindings, shipped configuration, input, help, footer, action panel,
  and README agree;
- the full workspace passes strict Clippy without broad suppression of
  architectural findings;
- canonical status documents accurately report that Foundation acceptance is
  reopened until the final gate passes.

### 3.2 Explicitly deferred

The exclusions from the recovery plan remain unchanged:

- procedural generation in the Foundation path;
- full overworld travel or a Travel-mode expansion;
- raids and colony events;
- sanity;
- theology-driven mechanics;
- faction reputation and final faction canon;
- additional enemies, items, dungeons, floors, or station types;
- multiple save slots or autosaves;
- graphical or asset-heavy presentation;
- broad balancing or content polish.

The Foundation dungeon-entry cost is not an overworld implementation. It is a
single preparation consequence on the existing direct shelter-to-dungeon
action.

### 3.3 Prohibited shortcuts

This work must not:

- special-case the TUI to mutate `ColonyResources`;
- make tests pass by adding supplies to the player entity;
- call production systems directly from an End Day/Rest command;
- teleport the player or insert loot in acceptance tests;
- hide the defeat command error by lowering log severity;
- remove the Game Log cap or use deduplication as a substitute for stopping
  per-frame writes;
- lower the application frame rate as the only rendering fix;
- increase minimum terminal dimensions to avoid supporting the declared
  80x24 profile;
- hardcode a second control table in widgets or application code;
- use broad crate-level Clippy allowances to declare the quality gate green;
- activate preserved deferred plugins to reuse unrelated code.

## 4. Confirmed Stabilization Defects

Each implementation change, test, and phase record must reference these IDs.

| ID | Owner | Confirmed defect | Required outcome |
|---|---|---|---|
| FS-001 | Phase 0 | Active documents claim an unqualified Foundation pass after the discovery gate failed | One honest active status and one active plan |
| FS-002 | Phases 1–2 | Canonical station setup does not assert global resource deduction | Production-path acceptance proof checks before/after colony resources |
| FS-003 | Phase 2 | Build validation reads `ColonyResources` while cost resolution targets player `Pools` | Validation and payment use one typed resource owner |
| FS-004 | Phase 2 | Foundation player state contains a second Supplies pool | Colony supplies have one Foundation authority |
| FS-005 | Phase 2 | TUI Travel directly requests Tactical mode and pays no cost | A validated Foundation action charges colony supplies and transitions once |
| FS-006 | Phase 3 | Normal fatal combat emits an invalid/despawned-entity Bevy command error | One cleanup owner and error-free defeat |
| FS-007 | Phase 4 | Exit and shelter-gate detection push the same hint every frame | Context hint is derived or edge-triggered without log churn |
| FS-008 | Phase 4 | `draw_ui` invokes terminal drawing on every 60 Hz application update | Terminal draws are invalidation-driven |
| FS-009 | Phase 4 | Render errors are discarded | Draw failure is surfaced and terminal shutdown remains controlled |
| FS-010 | Phase 5 | 80x24 footer wraps/clips and visually corrupts command labels | Exact-size snapshots prove a readable layout |
| FS-011 | Phase 5 | Build options are outside the visible map panel at 80x24 | A visible modal/panel contains every option and control |
| FS-012 | Phase 5 | Inventory lacks contextual controls and `I` does not toggle back | Inventory exposes and accepts one consistent return path |
| FS-013 | Phase 5 | Newest-first log rendering reads combat causality backward | Visible window is oldest-to-newest |
| FS-014 | Phase 6 | Reaching a production boundary requires 24 manually repeated actions | Shelter exposes a safe Rest Until Next Day action |
| FS-015 | Phase 6 | Multiple buffered gameplay keys can be silently discarded by the action lock | Bounded ordered input handling with visible overflow behavior |
| FS-016 | Phase 7 | README, built-in bindings, and shipped config disagree on assignment | One binding specification and drift tests |
| FS-017 | Phase 7 | Strict workspace Clippy fails | Strict lint gate passes without behavior change |
| FS-018 | Phases 1–8 | Existing gates pass while missing runtime economy, layout, and command-error regressions | Acceptance suite fails for these defects before fixes and proves them after |

New observations are not added as implementation tasks until reproduced by a
test, terminal capture, or direct ownership proof. New product choices trigger
the stop protocol.

## 5. Locked Stabilization Decisions

These decisions apply only to the Foundation stabilization scope.

### S-01 — Stabilization before Product P2

No Product P2 work begins until this plan passes. The recovery code remains the
baseline; this is not a rewrite.

### S-02 — One active execution plan

During execution, this document is the sole active execution plan. The
Foundation Recovery Plan remains the immutable chronological record of what
was previously attempted and accepted, with a post-acceptance note identifying
reopened gates.

### S-03 — Colony resources have one owner

`bd_core::colony::production::ColonyResources` owns Foundation shelter
resources. A Foundation player entity does not own a second Supplies pool.

An action requirement and its accepted cost must resolve against the same
typed owner in the same transaction. The action model may distinguish actor,
target, and colony resource ownership explicitly; it may not infer ownership
from `PoolKind` in scattered branches.

### S-04 — Foundation dungeon entry is an action

The existing direct shelter-to-fixed-dungeon interaction remains. It is routed
through one canonical Foundation action and costs the existing named value of
2 colony Supplies.

The action:

- validates Outpost mode and sufficient colony Supplies;
- charges exactly once;
- emits the Tactical transition only after accepted cost resolution;
- emits a typed denial when blocked;
- records complete replay data;
- does not activate the deferred overworld or Travel mode.

Changing the value of 2 is balance work and is outside this plan.

### S-05 — Context hints are projections

“Press to extract,” “press to travel,” pickup availability, and comparable
location hints are derived from authoritative state in the action/context view
model. Merely standing still never creates another gameplay result or log
entry.

### S-06 — Rendering is invalidation-driven

The Bevy schedule may continue to update at its existing cadence, but Ratatui
draws only when:

- visible view-model content changes;
- the active screen or interaction mode changes;
- terminal dimensions change;
- the terminal is initialized or restored;
- an explicit forced redraw is requested after an error/recovery boundary.

The invalidation owner belongs in `bd_tui`. Simulation code does not know about
terminal redraws. A terminal draw error is reported to stderr, requests a clean
application exit, and must still restore terminal state.

### S-07 — Supported terminal profiles are real contracts

- 80x24 is the required baseline play profile.
- 60x20 is a supported compact profile.
- Smaller terminals show the existing resize message.

An exact-size buffer snapshot, not a string-presence assertion, proves each
profile. The build menu may replace or overlay the map while open; it must not
depend on unused lines below the map panel.

### S-08 — Inventory navigation is contextual

The Inventory command toggles between Inventory and the gameplay screen for the
current session mode. A visible Back control performs the same return. Existing
`Z` map behavior may remain as a compatibility binding, but it is not the only
documented exit.

### S-09 — Rendered logs are chronological

`GameLog` may retain its bounded internal representation. The player-visible
window renders selected entries oldest-to-newest so cause precedes result.

### S-10 — Rest reaches, but does not bypass, the day boundary

The shelter gains one semantic `Rest Until Next Day` command. It is available
only in Outpost mode.

The command advances authoritative time to the next boundary and emits exactly
one `DayAdvanced`. Production, consumption, gathering, mood, and summaries
remain consumers of that boundary. The command does not invoke those systems
directly and is unavailable during Tactical, Game Over, build, and event
interaction.

### S-11 — Buffered input is bounded and semantic

Gameplay commands are translated to semantic commands and processed in order.
When the player-action lock is active, pending gameplay commands remain in a
small named-capacity queue. Overflow is rejected with one visible warning.
Screen, cancel, quit, and terminal lifecycle controls remain responsive and are
not trapped behind gameplay turns.

Queue capacity is an implementation constant with a test; changing it is not a
balance decision.

### S-12 — Strict lint is a release gate

`cargo clippy --workspace --all-targets -- -D warnings` must pass. Touched
systems should use bounded `SystemParam` owners, helper types, and derived
defaults where appropriate. Narrow local lint allowances require an adjacent
reason and plan evidence; blanket suppression is prohibited.

## 6. Implementation Protocol

Every behavior phase follows the recovery protocol:

1. Re-read the named GDD sections and locked decisions.
2. State the behavior contract and failure cases in test names or test comments.
3. Add a failing test through the intended production path.
4. Run it and record the expected red failure.
5. Implement the smallest coherent behavior that passes.
6. Refactor only while focused tests remain green.
7. Run phase-specific tests.
8. Run the workspace gate.
9. Perform the phase GDD drift check.
10. Run the phase manual check when required.
11. Record evidence using the template in Section 18.
12. Begin the next phase only after the exit gate passes.

Rules:

- Tests are written before implementation.
- A test that asserts the current wrong architecture is not acceptance proof.
- Unit fixtures may construct focused state, but integration tests must use
  production startup, actions, transitions, and persistence.
- No phase is complete because compilation or headless tests pass.
- At most one phase may be In progress.
- Unexpected runtime warnings fail the relevant phase even when the process
  exits with code 0.

## 7. Ambiguity and Stop Protocol

Stop and ask the owner when:

- fixing a defect requires enabling a deferred system;
- the 2-Supplies direct-entry contract conflicts with canonical content;
- a Foundation player Supplies pool is proven to have another active,
  documented responsibility;
- Rest Until Next Day would require redefining daily production semantics;
- Bevy-Ratatui cannot support invalidation-driven rendering without replacing
  the terminal runtime;
- save compatibility with released external saves is discovered;
- a control change cannot preserve both accessibility and current semantic
  binding ownership;
- a new item, station, enemy, map, faction, or narrative requirement appears;
- canonical documents conflict.

When stopped:

1. record the exact conflict in phase evidence;
2. preserve the failing test;
3. identify the smallest mutually exclusive choices;
4. ask the owner;
5. update the decision register;
6. resume only after authority is consistent.

## 8. Phase 0 — Authority, Baseline, and Reopened Status

### Purpose

Make the regression state honest and capture a reproducible baseline before
code changes.

### Dependencies

None.

### Documentation work

1. Name this document as the sole active execution plan.
2. Keep `FOUNDATION-RECOVERY-PLAN.md` as a completed chronological record.
3. Add a post-acceptance regression note to that record linking FS-001 through
   FS-018 and this plan.
4. Mark the Foundation MVP scenario acceptance status Reopened.
5. Update:
   - `docs/README.md`;
   - `docs/DOCUMENT-INVENTORY.md`;
   - `docs/DECISIONS-TO-LOCK.md`;
   - `docs/MIGRATION-AND-DEPRECATION.md`;
   - `Kernel-direction.md`;
   - `broken-divinity/README.md`;
   - `broken-divinity/docs/README.md`.
6. Correct current README controls to the shipped runtime while Phase 7 owns
   the permanent single-source repair.
7. Do not edit the GDD; no product scope changed.

### Baseline capture

Record:

- branch and commit;
- `git status --short`;
- `git diff --stat`;
- Rust and Cargo versions;
- isolated XDG paths used by manual testing;
- exact terminal dimensions;
- current save version and content version;
- current automated results;
- current strict Clippy failures;
- clean-session captures for:
  - construction resource mismatch;
  - unpaid dungeon entry;
  - extraction prompt churn;
  - 80x24 layout;
  - defeat command error.

Required commands:

```text
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo run -p bd_app -- --validate
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
```

Clippy is expected red at baseline. Any other regression not already in the
defect table must be triaged before Phase 1.

### Internal validation

- One active plan is discoverable from every active index.
- No active document claims the Foundation is currently accepted.
- The completed recovery evidence remains preserved.
- Product P2 remains unauthorized.
- Every confirmed defect has reproducible evidence.

### GDD drift check

Review GDD Sections 2, 3, 6, and 8 plus D-01, D-02, D-09 through D-12, D-15,
and D-16. Phase 0 changes status and navigation only.

### Exit gate

- Baseline is reproducible.
- Authority is unambiguous.
- Discovery evidence is attached to stable defect IDs.
- No gameplay code changed.

### Phase 0 completion evidence — 2026-07-24

```text
Phase: 0 — Authority, Baseline, and Reopened Status
Status: Complete
Scope: FS-001 and reproducible baseline evidence for FS-002 through FS-018
Baseline branch: main
Baseline commit: 1ac674f4e452287330c7da01df90207dcda7efc7
Pre-existing worktree changes: broken-divinity/README.md and
  broken-divinity/docs/README.md documentation-authority corrections
Rust: rustc 1.97.0 (2d8144b78 2026-07-07)
Cargo: cargo 1.97.0 (c980f4866 2026-06-30)
Save version: 5
Content version: foundation-2026-07-24
Terminal profile: 80x24, TERM=xterm-256color
Isolated XDG root: /tmp/bd-foundation-stabilization-phase0
```

Automated baseline:

- `cargo fmt --all -- --check`: PASS;
- `cargo check --workspace`: PASS;
- `cargo test --workspace`: PASS, 370 passed and 2 ignored;
- `cargo run -p bd_app -- --validate`: PASS;
- `git diff --check`: PASS;
- `cargo clippy --workspace --all-targets -- -D warnings`: expected RED,
  25 library-target findings and 8 additional test-target findings.

Clean-session evidence:

- FS-002–FS-004: after building a 2-Supplies Stove, the 80x24 Stats panel
  continued to display 10 colony Supplies. The resulting version-5 save
  contained player `PoolKind::Supplies = 8`, `ColonyResources::Supplies = 10`,
  and the built Stove.
- FS-005: pressing the configured Travel command entered
  `dungeon.foundation` immediately. The resulting replay contained only the
  earlier build action, session turn remained 0, and colony Supplies remained
  10.
- FS-006: deterministic waiting until fatal enemy damage reached Game Over but
  emitted Bevy's `Entity despawned` command warning for entity `0v0`.
- FS-007–FS-008: remaining idle at the shelter gate produced repeated
  full-terminal control sequences at the 60 Hz application rate while the
  shelter-gate hint occupied the log.
- FS-010: the 80x24 footer began at column 52 on row 22, wrapped across three
  rows, and split/corrupted command labels.
- FS-011: opening Build at 80x24 showed only the log instruction; the five
  station options were not visible until a selection moved the application
  into ghost placement.
- FS-015: sending the valid sequence `b`, `1`, `d`, `Enter` in one buffered
  input burst dropped the station selection and produced `Select a station
  type first (1-5).`

GDD Sections 2, 3, 6, and 8 and decisions D-01, D-02, D-09 through D-12,
D-15, and D-16 were reviewed. The stabilization defects are regressions
against costly preparation, shelter continuity, tactical consequence, the
Foundation scope, and the locked terminal runtime; no Product P2 behavior was
introduced. Authority searches found one active plan, all active status
documents report acceptance reopened, and Product P2 remains unauthorized.
No gameplay code changed in Phase 0.

## 9. Phase 1 — Honest Stabilization Acceptance Harness

### Purpose

Make the current defects fail automatically before implementation.

### Dependencies

Phase 0 complete.

### Tests first

Extend production-path acceptance with initially failing tests:

```text
construction_deducts_authoritative_colony_supplies_once
construction_denial_preserves_all_state
foundation_player_has_no_colony_supplies_pool
dungeon_entry_deducts_colony_supplies_once
dungeon_entry_denial_preserves_mode_turn_and_resources
dungeon_entry_replay_preserves_cost_and_transition
fatal_enemy_action_reaches_game_over_without_command_error
fatal_action_emits_one_defeat_and_one_cleanup
idle_on_extraction_tile_does_not_mutate_log
idle_on_shelter_gate_does_not_mutate_log
idle_unchanged_ui_does_not_draw_again
render_failure_is_observable
outpost_80x24_snapshot_is_readable
build_menu_80x24_snapshot_shows_all_options
inventory_80x24_snapshot_shows_return_control
compact_60x20_snapshots_preserve_required_controls
rendered_combat_log_is_chronological
rest_until_next_day_emits_one_day_boundary
rest_until_next_day_is_rejected_outside_outpost
buffered_gameplay_commands_resolve_in_order
buffered_command_overflow_is_visible_and_bounded
shipped_bindings_match_builtin_bindings
readme_default_controls_match_shipped_bindings
```

### Harness approach

1. Extend `bd_test_support` read-only summaries with:
   - authoritative colony pool values;
   - player pool kinds;
   - result counts;
   - entity cleanup counts;
   - game-log snapshots;
   - render-call counts.
2. Do not expose mutation helpers for those values.
3. Add a test-local Bevy command-error recorder so a warning is a failed
   assertion, not console noise.
4. Use Ratatui `TestBackend` buffers for exact-size screen snapshots.
5. Add an injectable terminal draw boundary or test adapter; do not implement
   production invalidation yet.
6. Add a semantic input source fixture that feeds multiple keys while retaining
   production routing.
7. Keep all expected red tests checked in and named by behavior.

### Internal validation

- Each confirmed defect fails for its observed reason.
- No failure depends on wall-clock timing.
- TUI snapshots assert final cell content and panel boundaries.
- Command-error capture fails on the observed stale-entity operation.
- Existing 14 canonical tests remain intact but no longer constitute the whole
  Foundation gate.

### GDD drift check

Map every new test to GDD Sections 2, 3, 6, and 8. Tests must not assert P2
content or deferred mechanics.

### Exit gate

- The complete stabilization queue is red.
- Every red test has an assigned later phase.
- No production behavior was changed except test seams that preserve runtime
  behavior.

### Phase 1 completion evidence — 2026-07-24

The complete named stabilization queue is checked in across
`foundation_stabilization.rs`, `bd_tui` exact-buffer/runtime-boundary tests,
and application configuration tests.

- Foundation stabilization target: 7 expected failures, covering authoritative
  construction payment, duplicate player Supplies, missing entry action,
  unpaid/replay-less entry, idle extraction/gate log churn, and missing Rest.
  Denial atomicity and focused cleanup-count regressions were already green.
- TUI library target: 9 expected failures after strengthening the 80x24
  contract, covering render invalidation/error observation, ordered/bounded
  buffered input, 80x24 status/control separation, hidden build choices,
  compact global controls, inventory return, and chronological logs.
- Binding target: shipped defaults versus built-in defaults fails exactly at
  `AssignTask` (`c` versus `a`); README versus shipped defaults is green.
- Existing canonical Foundation scenario remains unchanged.
- The test-local full-runtime plugin seam preserves normal runtime behavior
  when a Ratatui context is present and permits headless schedule integration.
- The Bevy command error did not reproduce in the headless schedule, including
  stale buffered input and post-defeat idle updates. A real isolated 80x24
  Ratatui run did reproduce the `Entity despawned` command warning on the fatal
  enemy frame. Phase 3 therefore retains both the clean headless assertions
  and the terminal warning as a mandatory red manual gate; it may not declare
  the defect absent from the headless result alone.

GDD Sections 2, 3, 6, and 8 were rechecked. The harness asserts costly
preparation, the fixed Foundation round trip, deterministic defeat, and
readable Ratatui behavior without adding deferred content or Product P2 scope.
Only read-only diagnostics, exact-buffer rendering access, and unconnected
render/input seams changed production crates.

## 10. Phase 2 — Authoritative Colony Economy and Dungeon Entry

### Purpose

Repair the most serious gameplay defect: preparation currently has visible
objects but unreliable costs.

### Dependencies

Phase 1 red economy tests exist.

### Tests first

Run and confirm the expected failures for:

- FS-002 through FS-005 tests from Phase 1;
- save/load after construction;
- save/load after paid dungeon entry;
- two valid builds consume two costs;
- a third build at insufficient resources is denied;
- a rejected build creates no station and advances no turn;
- a rejected dungeon entry creates no dungeon entities;
- loading an older development save with duplicate player Supplies rejects
  with a readable version error.

### Implementation approach

1. Introduce an explicit typed cost owner in the action model.
2. Route actor AP/Health costs to entity `Pools`.
3. Route colony resource requirements and costs to `ColonyResources`.
4. Validate and apply an accepted cost against the same owner.
5. Ensure cost application precedes effect emission and transition.
6. Remove Foundation Supplies from the player blueprint/snapshot authority.
7. Audit every registered Foundation action for cost owner.
8. Keep station costs and the direct-entry cost as named values; do not add raw
   values to resolver logic.
9. Route `UiCommand::Travel` to the canonical Foundation dungeon-entry
   `ActionIntent`.
10. Emit Tactical transition as an accepted action effect.
11. Preserve complete replay records and deterministic continuation.
12. Bump the development save version for the new ownership contract. Follow
    R-05 and reject earlier development saves with a readable incompatibility
    result; silent split ownership is not accepted.
13. Remove or deprecate unused alternate travel-cost constants/actions from
    Foundation registration without deleting future overworld infrastructure.

### Encapsulation and maintainability checks

- `bd_tui` emits a semantic action and never mutates resources.
- `bd_app` does not own economy rules.
- `bd_core` exposes no mutable colony resource field to the TUI.
- Resource-owner matching occurs in one resolver abstraction.
- Adding a future colony resource cost requires data/definition extension, not
  another TUI branch.

### Internal validation

- Initial resources are visible.
- One Stove costs exactly the declared amount.
- The same amount appears in saved authoritative colony state.
- The player does not retain a hidden duplicate Supplies pool.
- Dungeon entry costs 2 Supplies exactly once.
- Insufficient resources produce a typed, player-visible denial.
- Rejected actions do not advance time, spawn entities, or partially charge.

### Manual validation

At 80x24 in an isolated data directory:

1. launch to the shelter;
2. record visible Supplies;
3. build a Stove;
4. verify visible Supplies decrease by the declared cost;
5. save, restart, and load;
6. verify the same value remains;
7. enter the fixed dungeon;
8. verify another 2 Supplies are charged;
9. extract and verify no cost is reapplied;
10. exercise an insufficient-resource branch and verify readable denial.

### GDD drift check

Review:

- GDD “Preparation, Pressure, and Consequence”;
- GDD “Shelter and Colony”;
- GDD “Overworld Travel” only for the general cost principle;
- GDD “MVP Foundation”;
- D-01, D-09, D-10, D-12, D-15, and D-16.

Confirm that full overworld travel remains deferred.

### Exit gate

- FS-002 through FS-005 are green.
- One resource owner is proven in runtime, tests, and saves.
- Construction and entry costs are consequential and exact.
- No deferred travel system is active.

### Phase 2 completion evidence — 2026-07-24

- `ColonyResources` is the sole Foundation shelter-resource owner.
  `Effect::ColonyPoolDelta` distinguishes that typed owner from actor `Pools`;
  station validation and payment now use the same owner.
- The player content blueprint no longer contains Supplies. A version-6 manual
  save confirms the player has Health, AP, and virtue pools only while
  `colony_resources` owns Supplies.
- `ability.enter_foundation_dungeon` validates Outpost mode and 2 colony
  Supplies, charges exactly once, records replay, and emits the fixed
  `dungeon.foundation` transition. The TUI emits this semantic action and
  observes completed transitions for screen changes.
- Save version 6 rejects older split-ownership development saves with a
  readable version mismatch.
- All 12 Phase-2 stabilization assertions are green, including build and entry
  denial atomicity, two-build charging, third-build rejection, replay, and
  save/load. The 14 canonical Foundation scenario tests, 12 persistence tests,
  and 7 station tests are green.
- `cargo fmt --all -- --check`, `cargo check --workspace`,
  `cargo run -p bd_app -- --validate`, and `git diff --check` pass. Only the
  intentional later-phase runtime-seam dead-code warnings remain before the
  Phase-7 strict lint gate.
- Isolated 80x24 manual run:
  initial Supplies 10; one Stove visibly reduced Supplies to 8; F5 wrote a
  version-6 save with one Stove and authoritative Supplies 8; Travel visibly
  reduced Supplies to 6 and entered the fixed dungeon. Headless denial tests
  prove no station, turn, entity, mode, or resource mutation on insufficient
  construction or entry.

GDD Preparation/Pressure/Consequence, Shelter and Colony, Overworld Travel,
MVP Foundation, and D-01, D-09, D-10, D-12, D-15, and D-16 were rechecked.
The change preserves direct fixed-dungeon entry with a preparation cost and
does not activate Travel mode, overworld systems, procgen, raids, or events.

## 11. Phase 3 — Deterministic Defeat and Entity Cleanup

### Purpose

Make defeat a clean simulation result rather than a visually successful screen
over an ECS command error.

### Dependencies

Phase 2 economy stable; Phase 1 command-error test red.

### Tests first

Confirm red and add any required focused tests:

```text
fatal_enemy_action_emits_one_entity_defeated
player_defeat_marks_session_once
player_defeat_requests_game_over_once
player_cleanup_is_owned_once
later_queued_damage_cannot_queue_second_cleanup
fatal_status_and_cleanup_commands_do_not_target_despawned_player
game_over_save_load_contains_no_invalid_relationship
restart_after_defeat_creates_exactly_one_player
```

### Diagnostic approach

1. Enable command names or install a test command-error handler in the
   diagnostic target.
2. Identify the exact stale command before changing cleanup order.
3. Trace:
   - fatal `PoolDeltaApplied`;
   - `EntityDefeated`;
   - player defeat observation;
   - status insertion/removal;
   - transient cleanup;
   - relationship cleanup;
   - entity despawn application.
4. Record the duplicate/stale owner in phase evidence.

Do not assume the warning comes from `cleanup_defeated_entities` merely because
that system despawns the player.

### Implementation approach

1. Assign one system responsibility for physical entity cleanup.
2. Keep session outcome/Game Over observation separate from despawn ownership.
3. Make cleanup idempotent when multiple result readers observe the same
   defeat.
4. Do not queue component/status commands after cleanup has been requested.
5. Preserve the distinct Defeated outcome and zero-loot behavior.
6. Preserve save/load and restart semantics.
7. Keep enemy cleanup generic unless the diagnosed ownership requires a typed
   distinction.

### Internal validation

- One fatal action produces one defeat record.
- Game Over appears once.
- The player entity is despawned exactly once after the session records
  Game Over.
- No Bevy command warning/error is emitted.
- Saving and loading Game Over remains valid.
- Restart creates one player and no leaked dungeon transient entities.

### Manual validation

Run the normal enemy-defeat path three times from clean isolated sessions:

- allow the Rat to defeat the player;
- inspect stderr;
- save at Game Over;
- restart to title;
- load the defeated save;
- restart again;
- quit and verify terminal restoration.

Any warning fails the phase.

### GDD drift check

Review GDD Combat, Core Player Experience, MVP Foundation, D-10, D-15, and
D-16. Defeat remains costly and distinct; no new recovery or permadeath design
is introduced.

### Exit gate

- FS-006 is green in headless and terminal runs.
- Defeat, persistence, restart, and cleanup agree.
- No command error is ignored or downgraded.

### Phase 3 completion evidence — 2026-07-25

- The real-terminal warning was reproduced before implementation:
  `release_enemy_phase_lock` and `cleanup_defeated_entities` both queued
  commands in `BdSet::ResultEmission`, allowing player despawn to apply before
  removal of `AwaitingEnemyPhase`.
- Cleanup now has explicit ordering: enemy-phase status release runs before
  the single generic defeated-entity cleanup owner. Session outcome observation
  remains separate from physical entity cleanup.
- Focused acceptance coverage proves one fatal-action defeat/cleanup, one
  player-defeat session outcome, no command error, valid Game Over save/load,
  and exactly one player after restart. The pool regression suite proves later
  queued lethal damage cannot emit duplicate defeat results.
- Restart initialization now restores a missing player independently of the
  one-time outpost party bootstrap. Leaving Game Over for a non-tactical mode
  also cleans dungeon-transient entities, closing the lifecycle path that
  bypassed the normal Tactical exit.
- The 14 canonical Foundation scenario tests, 12 persistence tests, 13 pool
  tests, Phase-3 stabilization assertions, `cargo fmt --all -- --check`,
  `cargo check --workspace`, `cargo run -p bd_app -- --validate`, and
  `git diff --check` pass. Only intentional later-phase runtime-seam dead-code
  warnings remain before the Phase-7 strict lint gate.
- Three isolated terminal defeat paths completed without
  `Encountered an error`, stale-entity, or despawn warnings. The runs covered
  Game Over save creation, restart to title, loading the defeated save,
  restarting again, clean quit, and alternate-screen restoration.

GDD Combat, Core Player Experience, MVP Foundation, D-10, D-15, and D-16 were
rechecked. Defeat remains a distinct costly outcome with zero loot; the repair
adds no recovery, permadeath, content, or deferred-system design.

## 12. Phase 4 — Idle-State Discipline, Context Hints, and Rendering

### Purpose

Stop application frames from manufacturing logs and terminal output.

### Dependencies

Simulation outcomes and cleanup stable.

### Tests first

Confirm red:

- 300 idle updates on an extraction tile leave `GameLog` unchanged;
- 300 idle updates on the shelter gate leave `GameLog` unchanged;
- context action availability still exposes Extract/Travel;
- extraction remains explicit;
- the first visible frame draws once;
- unchanged idle updates do not draw again;
- a movement, log, screen, mode, inventory, build-menu, or resize change draws
  once;
- a failed terminal draw is observable, requests clean application shutdown,
  and restores terminal state;
- idle updates do not alter turn, day, replay, session, resources, or entities.

### Implementation approach

1. Remove per-frame hint writes from spatial detection.
2. Project location context through `ActionListViewModel` or a dedicated
   read-only context view model.
3. Keep extraction/travel validation in the action pipeline.
4. Add a `bd_tui`-owned invalidation resource or stable visible-frame
   fingerprint.
5. Mark dirty from view-model/screen/terminal-size changes at one boundary.
6. Draw the first frame and only subsequent dirty frames.
7. Clear dirty state only after a successful draw.
8. Handle `RatatuiContext::draw` errors explicitly.
9. Keep the 60 Hz simulation runner unless profiling proves it unnecessary;
   draw suppression, not timing reduction, satisfies this phase.
10. Add render/log counters to diagnostics without exposing them as game
    mechanics.

### Internal validation

- The log remains bounded and meaningful.
- Standing on an exit retains one stable contextual Extract instruction.
- No prompt is serialized dozens of times.
- Idle terminal output stops after the completed frame.
- Movement and screen changes remain visually immediate.
- Resize forces a correct redraw.
- Draw failures are readable and terminal teardown is safe.

### Manual validation

At both 80x24 and 60x20:

1. idle on title for five seconds;
2. idle in shelter for five seconds;
3. idle on shelter gate for five seconds;
4. idle in dungeon for five seconds;
5. idle on extraction tile for five seconds;
6. resize once;
7. extract.

Capture terminal bytes or draw counters. After each stable frame, idle output
must remain zero until a real invalidation.

### GDD drift check

Review D-02 and the GDD Ratatui/Foundation requirements. This phase changes
presentation efficiency and truthful guidance only.

### Exit gate

- FS-007 through FS-009 are green.
- Idle frames are behaviorally and visually quiet.
- Context actions remain discoverable.

### Phase 4 completion evidence — 2026-07-25

- Spatial exit detection no longer writes frame-driven hints into `GameLog`.
  Extraction remains an explicit action validated by the shared `AtExit`
  requirement.
- `ActionListViewModel` now projects extraction context read-only: Extract
  is absent when invalid and appears enabled on the active exit. Shelter
  Travel remains visible through the canonical action projection.
- `bd_tui` owns one `RenderInvalidation` resource. A stable fingerprint covers
  terminal size, screen definition, visible view models, theme/symbol state,
  bindings, mode, build interaction, turn, and day. The first frame draws;
  unchanged frames do not; visible changes and resize invalidate once.
- Failed terminal-size reads and draw attempts remain dirty, record an
  observable error, and request application shutdown. Draw attempts expose a
  trace-level counter without adding a gameplay mechanic.
- Two 300-update acceptance paths prove that idling on the dungeon extraction
  tile or shelter gate preserves logs, complete Foundation summary, entity
  count, turn/day, replay/session state, and colony Supplies.
- Focused render, context-view-model, action-panel, and explicit-extraction
  tests pass. All 194 core tests, 16 input/help tests, the 14 canonical
  Foundation scenario tests, and 12 persistence tests pass.
- Isolated 80x24 and 60x20 terminal runs remained at zero output bytes for
  five-second idles on the shelter, shelter gate, dungeon, extraction tile,
  and post-extraction shelter. Movement, travel, combat, and extraction
  redrew immediately. A live 80x24-to-60x20 resize produced one correct
  compact redraw followed by zero idle bytes. Clean quit restored the
  alternate screen in every run.
- `cargo fmt --all -- --check`, `cargo check --workspace`,
  `cargo run -p bd_app -- --validate`, and `git diff --check` pass. Only the
  intentional Phase-6 input-queue dead-code warnings remain before the
  Phase-7 strict lint gate.

D-02 and the GDD MVP Foundation shell requirements were rechecked. The phase
changes presentation efficiency and truthful contextual guidance only; it
does not alter the dungeon loop, action authority, balance, or deferred scope.

## 13. Phase 5 — Supported Terminal UX and Interaction Coherence

### Purpose

Make the existing mechanics readable and operable at the terminal sizes the
project claims to support.

### Dependencies

Invalidation and context projection stable.

### Tests first

Use exact Ratatui buffers for:

- title at 80x24 and 60x20;
- outpost at 80x24 and 60x20;
- build selection at 80x24 and 60x20;
- build placement at 80x24 and 60x20;
- combat at 80x24 and 60x20;
- inventory at 80x24 and 60x20;
- Game Over at 80x24 and 60x20;
- long save-path and denial messages;
- longest current configured key labels;
- oldest-to-newest combat log ordering.

Snapshots must assert:

- borders remain inside the buffer;
- no command token is split into misleading fragments;
- required global controls remain visible;
- contextual actions are visible in their panel;
- build menu shows five names, five costs, selected row, confirm, and cancel;
- inventory shows item state, Use, Back/Inventory toggle, Save/Load, and Quit;
- stats truncate intentionally with a visible ellipsis where needed;
- no panel writes into the footer.

### Implementation approach

1. Make `TerminalLayout` select actual screen definitions or layout policies;
   do not compute it and then render the same full screen.
2. Replace the raw wrapping footer string with descriptor-grouped lines.
3. Keep footer responsibility narrow:
   - turn/day/version status;
   - global controls;
   - mode-specific controls only when they fit as complete tokens.
4. Keep primary contextual actions in the Actions panel.
5. Render build selection as a modal or dedicated panel that owns enough rows.
6. Keep build placement instructions visible after selection.
7. Add semantic Back behavior and Inventory toggle behavior from S-08.
8. Render the visible log window oldest-to-newest.
9. Define deliberate truncation helpers for stats and paths.
10. Preserve theme, symbol, and view-model boundaries; widgets remain
    simulation-read-only.

### UX quick wins included

- Show current/required Supplies in the build menu.
- Explain insufficient build or travel resources before input.
- Show `Extract` only when valid.
- Show “No target,” “Nothing here,” and similar denials without crowding out
  enabled primary actions.
- Keep save/load success readable even when paths are long.

### Internal validation

- A first-time player can identify every required control without README access.
- Build choices are never hidden.
- Inventory has a reversible, visible navigation path.
- Combat messages read in causal order.
- Full and compact layouts are distinct and deterministic.

### Manual validation

At 80x24 and 60x20:

- inspect every screen and modal;
- build and staff a Stove;
- open and close Inventory using the visible control;
- produce a long save message;
- enter combat, attack, and verify log order;
- reach Game Over and confirm all controls fit.

### GDD drift check

Review GDD Core Player Experience, Shelter and Colony, Combat, MVP Foundation,
D-02, D-10, and D-12. No graphical redesign or new mechanic is authorized.

### Exit gate

- FS-010 through FS-013 are green.
- Exact-size snapshots and manual runs agree.
- The declared terminal profiles are truthful.

### Phase 5 completion evidence — 2026-07-25

- `TerminalLayout::Compact` now selects deliberate compact screen policies:
  secondary shelter-party and inventory-side panels are removed, while map,
  contextual actions, recent feedback, and required controls retain owned
  space. Full and compact layouts are structurally distinct.
- The footer is three non-wrapping responsibilities: status, complete
  mode-specific tokens that fit, and complete global tokens. Save, Load, and
  Quit remain visible at 60x20; primary location-dependent Extract guidance is
  owned only by the truthful Actions panel.
- Build selection is a centered modal with all five station names, costs,
  current Supplies, selected row, confirm, and cancel. Build placement has a
  persistent movement/confirm/cancel banner at both supported profiles.
- Inventory displays usable/equipped state, advertises `Back:i`, and the same
  semantic Inventory command now toggles back to the active Outpost or Combat
  screen. Save/Load/Quit remain visible.
- Stats and long feedback use deliberate ellipsis helpers. Path truncation
  preserves the useful filename, and recent logs select the newest window
  while rendering it oldest-to-newest.
- A real chronology defect was fixed at its owners: action cause effects now
  precede damage effects, `GameLog`'s newest-first storage is reversed only at
  view-model projection, and one action emits one compact combined
  skill/virtue progression result. Fatal combat visibly reads cause, damage,
  progression, then defeat.
- Exact TestBackend coverage proves title, Outpost, build selection, build
  placement, Combat, Inventory, and Game Over at 80x24 and 60x20; rows retain
  exact width, panel borders close before the footer, required tokens are not
  split, and long/current feedback remains causal. All 195 core tests,
  20 input/help tests, 11 screen-definition tests, 8 view-model tests, the
  14 canonical Foundation scenario tests, and 12 persistence tests pass.
- Isolated manual runs at both profiles inspected every required screen and
  modal, built and staffed a Stove, toggled Inventory open/back, saved through
  long isolated paths, entered and fought in Combat, and reached Game Over.
  The 60x20 run showed chronological fatal-combat feedback and readable
  middle-truncated save/load paths. All terminal exits restored the alternate
  screen without warnings.
- `cargo fmt --all -- --check`, `cargo check --workspace`,
  `cargo run -p bd_app -- --validate`, and `git diff --check` pass. The full
  TUI library has 41 green tests and only the two intentional Phase-6 buffered
  input reds. The stabilization target has 18 green tests and only the
  intentional Phase-6 unknown Rest-action red.

GDD Core Player Experience, Shelter and Colony, Combat, MVP Foundation, D-02,
D-10, and D-12 were rechecked. These changes make existing Foundation
mechanics legible and reversible without adding content, changing balance, or
activating deferred systems.

## 14. Phase 6 — Daily-Cycle Access and Buffered Input

### Purpose

Make the existing colony production cycle usable without weakening the
authoritative day boundary or silently dropping actions.

### Dependencies

Economy and terminal interaction stable.

### Tests first

```text
rest_until_next_day_is_outpost_only
rest_advances_exactly_remaining_turns
rest_emits_exactly_one_day_advanced
rest_runs_each_daily_consumer_once
rest_produces_one_daily_summary
rest_persists_day_resources_and_summary
rest_replay_is_deterministic
rest_is_denied_during_build_event_tactical_and_game_over
buffered_moves_resolve_in_input_order
buffered_waits_advance_the_expected_turn_count
buffered_actions_respect_enemy_phase_lock
buffer_capacity_is_bounded
buffer_overflow_warns_once
cancel_quit_and_screen_controls_are_not_starved
```

### Implementation approach

1. Add one semantic Rest Until Next Day command to the binding catalog.
2. Route it through one Foundation action.
3. Compute turns remaining from authoritative `GameTime`.
4. Advance time through a typed request that results in one boundary.
5. Leave all daily consumers attached to `DayAdvanced`.
6. Persist and replay the action deterministically.
7. Display the target day and turns remaining in action guidance.
8. Introduce one bounded semantic gameplay-input queue in `bd_tui`.
9. Drain at most the actions the simulation can accept in order.
10. Do not queue mode-invalid commands.
11. Emit one warning when capacity is exceeded; avoid one warning per dropped
    key.
12. Keep immediate UI/lifecycle controls outside the gameplay queue.

### Internal validation

- One Rest command produces the same final day transaction as the equivalent
  valid waits.
- Production, consumption, gathering, and mood consequences occur once.
- Save/load does not replay the daily transaction.
- Rapid input is deterministic and visible.
- The queue cannot grow without bound.
- Tactical enemy phases cannot be bypassed by buffered actions.

### Manual validation

1. Build and staff a Stove.
2. Use the visible Rest command.
3. Verify the day and exact daily resource summary.
4. Save, restart, and load.
5. Verify the transaction is not repeated.
6. Paste or rapidly enter a bounded movement/wait sequence.
7. Verify order and overflow feedback.
8. Confirm Rest is absent in the dungeon.

### GDD drift check

Review GDD Shelter and Colony, Preparation/Pressure/Consequence, MVP Foundation,
D-12, and recovery decision R-11. This phase exposes the existing daily cycle;
it does not add events, raids, crafting, or colony simulation depth.

### Exit gate

- FS-014 and FS-015 are green.
- Daily production is accessible and exact.
- Buffered input is deterministic, bounded, and honest.

### Phase 6 completion evidence — 2026-07-25

- `Rest Until Next Day` is one Outpost-only semantic command and one validated
  Foundation action. It emits a typed time request; it does not call production,
  consumption, gathering, mood, or summary systems directly.
- Rest advances from the authoritative current turn to exactly the next day at
  turn zero and emits one `DayAdvanced`. Equivalence tests prove the resulting
  daily transaction matches individual waits, including a staffed Stove.
- Rest is denied by the action layer during build and active-event interaction,
  and by mode outside Outpost. Its action guidance names the target day and
  exact turns remaining; dungeon guidance omits it.
- Save/load preserves the completed day, resources, station, assignment, and
  latest summary without replaying consumers. Two same-seed Rest runs produce
  identical summaries and replay streams.
- `bd_tui` owns one four-command semantic gameplay queue. Production-path tests
  prove input order, four-command capacity, one visible overflow warning,
  responsive quit handling, and enemy-phase lock compliance.
- A live batching defect found during validation was fixed and retained as a
  regression: `b`, `1`, `d`, `Enter` in one input batch now remains modal and
  builds exactly one selected station instead of losing placement or moving the
  player.
- Focused validation is green: 13 daily-cycle tests, 21 Foundation
  stabilization tests, 21 input/help tests, 43 TUI library tests, and 5
  production input-routing tests.
- The isolated 80x24 manual run built and staffed a Stove, rested from day 0 to
  day 1, showed one daily summary with Supplies 8, saved, restarted, and loaded
  the same day/resources without repeating the transaction. Five rapid waits
  resolved four in order with one overflow warning. Rest was absent after
  entering the fixed dungeon.
- GDD Preparation/Pressure/Consequence, Shelter and Colony, MVP Foundation,
  D-12, S-10, S-11, and recovery decision R-11 were rechecked. Raids, events,
  crafting, sanity, procgen, and deeper colony simulation remain deferred.

The workspace gate reaches only the already assigned Phase 7 binding-authority
failure (`AssignTask`: built-in `a`, shipped `c`); no Phase 6 test remains red.

## 15. Phase 7 — Binding Authority, Maintainability, and Documentation

### Purpose

Remove drift that allowed tests and documentation to describe a different
runtime, then make the strict quality gate green.

### Dependencies

Player behavior stable.

### Tests and checks first

- Built-in defaults equal parsed shipped `config/default.toml`.
- Every README control row maps to a known semantic command and current
  shipped default.
- Input, help, footer, action panels, build mode, inventory, title, and Game
  Over use the same binding owner.
- No TUI branch contains a raw gameplay key except fixed numbered-menu
  selection explicitly allowed by the recovery plan.
- Run strict Clippy and preserve its red output as baseline evidence.

### Binding implementation approach

1. Keep semantic command definitions in `bd_tui`.
2. Define built-in default bindings once.
3. Make `bd_app::Config::default` derive key defaults from that specification
   rather than restating characters.
4. Test shipped TOML against the same specification.
5. Update README from the verified shipped defaults.
6. Preserve configurable bindings and conflict validation.
7. Keep fixed `1`–`5` build-menu selection explicitly local and documented.

### Clippy and maintainability approach

1. Fix behavior-neutral lints first:
   - derived defaults;
   - `&Path` instead of `&PathBuf`;
   - simplified `Option` operations;
   - unnecessary casts;
   - test range assertions.
2. Address architectural lints in touched modules:
   - group Bevy resources/messages into nameable `SystemParam` owners;
   - introduce query type aliases where they clarify responsibility;
   - split functions only when responsibilities are independently nameable;
   - avoid catch-all utility modules.
3. Keep action validation, cost compilation, effect resolution, and mutation as
   separate stages.
4. Keep transition, extraction, and location cleanup ownership explicit.
5. Use a narrow lint allowance only when Bevy system signatures genuinely
   require it and record the reason next to the function.
6. Run all behavior tests after each refactor group.

### Documentation work

Update:

- `broken-divinity/README.md`;
- `docs/README.md`;
- `docs/MVP-SCENARIO.md`;
- `docs/DOCUMENT-INVENTORY.md`;
- `docs/MIGRATION-AND-DEPRECATION.md`;
- this plan’s phase evidence.

Do not mark Foundation accepted yet. Phase 8 owns acceptance.

### Internal validation

- There is one effective default binding specification.
- Runtime and documentation agree.
- Strict Clippy passes.
- No test behavior changed during lint-only refactors.
- No module gained a second unrelated responsibility to satisfy a lint.

### GDD drift check

Review D-02, D-07, D-09, Kernel ownership boundaries, and recovery decisions
R-01, R-02, R-12, and R-14.

### Exit gate

- FS-016 and FS-017 are green.
- Documentation is truthful but still reports acceptance pending.
- Formatting, checking, tests, validation, strict Clippy, and diff checks pass.

### Phase 7 completion evidence — 2026-07-25

- The built-in `AssignTask:a` defect was confirmed as both authority drift and
  an Outpost conflict with Move West. The semantic built-in is now `c`, matching
  shipped TOML, runtime guidance, and README.
- `KeyBindingConfig::default` no longer restates default characters. It derives
  every configurable key from `bd_tui::CommandBindings`, while parsed shipped
  TOML remains independently checked against the same semantic catalog.
- Binding parity now includes Rest. README verification checks each complete
  configurable control row plus the explicitly fixed `1`–`5` build-menu
  interaction. Title build startup consults the configured semantic binding
  rather than a raw `b`.
- Runtime input, help, footer controls, action panels, title, Game Over,
  inventory, and build mode derive from the same command descriptors and
  bindings. The only raw character branches in player routing are numbered
  build/event choices; Enter and F1 remain fixed modal/debug controls.
- Strict Clippy’s initial 33 diagnostics were recorded and resolved. Mechanical
  fixes include derived defaults, `&Path`, simplified `Option` handling,
  unnecessary-cast removal, scoped borrows, and test assertions. Bevy
  signatures retain only narrow, locally explained complexity/argument
  allowances where independent ECS owners are intentionally explicit.
- Root and repository documentation now state that repair Phases 0–7 are
  complete while Foundation acceptance remains pending Phase 8.
  `MVP-SCENARIO.md`, `DOCUMENT-INVENTORY.md`,
  `MIGRATION-AND-DEPRECATION.md`, the repository README, and repository docs
  index are synchronized.
- Complete Phase 7 validation is green: `cargo fmt --all -- --check`,
  `cargo check --workspace --all-targets`, all workspace/all-target tests,
  content validation, strict workspace/all-target Clippy with warnings denied,
  and `git diff --check`.
- D-02, D-07, D-09, Kernel input/presentation ownership, and recovery decisions
  R-01, R-02, R-12, and R-14 were rechecked. No Product P2 system or product
  design requirement was activated.

## 16. Phase 8 — Final Foundation Audit and Handoff

### Purpose

Prove the repaired Foundation as a player experience, not merely as isolated
systems.

### Dependencies

Phases 0–7 complete.

### Automated gate

Run from repository root:

```text
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo test -p bd_app --test foundation_scenario -- --test-threads=1
cargo test -p bd_app --test persistence -- --test-threads=1
cargo test -p bd_app --test colony_day_cycle -- --test-threads=1
cargo test -p bd_app --test stress -- --test-threads=1
cargo test -p bd_tui --test input_help -- --test-threads=1
cargo run -p bd_app -- --validate
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
```

Also run the new stabilization acceptance target if implemented separately.

Zero warnings/errors are required. Documented ignored real-terminal tests do
not substitute for the manual gate.

### Canonical manual scenario

Use a clean isolated configuration/data directory and 80x24 terminal:

1. Launch and enter the shelter.
2. Verify three survivors and initial colony resources.
3. Open the build menu and read all five options/costs.
4. Build one Stove and verify the exact visible resource deduction.
5. Staff the Stove.
6. Rest until the next day and verify one daily summary.
7. Save, quit, restart, and load; verify shelter continuity.
8. Enter the fixed dungeon and verify the 2-Supplies entry deduction.
9. Explore and fight the Rat.
10. Observe Melee/Thumos feedback in chronological order.
11. Pick up the healing item.
12. Complete one branch using it and observe Medicine/Temperance.
13. Complete a separate branch retaining and extracting it.
14. Verify exactly one item enters colony storage.
15. Save/reload the extracted state and verify no repeated cost, loot, skill,
    virtue, production, or summary.
16. Complete a normal defeat branch.
17. Verify zero loot, clean Game Over, no stderr warning, save/load, restart,
    and one player authority.
18. Quit and verify terminal restoration.

### Terminal quietness scenario

At 80x24 and 60x20:

- inspect title, shelter, build, inventory, combat, and Game Over;
- idle each stable screen for five seconds;
- stand on gate and extraction tiles for five seconds;
- verify no repeated log entries or terminal output;
- resize and verify exactly one correct redraw.

### Failure scenarios

- insufficient construction resources;
- insufficient dungeon-entry resources;
- blocked movement;
- attack with no target;
- pickup with nothing present;
- use with no usable item;
- missing save;
- corrupt save;
- incompatible save version;
- invalid content/configuration;
- input queue overflow;
- terminal draw failure through the diagnostic seam.

Every failure must be readable and leave authoritative state valid.

### Final GDD reconciliation

Review GDD Sections 1–10 and D-01 through D-16. Record each Foundation
requirement as:

- Proven;
- Explicitly deferred;
- Failed.

No failed item may be called complete. Deferred items remain owner-approved
exclusions and cannot be reclassified as partial implementation.

### Final documentation work

Only after every automated and manual gate passes:

1. mark this plan Complete;
2. mark `MVP-SCENARIO.md` Accepted with the new evidence date;
3. update `docs/README.md` current status;
4. append the final evidence record to this plan;
5. update the completed recovery record with a link to the stabilization
   result;
6. update README controls and current limitations;
7. keep Product P2 unauthorized until a separate owner-approved plan exists.

### Exit gate

The Foundation is accepted only when:

- all FS defects are closed with evidence;
- the expanded acceptance queue passes;
- economy costs are authoritative and exact;
- defeat is error-free;
- idle behavior and rendering are quiet;
- exact terminal profiles pass;
- daily-cycle and input UX pass;
- persistence/determinism remain correct;
- strict Clippy and all workspace gates pass;
- canonical documents report the same result.

## 17. Validation Matrix

| Defects | Requirement | Automated proof | Manual proof | Authority |
|---|---|---|---|---|
| FS-001 | Honest active status | active-document authority audit | index/plan navigation | D-07/D-08 |
| FS-002, FS-003, FS-004 | One colony resource owner | economy integration and snapshot tests | build/save/load inspection | GDD 2/6/8, D-12 |
| FS-005 | Paid dungeon entry | action/cost/denial/replay tests | visible 2-Supplies deduction | GDD 2/3, D-10/D-15 |
| FS-006 | Clean defeat | command-error and cleanup tests | three defeat/recovery runs | GDD 3/6/8 |
| FS-007 | Stable context hints | idle log tests | gate/exit idle captures | D-02/D-10 |
| FS-008, FS-009 | Quiet, fallible rendering | draw-count/invalidation/error tests | idle byte and draw-failure captures | D-02 |
| FS-010 | 80x24 and 60x20 UI | exact buffer snapshots | complete screen inspection | D-02 |
| FS-011 | Build discoverability | modal snapshots/action tests | build one Stove | GDD 6/8, D-12 |
| FS-012 | Inventory navigation | command/screen tests | toggle and Back | GDD 8 |
| FS-013 | Chronological feedback | log projection tests | combat/item feedback | GDD 3/6 |
| FS-014 | Accessible day cycle | exact boundary tests | Rest and daily summary | GDD 6/8, D-12 |
| FS-015 | Bounded input | queue order/capacity tests | rapid/pasted sequence | Kernel action discipline |
| FS-016 | Binding authority | config/runtime/docs drift tests | displayed controls work | D-02/D-07 |
| FS-017 | Maintainability | strict Clippy and architecture tests | N/A | Kernel/R-01/R-02 |
| FS-018 | Complete acceptance coverage | expanded stabilization and state-matrix suites | full clean-session scenario | D-08/D-15 |
| Regression guard | Persistence | complete state matrix | restart/load branches | D-15 |
| Scope guard | Deferred isolation | plugin/resource tests | no deferred behavior | D-04/D-06/D-15 |

## 18. Phase Evidence Template

Every phase completion record must include:

```text
Phase:
Status: Pending | In progress | Blocked | Complete
Scope:
Defect IDs addressed:
Baseline commit:
Tests added before implementation:
Expected red failure:
Implementation files:
Validation commands:
Automated results:
Manual evidence:
Unexpected results:
GDD sections reviewed:
Decision IDs reviewed:
Drift found:
Documentation updated:
Known limitations:
Exit gate result:
```

At most one phase may be In progress.

## 19. Final Definition of Done

Broken Divinity is back in a trustworthy Foundation MVP state when:

- one canonical GDD and one active execution plan exist;
- active status documents are evidence-backed;
- the expanded canonical scenario exercises real colony resource costs;
- build and dungeon-entry validation/payment share one owner;
- rejected actions do not partially mutate state;
- shelter, station, assignment, production, storage, and resources persist;
- the fixed dungeon loop and exact extraction remain complete;
- normal defeat emits no runtime warning or stale command;
- standing still cannot manufacture gameplay logs;
- unchanged UI does not redraw continuously;
- 80x24 and 60x20 screens are readable and snapshot-tested;
- build and inventory interactions are self-explanatory;
- log feedback reads in causal order;
- the daily cycle is accessible through authoritative time;
- rapid input is bounded, ordered, and honest;
- bindings/config/help/footer/actions/README agree;
- save/load and deterministic continuation remain correct;
- strict Clippy and all automated gates pass;
- the complete manual scenario passes from clean launch;
- deferred systems remain inactive;
- no completion claim relies only on the old 14-test gate.

## 20. Plan Validation Record

### Sources reviewed

- `GDD.md`;
- `Kernel.md`;
- `Kernel-direction.md`;
- `docs/DECISIONS-TO-LOCK.md`;
- `docs/MIGRATION-AND-DEPRECATION.md`;
- `docs/MVP-SCENARIO.md`;
- `docs/FOUNDATION-RECOVERY-PLAN.md`;
- `docs/DOCUMENT-INVENTORY.md`;
- discovery terminal captures and isolated saves;
- current action, spatial, pool, save, TUI, configuration, and test code.

### Logic and sequencing review

- Status is corrected before coding.
- Acceptance gaps are made red before implementation.
- Economy is fixed before UX depends on its values.
- Defeat ownership is fixed before render/log behavior can hide errors.
- Per-frame state mutation is removed before snapshot/layout work.
- Layout and navigation are fixed before adding Rest/input controls.
- Behavior stabilizes before broad lint refactoring.
- Final acceptance includes both headless and terminal evidence.

### Protocol review

- **TDD:** Every behavior phase names the red tests required before changes.
- **DRY:** Colony-cost ownership, binding defaults, context hints, and redraw
  invalidation each have one owner.
- **SRP:** Simulation, action costs, session outcomes, cleanup, input, view
  models, rendering, and application lifecycle remain separate.
- **Open/Closed:** New cost owners and semantic commands extend typed models
  instead of adding command-specific TUI mutation.
- **Data-driven:** Existing content remains authoritative; no new balance table
  is hardcoded in widgets. Named Foundation constants remain named and tested.
- **Encapsulation:** TUI and application layers cannot mutate colony resources
  directly.
- **Magic numbers:** Existing 2-Supplies entry cost and queue capacity are named
  and tested; raw values do not enter resolver logic.

### Risk review

Highest-risk work:

- changing resource ownership without corrupting save/load;
- preserving deterministic replay while making entry an action;
- diagnosing defeat before changing cleanup;
- invalidation-driven drawing around terminal resize/lifecycle;
- multi-turn Rest without double-running daily consumers;
- queued input around tactical enemy-phase locks.

Each is isolated behind a dedicated phase and exit gate.

### Validation result

The plan is implementable against the existing crate boundaries, preserves
working Foundation code, reopens only disproven gates, and does not activate
Product P2 or deferred systems.

## 21. Phase 8 Completion Evidence

```text
Phase: 8 — Final Foundation Audit and Handoff
Status: Complete
Scope: Final automated, terminal, failure-path, persistence, and design audit
Defect IDs addressed: FS-001 through FS-018 plus three final-audit regressions
Baseline commit: 1ac674f with the validated Phase 7 working tree
Tests added before implementation:
- log_message_and_level_round_trip_without_text_prefixing
- compact_60x20_snapshots_preserve_required_controls (expanded with loot state)
- title_displays_persistence_failures_at_both_supported_profiles
Expected red failure:
- save/load changed "Save requested." into "Info: Save requested."
- compact stats hid the stored-loot value at 60x20
- title-screen load failures were recorded but invisible
Implementation files:
- broken-divinity/crates/bd_core/src/save.rs
- broken-divinity/crates/bd_tui/src/screens.rs
- broken-divinity/crates/bd_tui/src/lib.rs
- canonical status/evidence documents
Exit gate result: Pass
```

### Automated results

The final post-fix run passed:

- `cargo fmt --all -- --check`;
- `cargo check --workspace`;
- `cargo test --workspace`;
- `cargo test -p bd_app --test foundation_scenario -- --test-threads=1`
  (14/14);
- `cargo test -p bd_app --test persistence -- --test-threads=1` (12/12);
- `cargo test -p bd_app --test colony_day_cycle -- --test-threads=1`
  (13/13);
- `cargo test -p bd_app --test stress -- --test-threads=1` (6/6);
- `cargo test -p bd_tui --test input_help -- --test-threads=1` (21/21);
- the separate Foundation stabilization target (21/21) and semantic input
  target (5/5) through the workspace run;
- `cargo run -p bd_app -- --validate`;
- `cargo clippy --workspace --all-targets -- -D warnings`;
- `git diff --check`.

The workspace run includes 196 `bd_core`, 44 `bd_tui`, 21 `bd_data`, 20
`bd_app`, 7 test-support, and all integration tests. Two documented legacy
terminal/diagnostic tests remain ignored; exact-buffer tests and the required
real-terminal scenarios replace neither other nor the acceptance evidence.

### Manual results

Clean isolated 80x24 branches proved:

- three survivors, 10 initial Supplies, all five station choices at 2 Supplies;
- one Stove construction (10 to 8), staffing, Rest to Day 1, and exactly one
  daily summary;
- shelter save/restart/load continuity;
- direct fixed-dungeon entry (8 to 6), exploration, Rat combat, and
  chronological Melee/Thumos feedback;
- healing-item pickup and use (12 to 20 HP) with Medicine/Temperance;
- a separate retained-item extraction branch applying one stored item exactly
  once;
- extracted-state reload without repeated cost, loot, progression,
  production, or summary;
- ordinary defeat with zero loot, clean Game Over, defeated-state save/load,
  restart, exactly one player glyph/authority, and terminal restoration;
- a fresh version-7 save/load retaining exact log text without a duplicated
  severity prefix;
- a missing-save load failure visibly reported from the title.

The 60x20 profile proved title, shelter, compact resource/run status, all build
choices, inventory return controls, combat, Game Over, and terminal
restoration. Title, shelter, build, inventory, combat, and Game Over remained
byte-quiet for five-second idle windows. Gate and extraction idling produced
no repeated gameplay logs or terminal output. A live 60x20 to 80x24 resize
produced one correct redraw followed by a quiet terminal.

Automated failure tests additionally proved readable, atomic handling for
insufficient build/entry resources, blocked movement, unavailable attack,
pickup, and item use, corrupt/incompatible saves, invalid content/config,
bounded input overflow, and the terminal draw-failure seam.

### Unexpected results and resolution

1. The snapshot stored display-formatted log strings and restored every entry
   as `Info`, duplicating prefixes and losing severity. Save version 7 now
   serializes typed log entries, preserves exact order/text/level, and rejects
   older development saves readably.
2. The compact stats pane rendered `Stored loot:` without its value. Compact
   labels now preserve loot and run outcome at the 60x20 support boundary.
3. Title-screen persistence warnings were present in state but not visible.
   The title now renders the newest warning while ignoring ordinary startup
   information.

All three were made red before implementation and were followed by the complete
acceptance matrix.

### Final GDD and decision reconciliation

| Authority | Foundation result |
|---|---|
| GDD 1 — Game statement | Proven at Foundation scale: shelter continuity and costly dungeon survival form one playable loop. Deeper sacred legitimacy remains deferred. |
| GDD 2 — Design pillars | Proven for preparation, pressure, consequence, and preservation of the theology direction. Theology-driven mechanics are explicitly deferred. |
| GDD 3 — Core experience | Proven for maintain, prepare, enter, fight, extract, return, and stabilize. Overworld pressure and power/law choices are deferred. |
| GDD 4 — World foundations | Explicitly deferred narrative/world-system content; no conflicting canon was added. |
| GDD 5 — Factions/narrative | Proven for two data-driven placeholder faction identities and hostility. Named factions, investigation, and deeper narrative are deferred. |
| GDD 6 — Gameplay structure | Proven for fixed tactical combat, physical shelter, resources, stations, survivors, production, loot, extraction, and defeat. Full overworld, raids, events, and sanity are deferred. |
| GDD 7 — Virtues/progression | Proven for extensible Melee/Ranged/Repair/Medicine state and representative Thumos/Temperance hooks; exact mappings and balance remain deferred. |
| GDD 8 — Scope anchors | Proven in full for the Foundation list; every listed exclusion remains inactive. |
| GDD 9 — Constraints | Proven: the playable loop was clarified without inventing generic theology content or replacing virtues with conventional attributes. |
| GDD 10 — Open questions | Proven only to the locked Foundation minimum; open mappings, richer factions, and post-Foundation depth remain open. |
| D-01–D-02 | Proven: Foundation boundary and Ratatui runtime. |
| D-03 | Proven: exactly two extensible data-driven placeholder factions. |
| D-04 and D-06 | Explicitly deferred; sanity and theology-driven mechanics are inactive. |
| D-05 | Proven through skill growth plus action-to-virtue expression. |
| D-07–D-09 | Proven through canonical document ownership, completed stabilization evidence, and preservation of reusable/deferred code. |
| D-10–D-12 | Proven through the complete fixed dungeon loop and basic persistent colony; procgen, raids, and events remain deferred. |
| D-13–D-14 | Proven through four practical skill lanes and representative virtue hooks. |
| D-15 | Proven through deterministic fixed content, faction identity, combat, loot, colony return, and persistence with deferred isolation. |
| D-16 | Proven through one colony resource owner, exact 2-Supplies entry, Rest to one authoritative day boundary, and accepted 80x24/60x20 profiles. |

No Foundation authority item is Failed. Deferred items remain exclusions and
were not reclassified as partial implementation.

### Final handoff

Foundation stabilization is accepted. This plan and
`FOUNDATION-RECOVERY-PLAN.md` are completed evidence records; neither
authorizes more implementation. Product P2 requires a new owner-approved plan
and explicit selection of which preserved deferred systems enter scope.

## 22. Post-acceptance correction — 2026-07-25

A later clean discovery run reopened the acceptance result above. It proved
that Tactical day boundaries skip the colony transaction and that the adverse
economy path, station truth, colony management, run-history semantics,
transition placement, controls, feedback, and dungeon depth were not covered
adequately by this plan’s gate.

The owner-authorized response is
[FOUNDATION-MVP-CORRECTION-PLAN.md](FOUNDATION-MVP-CORRECTION-PLAN.md).
This stabilization plan remains completed evidence for the defects it fixed,
but its final acceptance claim is no longer current.

## 23. Correction completion cross-reference — 2026-07-25

The later owner-authorized
[FOUNDATION-MVP-CORRECTION-PLAN.md](FOUNDATION-MVP-CORRECTION-PLAN.md)
completed its full automated and clean-terminal gate. It corrected the
cross-mode day transaction, adverse economy recovery, station truth,
management targeting, completed-run history, shelter return placement,
controls, feedback, persistence path wiring, and fixed-dungeon experience.

Foundation acceptance is current again. This file remains the chronological
stabilization record; the correction record owns the latest acceptance
evidence. Product P2 remains unauthorized.
