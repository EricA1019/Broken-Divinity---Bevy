# Decision Log

## 2026-07-08 — color-eyre over anyhow

### Problem
Need an error-reporting crate for the app boundary. Both `anyhow` and `color-eyre` are mature options.

### Options tested
- `anyhow` — lightweight, widely used
- `color-eyre` — colorful error reports with span traces

### Accept criteria
- Works with `thiserror` in library crates
- Integrates with `tracing`
- Good terminal UX (the app IS a terminal)

### Reject criteria
- Heavy compile times
- Incompatible with terminal raw mode

### Result
Chose `color-eyre`. It resolved transitively from bevy_ratatui's dep tree, provides colorful panic/error output that suits a terminal app, and has zero additional compile cost.

### Reason
Better terminal UX. The app runs in a terminal — error readability matters.

### Follow-up work
None.

---

## 2026-07-08 — Bevy 0.18.1 over 0.19.0

### Problem
Bevy 0.19.0 is the latest, but requires rustc 1.95.0.

### Options tested
- Bevy 0.18.1 + rustc 1.91.1 ✅ compiles
- Bevy 0.19.0 + rustc 1.91.1 ❌ requires 1.95.0

### Result
Stay on Bevy 0.18.1. Upgrade when bevy_ratatui ships a 0.19-compatible release.

### Reason
Toolchain stability and bevy_ratatui compatibility.

---

## 2026-07-08 — Workspace structure

### Problem
Single crate vs. multi-crate workspace.

### Result
Five crates: `bd_app` (binary), `bd_core` (ECS), `bd_tui` (terminal), `bd_data` (content), `bd_test_support` (testing).

### Reason
Clear dependency boundaries: `bd_app → bd_tui → bd_core`, `bd_app → bd_data → bd_core`. `bd_test_support` is test-only. This prevents circular deps and enforces the UI/gameplay boundary.
