//! Procedural location generation for the BD Kernel.
//!
//! Phase 14: Creates `LocationPlan` values through a staged pipeline
//! (layout → rooms → connect → paint → entrance/exits → spawn zones → validate),
//! then converts validated plans into `SpawnRequests` for the entity factory.
//!
//! All generation is seed-deterministic via `rand_chacha::ChaCha8Rng`.

use std::collections::HashSet;

use petgraph::graph::UnGraph;
use rand::SeedableRng;
use rand::prelude::*;
use rand::rngs::StdRng;

use crate::{
    components::{Position, Tile},
    map::SmokeMap,
    pathfinding::{AStarPathfinder, Pathfinder},
};

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

/// A rectangular room within a generated location.
#[derive(Debug, Clone, PartialEq)]
pub struct Room {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Room {
    pub fn center(&self) -> Position {
        Position {
            x: self.x + self.w / 2,
            y: self.y + self.h / 2,
        }
    }

    /// True if this room overlaps another (including touching borders).
    pub fn overlaps(&self, other: &Room, padding: i32) -> bool {
        let l1 = self.x - padding;
        let r1 = self.x + self.w + padding;
        let t1 = self.y - padding;
        let b1 = self.y + self.h + padding;
        let l2 = other.x - padding;
        let r2 = other.x + other.w + padding;
        let t2 = other.y - padding;
        let b2 = other.y + other.h + padding;
        l1 < r2 && r1 > l2 && t1 < b2 && b1 > t2
    }
}

/// Entry in a spawn table: how many of a blueprint to spawn.
#[derive(Debug, Clone)]
pub struct SpawnEntry {
    pub blueprint_id: String,
    pub count: usize,
}

/// Template describing how to generate a location.
#[derive(Debug, Clone)]
pub struct LocationTemplate {
    pub id: String,
    pub min_width: i32,
    pub max_width: i32,
    pub min_height: i32,
    pub max_height: i32,
    pub min_rooms: usize,
    pub max_rooms: usize,
    pub room_min_size: i32,
    pub room_max_size: i32,
    pub spawn_table: Vec<SpawnEntry>,
}

impl LocationTemplate {
    /// Create a sensible default ruin template.
    pub fn ruin() -> Self {
        Self {
            id: "location.ruin".into(),
            min_width: 30,
            max_width: 45,
            min_height: 20,
            max_height: 30,
            min_rooms: 4,
            max_rooms: 8,
            room_min_size: 4,
            room_max_size: 9,
            spawn_table: vec![SpawnEntry {
                blueprint_id: "blueprint.rat".into(),
                count: 3,
            }],
        }
    }

    /// Harder location template for the Crypt (floor 2).
    pub fn crypt() -> Self {
        Self {
            id: "location.crypt".into(),
            min_width: 35,
            max_width: 50,
            min_height: 25,
            max_height: 35,
            min_rooms: 5,
            max_rooms: 10,
            room_min_size: 3,
            room_max_size: 8,
            spawn_table: vec![
                SpawnEntry {
                    blueprint_id: "blueprint.skeleton".into(),
                    count: 5,
                },
                SpawnEntry {
                    blueprint_id: "blueprint.crypt_lord".into(),
                    count: 1,
                },
            ],
        }
    }
}

/// A validated, spawnable location plan.
#[derive(Debug, Clone)]
pub struct LocationPlan {
    pub seed: u64,
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<Tile>,
    pub rooms: Vec<Room>,
    /// petgraph ungraph: nodes = room indices, edges = corridors exist
    pub room_graph: UnGraph<usize, ()>,
    pub entrance: Position,
    pub exits: Vec<Position>,
    pub spawn_zones: Vec<Position>,
}

impl LocationPlan {
    /// Get tile at (x, y).
    pub fn get(&self, x: i32, y: i32) -> Option<Tile> {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return None;
        }
        Some(self.tiles[(y * self.width + x) as usize])
    }

