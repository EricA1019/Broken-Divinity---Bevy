//! Signal types for the BD Kernel — intents, requests, and results.

use bevy_ecs::{entity::Entity, prelude::*};
use serde::{Deserialize, Serialize};

use crate::{components::Position, direction::Direction};

// ── Movement ──

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
    NotEnoughAP,
}

// ── Actions ──

/// Intent to perform an action.
#[derive(Message, Debug, Clone)]
pub struct ActionIntent {
    pub actor: Entity,
    pub action_id: String,
    pub direction: Option<Direction>,
    pub target: Option<Entity>,
}

/// An action was denied with a reason.
#[derive(Message, Debug, Clone)]
pub struct ActionDenied {
    pub actor: Entity,
    pub action_id: String,
    pub reason: DenialReason,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DenialReason {
    NotEnoughPool(PoolKind),
    BlockedTile,
    OutOfRange,
    NoTarget,
    InvalidTarget,
    ActorDefeated,
    /// Fallback for unclassified denials.
    Other(String),
}

// ── Pool deltas ──

/// Kind of pool (health, AP, stress, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PoolKind {
    Health,
    ActionPoints,
    Stress,
    Corruption,
    Faith,
    Morale,
    Supplies,
    Mood,
    Sanity,
    Temperance,
    Justice,
    Prudence,
    Fortitude,
    Thumos,
    Metis,
    Kleos,
    RepPuritans,
    RepWanderers,
    RepBrokenChoir,
    RepDemons,
    RepHumanSettlements,
}

/// Tag categorizing a pool delta for modifier routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeltaTag {
    Physical,
    Divine,
    Poison,
    Recovery,
    MovementCost,
    /// Generic action cost or effect.
    Action,
}

/// Request to change a pool value. Negative = damage/cost, positive = heal/restore.
#[derive(Message, Debug, Clone)]
pub struct PoolDeltaRequested {
    pub source: Option<Entity>,
    pub target: Entity,
    pub kind: PoolKind,
    pub amount: i32,
    pub tags: Vec<DeltaTag>,
    pub reason: String,
}

/// A pool delta was successfully applied.
#[derive(Message, Debug, Clone)]
pub struct PoolDeltaApplied {
    pub source: Option<Entity>,
    pub target: Entity,
    pub kind: PoolKind,
    pub before: i32,
    pub after: i32,
    pub amount_applied: i32,
    pub tags: Vec<DeltaTag>,
    pub reason: String,
}

/// An entity was defeated (health reached minimum).
#[derive(Message, Debug, Clone)]
pub struct EntityDefeated {
    pub entity: Entity,
    pub kind: PoolKind,
}
