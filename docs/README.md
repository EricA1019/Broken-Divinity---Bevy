# Broken Divinity Documentation Hub

**Updated:** 2026-08-06

## Directory Structure

```
docs/
├── README.md              # This file — navigation hub
├── active/                # Current implementation plans (the ones you execute)
│   ├── FOUNDATION-TEST-AND-UX-HARDENING-PLAN.md
│   ├── FOUNDATION-BASIC-COLONY-LOOP-PLAN.md
│   └── FOUNDATION-UI-IMPROVEMENT-PLAN.md
├── authority/             # Locked decisions, guardrails, testing standards
│   ├── DECISIONS-TO-LOCK.md
│   ├── ARCHITECTURE_GUARDRAILS.md
│   ├── AUTHORITATIVE-TESTING-STANDARD-AND-MIGRATION-PLAN.md
│   ├── DEPENDENCY_MATRIX.md
│   ├── DOCUMENT-INVENTORY.md
│   ├── MVP-SCENARIO.md
│   ├── MIGRATION-AND-DEPRECATION.md
│   └── WORKSPACE-HYGIENE-PLAN.md
├── reference/             # Game design, lore, status reports, mockups
│   ├── GAME-STATUS-2026-08-01.md
│   ├── PHASE_EXIT_CRITERIA.md
│   ├── FOUNDATION-UI-STYLE-MOCKUPS.md
│   ├── UX_PLAYTEST_REPORT.md
│   ├── playtest-report-2026-04-06.md
│   ├── lore/              # Canonical worldbuilding
│   ├── gameplay/          # Mechanics design specs
│   └── mockups/           # Visual mockups
├── handoff/               # Active UI9-C implementation handoff artifacts
│   ├── UI9-C-CONTEXT-CANDIDATE-HANDOFF-PROMPT.md
│   ├── UI9-C-CONTEXT-CANDIDATE-HANDOFF-PROMPT-v2.md
│   ├── UI9-C-CONTEXT-CANDIDATE-HANDOFF-BODY-v3.md
│   └── UI9-C-CONTEXT-CANDIDATE-HANDOFF-BODY-v4.md
├── decisions/             # Historical technical decision log
├── tech/                  # Archived technical references (Bevy 0.14 era)
├── archive/               # Completed plans, retired docs, legacy artifacts
└── bug-reports/           # Bug report templates and logs
```

## Authority Order

1. [`GDD.md`](../GDD.md) — sole product and player-experience authority
2. [`authority/DECISIONS-TO-LOCK.md`](authority/DECISIONS-TO-LOCK.md) — locked product choices
3. [`Kernel.md`](../Kernel.md) — technical architecture authority
4. [`Kernel-direction.md`](../Kernel-direction.md) — subordinate technical appendix
5. [`authority/MIGRATION-AND-DEPRECATION.md`](authority/MIGRATION-AND-DEPRECATION.md) — preservation policy
6. [`authority/MVP-SCENARIO.md`](authority/MVP-SCENARIO.md) — canonical Foundation acceptance scenario
7. [`authority/AUTHORITATIVE-TESTING-STANDARD-AND-MIGRATION-PLAN.md`](authority/AUTHORITATIVE-TESTING-STANDARD-AND-MIGRATION-PLAN.md) — active testing policy, evidence, metrics, suite-migration authority
8. [`active/FOUNDATION-TEST-AND-UX-HARDENING-PLAN.md`](active/FOUNDATION-TEST-AND-UX-HARDENING-PLAN.md) — active Foundation behavior and colony UX implementation authority
9. [`active/FOUNDATION-BASIC-COLONY-LOOP-PLAN.md`](active/FOUNDATION-BASIC-COLONY-LOOP-PLAN.md) — owner-approved D-20 colony-loop implementation
10. [`active/FOUNDATION-UI-IMPROVEMENT-PLAN.md`](active/FOUNDATION-UI-IMPROVEMENT-PLAN.md) — Foundation Ratatui improvement sequence and UI phase gates
11. [`authority/DOCUMENT-INVENTORY.md`](authority/DOCUMENT-INVENTORY.md) — classification and ownership of all project documentation

**Completed plans** are in [`archive/`](archive/). They are evidence records,
not active authority. Do not execute from them.

## Required Reading for an Implementation Agent

1. This hub
2. Root `GDD.md` and `Kernel.md`
3. `authority/DECISIONS-TO-LOCK.md`
4. `authority/MVP-SCENARIO.md`
5. `authority/AUTHORITATIVE-TESTING-STANDARD-AND-MIGRATION-PLAN.md`
6. The relevant active plan(s) in `active/`
7. `authority/DOCUMENT-INVENTORY.md` when consulting any other document
8. Current code and tests

## Stop Rule

Stop and ask the project owner when:
- canonical documents conflict
- a product choice is missing
- work would activate a deferred system
- preserving existing work would materially expand scope
- a proposed implementation creates another source of truth

Do not resolve ambiguity by selecting an older plan or design copy.
