---
name: add-egui-panel
description: "Adding a new egui panel or window — draw/process split, UiAction resource, registration in main.rs."
triggers:
  - "add panel"
  - "new panel"
  - "egui window"
  - "new UI"
  - "add menu"
  - "add modal"
edges:
  - target: context/conventions.md
    condition: to verify the draw/process split pattern
  - target: context/architecture.md
    condition: to understand UI system placement (Tier 5)
last_updated: 2026-04-06
---

# Add an egui Panel

## Context

Load `context/conventions.md` for the draw/process split pattern. All UI is egui — no Bevy native Node UI. UI code lives in Tier 5. If the panel needs game data display, consider if it's a HUD element (use hud-ui Copilot skill) or a full panel (use egui-panel skill).

## Steps

1. **Create the draw system** — renders UI, writes intentions to `UiAction` resource:
   ```rust
   pub fn draw_my_panel(
       mut contexts: Query<&mut EguiContext>,
       mut ui_action: ResMut<UiAction>,
       game_data: Res<SomeData>,
   ) {
       let Ok(mut ctx) = contexts.single_mut() else { return; };
       egui::Window::new("My Panel").show(ctx.get_mut(), |ui| {
           if ui.button("Do Thing").clicked() {
               ui_action.0 = Some(Action::DoThing);
           }
       });
   }
   ```
2. **Create the process system** — reads `UiAction`, performs actual game logic:
   ```rust
   pub fn process_my_panel(
       mut ui_action: ResMut<UiAction>,
       mut game_state: ResMut<SomeState>,
   ) {
       let Some(action) = ui_action.0.take() else { return; };
       match action {
           Action::DoThing => { /* actual logic here */ }
       }
   }
   ```
3. **Register both systems** in main.rs:
   ```rust
   // Draw — in EguiPrimaryContextPass
    .add_systems(EguiPrimaryContextPass, draw_my_panel.run_if(in_state(AppState::Dungeon)))
   // Process — in Update
    .add_systems(Update, process_my_panel.run_if(in_state(AppState::Dungeon)))
   ```

## Gotchas

- **Draw system in wrong schedule**: draw systems MUST be in `EguiPrimaryContextPass`, not `Update`
- **Logic in draw system**: draw systems only write to `UiAction` — never mutate game state directly
- **Missing state gate**: both draw and process systems need AppState gating
- **Context query failure**: always use `let Ok(mut ctx) = contexts.single_mut() else { return; };`

## Verify

- [ ] Draw system registered in `EguiPrimaryContextPass`
- [ ] Process system registered in `Update`
- [ ] Both systems are state-gated
- [ ] Draw system only reads game state + writes UiAction
- [ ] Process system handles UiAction and performs actual mutations
- [ ] Graceful query failure on EguiContext

## Debug

- **Panel doesn't appear**: check state gating — is the AppState correct? Is the draw system in `EguiPrimaryContextPass`?
- **Button click does nothing**: check process system reads the same UiAction variant the draw system writes
- **UI flickers**: draw system might be fighting with another panel for the same window name
