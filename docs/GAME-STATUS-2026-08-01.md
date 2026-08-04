# Game Status Report — Broken Divinity

**Date**: 2026-08-01
**Build**: `bd_app` (Bevy 0.18 + Ratatui terminal TUI), Kernel v0.1.0, workspace `broken-divinity`
**Method**: Live real-terminal playtest (drove the game via keypresses through the full loop) + automated suite + content validation
**Toolchain**: `cargo +stable` (rustc 1.97.0) — see Environment Notes
**Evidence**: `/memories/session/playtest-2026-08-01.md` (session notes), this report

---

## 1. Summary

The game runs cleanly and the **entire Foundation vertical slice is functional and
live-verified**: title screen → shelter colony (build, staffing, day cycle, production
economy) → travel → dungeon exploration (movement, FOV, combat, loot, item use) →
extraction → save/load round-trip. No crashes, panics, or dead flows were observed.

- **642 automated tests pass** across 35 suites, 0 failures.
- Content validation passes: `1 dungeon, 1 item, 4 skills, 2 factions, 4 actions,
  6 stations, 3 blueprints`.
- The colony production economy is real: a staffed Stove's +3 Supplies/day exactly
  offsets the −3 food/day consumption.
- Save/load persistence works: F5 writes to `~/.local/share/broken-divinity/saves/manual-slot.ron`;
  F9 restores the exact state (verified Day 1 / Supplies 7 after quit + relaunch).

This is the Foundation Build scope: a single fixed 12×8 dungeon, one enemy type
(Rat), one usable item (Healing Potion), one buildable station tier, and placeholder
faction/virtue hooks. Product P2 content is intentionally not present.

---

## 2. Automated Health

| Metric | Result |
|--------|--------|
| `cargo build -p bd_app` | ✅ Clean |
| `cargo test --workspace` | ✅ 642 passed, 0 failed, 35 green suites |
| Headless validation (`bd --validate`) | ✅ Content validation PASSED |
| Clean quit / startup | ✅ `Broken Divinity Kernel exited cleanly` every run |

The 642-test count matches the current CHANGELOG claim. Contract registry owns
88 contracts per the changelog; no contract is Red (automated layers green;
PTY/owner review steps remain open per the contract evidence workflow).

---

## 3. What's Working (live-verified)

### 3.1 Startup & Shell
- Title screen with wordmark, "Press any key to begin", and `SaveAvailability`
  (correctly shows *Load unavailable — No save* on a fresh install, and flips once
  a save exists).
- Content + config load from built-in defaults; clean shutdown on `q`.

### 3.2 Shelter Colony (Outpost)
- **Screen layout**: Party panel, Map, Stats, Actions, Log — all render at the
  supported terminal profiles.
- **Movement** (`wasd`/arrows) with wall-blocking ("Blocked.") and AP/turn economy.
- **Build** (`b`): station selection → ghost placement → construction site
  ("0/4 work") → Supplies cost deducted (10→8) → auto-completes on next rest.
- **Day cycle** (`n` Rest): day advances, food −3/day consumed, DailySummary line
  ("Day 1: Supplies 10→7 (−3); Materials 0→0 …").
- **Station staffing** (`e`): Survivor→Station wizard; assigned Survivor 1 to the
  Stove; worker glyph (`*`) appears on the station; panel reflects the assignment.
- **Production economy**: staffed Stove produced +3 Supplies/day on Day 2,
  netting 0 (5→5) against food consumption — the colony loop closes.
- **Survivors**: 3 spawn idle (Mood 100), daily projection and "Next worker"
  forecast shown in the Party panel.
- **Log** captures every action and daily summary.

### 3.3 Travel & Dungeon
- **Travel** (`t`): commits supplies ("You commit supplies and enter the ruin"),
  mode transitions Outpost → Travel → Dungeon.
- **Dungeon**: fixed 12×8 data-defined map (walls, door, extraction), FOV
  updates as the player moves.
