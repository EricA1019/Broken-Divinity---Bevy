# Progression

Progression in Broken Divinity operates on multiple axes: player character skills, perks, equipment, station upgrades, and narrative gates. At MVP, progression is **shallow but present** — enough to feel growth without complex builds.

---

## Player Character Skills

### Skill System

The player character uses a set of skills (`SkillId`) that improve through use. Skills govern success chances for the d100 system (see [combat.md](combat.md)).

### MVP Skills

| Skill | Governs | Starting Value |
|-------|---------|---------------|
| **Melee** | Melee attack rolls | 40 |
| **Ranged** | Ranged attack rolls | 35 |
| **Evasion** | Dodge chance when targeted | 30 |
| **Toughness** | Damage reduction, bleed resistance | 25 |
| **Stealth** | LOS avoidance (simplified at MVP) | 20 |
| **Awareness** | Enemy detection range, trap detection | 25 |
| **Repair** | Equipment repair speed/quality at Workbench | 30 |
| **Leadership** | Survivor work efficiency bonus (small) | 20 |

### Skill Advancement

Skills improve through **use**, not XP allocation:

| Trigger | XP Gained |
|---------|-----------|
| Successful skill check | +3 XP |
| Failed skill check | +1 XP (you learn from failure) |
| Critical success (roll ≤ skill / 5) | +5 XP |

*Note: XP Grinding has diminishing returns. Continually grinding weak enemies or low-risk actions will eventually yield 0 XP.*

### Level-Up Formula

Quadratic scaling: XP required for level N = `50 × N²`

| Level | Total XP Required | Cumulative XP |
|-------|-------------------|---------------|
| 1 → 2 | 50 | 50 |
| 2 → 3 | 200 | 250 |
| 3 → 4 | 450 | 700 |
| 4 → 5 | 800 | 1,500 |
| 5 → 6 | 1,250 | 2,750 |

Each skill level adds **+2 to the skill value** used in d100 checks (e.g., Ranged 35 + 3 levels = 41 effective).

### Skill Cap
MVP skill cap: **level 10** per skill (effective value = base + 20). This keeps the d100 system in a meaningful range where failure is always possible.

---

## Perks

### MVP: 3 Tiers, Level-Gated

Perks are passive bonuses unlocked by reaching skill level thresholds. Each skill has its own perk tree (small at MVP).

### Tier Gates

| Tier | Skill Level Required | Perks Available |
|------|---------------------|-----------------|
| T1 | 2 | 2 choices (pick 1) |
| T2 | 5 | 2 choices (pick 1) |
| T3 | 9 | 1 (capstone) |

### MVP Perk Examples

**Melee**
| Tier | Option A | Option B |
|------|----------|----------|
| T1 | **Heavy Swing**: +15% melee damage | **Quick Strike**: Melee costs 1 fewer AP |
| T2 | **Cleave**: Melee hits adjacent enemy for 50% damage | **Riposte**: Counter-attack on melee dodge |
| T3 | **Execution**: Instant kill on enemies below 15% HP | — |

**Ranged**
| Tier | Option A | Option B |
|------|----------|----------|
| T1 | **Steady Aim**: +10% ranged accuracy | **Quick Draw**: First shot each combat costs 0 AP |
| T2 | **Piercing Shot**: Ignore 50% armor | **Suppressing Fire**: Target loses 2 AP next turn |
| T3 | **Deadeye**: Headshots (called shots) available | — |

**Toughness**
| Tier | Option A | Option B |
|------|----------|----------|
| T1 | **Thick Skin**: +2 flat damage reduction | **Endurance**: +15 max HP |
| T2 | **Iron Will**: Bleed/stun duration −1 turn | **Second Wind**: Auto-heal 10 HP once per combat when below 25% |
| T3 | **Unkillable**: Survive lethal hit once per dungeon (1 HP) | — |

The remaining skills (Evasion, Stealth, Awareness, Repair, Leadership) follow the same pattern but are designed in Phase 2 when those systems deepen.

