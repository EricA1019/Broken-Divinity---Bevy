# Broken Divinity Documentation Hub

**Status:** Active navigation authority

**Updated:** 2026-07-28

Read this file before using any other Broken Divinity document.

## Authority order

1. [GDD.md](../GDD.md) — sole product and player-experience authority.
2. [DECISIONS-TO-LOCK.md](DECISIONS-TO-LOCK.md) — locked product choices.
3. [Kernel.md](../Kernel.md) — technical architecture authority.
4. [Kernel-direction.md](../Kernel-direction.md) — subordinate technical appendix.
5. [MIGRATION-AND-DEPRECATION.md](MIGRATION-AND-DEPRECATION.md) — preservation policy.
6. [MVP-SCENARIO.md](MVP-SCENARIO.md) — canonical Foundation acceptance scenario.
7. [AUTHORITATIVE-TESTING-STANDARD-AND-MIGRATION-PLAN.md](AUTHORITATIVE-TESTING-STANDARD-AND-MIGRATION-PLAN.md) — active testing policy, evidence, metrics, and suite-migration authority.
8. [FOUNDATION-TEST-AND-UX-HARDENING-PLAN.md](FOUNDATION-TEST-AND-UX-HARDENING-PLAN.md) — active Foundation behavior and colony UX implementation authority.
9. [FOUNDATION-BASIC-COLONY-LOOP-PLAN.md](FOUNDATION-BASIC-COLONY-LOOP-PLAN.md) — owner-approved D-20 basic colony-loop implementation and evidence plan.
10. [FOUNDATION-UI-IMPROVEMENT-PLAN.md](FOUNDATION-UI-IMPROVEMENT-PLAN.md) — Foundation Ratatui improvement sequence and UI phase gates.
11. [FOUNDATION-MVP-CORRECTION-PLAN.md](FOUNDATION-MVP-CORRECTION-PLAN.md) — completed Foundation correction and acceptance record.
12. [FOUNDATION-STABILIZATION-PLAN.md](FOUNDATION-STABILIZATION-PLAN.md) — completed stabilization execution record.
13. [FOUNDATION-RECOVERY-PLAN.md](FOUNDATION-RECOVERY-PLAN.md) — completed Foundation execution record and prior evidence.
14. [DOCUMENT-INVENTORY.md](DOCUMENT-INVENTORY.md) — classification and ownership of all project documentation.

Code, tests, content, reference documents, and archived plans are evidence.
They do not override the authority order.

## Current project status

The completed 2026-07-25 correction gate proved cross-mode daily transactions,
recoverable economy, data-driven stations, explicit named management,
completed-run history, named shelter returns, persistence, and the fixed
dungeon experience. A later deeper colony UX audit reopened the affected
Foundation gates after proving player-trapping construction, a missing shelter
viewport, invisible required targets, incoherent worker movement/presentation,
semantic glyph collisions, compact truncation, and a management-time contract
violation.

`AUTHORITATIVE-TESTING-STANDARD-AND-MIGRATION-PLAN.md` owns testing policy,
evidence sufficiency, metrics, and suite migration.
`FOUNDATION-TEST-AND-UX-HARDENING-PLAN.md` owns behavior and implementation
order. These are coordinated authorities with non-overlapping roles.
`FOUNDATION-BASIC-COLONY-LOOP-PLAN.md` defines the active test-first D-20
colony vertical slice. Its Phase C0 defaults were owner approved and recorded
on 2026-07-27.
`FOUNDATION-UI-IMPROVEMENT-PLAN.md` defines the test-first execution order for
improving existing Foundation presentation. It does not authorize gameplay
expansion, and implementation has not started.
`FOUNDATION-MVP-CORRECTION-PLAN.md` remains the completed correction record;
its unaffected simulation, persistence, economy, and dungeon evidence is
preserved. The stabilization and recovery plans remain earlier chronological
evidence records.

Product P2 is not authorized and still requires a separate owner-approved
plan.

## Active product boundary

The immediate target is:

- one persistent shelter;
- three survivors;
- five station types;
- assignment and one production cycle;
- one fixed hand-authored dungeon;
- movement, combat, loot, explicit extraction, and defeat;
- return results applied to the colony exactly once;
- practical skill growth and two representative virtue hooks;
- two data-driven placeholder factions;
- deterministic save/load;
- a clear Ratatui player path.

Deferred:

- procgen in the Foundation path;
- full overworld travel;
- raids and colony events;
- sanity;
- theology-driven mechanics;
- faction reputation;
- final faction canon;
- deeper narrative.

## Document roles

### Active

- Root design and technical authority files.
- Locked decisions and migration policy.
- MVP scenario.
- Authoritative Testing Standard and Migration Plan as active test-governance
  authority.
- Foundation Test and Colony UX Hardening Plan as active behavior execution
  authority.
- Foundation Basic Colony Loop Plan as the owner-approved D-20 colony-loop
  execution plan.
- Foundation UI Improvement Plan as the active presentation implementation
  sequence; implementation is pending.
- Foundation MVP Correction Plan as completed acceptance evidence.
- Foundation Stabilization Plan as completed evidence.
- Foundation Recovery Plan as completed evidence.
- Documentation Inventory.

### Reference

`docs/design/` and selected `broken-divinity/docs/` material may provide useful
history, lore, and implementation context. It remains subordinate to the root
GDD and canonical Foundation scenario.

### Historical

All files under `docs/archive/` are historical. They are preserved for evidence
and must not be used as current instructions.

The old `ACTIVE-PLAN.md`, prior phase contracts, historical UX plan, stale
repository-local GDD, and old repository development plan were archived during
Foundation Recovery Phase 0.

## Required reading order for an implementation agent

1. This hub.
2. Root `GDD.md`.
3. `DECISIONS-TO-LOCK.md`.
4. Relevant `Kernel.md` guardrails.
5. `MVP-SCENARIO.md`.
6. `AUTHORITATIVE-TESTING-STANDARD-AND-MIGRATION-PLAN.md`.
7. `FOUNDATION-TEST-AND-UX-HARDENING-PLAN.md`.
8. `FOUNDATION-BASIC-COLONY-LOOP-PLAN.md` when working on the active D-20
   colony vertical slice.
9. `FOUNDATION-UI-IMPROVEMENT-PLAN.md` when working on Foundation
   presentation or interaction polish.
10. `FOUNDATION-MVP-CORRECTION-PLAN.md` for completed correction and acceptance evidence.
11. `FOUNDATION-STABILIZATION-PLAN.md` for completed stabilization evidence.
12. `FOUNDATION-RECOVERY-PLAN.md` for completed Foundation evidence.
13. `DOCUMENT-INVENTORY.md` when consulting any other document.
14. Current code and tests.

## Stop rule

Stop and ask the project owner when:

- canonical documents conflict;
- a product choice is missing;
- work would activate a deferred system;
- preserving existing work would materially expand scope;
- a proposed implementation creates another source of truth.

Do not resolve ambiguity by selecting an older plan or design copy.
