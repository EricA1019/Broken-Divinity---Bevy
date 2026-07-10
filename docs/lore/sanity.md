# Sanity

Sanity in Broken Divinity is a **dual-track system** — two separate meters that erode through different mechanisms and produce different consequences. Sanity is not a health bar. It is the **cost of knowledge**.

## The Two Tracks

### Raid Exposure (Short-Term)

Accumulated during dungeon runs by encountering supernatural phenomena. Resets (partially) when you return to shelter.

**Sources of raid exposure:**
- Encountering angelic entities (awe/compulsion pressure)
- Encountering demonic entities (temptation/paranoia pressure)
- Witnessing Veil phenomena (thin spots, dimensional bleed, reality warping)
- Using thaumic equipment (operator exposure)
- Reading certain texts or interacting with anomalous objects
- Ally death or critical injury

**Effects at threshold levels:**
| Level | Name | Effect |
|-------|------|--------|
| 0-25% | Steady | No effect |
| 25-50% | Unsettled | Minor perception penalties, occasional false UI signals |
| 50-75% | Shaken | Stat penalties, unreliable perception, involuntary reactions |
| 75-90% | Breaking | Severe penalties, hallucinations, control loss (character acts without input) |
| 90-100% | Broken | Run ends. Character must be evacuated or is lost. |

**Recovery:**
- Partial reset on returning to shelter (drops to ~25% of accumulated)
- Shelter facilities can improve recovery rate
- Some items provide field reduction (at a cost)
- Rest, food, and social interaction at shelter reduce the residual

### Long-Term Erosion (Permanent)

A separate track that accumulates over the entire playthrough. **Never fully resets.** This is the price of surviving in the post-Sundering world.

**Sources of long-term erosion:**
- Repeated raid exposure (each run leaves a residual)
- Traumatic narrative events (major story beats, companion death)
- Prolonged thaumic equipment use
- Specific faction interactions (Infernal bargains, Puritan rituals, Occultist experiments)
- Discovery of deep lore (the truth has a cost)

**Effects at threshold levels:**
| Level | Name | Effect |
|-------|------|--------|
| 0-20% | Intact | No effect |
| 20-40% | Weathered | Subtle personality shifts, new dialogue options (darker, more desperate) |
| 40-60% | Scarred | Permanent minor stat changes, unlocks risky abilities, some NPCs react differently |
| 60-80% | Fractured | Significant personality shifts, some actions become unavailable (too traumatized), others unlock (nothing left to lose) |
| 80-100% | Hollowed | Endgame territory. Character is fundamentally changed. Unique endings, abilities, and interactions. Not "dead" — transformed. |

**Mitigation (not recovery):**
- Long-term erosion cannot be reversed, only slowed
- Companion relationships slow accumulation
- Shelter upgrades provide buffers
- Some choices accelerate it (Infernal bargains, forced Veil exposure)
- Some choices slow it (avoiding deep lore, staying at shelter, refusing power)

## Design Intent

### Why Two Tracks?

**Raid exposure** creates **tactical pressure** during dungeons. It's a resource to manage — push deeper and risk break, or retreat and lose progress. It makes every room a risk/reward decision.

**Long-term erosion** creates **narrative pressure** across the playthrough. The character you started with is not the character you end with. Knowledge changes you. Power costs something. The player who digs deepest into the Sundering mystery pays the highest price.

### The Cost of Knowledge

This is the thematic core: **learning the truth erodes your sanity**. The factions that know the most (Occultists, deep Puritans) are also the most damaged. The player who pursues the investigative endgame (discover the Sundering truth) will have a high-erosion character. The player who stays on the surface (sandbox survival) keeps their sanity but never learns what happened.

This is a deliberate tradeoff, not a punishment. High-erosion characters **unlock content** that low-erosion characters never see.

### Species Interaction

| Species | Raid Exposure | Long-Term Erosion |
|---------|--------------|-------------------|
| Human | Standard | Standard |
| Angelic | Resistant to angelic sources, vulnerable to infernal | Slower accumulation but harder to mitigate once gained |
| Demonic | Resistant to infernal sources, vulnerable to angelic | Faster accumulation but more "functional" at high levels |

## Sanity and UI

Sanity is communicated through **unreliable narrator** mechanics:

- At low exposure: UI is clean and accurate
- At medium exposure: Subtle distortions — minimap flickers, enemy counts may be off by one, item descriptions shift slightly
- At high exposure: Significant distortions — false enemies on minimap, stat displays fluctuate, messages in the log that didn't happen
- At breaking: The UI actively lies — false item pickups, phantom allies, distorted map

The player should never be 100% certain whether what they're seeing is real. This is the horror.

See also: [species.md](species.md) for species-specific sanity interactions, [thaumaturgy.md](thaumaturgy.md) for thaumic exposure, [the-world-now.md](the-world-now.md) for ambient Veil effects.
