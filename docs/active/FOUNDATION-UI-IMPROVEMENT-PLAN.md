# Broken Divinity Foundation UI Improvement Plan

**Status:** Owner-approved; red-first UI guide suite authored, production
implementation has not started

**Created:** 2026-07-28

**Plan owner:** Project owner

**Execution target:** Coding agents working in small, independently validated
TDD batches

**Product boundary:** Ratatui presentation and interaction quality for the
existing Foundation colony, fixed dungeon, persistence, and terminal shell

**Coordination:** This plan owns UI implementation order and phase exit gates.
The authoritative testing standard owns evidence sufficiency and contract
registration. The Foundation UX hardening plan owns D-18 behavior. The basic
colony-loop plan owns D-20 production and construction behavior.

**Never authorizes:** New gameplay, Product P2 automation, production queues,
station upgrades, raids, events, sanity, overworld expansion, procedural
topology, new dungeon content, mouse support, a graphical asset pipeline, or a
replacement runtime.

---

## 1. Purpose

The Foundation mechanics are stronger than their current presentation. The
shelter is physical, survivors travel and work, construction and production
have turn-based progress, management is paused, the fixed dungeon loop is
playable, and save/load is deterministic. The UI still asks the player to
reconstruct too much from terse panels, transient messages, and raw symbols.

The finished Foundation interface must let a player answer:

1. Where am I?
2. What mode or workflow am I in?
3. What is selected?
4. What can I do now?
5. Why is an action unavailable?
6. What is each survivor doing?
7. Where is each assigned survivor going?
8. What is being gathered, carried, constructed, or refined?
9. What changed after the last action?
10. What must I do next?

The plan separates semantic correctness, canvas contents, resolved styles,
geometry, transitions, and real-terminal behavior. A substring in an
off-screen buffer is not proof of a usable interface.

---

## 2. Authority and Required Reading

Before changing a file, read:

1. [README.md](README.md);
2. root [GDD.md](../GDD.md), especially the player loop, Shelter and Colony,
   Minimum Colony Foundation, and Minimum Dungeon Foundation;
3. [DECISIONS-TO-LOCK.md](DECISIONS-TO-LOCK.md), especially D-09 through
   D-12 and D-16 through D-20;
4. root [Kernel.md](../Kernel.md), especially semantic ASCII, view-model
   boundaries, input ownership, schedules, content, and persistence;
5. [MVP-SCENARIO.md](MVP-SCENARIO.md);
6. [AUTHORITATIVE-TESTING-STANDARD-AND-MIGRATION-PLAN.md](AUTHORITATIVE-TESTING-STANDARD-AND-MIGRATION-PLAN.md);
7. [FOUNDATION-TEST-AND-UX-HARDENING-PLAN.md](FOUNDATION-TEST-AND-UX-HARDENING-PLAN.md);
8. [FOUNDATION-BASIC-COLONY-LOOP-PLAN.md](FOUNDATION-BASIC-COLONY-LOOP-PLAN.md);
9. [MIGRATION-AND-DEPRECATION.md](MIGRATION-AND-DEPRECATION.md);
10. the contract registry, requirement map, evidence report, visual matrix,
    current implementation, and affected tests.

| Source | Owns |
|---|---|
| GDD | Product intent and player experience |
| Locked decisions | Approved Foundation behavior and scope |
| Kernel | Architecture and ownership boundaries |
| Testing standard | Test design and evidence sufficiency |
| D-18 hardening plan | Shelter UX and worker behavior |
| D-20 colony plan | Physical production and construction |
| This plan | UI sequence and phase gates |
| Contract registry | Machine-readable ownership, never product truth |
| Tests | Evidence only |

If a UI task appears to require gameplay changes, stop and check the higher
authority. Do not turn a presentation preference into a simulation rule.

---

## 3. Confirmed Starting Point

### Reusable implementation

- Ratatui rendering in `bd_tui`;
- data-defined screen and panel structures;
- semantic visual tokens and centralized styles;
- 80x24 and 60x20 profiles;
- player-following shelter viewport;
- paused build and management interactions;
- typed survivor activities and assignments;
- data-driven station, resource, source, recipe, theme, and symbol content;
- structured daily summaries, contextual actions, and a message log;
- deterministic fixtures and production-path input support;
- a contract registry and layered visual-evidence model.

Extend these systems. Do not create a parallel renderer, duplicate
interaction state, second theme system, or UI-only simulation model.

### Confirmed UI weaknesses

- shelter information hierarchy is unsettled;
- worker activity lacks complete direct visual evidence;
- off-screen targets lack complete name and distance context;
- selection and placement need stronger style and transition evidence;
- management needs complete stage, result, style, and PTY evidence;
- compact decisive-text coverage is incomplete;
- map layers may hide player or survivor presentation;
- modal close and resize need stale-cell proofs;
- zero-resource, day-summary, persistence, and dungeon scenes lack complete
  player-facing evidence;
- dungeon movement, attack denial, pickup, extraction, and outcomes need
  stronger rendered transitions;
- alternate-screen and cursor restoration remain incomplete;
- most visual scenes are `GreenUnreviewed`, not `Accepted`.

### Protected baseline

At plan creation, validated Foundation changes remain uncommitted. The latest
reported gate is 619 listed, 619 passed, 0 failed, 0 ignored, with 73 required
contracts all `GreenUnreviewed`. Phase UI0 must reproduce this result. Never
reset, clean, stash, or overwrite the existing work.

