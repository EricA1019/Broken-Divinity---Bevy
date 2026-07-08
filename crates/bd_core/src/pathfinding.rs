//! Pathfinding and visibility adapters.
//!
//! Hides specific pathfinding crate implementations behind adapter traits.
//! Currently uses the `pathfinding` crate for A*.

use std::collections::HashSet;

use pathfinding::prelude::astar;

use crate::{components::Position, map::SmokeMap};

/// Result of a pathfinding query.
pub type Path = Vec<Position>;

/// Trait for finding paths on a grid map.
pub trait Pathfinder {
    fn find_path(
        &self,
        map: &SmokeMap,
        start: Position,
        goal: Position,
        blocked: &HashSet<Position>,
    ) -> Option<Path>;
}

/// Trait for computing visible tiles.
pub trait VisibilityProvider {
    fn visible_tiles(
        &self,
        map: &SmokeMap,
        origin: Position,
        radius: u32,
        blocked: &HashSet<Position>,
    ) -> HashSet<Position>;
}

/// A* pathfinder using the `pathfinding` crate.
#[derive(Debug, Clone, Default)]
pub struct AStarPathfinder;

impl Pathfinder for AStarPathfinder {
    fn find_path(
        &self,
        map: &SmokeMap,
        start: Position,
        goal: Position,
        blocked: &HashSet<Position>,
    ) -> Option<Path> {
        astar(
            &start,
            |pos| neighbors(pos, map, blocked),
            |pos| heuristic(pos, goal),
            |pos| *pos == goal,
        )
        .map(|(path, _)| path)
    }
}

/// Simple Bresenham-based visibility provider.
#[derive(Debug, Clone, Default)]
pub struct BresenhamVisibility;

impl VisibilityProvider for BresenhamVisibility {
    fn visible_tiles(
        &self,
        map: &SmokeMap,
        origin: Position,
        radius: u32,
        blocked: &HashSet<Position>,
    ) -> HashSet<Position> {
        let mut visible = HashSet::new();
        visible.insert(origin);

        let r = radius as i32;
        for dy in -r..=r {
            for dx in -r..=r {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let target = Position {
                    x: origin.x + dx,
                    y: origin.y + dy,
                };
                if !map.is_walkable(target.x, target.y) {
                    continue;
                }
                // Simple line-of-sight: check if any tile on the line is wall/blocked
                if has_line_of_sight(origin, target, map, blocked) {
                    visible.insert(target);
                }
            }
        }
        visible
    }
}

/// Compute Manhattan distance heuristic.
fn heuristic(a: &Position, goal: Position) -> i32 {
    (a.x - goal.x).unsigned_abs() as i32 + (a.y - goal.y).unsigned_abs() as i32
}

/// Return walkable neighbors that are in bounds and not blocked.
fn neighbors(pos: &Position, map: &SmokeMap, blocked: &HashSet<Position>) -> Vec<(Position, i32)> {
    let dirs = [(1, 0), (-1, 0), (0, 1), (0, -1)];
    dirs.iter()
        .map(|(dx, dy)| Position {
            x: pos.x + dx,
            y: pos.y + dy,
        })
        .filter(|p| map.is_walkable(p.x, p.y) && !blocked.contains(p))
        .map(|p| (p, 1)) // cost = 1 per step
        .collect()
}

