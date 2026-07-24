//! Direction enum for movement intents.

use serde::{Deserialize, Serialize};

use crate::components::Position;

/// Cardinal directions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    /// Delta (dx, dy) for this direction.
    pub fn delta(&self) -> (i32, i32) {
        match self {
            Direction::North => (0, -1),
            Direction::South => (0, 1),
            Direction::East => (1, 0),
            Direction::West => (-1, 0),
        }
    }

    /// Best cardinal direction to move from `from` toward `to`.
    /// Prefers horizontal movement when both axes differ (arbitrary but deterministic).
    pub fn toward(from: Position, to: Position) -> Direction {
        let dx = to.x - from.x;
        let dy = to.y - from.y;
        if dx.abs() >= dy.abs() {
            if dx > 0 {
                Direction::East
            } else {
                Direction::West
            }
        } else if dy > 0 {
            Direction::South
        } else {
            Direction::North
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Position;

    #[test]
    fn toward_east_when_target_is_east() {
        let from = Position { x: 5, y: 5 };
        let to = Position { x: 8, y: 5 };
        assert_eq!(Direction::toward(from, to), Direction::East);
    }

    #[test]
    fn toward_west_when_target_is_west() {
        let from = Position { x: 5, y: 5 };
        let to = Position { x: 2, y: 5 };
        assert_eq!(Direction::toward(from, to), Direction::West);
    }

    #[test]
    fn toward_south_when_target_is_south() {
        let from = Position { x: 5, y: 5 };
        let to = Position { x: 5, y: 8 };
        assert_eq!(Direction::toward(from, to), Direction::South);
    }

    #[test]
    fn toward_north_when_target_is_north() {
        let from = Position { x: 5, y: 5 };
        let to = Position { x: 5, y: 2 };
        assert_eq!(Direction::toward(from, to), Direction::North);
    }

    #[test]
    fn toward_prefers_horizontal_when_both_differ() {
        let from = Position { x: 5, y: 5 };
        let to = Position { x: 8, y: 8 };
        // dx=3, dy=3 → dx.abs >= dy.abs → East
        assert_eq!(Direction::toward(from, to), Direction::East);
    }
}