The 2026-07-28 UI test-authoring pass reproduced that baseline before adding
17 tests and 13 contracts. The current intentional development baseline is
636 listed, 625 passed, 11 failed, and 0 ignored, with 86 required contracts:
75 `GreenUnreviewed` and 11 `Red`. Exact failures are recorded in
`testing/FOUNDATION-TEST-EVIDENCE.md`. This is an authorized red TDD state, not
a regression in the preceding 619 tests.

---

## 4. Locked UI Outcomes

### UI-01 — Explicit interaction state

- The active mode or workflow is always identifiable.
- World, build selection, placement, task management, station staffing, help,
  and terminal outcomes remain distinct.
- Paused workflows remain paused.
- Footer controls derive from the active state and agree with modal controls.

### UI-02 — Readable colony state

- The map remains the primary physical shelter representation.
- Survivor rows expose name, activity, target, and relevant progress.
- Stockpiles, cargo, construction, and production are distinct.
- Blocked work includes a reason.
- Off-screen assignments expose direction, target, and distance.
- Next worker completion and next-day effects are distinguishable.

### UI-03 — Accessible visual semantics

- Important state differences do not rely on color alone.
- Selection, disabled, danger, valid, and invalid states are unambiguous.
- Invalid placement remains obvious in reduced-color terminals.
- Player and survivors cannot disappear through layer order.
- The active legend explains visible semantic categories.

### UI-04 — Understandable results

- Each accepted command produces at most one primary result.
- Failure feedback explains the blocker.
- Recent results and logs format the same structured facts.
- No success result appears for an atomic rejection.

### UI-05 — Supported terminal profiles

- 80x24 is baseline; 60x20 is fully supported.
- Required text does not end mid-word.
- Selection, cost, availability, target, controls, result, and decisive deltas
  remain accessible.
- Resize redraws authoritative state without stale cells.

### UI-06 — Legible dungeon loop

- Movement, blocked movement, attack, denial, enemy response, pickup,
  extraction, and defeat have distinct rendered outcomes.
- Health, loot, extraction readiness, and contextual actions do not require
  mining the historical log.
- Terminal outcomes expose a valid next step.

---

## 5. Architecture Rules

### Ownership

- `bd_core` owns facts and simulation transitions.
- app orchestration owns interaction state and accepted input.
- `bd_tui::view_models` owns display-ready projections.
- `bd_tui::screens` owns composition and layout.
- visual and theme registries own semantic styles and symbols.
- test support owns fixtures and observations, not expected gameplay logic.

### DRY and SRP

- One formatter owns each repeated activity, progress, cost, denial, delta,
  and distance presentation.
- One footer projection owns active controls.
- One view model owns each panel projection.
- One visual registry owns symbols and styles.
- Each new type has one responsibility: observe, lay out, project, render,
  interact, or format.

### Open/Closed and data-driven behavior

- New content flows through catalogs without renderer branches per content ID.
- UI switches on semantic categories, not each station or resource.
- Named layout profiles replace scattered dimensions.
- Existing content-driven theme and symbols remain authoritative.

### Encapsulation

- Do not expose ECS entity IDs in UI evidence.
- Do not add public production APIs solely for tests.
- Use stable IDs, names, coordinates, and normalized projections.
- Keep visual observations in test support, not the gameplay API.

---

## 6. Mandatory TDD Protocol

For every numbered task:

1. identify the authority and existing contract owner;
2. classify the test as primary, supporting, or genuinely new;
3. write the smallest deterministic failing test;
4. run it and record the expected failure;
5. if it passes, determine whether behavior exists or the test is weak;
6. implement the smallest production change;
7. rerun the focused test and neighboring crate tests;
8. run the affected player workflow;
9. run signed candidate mode when implementing a delegated red handoff, or the
   argument-free canonical gate when performing independent review;
10. compare to the GDD and this plan;
11. update registry and evidence only where status actually changed.

A useful UI failure reports contract ID, fixture, profile, interaction state,
input, expected and actual semantics, relevant rectangles, style or symbol
differences, a compact canvas excerpt, and authoritative state context.

Whole-buffer substrings, catalog checks, widget-only tests, test count, and
synthetic buffers cannot independently close player-facing contracts.

---

## 7. Phase UI0 — Baseline and Visual Observation

### UI0.1 — Reproduce baseline

1. Record `git status --short`.
2. Identify ownership of current modifications.
3. Run the canonical gate and record all totals.
4. Compare to the protected baseline.
5. Stop on unexplained differences.
6. Obtain separate authorization before checkpointing or committing.

### UI0.2 — Map contract ownership

Inventory all relevant `VISUAL-*`, `INPUT-*`, `SHELL-*`, `COLONY-*`,
`DUNGEON-*`, and `PERSIST-*` contracts. Map every task in this plan to one
owner. Duplicate primary ownership is a phase failure.

### UI0.3 — Canonical fixtures

Create deterministic fixtures for:

- title with and without a save;
- clean shelter;
- gathering, carrying, refining, construction, blocked work, and off-screen
  assignment;
- zero Supplies and day summary;
- build catalog, valid placement, occupied placement, and egress rejection;
- each task/staffing stage;
- dungeon exploration, combat, loot, extraction, defeat;
- save and load outcomes.

Use stable names, content IDs, seed, coordinates, and query-order-independent
setup.

### UI0.4 — Four-layer observations

Provide discrete semantic, canvas, resolved-style, and geometry observations.
Prove the harness detects foreground-only, modifier-only, glyph-only,
one-cell geometry, overlap, clipping, stale-cell, and nondeterministic changes.

### UI0.5 — Resize and lifecycle support

Support 80x24, 60x20, round-trip resize, modal close between renders, stale
cell inspection, and PTY startup/shutdown capture.

### Exit gate