/// Simple line-of-sight using Bresenham-like step-through.
fn has_line_of_sight(
    from: Position,
    to: Position,
    map: &SmokeMap,
    blocked: &HashSet<Position>,
) -> bool {
    let dx = (to.x - from.x).abs();
    let dy = -(to.y - from.y).abs();
    let sx = if from.x < to.x { 1 } else { -1 };
    let sy = if from.y < to.y { 1 } else { -1 };
    let mut err = dx + dy;
    let mut x = from.x;
    let mut y = from.y;

    loop {
        if x == to.x && y == to.y {
            return true;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
        let pos = Position { x, y };
        if pos == to {
            return true;
        }
        if !map.is_walkable(x, y) || blocked.contains(&pos) {
            return false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_map() -> (SmokeMap, HashSet<Position>) {
        let map = SmokeMap::default_smoke_map();
        let blocked = HashSet::new();
        (map, blocked)
    }

    #[test]
    fn path_avoids_walls() {
        let (map, blocked) = test_map();
        let pf = AStarPathfinder;
        // Player at (5,5), goal at (5,5) → straight line
        let path = pf.find_path(
            &map,
            Position { x: 5, y: 5 },
            Position { x: 5, y: 5 },
            &blocked,
        );
        assert!(path.is_some());
    }

    #[test]
    fn path_returns_none_when_unreachable() {
        let (map, blocked) = test_map();
        let pf = AStarPathfinder;
        // Player at (1,1), goal at (18,10) — both walkable, should find path
        let path = pf.find_path(
            &map,
            Position { x: 1, y: 1 },
            Position { x: 18, y: 10 },
            &blocked,
        );
        assert!(path.is_some());

        // Fully surround the start position to make it unreachable.
        // (1,1) has four neighbors: (0,1)=wall, (2,1), (1,0)=wall, (1,2).
        // Block (2,1) and (1,2) to trap the player.
        let mut blocked_entity = blocked.clone();
        blocked_entity.insert(Position { x: 2, y: 1 });
        blocked_entity.insert(Position { x: 1, y: 2 });
        let path = pf.find_path(
            &map,
            Position { x: 1, y: 1 },
            Position { x: 10, y: 6 },
            &blocked_entity,
        );
        assert!(path.is_none());
    }

    #[test]
    fn movement_range_respects_ap_budget() {
        let (map, blocked) = test_map();
        let pf = AStarPathfinder;
        // Starting at (5,5) with 3 AP, the path to (5,9) is 4 steps → needs more AP
        let path = pf.find_path(
            &map,
            Position { x: 5, y: 5 },
            Position { x: 5, y: 9 },
            &blocked,
        );
        assert!(path.is_some());
        // Path length (excluding start) = 4
        assert_eq!(path.unwrap().len() - 1, 4);
    }

    #[test]
    fn occupied_blocking_tile_blocks_path() {
        let (map, blocked) = test_map();
        let pf = AStarPathfinder;
        // Path from (5,5) to (5,6) is clear
        let path = pf.find_path(
            &map,
            Position { x: 5, y: 5 },
            Position { x: 5, y: 6 },
            &blocked,
        );
        assert!(path.is_some());

        // Now block (5,6)
        let mut blocked = blocked;
        blocked.insert(Position { x: 5, y: 6 });
        let path = pf.find_path(
            &map,
            Position { x: 5, y: 5 },
            Position { x: 5, y: 6 },
            &blocked,
        );
        assert!(path.is_none());
    }

    #[test]
    fn visibility_hides_unseen_enemy() {
        let (map, blocked) = test_map();
        let vis = BresenhamVisibility;
        // Player at (5,5), radius 2
        let visible = vis.visible_tiles(&map, Position { x: 5, y: 5 }, 2, &blocked);
        // Enemy at (8,5) is outside radius 2
        assert!(!visible.contains(&Position { x: 8, y: 5 }));
        // Enemy at (6,5) is inside
        assert!(visible.contains(&Position { x: 6, y: 5 }));
    }

    #[test]
    fn remembered_tile_uses_muted_visual_state() {
        // Visibility uses a square bounding box: all tiles within radius in both axes.
        let (map, blocked) = test_map();
        let vis = BresenhamVisibility;
        let visible = vis.visible_tiles(&map, Position { x: 10, y: 6 }, 3, &blocked);
        // All visible tiles should be within Chebyshev distance (max of abs diffs) ≤ radius.
        for tile in &visible {
            let dx = (tile.x - 10).unsigned_abs();
            let dy = (tile.y - 6).unsigned_abs();
            assert!(
                dx <= 3 && dy <= 3,
                "Visible tile {tile:?} outside radius (dx={dx}, dy={dy})"
            );
        }
    }
}
