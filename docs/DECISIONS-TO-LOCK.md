# Broken Divinity Decisions to Lock

This is a decision register, not an implementation plan. Each item must be resolved before it becomes a detailed task. Until then, agents should preserve the ambiguity and should not invent a choice in code.

## D-01 — Product MVP boundary — LOCKED

**Conflict:** `GDD.md` defines MVP as the complete shelter, overworld, dungeon, combat, extraction, and return loop. `Kernel.md` defines its tactical MVP as a compact ruin loop with a placeholder outpost.

**Decision:** The first MVP is a functioning kernel shell and playable foundation. The kernel’s core functions are locked down first, then the game expands from that foundation. Dungeon and colony basics are the immediate product focus; the full shelter-to-overworld-to-dungeon vision is a later expansion.

## D-02 — Runtime and presentation authority — LOCKED

**Current evidence:** The current workspace uses Bevy-Ratatui/Ratatui in `broken-divinity/Cargo.toml`. Older project documentation still describes Bevy plus `bevy_egui`.

**Decision:** Ratatui through Bevy-Ratatui is the player-facing runtime direction. It is lighter-weight and appropriate because the project has no established asset pipeline that would justify a heavier presentation layer.

## D-03 — Faction scope for MVP — LOCKED

**Conflict:** The root GDD uses a minimal anchor set and leaves the major demon counterpart unnamed. Other project documents specify Michael’s Host, Fort Pershing, The Collective, and additional hardcoded factions.

**Decision:** MVP uses two random placeholder factions. Their definitions must be data-driven and extensible so named and more complex factions can be added later without replacing the faction system.

## D-04 — Sanity model — LOCKED FOR MVP

**Conflict:** Some documents describe a single short-term exposure track; others describe short-term Raid Exposure plus persistent Long-Term Erosion.

**Decision:** Sanity is deferred. MVP focuses on core dungeon mechanics and the colony side of the game. Sanity may be added after those foundations are stable.

## D-05 — Progression model — DIRECTION LOCKED

**Conflict:** The root GDD makes six virtues foundational and retains practical proficiencies only as supporting lanes. Older gameplay documents use a conventional skill-first model.

**Decision:** Progression is a blend. Practical skills improve through use and training, while actions and choices express virtues. Exact skill-to-virtue mappings and balance remain future planning work.

## D-06 — Theology-to-system requirements — DEFERRED

**Current gap:** The design requires theology to affect law, legitimacy, corruption, faction response, settlement evolution, and sanity, but the technical MVP acceptance criteria do not require those connections.

**Decision:** Theology-driven mechanics are deferred until after the MVP foundation. The long-term design pillar remains valid, but it is not a current implementation gate.

## D-07 — Canonical documentation location — LOCKED

**Current state:** Root-level documents, `broken-divinity/docs/`, and mirrored `docs/design/` material all contain overlapping information.

**Decision:** Product design belongs in `GDD.md` and other explicitly marked root-level canonical design files. The `docs/` tree is for general documentation, decisions, technical notes, migration records, and history. Existing project-local docs remain available as implementation references until reconciled.

## D-08 — Foundation-plan replacement and stabilization — LOCKED

**Current state:** The historical UX recovery plan is archived at
`docs/archive/foundation-plan-2026-07-24/HISTORICAL-UX-RECOVERY-PLAN.md`.
The former `ACTIVE-PLAN.md` and its phase contracts are archived beside it
because their completion claims did not pass the final integration audit.

**Decision:** `docs/FOUNDATION-RECOVERY-PLAN.md` was the sole Foundation
recovery plan and remains the completed chronological evidence record. A later
clean-session audit disproved parts of its final acceptance gate.

`docs/FOUNDATION-STABILIZATION-PLAN.md` was the sole active execution plan for
the first reopened Foundation gates and remains a completed chronological
record. Its acceptance was later reopened by discovery evidence.
`docs/FOUNDATION-MVP-CORRECTION-PLAN.md` then became the owner-approved
implementation authority and is now the completed correction acceptance
record.
The later deep colony UX audit reopened only the affected Foundation test,
spatial-safety, and presentation gates. The active implementation authority is
now `docs/FOUNDATION-TEST-AND-UX-HARDENING-PLAN.md`. Product P2 and every
deferred system still require a separate owner-approved plan.

## D-09 — Code and documentation preservation — LOCKED

**Decision:** Existing code, tests, content, and useful documentation are preserved by default. Working systems should be reused, adapted, or explicitly deprecated; they should not be discarded merely because the architecture is being reorganized. Deprecation must identify a replacement or migration destination.

## D-10 — MVP dungeon loop — LOCKED

**Decision:** The MVP dungeon must support the complete playable loop: enter a dungeon, explore, fight, acquire loot, reach the exit, and leave with the result applied to the colony/run state. The first slice needs only enough content to prove this loop; it does not need a broad content roster.

