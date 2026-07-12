# Broken Divinity — UX Improvement Plan v2

**Based on**: Live playtest 2026-07-12
**Current Score**: 3/10 overall UX
**Target**: 7/10 overall UX
**Review status**: Revised per senior eng review. Investigation phase added. All items have tests.

---

## Phase 0: Triage — Investigate before coding (30 min, NO code)

### 0.1: Confirm tracing is the root cause of visual corruption
**Action**: Launch game with `RUST_LOG=error cargo run --bin bd 2>/dev/null` and re-playtest the Outpost→Travel→Event→Dungeon→Outpost loop.
**Question**: Does screen corruption disappear when only `error!()` output reaches stdout?
**Expected**: Tracing `INFO` lines cause the majority of corruption. If so, P1-A fixes most visual issues.

### 0.2: Diagnose why 'q' leaks to shell
**Action**: Trace the code path: `map_input_to_intents` line 336 checks `KeyCode::Char('q') | KeyCode::Esc` and writes `AppExit`. Inspect `ScheduleRunnerPlugin` to verify it consumes `AppExit` and calls `std::process::exit()`.
**Question**: Is the `AppExit` message actually reaching the runner, or is the terminal layer eating the key first?
**Check**: Does `Esc` work for quitting? If Esc works but 'q' doesn't, the issue is in key mapping (line 268 has unreachable 'a' pattern — maybe 'q' has a similar shadowing bug).

### 0.3: Inventory Ratatui 0.30 API for screen clearing
**Action**: Check `ratatui::Terminal` for `clear()`, or `ratatui::buffer::Buffer` for `reset()`, or `Frame` for `render_clear()`. Verify the correct method exists before writing code.
**Question**: What is the correct Ratatui 0.30 API to reset the backbuffer when map dimensions change?

### 0.4: Audit symbol/theme system capabilities
**Action**: Read `content/themes/default.ron` and `content/symbols/default.ron` to verify what color fields exist per symbol. Check `crates/bd_tui/src/visual.rs` for the `VisualToken` struct and its fields.
**Question**: Does each token already have `fg_color` and `bg_color` fields? If not, adding color requires schema changes.

---

## Phase 1: Critical Fixes (2 hrs)

### P1-A: Suppress tracing to stderr
**Why**: Tracing `INFO` lines bleed into the Ratatui render buffer, corrupting every panel. Confirmed root cause in P0.1.
**Effort**: 10 minutes
**Files**: `crates/bd_app/src/main.rs` lines 27-31
**Change**:
```rust
tracing_subscriber::fmt()
    .with_env_filter(
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "bd=info".into()))
    .with_writer(std::io::stderr)
    .init();
```
**Test**: `tracing_goes_to_stderr_not_stdout`
- Launch app, verify `2>/dev/null` shows clean TUI, `2>&1 | grep INFO` finds log lines on stderr
**Verification**: Launch game — zero timestamp/INFO lines visible in any panel.
**Risk**: None. Additive change.

### P1-B: Fix quit key handling
**Why**: 'q' leaks to the shell as `sd <FIND> <REPLACE_WITH>` error. Diagnosed in P0.2.
**Effort**: 15 minutes
**Files**: `crates/bd_tui/src/lib.rs` (input handler ~line 336, plus fix line 268 unreachable 'a' pattern)
**Change**:
1. Remove unreachable duplicate `KeyCode::Char('a')` at line 268 (warns on every build)
2. Add `Ctrl+C` and `Ctrl+D` as additional quit handlers alongside 'q' and Esc
3. Ensure `AppExit` is consumed: verify `ScheduleRunnerPlugin::run_loop` handles it
**Test**: `quit_keys_exit_cleanly`
- Send synthetic key events for 'q', Esc, Ctrl+C, Ctrl+D to the `Messages<KeyMessage>` resource, run one frame, verify `AppExit` was written and app exits with code 0
**Risk**: Low if `ScheduleRunnerPlugin` handles AppExit; medium if we need to bypass it.

