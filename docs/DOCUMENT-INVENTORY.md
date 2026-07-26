# Broken Divinity Documentation Inventory

**Status:** Active documentation authority map

**Inventoried:** 2026-07-26

**Active testing-governance plan:** [AUTHORITATIVE-TESTING-STANDARD-AND-MIGRATION-PLAN.md](AUTHORITATIVE-TESTING-STANDARD-AND-MIGRATION-PLAN.md).

**Active behavior implementation plan:** [FOUNDATION-TEST-AND-UX-HARDENING-PLAN.md](FOUNDATION-TEST-AND-UX-HARDENING-PLAN.md).

The plans have non-overlapping authority: the testing plan owns evidence,
metrics, and suite migration; the hardening plan owns behavior and
implementation order. Product P2 still requires a separate owner-approved
plan.

**Completed Foundation correction record:** [FOUNDATION-MVP-CORRECTION-PLAN.md](FOUNDATION-MVP-CORRECTION-PLAN.md)

**Storage policy:** Preserve its type/content and disable construction until it
has an implemented Foundation effect.

**Completed Foundation recovery record:** [FOUNDATION-RECOVERY-PLAN.md](FOUNDATION-RECOVERY-PLAN.md)

**Completed Foundation stabilization record:** [FOUNDATION-STABILIZATION-PLAN.md](FOUNDATION-STABILIZATION-PLAN.md)

## Classification meanings

- **Active** — current authority or required coordination document.
- **Reference** — useful supporting material subordinate to active authority.
- **Reconcile** — contains potentially useful but conflicting or duplicated material.
- **Historical** — preserved evidence that must not direct current work.
- **Deprecated** — superseded path with a named replacement.

## Active project authority

| Path | Classification | Role |
|---|---|---|
| `GDD.md` | Active | Sole product design authority |
| `Kernel.md` | Active | Technical architecture authority |
| `Kernel-direction.md` | Active | Technical appendix subordinate to `Kernel.md` |
| `docs/README.md` | Active | Documentation hub and reading order |
| `docs/DECISIONS-TO-LOCK.md` | Active | Locked product decisions |
| `docs/MIGRATION-AND-DEPRECATION.md` | Active | Preservation and deprecation policy |
| `docs/MVP-SCENARIO.md` | Active | Canonical Foundation acceptance scenario |
| `docs/AUTHORITATIVE-TESTING-STANDARD-AND-MIGRATION-PLAN.md` | Active plan | Foundation testing policy, evidence sufficiency, metrics, and suite-migration authority |
| `docs/FOUNDATION-TEST-AND-UX-HARDENING-PLAN.md` | Active plan | Foundation behavior and colony UX implementation-order authority |
| `docs/FOUNDATION-MVP-CORRECTION-PLAN.md` | Active record | Completed owner-authorized Foundation correction evidence; affected UX/test gates were later reopened |
| `docs/FOUNDATION-STABILIZATION-PLAN.md` | Active record | Completed stabilization evidence; its acceptance was reopened and superseded by the accepted correction record |
| `docs/FOUNDATION-RECOVERY-PLAN.md` | Active record | Completed earlier recovery evidence; final result is linked to stabilization |
| `docs/DOCUMENT-INVENTORY.md` | Active | Documentation classification and ownership map |

No other file may claim product or implementation authority.

## Project-level design reference library

Everything under `docs/design/` is non-authoritative reference material. A
requirement found here must be confirmed in the root GDD and active recovery
plan before implementation.

### Gameplay

| Path | Classification | Notes |
|---|---|---|
| `docs/design/gameplay/README.md` | Reconcile | Differs from repository-local copy |
| `docs/design/gameplay/colony.md` | Reconcile | Differs from repository-local copy |
| `docs/design/gameplay/combat.md` | Reconcile | Differs from repository-local copy |
| `docs/design/gameplay/overworld.md` | Reconcile | Differs from repository-local copy; overworld deferred |
| `docs/design/gameplay/phase-roadmap.md` | Historical | Differs from repository-local copy; not an execution plan |
| `docs/design/gameplay/procgen.md` | Reference | Identical to repository-local copy; procgen deferred |
| `docs/design/gameplay/progression.md` | Reconcile | Differs from repository-local copy |
| `docs/design/gameplay/virtues.md` | Reference | Identical to repository-local copy |

### Lore

