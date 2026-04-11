//! Turn system — phase sequencing, action budgets, and turn counter.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::components::Enemy;

// ---------------------------------------------------------------------------
// States
// ---------------------------------------------------------------------------

/// Sub-state governing the turn-based loop inside a dungeon.
#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash, Reflect)]
pub enum TurnPhase {
    #[default]
    AwaitingInput,
    PlayerTurn,
    EnemyTurn,
    WorldTick,
}

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Universal turn counter — incremented once per full round in WorldTick.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, Default, Reflect)]
#[reflect(Resource)]
pub struct GameTime {
    pub turn: u32,
}

/// Holds the player's chosen action until PlayerTurn consumes it.
#[derive(Resource, Default)]
pub struct PlayerAction(pub Option<PendingAction>);

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Speed-based action budget. Each entity gets `speed` actions per round.
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct ActionBudget {
    pub speed: u8,
    pub remaining: u8,
}

impl ActionBudget {
    pub fn new(speed: u8) -> Self {
        Self {
            speed,
            remaining: speed,
        }
    }
}

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Player input mapped to a concrete action.
#[derive(Debug, Clone)]
pub enum PendingAction {
    Move { dx: i32, dy: i32 },
    MeleeAttack { target: Entity },
    Wait,
    UseItem(usize), // inventory slot index
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Maximum frames the `EnemyTurn` phase is allowed to persist before being
/// force-advanced to `WorldTick`.  Prevents soft-locks when an entity has a
/// non-zero budget but no system drains it (e.g. mid-turn spawns).
const ENEMY_TURN_MAX_FRAMES: u32 = 3;

/// Tracks how many consecutive frames we have spent in `EnemyTurn`.
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct EnemyTurnFrameCounter(pub u32);

/// Advances the turn phase state machine.
///
/// Runs only in `AppState::Dungeon`.
///
/// - **AwaitingInput** → does nothing (waits for input system to populate `PlayerAction`).
/// - **PlayerTurn** → after player acts, checks whether enemies still have actions.
///   If yes → `EnemyTurn`, otherwise → `WorldTick`.
/// - **EnemyTurn** → after enemies act, transitions to `WorldTick`.
///   Force-advances after `ENEMY_TURN_MAX_FRAMES` frames to prevent soft-locks.
/// - **WorldTick** → increments `GameTime.turn`, resets budgets, returns to `AwaitingInput`.
pub fn advance_turn_phase(
    phase: Res<State<TurnPhase>>,
    mut next_phase: ResMut<NextState<TurnPhase>>,
    mut game_time: ResMut<GameTime>,
    mut budgets: Query<(&mut ActionBudget, Option<&Enemy>)>,
    mut frame_counter: ResMut<EnemyTurnFrameCounter>,
) {
    match phase.get() {
        TurnPhase::AwaitingInput => {
            // Input systems handle transition to PlayerTurn.
            frame_counter.0 = 0;
        }
        TurnPhase::PlayerTurn => {
            frame_counter.0 = 0;
            let enemies_have_actions = budgets
                .iter()
                .any(|(b, enemy)| enemy.is_some() && b.remaining > 0);
            if enemies_have_actions {
                next_phase.set(TurnPhase::EnemyTurn);
            } else {
                next_phase.set(TurnPhase::WorldTick);
            }
        }
        TurnPhase::EnemyTurn => {
            frame_counter.0 += 1;
            let enemies_done = budgets
                .iter()
                .filter(|(_, enemy)| enemy.is_some())
                .all(|(b, _)| b.remaining == 0);
            if enemies_done || frame_counter.0 >= ENEMY_TURN_MAX_FRAMES {
                frame_counter.0 = 0;
                next_phase.set(TurnPhase::WorldTick);
            }
        }
        TurnPhase::WorldTick => {
            game_time.turn += 1;
            for (mut budget, _) in budgets.iter_mut() {
                budget.remaining = budget.speed;
            }
            next_phase.set(TurnPhase::AwaitingInput);
        }
    }
}

/// Resets all action budgets — `remaining` ← `speed`.
///
/// Runs during `WorldTick` as an explicit system for ordering control.
pub fn reset_action_budgets(mut budgets: Query<&mut ActionBudget>) {
    for mut budget in budgets.iter_mut() {
        budget.remaining = budget.speed;
    }
}

/// Ticks down `SprintCooldown` by 1 each turn, clamped to 0.
///
/// Runs during `WorldTick`.
pub fn tick_sprint_cooldown(
    mut query: Query<&mut crate::core::abilities::SprintCooldown>,
) {
    for mut cd in query.iter_mut() {
        if cd.remaining > 0 {
            cd.remaining -= 1;
        }
    }
}