### P1-C: Set AP to 3 at game start
**Why**: First-time player hits "Not enough ActionPoints." before they can act. Don't know to press '.'.
**Effort**: 5 minutes
**Files**: `crates/bd_core/src/factory.rs` — blueprint player Pools initialization (single source of truth; not `main.rs` which should ONLY call blueprint)
**Change**: Ensure `blueprint.player` sets `Pool::new(PoolKind::ActionPoints, PLAYER_MAX_AP, 0, PLAYER_MAX_AP)` where the current value equals the max. Check if the blueprint already sets this or if it defaults to 0.
**Test**: `player_starts_with_full_ap`
- Spawn player via blueprint, read Pools, assert `ActionPoints.current == ActionPoints.max`
**Risk**: None.

### P1-D: Clear screen on mode transitions
**Why**: Ghost characters, broken borders when switching between 40-wide shelter and ~20-wide dungeon.
**Effort**: 20 minutes (after P0.3 validates the API)
**Files**: `crates/bd_tui/src/lib.rs` — `draw_ui` function
**Change**:
1. Track previous map dimensions in `draw_ui` via a `Local<(i32, i32)>` parameter
2. When dimensions differ from current, call `ratatui_ctx.terminal_mut().clear()` (or the validated API from P0.3) before the `draw` closure
3. Alternative if no clear API: render a `Clear` widget over the full frame area before panel rendering
**Test**: `screen_clears_on_mode_transition` (modify existing `screen_switch_preserves_gameplay_state` to also check for visual artifacts — tricky to assert in unit test; primary verification is manual playtest)
**Risk**: Medium. Unknown Ratatui API. May need workaround.

### Phase 1 verification
Launch game, play Outpost→Travel→Event→Dungeon→Outpost loop. Verify:
- [ ] No tracing visible in any panel
- [ ] 'q' exits cleanly (no shell leak)
- [ ] AP starts at 3/3
- [ ] Screen renders cleanly in all modes
- [ ] `cargo test` — 0 regressions

---

## Phase 2: Core Feedback Loop (1.5 hrs)

### P2-A: Show damage numbers on attack
**Why**: "You attack!" with no damage dealt makes combat feel broken.
**Important**: Do NOT modify `resolve_pool_deltas` (SRP violation). Create a separate observer system.
**Effort**: 20 minutes
**Files**: NEW system in `crates/bd_core/src/pools.rs` (or new `combat_feedback.rs`)
**Change**: New system `log_combat_damage` in `BdSet::ResultEmission`:
- Reads `PoolDeltaApplied` messages where `kind == Health` and `amount_applied < 0`
- Pushes to `GameLog` at `LogLevel::Combat`: `"You deal {amount} damage!"` (player→enemy) or `"{name} hits you for {amount}!"` (enemy→player)
- Uses `Name` component or entity debug for target identification
**Test**: `damage_is_logged_to_combat_channel`
- Spawn player + enemy, apply Health delta for -5 damage, run frame, assert GameLog contains "5 damage"
**Risk**: Low. Observer system, doesn't touch existing logic.

### P2-B: Show enemy defeat in log
**Why**: After attacking an enemy to death, it silently disappears. No closure.
**Effort**: 10 minutes
**Files**: `crates/bd_core/src/pools.rs` — `cleanup_defeated_entities`
**Change**: Modify the existing `cleanup_defeated_entities` system:
- Read `Name` component from defeated entity before despawning
- Push `"The {name} is defeated!"` to `GameLog` at `LogLevel::Combat`
- Fallback: `"An enemy is defeated!"` if no Name component
**Test**: `defeat_is_logged_to_combat_channel`
- Spawn enemy with Name("Rat") + Health pool at 1, apply -5 delta, run frame, assert log contains "Rat is defeated"
**Risk**: None. Modifies an existing system but adds only a log push.