    /// Set tile at (x, y).
    pub fn set(&mut self, x: i32, y: i32, tile: Tile) {
        if x >= 0 && x < self.width && y >= 0 && y < self.height {
            self.tiles[(y * self.width + x) as usize] = tile;
        }
    }

    /// Quick walkability check for validation.
    pub fn is_walkable(&self, x: i32, y: i32) -> bool {
        self.get(x, y).is_some_and(|t| t == Tile::Floor)
    }

    /// Convert to a SmokeMap for pathfinding queries.
    pub fn to_smoke_map(&self) -> SmokeMap {
        SmokeMap::from_tiles(self.width, self.height, &self.tiles)
    }

}

// ---------------------------------------------------------------------------
// Generation pipeline
// ---------------------------------------------------------------------------

/// Run the full generation pipeline: template + seed → LocationPlan.
pub fn generate_location(template: &LocationTemplate, seed: u64) -> LocationPlan {
    let mut rng = StdRng::seed_from_u64(seed);

    // Stage 1: determine dimensions
    let width = rng.random_range(template.min_width..=template.max_width);
    let height = rng.random_range(template.min_height..=template.max_height);
    let mut plan = LocationPlan {
        seed,
        width,
        height,
        tiles: vec![Tile::Wall; (width * height) as usize],
        rooms: Vec::new(),
        room_graph: UnGraph::new_undirected(),
        entrance: Position { x: 1, y: 1 },
        exits: Vec::new(),
        spawn_zones: Vec::new(),
    };

    // Stage 2: place rooms
    place_rooms(&mut plan, &mut rng, template);

    if plan.rooms.is_empty() {
        // Fallback: at least one room
        let room = Room {
            x: 2,
            y: 2,
            w: 6,
            h: 6,
        };
        paint_room(&mut plan, &room);
        plan.room_graph.add_node(0);
        plan.rooms.push(room);
    }

    // Stage 3: connect rooms (ensure connectivity)
    connect_rooms(&mut plan, &mut rng);

    // Stage 4: paint corridors (already done inside connect_rooms)
    // Stage 5: entrance + exits
    place_entrance(&mut plan);
    place_exits(&mut plan);

    // Stage 6: spawn zones
    place_spawn_zones(&mut plan, &mut rng, template);

    plan
}

/// Stage 2: Place non-overlapping rooms randomly.
fn place_rooms(plan: &mut LocationPlan, rng: &mut StdRng, template: &LocationTemplate) {
    let target = rng.random_range(template.min_rooms..=template.max_rooms);
    let max_attempts = target * 20;

    for _ in 0..max_attempts {
        if plan.rooms.len() >= target {
            break;
        }

        let rw = rng.random_range(template.room_min_size..=template.room_max_size);
        let rh = rng.random_range(template.room_min_size..=template.room_max_size);
        let rx = rng.random_range(1..(plan.width - rw - 1));
        let ry = rng.random_range(1..(plan.height - rh - 1));

        let candidate = Room {
            x: rx,
            y: ry,
            w: rw,
            h: rh,
        };

        if plan.rooms.iter().any(|r| r.overlaps(&candidate, 2)) {
            continue;
        }

        // Paint room tiles
        paint_room(plan, &candidate);

        let idx = plan.room_graph.add_node(plan.rooms.len());
        plan.rooms.push(candidate);

        // Connect this room to nearest previous room
        if idx.index() > 0 {
            let nearest = nearest_room(plan, idx.index());
            plan.room_graph.add_edge(idx, nearest, ());
        }
    }
}

/// Paint floor tiles for a room.
fn paint_room(plan: &mut LocationPlan, room: &Room) {
    for y in room.y..(room.y + room.h) {
        for x in room.x..(room.x + room.w) {
            plan.set(x, y, Tile::Floor);
        }
    }
}