- **Combat** (`f`): "Quick attack! Rat takes 3/5 damage", enemy retaliation,
  skill XP ("Melee 1→3; Thumos +1"), defeat handling ("Rat is defeated!") with
  despawn.
- **Items**: pickup (`p`), use (`u`) — Healing Potion heals 8 HP (capped at max),
  grants Medicine/Temperance XP, and is consumed.
- **Extraction** (`r` on the door tile): "Extracted; loot secured: 0" → returns to
  colony with survivor/station state intact.
- **Input safety**: queued input is capped and overflow is logged ("Input queue
  full; additional gameplay commands were dropped") rather than crashing.

### 3.4 Persistence
- **Save** (F5): atomic write to manual slot; logs "Saved N entities to
  …/manual-slot.ron"; updates `SaveAvailability`.
- **Load** (F9): restores full colony state. Verified round-trip: save at
  Day 1 / Supplies 7 → quit → relaunch → load → **Day 1 / Supplies 7 restored**,
  log "Loaded save from …/manual-slot.ron (turn 0)".

---

## 4. Scope & Boundaries (what this build is / is not)

**In Foundation scope (present):** shelter colony loop, station build/staff/produce,
survivors, data-driven content (1 dungeon / 6 stations / 3 blueprints / 4 actions),
manual-slot persistence, terminal TUI, placeholder factions (2) and virtue hooks (4 skills).

**Not in Foundation scope (per the hardening plan's "Does not authorize"):** product P2,
procgen in the Foundation path, overworld expansion, raids, colony events, sanity
mechanics, theology-driven mechanics, faction reputation, final factions, new dungeon
content, broad balance changes, new runtime technology.

---

## 5. Observations & Minor Issues

All minor; none block the loop. Noted for the owner's triage:

1. **Healing has no explicit feedback log.** The potion heals correctly (verified
   via HP math: 19 + 8 heal then −8 rat hit = 19), but there is no "You healed X HP"
   log line, so the heal is invisible to the player. UX gap, mechanics fine.
2. **Input queue cap**: batching many keys at once (e.g., 8 moves) drops extras
   with a log notice. By-design defensive behavior; only affects scripted/rapid input.
3. **`SaveAvailability` title-screen framing**: with a save present the title flow
   is less explicit (fresh game auto-enters colony; F9 load works from there).
   Minor polish item.
4. **No build-system default toolchain**: the repo has no `rust-toolchain.toml` and
   the machine has no rustup default, so plain `cargo` fails ("could not choose a
   version of cargo"). `cargo +stable` (1.97.0) is required; 1.85 cannot build
   current deps (need rustc ≥ 1.88).
5. **Test residue**: a verification save (Day 1, Supplies 7) was left at
   `~/.local/share/broken-divinity/saves/manual-slot.ron` during this session.
   Delete it for a clean player state.

---

## 6. Environment Notes

- **Sandbox**: the terminal sandbox virtualizes `$HOME`, so save writes to
  `~/.local/share/broken-divinity/` do not persist unless the game is run with
  unsandboxed filesystem access. This is an environment artifact, not a game bug.
- **Key sends (crossterm)**: F5 = `ESC[15~`, F9 = `ESC[20~` when driving the TUI
  programmatically.
- **Run**: `cargo +stable run -p bd_app` (or `./target/debug/bd` after build).
- **Validate**: `./target/debug/bd --validate` for headless content checks.

---

## 7. Recommended Next Steps (owner decision)

- Triage the minor issues in §5 (heal feedback log is the highest-value UX fix).
- Consider pinning a default toolchain (`rustup default stable` or add
  `rust-toolchain.toml`) so plain `cargo` works for future sessions.
- When ready, open a new owner-approved plan for P2 scope (per
  `docs/FOUNDATION-TEST-AND-UX-HARDENING-PLAN.md` boundaries).
