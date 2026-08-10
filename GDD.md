# Broken Divinity Product Design Authority

**Status:** Current working product direction; unresolved choices are tracked in [docs/authority/DECISIONS-TO-LOCK.md](docs/authority/DECISIONS-TO-LOCK.md).

**Working Guide**

**Last Updated:** 2026-07-24

**Project:** Broken Divinity

**Purpose:** Define what Broken Divinity is, what the player should experience, and which product scope is intended. This document is for product direction, not technical implementation.

---

## 1. Game Statement

Broken Divinity is a survival game about living inside the ruins of sacred legitimacy.

The player holds together a fragile shelter, travels a hostile overworld, enters dangerous dungeons, survives weighty tactical combat, and returns with what they can carry. Around that practical loop sits a world of broken theology, inherited violence, rival sacred powers, and conflicting truths.

The game is not generic dark fantasy with religious flavor layered on top. Theology is part of the world’s material structure. It shapes law, legitimacy, corruption, faction identity, and what kinds of futures can still be built.

At its core, Broken Divinity is about survival, legitimacy, and inheritance after sacred collapse.

---

## 2. Design Pillars

### Survival Under Unresolved Theology

The player is not primarily a prophet or chosen theologian. The player is a survivor trying to keep people alive and make practical decisions inside a world where those decisions always carry theological weight.

### No Morally Simple Powers

No major power should collapse into a clean heroic or villainous role.

- Yahweh is not secretly just evil.
- Demons are not secretly innocent.
- Angels are not rigid fools.
- Humans are not innocent bystanders.
- The old gods were not all monsters, and they were not all benign.

### Inheritance Over Reset

The setting is about what remains after an order breaks. The player is not starting from a blank slate. They inherit law, trauma, ruins, names, debts, and damaged forms of protection.

### Theology Must Matter in Systems

Theology should not stay in codex entries or dialogue. It should shape progression, faction response, corruption, sanity, settlement decisions, and the meaning of survival.

### Preparation, Pressure, and Consequence

Broken Divinity should feel tense, deliberate, and costly. Combat, travel, and colony management should all reinforce the idea that bad preparation, bad judgment, and bad luck can spiral quickly.

---

## 3. Core Player Experience

The intended player experience is:

1. Maintain a shelter and the people inside it.
2. Prepare for travel and decide what risks are worth taking.
3. Move through the overworld where time, weather, and encounters create pressure.
4. Enter dungeons that contain danger, loot, sacred contamination, and lore.
5. Fight only as much as necessary and survive tactical encounters that can always turn.
6. Extract, return, and stabilize the shelter with the gains.
7. Decide what powers, bargains, laws, and truths are tolerable.

The player should feel like a pragmatic builder under theological pressure, not a detached lore tourist.

---

## 4. World Foundations

### Yahweh

Yahweh is the central unresolved figure in the setting and is best framed as Father, Conqueror, King, Judge, and Wound.

He protected, formed, disciplined, and gave law. He also conquered, displaced, renamed, and built sacred order on violence that later could not simply be erased.

The setting should preserve the tension that necessary does not mean clean, and victory does not mean innocence.

### Angels

Angels are sacred intelligences built around command, office, hierarchy, witness, ritual, and protocol. They are the surviving servants of a sacred order that sought lawful coherence and universal legitimacy.

Their defining crisis is not mere sadness or loss of faith. It is the breakdown of a command architecture that no longer resolves the world cleanly.

Three major responses define the surviving angelic world:

- The Broken Choir: corrupted sacred order still trying to obey forms that no longer resolve.
- The Puritans: angels and allies who insist the last valid commands still bind and the war remains lawful.
- The Wanderers: angels who build new local protocols around witnessed righteousness without abandoning angelic structure.

Michael is the clearest Puritan answer. Gabriel is the clearest Wanderer answer.

### Demons

Demons are not generic infernal monsters. They are the older gods, spirits, sacred powers, and divine remnants of worlds that were conquered, subordinated, renamed, exiled, fragmented, or warped under later sacred order.

"Demon" remains the default umbrella term, but the setting should always remember that the word is not innocent. It is the name victorious sacred order placed on defeated powers.

The demonic umbrella includes regional gods, city gods, household powers, storm and river spirits, fertility and wilderness powers, underworld judges, oath and plague spirits, grave powers, and primordial beings of sea or chaos.

If angels are beings of command, office, and protocol, demons are beings of place, reciprocity, domain, and name.

### Humans and Thaumaturgy