- baseline reproduced;
- every scene has a stable fixture;
- observation layers fail independently with useful diagnostics;
- no product behavior changed;
- canonical gate remains green except intentionally registered next-phase red
  tests;
- GDD and D-18/D-20 drift review recorded.

---

## 8. Phase UI1 — Semantic Visual Language

### UI1.1 — State catalog

Audit tokens for player; idle, traveling, working, blocked, resting,
constructing, and carrying survivors; fixtures; construction sites; staffed
and unstaffed stations; enemy; loot; exit; selection; valid/invalid placement;
disabled; warning; success; and information.

Tests first:

- every state resolves through the registry;
- simultaneous categories are unambiguous;
- layer order cannot hide player or survivor;
- unknown content has a safe visible fallback.

### UI1.2 — Accessible differences

Require at least two applicable channels among glyph, text, color, modifier,
and border for selected/unselected, valid/invalid, enabled/disabled,
idle/working, working/blocked, site/station, staffed/unstaffed, loot/fixture,
and player/survivor.

### UI1.3 — Layer priority

Define one priority for terrain, fixtures, stations, workers, player,
previews, indicators, and overlays. Test exact-one-player rendering, survivor
visibility, preview visibility, and indicator/modal non-overlap.

### UI1.4 — Contextual legend

Explain only visible or immediately relevant categories and controls. Cover
shelter worker states, placement semantics, dungeon enemy/loot/exit, and both
profiles.

### Exit gate

- critical pairs remain distinguishable without color;
- visual registry validation passes;
- player and survivors remain visible;
- legend agrees with symbols and active state;
- no content-ID presentation branches;
- focused, neighboring, workflow, and canonical gates pass.

---

## 9. Phase UI2 — Colony Information Hierarchy

### UI2.1 — Named layouts

At 80x24, prioritize map, workers, stockpile/production, contextual actions,
recent result, then footer. At 60x20, prioritize map, selected detail,
essential stockpile, contextual action, and recent result. Use named profile
values.

Tests first: exact rectangles, no overlap, no zero-sized required panel,
minimum map viewport, and text contained by its owner.

### UI2.2 — Colony overview projection

Project day/time, next boundary, stockpiles, active construction, active
production, blocked-worker count, and recent result from authoritative state.
The renderer must not independently reconstruct totals.

### UI2.3 — Worker rows

Each row exposes applicable name, activity, task/recipe, target, distance,
cargo, work progress, and blocked reason.

Example information shape:

```text
Mara  TRAVEL  Trees · 6 away
Iven  WORK    Raw Water · 2/3
Tala  BUILD   Workbench · 3/4
```

Test every activity and transitions through assignment, travel, arrival,
gathering, carrying, refining, construction, blocking, and completion. Assert
fields and completeness rather than incidental punctuation.

### UI2.4 — Separate economy concepts

Present colony stockpiles, survivor cargo, construction, recipe input/output,
next worker completion, and next-day transaction separately.

Tests cover zero resources, raw and refined resources, cargo not counted as
stockpile, no partial output, and worker/day distinction.

### UI2.5 — Off-screen targets

Expose direction, target name, distance, and associated survivor where space
allows. Define deterministic aggregation for shared edges. Test every
direction, on-screen transition, player movement, multiple targets, compact
mode, and no unassigned noise.

### UI2.6 — Recent result and summary

Use existing structured facts. Show one concise recent result and structured
daily collected, consumed, produced, constructed, blocked, and shortage
categories where present. Keep detail in history without duplication.

### Exit gate

- every survivor's activity and target are readable;
- stockpile, cargo, construction, and production are distinct;
- off-screen assignments are discoverable;
- zero-resource recovery remains readable;
- both profiles preserve decisive information;
- full shelter workflow and canonical gate pass;
- GDD drift review passes.

---

## 10. Phase UI3 — Build Workflow

### UI3.1 — Catalog

Selected detail exposes name, cost, sufficiency, effect, work requirement,
availability, denial reason, and controls. Test selection transitions,
unaffordable entries, complete detail, compact wrapping, and footer agreement.

### UI3.2 — Placement

Expose independent cursor/ghost, station identity, valid/invalid status,
denial reason, and contextual controls.

Test entry, cumulative movement, valid tile, occupied tile, unwalkable tile,
egress rejection, press/repeat/release, atomic cancel, and exactly-one payment
and site on success.

### UI3.3 — Results

Success reports station, location context, work requirement, and construction
status. Rejection reports why and remains correctable where the authoritative
workflow requires it.

### UI3.4 — Visual acceptance

Validate selected row, valid and invalid preview, each map edge, close/reopen,
close after resize, and no stale border, text, or preview.

### Exit gate

- selection, inspection, movement, confirm, and cancel are paused;
- invalid placement is obvious without color;
- rejection is atomic;
- D-20 construction behavior is unchanged;
- transition, style, geometry, PTY, workflow, canonical, and drift gates pass.

---

## 11. Phase UI4 — Task and Station Management

### UI4.1 — Task stages

Use:

```text
1 Survivor  >  2 Task  >  3 Confirm
```

Expose current stage, prior selections, active selection, denial reason, and
navigation/back/cancel controls.

### UI4.2 — Station stages

Use:

```text
1 Survivor  >  2 Station  >  3 Recipe  >  4 Confirm
```

Skip recipe selection only for exactly one compatible recipe while preserving
explicit review. Review survivor, station, recipe, source/input, output,
gather turns, refine turns, and blocker.

### UI4.3 — Distinct intentions

Direct gathering, station production, rest, implemented defense, and automatic
idle construction must not appear interchangeable.

### UI4.4 — Workflow tests

