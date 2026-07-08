# Architecture Guardrails

Last updated: 2026-07-08

## Mutation Ownership

- **Only resolver systems mutate gameplay state.**
- UI emits intents only.
- AI emits intents only.
- Debug mutation is gated behind `DebugIntent`.
- Triggers emit effects only.
- Modifiers modify requests only.
- Effects describe requested mutations.
- Resolvers perform mutations.

### Forbidden
- UI directly changing Health/ActionPoints
- AI directly moving entities
- Statuses directly editing pools
- Items directly mutating components outside the effect pipeline
- Debug inspector silently editing world state

## Signal Discipline

Every signal must have:
- Clear owner (which system/module produces it)
- Clear schedule stage
- Clear reader/resolver
- Trace entry (in debug builds)
- Failure mode

Trigger chains must have:
- Max depth (default: 10)
- Cycle detection
- Debug trace
- Clear error behavior on overflow

## UI Boundary

TUI systems may:
- Read view models
- Draw widgets
- Emit input intents

TUI systems may NOT:
- Mutate gameplay components
- Apply effects
- Resolve combat
- Query arbitrary gameplay internals
- Bypass the intent pipeline

## Debug Boundary

- Debug tools are read-only by default
- Debug mutation requires explicit `DebugIntent`
- Debug mutation requires explicit debug-only effect path
- Debug mutation requires debug mode gate
- Debug mutation requires clear trace entry

## Save/Load Boundary

- Save/load may serialize and restore state
- Save/load may NOT silently apply gameplay rules
- Allowed: restore validated snapshot, run explicit migration, report invalid save
- Not allowed: quietly fix invalid gameplay state, apply combat/effects during load, invent missing content without migration rule

## ASCII Boundary

- No raw glyphs outside the ASCII/theme layer
- Allowed: `VisualToken::Player`, `VisualToken::Enemy`, `StyleToken::Danger`
- Not allowed: `'@'`, `'#'`, `Color::Red`, `Color::Blue`

## Dependency Flow

```
bd_app ──→ bd_tui ──→ bd_core
  │                      │
  └──→ bd_data ─────────┘

bd_test_support ──→ bd_core
```

- `bd_core`: No deps on other BD crates
- `bd_tui`: Depends on `bd_core` only
- `bd_data`: Depends on `bd_core` only
- `bd_app`: Depends on `bd_tui`, `bd_data`, `bd_core`
- `bd_test_support`: Depends on `bd_core` only
