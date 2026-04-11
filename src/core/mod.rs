pub mod abilities;
pub mod camera;
pub mod components;
pub mod fov;
pub mod gamelog;
pub mod inventory;
pub mod items;
pub mod movement;
pub mod perks;
pub mod player;
pub mod resources;
pub mod sanity;
pub mod save;
pub mod state;
pub mod stats;
pub mod status;
pub mod tilemap;
pub mod turn;

use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    // --- Type registration for BRP reflection ---
    app.register_type::<state::AppState>()
        .register_type::<turn::TurnPhase>()
        .register_type::<turn::GameTime>()
        .register_type::<turn::ActionBudget>()
        .register_type::<turn::EnemyTurnFrameCounter>()
        .register_type::<components::TileKind>()
        .register_type::<components::Player>()
        .register_type::<components::Enemy>()
        .register_type::<components::Position>()
        .register_type::<components::Viewshed>()
        .register_type::<stats::SkillId>()
        .register_type::<stats::SkillState>()
        .register_type::<stats::CombatStats>()
        .register_type::<stats::EntityName>()
        .register_type::<items::ItemKind>()
        .register_type::<items::ItemStack>()
        .register_type::<items::ItemDrop>()
        .register_type::<inventory::Inventory>()
        .register_type::<inventory::Equipment>()
        .register_type::<inventory::ArmorDurability>()
        .register_type::<inventory::RangedWeaponState>()
        .register_type::<status::StatusKind>()
        .register_type::<status::StatusEffect>()
        .register_type::<status::StatusEffects>()
        .register_type::<sanity::RaidExposure>()
        .register_type::<sanity::Hallucination>()
        .register_type::<perks::PerkId>()
        .register_type::<perks::PlayerPerks>()
        .register_type::<perks::PendingPerkChoices>()
        .register_type::<gamelog::LogEntry>()
        .register_type::<gamelog::LogColor>()
        .register_type::<gamelog::GameLog>()
        .register_type::<resources::WorldSeed>()
        .register_type::<resources::ShelterResources>()
        .register_type::<resources::ResourceKind>();

    app.init_state::<turn::TurnPhase>()
        .init_resource::<turn::GameTime>()
        .init_resource::<turn::PlayerAction>()
        .init_resource::<turn::EnemyTurnFrameCounter>()
        .init_resource::<gamelog::GameLog>()
        .init_resource::<perks::PendingPerkChoices>()
        .add_systems(Startup, camera::setup_camera)
        .add_systems(
            Update,
            (
                movement::grid_movement,
                movement::sync_position_to_transform,
                fov::update_viewshed,
                camera::camera_follow,
            )
                .chain()
                .run_if(in_state(state::AppState::Dungeon).or(in_state(state::AppState::Colony))),
        )
        .add_systems(
            Update,
            turn::advance_turn_phase.run_if(in_state(state::AppState::Dungeon)),
        )
        .add_systems(
            Update,
            (turn::reset_action_budgets, turn::tick_sprint_cooldown, status::tick_status_effects)
                .run_if(in_state(state::AppState::Dungeon))
                .run_if(in_state(turn::TurnPhase::WorldTick)),
        );
}
