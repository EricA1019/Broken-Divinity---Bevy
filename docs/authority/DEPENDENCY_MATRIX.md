# Dependency Matrix

Generated: 2026-08-01
Bevy: 0.18.1 (0.19 requires rustc 1.95.0)
MSRV: 1.85 (`rust-version` in `Cargo.toml`); builds verified on stable (1.97)

## Accepted (used in source)

| Crate | Version | Purpose | Used in |
|---|---|---|---|
| `bevy` | 0.18.1 | ECS + app framework | all crates; terminal-only, no rendering features |
| `bevy_app` | 0.18.1 | App/plugin system | all crates |
| `bevy_ecs` | 0.18.1 | Entity-component-system | all crates |
| `bevy_ratatui` | 0.11.1 | Bevy↔Ratatui bridge | `bd_tui` |
| `ratatui` | 0.30 | Terminal UI widgets | `bd_tui` |
| `crossterm` | 0.29 | Terminal raw mode + input | `bd_app`, `bd_tui` |
| `tracing` | 0.1 | Structured diagnostics | all crates |
| `tracing-subscriber` | 0.3 | Tracing output | `bd_app` |
| `serde` | 1.0 | Serialization | `bd_core`, `bd_data`, `bd_tui`, `bd_test_support` |
| `ron` | 0.12 | RON content files | `bd_core`, `bd_data`, `bd_app`, `bd_test_support` |
| `serde_json` | 1.0 | JSON export | `bd_test_support` (contract report) |
| `thiserror` | 2.0 | Typed library errors | `bd_core`, `bd_data`, `bd_tui` |
| `rand` | 0.10 | RNG | `bd_core`, `bd_test_support` |
| `rand_chacha` | 0.10 | Deterministic seeding | `bd_core`, `bd_test_support` |
| `directories` | 6.0 | OS config/data paths | `bd_app` |
| `toml` | 0.9 | User config parsing | `bd_app` |
| `pathfinding` | 4.14 | A* behind the `pathfinding` adapter | `bd_core` (colony movement) |
| `petgraph` | 0.7 | Graph algorithms | `bd_core` (procgen) |

## Declared but not used

| Crate | Version | Why it is unused |
|---|---|---|
| `bevy_time` | 0.18.1 | Time is owned by `bd_core::time`; no crate imports it |
| `color-eyre` | 0.6 | App boundary still returns `Result<(), String>`; nothing imports it |
| `insta` | 1.48 | Declared in `bd_test_support`; no snapshot tests use it |
| `schemars` | 0.8 | JSON Schema generation spike deferred |

## Rejected

| Crate | Reason |
|---|---|
| `anyhow` | `color-eyre` was chosen for terminal UX, but neither is wired yet; `thiserror` covers typed errors |
| `bevy_ecs_tilemap` | Terminal-only app, no tilemap needed |
| `bevy_egui` | Terminal-only app, no GUI needed |
| `bevy 0.19.0` | Requires rustc 1.95.0; MSRV is 1.85 |

## Spike Candidates (not yet declared)

| Crate | Purpose | Phase |
|---|---|---|
| `bracket-pathfinding` | Roguelike FOV/Dijkstra | Phase 13 |
| `bevy_save` | World save/load | Phase 17 |
| `moonshine-save` | World save/load alt | Phase 17 |
| `bevy-persistent` | Settings persistence | Phase 16 |
| `leafwing-input-manager` | Advanced input binding | Deferred |

## Rejected

| Crate | Reason |
|---|---|
| `anyhow` | Chose `color-eyre` for better terminal error UX |
| `bevy_ecs_tilemap` | Terminal-only app, no tilemap needed |
| `bevy_egui` | Terminal-only app, no GUI needed |
| `bevy 0.19.0` | Requires rustc 1.95.0; we're on 1.91.1 |