## D-11 — MVP dungeon layout — LOCKED

**Decision:** The first dungeon is fixed and hand-authored. Procedural generation is not part of the foundation entry path. Existing procgen code is preserved as reusable future infrastructure and must not be deleted, but it is deferred until the fixed dungeon loop is stable.

## D-12 — MVP colony scope — LOCKED

**Decision:** The colony foundation keeps the existing basic shelter, survivor assignment, resource, station, and production foundations where they fit, including the current five station types and small survivor-management slice. Raids and colony events are deferred. The colony must provide a stable destination for extracted loot and a place to observe basic management state.

## D-13 — Initial practical skills — LOCKED FOR FOUNDATION

**Decision:** The initial practical skill set is melee, ranged, repair, and medicine. Skills improve through use or training. The skill model must be extensible, but the foundation does not require a complete skill tree or final balance pass.

## D-14 — Virtue implementation order — LOCKED FOR FOUNDATION

**Decision:** Build the progression interfaces and action-to-virtue hooks as part of the foundation, then implement a small representative set of virtue-reflective actions. Full virtue mapping, theological interpretation, perks, and balance are post-foundation expansion.

## D-15 — Foundation content and state — LOCKED

**Decision:** The foundation uses a deterministic run state, a fixed dungeon definition, two data-driven placeholder factions, basic combat and loot, and colony return/extraction. Overworld travel, procgen, raids, events, sanity, theology-driven mechanics, faction reputation, and final narrative content are explicitly outside this foundation.

## D-16 — Foundation stabilization behavior — LOCKED

**Current evidence:** The clean-session audit found that station construction
and direct dungeon entry do not apply visible colony costs coherently, while
the existing 24-turn daily cycle is technically present but not reasonably
accessible through the terminal.

**Decision:**

- `ColonyResources` is the sole owner of Foundation shelter resources.
- Station validation and payment use that same owner.
- The existing direct shelter-to-fixed-dungeon interaction remains and costs
  the existing named value of 2 colony Supplies through the shared action
  pipeline. This does not activate full overworld travel.
- The shelter exposes one Rest Until Next Day action that reaches the existing
  authoritative day boundary. Production, consumption, gathering, and mood
  consequences continue to consume that boundary exactly once.
- 80x24 remains the required baseline terminal profile and 60x20 remains the
  supported compact profile.

Balance changes, full travel, deeper colony simulation, and new content remain
outside Foundation stabilization.

## D-17 — Foundation MVP correction contract — LOCKED

**Current evidence:** The 2026-07-25 post-stabilization discovery run proved
the listed defects. The completed correction record now proves each decision
below through production-path tests and clean 80x24/60x20 terminal runs.

**Decision:**

- every `DayAdvanced` boundary runs one mode-independent colony transaction;
- zero Supplies has an explicit, targetable recovery path through survivor
  gathering and guaranteed shelter resource nodes;
- one validated station catalog owns cost, description, staffing, and effect;
- Storage remains preserved but cannot be constructed until it has an
  implemented Foundation effect;
- survivor task and station assignment target named entities explicitly;
- active-run state and last-completed-run state have separate persisted owners;
- extraction and defeat restart use a validated shelter return point;
- the redundant `Z` map/combat command is removed;
- the fixed dungeon receives a content-only depth pass after behavior is
  stable.

The detailed owner-authorized sequence is
`docs/FOUNDATION-MVP-CORRECTION-PLAN.md`; its final gate passed on 2026-07-25.
This decision does not activate Product P2 or any deferred system.

## D-18 — Foundation test and colony UX hardening contract — LOCKED

**Current evidence:** A later deep real-terminal playtest at 80x24 and 60x20
proved that the green test suite and completed correction gate did not cover
player trapping, camera/viewport behavior, resource-node discoverability,
worker presentation, semantic glyph collisions, compact text completeness, or
the documented management-time contract. The owner approved the recommended
behavioral corrections and requested an implementation plan suitable for a
smaller coding model.

**Decision:**

- task and station management is paused; navigation, confirmation, and
  cancellation do not advance game time;
- survivor assignments remain durable tasks, while player-facing activity
  explicitly distinguishes Idle, EnRoute, Working, Blocked, Resting, and any
  later active Defending state;
- assigned survivors move deterministically on later time-advancing Outpost
  turns, not during paused assignment;
- Rest Until Next Day simulates the same survivor movement steps as the
  equivalent individual Outpost turns;
- station and gathering output requires the assigned survivor to reach a valid
  cardinally adjacent work tile;
- survivors do not occupy station/resource tiles or stack on one another;
- every accepted station placement preserves a path from the player to the
  shelter gate and an egress rejection is atomic;