Humans are the interpretive species. They endure partly because they do not belong completely to any one sacred order. They survive through incompleteness, rebuilding, reinterpretation, compromise, and dangerous improvisation.

Thaumaturgy is the human response to sacred instability through knowledge, tools, and manipulation. It is neither angelic power nor demonic power. It is a specifically human form of intervention in a broken sacred world.

---

## 5. Factions and Narrative Structure

Factions should not merely organize combat encounters. They should embody rival answers to legitimacy, survival, truth, purity, law, reciprocity, and inheritance.

For the initial MVP foundation, factions are represented by two random placeholder factions loaded from data. They exist to prove faction ownership, hostility, encounter identity, and extensibility without prematurely locking the final canon.

The final named roster is intentionally deferred. Michael's Host, older powers, occultist groups, military remnants, and survivor groups may become major factions later, but none is required to define the first foundation slice.

Gabriel should function as companion, philosophical counterpoint, and witness-gatherer, with investigation as the spine of their narrative role.

Narrative truth should be layered and contradictory. No single faction should own the whole truth. Different factions preserve different pieces, distort different pieces, and silence different pieces.

The investigation layer should currently be treated as an optional side spine: persistent, meaningful, and thematically central, but still easier to defer than the basic survival loop. Its strongest hook should be access to lore fragments, faction truth, and competing witness accounts rather than being the main source of direct power escalation.

---

## 6. Core Gameplay Structure

### Combat

Combat is one of the game's highest-priority systems and should feel weighty.

The guiding ideas are:

- preparation matters more than reflex alone
- every encounter can spiral
- ammo, medicine, and armor constrain aggression
- information is tactical advantage
- death is always on the table

The current gameplay direction supports a tactical d100-based system with cover, damage types, armor durability, action budgets, and a small set of high-value abilities.

### Shelter and Colony

The shelter is the other core loop. It is a physical place, not just an abstract menu.

The player's home should currently be treated as a settlement that begins as a practical survivor shelter but can grow into a more ideologically charged civic project depending on play, alliances, laws, and tolerated powers.

The player should eventually walk the shelter, place stations, manage resources, assign survivors, and deal with raids as direct threats to continuity. For the initial foundation, shelter management, resources, assignments, stations, and production are in scope; raids and events come later.

At first, the colony can remain thin and functional, but it should already feel like something being held together under pressure.

The settlement should also be able to evolve in broad ideological directions without forcing a single ordained endpoint. The strongest current guide-level paths are:

- pragmatic survivor haven
- Puritan lawful refuge
- mixed coexistence settlement
- thaumaturgic enclave
- demon-bargained protectorate
- militarized fortress remnant

Settlement-scale theology should eventually shape more than narrative flavor. These are post-MVP directions:

- law and civic restrictions
- faction trust and diplomatic access
- sanity and corruption pressure
- who can live in the settlement safely
- raid type and defensive support
- station bonuses and production priorities

### Overworld Travel

The overworld is the connective tissue between shelter and dungeon.

Travel should cost time, food, water, and safety. Weather, encounters, and uncertainty should make the journey itself part of the game rather than a blank transition layer.

### Dungeons

Dungeons are where loot, danger, anomalies, sacred contamination, and lore fragments concentrate.

They should not just be combat spaces. They should be places where the world's broken theology becomes materially dangerous and discoverable.

### Sanity — Deferred

Sanity is part of the long-term design but is deferred while the core dungeon and colony mechanics are established. No sanity track or sanity-driven content is required for the initial MVP foundation.

---

## 7. Virtues and Progression

The player's deeper identity should be expressed through virtues rather than generic fantasy attributes.

The six core virtues are:

- Temperance
- Justice
- Prudence
- Fortitude
- Thumos
- Metis

Kleos functions separately as earned mythic significance.

Virtues should be the root layer of progression. They should express how the player resists corruption, interprets duty, plans under pressure, survives suffering, acts decisively, or lives through cunning.

Concrete proficiencies may remain, but only where they represent practical training rather than abstract human qualities already covered by virtues.

The safest retained examples at the current design level are:

- melee training
- ranged training
- stealth or quiet movement
- repair and technical work
- first aid or medicine
- craft or ritual technique

This is a hybrid direction, not an additive one. Skills improve through actions, use, and training. Actions and choices reflect virtues, giving practical skill growth a meaningful behavioral context. Exact mappings and balance are deferred until the core action loop is stable.

---

## 8. Current Scope Anchors

The initial MVP is a functioning foundation rather than the complete final game. The kernel and shell are locked down first, then the game expands through deeper dungeon, colony, overworld, faction, and theological systems.

