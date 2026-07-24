# Repository Documentation Index

**Status:** Technical/reference documentation subordinate to the project-level documentation hub.

Current authority lives outside this nested repository:

1. [Product GDD](../../GDD.md)
2. [Technical architecture](../../Kernel.md)
3. [Locked decisions](../../docs/DECISIONS-TO-LOCK.md)
4. [Foundation MVP scenario](../../docs/MVP-SCENARIO.md)
5. [Foundation Recovery Plan](../../docs/FOUNDATION-RECOVERY-PLAN.md)

Nothing in this directory overrides those documents.

## Active reference

- `ARCHITECTURE_GUARDRAILS.md` — earlier technical guardrails; use only where consistent with `Kernel.md`.
- `DEPENDENCY_MATRIX.md` — dependency record.
- `PHASE_EXIT_CRITERIA.md` — historical engineering milestone criteria.
- `decisions/` — historical technical decision records; a record is active only if reaffirmed by current authority.
- `tech/` — technical reference and prior audit material.
- `playtest-report-2026-04-06.md` — historical playtest evidence.

## Reconcile

These trees overlap with project-level `docs/design/` and may contain unique differences:

- `gameplay/`
- `lore/`
- `ui/`

They are reference material, not current product authority. No implementation
requirement may be taken from them without confirming it in the root GDD and
the active recovery plan.

## Historical archive

- `archive/GDD-LEGACY.md` — superseded game design document.
- `archive/DEV-PLAN-LEGACY.md` — superseded vertical-slice plan.

## Deferred-system warning

Documents describing procgen, overworld travel, raids, events, sanity,
reputation, Gabriel, final factions, or deeper narrative concern future work
unless the active recovery plan explicitly brings them into scope.