### Phase 2 Perk Additions
- Full perk trees for all 8 skills
- Cross-skill perks (e.g., Melee + Stealth = "Ambush Strike")
- Survivor perks (background traits unlock perk-like bonuses)

---

## Equipment

### MVP: Tiered Gear with Binary Durability

Equipment is the primary loot progression. Better gear = better survival.

### Equipment Slots

| Slot | Affects |
|------|---------|
| **Weapon (main)** | Damage, attack type, range |
| **Armor (body)** | Armor rating (AR), damage reduction |
| **Accessory** | Passive bonus (Phase 2: thaumic equipment) |

Accessory slot exists in data but has minimal content at MVP (maybe 2-3 items).

### Weapon Properties

| Property | Description |
|----------|-------------|
| Damage | Base damage value |
| Damage type | Physical, Ballistic (see combat.md) |
| Range | Melee (1 tile) or Ranged (line of sight) |
| AP cost | Action points to use |
| Ammo | Ranged weapons consume ammo (universal at MVP) |
| Accuracy modifier | Bonus/penalty to hit roll |

### Weapon Tiers (MVP)

| Tier | Examples | Damage Range | Found In |
|------|----------|-------------|----------|
| 0 - Improvised | Pipe, shiv, sling | 4-8 | Starting gear, junk loot |
| 1 - Common | Machete, pistol, shotgun | 8-15 | Early dungeon floors |
| 2 - Military | Combat knife, rifle, SMG | 15-22 | Military theme, deep floors |
| 3 - Rare | Named weapons (proc-gen names) | 20-30 | Boss drops, vault rooms |

### Armor

Armor uses a **binary durability** system (see [combat.md](combat.md)):

| State | Effect |
|-------|--------|
| **Intact** | Full armor rating (AR) applies to damage reduction |
| **Broken** | AR = 0. Must be repaired at Workbench. |

Armor breaks when cumulative damage absorbed exceeds its durability threshold.

### Armor Tiers (MVP)

| Tier | AR | Durability | Found In |
|------|-----|-----------|----------|
| 0 - Scavenged | 2 | 15 | Starting, common loot |
| 1 - Reinforced | 4 | 25 | Dungeon mid-floors |
| 2 - Military | 6 | 40 | Military theme, deep floors |
| 3 - Plated | 8 | 60 | Rare drops |

### Repair

Repair requires Workbench + scrap + time. The Repair skill affects speed:
- Base repair time: 3 shelter turns per armor piece
- Repair skill bonus: −1 turn per 3 skill levels (min 1)

### Phase 2 Equipment Additions
- Thaumic equipment (supernatural bonuses + sanity cost)
- Ammo types (AP, incendiary, hollow-point — each with properties)
- Weapon mods (scopes, grips, bayonets)
- Survivor loadouts (equip survivors from shelter inventory)

### Phase 3 Equipment Additions
- Caliber system (weapons require specific ammo calibers)
- Legendary items (unique effect + lore)
- Crafting from blueprint + materials (not just looting)

---

## Station Upgrades

### Research Table Progression

The Research Table is the tech gate for station upgrades. It uses a simple unlock tree at MVP.

### Unlock Tree (MVP)

```
Research Table T1 (available from start)
  ├── Station Upgrade: Workbench T2
  ├── Station Upgrade: CookingStation T2
  ├── Station Upgrade: WaterPurifier T2
  ├── Station Upgrade: Generator T2
  └── Station Unlock: AmmoPress (if not yet built)

Research Table T2 (requires scrap + time)
  ├── Station Upgrade: all remaining stations to T2
  └── Shelter Expansion: larger rooms, reinforced walls
```

T3 research is Phase 2+ content.

### Research Costs

| Research | Scrap Cost | Time (shelter turns) |
|----------|-----------|---------------------|
| T2 station upgrade | 20 | 30 |
| Research Table → T2 | 40 | 50 |

A survivor assigned to the Research Table works through the research queue. No multi-track research at MVP.

