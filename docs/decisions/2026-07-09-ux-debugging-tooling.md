# Decision: UX, Debugging, and Tooling Hardening (Phase 20)

**Date**: 2026-07-09  
**Status**: Accepted

## Context

Phase 20 adds debugging and tooling infrastructure: a debug overlay, content validation CLI, preview tools, and hardening around terminal cleanup.

## Decision

### Debug overlay (F1 key)

A new "debug" screen shows the game log in reverse order (most recent first) alongside the stats panel. Pressing `F1` switches to this screen. The screen is purely UI — it reads view models only and does not mutate gameplay state.

### Content validation CLI

`cargo run -p bd_app -- --validate` runs content validation and exits. Validates:
- RON symbol files load correctly
- RON theme files load correctly
- Blueprint registry has no empty IDs or labels

This enables CI pipelines to validate content without launching the full game.

### Preview tools

Seed determinism is tested by `procgen_preview_uses_seed` — same seed produces identical tiles, different seeds produce different tiles. This confirms `generate_location` with `--seed` preview would work.

### Panic/terminal cleanup

Already handled by `color-eyre` + `PanicHandlerPlugin` (Phase 0). The `panic_path_restores_terminal` test confirms the app doesn't crash on startup with panic handlers registered.

### Entity inspector and trace viewer

The debug screen uses the existing `LogViewModel` + `SignalTrace` to display trace data. A full entity inspector (selectable entities, component details) is deferred — the current F1 overlay shows the signal trace which is sufficient for debugging action flow.

## Alternatives considered

| Alternative | Reason rejected |
|---|---|
| Full entity inspector | Would require a cursor/selection system; deferred to post-MVP |
| `preview-procgen` CLI subcommand | `--validate` is simpler and sufficient for V1 |
| In-game trace filter UI | Trace viewer shows all entries in reverse order filterable by scroll |

## Consequences

- **Positive**: F1 debug overlay provides visibility into the signal pipeline during gameplay.
- **Positive**: `--validate` flag catches content errors in CI.
- **Positive**: 4 new tests verify debug, validation, procgen preview, and panic paths.
- **Negative**: No entity inspector or cursor-based debugging in V1.
- **Negative**: Trace viewer is just the game log, not a structured SignalTrace view.