Test open, next, previous, confirm stage, back, cancel, unavailable confirm,
direct gathering, station assignment, reassignment, press/repeat/release,
paused time, zero worker tick, zero resource mutation, and no stale cells.

### UI4.5 — Assignment result

Report survivor, assignment, target, and resulting authoritative activity once.
Never report Working when the worker is EnRoute.

### Exit gate

- subject and target are explicit before mutation;
- navigation and confirmation remain paused;
- cancellation is atomic;
- gathering and production remain distinct;
- feedback agrees with worker activity;
- both profiles and all visual/PTY/canonical/drift gates pass.

---

## 12. Phase UI5 — Feedback and Logs

### UI5.1 — Structured results

Project facts for assignment, rejection, blocking, gathering, carrying,
delivery, construction, refining, build rejection, dungeon entry denial, loot,
extraction, save, and load. Format each fact at one presentation boundary.

### UI5.2 — Priority

Use decisive recent result, warning/required response, and historical routine
log levels. Routine movement must not immediately bury failure or completion.

### UI5.3 — Result contracts

Test exact-one-primary-result, no false success, exact authoritative delta,
accurate resulting activity, no render duplication, controlled blocked-work
noise, no daily double-count, and complete compact summary.

### UI5.4 — Edge states

Cover empty/full log, long data-defined names, zero delta, simultaneous
completions, blocked plus completed work, and recoverable load failure.

### Exit gate

- every meaningful command has accurate feedback;
- one event has one player-facing meaning;
- no duplicate production or gathering result;
- logs remain readable;
- both profiles, PTY, canonical, and drift gates pass.

---

## 13. Phase UI6 — Dungeon Readability

### UI6.1 — Status hierarchy

Expose health, relevant action/turn state, carried loot, extraction readiness,
target context, and recent result without requiring log archaeology.

### UI6.2 — Map semantics

Validate distinct player, enemy, loot, exit, terrain, blocked movement, and
target presentation.

### UI6.3 — Action availability

Explain attack without a target, pickup without loot, extraction away from the
exit, blocked movement, and entry without Supplies. Footer controls must route
in the active state.

### UI6.4 — Transition tests

Test legal and blocked movement, enemy response, legal and invalid attack,
damage, enemy defeat, loot state, pickup, exit, extraction exactly once,
defeat return, and terminal next action. Compare before/after semantics,
canvas, and authoritative state.

### UI6.5 — Complete path

Run enter, explore, fight, survive, collect, exit, extract, and apply results
exactly once at both profiles.

### Exit gate

- loop is understandable without debug state;
- combat, pickup, and extraction are decisive;
- denials explain why;
- no mechanics or content were added;
- visual, workflow, PTY, canonical, and drift gates pass.

---

## 14. Phase UI7 — Shell, Persistence, Resize, and Lifecycle

### UI7.1 — Title

Cover New Game, Load with no save, Load with valid save, corrupt/incompatible
save, and quit. Disabled load includes a reason and cannot activate.

### UI7.2 — Save/load results

Show success and actionable failure. Loading preserves the current valid state
on failure. On success, projection agrees on resources, assignments,
activities, cargo, work progress, construction, production, viewport, mode,
and durable summaries.

### UI7.3 — Resize

Test both directions, repeated resize, and resize in help, catalog, placement,
task management, staffing, dungeon, and terminal outcomes. No stale cells or
overlap may remain.

### UI7.4 — Terminal lifecycle

PTY evidence covers alternate-screen entry, cursor hiding, and restoration on
title, shelter, dungeon, outcome, and handled-error exits. The splash and
launch path must not leave malformed output.

### Exit gate

- title choices are safe;
- persistence outcomes are visible and correct;
- resize is clean;
- every exit restores the terminal;
- both profiles, PTY, canonical, and drift gates pass.

---

## 15. Phase UI8 — Full Acceptance

### Colony journey

At both profiles: start; inspect; assign direct gathering; observe travel and
three work turns; observe output; place a site; observe automatic construction;
assign station recipe; observe gather/carry/refine; inspect summary; save;
load; confirm durable state.

### Dungeon journey

At both profiles: enter; explore; fight; collect loot; reach exit; extract;
confirm colony application exactly once.

### Recovery journey

Prove zero-Supplies recovery, correctable invalid placement, atomic management
cancel, blocked reason, recoverable persistence failure, and resize without
losing workflow state.

### Visual review

For every canonical scene, inspect semantics, complete canvas, resolved style,
geometry, transition, and required PTY evidence. Record reviewer, date,
profile, result, and artifact. Similar scenes cannot share acceptance.

### Final gate

Run focused tests, affected crates, Foundation workflows,
`bash scripts/test-gate.sh`, formatting, strict Clippy, workspace check,
whitespace check, registry validation, and generated metrics.

### Exit gate

- zero required failures or ignores;
- no unexplained warning or visual diff;
- evidence layers are complete;
- `GreenUnreviewed` and `Accepted` are reported honestly;
- complete colony and dungeon journeys pass at both profiles;
- lifecycle is accepted;
- GDD, D-18, D-19, and D-20 drift review passes.

---

## 16. Small-Agent Execution Batches

