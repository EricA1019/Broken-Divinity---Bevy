# Combat

Combat is Broken Divinity's highest-priority system. It must feel **weighty** — every encounter is a risk/reward calculation where preparation matters more than reflexes. The battle is won or lost before the first shot.

## Design Philosophy

### Benchmarks
- **XCOM**: Position, cover, odds displayed, risk/reward per shot. You know the percentages. You accept the gamble.
- **Caves of Qud**: Fast turns, deep systemic interactions, surprising outcomes. Progression from dying to a dog → fighting hordes → facing bosses. But even endgame, carelessness kills.
- **Quasimorph**: Metal, over-the-top feel. The player starts to feel powerful — then gets punished for cockiness. Overconfidence is the slow killer.

### Core Principles
1. **Preparation wins fights.** The right gear, the right position, the right ability at the right time. Entering a room blind should feel reckless.
2. **Every encounter can spiral.** One missed shot, one unlucky status proc, one extra enemy from a noise alert — and a controlled situation becomes desperate.
3. **Resources constrain aggression.** Ammo is finite. Meds are finite. Sanity drains. Armor degrades. You can't fight everything.
4. **Information is tactical advantage.** Knowing what's in the next room (scouting), knowing enemy positions (LOS), knowing your odds (cover display) — all are power.
5. **Death is always on the table.** Full permadeath at MVP. Every fight could be the last. This makes every encounter tense, even against weak enemies.

---

## Skill Check System

### d100 Resolution
All combat actions that target an entity use the same d100 roll:

```
target_number = skill_level + 25 + modifiers − target_dv

success:  roll (1–100) ≤ target_number
critical: roll ≤ (raw_skill_level / 5)
fumble:   roll = 100 (always miss, potential equipment jam/break)
```

### Modifiers (cumulative)

| Source | Modifier | Phase |
|--------|----------|-------|
| Half cover (target) | −20 | **MVP** |
| Full cover (target) | −40 | **MVP** |
| Range penalty | −2 per tile beyond optimal range | **MVP** |
| Sanity penalty | −5 at 50%, −15 at 25%, −30 at 10% | **MVP** |
| Starvation penalty | −5 (hungry), −15 (starving), −30 (critical) | **MVP** |
| Perk bonus | Varies by perk | **MVP** (skeleton) |
| Flanking bonus | +15 if target has no cover on your side | Phase 2 |
| Stealth bonus | Auto-crit from Unaware | Phase 2 |
| Height advantage | +10 from elevated position | Phase 3 |

### Why d100?
Granular enough for meaningful modifier stacking. A +5 bonus matters. XCOM-style — the player can calculate their approximate odds before committing.

---

## Cover System

### MVP: Environmental Cover

Cover comes from the environment — walls, pillars, furniture, barricades. No "cover tiles" as a distinct tile type; instead, adjacency to blocking objects grants cover.

| Cover Level | To-Hit Penalty | Source Examples |
|-------------|---------------|----------------|
| **None** | 0 | Open floor, no adjacent obstacles on attacker's side |
| **Half** | −20% | Adjacent to a wall segment on attacker's side, behind low furniture |
| **Full** | −40% | Behind a corner/pillar (only one tile of exposure), behind barricades |

### How Cover Is Determined
1. Draw a line from attacker to target.
2. Check tiles adjacent to the target on the side facing the attacker.
3. If any adjacent tile on that line is a wall/obstacle: half cover.
4. If the target is at a corner (wall on two sides flanking the line of fire): full cover.
5. If no obstacles adjacent on that axis: no cover.

### Cover Display
The UI shows cover status for the currently targeted enemy before the player commits to a shot. XCOM-style: the player should **never be surprised** by a cover modifier.

### Phase 2: Flanking
When the attacker approaches from a side with no cover, the target gets no cover bonus and the attacker gets +15 flanking. This rewards positioning and multi-entity tactics.

### Phase 3: Concealment vs Cover
Cover (blocks bullets) and concealment (hides from detection) become separate systems. Fog, darkness, and foliage provide concealment but not cover.

---

## Damage System

### Damage Formula

```
base_damage  = weapon_base + (skill_level / 4) − target_ar   (min 1)
               (target_ar drops to 0 if armor is broken)
variance     = ±20% random
crit_mult    = 2.0× (if critical hit)
magic_reduc  = −(target_md / 2)  if damage_type is Celestial, Infernal, or Thaumic

final_damage = max(1, base_damage × variance × crit_mult − magic_reduc)
```

### Damage Types (6)

**Physical** (resisted by AR — Armor Rating):