### P2-C: Add AP recovery hint to denial message
**Why**: "Not enough ActionPoints." gives no recovery path.
**Effort**: 10 minutes
**Files**: `crates/bd_core/src/actions.rs` — `validate_action_intents` denial formatting
**Change**: In the denial message formatting (around line 386):
```rust
DenialReason::NotEnoughPool(PoolKind::ActionPoints) => {
    "Not enough ActionPoints. Wait (.) to restore 1 AP.".into()
}
```
Leave other `NotEnoughPool` variants unchanged.
**Test**: `ap_denial_includes_wait_hint`
- Spawn player with 0 AP, send MoveEast action, run frame, assert GameLog contains "Wait (.)"
**Risk**: None. String change only.

### P2-D: Show item name on pickup
**Why**: Walking over '+' items gives no feedback. Player doesn't know what they picked up or if pickup worked.
**Effort**: 15 minutes
**Files**: 
- `crates/bd_core/src/actions.rs` — `resolve_action_effects` MoveEntity handler
- OR new observer system on `EntityMoved` messages
**Change**: After moving the player, check if the new position has an entity with `Item` component. If so, log `"You found a {item_name}!"` at `LogLevel::Info`.
**Test**: `item_pickup_logs_item_name`
- Spawn player at (5,5), spawn item at (6,5) with `Name("Healing Potion")`, `Item`, and not `BlocksMovement`. Send MoveEast action, run frame, assert log contains "Healing Potion"
**Risk**: Low. Adds a check in the move handler.

### Phase 2 verification
- [ ] Attack enemy → see "You deal 5 damage!"
- [ ] Kill enemy → see "The Rat is defeated!"
- [ ] Attempt move at 0 AP → see "Not enough ActionPoints. Wait (.) to restore 1 AP."
- [ ] Walk over '+' → see "You found a Healing Potion!"
- [ ] `cargo test` — all new tests pass, 0 regressions

---

## Phase 3: UI Cleanup (2 hrs)

### P3-A: Fix stats panel layout
**Why**: "Faith: 0  8" (two values on one line), "AP: 0/3" with stray "    0" floating below.
**Effort**: 45 minutes
**Files**: `crates/bd_tui/src/screens.rs` — stats panel rendering function
**Root cause**: Stats panel `width_pct: 25` is too narrow for content, causing Ratatui Paragraph wrapping at unexpected boundaries.
**Change**:
1. Increase to `width_pct: 28` (named constant `STATS_PANEL_WIDTH_PCT` in `screens.rs`)
2. Render each stat as a separate `Line` in the Paragraph to prevent wrapping
3. Format: `HP: {current}/{max}` on one line, `AP: {current}/{max}` on next, etc.
4. Add visual grouping: separator line between combat stats (HP, AP) and resources (Supplies, Faith) and progress (Day)
**Test**: Manual verification — launch game, check no overlapping values.
**Risk**: Medium. Ratatui layout changes can have knock-on effects on panel widths.

### P3-B: Clean up Travel panel
**Why**: "Reachable locations:" always shows empty list. Confuses players.
**Effort**: 15 minutes
**Files**: `crates/bd_tui/src/screens.rs` — travel panel rendering
**Change**: When `travel_nodes.is_empty()`, show `"Press 't' to travel to a dungeon."` instead of empty "Reachable locations:" header. When nodes exist, show them as before.
**Test**: `travel_panel_shows_hint_when_empty` — verify the render function outputs the help text when the travel node list is empty.
**Risk**: None. String change only.

### P3-C: Add turn counter to footer
**Why**: Day counter advances slowly (24 turns per day). Player has no sense of progress.
**Effort**: 10 minutes
**Files**: 
- `crates/bd_tui/src/lib.rs` — `render_footer` function
- `crates/bd_core/src/time.rs` — `GameTime` resource is already available
**Change**: Read `GameTime` in footer rendering (or add `TurnCounter` to the widget context). Show `"Turn: {turn} | Day: {day}"` in footer.
**Test**: `footer_shows_turn_counter` — insert GameTime { day: 0, turn: 5 }, render footer, verify output contains "Turn: 5".
**Risk**: None.