| Batch | Scope | Expected production surface |
|---|---|---|
| UI0-A | Baseline and contract inventory | Evidence only |
| UI0-B | Canonical fixtures | `bd_test_support` |
| UI0-C | Semantic/canvas/style observation | Test support |
| UI0-D | Geometry/resize/stale-cell observation | Test support |
| UI1-A | Semantic token gaps | Visual/theme registry |
| UI1-B | Accessibility and layer priority | Visual registry/map |
| UI1-C | Contextual legend | View model/screen |
| UI2-A | Named layouts | Screen/layout definitions |
| UI2-B | Colony overview | View models |
| UI2-C | Worker rows | View models/panel |
| UI2-D | Economy separation | View models/panels |
| UI2-E | Off-screen indicators | Map projection/renderer |
| UI2-F | Results and summary | Result projection/panels |
| UI3-A | Build catalog | Build view model/modal |
| UI3-B | Placement | Placement projection/map |
| UI3-C | Build transitions | Orchestration/tests |
| UI4-A | Task stages | Interaction projection/modal |
| UI4-B | Staffing stages | Interaction projection/modal |
| UI4-C | Assignment result | Result projection |
| UI5-A | Structured results | Event presentation boundary |
| UI5-B | Result priority/log | Result/log view models |
| UI5-C | Edge diagnostics | Tests/minimal presentation |
| UI6-A | Dungeon status | Dungeon view model/screen |
| UI6-B | Dungeon denials | Action projection |
| UI6-C | Dungeon transitions | Renderer/workflows |
| UI7-A | Title/persistence | Shell view models |
| UI7-B | Resize redraw | Composition/shell |
| UI7-C | Terminal lifecycle | App terminal boundary |
| UI8-A | Colony acceptance | Workflows/evidence |
| UI8-B | Dungeon acceptance | Workflows/evidence |
| UI8-C | Visual review/final audit | Evidence documents |

---

## 17. Phase UI9 — Provisions and Contextual Colony Interaction

### 17.1 Owner-approved outcome

This phase deepens the existing Foundation colony presentation without adding
new production simulation. It owns three player-visible outcomes:

1. Supplies is no longer a flat counter. It presents exact stock, a responsive
   shared gauge, next-day delta/result, and a text condition at 80x24 and
   60x20.
2. Entering cardinal interaction range of a station or resource node produces
   one deduplicated `NEARBY` Chronicle fact and exposes Interact without
   requiring glyph memorization.
3. One reusable context presentation serves stations, resource nodes, and
   colonists. It identifies the exact target, exposes only applicable actions,
   and shows `Set Production` as unavailable/coming later for this UI-only
   slice.

This phase does not authorize queues, priorities, automation, station
upgrades, depletion balance, new recipes, or a second production model.
Existing direct gathering, staffing, recipe work, construction, and paused
management remain authoritative.

### 17.2 Required authority lock before input implementation

The current D-20/THC-01 contract assigns `e` directly to station staffing.
The proposed long-term interaction language makes Interact the primary entry
point and routes station staffing through a station context action. The
implementation agent must not silently rewrite that locked control.

Before changing the physical binding, obtain and record one of these owner
decisions:

- approve `e` as contextual Interact and amend the D-20/THC-01 control wording,
  while retaining `c` as the direct-task shortcut; or
- preserve `e` staffing and bind Interact to a different configured command.

The red presentation contracts intentionally require a semantic Interact
control but do not own the physical key. Help, footer, action projection, and
input routing must derive from the eventual configured binding.

### 17.3 Shared presentation model

Project display-ready data once. Rendering must not inspect ECS components,
parse forecast prose, infer identity from a glyph, or switch on individual
content IDs.

The reusable resource projection carries:

- semantic resource identity and short/long label;
- exact current and maximum values;
- optional next-boundary delta and resulting value;
- semantic condition such as Stable, Low, or Critical;
- tone/token only, never a renderer-owned raw color.

The reusable interaction projection carries:

- stable target selector suitable for duplicate station types;
- semantic category and visual token;
- display name, position, distance, and concise status;
- structured detail rows supplied by the owning domain/catalog projection;
- ordered context actions with semantic action ID, label, binding hint,
  applicability, reachability, enabled state, and denial reason.

These action states are intentionally distinct. `applicable` means the target
and domain state support the action. `reachable` means the current interaction
state has an owner-approved binding and reducer route. `enabled` means invoking
that route now can produce the claimed result. A label, a guessed key, or a key
owned by a normal-world command proves none of those facts.

Until UI9-D receives the Section 17.2 owner lock and supplies the Context input
reducer, UI9-C action rows are previews only. They remain visibly disabled with
a truthful reason such as `Open Interact menu` or `Interaction binding pending`.
`Set Production` keeps the stronger `Coming later` reason. The presentation
must never show `Enter`, `a`, `p`, or any other invented hint as an enabled
Context control merely because that glyph looks plausible.

Representative adapters populate the same projection:

| Target | Required passive detail | Initial context actions |
|---|---|---|
| Station | name, operational/construction state, staffing, active recipe/progress | Inspect; Assign Worker when valid; Set Production disabled as `Coming later` |
| Resource node | human source label, output, renewable/depleted state, assigned worker/progress | Inspect; existing gather-assignment entry where valid |
| Colonist | name, activity, target, progress, cargo/blocker when applicable | Inspect; existing task-assignment entry where valid |
| Construction site | station label, completed/required work, current worker | Inspect progress |

Unknown future categories retain a visible fallback containing identity and
Inspect; they never panic or silently disappear.

### 17.4 B3 Provisions composition

Use the existing shared panel, meter, chip, ribbon, and theme primitives. Do
not create a Supplies-only gauge implementation.

Baseline information shape:

```text
┌ ◆ PROVISIONS ───────────────────────────────────┐
│ SUP 10 [#---------]  DAWN -3 → 07  [LOW]       │
│ MAT 02   PLT 01   FTH 00              DAY 02   │
└─────────────────────────────────────────────────┘
```