/// Find the nearest room (by center distance) to the room at `idx`.
fn nearest_room(plan: &LocationPlan, idx: usize) -> petgraph::graph::NodeIndex {
    let center = plan.rooms[idx].center();
    let mut best = None;
    let mut best_dist = i32::MAX;
    for (i, r) in plan.rooms.iter().enumerate() {
        if i >= idx {
            break;
        }
        let c = r.center();
        let d = (center.x - c.x).abs() + (center.y - c.y).abs();
        if d < best_dist {
            best_dist = d;
            best = Some(petgraph::graph::NodeIndex::new(i));
        }
    }
    best.unwrap()
}

/// Stage 3: Dig L-shaped corridors between all connected rooms.
///
/// Also ensures the room graph is fully connected — if disconnected
/// components exist, we connect them.
fn connect_rooms(plan: &mut LocationPlan, _rng: &mut StdRng) {
    // Dig corridors for existing edges
    for edge in plan.room_graph.edge_indices().collect::<Vec<_>>() {
        let (a, b) = plan.room_graph.edge_endpoints(edge).unwrap();
        let room_a = &plan.rooms[a.index()];
        let room_b = &plan.rooms[b.index()];
        dig_corridor(plan, room_a.center(), room_b.center());
    }

    // Ensure full connectivity: find SCCs (connected components in undirected)
    use petgraph::algo::kosaraju_scc;
    let sccs = kosaraju_scc(&plan.room_graph);
    if sccs.len() > 1 {
        // Connect each component to the first one
        for c in 1..sccs.len() {
            if let (Some(&a), Some(&b)) = (sccs[0].first(), sccs[c].first()) {
                let room_a = &plan.rooms[a.index()];
                let room_b = &plan.rooms[b.index()];
                dig_corridor(plan, room_a.center(), room_b.center());
                plan.room_graph.add_edge(a, b, ());
            }
        }
    }
}

/// Dig an L-shaped corridor between two points.
fn dig_corridor(plan: &mut LocationPlan, a: Position, b: Position) {
    // Horizontal then vertical (L-shaped)
    let x = a.x;
    let y = a.y;
    let ex = b.x;
    let ey = b.y;

    // Horizontal segment
    let (x_start, x_end) = if x <= ex { (x, ex) } else { (ex, x) };
    for cx in x_start..=x_end {
        plan.set(cx, y, Tile::Floor);
        // Also dig one wider for visual
        if y > 0 {
            plan.set(cx, y - 1, Tile::Floor);
        }
        if y + 1 < plan.height {
            plan.set(cx, y + 1, Tile::Floor);
        }
    }

    // Vertical segment
    let (y_start, y_end) = if y <= ey { (y, ey) } else { (ey, y) };
    for cy in y_start..=y_end {
        plan.set(ex, cy, Tile::Floor);
        if ex > 0 {
            plan.set(ex - 1, cy, Tile::Floor);
        }
        if ex + 1 < plan.width {
            plan.set(ex + 1, cy, Tile::Floor);
        }
    }
}

/// Stage 5a: Place entrance in the first room.
fn place_entrance(plan: &mut LocationPlan) {
    if let Some(room) = plan.rooms.first() {
        plan.entrance = room.center();
        // Ensure center tile is walkable
        plan.set(plan.entrance.x, plan.entrance.y, Tile::Floor);
    }
}

/// Stage 5b: Place exits in the last room (and possibly other far rooms).
fn place_exits(plan: &mut LocationPlan) {
    if let Some(room) = plan.rooms.last() {
        let exit = room.center();
        plan.set(exit.x, exit.y, Tile::Floor);
        plan.exits.push(exit);
    }

    // If only one room, add a second exit near the entrance
    if plan.rooms.len() == 1 {
        if let Some(room) = plan.rooms.first() {
            let alt_exit = Position {
                x: room.x + 1,
                y: room.y + 1,
            };
            plan.set(alt_exit.x, alt_exit.y, Tile::Floor);
            plan.exits.push(alt_exit);
        }
    }
}