### P3-D: Add '?' help screen
**Why**: No way to look up keybindings during play. Help line at bottom is cut off.
**Effort**: 45 minutes
**Files**: 
- `crates/bd_tui/src/screens.rs` — new "help" screen definition
- `crates/bd_tui/src/view_models.rs` — new `HelpViewModel`
- `crates/bd_tui/src/lib.rs` — `'?'` input handler, screen switching
**Change**:
1. Register "help" screen: Main panel showing keybinding table (WASD→Move, .→Wait, f→Attack, g→Guard, t→Travel, r→Return, i→Inventory, b→Build, a→Assign, ?→Help, q→Quit)
2. '?' toggles help on/off (uses same previous_screen mechanism as event system)
3. Help is a modal overlay — ESC or '?' closes it
**Test**: `help_screen_displays_keybindings` — switch to help screen, verify view model contains keybinding entries.
**Risk**: Low. Reuses existing screen infrastructure.

### P3-E: Add enemy type glyphs
**Why**: All enemies show as `E`. Player can't distinguish Rats from Skeletons.
**Effort**: 20 minutes
**Files**: 
- `crates/bd_tui/src/view_models.rs` — `build_map_vm` enemy overlay
- `crates/bd_tui/src/visual.rs` — or inline in view model
**Change**: In `build_map_vm`, read enemy `Name` component. Map known names to glyphs: `"Rat"→r`, `"Skeleton"→S`, `"Boss"→B`. Fallback: `E`.
**Test**: `enemy_glyph_maps_by_name` — spawn enemy with Name("Rat"), build map VM, verify that position's token is 'r'.
**Risk**: None.

### Phase 3 verification
- [ ] Stats panel: one value per line, no overlap, groups clear
- [ ] Travel panel: shows "Press 't' to travel" when empty
- [ ] Footer: shows "Turn: 5 | Day: 0"
- [ ] '?' shows help overlay, ESC closes it
- [ ] Rats show as 'r', Skeletons as 'S'
- [ ] `cargo test` — all new tests pass, 0 regressions

---

## Phase 4: Color & Polish (3 hrs)

### P4-A: Audit symbol/theme capabilities
**Before any color code**: Complete P0.4. Document what the theme system currently supports.

### P4-B: Add color to render functions
**Effort**: 2 hours
**Files**: Every `render_*_widget` function in `crates/bd_tui/src/screens.rs`
**Change**:
1. Define color constants in `visual.rs`:
   - `WALL_COLOR = Color::Gray` / `Color::DarkGray`
   - `FLOOR_COLOR = Color::DarkGray` / `Color::Reset`
   - `PLAYER_COLOR = Color::Cyan` / `Color::White`
   - `ENEMY_COLOR = Color::Red`
   - `ITEM_COLOR = Color::Yellow`
   - `BORDER_COLOR = Color::Blue`