- the 40x30 shelter uses one player-following, edge-clamped viewport transform
  for all terrain and entity layers;
- assigned off-screen targets remain discoverable at both supported terminal
  profiles;
- survivor, station, resource, staffing, work-state, and target indicators use
  semantic visual tokens with unambiguous simultaneous glyph/style pairs;
- Help or an equally accessible legend explains the active shelter visual
  language;
- 60x20 remains a genuinely supported profile with complete decisive text and
  consistent controls;
- test names may claim only directly asserted behavior, and Foundation
  acceptance requires evidence at the domain, schedule/state, presentation,
  player-path, and real-terminal layers where applicable.

The owner-approved implementation sequence and stop rules are defined only in
`docs/FOUNDATION-TEST-AND-UX-HARDENING-PLAN.md`. This decision repairs
Foundation trust and usability; it does not authorize Product P2 or any
deferred feature.

## D-19 — Authoritative testing standard and suite migration — LOCKED

**Current evidence:** The Foundation suite contains hundreds of useful tests,
but repeated green-suite claims have not predicted player-facing quality.
Current evidence mixes primary acceptance, support, deferred systems, legacy
fixtures, substring checks, count-only persistence, and manual observations.
Test totals have also been reported without consistently distinguishing listed,
passed, failed, and ignored tests.

**Decision:**

- Foundation acceptance is requirement-driven, not test-count-driven;
- every required contract has exactly one registered primary test and explicit
  evidence layers;
- every active test is classified as required, support, regression, future,
  deferred, diagnostic, or legacy pending retirement;
- player-facing contracts require production-path workflow and presentation
  evidence where applicable;
- primary tests use stable identity, explicit frame control, normalized state
  diffs, and high-information failure reports;
- deferred and legacy tests remain visible but do not determine Foundation
  acceptance;
- weak or duplicate tests may be moved or retired only after a named stronger
  replacement passes and the migration is recorded;
- visual acceptance requires semantic, canvas, resolved-style, geometry,
  transition, and real-terminal evidence where applicable;
- generated metrics must distinguish listed, passed, failed, ignored,
  deferred, and accepted contracts;
- line coverage and total test count are diagnostic metrics, not acceptance
  gates.

`docs/AUTHORITATIVE-TESTING-STANDARD-AND-MIGRATION-PLAN.md` owns testing policy,
evidence sufficiency, metrics, and suite migration.
`docs/FOUNDATION-TEST-AND-UX-HARDENING-PLAN.md` continues to own behavior and
implementation order. Neither document authorizes Product P2 or any deferred
feature.

## D-20 — Foundation basic colony loop — LOCKED

**Current evidence:** The D-18 hardening pass repaired worker movement,
occupancy, paused management, viewport behavior, and physical work range, but
the colony still does not present one coherent source-to-station production
loop. Build placement remains effectively locked to the player's cardinal
neighbors. Existing shelter resource nodes are placed by a map-dimension
formula without run-seed variation, configured spacing, complete type
coverage, or a complete-or-error placement contract. Gathering and station
production remain separate daily-boundary abstractions with no raw cargo or
recipe-backed refining.

**Decision:**

- deterministic colony resource-fixture placement is active Foundation work;
  it places fixtures only on the existing fixed shelter map and does not
  activate procedural shelter topology, procedural dungeons, overworld
  generation, raids, or events;
- Foundation content defines three initial placeholder chains:
  - Trees → Raw Timber → Refined Materials → Materials;
  - Water Source → Raw Water → Refined Supplies → Supplies;
  - Wild Plants → Raw Plants → Refined Plants → Wild Plants;
- stable content IDs own source, raw-resource, recipe, station, and result
  identity; temporary labels may change without changing simulation identity;
- one data-defined basic processing station supports the three placeholder
  recipes and one guaranteed starter instance exists so zero-Supplies
  recovery cannot depend on first paying a construction cost;
- additional processing-station instances may use the existing data-driven
  build catalog; the existing five station types remain preserved and Storage
  remains disabled until it receives a separately approved effect;
- station-backed production assignment uses paused `e` management:
  named survivor → named compatible station → named recipe when required →
  review → confirm; `c` continues to own non-station survivor tasks;
- one accepted Outpost worker tick performs at most one cardinal movement
  step, one gathering operation, or one refining operation;
- reaching a work tile changes the worker to a ready/working stage; gathering
  or refining consumes a later worker tick rather than occurring on the
  arrival tick;
- rendering, paused UI, Tactical turns, saving, and loading produce zero
  Outpost worker ticks; Rest replays the same ordered ticks as equivalent
  individual Outpost turns;
- Foundation content declares one initial node for each active source through
  configured counts; generation logic must not hardcode the number or source
  roster;
