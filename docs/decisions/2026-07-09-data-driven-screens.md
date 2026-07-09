# Decision: Data-Driven TUI Screens (Phase 15)

**Date**: 2026-07-09  
**Status**: Accepted

## Context

Phase 15 moves the TUI layout from hardcoded Ratatui `Layout::split` calls in `draw_ui` to data-driven screen definitions with registered widgets. This enables:
1. Multiple screens (combat, inventory) without duplicating layout code.
2. Screen switching without gameplay state mutation.
3. Validation at startup that all widget bindings are correct.

## Decision

### Screen definitions as Rust fixtures

`ScreenDefinition` and `PanelDefinition` are Rust structs, not RON files. Following the plan's rule ("Do not force unstable models into data files"), we keep them in Rust until the schema stabilizes.

```rust
pub struct ScreenDefinition {
    pub id: String,
    pub panels: Vec<PanelDefinition>,
}

pub struct PanelDefinition {
    pub id: String,
    pub layout: PanelLayout,
    pub view_model: String,
}
```

### Panel layout: percentage-based regions

`PanelLayout` uses 5 variants: `Left`, `Right`, `Top`, `Bottom` (each with a percentage), and `Main` (the remaining space). Exactly one `Main` panel per screen is enforced by validation.

### Widget registry

`WidgetRegistry` maps panel IDs to `WidgetBinding` closures. Each binding specifies its expected view-model type and a render function. This decouples *what* to render from *where* to render it.

### Screen switching via `ScreenIntent` message

`ScreenIntent { screen_id }` is a Bevy Message (not Event, following Phase 4's decision). The `process_screen_intents` system updates `ScreenState` which `draw_ui` reads to select the current screen definition.

### Two screens (reuse proof)

1. **Combat screen** — map (Main), stats (Right 25%), log (Bottom 30%), actions (Bottom 12%)
2. **Inventory screen** — inventory_list (Main), equipment (Right 30%), stats (Top 15%), log (Bottom 20%)

Both use the same `WidgetRegistry` — only the layout differs.

### Validation at startup

`validate_screens()` checks:
- Every panel has a registered widget.
- View-model types match between definition and binding.
- Exactly one `Main` panel per screen.
- No empty screens.
- Unused widgets generate warnings (not errors).

## Alternatives considered

| Alternative | Reason rejected |
|---|---|
| RON data files | Schema not stable yet; Rust fixtures first per plan |
| `Box<dyn FnOnce>` system closures | `Fn` works in `HashMap` for repeated rendering; `FnOnce` would require per-frame registration |
| Event-based screen switching | Phase 4 chose Messages; ScreenIntent follows that decision |
| Single enum dispatch instead of registry | Registry allows adding widgets without modifying a central match |

## Consequences

- **Positive**: Adding a new screen is data-only — define panels, register new widgets, done.
- **Positive**: Validation catches missing/broken bindings at startup.
- **Positive**: Screen switching cannot mutate gameplay state (only changes `ScreenState` resource).
- **Negative**: Widget render closures in `HashMap` require `#[allow(clippy::type_complexity)]`.
- **Negative**: `draw_ui` has 11 parameters; mitigated by `#[allow(clippy::too_many_arguments)]`.
- **Neutral**: Old render functions (`render_map`, `render_stats`, etc.) migrated from `lib.rs` to `screens.rs` with the same rendering logic.