### MVP Foundation

The initial foundation should deliver:

- a functional Bevy-Ratatui/Ratatui shell
- core kernel functions and stable action flow
- a fixed, hand-authored dungeon with movement, exploration, encounters, combat, loot, and extraction
- the complete first dungeon loop: enter, explore, fight, collect, leave, and return the result to the colony
- basic colony/shelter state, survivor assignment, stations, resources, and production
- two data-driven placeholder factions
- reusable content loading and validation
- tests and a buildable foundation that can expand without replacing the core

Deferred from this foundation: procedural dungeon generation, procedural
shelter-map topology, overworld generation, raids, colony events, sanity,
theology-driven mechanics, the full overworld loop, faction reputation, final
faction canon, and deeper narrative integration. Deterministic placement of
data-defined resource fixtures on the existing fixed shelter map is part of
the basic colony foundation; it does not activate broader procedural
generation.

### Product P2

Product P2 should deepen colony identity, faction response, additional dungeon theming, sanity, and a stronger narrative layer.

This is where contradictory testimony, faction trust, and richer faction-specific meaning can begin to matter mechanically.

### Product P3

Product P3 is where the larger living-world vision can fully expand: deeper colony management, living overworld systems, historical generation, and broader faction complexity.

That later depth should still remain consistent with the survival-first, theology-shaped foundation.

---

## 9. Current Design Constraints

- Do not drift back into generic dark fantasy with religious names pasted on top.
- Do not reduce theology to flavor once theology-driven systems enter implementation.
- Do not make the player a detached investigator with no practical survival burden.
- Do not flatten angels, demons, or humans into single moral roles.
- Do not let renamed generic attributes undermine the purpose of the virtue system.
- Do not let lore sophistication erase the need for clean, playable loops.

---

## 10. Open Questions

The following issues are not yet fully locked and should guide the next design questions.

### Core action-to-virtue mapping

The hybrid progression direction is locked, but the exact virtue effects, skill growth rates, and action mappings remain open.

### Minimum colony foundation

The foundation retains the current basic shelter, survivor, station, resource, assignment, and production structures where their behavior is sound. Raids and events are explicitly deferred.

The basic colony loop must be physical and readable rather than a set of
unrelated daily counters. The player can move a paused build preview
independently of the player character, place a station at the selected legal
coordinate, and assign a named survivor to a named station-backed recipe.
Resource fixtures are placed deterministically from the run seed on the fixed
shelter map. A worker travels to a matching fixture, gathers one raw unit,
carries it to a compatible station, and refines it into a visible
data-defined placeholder result. This first loop proves the reusable
foundation; deeper logistics, queues, upgrades, depletion balance, and colony
automation remain later work.

Gathering and refining are visible turn-based work rather than instant
adjacency rewards. Recipe data defines the positive number of worker turns
required and the amount produced when that work completes; partial work never
creates partial resources. For the Foundation balance profile, gathering takes
three work turns and refining takes two work turns.

The simple survivor gathering tasks use the same readable worker-tick
language. A survivor travels to the matching source, arrival grants nothing,
and three later adjacent work turns produce one configured colony resource.
Day advancement does not grant a second legacy gathering result. This direct
path preserves emergency recovery without requiring a processing station;
station-backed recipes remain the separate gather-carry-refine path.

Placing a buildable station creates a paid construction site rather than an
immediately usable station. Station data defines its positive construction
work requirement; the Foundation profile uses four work turns. Genuinely idle
survivors automatically travel to reachable construction sites and contribute
one work unit per accepted Outpost worker tick. Construction never takes over
an assigned, resting, defending, or production worker. A completed site
becomes the selected station exactly once.

### Minimum dungeon foundation

The foundation dungeon is fixed rather than procedural. The minimum player-visible loop is: enter, move and explore, fight at least one enemy type, collect loot, reach an exit, leave, and apply the result to the colony/run state.

### Placeholder faction schema

The foundation uses two data-driven placeholder factions with stable IDs and minimal display, identity, and hostility fields. Richer faction effects and reputation remain open for later planning.

### Kernel MVP exit gate

The active plan defines “functional shell,” “core kernel functions,” and “basic mechanics” as executable tests and player-visible behavior rather than module existence alone.

---

## Closing Statement

Broken Divinity should feel like a game where the player survives inside the ruins of sacred order, fights for continuity without certainty, and slowly decides what kind of future can be built from inherited law, inherited violence, inherited names, and inherited wounds.
