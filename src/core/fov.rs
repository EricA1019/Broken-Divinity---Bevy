#![allow(clippy::needless_range_loop, clippy::too_many_arguments)]

//! Field-of-view (FOV) using recursive shadowcasting.
//!
//! Recalculates when the player moves.
//! Tiles outside viewshed are hidden; previously-seen tiles are dimmed ("remembered").

use crate::core::components::{Position, TileKind, Viewshed};
use crate::core::movement::MapTiles;
use bevy::prelude::*;

/// Recalculates the viewshed for any entity whose position changed.
pub fn update_viewshed(
    mut query: Query<(&Position, &mut Viewshed), Changed<Position>>,
    map: Option<Res<MapTiles>>,
) {
    let Some(map) = map else {
        return;
    };

    for (pos, mut viewshed) in query.iter_mut() {
        viewshed.visible_tiles.clear();
        compute_fov(
            pos.x,
            pos.y,
            viewshed.range as i32,
            &map,
            &mut viewshed.visible_tiles,
        );
        viewshed.dirty = false;
    }
}

/// Returns true if the tile at (x, y) blocks light.
fn is_blocking(x: i32, y: i32, map: &MapTiles) -> bool {
    if x < 0 || y < 0 || x as usize >= map.width || y as usize >= map.height {
        return true;
    }
    let t = map.tiles[y as usize][x as usize];
    t == TileKind::Wall || t == TileKind::Door
}

/// Compute FOV using recursive shadowcasting.
fn compute_fov(ox: i32, oy: i32, range: i32, map: &MapTiles, visible: &mut Vec<Position>) {
    visible.push(Position::new(ox, oy));

    for octant in 0..8 {
        cast_light(ox, oy, range, 1, 1.0, 0.0, octant, map, visible);
    }
}

fn cast_light(
    ox: i32,
    oy: i32,
    range: i32,
    row: i32,
    mut start: f64,
    end: f64,
    octant: usize,
    map: &MapTiles,
    visible: &mut Vec<Position>,
) {
    if start < end {
        return;
    }

    let radius_sq = (range * range) as f32;

    for j in row..=range {
        let mut blocked = false;
        let mut next_start = start;

        for i in (0..=j).rev() {
            let slope_l = (i as f64 + 0.5) / (j as f64 - 0.5);
            let slope_r = (i as f64 - 0.5) / (j as f64 + 0.5);

            if slope_r > start {
                continue;
            }
            if slope_l < end {
                break;
            }

            // Transform to map coordinates
            let (dx, dy) = match octant {
                0 => (i, -j),
                1 => (j, -i),
                2 => (j, i),
                3 => (i, j),
                4 => (-i, j),
                5 => (-j, i),
                6 => (-j, -i),
                7 => (-i, -j),
                _ => (0, 0),
            };

            let x = ox + dx;
            let y = oy + dy;

            let pos = Position::new(x, y);
            if (dx * dx + dy * dy) as f32 <= radius_sq && !visible.contains(&pos) {
                visible.push(pos);
            }

            let blocking = j < range && is_blocking(x, y, map);

            if blocked {
                if blocking {
                    next_start = slope_r;
                } else {
                    blocked = false;
                    start = next_start;
                }
            } else if blocking {
                blocked = true;
                cast_light(ox, oy, range, j + 1, start, slope_l, octant, map, visible);
                next_start = slope_r;
            }
        }

        if blocked {
            break;
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::components::TileKind;
    use crate::core::movement::MapTiles;

    fn empty_map(w: usize, h: usize) -> MapTiles {
        MapTiles::new(vec![vec![TileKind::Floor; w]; h])
    }

    fn walled_room_map(w: usize, h: usize) -> MapTiles {
        let mut tiles = vec![vec![TileKind::Wall; w]; h];
        for y in 3..7 {
            for x in 3..7 {
                tiles[y][x] = TileKind::Floor;
            }
        }
        MapTiles::new(tiles)
    }

    #[test]
    fn test_open_floor_sees_full_range() {
        let map = empty_map(20, 20);
        let mut visible = Vec::new();
        compute_fov(10, 10, 5, &map, &mut visible);
        assert!(
            visible.len() > 50,
            "Expected >50 visible tiles, got {}",
            visible.len()
        );
    }

    #[test]
    fn test_wall_blocks_vision() {
        let map = walled_room_map(20, 20);
        let mut visible = Vec::new();
        // Player at 5,5 is inside the floor (3..7)
        compute_fov(5, 5, 10, &map, &mut visible);

        // Debug output: print visible tiles if failure
        let leaks: Vec<_> = visible
            .iter()
            .filter(|p| p.x < 3 || p.x > 6 || p.y < 3 || p.y > 6)
            .collect();
        if !leaks.is_empty() {
            println!("Leaks detected: {:?}", leaks);
            // Check if they are walls or beyond
            for leak in &leaks {
                println!(
                    "Leak at {:?} is {:?}",
                    leak, map.tiles[leak.y as usize][leak.x as usize]
                );
            }
        }

        let beyond_walls = visible
            .iter()
            .any(|p| p.x < 2 || p.x > 7 || p.y < 2 || p.y > 7);
        assert!(!beyond_walls, "Vision leaked BEYOND walls!");
    }

    #[test]
    fn test_origin_always_visible() {
        let map = empty_map(10, 10);
        let mut visible = Vec::new();
        compute_fov(5, 5, 3, &map, &mut visible);
        assert!(visible.contains(&Position::new(5, 5)));
    }

    #[test]
    fn test_symmetry() {
        let map = empty_map(20, 20);
        let mut vis_a = Vec::new();
        compute_fov(5, 5, 8, &map, &mut vis_a);

        let mut vis_b = Vec::new();
        compute_fov(10, 5, 8, &map, &mut vis_b);

        assert!(vis_a.contains(&Position::new(10, 5)), "A should see B");
        assert!(vis_b.contains(&Position::new(5, 5)), "B should see A");
    }
}
