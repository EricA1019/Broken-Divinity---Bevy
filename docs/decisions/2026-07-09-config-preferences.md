# Decision: Config, Preferences, and App Directories (Phase 16)

**Date**: 2026-07-09  
**Status**: Accepted

## Context

Phase 16 adds platform-correct path resolution, user-editable configuration, and a help-line bridge from key bindings to the TUI footer.

## Decision

### Directory resolution: `directories` crate

Use `directories::ProjectDirs` with qualifier `""`, organization `"bd-kernel"`, and application `"broken-divinity"`. This resolves to standard platform paths:

| Platform | Config dir | Data dir |
|---|---|---|
| Linux | `~/.config/broken-divinity/` | `~/.local/share/broken-divinity/` |
| macOS | `~/Library/Application Support/broken-divinity/` | `~/Library/Application Support/broken-divinity/` |
| Windows | `C:\Users\<user>\AppData\Roaming\broken-divinity\` | `C:\Users\<user>\AppData\Local\broken-divinity\` |

A fallback to `~/.config/broken-divinity/` is used when `ProjectDirs` returns `None`.

### Config format: TOML

Per the plan's recommendation ("TOML for user config if useful"). Config structs use Serde derives. The default config is embedded in the binary; a user config at `~/.config/broken-divinity/config.toml` is merged on top.

### `AppConfig` structure

Fields: `theme_id`, `keybindings` (nested `KeyBindingConfig`), `save_dir_override`, `log_level`, `debug_flags`.

Merging: user file fields that differ from defaults override the built-in defaults. This allows partial config files.

### `HelpLine` resource in `bd_core`

The `HelpLine(pub String)` resource is defined in `bd_core` and consumed by `bd_tui::render_footer`. It is set by `bd_app` at startup from the loaded `KeyBindingConfig`. This keeps the TUI layer free of config logic.

### Config loading flow

```
default AppConfig (embedded)
  → try read ~/.config/broken-divinity/config.toml
  → merge user fields
  → validate
  → set HelpLine resource
  → (future) set tracing log level
```

### Key bindings → help line

`KeyBindingConfig::help_line()` generates a string like:
`"Move:w↑s↓a←d→ | Wait:. | Attack:f | Guard:g | Inventory:i | Combat:z | Quit:q"`

This replaces the hardcoded `"phase 6 | q quit"` in `render_footer`.

## Alternatives considered

| Alternative | Reason rejected |
|---|---|
| `config` crate for loading | Adds another dep; TOML deserialization via `serde` is sufficient for V1 |
| `bevy-persistent` for settings | Premature — spike deferred; V1 just reads/writes TOML files |
| HelpLine in `bd_tui` | Would create awkward dependency: `bd_app` would need to write a `bd_tui` resource; putting it in `bd_core` is cleaner |
| Hardcoded YAML/JSON config | Plan specifies TOML for user config; YAML not in deps |

## Consequences

- **Positive**: No hand-rolled OS path guessing; `directories` handles all platforms.
- **Positive**: Config is user-editable TOML at a known location.
- **Positive**: Help line now derives from the same key bindings used by input mapping.
- **Negative**: The `KeyBindingConfig` values are not yet used by `map_input_to_intents` — key bindings are still hardcoded in the match block. Updating the input mapper to use the config is follow-up work.
- **Neutral**: `HelpLine` in `bd_core` is a simple string resource; could be generalized later.