| Type | Sources | Noise | Notes |
|------|---------|-------|-------|
| Ballistic | Guns, crossbows | Loud | Strongest per-hit, limited by ammo |
| Slash | Blades, claws | Quiet | Moderate damage, good status infliction |
| Blunt | Hammers, fists | Quiet | Lower damage, higher stun chance |

**Supernatural** (resisted by MD — Magic Defense):

| Type | Faction Source | Visual | Notes |
|------|---------------|--------|-------|
| Celestial | Angelic / Puritan | Cold pale gold, white glow | Sanity pressure on use |
| Infernal | Demonic / Infernal | Deep crimson, sickly orange | Sanity pressure on contact |
| Thaumic | Human thaumaturgy | Electric blue, unstable violet | Equipment can overload |

---

## Armor Durability

### MVP: Binary Armor System

Armor is either **working** (provides full AR) or **broken** (provides 0 AR). No gradual degradation. Armor durability loss is based on **actual damage taken** (post-armor reduction), not a flat rate per hit.

```
on_hit:
  armor.durability -= damage_dealt
  if armor.durability <= 0:
      armor.broken = true
      armor.ar_active = 0
      // "Your armor breaks!" in game log
```

**Repair**: Only at the shelter Workbench. Costs scrap. Takes turns.

### Why Binary?
- Simple to understand: "Is my armor broken? Yes/No."
- Creates clear decision moments: "I took a big hit — is it worth retreating to save my armor, or pushing forward with degraded protection?"
- Armor repair at shelter drives the return-to-base loop.

### Phase 2: Gradual Degradation
AR degrades proportionally with durability loss. Armor becomes less effective over a fight, not just on/off.

---

## Action Budget & Speed

### Turn Structure
Each entity gets `speed` actions per round (integer, typically 1-3). Higher speed = more actions = more dangerous.

### Action Costs

| Action | Cost | Notes |
|--------|------|-------|
| Move (1 tile) | 1 action | Standard movement |
| Melee Attack | 1 action | Bump into adjacent enemy |
| Shoot | 1 action + 1 ammo | Explicit targeting, generates loud noise |
| Reload | 1 action | Transfer ammo from inventory to clip |
| First Aid | 1 action + 1 med | Self-heal, scales with skill |
| Sprint | 1 action | Move 2 tiles in 1 action |
| Wait | 1 action | Skip, pass initiative |
| Use item | 1 action | Consumables, interactables |

*Note: Sprint incurs a 3-turn cooldown.*

### Round Flow
1. All entities start with `remaining = speed`
2. Entities act in speed-descending order (ties broken by initiative roll at round start)
3. Each action decrements `remaining` by its cost
4. When `remaining = 0`, entity is done for the round
5. When ALL entities exhausted → **WorldTurn phase** → tick status effects, drain sanity, consume needs, reset budgets

### Speed Values

| Entity | Typical Speed | Notes |
|--------|--------------|-------|
| Player | 2 | Base speed, modifiable by gear/perks |
| Human enemy | 1-2 | Raiders, scavengers |
| Lesser demon | 2 | Fast, aggressive |
| Greater demon | 2-3 | Multiple actions per round — dangerous |
| Angelic sentinel | 1 | Slow but high damage/AR |

---

## Status Effects

### MVP: 2 Effects

| Effect | Per-Turn Damage | Duration | Special | Sources |
|--------|----------------|----------|---------|---------|
| **Wounded** | 3 + hp.max/10 | 3-5 turns | DoT, stacks up to 3 | Weapon % on hit, hazards, enemy abilities |
| **Stunned** | 0 | 1 turn | Skip next action | Blunt weapon %, enemy abilities, traps |

### Infliction
Three source types:
1. **Weapon property**: Each weapon has optional `(StatusKind, chance%)`. On hit, roll chance.
2. **Abilities**: Specific abilities apply status as part of their effect.
3. **Environment**: Hazard tiles (fire, acid, stun traps).

### Phase 2 Additions
- **Burning** (fire DoT, prevents regeneration)
- **Bleeding** (bleed DoT, movement worsens it)
- **Frightened** (movement penalty, accuracy penalty, may flee)
- **Corrupted** (infernal contamination, escalating damage if not cleansed)

### Phase 3 Additions
- **Compelled** (celestial control, character acts against player input)
- **Mutated** (thaumic contamination, stat changes, visual changes)
- Status effect interactions (burning + bleeding = cauterized, corruption + compelled = ???)

---

## Abilities

### MVP: 4 Core Abilities

Every player character starts with these. They cannot be lost.

