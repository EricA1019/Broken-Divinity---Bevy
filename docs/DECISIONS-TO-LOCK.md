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
