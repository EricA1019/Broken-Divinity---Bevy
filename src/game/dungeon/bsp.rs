//! BSP dungeon generator.
//!
//! Pure function: `(width, height, seed) → DungeonFloor`.
//! No ECS dependencies — returns a data structure that the spawn system consumes.

use rand::RngExt;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::core::components::TileKind;

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// Axis-aligned rectangle used for rooms and BSP splits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }

    pub fn center(&self) -> (i32, i32) {
        (self.x + self.w / 2, self.y + self.h / 2)
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.w
            && self.x + self.w > other.x
            && self.y < other.y + other.h
            && self.y + self.h > other.y
    }
}

/// The output of the BSP generator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DungeonFloor {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Vec<TileKind>>,
    pub rooms: Vec<Rect>,
    pub spawn_point: (i32, i32),
}

impl DungeonFloor {
    /// Returns true if the given (x, y) coordinate is in-bounds and not a wall.
    pub fn is_walkable(&self, x: i32, y: i32) -> bool {
        x >= 0
            && y >= 0
            && (x as usize) < self.width
            && (y as usize) < self.height
            && self.tiles[y as usize][x as usize] != TileKind::Wall
    }

    /// BFS outward from (x, y) to find the nearest walkable tile.
    /// Returns `None` only if the entire map has no walkable tile.
    pub fn find_nearest_walkable(&self, x: i32, y: i32) -> Option<(i32, i32)> {
        use std::collections::VecDeque;

        let mut visited = vec![vec![false; self.width]; self.height];
        let mut queue = VecDeque::new();

        // Clamp starting search to bounds
        let sx = x.clamp(0, self.width as i32 - 1) as usize;
        let sy = y.clamp(0, self.height as i32 - 1) as usize;
        visited[sy][sx] = true;
        queue.push_back((sx, sy));

        while let Some((cx, cy)) = queue.pop_front() {
            if self.tiles[cy][cx] != TileKind::Wall {
                return Some((cx as i32, cy as i32));
            }
            for (dx, dy) in &[(0i32, 1i32), (0, -1), (1, 0), (-1, 0)] {
                let nx = cx as i32 + dx;
                let ny = cy as i32 + dy;
                if nx >= 0
                    && ny >= 0
                    && (nx as usize) < self.width
                    && (ny as usize) < self.height
                    && !visited[ny as usize][nx as usize]
                {
                    visited[ny as usize][nx as usize] = true;
                    queue.push_back((nx as usize, ny as usize));
                }
            }
        }
        None
    }

    /// Validate `spawn_point` and return a corrected one if needed.
    /// Returns `(x, y, was_adjusted)`.
    pub fn validated_spawn_point(&self) -> (i32, i32, bool) {
        let (sx, sy) = self.spawn_point;
        if self.is_walkable(sx, sy) {
            return (sx, sy, false);
        }
        if let Some((fx, fy)) = self.find_nearest_walkable(sx, sy) {
            (fx, fy, true)
        } else {
            // Absolute last resort — should never happen with a valid BSP floor
            (1, 1, true)
        }
    }
}

// ---------------------------------------------------------------------------
// BSP tree
// ---------------------------------------------------------------------------

#[derive(Debug)]
enum BspNode {
    Leaf {
        rect: Rect,
    },
    Split {
        left: Box<BspNode>,
        right: Box<BspNode>,
        _split_horizontal: bool,
    },
}

impl BspNode {
    fn leaf(rect: Rect) -> Self {
        BspNode::Leaf { rect }
    }

