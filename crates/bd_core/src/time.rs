//! Time system — core turn/day tracking for the BD Kernel.
//!
//! `GameTime` tracks the current day and turn number.
//! Time advances once per frame, after all gameplay state mutations.

use bevy_app::App;
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    BdSet,
    actions::{ActionDefinition, Effect, Requirement},
    gamelog::LogLevel,
};

// ── Constants ──

/// Number of turns per in-game day.
pub const TURNS_PER_DAY: u64 = 24;

// ── Resource ──

/// Turn boundary flags shared by the action, enemy, and time systems.
///
/// `.0` requests game-time advancement for the accepted player action.
/// `.1` requests exactly one enemy phase after that player action.
#[derive(Resource, Debug, Default, Serialize, Deserialize)]
pub struct ShouldAdvanceTime(pub bool, pub bool);

/// Typed plan compiled from one accepted player action.
///
/// `elapsed_turns` advances the authoritative clock. `outpost_worker_steps`
/// is consumed by the colony movement resolver before the clock crosses any
/// resulting day boundary. Tactical actions therefore advance the clock
/// without advancing colony workers.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TimeAdvancePlan {
    pub elapsed_turns: u64,
    pub outpost_worker_steps: u64,
    pub cause: TimeAdvanceCause,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TimeAdvanceCause {
    #[default]
    None,
    AcceptedAction,
    RestUntilNextDay,
}

/// Prevents a second player action while the enemy phase is resolving.
#[derive(Component, Debug, Default)]
pub struct AwaitingEnemyPhase;

/// Tracks the current game day and turn.
#[derive(Resource, Debug, Default, Clone, Serialize, Deserialize)]
pub struct GameTime {
    pub day: u64,
    pub turn: u64,
}

/// Inserted by advance_time when a turn advances. Consumed by regenerate_action_points.
/// This is a one-shot signal — it exists for exactly one frame.
#[derive(Resource, Debug, Clone, Copy)]
pub struct TurnJustAdvanced;

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DayAdvanced {
    pub day: u64,
}

/// Typed request emitted by the validated colony Rest action.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RestUntilNextDayRequested;

// ── Plugin registration ──

pub(crate) fn register_time(app: &mut App) {
    app.insert_resource(GameTime::default());
    app.init_resource::<ShouldAdvanceTime>();
    app.init_resource::<TimeAdvancePlan>();
    app.add_message::<DayAdvanced>();
    app.add_message::<RestUntilNextDayRequested>();
    app.add_systems(bevy_app::Update, advance_time.in_set(BdSet::ResultEmission));
}

pub(crate) fn register_rest_until_next_day_action() -> ActionDefinition {
    ActionDefinition {
        id: "ability.rest_until_next_day".into(),
        label: "Rest Until Next Day".into(),
        requirements: vec![
            Requirement::EntityAlive,
            Requirement::InMode(crate::spatial::GameMode::Outpost),
            Requirement::NoBlockingInteraction,
        ],
        cost_effects: vec![],
        effects: vec![
            Effect::RequestRestUntilNextDay,
            Effect::Log("You rest until the next day.".into(), LogLevel::Info),
        ],
    }
}

// ── System ──

/// Advances `GameTime` by one turn only when a player action was processed.
/// Runs in `ResultEmission` after all gameplay state mutations.
fn advance_time(
    mut commands: Commands,
    mut game_time: ResMut<GameTime>,
    mut session: ResMut<crate::session::RunSession>,
    mut should_advance: ResMut<ShouldAdvanceTime>,
    mut plan: ResMut<TimeAdvancePlan>,
    mut rest_requests: MessageReader<RestUntilNextDayRequested>,
    mut day_advanced: bevy_ecs::message::MessageWriter<DayAdvanced>,
) {
    let rest_requested = rest_requests.read().next().is_some();
    if plan.elapsed_turns == 0 && !should_advance.0 && !rest_requested {
        return;
    }
    let elapsed_turns = if plan.elapsed_turns > 0 {
        plan.elapsed_turns
    } else if rest_requested {
        TURNS_PER_DAY - game_time.turn
    } else {
        // Compatibility boundary for isolated tests and deferred systems that
        // still request one turn through ShouldAdvanceTime.
        1
    };
    should_advance.0 = false;
    *plan = TimeAdvancePlan::default();
    let previous_day = game_time.day;
    let total_turns = game_time.turn + elapsed_turns;
    game_time.day += total_turns / TURNS_PER_DAY;
    game_time.turn = total_turns % TURNS_PER_DAY;
    for day in (previous_day + 1)..=game_time.day {
        day_advanced.write(DayAdvanced { day });
    }
    session.day = game_time.day;
    session.turn = game_time.turn;
    // Signal that a turn just advanced — AP regen consumes this next frame
    commands.insert_resource(TurnJustAdvanced);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(crate::BdCorePlugin);
        app
    }

    #[test]
    fn time_starts_at_day_zero_turn_zero() {
        let mut app = test_app();
        app.update();
        let time = app.world().resource::<GameTime>();
        assert_eq!(time.day, 0);
    }

    #[test]
    fn time_advances_every_frame() {
        let mut app = test_app();
        app.update();
        let t1 = app.world().resource::<GameTime>().turn;
        // Set advance flag so time advances
        app.world_mut().resource_mut::<ShouldAdvanceTime>().0 = true;
        app.update();
        let t2 = app.world().resource::<GameTime>().turn;
        assert_eq!(t2 - t1, 1);
    }

    #[test]
    fn day_increments_after_turns_per_day() {
        let mut app = test_app();
        for _ in 0..TURNS_PER_DAY {
            app.world_mut().resource_mut::<ShouldAdvanceTime>().0 = true;
            app.update();
        }
        let time = app.world().resource::<GameTime>();
        assert_eq!(time.day, 1);
        assert_eq!(time.turn, 0);
    }
}
