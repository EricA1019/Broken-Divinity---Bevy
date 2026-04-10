pub mod ai;
pub mod anomalies;
pub mod bsp;
pub mod consumables;
pub mod enemies;
pub mod gabriel;
pub mod hazards;
pub mod loot;
pub mod lore;
pub mod melee;
pub mod ranged;
pub mod spawn;
pub mod theme;

use bevy::prelude::*;
use crate::core::state::AppState;
use crate::core::turn::TurnPhase;

pub fn plugin(app: &mut App) {
    // --- Type registration for BRP reflection ---
    app.register_type::<spawn::DungeonState>()
        .register_type::<theme::DungeonTheme>()
        .register_type::<gabriel::GabrielState>()
        .register_type::<gabriel::Gabriel>()
        .register_type::<gabriel::GabrielCompanion>()
        .register_type::<gabriel::GabrielDialogueStep>()
        .register_type::<gabriel::GabrielDialogueState>()
        .register_type::<hazards::HazardTile>()
        .register_type::<hazards::HazardKind>()
        .register_type::<anomalies::Anomaly>()
        .register_type::<anomalies::AnomalyKind>()
        .register_type::<enemies::RangedEnemy>()
        .register_type::<melee::BumpAttackTarget>()
        .register_type::<ranged::ShootTarget>();

    app.init_resource::<melee::BumpAttackTarget>()
        .init_resource::<melee::CombatRng>()
        .init_resource::<ranged::ShootTarget>()
        .init_resource::<gabriel::GabrielState>()
        .init_resource::<gabriel::GabrielDialogueState>()
        .init_resource::<lore::LoreJournal>()
        .add_systems(OnEnter(AppState::Dungeon), spawn::setup_dungeon)
        .add_systems(OnExit(AppState::Dungeon), (spawn::cleanup_dungeon, reset_turn_phase_on_exit))
        .add_systems(
            Update,
            (gabriel::start_gabriel_encounter, spawn::handle_stairs)
                .chain()
                .run_if(in_state(AppState::Dungeon))
                .run_if(in_state(TurnPhase::AwaitingInput)),
        )
        .add_systems(
            Update,
            melee::resolve_player_melee
                .run_if(in_state(AppState::Dungeon))
                .after(crate::core::movement::grid_movement),
        )
        .add_systems(
            Update,
            (ranged::handle_shoot_input, ranged::handle_reload_input)
                .run_if(in_state(AppState::Dungeon))
                .run_if(in_state(TurnPhase::AwaitingInput)),
        )
        .add_systems(
            Update,
            (ranged::resolve_ranged_attack, loot::pickup_items, hazards::check_hazard_tiles, lore::pickup_lore, consumables::resolve_consumable_use)
                .run_if(in_state(AppState::Dungeon))
                .run_if(in_state(TurnPhase::PlayerTurn)),
        )
        .add_systems(
            Update,
            anomalies::check_anomaly_proximity
                .run_if(in_state(AppState::Dungeon)),
        )
        .add_systems(
            Update,
            crate::core::sanity::check_hallucinations
                .run_if(in_state(AppState::Dungeon))
                .run_if(in_state(TurnPhase::WorldTick)),
        )
        .add_systems(
            Update,
            (gabriel::gabriel_turn, ai::enemy_ai_turn, melee::resolve_enemy_melee)
                .chain()
                .run_if(in_state(AppState::Dungeon))
                .run_if(in_state(TurnPhase::EnemyTurn)),
        );
}

/// Reset the turn phase to AwaitingInput when leaving the dungeon so re-entry
/// never starts mid-turn.
fn reset_turn_phase_on_exit(mut next_phase: ResMut<NextState<TurnPhase>>) {
    next_phase.set(TurnPhase::AwaitingInput);
}
