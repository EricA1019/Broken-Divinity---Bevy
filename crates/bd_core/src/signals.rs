//! Signal types for the BD Kernel — intents, requests, and results.

use bevy_ecs::{entity::Entity, prelude::*};

use crate::{components::Position, direction::Direction};

/// Intent to move an entity in a direction.
#[derive(Message, Debug, Clone)]
pub struct MoveIntent {
    pub entity: Entity,
    pub direction: Direction,
}

/// An entity has moved to a new position.
#[derive(Message, Debug, Clone)]
pub struct EntityMoved {
    pub entity: Entity,
    pub from: Position,
    pub to: Position,
}

/// A movement was blocked.
#[derive(Message, Debug, Clone)]
pub struct MoveBlocked {
    pub entity: Entity,
    pub direction: Direction,
    pub reason: MoveBlockReason,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveBlockReason {
    OutOfBounds,
    BlockedByWall,
    BlockedByEntity,
}