At 80x24, the resource summary may use a horizontal ribbon or a named panel
region, but the map remains the largest interactive area. At 60x20, preserve
the exact Supplies value, a recognizable partial track, the next-day delta and
result, and the text condition; secondary resources may collapse to metric
chips. Color supports the meaning but never owns it alone.

Thresholds and forecasts belong to authoritative projection data. The
renderer receives condition/delta/result and only formats them. A hardcoded
`100`, parsing `next_day_forecast`, or calculating colony pressure in the
screen renderer is not an acceptable green.

### 17.5 Proximity and Chronicle rules

- Interaction range is cardinal adjacency for this Foundation slice.
- Evaluate the change after an accepted player movement, not during rendering.
- Emit one structured `NEARBY` fact only for targets newly entering range.
- Remaining adjacent, waiting, rendering, resizing, Help, save, and load emit
  no duplicate nearby fact.
- Leaving range clears eligibility silently; later re-entry may notify again.
- If several targets enter together, use one deterministic focused fact plus a
  count, while Interact exposes the complete deterministic target list.
- Target ordering uses stable category/content identity and position, never
  ECS query order or raw entity bits.
- The Chronicle fact names the target and makes Interact discoverable.

The passive Context card may refresh from current authoritative proximity
without writing history. The Chronicle is historical edge-triggered feedback;
the tooltip is current state. Do not make the renderer append to `GameLog`.

The multi-target acceptance matrix includes unlike categories and two targets
with the same display name. One accepted move must produce one focused
Chronicle fact with a semantic count, while the current projection retains
every target in deterministic order. Duplicate names must remain
distinguishable by stable player-facing selector data such as category and
position/distance; numeric ECS identity is never displayed or used for order.

### 17.6 Context interaction state

Conceptual flow:

```text
World
  └─ Interact
       ├─ zero targets  -> one truthful denial, remain in World
       ├─ one target    -> Context Menu(target)
       └─ many targets  -> Target Picker -> Context Menu(target)
```

The context interaction is paused. Opening, navigating, inspecting,
cancelling, and activating a UI-only placeholder advance no turn, move no
worker, mutate no resource/task/station relationship, and emit no gameplay
success.

The final composed screen, not an isolated widget, must show:

- one identifiable context target;
- category/status and applicable detail;
- active selection;
- enabled/disabled actions with reasons;
- confirm/cancel controls agreeing with the footer;
- no normal-world action advertised as immediately active while the context
  menu owns input.

For UI9-C, before the Context reducer exists, the same composition must also
show that preview actions are unavailable rather than pretending that their
key hints work. After UI9-D, every enabled row must agree with configuration,
Help/footer text, the active input state, and a production workflow that
actually reaches the named target/action. The focused target owns the visible
detail and action set; actions from other nearby targets may not be flattened
into it.

`Set Production` is present for an operational station but disabled with a
clear `Coming later` reason. It must not emit an intent or success result. A
later owner-approved phase may route it to the existing recipe model; it may
not introduce a parallel queue or production formula.

### 17.7 Small-agent implementation batches

Execute exactly one red contract at a time. Do not edit or weaken later tests
to make an earlier batch green.

#### UI9-A — Provisions projection and shared gauge

1. Run the focused `VISUAL-ECON-002` test and retain its red diagnostic.
2. Replace the flat Supplies projection with structured display-ready facts.
3. Render through shared B3 meter/chip/panel primitives at both profiles.
4. Preserve HP/AP tracks, map primacy, exact day, other resource values,
   current worker/day separation, and existing theme ownership.
5. Run the focused test, `bd_tui --lib`, the final-buffer profile pair, and the
   signed candidate gate. Report CandidateGreen only until review.

#### UI9-B — Proximity projection and Chronicle edge

1. Run `COLONY-PROXIMITY-001` and retain the station/node failures.
2. Build one read-only nearby-target resolver shared by Chronicle feedback,
   passive context, and Interact availability.
3. Diff stable before/after nearby sets only after accepted movement.
4. Format one structured fact at the existing feedback boundary. Exercise a
   simultaneous station/node entry and require one deterministic focused fact
   plus a count while retaining both targets in the current projection.
5. Preserve movement cost/time, worker scheduling, resources, direct
   gathering, and render purity.
6. Rerun the focused workflow, neighboring input/log tests, and canonical
   gate.

#### UI9-C — Reusable passive context presentation

1. Run `VISUAL-CONTEXT-001` for station, node, and colonist at both profiles.
2. Populate one generic target/detail/action projection through category
   adapters. Keep applicability, input reachability, and executable state
   separate; before UI9-D, all preview actions are disabled with truthful
   reasons rather than invented controls.
3. Compose the Context card/menu through shared panels and selection/status
   primitives; do not add three separate renderers.
4. Cover operational/construction and staffing detail for stations, output and
   renewable/depleted detail for nodes, and activity/target/progress/cargo or
   blocker detail when applicable for colonists.
5. Add a duplicate-display-name multi-target case proving stable
   disambiguation and one focused action set rather than flattened actions.
6. Keep the map primary and prevent compact clipping or stale cells.
7. Rerun final buffers, action-binding cross-checks, observer mutation probes,
   and profile PTY captures.

#### UI9-B/C smaller-agent v2 review — withdrawn 2026-08-03

The next implementation agent begins at the corrective UI9-C batch and uses the repository-level
`$authoritative-test-pipeline` loop. Tests, fixtures, observers, registry
status, and evidence are read-only implementation constraints unless an
independently reproduced observer defect is reported and test repair is
separately authorized.

