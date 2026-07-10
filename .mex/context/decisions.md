---
name: decisions
description: Key architectural and technical decisions with reasoning. Load when making design choices or understanding why something is built a certain way.
triggers:
  - "why do we"
  - "why is it"
  - "decision"
  - "alternative"
  - "we chose"
edges:
  - target: context/architecture.md
    condition: when a decision relates to system structure
  - target: context/stack.md
    condition: when a decision relates to technology choice
last_updated: 2026-04-05
---

# Decisions

## Decision Log

### Messages API instead of Bevy Events
**Date:** 2026-04-05
**Status:** Active
**Decision:** All inter-system communication uses Bevy 0.18 Messages (`#[derive(Message)]`, `MessageWriter<T>`, `Messages<T>`) — never the legacy Events API.
**Reasoning:** Bevy 0.18 introduced Messages as the replacement for Events. Using the new API avoids deprecation churn and aligns with upstream direction.
**Alternatives considered:** Bevy Events (`EventWriter<T>`, `EventReader<T>`) — rejected because deprecated in 0.18.
**Consequences:** All cross-system communication patterns use `messages.drain()` for reading and `events.write()` for sending. Cannot use any code examples from Bevy <= 0.17 directly.

### Two AI Behaviors instead of Role-Based AI
**Date:** 2026-04-05
**Status:** Active
**Decision:** Enemies have exactly two behaviors — MeleeCharge and RangedKite — combined with affixes for variety, rather than many specialized AI roles.
**Reasoning:** Small role set is easier to balance and test. Affixes (Armored, Fast, Pack, Explosive, Regenerating, etc.) provide combinatorial variety without multiplying behavior code.
**Alternatives considered:** 6+ AI roles (Brute, Sniper, Flanker, etc.) — rejected because each role needed unique behavior trees with heavy maintenance and testing burden.
**Consequences:** New enemy variety comes from affixes and stat tuning, not new behavior implementations. The ai.rs module stays compact.

### Hybrid ASCII + 16×16 Sprites instead of Pure ASCII
**Date:** 2026-04-05
**Status:** Active
**Decision:** Rendering uses ASCII glyphs for dungeon tiles/walls with 16×16 pixel sprites for entities (player, enemies, items, NPCs).
**Reasoning:** Pure ASCII limits visual identity. Sprites for entities give personality and readability while keeping the traditional roguelike map feel.
**Alternatives considered:** Pure ASCII (rejected — lacks character), full tile-based (rejected — loses roguelike aesthetic and increases art burden).
**Consequences:** Need sprite sheets for entity types. Map rendering is glyph-based. Two rendering paths coexist.

### Dual Sanity System (Raid + Long-Term)
**Date:** 2026-04-05
**Status:** Active
**Decision:** Two separate sanity mechanics: Raid Exposure (per-dungeon ticking clock) and Long-Term Erosion (cumulative across runs).
**Reasoning:** Raid Exposure creates per-run tension ("get out before you lose it"), Long-Term Erosion creates campaign-level dread and recovery gameplay. Together they provide pressure at both time scales.
**Alternatives considered:** Single sanity meter (rejected — conflates short-term tactical pressure with long-term campaign stakes).
**Consequences:** Two separate drain systems, two separate recovery mechanisms. Hallucination spawning triggers from the faster-draining raid meter.

### OnceLock Data Loading instead of Bevy AssetServer
**Date:** 2026-04-05
**Status:** Active
**Decision:** Game data (rosters, dialogue, item catalogs) loads from RON files via `std::sync::OnceLock` at first access, not through Bevy's AssetServer.
**Reasoning:** RON data files are small, static, and loaded once. OnceLock is simpler than managing Bevy's async asset loading pipeline for data that never changes at runtime.
**Alternatives considered:** Bevy AssetServer (rejected — overkill for static data, adds async complexity), compile-time includes (rejected — forces recompile for data tweaks).
**Consequences:** Data files are loaded synchronously on first access. No hot-reload for data during development (requires restart). Data structs need `Deserialize` but not `Asset`.

### d100 Skill Check System instead of d20 or Percentile
**Date:** 2026-04-05
**Status:** Active
**Decision:** Combat and skill checks use a d100 roll-under system where `target_number = skill_level + 25 + modifiers − target_dv`.
**Reasoning:** d100 gives granular probability control. The +25 base offset means even untrained characters attempt checks. Criticals trigger when roll ≤ raw skill level AND the check succeeds.
**Alternatives considered:** d20 (rejected — too coarse, modifiers dominate quickly), 2d6 (rejected — bell curve makes modifiers nonlinear to reason about).
**Consequences:** All combat formulas, ability checks, and crafting use this single unified system. Modifier stacking is transparent to players (1 point = 1% change).