| Path | Classification | Notes |
|---|---|---|
| `docs/design/lore/Broken_Divinity_Primer.md` | Reference | Lore reference subordinate to root GDD |
| `docs/design/lore/README.md` | Reconcile | Differs from repository-local copy |
| `docs/design/lore/dungeon-themes.md` | Reference | Identical to repository-local copy |
| `docs/design/lore/factions.md` | Reference | Identical to repository-local copy; final faction canon deferred |
| `docs/design/lore/naming-conventions.md` | Reference | Identical to repository-local copy |
| `docs/design/lore/sanity.md` | Reference | Identical to repository-local copy; sanity deferred |
| `docs/design/lore/species.md` | Reference | Identical to repository-local copy |
| `docs/design/lore/thaumaturgy.md` | Reference | Identical to repository-local copy |
| `docs/design/lore/the-sundering.md` | Reference | Identical to repository-local copy |
| `docs/design/lore/the-world-now.md` | Reference | Identical to repository-local copy |
| `docs/design/lore/tone-guide.md` | Reference | Identical to repository-local copy |

### Technical and UI design

| Path | Classification | Notes |
|---|---|---|
| `docs/design/tech/architecture.md` | Reconcile | Differs from repository-local copy; subordinate to `Kernel.md` |
| `docs/design/tech/ui-design.md` | Reconcile | Differs from repository-local copy |
| `docs/design/ui/README.md` | Reconcile | Differs from repository-local copy |
| `docs/design/ui/ingame.md` | Reconcile | Differs from repository-local copy |
| `docs/design/ui/menus.md` | Reconcile | Differs from repository-local copy |
| `docs/design/ui/overworld.md` | Reconcile | Differs from repository-local copy; overworld deferred |
| `docs/design/ui/shelter.md` | Reconcile | Differs from repository-local copy |

## Project-level historical archive

All files under `docs/archive/` are Historical. They preserve prior work,
conversation records, handoffs, plans, and the Phase 0 baseline. They must not
direct implementation.

| Path | Classification |
|---|---|
| `docs/archive/Broken_Divinity_Lore_Conversation_Log.md` | Historical |
| `docs/archive/Broken_Divinity_Lore_Tracker_Theology_Angels_Virtues(1).md` | Historical |
| `docs/archive/Broken_Divinity_Lore_Tracker_Validation_Check.md` | Historical |
| `docs/archive/HANDOFF-2026-05-26-ALPHA-CLOSEOUT.md` | Historical |
| `docs/archive/HANDOFF-2026-05-26-AXT-SESSION.md` | Historical |
| `docs/archive/HANDOFF-2026-05-26.md` | Historical |
| `docs/archive/HANDOFF-2026-05-28-1931.md` | Historical |
| `docs/archive/HANDOFF-2026-05-29-1206.md` | Historical |
| `docs/archive/HANDOFF-2026-05-30-1503.md` | Historical |
| `docs/archive/HANDOFF-2026-05-30-PHASE5-7.md` | Historical |
| `docs/archive/PHASE-SHEET-2026-05-29.md` | Historical |
| `docs/archive/PLAN-2026-05-26-ALPHA-READINESS.md` | Historical |
| `docs/archive/PLAN-2026-05-26-UX-TRUST-PASS.md` | Historical |
| `docs/archive/PLAN-2026-05-27-ALPHA-RECOVERY-EXECUTION.md` | Historical |
| `docs/archive/PLAN-2026-05-28-UNIFIED-UI-FULL-MIGRATION-DEV-SWAP.md` | Historical |
| `docs/archive/PLAN-2026-05-28-UNIFIED-UI-INTEGRATION.md` | Historical |
| `docs/archive/PHASE-0-BASELINE-SHA256.md` | Historical baseline manifest |
| `docs/archive/bd-phase0-docs-baseline-2026-07-24.tar.gz` | Historical recoverable baseline |

### Superseded 2026-07-24 foundation plans

| Path | Classification | Replacement |
|---|---|---|
| `docs/archive/foundation-plan-2026-07-24/HISTORICAL-UX-RECOVERY-PLAN.md` | Historical | `docs/FOUNDATION-RECOVERY-PLAN.md` |
| `docs/archive/foundation-plan-2026-07-24/ACTIVE-PLAN-SUPERSEDED.md` | Historical | `docs/FOUNDATION-RECOVERY-PLAN.md` |
| `docs/archive/foundation-plan-2026-07-24/PHASE-3-CONTENT-CONTRACT.md` | Historical | Recovery Phases 1 and 8 |
| `docs/archive/foundation-plan-2026-07-24/PHASE-5-PROGRESSION-CONTRACT.md` | Historical | Recovery Phase 5 |
| `docs/archive/foundation-plan-2026-07-24/PHASE-6-DUNGEON-VERTICAL-SLICE.md` | Historical | Recovery Phases 1, 2, and 4 |
| `docs/archive/foundation-plan-2026-07-24/PHASE-7-COLONY-RETURN-CONTRACT.md` | Historical | Recovery Phases 2, 3, and 6 |
| `docs/archive/foundation-plan-2026-07-24/PHASE-8-FACTION-CONTRACT.md` | Historical | Recovery Phase 5 |
| `docs/archive/foundation-plan-2026-07-24/PHASE-9-PERSISTENCE-CONTRACT.md` | Historical | Recovery Phase 3 |
| `docs/archive/foundation-plan-2026-07-24/PHASE-10-UX-CONTRACT.md` | Historical | Recovery Phase 7 |
| `docs/archive/foundation-plan-2026-07-24/PHASE-10-COMBAT-LOOP-REMEDIATION.md` | Historical | Recovery Phases 1 and 4 |
| `docs/archive/foundation-plan-2026-07-24/PHASE-11-MVP-AUDIT-CONTRACT.md` | Historical failed gate | Recovery Phase 9 |