- placement uses a named validated spacing profile, produces unique
  unoccupied fixtures with reachable cardinal work tiles, and either returns
  a complete plan or a typed error without partial spawning;
- node placement is deterministic from run seed, fixed map, placement salt,
  and stable content identity; it runs once for a new colony, persists, and
  never regenerates on load or day advancement;
- Foundation nodes are renewable and non-depleting for this slice;
- a durable production job records stable recipe and target identity, exact
  production stage, and carried raw cargo;
- blocked workers retain cargo; a missing source creates no cargo; a missing
  station or route creates no output;
- cancellation or reassignment deposits carried raw cargo atomically into the
  sole `ColonyResources` owner; no raw input is silently destroyed;
- gathering changes raw cargo/input only; finished output appears only after
  one successful refine transition that consumes the configured input;
- per-turn physical work and the existing day-boundary transaction must not
  duplicate output; converted recipe stations do not also receive legacy free
  production;
- forecast and Rest consume the same transition semantics as ordinary turns,
  or present separately named next-worker and next-day results without
  claiming equivalence they do not calculate;
- build placement starts adjacent to the player but cursor movement is
  cumulative across the fixed shelter, remains paused, may preview invalid
  cells, and submits the absolute selected coordinate;
- placement confirmation revalidates the exact coordinate, preserves gate
  egress and a reachable station work tile, and remains atomic on rejection;
- while placing, the viewport follows the build cursor; cancellation or
  resolution restores player-following behavior;
- the player-facing colony projection exposes survivor, recipe, stage, target,
  cargo, blocked reason, and completed resource result at both supported
  terminal profiles.

The owner-approved implementation and evidence sequence is
`docs/FOUNDATION-BASIC-COLONY-LOOP-PLAN.md`. D-19 remains the testing authority.
This decision deepens the Foundation's basic colony loop only. It does not
authorize Product P2 colony automation, production queues, station upgrades,
resource depletion balance, procedural map topology, or any other deferred
system.

## D-21 — Turn-based work and idle construction — LOCKED

**Decision:**

- recipe content owns positive `gather_work_turns` and `refine_work_turns`
  values in addition to input and output amounts;
- the Foundation defaults are three gather work turns for one configured raw
  batch and two refine work turns for one configured finished batch;
- a work tick advances progress by exactly one only while the survivor is at a
  valid work tile; movement, arrival, blocked time, paused UI, rendering,
  Tactical turns, save, and load do not advance work progress;
- no raw or finished resource is credited before the corresponding work
  requirement is complete; completion credits the configured amount exactly
  once and resets operation progress before the next stage;
- buildable station content owns a positive `construction_work_turns` value;
  the Foundation default is four work turns;
- accepting placement pays once and creates a blocking construction site at
  the selected coordinate; it does not create an operational station;
- survivors whose durable task is `Idle` and who have no production job
  automatically select reachable construction work in stable order, travel
  with the existing worker movement rules, and contribute at most one work
  unit per accepted Outpost worker tick;
- automatic construction never reassigns survivors who are gathering,
  stationed, resting, defending, or carrying production cargo;
- a construction site becomes its selected operational station exactly once
  when its remaining work reaches zero; unfinished sites and work progress
  persist across save/load;
- legacy day-boundary gathering and free station production may remain only
  for explicitly retained non-recipe tasks/effects. They must not credit the
  same recipe or construction transaction.

This is Foundation worker scheduling, not a general priority, hauling,
blueprint, stockpile, or construction-queue system.

## D-22 — Direct gathering work coherence — LOCKED

**Decision:**

- `c` remains the owner of non-station survivor tasks, including direct
  Supplies, Materials, and Wild Plants gathering;
- each direct gathering definition owns a stable source identity, finished
  colony pool, positive output amount, and positive work-turn requirement;
- Foundation direct gathering requires three adjacent work turns and produces
  one configured resource; movement and arrival produce nothing;
- rendering, paused UI, blocked time, Tactical turns, save, and load produce
  no gathering work;
- a crossed day boundary does not credit legacy direct-gather output; Rest
  replays the same worker ticks as equivalent individual Outpost turns;
- partial direct-gather progress survives save/load and is cleared without
  output when the survivor is reassigned;
- direct gathering remains the station-independent emergency recovery path;
  recipe work remains the source-to-cargo-to-station refining path and the two
  paths never credit the same operation;
- survivor and management presentation uses human labels and exposes source,
  progress, configured result, and blocked reason without internal content
  IDs;
- the colony projection distinguishes next worker completion from next-day
  upkeep and exposes nonzero raw stockpiles separately from finished pools.

This decision improves Foundation work coherence and feedback. It does not
authorize queues, priorities, hauling, upgrades, depletion, raids, events, or
Product P2 automation.