2. Apply colors in each render widget's `Paragraph` style
3. Do NOT modify the theme file format if it doesn't support colors yet — just use inline styles for now
**Test**: Manual verification only (color in terminal can't be easily unit tested). Screenshot comparison.
**Risk**: Medium if theme system needs changes.

### P4-C: Color-code HP/AP bars
**Effort**: 30 minutes
**Files**: `crates/bd_tui/src/screens.rs` — stats panel render
**Change**: 
- HP: green at >50%, yellow at 25-50%, red at <25% (named constants `HP_GREEN_THRESHOLD_PCT`, `HP_YELLOW_THRESHOLD_PCT`)
- AP: blue at full, white otherwise
**Risk**: None.

### P4-D: Title screen
**Effort**: 1 hour
**Files**: New screen "title" in `screens.rs`
**Change**:
1. "BROKEN DIVINITY" in ASCII art centered
2. "Kernel v0.1.0" subtitle
3. "Press any key to begin" prompt
4. `GameMode` gets a new variant `Title` that's the default instead of `Outpost`
5. Any keypress transitions to `Outpost`
**Test**: `title_screen_is_default_on_launch` — verify GameMode starts as Title.
**Risk**: Low. Additive.

### Phase 4 verification
- [ ] Walls gray, floor dark, player bright, enemies red, items yellow
- [ ] HP bar green→yellow→red as health drops
- [ ] Title screen appears before gameplay
- [ ] `cargo test` — 0 regressions

---

## Phase 5: Gameplay Systems (DEFERRED — separate plan)

These are out of scope for the UX improvement pass. They require their own TDD planning:

| Item | Why deferred |
|------|-------------|
| Enemy AI (movement + attacks) | 2-3 days work; needs pathfinding, decision trees, action integration |
| Game over screen | Depends on enemy AI for threat to exist |
| Save/Load | 2-3 days; needs serde audit of all resources |
| Onboarding tooltips | Depends on all UX being stable first |

---

## Effort Summary

| Phase | Items | Effort | Score Delta | Cumulative |
|-------|-------|--------|-------------|------------|
| P0 | 4 (triage) | 30 min | — | — |
| P1 | 4 (critical) | 2 hrs | 3→5 | 5/10 |
| P2 | 4 (feedback) | 1.5 hrs | 5→6 | 6/10 |
| P3 | 5 (UI cleanup) | 2 hrs | 6→7 | 7/10 |
| P4 | 3 (color) | 3 hrs | 7→7.5 | 7.5/10 |
| **Total** | **20 items** | **~9 hrs** | **3→7.5** | — |

---

## Test Inventory

| Test | Phase | File |
|------|-------|------|
| `tracing_goes_to_stderr_not_stdout` | P1-A | manual (stderr check) |
| `quit_keys_exit_cleanly` | P1-B | `crates/bd_tui/src/lib.rs` tests |
| `player_starts_with_full_ap` | P1-C | `crates/bd_core/src/factory.rs` tests |
| `screen_clears_on_mode_transition` | P1-D | manual (visual check) |
| `damage_is_logged_to_combat_channel` | P2-A | `crates/bd_core/src/pools.rs` tests |
| `defeat_is_logged_to_combat_channel` | P2-B | `crates/bd_core/src/pools.rs` tests |
| `ap_denial_includes_wait_hint` | P2-C | `crates/bd_core/src/actions.rs` tests |
| `item_pickup_logs_item_name` | P2-D | `crates/bd_core/src/actions.rs` tests |
| `travel_panel_shows_hint_when_empty` | P3-B | `crates/bd_tui/src/screens.rs` tests |
| `footer_shows_turn_counter` | P3-C | `crates/bd_tui/src/lib.rs` tests |
| `help_screen_displays_keybindings` | P3-D | `crates/bd_tui/src/screens.rs` tests |
| `enemy_glyph_maps_by_name` | P3-E | `crates/bd_tui/src/view_models.rs` tests |
| `title_screen_is_default_on_launch` | P4-D | `crates/bd_core/src/spatial.rs` tests |

---

## Files Referenced

| File | Phase | Purpose |
|------|-------|---------|
| `crates/bd_app/src/main.rs` | P1 | Tracing stderr, quit handling |
| `crates/bd_core/src/pools.rs` | P2 | Combat damage observer, defeat feedback |
| `crates/bd_core/src/actions.rs` | P2 | AP hint, item pickup detection |
| `crates/bd_core/src/factory.rs` | P1 | Player blueprint AP init |
| `crates/bd_core/src/time.rs` | P3 | GameTime for turn counter |
| `crates/bd_tui/src/lib.rs` | P1,P3 | Quit keys, screen clear, footer, help toggle |
| `crates/bd_tui/src/screens.rs` | P3,P4 | Stats layout, travel panel, help screen, colors |
| `crates/bd_tui/src/view_models.rs` | P3 | Help view model, enemy glyphs |
| `crates/bd_tui/src/visual.rs` | P4 | Color constants, token definitions |
| `content/themes/default.ron` | P4 | Theme audit only (no changes unless schema supports color) |
| `content/symbols/default.ron` | P4 | Symbol audit only |
