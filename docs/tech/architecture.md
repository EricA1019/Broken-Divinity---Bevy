# Technical Architecture

> **RECONCILE — NOT TECHNICAL AUTHORITY**
> The current technical authority is [Kernel.md](../../../Kernel.md). This
> document may describe superseded graphical dependencies.

This document defines the technology stack, crate dependencies, project structure, rendering pipeline, procedural generation strategy, and scalability approach for Broken Divinity. Every version listed was verified against crates.io as of April 2026.

---

## Table of Contents

1. [Dependency Table](#dependency-table)
2. [Bevy Feature Configuration](#bevy-feature-configuration)
3. [Cargo.toml Skeleton](#cargotoml-skeleton)
4. [Module Architecture](#module-architecture)
5. [Rendering Pipeline](#rendering-pipeline)
6. [Procedural Generation Strategy](#procedural-generation-strategy)
7. [Scalability Design](#scalability-design)
8. [Dev Workflow](#dev-workflow)

---

## Dependency Table

### Core Engine

| Crate | Version | Purpose | Why This | Alternatives Rejected |
|-------|---------|---------|----------|----------------------|
| `bevy` | 0.18.1 | ECS engine, 2D rendering, input, audio, state machines | Industry-standard Rust ECS engine. Native 2D support, Messages API, state machines. | Macroquad (no ECS), ggez (less mature), custom (too much work) |
| `bevy_egui` | 0.39.1 | All UI — panels, HUD, modals, menus | Immediate-mode GUI with full Bevy integration. Multi-pass rendering via `EguiPrimaryContextPass`. | Bevy native UI (verbose, less flexible for rapid iteration), raw egui (no Bevy integration) |
| `bevy_ecs_tilemap` | 0.18.1 | GPU-chunked tilemap rendering for dungeons, shelter, overworld | Entity-per-tile (ECS-native), chunked GPU batching, supports layers and sparse maps. | Manual `SpriteBundle` rendering (more code, no GPU chunking, worse performance at scale) |

### Algorithms & Procgen

| Crate | Version | Purpose | Why This | Alternatives Rejected |
|-------|---------|---------|----------|----------------------|
| `pathfinding` | 4.15.0 | A*, Dijkstra, BFS for AI movement and player pathfinding | Actively maintained (updated 26 days ago), comprehensive algorithms, well-benchmarked. | `bracket-pathfinding` 0.8.7 (stale 3yr, bundles FOV we don't want), hand-rolled (unnecessary when crate is this good) |
| `spade` | 2.15.1 | Delaunay triangulation for overworld road network generation | Actively maintained (12 days ago), robust geometric predicates, includes Voronoi and constrained Delaunay. | `delaunator` 1.0.2 (stale 3yr, no Voronoi), hand-rolled MST (less elegant road networks) |
| `fastnoise-lite` | 1.1.1 | Noise generation for terrain variation, dungeon atmosphere, anomaly placement | Zero dependencies, 2.4K LOC, OpenSimplex2/Perlin/Cellular/Value + domain warp. Exactly what we need, nothing more. | `noise` 0.9.0 (heavier, more deps, overkill), `bracket-noise` 0.8.7 (stale) |
| `seahash` | 4.1.0 | Fast seed hashing for procgen seed derivation | Mature, 96M+ downloads, 3-20% faster than xxHash. Perfect for `hash(world_seed, domain_string) → u64`. | `xxhash-rust` (newer but seahash is proven and stable), raw XOR rotation (less distribution quality) |

### RNG & Serialization

| Crate | Version | Purpose | Why This | Alternatives Rejected |
|-------|---------|---------|----------|----------------------|
| `rand` | 0.10.0 | RNG traits and distributions | Industry standard. Edition 2024 compatible. | Nothing — this is the Rust RNG ecosystem |
| `rand_chacha` | 0.10.0 | Deterministic RNG engine (ChaCha8Rng) | Platform-independent, reproducible, fast. Same seed = same output on every machine. | `StdRng` (not guaranteed deterministic across versions), `SmallRng` (not reproducible) |
| `serde` | 1.0.228 | Serialization framework for save/load and data files | Industry standard. 900M+ downloads. | Nothing — this IS Rust serialization |
| `ron` | 0.12.1 | RON data format for game data files (rosters, dialogue, items) | Human-readable, Rust-native syntax, perfect for game data. Actively maintained (6 days ago). | JSON (verbose, no comments), TOML (poor for nested game data), YAML (fragile whitespace) |

### Dev Tools (not shipped)

| Crate | Version | Purpose | Why This | Alternatives Rejected |
|-------|---------|---------|----------|----------------------|
| `bevy-inspector-egui` | 0.36.0 | Runtime ECS entity/resource inspector | See all entities, components, resources live. Invaluable for debugging ECS state. | Print debugging (slow, incomplete), custom debug panels (unnecessary duplication) |

### Hand-Rolled (no crate)

| System | LOC Estimate | Why Hand-Roll |
|--------|-------------|---------------|
| **FOV (Symmetric Shadowcasting)** | ~150 | Simple algorithm, no stale dependency, full control over tile opacity rules. `bracket-pathfinding` bundles this but is 3yr stale and brings unwanted baggage. |
| **BSP Dungeon Generation** | ~200 | Core game mechanic — must own it for theme overlays, room typing, and custom corridor logic. No crate does exactly what we need. |
| **Shelter Layout Generation** | ~150 | BSP variant for compound generation. Too project-specific for a crate. |

---

## Bevy Feature Configuration

### The Problem

Bevy's default features include the entire 3D pipeline: PBR materials, glTF loading, 3D lighting, anti-aliasing LUTs, morph targets, mikktspace tangents. We use **none of this**. Default features compile ~72 feature flags.

### The Solution

Use `default-features = false` with the `2d` meta-feature:

```toml
bevy = { version = "0.18.1", default-features = false, features = ["2d"] }
```

### What `2d` Gives Us

The `2d` feature flag transitively enables:

| Feature | Provides |
|---------|----------|
| `2d_bevy_render` | Core 2D rendering pipeline, sprites, core pipeline, gizmos, post-process |
| `audio` | `bevy_audio` + vorbis codec |
| `default_app` | Asset loading, input focus, logging, state machines, windowing |
| `default_platform` | Gamepad (gilrs), winit, default font, multi-threading, x11/wayland |
| `picking` | Sprite picking, UI picking, mesh picking |
| `scene` | Scene serialization |
| `ui` | `bevy_ui` + UI rendering |

### What Gets Eliminated

By NOT using the `3d` feature, we skip:

| Eliminated Feature | Savings |
|-------------------|---------|
| `bevy_pbr` | PBR materials, shadow maps, environment maps |
| `bevy_gltf` | glTF model loading |
| `bevy_light` | 3D lighting system |
| `bevy_mikktspace` | Tangent space calculation |
| `bevy_anti_alias` | FXAA, MSAA, TAA, SMAA |
| `3d_api` | 3D mesh API, morph targets, tonemapping LUTs, KTX2, zstd |
| `gltf_animation` | glTF animation import |

This meaningfully reduces compile time and binary size.

### bevy_ui Note

The `2d` feature enables `bevy_ui` (Bevy's native Node-based UI). We don't use it — all UI goes through `bevy_egui`. However, disabling it would require dropping `ui` from the feature chain, which also removes `ui_picking`. We keep it enabled and simply don't import it. The compile cost is minimal.

---

## Cargo.toml Skeleton

```toml
[package]
name = "broken_divinity"
version = "0.1.0"
edition = "2024"
rust-version = "1.87"

[features]
default = []
dev = ["bevy/dynamic_linking"]

[dependencies]
# --- Engine ---
bevy = { version = "0.18.1", default-features = false, features = ["2d"] }
bevy_egui = "0.39.1"
bevy_ecs_tilemap = "0.18.1"

# --- Algorithms & Procgen ---
pathfinding = "4.15"
spade = "2.15"
fastnoise-lite = "1.1"
seahash = "4.1"

# --- RNG ---
rand = "0.10"
rand_chacha = "0.10"

# --- Serialization ---
serde = { version = "1.0", features = ["derive"] }
ron = "0.12"

[dev-dependencies]
bevy-inspector-egui = { version = "0.36.0", default-features = false }

# --- Profiles ---
[profile.dev]
opt-level = 1               # slight optimization for playable dev builds

[profile.dev.package."*"]
opt-level = 3               # full optimization for dependencies (compile once)

[profile.release]
opt-level = 3
lto = "thin"                # link-time optimization for smaller/faster binaries
strip = true                # strip debug symbols from release binary

# --- Lints ---
[lints.clippy]
all = { level = "warn" }
pedantic = { level = "warn", priority = -1 }
module_name_repetitions = "allow"     # we use combat::CombatStats intentionally
cast_possible_truncation = "allow"    # tile math uses i32/f32 casts frequently
cast_precision_loss = "allow"         # acceptable for game math
cast_sign_loss = "allow"              # tile coordinates cross signed/unsigned
needless_pass_by_value = "allow"      # Bevy systems require owned params
```

### Usage

```bash
# Dev build (fast iteration with dynamic linking)
cargo run --features dev

# Standard build
cargo run

# Release build
cargo build --release

# Tests
cargo test

# Lint
cargo clippy -- -W clippy::all
```

### Why `rust-version = "1.87"`

- Rust edition 2024 requires ≥1.85
- `pathfinding` 4.15.0 requires ≥1.87
- Our system runs 1.91 — no practical constraint
- Keeps MSRV honest for anyone cloning the repo

---

## Module Architecture

### 5-Tier Dependency Graph

Modules are organized in a strict hierarchy. Lower tiers **never** import from higher tiers. Cross-tier communication uses Resources or Messages.

```
Tier 5 — Orchestration     input, ui, main
   ↑                       (import from many modules — they are the glue)
Tier 4 — Meta Systems      dungeon, settlement, shelter_map, survivors, overworld, save_load
   ↑                       (coordinate multiple mechanics into gameplay loops)
Tier 3 — Behaviors         anomalies, sanity, hud, ai, rosters, stealth
   ↑                       (reactive systems that read game state and respond)
Tier 2 — Mechanics         combat, abilities, economy, perks, map, procgen, fov
   ↑                       (core game rules and algorithms)
Tier 1 — Domain Data       skills, equipment, items, dialogue, crafting
   ↑                       (data definitions and type catalogs)
Tier 0 — Core              components, states, resources
                            (shared types — no cross-imports, no game logic)
```

### Tier Rules

1. **Tier 0 imports NOTHING** from the project — only external crates
2. A module may import from **same tier or lower**, never higher
3. **Tier 5** (input, ui, main) are the only modules allowed to import broadly
4. New modules must be assigned a tier before implementation
5. Upward communication uses **Resources** (shared state) or **Messages** (events) — never direct function calls

### MVP Module Set (~15 modules)

| Module | Tier | MVP Scope |
|--------|------|-----------|
| `components.rs` | 0 | All shared ECS components: Health, Position, CombatStats, etc. |
| `states.rs` | 0 | AppState + TurnState enums |
| `resources.rs` | 0 | All shared resources: GameLog, GameTime, PendingAction, etc. |
| `skills.rs` | 1 | SkillId enum, skill definitions, XP tables |
| `equipment.rs` | 1 | Weapon/armor structs, tier tables, damage types |
| `items.rs` | 1 | Item definitions, loot tables |
| `combat.rs` | 2 | d100 skill checks, damage formulas, attack pipeline |
| `abilities.rs` | 2 | Attack, Shoot, First Aid, Sprint |
| `map.rs` | 2 | Map struct, TileType, tile queries |
| `procgen.rs` | 2 | BSP dungeon generation, room typing, themed decoration |
| `fov.rs` | 2 | Symmetric shadowcasting, visibility updates |
| `ai.rs` | 3 | MeleeCharge + RangedKite behaviors, alert states |
| `sanity.rs` | 3 | Single-track sanity bar, threshold effects |
| `dungeon.rs` | 4 | Dungeon lifecycle: enter, regen, exit, depth tracking |
| `settlement.rs` | 4 | Station types, resource production/consumption |
| `shelter_map.rs` | 4 | Shelter tilemap, room management, construction |
| `survivors.rs` | 4 | Survivor entities, needs, task assignment |
| `overworld.rs` | 4 | World graph, node travel, weather, encounters |
| `save_load.rs` | 4 | SaveGame serialization, serde round-trips |
| `input.rs` | 5 | Player input routing |
| `ui.rs` | 5 | egui draw/process split, all panels |
| `main.rs` | 5 | System registration by AppState lifecycle |

### Phase 2 Module Additions

| Module | Tier | Adds |
|--------|------|------|
| `economy.rs` | 2 | Crafting system, trade, advanced resource flow |
| `perks.rs` | 2 | Full perk trees for all 8 skills |
| `stealth.rs` | 3 | Facing, FOV cones, noise propagation, alert states |
| `rosters.rs` | 3 | Enemy roster loading, themed spawn tables |
| `dialogue.rs` | 1 | Dialogue tree loading and display |

### Phase 3 Module Additions

| Module | Tier | Adds |
|--------|------|------|
| `crafting.rs` | 1 | Blueprint system, material requirements |
| `anomalies.rs` | 3 | Anomaly types, contact effects, cleansing |
| `hud.rs` | 3 | Dedicated HUD bar rendering (health, sanity, action bars) |
| `expeditions.rs` | 4 | Survivor team dispatch, off-screen resolution |

### Cross-Tier Communication

```
Tier 2 (combat.rs)                    Tier 3 (ai.rs)
    │                                     │
    ├── writes AttackEvent ──────────────►├── reads AttackEvent
    │   (via MessageWriter)               │   (via Messages<T>::drain())
    │                                     │
    └── reads CombatStats ◄──────────────┘── reads CombatStats
        (via Query<&CombatStats>)             (via Query<&CombatStats>)
```

**Resources** (Tier 0) are the shared mailbox — any system can read/write them.
**Messages** (defined at any tier, registered in main.rs) are the event bus — producers write, consumers drain.

Neither pattern requires the consumer to import the producer module. The Message type lives in the module that defines it, and main.rs registers it.

---

## Rendering Pipeline

### Three-Layer Stack

```
┌─────────────────────────────────────┐
│  Layer 3: egui overlay              │  ← EguiPrimaryContextPass schedule
│  (panels, HUD, modals, menus)       │     All UI goes through bevy_egui
├─────────────────────────────────────┤
│  Layer 2: Entity sprites            │  ← Bevy 2D sprites (16×16 px)
│  (player, enemies, items, NPCs)     │     Standard SpriteBundle / TextureAtlas
├─────────────────────────────────────┤
│  Layer 1: Tilemap                   │  ← bevy_ecs_tilemap
│  (dungeon, shelter, overworld)      │     ASCII glyphs rendered as tile sprites
└─────────────────────────────────────┘
       Camera2d + OrthographicProjection::default_2d()
```

### Layer 1: Tilemap (bevy_ecs_tilemap)

Tilemaps for all three gamestates that use grid maps:

| State | Map Size | Tile Content |
|-------|----------|-------------|
| `InGame` (dungeon) | ~80×50 per floor | Wall, Floor, DoorClosed, DoorOpen, StairsUp, StairsDown |
| `AtShelter` | ~40×30 | Wall, Floor, Door, Station footprint tiles |
| Overworld roads | Variable segments | Terrain tiles during tile-walking segments |

Each tile is an entity with `TileBundle`. Per-tile components enable:
- FOV visibility toggling (revealed/visible/hidden)
- Damage tracking (wall HP for raids)
- Theme-specific texture swapping

Tiles use a sprite sheet where each glyph (ASCII character) maps to an atlas index. Theme overlays swap the atlas or tint color.

### Layer 2: Entity Sprites

Entities (player, enemies, items, NPCs) render as 16×16 sprites at tile positions. They use standard Bevy `Sprite` with `TextureAtlas`.

```rust
// Entity rendering approach
commands.spawn((
    Sprite {
        image: sprite_sheet.clone(),
        texture_atlas: Some(TextureAtlas {
            layout: atlas_layout.clone(),
            index: glyph_index,
        }),
        ..default()
    },
    Transform::from_xyz(tile_x * TILE_SIZE, tile_y * TILE_SIZE, ENTITY_Z),
    // ... ECS components
));
```

Z-ordering: Tilemap at z=0, items at z=1, enemies at z=2, player at z=3. This ensures entities render atop tiles and the player renders atop enemies.

### Layer 3: egui Overlay

All UI goes through `bevy_egui`. Systems are split:

- **Draw system**: Runs in `EguiPrimaryContextPass`. Reads game state, renders egui widgets, captures user actions into a `UiAction` resource.
- **Process system**: Runs in `Update`. Reads `UiAction`, mutates game state.

This split prevents borrowing conflicts (draw needs `&World`, process needs `&mut World`).

### Camera

```rust
commands.spawn((
    Camera2d,
    OrthographicProjection::default_2d(),
));
```

Camera follows the player entity. Smooth lerp tracking at MVP, snapping to tile boundaries optional later. Camera zoom allows seeing more/less of the map.

---

## Procedural Generation Strategy

### Seed Architecture

A single `u64` world seed entered at new-game (or randomized) derives all subsystem seeds. No RNG instance is ever shared between systems.

```
world_seed: u64
    │
    ├── seahash("dungeon", dungeon_id) ──→ ChaCha8Rng ──→ dungeon map tiles
    ├── seahash("overworld")           ──→ ChaCha8Rng ──→ node placement + roads
    ├── seahash("weather", day)        ──→ ChaCha8Rng ──→ daily weather roll
    ├── seahash("factions")            ──→ ChaCha8Rng ──→ proc-gen faction traits
    ├── seahash("encounter", road_id)  ──→ ChaCha8Rng ──→ overworld encounter rolls
    └── seahash("loot", context_id)    ──→ ChaCha8Rng ──→ loot table rolls
```

### Seed Derivation

```rust
use seahash::hash;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn derive_rng(world_seed: u64, domain: &str, id: u64) -> ChaCha8Rng {
    let combined = format!("{world_seed}:{domain}:{id}");
    let derived_seed = hash(combined.as_bytes());
    ChaCha8Rng::seed_from_u64(derived_seed)
}
```

### Determinism Contracts

1. **Same seed → same output.** Always. On every platform.
2. **No floating-point in generation decisions.** Integer math for room placement, corridor routing, entity spawning. Float only for rendering positions.
3. **Separable domains.** Regenerating weather doesn't change dungeon layout. Each domain gets its own derived seed.
4. **Reproducible.** `ChaCha8Rng` produces identical output on all platforms (it's a cryptographic PRNG with standardized output).
5. **Minimal save data.** Only `world_seed + player_decisions` need to be saved. The world can be regenerated from the seed.

### Dungeon Generation (BSP)

Algorithm summary (full detail in [docs/gameplay/procgen.md](../gameplay/procgen.md)):

```
1. Fill 80×50 grid with Wall
2. Recursively split into BSP partitions (min 8×8 leaf)
3. Place rooms (5×5 min) in each leaf with 1-tile wall buffer
4. Connect sibling rooms with L-shaped corridors (random bend point)
5. Place doors at corridor-room chokepoints (60% chance)
6. Place StairsUp (first room center) and StairsDown (farthest floor tile)
7. Assign room types: Empty(30%), Loot(20%), Enemy(25%), Hazard(10%), Objective(5%), Mixed(10%)
8. Apply theme overlay (tile textures, decoration sprites, hazard tiles)
9. Spawn themed enemies from rosters (scales with depth)
10. Spawn loot, decorations, anomalies
```

### Overworld Generation (Delaunay + MST)

Uses `spade` for triangulation:

```
1. Place shelter node at origin
2. Place dungeon/ruins/crossroads/landmark nodes via Poisson disk sampling in difficulty bands
3. Delaunay triangulate all nodes (spade::DelaunayTriangulation)
4. Extract minimum spanning tree (ensures full connectivity)
5. Add ~30% of remaining Delaunay edges (creates alternate routes, loops)
6. Prune unnaturally long edges
7. Store as adjacency list: Vec<(NodeId, NodeId, distance)>
```

The Delaunay triangulation produces naturally-spaced, non-crossing road networks. MST guarantees connectivity. Random extra edges create meaningful route choices.

### Noise Usage (fastnoise-lite)

| Application | Noise Type | Purpose |
|------------|-----------|---------|
| Dungeon room textures | Cellular | Variation in floor tile appearance (cracked, worn, clean) |
| Overworld terrain | OpenSimplex2 | Elevation hints for terrain painting along road tiles |
| Anomaly density | Perlin | Dense/sparse anomaly regions within dungeons |
| Weather probability | Value | Regional weather bias (some overworld zones rainier than others) |

Noise is aesthetic and probabilistic — it never determines structural layout (that's BSP/Delaunay). This keeps generation deterministic and debuggable.

### FOV (Symmetric Shadowcasting)

Hand-rolled ~150 LOC implementation. The algorithm:

```
For each of the 8 octants around the viewer:
    Cast rows outward from center
    Track which angles are blocked by opaque tiles
    Mark visible tiles
    Symmetric: if A sees B, then B sees A
```

Tile opacity is queried from `TileType::is_opaque()`. Works for dungeons, shelter, and overworld segments. Entities in non-visible tiles are hidden.

### Faction Generation

At world creation, proc-gen factions are rolled from the faction seed. See [docs/gameplay/procgen.md](../gameplay/procgen.md) for the full algorithm:

```
1. Roll count: 2-3 proc-gen factions
2. For each: roll archetype (Puritan 25%, Military 25%, Commune 20%, Cult 15%, Traders 15%)
3. Generate name from archetype naming patterns
4. Roll faction traits (1-2): Aggressive/Defensive/Isolationist/Expansionist, Thaumic-friendly/hostile
5. Assign home node and generate named NPCs
```

---

## Scalability Design

### How the Architecture Grows

The codebase is designed to scale from MVP (~15 modules) to Phase 3 (~30 modules) without refactoring the foundation. Each scaling mechanism is built into the initial architecture:

### 1. Data-Driven Content (RON + OnceLock)

Game content lives in `.ron` files under `assets/data/`, not in Rust code:

```
assets/data/
    rosters.ron         # enemy definitions, spawn tables
    items.ron           # equipment, consumables, quest items
    dialogue.ron        # dialogue trees
    factions.ron        # hardcoded faction definitions
    abilities.ron       # ability definitions and costs
```

**Scaling**: Adding a new enemy, item, dungeon theme, or dialogue tree is a data change, not a code change. New content ships without recompilation.

**Loading pattern**:
```rust
use std::sync::OnceLock;

fn load_rosters() -> Option<&'static RosterData> {
    static DATA: OnceLock<Option<RosterData>> = OnceLock::new();
    DATA.get_or_init(|| {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets/data/rosters.ron");
        let raw = std::fs::read_to_string(&path).ok()?;
        ron::from_str::<RosterData>(&raw).ok()
    }).as_ref()
}
```

### 2. Message Pipeline (Decoupled Communication)

Systems communicate through Messages (Bevy 0.18's event replacement). Producers and consumers are unaware of each other:

```rust
// combat.rs defines and writes
#[derive(Message)]
pub struct DamageEvent { pub target: Entity, pub amount: i32, pub damage_type: DamageType }

// sanity.rs reads — no import of combat.rs needed
fn on_damage(messages: Messages<DamageEvent>) {
    for event in messages.drain() {
        // Apply sanity effects for witnessing violence
    }
}
```

**Scaling**: Adding a new consumer (e.g., "screen shake on critical hits") means adding one new system that reads `DamageEvent`. No existing code changes. Adding a new producer (e.g., "trap damage") means writing `DamageEvent` from the new system. No existing code changes.

### 3. State Machine Gating

Systems are gated to AppState and TurnState. Adding new game modes means adding new states and registering new systems against them:

```rust
// MVP states
enum AppState { Loading, MainMenu, Overworld, AtShelter, InGame, DeathTransition, GameOver }

// Phase 2: add AtSettlement for outpost management
// Phase 3: add InExpedition for off-screen resolution
// Each new state gets its own systems without touching existing ones

.add_systems(Update, expedition_tick.run_if(in_state(AppState::InExpedition)))
```

**Scaling**: New game loops are additive. Existing systems never fire in states they weren't designed for.

### 4. Save Compatibility (serde defaults)

Every new field on a saved struct uses `#[serde(default)]`:

```rust
#[derive(Serialize, Deserialize)]
pub struct SavePlayerState {
    pub health: i32,           // MVP
    #[serde(default)]
    pub corruption: f32,       // Added in Phase 2 — old saves load as 0.0
    #[serde(default = "default_faction_rep")]
    pub faction_rep: HashMap<String, i32>,  // Added in Phase 2
}
```

**Scaling**: Players never lose save files when the game updates. New fields silently default. Old fields persist.

### 5. Module Tier System

The 5-tier graph enforces dependency direction at the architecture level:

- **Adding a new system** (e.g., crafting): Determine tier (Tier 1-2), implement, register in main.rs. Existing modules don't need changes.
- **Adding a new entity archetype** (e.g., vehicles): Define components in Tier 0 (`components.rs`), implement behavior in Tier 2-3, spawn from Tier 4. Existing archetypes are untouched.
- **Moving data downward** is always safe (e.g., moving a type from Tier 2 to Tier 0). Moving data upward is always a violation.

### What This Means in Practice

| Growth Vector | Mechanism | Impact on Existing Code |
|--------------|-----------|------------------------|
| New enemy types | RON data + spawn table entry | Zero code changes |
| New dungeon theme | RON tile mappings + decoration pool | Minimal (theme enum variant + procgen overlay) |
| New combat ability | RON definition + ability system handler | One new match arm in ability resolver |
| New UI panel | egui draw/process function pair | Register in main.rs, nothing else |
| New game state | AppState variant + state-gated systems | Zero impact on existing systems |
| New save fields | serde(default) annotation | Backward compatible automatically |
| New proc-gen system | Derived seed + ChaCha8Rng | Isolated — can't affect other generation |

---

## Dev Workflow

### Runtime ECS Inspector

`bevy-inspector-egui` is a dev-dependency that provides a world inspector plugin:

```rust
// Only in dev builds
#[cfg(feature = "dev")]
app.add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new());
```

This gives a live view of all entities, components, and resources. Extremely valuable for debugging:
- "Why isn't this enemy moving?" → Check its components live
- "Why is production zero?" → Inspect the station entity's resource outputs
- "What state is the game in?" → See AppState and TurnState resources

### Fast Iteration (Dynamic Linking)

```bash
cargo run --features dev
```

The `dev` feature enables `bevy/dynamic_linking`, which dynamically links Bevy instead of statically compiling it. This cuts incremental rebuild times from ~15-30s to ~2-5s after the first build.

**Never ship with dynamic linking.** `cargo build --release` uses the default features (no `dev`), which statically links everything.

### Clippy Configuration

The `[lints.clippy]` section in Cargo.toml enables `warn` on `all` and `pedantic` with specific allows for Bevy-idiomatic patterns:

- `module_name_repetitions`: We intentionally use `combat::CombatStats`
- `cast_possible_truncation`: Tile math crosses `i32`/`f32` frequently
- `needless_pass_by_value`: Bevy systems take owned `Query<>`, `Res<>`, etc.

### Test Patterns

Tests use `MinimalPlugins` (no rendering) and `run_system_once` for isolated system testing:

```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    #[test]
    fn damage_reduces_health() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let entity = app.world_mut().spawn(Health { current: 100, max: 100 }).id();

        app.world_mut().run_system_once(move |mut query: Query<&mut Health>| {
            let mut hp = query.get_mut(entity).unwrap();
            hp.current -= 25;
        });

        let hp = app.world().get::<Health>(entity).unwrap();
        assert_eq!(hp.current, 75);
    }
}
```

Tests live inside each module as `#[cfg(test)] mod tests` — not in a separate `tests/` directory.

### Directory Structure

```
broken_divinity/
├── Cargo.toml
├── src/
│   ├── main.rs              # system registration by AppState
│   ├── components.rs         # Tier 0: all shared ECS components
│   ├── states.rs             # Tier 0: AppState + TurnState
│   ├── resources.rs          # Tier 0: all shared resources
│   ├── skills.rs             # Tier 1: skill definitions
│   ├── equipment.rs          # Tier 1: weapon/armor types
│   ├── items.rs              # Tier 1: item definitions
│   ├── combat.rs             # Tier 2: d100, damage, attack pipeline
│   ├── abilities.rs          # Tier 2: ability resolution
│   ├── map.rs                # Tier 2: Map struct, tile queries
│   ├── procgen.rs            # Tier 2: BSP dungeon generation
│   ├── fov.rs                # Tier 2: symmetric shadowcasting
│   ├── ai.rs                 # Tier 3: enemy behaviors
│   ├── sanity.rs             # Tier 3: sanity bar, threshold effects
│   ├── dungeon.rs            # Tier 4: dungeon lifecycle
│   ├── settlement.rs         # Tier 4: stations, production
│   ├── shelter_map.rs        # Tier 4: shelter tilemap
│   ├── survivors.rs          # Tier 4: survivor entities, needs
│   ├── overworld.rs          # Tier 4: world graph, travel
│   ├── save_load.rs          # Tier 4: save/load serialization
│   ├── input.rs              # Tier 5: input routing
│   └── ui.rs                 # Tier 5: egui panels
├── assets/
│   ├── data/                 # RON game data files
│   │   ├── rosters.ron
│   │   ├── items.ron
│   │   └── ...
│   └── sprites/              # sprite sheets (16×16 tiles)
│       ├── entities.png
│       ├── tiles_urban.png
│       ├── tiles_underground.png
│       └── tiles_military.png
└── docs/                     # design documentation (you are here)
```

See also: [gameplay/combat.md](../gameplay/combat.md) for the d100 system, [gameplay/colony.md](../gameplay/colony.md) for settlement mechanics, [gameplay/procgen.md](../gameplay/procgen.md) for generation algorithms, [gameplay/phase-roadmap.md](../gameplay/phase-roadmap.md) for MVP → Phase 2 → Phase 3 scope.
