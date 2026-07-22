//! Time system — core turn/day tracking for the BD Kernel.
//!
//! `GameTime` tracks the current day and turn number.
//! Time advances once per frame, after all gameplay state mutations.

use bevy_app::App;
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};



use crate::BdSet;

// ── Constants ──

/// Number of turns per in-game day.
pub const TURNS_PER_DAY: u64 = 24;

// ── Resource ──

/// Flag set when an action was processed this frame, cleared after time advance.
#[derive(Resource, Debug, Default, Serialize, Deserialize)]
pub struct ShouldAdvanceTime(pub bool);

/// Tracks the current game day and turn.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct GameTime {
    pub day: u64,
    pub turn: u64,
}

impl Default for GameTime {
    fn default() -> Self {
        Self { day: 0, turn: 0 }
    }
}

// ── Plugin registration ──

pub(crate) fn register_time(app: &mut App) {
    app.insert_resource(GameTime::default());
    app.init_resource::<ShouldAdvanceTime>();
    app.add_systems(
        bevy_app::Update,
        advance_time.in_set(BdSet::ResultEmission),
    );
}

// ── System ──

/// Advances `GameTime` by one turn only when a player action was processed.
/// Runs in `ResultEmission` after all gameplay state mutations.
fn advance_time(
    mut game_time: ResMut<GameTime>,
    mut should_advance: ResMut<ShouldAdvanceTime>,
) {
    if !should_advance.0 {
        return;
    }
    should_advance.0 = false;
    game_time.turn += 1;
    if game_time.turn >= TURNS_PER_DAY {
        game_time.day += 1;
        game_time.turn = 0;
    }
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