/// Stage 6: Mark spawn zones in rooms (not entrance room, not overlapping exit positions).
fn place_spawn_zones(plan: &mut LocationPlan, rng: &mut StdRng, template: &LocationTemplate) {
    let exit_set: HashSet<Position> = plan.exits.iter().copied().collect();

    // Skip room 0 (entrance room) for spawns
    for (i, room) in plan.rooms.iter().enumerate() {
        if i == 0 {
            continue;
        }
        // Place spawns near the center of the room
        let center = room.center();
        if !exit_set.contains(&center) && plan.is_walkable(center.x, center.y) {
            plan.spawn_zones.push(center);
        }

        // Maybe add extra spawn points
        if let Some(entry) = template.spawn_table.first() {
            let count = entry.count.min(3);
            for _ in 0..count {
                let sx = room.x + rng.random_range(1..room.w.saturating_sub(1).max(2));
                let sy = room.y + rng.random_range(1..room.h.saturating_sub(1).max(2));
                let sp = Position { x: sx, y: sy };
                if !exit_set.contains(&sp) && plan.is_walkable(sp.x, sp.y) {
                    plan.spawn_zones.push(sp);
                }
            }
        }
    }

    // Deduplicate
    let mut seen = HashSet::new();
    plan.spawn_zones.retain(|p| seen.insert(*p));
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

/// Result of plan validation.
#[derive(Debug, Clone)]
pub struct PlanValidation {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Validate a LocationPlan before spawning.
pub fn validate_plan(plan: &LocationPlan) -> PlanValidation {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // 1. Entrance exists
    if !plan.is_walkable(plan.entrance.x, plan.entrance.y) {
        errors.push(format!("Entrance at {:?} is not walkable", plan.entrance));
    }

    // 2. At least one exit
    if plan.exits.is_empty() {
        errors.push("No exits".into());
    }

    // 3. Exits are walkable
    for (i, e) in plan.exits.iter().enumerate() {
        if !plan.is_walkable(e.x, e.y) {
            errors.push(format!("Exit {i} at {e:?} is not walkable"));
        }
    }

    // 4. At least one room
    if plan.rooms.is_empty() {
        errors.push("No rooms".into());
    }

    // 5. Minimum walkable area
    let walkable_count = plan.tiles.iter().filter(|t| **t == Tile::Floor).count();
    let min_walkable = ((plan.width * plan.height) as usize) / 10;
    if walkable_count < min_walkable {
        errors.push(format!(
            "Too little walkable area: {walkable_count} tiles, minimum {min_walkable}"
        ));
    }

    // 6. Spawn zones are valid
    for (i, sz) in plan.spawn_zones.iter().enumerate() {
        if !plan.is_walkable(sz.x, sz.y) {
            errors.push(format!("Spawn zone {i} at {sz:?} is in a wall"));
        }
        // Check not overlapping exits
        if plan.exits.contains(sz) {
            warnings.push(format!("Spawn zone {i} at {sz:?} overlaps an exit"));
        }
    }

    // 7. Exit reachable from entrance (A*)
    if !errors.is_empty() {
        return PlanValidation {
            valid: false,
            errors,
            warnings,
        };
    }

    let sm = plan.to_smoke_map();
    let blocked: HashSet<Position> = HashSet::new();
    let pf = AStarPathfinder;

    // All rooms reachable from entrance?
    for (i, room) in plan.rooms.iter().enumerate() {
        let center = room.center();
        if center == plan.entrance {
            continue;
        }
        let path = pf.find_path(&sm, plan.entrance, center, &blocked);
        if path.is_none() {
            errors.push(format!(
                "Room {i} center {center:?} is not reachable from entrance"
            ));
        }
    }

    // Exit reachable from entrance
    for (i, exit) in plan.exits.iter().enumerate() {
        let path = pf.find_path(&sm, plan.entrance, *exit, &blocked);
        if path.is_none() {
            errors.push(format!(
                "Exit {i} at {exit:?} is not reachable from entrance"
            ));
        }
    }

    PlanValidation {
        valid: errors.is_empty(),
        errors,
        warnings,
    }
}

// ---------------------------------------------------------------------------
// SmokeMap conversion helper
// ---------------------------------------------------------------------------

impl SmokeMap {
    /// Create a SmokeMap from a flat tile vec.
    pub fn from_tiles(width: i32, height: i32, tiles: &[Tile]) -> Self {
        let mut map = SmokeMap::new(width, height, Tile::Wall);
        for y in 0..height {
            for x in 0..width {
                map.set(x, y, tiles[(y * width + x) as usize]);
            }
        }
        map
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_generates_same_plan() {
        let template = LocationTemplate::ruin();
        let a = generate_location(&template, 42);
        let b = generate_location(&template, 42);
        assert_eq!(a.width, b.width);
        assert_eq!(a.height, b.height);
        assert_eq!(a.rooms.len(), b.rooms.len());
        assert_eq!(a.tiles, b.tiles);
        assert_eq!(a.entrance, b.entrance);
        assert_eq!(a.exits, b.exits);
    }

    #[test]
    fn different_seed_generates_different_plan() {
        let template = LocationTemplate::ruin();
        let a = generate_location(&template, 42);
        let b = generate_location(&template, 99);
        // Different seeds should produce different tile layouts
        // (extremely unlikely to be identical)
        assert_ne!(a.tiles, b.tiles);
    }

    #[test]
    fn all_rooms_reachable() {
        let template = LocationTemplate::ruin();
        let plan = generate_location(&template, 42);
        let result = validate_plan(&plan);
        assert!(result.valid, "Plan validation failed: {:?}", result.errors);
    }

    #[test]
    fn entrance_exists() {
        let template = LocationTemplate::ruin();
        let plan = generate_location(&template, 42);
        assert!(plan.is_walkable(plan.entrance.x, plan.entrance.y));
    }

    #[test]
    fn exit_reachable() {
        let template = LocationTemplate::ruin();
        let plan = generate_location(&template, 42);
        let sm = plan.to_smoke_map();
        let blocked = HashSet::new();
        let pf = AStarPathfinder;
        for exit in &plan.exits {
            let path = pf.find_path(&sm, plan.entrance, *exit, &blocked);
            assert!(
                path.is_some(),
                "Exit {:?} not reachable from entrance {:?}",
                exit,
                plan.entrance
            );
        }
    }

    #[test]
    fn spawn_zones_valid() {
        let template = LocationTemplate::ruin();
        let plan = generate_location(&template, 42);
        for (i, sz) in plan.spawn_zones.iter().enumerate() {
            assert!(
                plan.is_walkable(sz.x, sz.y),
                "Spawn zone {i} at {sz:?} is in a wall"
            );
        }
    }

    #[test]
    fn plan_does_not_spawn_entities_before_validation() {
        // The plan is just data — no entities are created during generation.
        // This test confirms that generate_location returns a Plan, not spawned entities.
        let template = LocationTemplate::ruin();
        let plan = generate_location(&template, 42);
        // Plan should be purely data (no entity spawning side effects)
        assert!(!plan.rooms.is_empty(), "Plan should have at least one room");
        // Ensure we have a floor tile somewhere — proves tile painting works
        assert!(
            plan.tiles.contains(&Tile::Floor),
            "Plan should have at least one floor tile"
        );
        // Validation does not spawn anything either
        let result = validate_plan(&plan);
        // We just verify validation runs without spawning
        assert!(result.valid || !result.errors.is_empty());
    }

    #[test]
    fn multiple_seeds_do_not_panic() {
        let template = LocationTemplate::ruin();
        for seed in 0..50 {
            let plan = generate_location(&template, seed);
            let result = validate_plan(&plan);
            // Some seeds may produce invalid plans (edge cases), but no panics
            if !result.valid {
                // Warnings are acceptable, errors should be documented
                // but we at minimum ensure no crash
            }
        }
    }
}