    /// Recursively split until nodes are too small or max depth reached.
    fn split(self, rng: &mut ChaCha8Rng, min_size: i32, depth: u32) -> Self {
        if depth == 0 {
            return self;
        }
        let BspNode::Leaf { rect } = self else {
            return self;
        };

        let can_split_h = rect.h > min_size * 2;
        let can_split_v = rect.w > min_size * 2;

        if !can_split_h && !can_split_v {
            return BspNode::Leaf { rect };
        }

        let split_horizontal = if can_split_h && can_split_v {
            rng.random_bool(0.5)
        } else {
            can_split_h
        };

        if split_horizontal {
            let split_at = rng.random_range(min_size..=(rect.h - min_size));
            let top = Rect::new(rect.x, rect.y, rect.w, split_at);
            let bottom = Rect::new(rect.x, rect.y + split_at, rect.w, rect.h - split_at);
            BspNode::Split {
                left: Box::new(BspNode::leaf(top).split(rng, min_size, depth - 1)),
                right: Box::new(BspNode::leaf(bottom).split(rng, min_size, depth - 1)),
                _split_horizontal: true,
            }
        } else {
            let split_at = rng.random_range(min_size..=(rect.w - min_size));
            let left = Rect::new(rect.x, rect.y, split_at, rect.h);
            let right = Rect::new(rect.x + split_at, rect.y, rect.w - split_at, rect.h);
            BspNode::Split {
                left: Box::new(BspNode::leaf(left).split(rng, min_size, depth - 1)),
                right: Box::new(BspNode::leaf(right).split(rng, min_size, depth - 1)),
                _split_horizontal: false,
            }
        }
    }