## Nested repository documentation

The index at `broken-divinity/docs/README.md` governs navigation inside the
nested code repository. All material remains subordinate to project-level
authority.

### Technical reference

| Path | Classification | Notes |
|---|---|---|
| `broken-divinity/docs/README.md` | Active index | Navigation only; not product authority |
| `broken-divinity/docs/ARCHITECTURE_GUARDRAILS.md` | Reference | Subordinate to `Kernel.md` |
| `broken-divinity/docs/DEPENDENCY_MATRIX.md` | Reference | Dependency evidence |
| `broken-divinity/docs/PHASE_EXIT_CRITERIA.md` | Historical | Earlier engineering milestone gates |
| `broken-divinity/docs/playtest-report-2026-04-06.md` | Historical | Earlier playtest evidence |
| `broken-divinity/docs/tech/alpha-risk-register-2026-05-26.md` | Historical | Earlier alpha risk record |
| `broken-divinity/docs/tech/alpha-signoff-report-2026-05-26.md` | Historical | Superseded signoff evidence |
| `broken-divinity/docs/tech/architecture.md` | Reconcile | Differs from project-level design copy |
| `broken-divinity/docs/tech/bevy-brp-smoke-checklist.md` | Reference | Tooling checklist |
| `broken-divinity/docs/tech/qa-log-profiles.md` | Reference | QA logging reference |
| `broken-divinity/docs/tech/ui-design.md` | Reconcile | Differs from project-level design copy |

### Historical repository decisions

These decisions are Historical unless the current decision register explicitly
reaffirms them:

| Path | Classification |
|---|---|
| `broken-divinity/docs/decisions/DecisionLog.md` | Historical |
| `broken-divinity/docs/decisions/2026-07-08-messages-vs-observers.md` | Historical |
| `broken-divinity/docs/decisions/2026-07-08-pathfinding-fov.md` | Historical |
| `broken-divinity/docs/decisions/2026-07-08-procedural-location-v1.md` | Historical; procgen deferred |
| `broken-divinity/docs/decisions/2026-07-09-bd-tactical-mvp.md` | Historical |
| `broken-divinity/docs/decisions/2026-07-09-config-preferences.md` | Historical |
| `broken-divinity/docs/decisions/2026-07-09-data-driven-screens.md` | Historical |
| `broken-divinity/docs/decisions/2026-07-09-outpost-travel-transitions.md` | Historical; full travel deferred |
| `broken-divinity/docs/decisions/2026-07-09-packaging-release.md` | Historical |
| `broken-divinity/docs/decisions/2026-07-09-performance-stability.md` | Historical |
| `broken-divinity/docs/decisions/2026-07-09-roguelike-prototype.md` | Historical; prototype deferred |
| `broken-divinity/docs/decisions/2026-07-09-save-load-replay.md` | Historical |
| `broken-divinity/docs/decisions/2026-07-09-ux-debugging-tooling.md` | Historical |

### Repository gameplay reference

| Path | Classification | Duplicate status |
|---|---|---|
| `broken-divinity/docs/gameplay/README.md` | Reconcile | Differs |
| `broken-divinity/docs/gameplay/colony.md` | Reconcile | Differs |
| `broken-divinity/docs/gameplay/combat.md` | Reconcile | Differs |
| `broken-divinity/docs/gameplay/overworld.md` | Reconcile | Differs; deferred |
| `broken-divinity/docs/gameplay/phase-roadmap.md` | Historical | Differs; not active |
| `broken-divinity/docs/gameplay/procgen.md` | Deprecated duplicate | Identical; project-level reference copy retained |
| `broken-divinity/docs/gameplay/progression.md` | Reconcile | Differs |
| `broken-divinity/docs/gameplay/virtues.md` | Deprecated duplicate | Identical; project-level reference copy retained |

### Repository lore reference

