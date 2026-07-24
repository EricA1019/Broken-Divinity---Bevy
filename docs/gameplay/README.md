# Gameplay Documentation

> **RECONCILE — NOT PRODUCT AUTHORITY**
> This tree differs from the project-level design reference. Current scope
> comes from the root [GDD](../../../GDD.md) and
> [Foundation Recovery Plan](../../../docs/FOUNDATION-RECOVERY-PLAN.md).

Detailed gameplay systems for Broken Divinity, organized by domain. Each file clearly separates **MVP** scope from **Phase 2** (Colony Foundation) and **Phase 3** (Full Colony).

## Reading Order

| # | File | Covers |
|---|------|--------|
| 1 | [phase-roadmap.md](phase-roadmap.md) | **Read first.** MVP → Phase 2 → Phase 3 feature breakdown, dependencies, what's explicitly deferred |
| 2 | [combat.md](combat.md) | d100 skill checks, cover system, action budget, damage, armor durability, status effects, abilities |
| 3 | [colony.md](colony.md) | Walkable shelter, stations, survivors, resources, raids, defense |
| 4 | [overworld.md](overworld.md) | Node graph, tile walking, weather, encounters, hell/heaven zones |
| 5 | [procgen.md](procgen.md) | Deterministic seeding, dungeon BSP, overworld gen, faction gen, long-term history sim |
| 6 | [progression.md](progression.md) | Skills, XP curves, perks, gear tiers, faction gates |

## Development Phases (Summary)

### MVP — "The Bones"
The complete game loop at minimum depth. Every major system present, some shallow. The player should feel the tone, the tension, and the core combat/colony rhythm.

### Phase 2 — Colony Foundation
Deepen economy, crafting, and survivor identity. Factions become mechanically meaningful. More dungeon themes. The colony starts to feel alive.

### Phase 3 — Full Colony
RimWorld-depth colony management. Song of Syx proc-gen history. Living overworld. Full stealth system. The long-term vision.

See [phase-roadmap.md](phase-roadmap.md) for the full breakdown.

## Cross-References

| Need | File |
|------|------|
| Lore, worldbuilding, factions | [docs/lore/](../lore/README.md) |
| High-level game design overview | [root GDD](../../../GDD.md) |
| Combat formulas (implementation-level) | [gameplay-mechanics skill](not in docs — Copilot skill) |
| Colony system implementation | [colony-management skill](not in docs — Copilot skill) |
| Procgen implementation | [procgen skill](not in docs — Copilot skill) |