The v2 candidate made useful production progress and passed its signed gate,
but independent review found a mixed-source false green: display prose was
parsed to remove semantic segments while the Context title independently
derived staffing from a poisoned parallel worker field. The v2 prompt,
manifest, baseline, and digest are historical evidence only and must not be
reused. A later implementation handoff requires a newly validated red set and
new v3 baseline/manifest/digest. It must protect this plan, every listed UI9-C
primary/supporting test and shared observer/fixture, the sealed dirty-worktree
baseline, and the mandatory authority/status files enforced by the repository
gate. Its whole-batch closing command will be:

```text
bash scripts/test-gate.sh \
  --candidate-manifest <reviewer-supplied-path> \
  --manifest-sha256 <reviewer-supplied-digest>
```

Only `STATUS=CandidateGreen` is a valid implementation handoff. The reviewer
then audits semantics and DRY ownership, updates every status ledger together,
and runs the argument-free canonical gate.

UI9-B is now a green preservation boundary. Run it after each proximity-owner
change in this order, one exact test at a time:

1. `cargo test -p bd_app --test phase6_input entering_adjacent_range_emits_one_deduplicated_nearby_hint -- --exact --nocapture`
   - current green: station entry emits once and unbound Interact is disabled
     with a truthful reason;
2. `cargo test -p bd_app --test phase6_input entering_water_node_range_emits_one_deduplicated_nearby_hint -- --exact --nocapture`
   - current green: the node follows the same projection and action truth;
3. `cargo test -p bd_app --test phase6_input simultaneous_station_and_node_entry_emits_one_focused_nearby_fact_with_count -- --exact --nocapture`
   - current green: one accepted move emits one deterministic focused fact plus
     a semantic count while retaining the complete target set;
4. `cargo test -p bd_app --test phase6_input leaving_range_is_silent_and_reentry_emits_exactly_once_again -- --exact --nocapture`
   - current green preservation guard: leaving is silent and later re-entry
     emits once again.

All four UI9-B rows must remain green together. A regression is
`STATUS=NotComplete`, not part of the authorized UI9-C implementation.

Then run UI9-C in bounded groups:

Use `cargo test -p bd_tui --lib ui_development_contract_tests::<test-name> -- --exact --nocapture`
for each named UI9-C row below so no earlier case can hide its diagnostic.

- category projection: `nearby_station_context_is_complete_at_supported_profiles`,
  `nearby_node_context_is_complete_at_supported_profiles`, and
  `nearby_colonist_context_is_complete_at_supported_profiles` independently;
- authoritative state: `context_detail_and_actions_follow_authoritative_target_state`
  for construction and
  `depleted_node_context_changes_detail_and_action_applicability` for depletion;
- active authoritative state:
  `staffed_station_context_includes_worker_recipe_and_progress`,
  `assigned_node_context_includes_worker_and_progress`,
  `assigned_colonist_context_includes_target_and_progress`,
  `carrying_colonist_context_includes_target_and_cargo`, and
  `blocked_colonist_context_includes_target_and_reason` independently;
- shared-owner seam:
  `context_view_model_transports_shared_detail_without_semantic_parsing`, which
  proves complete semantic-segment transport independently, and
  `final_context_consumes_the_shared_detail_projection_once`, which supplies a
  distinctive Unstaffed shared `detail` projection with an opposing staffed
  worker decoy and requires every final title/body decision to remain coherent;
- action truth: `passive_context_never_advertises_unroutable_actions_as_enabled`,
  `passive_node_context_never_advertises_unroutable_actions_as_enabled`, and
  `passive_colonist_context_never_advertises_unroutable_actions_as_enabled`,
  plus `a_binding_without_a_context_reducer_does_not_enable_interact`;
- final composition: `station_context_survives_final_composition_at_supported_profiles`,
  `node_context_survives_final_composition_at_supported_profiles`, and
  `colonist_context_survives_final_composition_at_supported_profiles`, plus the
  five independently named active-state final-composition rows; and
- multi-target focus:
  `duplicate_named_nearby_targets_remain_distinguishable_in_context`.

The post-v2 red baseline retains the valid production staffing work and has two
independently observable shared-owner failures. The transport case rejects
category/staffing segments stripped from formatted `detail`; the paired final
case rejects a `Station Staffed` title when the authoritative status/detail are
Unstaffed and only the poisoned legacy worker field implies staffed. All
production-reachable staffed-station, assigned-node, active-colonist,
bound-without-reducer, default category, construction/depletion,
duplicate-target, and neighboring cases are preservation guards; they are not
substitutes for these two reds.

For every group, use the loop `READ → BASELINE → DIAGNOSE → IMPLEMENT → FOCUSED
VALIDATION → CLASSIFY`. Repeat while red. After the whole registered UI9-C set
passes, run neighboring targets and the signed candidate gate, inspect the production
diff for category-specific renderers or duplicated action rules, and report
`CandidateGreen` only. UI9-D remains unauthorized until Section 17.2 receives
an owner decision; do not bind Interact or implement the menu reducer in this
handoff.

#### UI9-C v3 shared-owner review — withdrawn 2026-08-03

The v3 implementation repaired the two mixed-source reds inside
`crates/bd_tui/src/view_models.rs`: complete `NearbyTarget.detail` now reaches
the Context view model, and the title follows authoritative category/status
rather than poisoned worker/recipe/progress fields. Those two exact rows are
green preservation boundaries for the next batch.

Independent final-composition review then reproduced three remaining 60x20
failures: the default station loses the Set Production denial reason, the
staffed station loses later action/reason content, and the assigned node clips
the Assign Gatherer row. The complete semantic detail plus required action and
denial text cannot fit the existing three-row compact Context composition
through a legal `view_models.rs`-only change. The v3 write set is therefore
exhausted and its `NotComplete` status is correct.