| Path | Classification | Duplicate status |
|---|---|---|
| `broken-divinity/docs/lore/README.md` | Reconcile | Differs |
| `broken-divinity/docs/lore/dungeon-themes.md` | Deprecated duplicate | Identical |
| `broken-divinity/docs/lore/factions.md` | Deprecated duplicate | Identical |
| `broken-divinity/docs/lore/naming-conventions.md` | Deprecated duplicate | Identical |
| `broken-divinity/docs/lore/sanity.md` | Deprecated duplicate | Identical; sanity deferred |
| `broken-divinity/docs/lore/species.md` | Deprecated duplicate | Identical |
| `broken-divinity/docs/lore/thaumaturgy.md` | Deprecated duplicate | Identical |
| `broken-divinity/docs/lore/the-sundering.md` | Deprecated duplicate | Identical |
| `broken-divinity/docs/lore/the-world-now.md` | Deprecated duplicate | Identical |
| `broken-divinity/docs/lore/tone-guide.md` | Deprecated duplicate | Identical |

Deprecated duplicates are preserved during Foundation Recovery. Their named
replacement is the matching path under `docs/design/`.

### Repository UI reference

| Path | Classification | Duplicate status |
|---|---|---|
| `broken-divinity/docs/ui/README.md` | Reconcile | Differs |
| `broken-divinity/docs/ui/ingame.md` | Reconcile | Differs |
| `broken-divinity/docs/ui/menus.md` | Reconcile | Differs |
| `broken-divinity/docs/ui/overworld.md` | Reconcile | Differs; overworld deferred |
| `broken-divinity/docs/ui/shelter.md` | Reconcile | Differs |

### Repository historical archive

| Path | Classification | Replacement |
|---|---|---|
| `broken-divinity/docs/archive/GDD-LEGACY.md` | Historical | Root `GDD.md` |
| `broken-divinity/docs/archive/DEV-PLAN-LEGACY.md` | Historical | `docs/FOUNDATION-RECOVERY-PLAN.md` |

## Other repository-root documentation

These files are outside the two documentation trees but can still confuse
agents. They are classified here for completeness.

| Path | Classification | Notes |
|---|---|---|
| `broken-divinity/README.md` | Active repository guide | Must remain consistent with current controls and authority |
| `broken-divinity/CHANGELOG.md` | Reference | Historical change record |
| `broken-divinity/KNOWN_ISSUES.md` | Historical | 2026-07-11 snapshot; current limitations are in the completed recovery plan |
| `broken-divinity/PLAN-2026-05-26-ALPHA-READINESS.md` | Historical | Not active |
| `broken-divinity/PROTOTYPE_ANALYSIS_REPORT.md` | Historical | Prototype evidence |
| `broken-divinity/UX_PLAYTEST_REPORT.md` | Historical | Earlier UX evidence |
| `broken-divinity/graphify-out/GRAPH_REPORT.md` | Generated reference | Not authority |
| `broken-divinity/metrics/AXT-00-baseline-evidence-2026-05-26.md` | Historical evidence | Not current baseline |
| `broken-divinity/metrics/BX-00-baseline-notes-2026-05-26.md` | Historical evidence | Not current baseline |
| `broken-divinity/testing/COMPREHENSIVE_TEST_REPORT.md` | Historical evidence | Passing claims require rerun |
| `broken-divinity/testing/README.md` | Reference | Test tooling notes |
| `broken-divinity/testing/WEEK2_IMPLEMENTATION_SUMMARY.md` | Historical | Not current completion evidence |

The `.mex/` tree and `.github/copilot-instructions.md` are agent/tooling
artifacts. They are subordinate to the current project authority and must not
override the recovery plan. The vendored `tools/graphify/` documentation is
third-party/tool documentation and is outside Broken Divinity product scope.

## Duplicate reconciliation result

The following paired files are byte-identical:

- `gameplay/procgen.md`;
- `gameplay/virtues.md`;
- `lore/dungeon-themes.md`;
- `lore/factions.md`;
- `lore/naming-conventions.md`;
- `lore/sanity.md`;
- `lore/species.md`;
- `lore/thaumaturgy.md`;
- `lore/the-sundering.md`;
- `lore/the-world-now.md`;
- `lore/tone-guide.md`.

All other paired gameplay, lore index, technical, and UI files differ. They
remain Reconcile until their unique content is compared against the root GDD.

## Ownership rule

- Product changes update the root GDD.
- Locked choices update `DECISIONS-TO-LOCK.md`.
- Technical architecture changes update `Kernel.md`.
- Completed Foundation stabilization and acceptance evidence lives in
  `FOUNDATION-STABILIZATION-PLAN.md`.
- `FOUNDATION-RECOVERY-PLAN.md` remains the completed chronological recovery
  record and receives only cross-links or post-acceptance corrections.
- Current Foundation test and colony UX execution is owned only by
  `FOUNDATION-TEST-AND-UX-HARDENING-PLAN.md`.
- New Product P2 execution still requires a separate owner-approved plan.
- Reference or lore material does not become active through editing alone; it
  must be promoted explicitly through the authority chain.
