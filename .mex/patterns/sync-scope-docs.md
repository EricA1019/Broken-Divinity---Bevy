---
name: sync-scope-docs
description: "Reconciling scope drift between the roadmap, detailed gameplay docs, and the MVP execution plan."
triggers:
  - "sync roadmap"
  - "fix doc contradictions"
  - "scope drift"
  - "align dev plan"
  - "roadmap review"
edges:
  - target: context/architecture.md
    condition: when changes alter state flow, tiers, or feature sequencing
  - target: context/conventions.md
    condition: when verifying the updated docs and follow-on implementation plan
last_updated: 2026-04-06
---

# Sync Scope Docs

## Context

Load `docs/gameplay/phase-roadmap.md` first. Then load the detailed gameplay docs that the contradictory section points to (`combat.md`, `colony.md`, `overworld.md`, `progression.md`, `procgen.md`). If an execution plan exists, load `docs/dev-plan.md` too.

## Steps

1. **Identify the source-of-truth conflict** — list each contradiction explicitly, with the master-scope doc on one side and the detailed/system doc on the other.
2. **Choose the winning design** — decide whether the roadmap is stale or the detailed doc is over-scoped. Do not leave both versions partially true.
3. **Patch the roadmap first** — update MVP / Phase 2 / Phase 3 boundaries so the roadmap is the clean master scope again.
4. **Patch the detailed docs second** — update system-level docs so terms, thresholds, triggers, and narrative beats match the roadmap.
5. **Patch the execution plan third** — move work between slices if the corrected scope changes sequencing or prerequisites.
6. **Sync MEX** — update `ROUTER.md` current state and any stale `context/` files that still describe the old scope.

## Gotchas

- Do not fix only one side of a contradiction; roadmap, detailed doc, and plan must all agree.
- Watch for hidden knock-on changes: narrative gates, AppState names, save/load sequencing, and phase tags tend to drift together.
- If a feature moved phases, update any "explicitly not in MVP" or dependency-chain sections too.
- Avoid vague success criteria. Convert them into observable completion checks when you touch the roadmap.

## Verify

- [ ] Roadmap and detailed gameplay docs agree on what ships in MVP
- [ ] Phase tags are consistent across roadmap and system docs
- [ ] Execution plan slices no longer implement deferred features early or required features too late
- [ ] Narrative beats occur in the same place across roadmap, progression, overworld, and dev plan
- [ ] MEX context no longer repeats the stale assumptions that were just fixed

*** Add File: /home/eric/Projects/Broken Divinity --Bevy/.cargo/config.toml
[build]
# Placeholder config file so MEX setup references resolve.

*** Add File: /home/eric/Projects/Broken Divinity --Bevy/native/assets/data/rosters.ron
// Placeholder RON data file for MVP data scaffolding.
()

*** Add File: /home/eric/Projects/Broken Divinity --Bevy/.mex/sync.sh
#!/usr/bin/env sh

npx promexeus sync "$@"

*** Add File: /home/eric/Projects/Broken Divinity --Bevy/src/core/components.rs
//! Shared ECS components will live here as the MVP slices are implemented.

*** Add File: /home/eric/Projects/Broken Divinity --Bevy/src/core/resources.rs
//! Shared ECS resources will live here as the MVP slices are implemented.