# Factions

## Architecture

Factions in Broken Divinity are a **mix of hardcoded archetypes and proc-gen groups**, inspired by Caves of Qud. The world feels different every run because most factions are generated, but narrative anchors (hardcoded factions, always-generated characters) provide consistency.

## Five Archetypes

Every faction — hardcoded or proc-gen — belongs to one of five archetypes. The archetype determines magic affinity, general disposition, and naming conventions.

| Archetype | Magic Affinity | Style | Enemy Naming | NPC Naming |
|-----------|---------------|-------|--------------|------------|
| **Puritan/Angel** | Celestial | Theocratic, authoritarian, angel-backed | Religious titles, corrupted saints | Biblical/Hebrew names, ecclesiastical titles |
| **Infernal/Demon** | Infernal | Corruptors, invaders, hierarchical by power | Visceral, body-horror | Akkadian/Sumerian names, harsh consonants |
| **Thaumic/Occultist** | Thaumic | Knowledge seekers, equipment-based magic | Scientific terms twisted | Chosen symbolic names, academic titles |
| **Conventional/Military** | None | Disciplined remnants, conventional arms | Military designations gone wrong | Rank + surname |
| **Independent/Survivor** | None (scavenged) | Pragmatic settlement-builders | Mundane gone feral | Normal human names |

## Hardcoded Factions

These five factions are **always present** in every generated world. They anchor the narrative and provide reliable points of reference.

### Survivors (Player Faction)
- **Archetype**: Independent/Survivor
- **Identity**: Pragmatic scavengers and builders. No ideology beyond survival.
- **Organization**: The player's settlement + loose trade networks with other independent groups.
- **Magic**: None innate. May use scavenged thaumic tech or recruit Occultist allies.
- **Internal tension**: Disagreement over whether to engage with supernatural factions or isolate.
- **Narrative role**: The player's home. The lens through which the world is experienced.

### Michael's Host
- **Archetype**: Puritan/Angel
- **Identity**: Militant theocratic order led by the Archangel Michael. The strongest Puritan faction.
- **Doctrine**: The Sundering was divine judgment. Thaumaturgy is heresy. Angels are divine messengers. Demons must be purged.
- **Organization**: Hierarchical, cathedral-fortresses, anointed warriors.
- **Magic**: Celestial — channeled through faith, prayer, and sanctified relics. Practitioners are called Anointed.
- **Relationship to angels**: The Ascended Host (angelic beings) empowers Michael's Host, but their true agenda may differ from Church doctrine. It's symbiosis, not charity — angels need human faith as an anchor in the material world.
- **Enemy of**: All Infernal factions, all Thaumic factions (heretics). Tolerates Survivors and Military as potential converts.
- **Narrative role**: Primary source of Celestial-side information about The Sundering. Their version of the truth blames human sin and Occultist experimentation.

### The Court of Irkalla
- **Archetype**: Infernal/Demon
- **Identity**: A demon court rooted in Sumerian underworld mythology. An active invasion force working to widen the Veil permanently.
- **Organization**: Hierarchical by power — bestial imps at the bottom, cunning lords at the top. The court has internal politics, rivalries, and factions within the faction.
- **Magic**: Infernal — innate, manifests as corruption, fire, flesh-warping, and psychic assault.
- **Goals**: Widen the Veil, establish permanent infernal territory in the material world, corrupt or enslave humans.
- **Relationship to other demons**: Not all demon factions serve Irkalla. Proc-gen Infernal factions may be rivals, splinter groups, or independents.
- **Narrative role**: Primary source of Infernal-side information about The Sundering. Their version claims demons were invited or that the Veil was always meant to fall.

### The Lethean Circle
- **Archetype**: Thaumic/Occultist
- **Identity**: A coven of thaumaturges who have stabilized a rift and use it as a power source and research site. "Lethean" references the river of forgetting — they believe humanity must remember what thaumaturgy truly is.
- **Organization**: Nomadic outer ring (field researchers) + fortified inner lab (the stabilized rift). Apprenticeship chains.
- **Magic**: Thaumic — requires equipment (rift stabilizers, focusing lenses, conduit arrays) and training. Not innate.
- **Danger**: Their experiments are genuinely risky. The stabilized rift could fail. Some members become corrupted.
- **Enemy of**: Puritan factions (who call them heretics). Wary alliance with Survivors when interests align.
- **Narrative role**: Primary source of thaumic/scientific information about The Sundering. Their version blames a specific pre-Sundering experiment — and they may know more than they're telling.