    /// Collect all leaf rectangles.
    fn collect_leaves(&self, out: &mut Vec<Rect>) {
        match self {
            BspNode::Leaf { rect } => out.push(*rect),
            BspNode::Split { left, right, .. } => {
                left.collect_leaves(out);
                right.collect_leaves(out);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Generator
// ---------------------------------------------------------------------------

/// Minimum room dimension (inner area after border).
const MIN_ROOM_SIZE: i32 = 5;
/// Maximum BSP recursion depth.
const MAX_DEPTH: u32 = 6;
/// Padding inside each leaf for the carved room.
const ROOM_PADDING: i32 = 1;

pub fn generate_floor(width: usize, height: usize, seed: u64) -> DungeonFloor {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);

    // 1. Build BSP tree
    let root = BspNode::leaf(Rect::new(0, 0, width as i32, height as i32));
    let root = root.split(&mut rng, MIN_ROOM_SIZE + ROOM_PADDING * 2, MAX_DEPTH);

    let mut leaves = Vec::new();
    root.collect_leaves(&mut leaves);

    // 2. Carve rooms inside each leaf
    let mut tiles = vec![vec![TileKind::Wall; width]; height];
    let mut rooms: Vec<Rect> = Vec::new();

    for leaf in &leaves {
        // Room must be at least MIN_ROOM_SIZE in each dimension
        let room_w =
            rng.random_range(MIN_ROOM_SIZE..=(leaf.w - ROOM_PADDING * 2).max(MIN_ROOM_SIZE));
        let room_h =
            rng.random_range(MIN_ROOM_SIZE..=(leaf.h - ROOM_PADDING * 2).max(MIN_ROOM_SIZE));
        let room_x = leaf.x
            + rng.random_range(ROOM_PADDING..=(leaf.w - room_w - ROOM_PADDING).max(ROOM_PADDING));
        let room_y = leaf.y
            + rng.random_range(ROOM_PADDING..=(leaf.h - room_h - ROOM_PADDING).max(ROOM_PADDING));

        let room = Rect::new(room_x, room_y, room_w, room_h);
        carve_room(&mut tiles, &room);
        rooms.push(room);
    }

    // 3. Connect rooms with L-shaped corridors (walk the BSP tree)
    connect_bsp(&root, &mut tiles, &mut rng);

    // 4. Place doors at corridor-room junctions
    place_doors(&mut tiles, width, height);

    // 5. Place stairs
    if let (Some(up_room), Some(down_room)) = (rooms.first(), rooms.last()) {
        let (ux, uy) = up_room.center();
        let (dx, dy) = down_room.center();
        tiles[uy as usize][ux as usize] = TileKind::StairsUp;
        tiles[dy as usize][dx as usize] = TileKind::StairsDown;
    }

    let spawn_point = rooms.first().map(|r| r.center()).unwrap_or((1, 1));

    DungeonFloor {
        width,
        height,
        tiles,
        rooms,
        spawn_point,
    }
}

fn carve_room(tiles: &mut [Vec<TileKind>], room: &Rect) {
    for y in room.y..room.y + room.h {
        for x in room.x..room.x + room.w {
            if y >= 0 && (y as usize) < tiles.len() && x >= 0 && (x as usize) < tiles[0].len() {
                tiles[y as usize][x as usize] = TileKind::Floor;
            }
        }
    }
}

fn carve_corridor(tiles: &mut [Vec<TileKind>], x1: i32, y1: i32, x2: i32, y2: i32) {
    let mut x = x1;
    let mut y = y1;

    // Horizontal first, then vertical (L-shape)
    while x != x2 {
        if x >= 0
            && (x as usize) < tiles[0].len()
            && y >= 0
            && (y as usize) < tiles.len()
            && tiles[y as usize][x as usize] == TileKind::Wall
        {
            tiles[y as usize][x as usize] = TileKind::Floor;
        }
        x += if x2 > x { 1 } else { -1 };
    }
    while y != y2 {
        if x >= 0
            && (x as usize) < tiles[0].len()
            && y >= 0
            && (y as usize) < tiles.len()
            && tiles[y as usize][x as usize] == TileKind::Wall
        {
            tiles[y as usize][x as usize] = TileKind::Floor;
        }
        y += if y2 > y { 1 } else { -1 };
    }
}

/// Recursively connect sibling leaves in the BSP tree.
fn connect_bsp(node: &BspNode, tiles: &mut [Vec<TileKind>], rng: &mut ChaCha8Rng) {
    match node {
        BspNode::Leaf { .. } => {}
        BspNode::Split { left, right, .. } => {
            connect_bsp(left, tiles, rng);
            connect_bsp(right, tiles, rng);

            let mut left_leaves = Vec::new();
            let mut right_leaves = Vec::new();
            left.collect_leaves(&mut left_leaves);
            right.collect_leaves(&mut right_leaves);

            // Pick a random leaf from each side and connect their centers
            let l = left_leaves[rng.random_range(0..left_leaves.len())];
            let r = right_leaves[rng.random_range(0..right_leaves.len())];

            let (lx, ly) = l.center();
            let (rx, ry) = r.center();

            // Randomly choose L-shape direction
            if rng.random_bool(0.5) {
                carve_corridor(tiles, lx, ly, rx, ly);
                carve_corridor(tiles, rx, ly, rx, ry);
            } else {
                carve_corridor(tiles, lx, ly, lx, ry);
                carve_corridor(tiles, lx, ry, rx, ry);
            }
        }
    }
}

/// Place doors where corridors meet rooms (floor tile adjacent to exactly 2 walls).
fn place_doors(tiles: &mut [Vec<TileKind>], width: usize, height: usize) {
    let mut candidates: Vec<(usize, usize)> = Vec::new();

    for y in 1..height - 1 {
        for x in 1..width - 1 {
            if tiles[y][x] != TileKind::Floor {
                continue;
            }
            // Count wall neighbors
            let wall_neighbors = [
                tiles[y - 1][x],
                tiles[y + 1][x],
                tiles[y][x - 1],
                tiles[y][x + 1],
            ]
            .iter()
            .filter(|&&t| t == TileKind::Wall)
            .count();

            // Door candidate: exactly 2 opposing walls (corridor bottleneck)
            if wall_neighbors == 2 {
                let horizontal_walls =
                    tiles[y][x - 1] == TileKind::Wall && tiles[y][x + 1] == TileKind::Wall;
                let vertical_walls =
                    tiles[y - 1][x] == TileKind::Wall && tiles[y + 1][x] == TileKind::Wall;
                if horizontal_walls || vertical_walls {
                    candidates.push((x, y));
                }
            }
        }
    }

    // Place doors on ~40% of candidates to avoid over-cluttering
    for (x, y) in candidates {
        // Simple deterministic skip: only place every ~2.5 doors
        if (x + y) % 5 < 2 {
            tiles[y][x] = TileKind::Door;
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_room_count() {
        let floor = generate_floor(80, 60, 42);
        assert!(
            floor.rooms.len() >= 4,
            "Expected at least 4 rooms, got {}",
            floor.rooms.len()
        );
    }

    #[test]
    fn test_no_room_overlaps() {
        let floor = generate_floor(80, 60, 42);
        for (i, a) in floor.rooms.iter().enumerate() {
            for (j, b) in floor.rooms.iter().enumerate() {
                if i != j {
                    assert!(
                        !a.intersects(b),
                        "Room {i} ({a:?}) overlaps room {j} ({b:?})"
                    );
                }
            }
        }
    }

    #[test]
    fn test_connectivity() {
        // BFS from spawn_point — every floor tile should be reachable.
        let floor = generate_floor(80, 60, 42);
        let (sx, sy) = floor.spawn_point;
        let mut visited = vec![vec![false; floor.width]; floor.height];
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((sx as usize, sy as usize));
        visited[sy as usize][sx as usize] = true;

        while let Some((x, y)) = queue.pop_front() {
            for (dx, dy) in &[(0, 1), (0, -1), (1, 0), (-1, 0)] {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx >= 0
                    && ny >= 0
                    && (nx as usize) < floor.width
                    && (ny as usize) < floor.height
                    && !visited[ny as usize][nx as usize]
                    && floor.tiles[ny as usize][nx as usize] != TileKind::Wall
                {
                    visited[ny as usize][nx as usize] = true;
                    queue.push_back((nx as usize, ny as usize));
                }
            }
        }

        // Count reachable floor tiles
        let reachable = visited.iter().flatten().filter(|&&v| v).count();
        let total_floor = floor
            .tiles
            .iter()
            .flatten()
            .filter(|&&t| t != TileKind::Wall)
            .count();

        assert_eq!(
            reachable, total_floor,
            "Not all floor tiles are reachable! reachable={reachable}, total_floor={total_floor}"
        );
    }

    #[test]
    fn test_stairs_present() {
        let floor = generate_floor(80, 60, 42);
        let has_up = floor
            .tiles
            .iter()
            .flatten()
            .any(|&t| t == TileKind::StairsUp);
        let has_down = floor
            .tiles
            .iter()
            .flatten()
            .any(|&t| t == TileKind::StairsDown);
        assert!(has_up, "Missing StairsUp");
        assert!(has_down, "Missing StairsDown");
    }

    #[test]
    fn test_determinism() {
        let a = generate_floor(80, 60, 123);
        let b = generate_floor(80, 60, 123);
        assert_eq!(a.rooms.len(), b.rooms.len());
        for y in 0..a.height {
            for x in 0..a.width {
                assert_eq!(a.tiles[y][x], b.tiles[y][x], "Mismatch at ({x},{y})");
            }
        }
    }

    #[test]
    fn test_validated_spawn_point_normal() {
        let floor = generate_floor(80, 60, 42);
        let (x, y, adjusted) = floor.validated_spawn_point();
        assert!(
            !adjusted,
            "Normal BSP spawn point should not need adjustment"
        );
        assert!(floor.is_walkable(x, y), "Validated spawn must be walkable");
    }

    #[test]
    fn test_validated_spawn_point_fallback() {
        // Construct a floor with spawn_point inside a wall to trigger fallback
        let mut floor = generate_floor(80, 60, 42);
        // Place spawn_point on a known wall tile (0,0 is always a border wall)
        floor.spawn_point = (0, 0);
        assert!(!floor.is_walkable(0, 0), "Border should be a wall");

        let (x, y, adjusted) = floor.validated_spawn_point();
        assert!(adjusted, "Should have adjusted away from wall");
        assert!(
            floor.is_walkable(x, y),
            "Fallback must land on walkable tile"
        );
    }

    #[test]
    fn test_find_nearest_walkable_out_of_bounds() {
        let floor = generate_floor(80, 60, 42);
        // Out-of-bounds coordinates should still find a valid tile
        let result = floor.find_nearest_walkable(-5, -5);
        assert!(
            result.is_some(),
            "Should find a walkable tile even from OOB start"
        );
        let (x, y) = result.unwrap();
        assert!(floor.is_walkable(x, y));
    }
}