---

## Narrative Gates

### MVP: Gabriel Encounter

One mandatory narrative encounter anchors the MVP narrative hook:

| Gate | Trigger | Unlocks |
|------|---------|---------|
| **Gabriel's Warning** | Scripted room on floor 2 of the first dungeon | Gabriel joins as the first companion and reveals the Michael's Host threat |

This is minimal narrative — a scripted event with dialogue, not a branching questline. It exists to anchor the first dungeon run with a concrete story beat and reveal the angelic threat early.

### Phase 2 Narrative Additions
- Faction questlines (each faction offers 2-3 missions)
- Decision points (aid or betray factions, with reputation consequences)
- Dungeon objectives tied to faction requests

---

## Progression Curve

### MVP Intended Pacing

| Game Phase | Timeframe (approx) | Player State |
|------------|-------------------|--------------|
| **Early** | First 2-3 dungeons | T0-T1 gear, skills 1-2, shelter basic, 3 survivors, Gabriel encounter |
| **Mid** | Dungeons 3-5 | T1-T2 gear, skills 2-4, T1 perks, first station upgrades |
| **Late** | Dungeons 5-8+ | T2-T3 gear, skills 4-6, T2 perks, harder raids and deeper dungeons |
| **Endgame** | Deep-run campaign state | T3 gear, skills 6+, T3 perks, hardest dungeons, faction conflicts |

### Difficulty Scaling

Progression should feel like:
1. **Scraping by** — every resource matters, every fight is scary
2. **Getting a foothold** — shelter feels stable, gear makes a difference
3. **Dangerous confidence** — you're stronger but so are the enemies
4. **Endgame tension** — powerful but the world is closing in (factions, harder raids, deeper dungeons)

The game should NEVER reach a "I'm so powerful nothing threatens me" state. Permadeath ensures stakes remain real, and enemy scaling keeps pressure on.

---

## Survivor Progression

### MVP: Minimal Growth

Survivors don't meaningfully "level up" at MVP. Their background trait gives them a slight efficiency bonus at one station type (e.g., "Former Medic" = +15% MedicalBay output). That's it.

### Phase 2 Survivor Additions
- Skills that grow through work (assigned to Workbench → Repair skill improves)
- Background traits expand (2-3 traits per survivor)
- Combat readiness from MilitiaTraining station (survivors fight better in raids)

### Phase 3 Survivor Additions
- Full skill trees per survivor
- Personality and mood affecting work output
- Specialist roles (doctor, engineer, soldier — trained at stations)

---

## Sanity Progression

### MVP: Single-Track Bar

Sanity is a 0-100 **Raid Exposure** bar where **0 = calm** and **100 = broken**.

| Threshold | Value | Effect |
|-----------|-------|--------|
| Stable | 0-29 | No effect |
| Stressed | 30-59 | Minor visual glitches, occasional unreliable game log messages |
| Unstable | 60-84 | Hallucinated enemies on map (disappear when approached), stat display jitter |
| Critical | 85-100 | Input lag, false threat indicators, heavy visual distortion |

### Sanity Drain Sources
- Combat events (taking hits, kills, supernatural enemy abilities)
- Thaumic exposure (anomaly tiles, thaumic equipment)
- Ashfall weather (overworld)
- Witnessing death (survivor or enemy)

### Sanity Recovery & Resets
- Returning to the shelter resets Raid Exposure to 0
- Rare consumables or special events can reduce exposure during a run
- Combat and anomalies increase exposure; they do not restore it

### Phase 2 Sanity Additions
- Dual-track system (Stress + Corruption as separate bars)
- Corruption from thaumic equipment (doesn't recover naturally)
- Mental break events at Critical thresholds

See also: [combat.md](combat.md) for skill checks in combat, [colony.md](colony.md) for station upgrade paths, [overworld.md](overworld.md) for sanity drain during travel, [phase-roadmap.md](phase-roadmap.md) for progression scope per phase.
