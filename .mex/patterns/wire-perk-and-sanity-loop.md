---
name: wire-perk-and-sanity-loop
description: Pattern for wiring combat XP, deterministic perk unlock popups, passive perk effects, and sanity threshold behaviors into the active dungeon loop.
last_updated: 2026-04-06
---

# Task: Wire Perk And Sanity Loop

## Use When
- Combat actions should grant skill XP and unlock perks at thresholds.
- A perk popup must pause normal input until the player claims the unlock.
- Sanity thresholds need to affect gameplay instead of only logging flavor text.

## Steps
1. Add a non-persistent queue resource for pending perk unlocks near the perk domain, not in menu/UI code.
2. Award XP at the combat resolution point, not in input handling. Use the actual target's combat skill for diminishing returns.
3. When a skill levels up, enqueue only newly available perks that are not already unlocked.
4. Draw the unlock popup in egui from the queue resource and mutate the player's `PlayerPerks` only when the user confirms the unlock.
5. Block movement and other combat input while a perk popup is pending so the popup is a real gate, not cosmetic UI.
6. Apply passive perk effects where the numbers matter: outgoing damage, ranged accuracy, incoming armor, free reloads, sanity reduction, and emergency healing.
7. Hook sanity events at the actual runtime sources: hits taken, kills, anomalies, and hazards.
8. Implement sanity thresholds as behavior changes, not just messages: combat penalties at stressed+, hallucination spawns at shaken+, and movement override at breaking.
9. Reset raid exposure when the player re-enters shelter from a runtime snapshot so dungeon stress does not leak through colony recovery.

## Gotchas
- The current perk data is linear per skill tier, not branchy. The popup is an unlock/claim modal, not a real choose-one-of-many tree unless the data model changes.
- Overworld does not keep a live player entity, so any pending perk queue should be treated as ephemeral UI state and not relied on for persistence.
- If hallucinations use the `Enemy` marker so the player can target them, exclude them from real enemy AI and enemy attack queries.
- Register hallucination spawning on the turn loop, not every render frame, or repeated spawns become nondeterministic noise.

## Verify
- `cargo build -p broken_divinity`
- `cargo test -p broken_divinity`
- Confirm movement and ranged reload/shoot input stop while a perk popup is visible.
- Confirm entering Colony from a runtime snapshot clears raid exposure.