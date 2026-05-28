//! Shelter map generator.
//!
//! Pure function: `(seed) → ShelterData`.
//! Produces a 40×30 walled compound with three starting rooms connected by
//! L-shaped corridors. Uses the same tile infrastructure as the dungeon BSP
//! generator.

use crate::core::components::TileKind;
use crate::game::dungeon::bsp::Rect;

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// Which purpose a shelter room serves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShelterRoomKind {
    Entrance,
    Quarters,
    Workshop,
    Storage,
    Expansion,
}

/// A room inside the shelter compound.
#[derive(Debug, Clone, Copy)]
pub struct ShelterRoom {
    pub rect: Rect,
    pub kind: ShelterRoomKind,
}

/// Output of [`generate_shelter`].
#[derive(Debug, Clone)]
pub struct ShelterData {
    pub tiles: Vec<Vec<TileKind>>,
    pub rooms: Vec<ShelterRoom>,
    pub spawn_point: (i32, i32),
    pub width: usize,
    pub height: usize,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const WIDTH: usize = 40;
const HEIGHT: usize = 30;

// ---------------------------------------------------------------------------
// Generator
// ---------------------------------------------------------------------------

/// Generate a shelter compound.
///
/// The `seed` parameter is accepted for API parity with the dungeon generator
/// and future procedural expansion placement; the three starter rooms are
/// deterministic regardless of seed.
pub fn generate_shelter(_seed: u64) -> ShelterData {
    let mut tiles = vec![vec![TileKind::Wall; WIDTH]; HEIGHT];

    // -- Define the three starting rooms ----------------------------------
    let entrance = ShelterRoom {
        rect: Rect::new(15, 24, 10, 5),
        kind: ShelterRoomKind::Entrance,
    };
    let quarters = ShelterRoom {
        rect: Rect::new(2, 2, 10, 8),
        kind: ShelterRoomKind::Quarters,
    };
    let workshop = ShelterRoom {
        rect: Rect::new(25, 10, 12, 8),
        kind: ShelterRoomKind::Workshop,
    };

    let rooms = vec![entrance, quarters, workshop];

    // -- Carve rooms ------------------------------------------------------
    for room in &rooms {
        carve_room(&mut tiles, &room.rect);
    }

    // -- Connect rooms with corridors -------------------------------------
    // Entrance ↔ Quarters
    let (ec, qc) = (entrance.rect.center(), quarters.rect.center());
    carve_corridor(&mut tiles, ec, qc);

    // Entrance ↔ Workshop
    let wc = workshop.rect.center();
    carve_corridor(&mut tiles, ec, wc);

    // -- Place doors at corridor-room junctions ---------------------------
    place_doors(&mut tiles, &rooms);

    // -- Gate: mark the bottom edge of the entrance as exit to overworld --
    let gate_x = entrance.rect.x + entrance.rect.w / 2;
    let gate_y = (entrance.rect.y + entrance.rect.h - 1).min(HEIGHT as i32 - 1);
    tiles[gate_y as usize][gate_x as usize] = TileKind::StairsUp;

    // -- Spawn point: center of the entrance room -------------------------
    let spawn_point = entrance.rect.center();

    ShelterData {
        tiles,
        rooms,
        spawn_point,
        width: WIDTH,
        height: HEIGHT,
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Carve floor tiles for a room interior (1-tile wall border kept intact via
/// the perimeter being all-wall by default).
fn carve_room(tiles: &mut [Vec<TileKind>], rect: &Rect) {
    for y in rect.y..rect.y + rect.h {
        for x in rect.x..rect.x + rect.w {
            if in_bounds(x, y) {
                tiles[y as usize][x as usize] = TileKind::Floor;
            }
        }
    }
}

/// Carve an L-shaped corridor (horizontal first, then vertical), 2 tiles wide.
fn carve_corridor(tiles: &mut [Vec<TileKind>], from: (i32, i32), to: (i32, i32)) {
    let (x1, y1) = from;
    let (x2, y2) = to;

    // Horizontal leg
    let (min_x, max_x) = if x1 < x2 { (x1, x2) } else { (x2, x1) };
    for x in min_x..=max_x {
        for dy in 0..2 {
            let y = y1 + dy;
            if in_bounds(x, y) {
                tiles[y as usize][x as usize] = TileKind::Floor;
            }
        }
    }

    // Vertical leg
    let (min_y, max_y) = if y1 < y2 { (y1, y2) } else { (y2, y1) };
    for y in min_y..=max_y {
        for dx in 0..2 {
            let x = x2 + dx;
            if in_bounds(x, y) {
                tiles[y as usize][x as usize] = TileKind::Floor;
            }
        }
    }
}

/// Place Door tiles where corridors meet room edges.
///
/// Scans each room perimeter: any Floor tile on the perimeter that has a Floor
/// neighbour outside the room is a junction → place a Door.
fn place_doors(tiles: &mut [Vec<TileKind>], rooms: &[ShelterRoom]) {
    for room in rooms {
        let r = &room.rect;
        for x in r.x..r.x + r.w {
            try_place_door(tiles, x, r.y - 1, r); // top edge
            try_place_door(tiles, x, r.y + r.h, r); // bottom edge
        }
        for y in r.y..r.y + r.h {
            try_place_door(tiles, r.x - 1, y, r); // left edge
            try_place_door(tiles, r.x + r.w, y, r); // right edge
        }
    }
}

fn try_place_door(tiles: &mut [Vec<TileKind>], x: i32, y: i32, _room: &Rect) {
    if !in_bounds(x, y) {
        return;
    }
    let ux = x as usize;
    let uy = y as usize;
    if tiles[uy][ux] == TileKind::Floor {
        // Check if this floor tile is adjacent to a wall — that makes it a
        // threshold between corridor and room interior.
        let neighbours = [(0i32, 1i32), (0, -1), (1, 0), (-1, 0)];
        let has_wall_neighbour = neighbours.iter().any(|(dx, dy)| {
            let nx = x + dx;
            let ny = y + dy;
            in_bounds(nx, ny) && tiles[ny as usize][nx as usize] == TileKind::Wall
        });
        if has_wall_neighbour {
            tiles[uy][ux] = TileKind::Door;
        }
    }
}

fn in_bounds(x: i32, y: i32) -> bool {
    x >= 0 && y >= 0 && (x as usize) < WIDTH && (y as usize) < HEIGHT
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shelter_three_rooms() {
        let data = generate_shelter(42);
        assert_eq!(data.rooms.len(), 3);
        assert_eq!(data.rooms[0].kind, ShelterRoomKind::Entrance);
        assert_eq!(data.rooms[1].kind, ShelterRoomKind::Quarters);
        assert_eq!(data.rooms[2].kind, ShelterRoomKind::Workshop);
        assert_eq!(data.width, 40);
        assert_eq!(data.height, 30);
    }

    #[test]
    fn test_shelter_spawn_in_entrance() {
        let data = generate_shelter(42);
        let (sx, sy) = data.spawn_point;
        let entrance = &data.rooms[0].rect;
        assert!(
            sx >= entrance.x
                && sx < entrance.x + entrance.w
                && sy >= entrance.y
                && sy < entrance.y + entrance.h,
            "spawn_point should be inside the entrance room"
        );
    }

    #[test]
    fn test_shelter_connectivity() {
        let data = generate_shelter(42);
        // Flood-fill from spawn point. Every room center must be reachable.
        let mut visited = vec![vec![false; data.width]; data.height];
        let mut stack = vec![data.spawn_point];
        while let Some((x, y)) = stack.pop() {
            if !in_bounds(x, y) {
                continue;
            }
            let ux = x as usize;
            let uy = y as usize;
            if visited[uy][ux] {
                continue;
            }
            let tile = data.tiles[uy][ux];
            if tile == TileKind::Wall {
                continue;
            }
            visited[uy][ux] = true;
            stack.push((x + 1, y));
            stack.push((x - 1, y));
            stack.push((x, y + 1));
            stack.push((x, y - 1));
        }

        for room in &data.rooms {
            let (cx, cy) = room.rect.center();
            assert!(
                visited[cy as usize][cx as usize],
                "{:?} room center ({},{}) is not reachable from spawn",
                room.kind, cx, cy
            );
        }
    }

    #[test]
    fn test_shelter_has_gate() {
        let data = generate_shelter(42);
        let has_gate = data
            .tiles
            .iter()
            .any(|row| row.contains(&TileKind::StairsUp));
        assert!(has_gate, "shelter should have a StairsUp gate tile");
    }
}