The v3 body, baseline, manifest, and digest are withdrawn historical evidence
and must remain unchanged. A candidate-created execution-handoff document is
not authority and must not be carried into the next baseline. The next
reviewer-sealed UI9-C v4 batch may modify exactly:

```text
crates/bd_tui/src/view_models.rs
crates/bd_tui/src/screens.rs
```

V4 owns one reusable Context presentation/composition correction. It may
arrange, wrap, style, or allocate the generic structured target/detail/action
representation in `screens.rs`, but may not parse or rederive domain facts,
introduce category/profile/fixture branches, weaken detail/action truth, change
input routing, or implement UI9-D. It must begin from the three independently
reproduced 60x20 diagnostics, preserve the two v3 shared-owner greens and all
registered UI9-C category/state/action/focus rows, then close through the
signed candidate gate and the required in-chat implementation handoff.

#### UI9-D — Input reducer and menu shell

This batch starts only after Section 17.2 is owner-locked.

Its first red workflow must cross-check configuration, Help/footer, projected
key hints, and runtime routing. It opens Context through the production key,
selects a target when several are present, invokes one enabled action, cancels,
and proves all UI-only steps are paused. No action may be promoted from preview
to enabled until that workflow exists.

1. Add one semantic Interact command and derive Help/footer/action hints from
   the configured binding.
2. Implement zero/one/many target transitions in the existing paused
   interaction reducer rather than a second input pipeline.
3. Keep `Set Production` disabled and non-mutating.
4. Preserve existing `c` direct-task behavior and route any retained staffing
   entry to the existing workflow.
5. Add production-input workflow evidence for open, choose target, navigate,
   cancel, resize/close cleanup, and forbidden mutations.

### 17.8 Invalid shortcuts the green audit must reject

- hardcoding Water Source, Basic Processing, Mara, test coordinates, or the
  two supported terminal sizes in production;
- deriving tooltips from glyphs or renderer ECS queries;
- appending Chronicle messages during draw/projection frames;
- printing the forecast string beside a decorative unrelated bar;
- one station menu, one node menu, and one colonist menu with duplicated
  layout/control logic;
- making `Set Production` enabled while it performs no mutation;
- deleting HP/AP, worker detail, compact content, or existing shortcuts to
  free space;
- editing fixtures, assertions, contract status, or observers merely to make
  the gate green.

### 17.9 Closing evidence and status

The implementation agent may report `CandidateGreen` only after the focused
owners, completion-critical supports, neighbors, and signed candidate gate
pass. `VerifiedGreen` requires independent review, atomic status-ledger
reconciliation, and the zero-exit canonical gate. `ReviewedGreen` additionally
requires production-diff/DRY
review, final 80x24 and 60x20 buffers, required PTY workflows, authority drift
review, and aligned registry/evidence records.

At the current handoff, `VISUAL-ECON-002` and `COLONY-PROXIMITY-001` are
`GreenUnreviewed`; `VISUAL-CONTEXT-001` is accurately `Red` on the independent
shared-detail transport and mixed-source final-coherence seams above. The input/menu-shell contract remains
unregistered until the Section 17.2 key decision is recorded; a test must not
silently decide that owner choice.
Each batch must be independently reviewable and revertible. Never mix harness
work, layout redesign, gameplay changes, and unrelated cleanup.

---

## 18. Per-Batch Completion Record

Record:

```text
Batch:
Authority:
Contract owner:
Files intended before editing:
Focused test:
Expected red result:
Production change:
Focused green result:
Neighboring regression result:
Player workflow result:
Canonical gate totals:
Visual/PTY evidence:
GDD drift result:
Evidence documents updated:
Unexpected results:
```

A batch is incomplete if a field is omitted without explanation.

---

## 19. Risk Controls

| Risk | Required control |
|---|---|
| UI changes simulation timing | Assert no time, worker, position, or resource mutation during paused workflows |
| Renderer becomes a domain model | Project typed authoritative state through view models |
| Compact profile diverges | Share projections; vary composition only |
| Snapshots become brittle | Use granular contracts; snapshots support review |
| Color hides ambiguity | Reduced-color glyph/modifier tests |
| Messages duplicate | One structured result projection |
| Polish expands gameplay | Stop when a required fact is absent from authority |
| Dirty work is lost | Inspect status, patch narrowly, never reset/clean |
| Synthetic green overclaims UX | Require PTY and manual visual evidence |

---

## 20. Non-Goals

No mouse, wall-clock animation, graphical tiles, new mechanics, queues,
priorities, upgrades, deeper automation, raids, events, sanity, overworld,
procedural topology, new dungeon content, faction reputation, final narrative,
broad balance changes, Ratatui replacement, or ECS rewrite.

---

## 21. Definition of Done

The pass is complete only when:

- complete colony and dungeon journeys work at 80x24 and 60x20;
- mode, selection, actions, denials, worker state, targets, progress,
  resources, and results are readable;
- build and management remain paused and atomic;
- critical states remain distinct without color;
- player and survivors cannot disappear through layer order;
- off-screen targets remain discoverable;
- accepted actions produce at most one accurate primary result;
- dungeon combat, loot, extraction, and defeat are clear;
- persistence and resize preserve coherent presentation;
- every exit restores the terminal;
- required tests pass with zero required ignores;
- applicable visual evidence is complete and honestly classified;
- canonical, lint, format, check, and whitespace gates pass;
- registry, requirement map, evidence report, and visual matrix agree;
- final review finds no GDD or locked-decision drift;
- no deferred feature was activated.

Passing tests alone does not satisfy this definition.
