# Dependency Matrix

Generated: 2026-07-08
Bevy: 0.18.1 (0.19 requires rustc 1.95.0 — we're on 1.91.1)
Rust: 1.91.1

| Crate | Version | Purpose | Status | Notes |
|---|---|---|---|---|
| `bevy` | 0.18.1 | ECS + app framework | ✅ Accepted | Terminal-only, no rendering features |
| `bevy_app` | 0.18.1 | App/plugin system | ✅ Accepted | |
| `bevy_ecs` | 0.18.1 | Entity-component-system | ✅ Accepted | |
| `bevy_time` | 0.18.1 | Time resources | ✅ Accepted | |
| `bevy_ratatui` | 0.11.1 | Bevy↔Ratatui bridge | ✅ Accepted | Uses Bevy message system |
| `ratatui` | 0.30.2 | Terminal UI widgets | ✅ Accepted | |
| `crossterm` | 0.29.0 | Terminal raw mode + input | ✅ Accepted | |
| `tracing` | 0.1.44 | Structured diagnostics | ✅ Accepted | |
| `tracing-subscriber` | 0.3.23 | Tracing output | ✅ Accepted | |
| `color-eyre` | 0.6.5 | Colorful error reports | ✅ Accepted | Chosen over anyhow for terminal UX |
| `serde` | 1.0 | Serialization | ✅ Accepted | |
| `ron` | 0.12 | RON content files | ✅ Accepted | |
| `serde_json` | 1.0 | JSON debug export | ✅ Accepted | |
| `thiserror` | 2.0.18 | Typed library errors | ✅ Accepted | |
| `insta` | 1.48.0 | Snapshot testing | ✅ Accepted | |
| `rand` | 0.10 | RNG | ✅ Accepted | |
| `rand_chacha` | 0.10 | Deterministic RNG | ✅ Accepted | |

## Deferred (declared, not yet used)

| Crate | Version | Purpose | Phase | Notes |
|---|---|---|---|---|
| `pathfinding` | 4.14 | A*/BFS pathfinding | Phase 13 | Spike against bracket-pathfinding |
| `petgraph` | 0.7 | Graph algorithms | Phase 14 | Room/world graphs |
| `directories` | 6.0 | OS app dirs | Phase 16 | ProjectDirs integration |
| `toml` | 0.9 | TOML config parsing | Phase 16 | User config files |
| `schemars` | 0.8 | JSON Schema generation | Phase 8 | Spike after content types stabilize |

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
