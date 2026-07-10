---
name: stack
description: Technology stack, library choices, and the reasoning behind them. Load when working with specific technologies or making decisions about libraries and tools.
triggers:
  - "library"
  - "package"
  - "dependency"
  - "which tool"
  - "technology"
edges:
  - target: context/decisions.md
    condition: when the reasoning behind a tech choice is needed
  - target: context/conventions.md
    condition: when understanding how to use a technology in this codebase
last_updated: 2026-04-05
---

# Stack

## Core Technologies

- **Rust** (edition 2024, rust-version 1.85+) — primary language
- **Bevy 0.18.1** — ECS game engine, 2D rendering, input, audio, state machines
- **bevy_egui** — immediate-mode GUI for all panels, HUD elements, and menus
- **RON** — data file format for game data (rosters, dialogue, item catalogs)

## Key Libraries (Confirmed post-UI design)

- **serde 1.0.228 + ron 0.12.1** — serialization for save/load and RON data file parsing
- **rand 0.10.0 + rand_chacha 0.10.0** — deterministic RNG; `ChaCha8Rng` seeded per system via seahash-derived seeds
- **bevy_egui 0.39.1** — locked in UI framework for all UI (immediate-mode), using `EguiPrimaryContextPass` schedule for draw systems
- **bevy_ecs_tilemap 0.18.1** — GPU-chunked tilemap rendering (entity-per-tile) for dungeons, shelter, overworld
- **pathfinding 4.15.0** — A*, Dijkstra, BFS for AI movement and player pathfinding
- **spade 2.15.1** — Delaunay triangulation + Voronoi for overworld road network generation
- **fastnoise-lite 1.1.1** — zero-dep noise (OpenSimplex2/Perlin/Cellular) for terrain variation and anomaly density
- **seahash 4.1.0** — fast seed hashing for procgen seed derivation (`hash(world_seed, domain) → u64`)
- **OnceLock** (std) — lazy-load game data at startup from RON files, no external loader crate
- **bevy-inspector-egui 0.36.0** (dev-only) — runtime ECS entity/resource inspector

## What We Deliberately Do NOT Use

- No `bevy::prelude::Event` — all inter-system communication uses `#[derive(Message)]` + `MessageWriter<T>` / `Messages<T>` (Bevy 0.18 Messages API)
- No async-trait crate — async fn in trait is native in Rust edition 2024
- No ECS plugin architecture — systems are registered directly in main.rs, not via Plugin structs
- No Bevy UI (Node-based) — all UI is egui; the native Bevy UI system is unused
- No external asset pipeline — sprites are pre-built sprite sheets, data is RON files

## Version Constraints

- **Bevy 0.18.1** is a hard pin — API differs significantly from 0.15/0.16 (Messages not Events, `Camera2d`, `OrthographicProjection::default_2d()`, `ScalingMode` path)
- **Rust edition 2024** — enables async fn in traits, new `use` semantics; minimum supported version is 1.87 (bumped from 1.85 due to pathfinding crate MSRV)
- **serde defaults** are mandatory on all save-game fields — `#[serde(default)]` or `#[serde(default = "fn")]` for backward-compatible save files