| Ability | Type | Cost | Range | Noise | Effect |
|---------|------|------|-------|-------|--------|
| **Attack** | Melee | 1 action | Adjacent | Quiet | Bump-to-attack, melee damage |
| **Shoot** | Ranged | 1 action + 1 ammo | Weapon range | Loud | Explicit targeting, ranged damage |
| **First Aid** | Utility | 1 action + 1 med | Self | Quiet | Heal `skill_level + 10` HP |
| **Sprint** | Movement | 1 action | Self | Quiet | Move 2 tiles instead of 1 (3-turn cooldown) |

### Ability Acquisition
MVP: All 4 are innate. No ability learning.
Phase 2: New abilities unlocked by skill level thresholds and perks.
Phase 3: Faction-specific abilities unlocked by reputation.

---

## Ammo System

### MVP: Universal Ammo

One ammo type for all ranged weapons. Simple, clear.

- **Clip size**: Weapon-specific (pistol 6, rifle 8, shotgun 4, etc.)
- **Reload**: 1 action, transfers ammo from inventory pool to clip
- **Empty clip**: Cannot shoot. Must reload.
- **Ammo sources**: Loot, shelter AmmoPress station, scavenging

### Phase 2: Ammo Types
- Light (pistols, SMGs)
- Heavy (rifles, shotguns)
- Energy (thaumic weapons)

### Phase 3: Caliber System
Per-weapon caliber. Scarcity becomes a real constraint. Trading for compatible ammo drives faction interaction.

---

## Noise & Detection

### MVP: Simplified

- Ranged attacks are **Loud** (enemies in range become aware)
- Melee attacks are **Quiet** (only adjacent enemies react)
- All enemies detect by **LOS only** — no facing direction, no FOV cones
- Enemies either see you or don't. Binary.

### Phase 2: Full Stealth System
See [phase-roadmap.md](phase-roadmap.md) for the complete stealth expansion: facing, FOV cones, alert states, noise propagation ranges.

---

## Combat Encounter Design

### Difficulty Progression (MVP)

| Stage | Enemies | Character | Tension Source |
|-------|---------|-----------|---------------|
| Early game | 1-2 weak (feral dogs, lone raider) | Low skills, poor gear, limited ammo | Everything is a threat. One dog can kill you. |
| Mid game | 2-4 mixed (raiders with ranged, lesser demons) | Moderate skills, decent armor, some abilities | Numbers and positioning matter. Ammo management. |
| Late game | 3-6 varied (mixed factions, greater demons) | High skills, good gear, action budget | Combined arms. Supernatural damage. Sanity pressure. |

### Encounter Principles
1. **Encounters should be avoidable.** The smart player routes around fights they can't win.
2. **Running is valid.** Stairs are an escape. Doors can be closed. Retreat is a tactic.
3. **Noise creates cascading danger.** A loud fight attracts more enemies (Phase 2 stealth makes this mechanical; MVP, it's designed into enemy placement).
4. **Resource cost is the real damage.** Winning a fight but spending 6 ammo and 2 meds means you can't afford the next one.
5. **Sanity is the hidden timer.** Every fight near supernatural enemies drains sanity. You can "win" every combat and still lose the run.

### Shelter Raid Combat

Raids use the same combat system as dungeons, played on the shelter tilemap:

1. **Raid warning**: Event triggers (faction hostility threshold, shelter visibility too high)
2. **Transition screen**: Player gets a preparation pause — reassign survivor presets (flee, defend, hold position), check stockpiles, equip gear
3. **Combat begins**: Turn-based. Player controls their character only. Survivors act autonomously based on their preset.
4. **Raider behavior**: Enter from shelter perimeter. Target food stores and stations. Will fight defenders who block them.
5. **Resolution**: Raiders retreat when losses exceed threshold (50%+), or all defeated, or they grab enough loot and flee.

### Survivor Combat Presets (MVP)

| Preset | Behavior | When to Use |
|--------|----------|-------------|
| **Flee** | Run to interior, hide | Low-combat survivors, protect valuable workers |
| **Defend** | Hold position at assigned station, fight if engaged | Station guards, sentry duty |
| **Support** | Follow player at range, assist when player engages | Ranged-capable survivors |
| **Hold Gate** | Position at shelter entrance, block raider advance | Strong melee survivors |

Survivors are **never directly commandable**. The player influences them through preset assignment, not turn-by-turn orders. This creates a planning layer — the right presets win raids; wrong presets cause casualties.

## Inventory Limits

Inventory uses a **slot-based limit only** (e.g. 20 slots). Items stack or consume slots, creating resource scarcity decisions during exploration. Encumbrance or weight limits are not used.

See also: [colony.md](colony.md) for survivor system details, [progression.md](progression.md) for skill/perk effects on combat, [phase-roadmap.md](phase-roadmap.md) for Phase 2/3 combat additions.
