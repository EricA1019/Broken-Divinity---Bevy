pub mod mapgen;
pub mod raids;
pub mod research;
pub mod spawn;
pub mod station_catalog;
pub mod stations;
pub mod survivors;

use crate::core::{resources, state::AppState, turn};
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    // --- Type registration for BRP reflection ---
    app.register_type::<spawn::ShelterState>()
        .register_type::<spawn::GateAffordanceConfig>()
        .register_type::<raids::RaidChance>()
        .register_type::<raids::ActiveRaid>()
        .register_type::<raids::RaidPhase>()
        .register_type::<raids::PendingRaidReport>()
        .register_type::<survivors::Survivor>()
        .register_type::<survivors::SurvivorNeeds>()
        .register_type::<survivors::SurvivorTask>()
        .register_type::<stations::Station>()
        .register_type::<stations::StationType>();

    app.init_resource::<raids::RaidChance>()
        .init_resource::<spawn::GateAffordanceConfig>()
        .init_resource::<resources::ColonyTickTimer>()
        .init_resource::<research::CompletedResearch>()
        .add_systems(
            OnEnter(AppState::Colony),
            (
                resources::reset_colony_tick_timer,
                spawn::setup_shelter,
                survivors::spawn_initial_survivors,
                raids::deliver_pending_raid_report,
            )
                .chain(),
        )
        .add_systems(
            OnExit(AppState::Colony),
            (
                crate::core::save::cache_colony_runtime_state,
                spawn::cleanup_shelter,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                resources::tick_colony_timer,
                (
                    (
                        turn::advance_game_time,
                        survivors::tick_survivor_needs,
                        survivors::consume_shelter_resources,
                        survivors::survivor_death,
                        survivors::survivor_ai,
                        stations::station_production,
                        research::tick_research,
                        raids::check_raid_trigger,
                    )
                        .chain()
                        .run_if(raids::colony_sim_unpaused),
                    raids::resolve_active_raid,
                )
                    .chain()
                    .run_if(resources::colony_tick_ready),
            )
                .chain()
                .run_if(in_state(AppState::Colony)),
        );
}