### Fort Pershing Garrison
- **Archetype**: Conventional/Military
- **Identity**: Surviving fragment of pre-Sundering military forces. Chain of command preserved. Discipline and protocol over ideology.
- **Organization**: Bunker-based, hierarchical, resource-hoarding. Multiple outposts connected by radio (one of the few reliable communication networks).
- **Magic**: None. Conventional arms, fortification, discipline. Suspicious of all supernatural forces.
- **Relationship**: Allied with Survivors out of necessity. Wary cooperation with Puritans (shared interest in order, disagreement on supernatural). Hostile to all Infernal factions. Deeply suspicious of Occultists.
- **Narrative role**: Source of pre-Sundering military records, bunker archives, and classified documents. Their version of the truth is fragmented — they have data but don't understand the supernatural context.

## Proc-Gen Factions

At world generation, **10-20 additional factions** are seeded. Their traits are randomized within archetype constraints. During play, events can spawn new factions or destroy existing ones.

### Generated Traits

Each proc-gen faction has the following traits (Caves of Qud-depth):

| Trait | Description | Generation Rule |
|-------|-------------|----------------|
| **Name** | Faction name following archetype naming patterns | Generated from archetype-specific word pools |
| **Archetype** | One of the five archetypes | Weighted random — Independent most common, Puritan/Infernal least |
| **Alignment** | Hostile / Suspicious / Neutral / Friendly / Allied (toward player) | Starts based on archetype; shifts via gameplay |
| **Territory** | Overworld nodes they control (1-5) | Placed during world-gen; can expand/contract |
| **Magic Affinity** | Celestial / Infernal / Thaumic / None | Determined by archetype |
| **Leader** | Named NPC with personality | Generated: name, personality trait (aggressive/diplomatic/paranoid/zealous/scholarly/mercantile) |
| **Doctrine** | What they believe and enforce | Text template filled from archetype + randomized elements |
| **Preferred Enemies** | Which other factions/archetypes they actively oppose | 1-2 enemy archetypes or specific factions |
| **Trade Goods** | What they produce and what they want | Resource pair from economy system |
| **Strength** | Military/magical power level (1-5) | Determines raid difficulty, trade leverage |
| **Version of The Sundering** | Their narrative about what happened | Template based on archetype + randomized details |

### Faction Name Generation

Names are built from archetype-specific word pools:

| Archetype | Pattern | Examples |
|-----------|---------|----------|
| Puritan/Angel | The [Saint/Virtue] + [Order/Congregation/Host] | The Mercy Congregation, The Elijah Order, The Seraphic Watch |
| Infernal/Demon | The [Title] of [Sumerian word] | The Teeth of Namtar, The Asag Dominion, The Utukku Pact |
| Thaumic/Occultist | The [Concept] + [Circle/Archive/Conduit] | The Resonance Archive, The Fracture Circle, The Null Conduit |
| Conventional/Military | [Location/Rank] + [Unit type] | Camp Ridgeline Militia, Colonel Hart's Brigade, The Blackwood Regulars |
| Independent/Survivor | [Place] + [Descriptor] | Bridgetown Collective, The Quarry Folk, Ashfield Traders |

### Faction Events

During play, the following events can affect factions:

- **Faction founded**: A settlement grows strong enough to declare independence or a charismatic leader gathers followers
- **Faction destroyed**: Overrun by enemies, internal collapse, leader death
- **Faction split**: Internal disagreement creates two smaller factions
- **Alliance formed**: Two factions with shared enemies ally
- **War declared**: Faction hostility escalates to open conflict
- **Leader change**: New leader shifts doctrine or alignment

## Reputation System

Reputation is tracked **per-faction**, ranging from:

```
Hostile → Suspicious → Neutral → Friendly → Allied
```

### How Reputation Changes
- **Positive**: Completing faction quests, trading, defending faction territory, killing their enemies, returning their people
- **Negative**: Attacking faction members, raiding faction territory, allying with their enemies, failing faction quests
- **Indirect**: Actions that help/harm an archetype affect reputation with all factions of that archetype (smaller magnitude)

### What Reputation Unlocks

| Level | Unlocks |
|-------|---------|
| Suspicious | Can approach without being attacked. Basic trade. |
| Neutral | Standard trade catalog. Safe passage through territory. |
| Friendly | Full trade catalog. NPC recruits. Side quests. |
| Allied | Faction-gated narrative truths. Elite recruits. Joint operations. Endgame quest access. |

See also: [naming-conventions.md](naming-conventions.md) for faction naming patterns, [the-sundering.md](the-sundering.md) for faction narratives about The Sundering.
