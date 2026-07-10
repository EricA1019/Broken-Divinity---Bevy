//! bd_core — Core ECS components, resources, and systems for the BD Kernel.
//!
//! This crate defines the foundational ECS types and the BdCorePlugin.
//! Gameplay systems (pools, actions, statuses, etc.) are added in later phases.

use bevy_app::{App, Plugin};
use bevy_ecs::prelude::*;
use bevy_ecs::schedule::{IntoScheduleConfigs, SystemSet};

pub mod components;
pub mod direction;
pub mod gamelog;
pub mod ids;
pub mod map;
pub mod pathfinding;
pub mod pools;
pub mod procgen;
pub mod save;
pub mod spatial;
pub mod signals;
pub mod statuses;
pub mod time;
pub mod trace;

mod actions;
pub mod factory;
pub mod colony;
pub mod combat;
pub mod dialogue;
pub mod factions;
pub mod gabriel;
pub mod inventory;
pub mod overworld;
pub mod party;
pub mod sanity;
pub mod virtues;
pub mod relationships;

use crate::trace::{SignalTrace, TriggerExecutionGuard};
use gamelog::GameLog;
use map::SmokeMap;

/// Help line string derived from key bindings, consumed by the TUI footer.
#[derive(Resource, Debug, Clone)]
pub struct HelpLine(pub String);

impl Default for HelpLine {
    fn default() -> Self {
        Self("Move:w↑s↓a←d→ | Wait:. | Attack:f | Guard:g | Inventory:i | Combat:z | Quit:q".into())
    }
}

/// Core system sets defining the execution order for all kernel systems.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum BdSet {
    /// Read input and produce intents.
    Input,
    /// Collect and normalize intents from all sources.
    IntentCollection,
    /// Validate intents against current world state.
    Validation,
    /// Resolve costs (e.g., PoolDelta compilation).
    CostResolution,
    /// Emit effects from validated actions.
    EffectEmission,
    /// Apply modifiers to pending requests.
    ModifierApplication,
    /// Perform actual state mutations (the only stage that mutates gameplay).
    Mutation,
    /// Emit results/logs from mutations.
    ResultEmission,
    /// Build view models for the presentation layer.
    ViewModelBuild,
    /// Render to terminal.
    Render,
}

/// Core plugin — registers system sets, resources, and foundational systems.
pub struct BdCorePlugin;

impl Plugin for BdCorePlugin {
    fn build(&self, app: &mut App) {
        // Register system sets in dependency order
        app.configure_sets(
            bevy_app::Update,
            (
                BdSet::Input,
                BdSet::IntentCollection,
                BdSet::Validation,
                BdSet::CostResolution,
                BdSet::EffectEmission,
                BdSet::ModifierApplication,
                BdSet::Mutation,
                BdSet::ResultEmission,
                BdSet::ViewModelBuild,
                BdSet::Render,
            )
                .chain(),
        );

        // Register resources
        app.insert_resource(SmokeMap::default_smoke_map());
        app.insert_resource(GameLog::default());
        app.insert_resource(SignalTrace::default());
        app.insert_resource(TriggerExecutionGuard::default());
        app.insert_resource(HelpLine::default());

        // Register movement-related message types (used by actions)
        app.add_message::<crate::signals::MoveIntent>();
        app.add_message::<crate::signals::MoveBlocked>();
        app.add_message::<crate::signals::EntityMoved>();

        // Register time system (observes existing messages)
        time::register_time(app);

        // Register pool delta pipeline
        pools::register_pools(app);

        // Register entity cleanup on defeat
        pools::register_cleanup(app);

        // Register action system (replaces direct movement systems)
        actions::register_actions(app);

        // Register status/trigger/modifier system
        statuses::register_statuses(app);
        inventory::register_inventory(app);

        // Register station build action
        app.world_mut()
            .resource_mut::<crate::actions::ActionRegistry>()
            .register(crate::colony::stations::register_station_actions());

        // Register survivor actions
        app.world_mut()
            .resource_mut::<crate::actions::ActionRegistry>()
            .register(crate::colony::survivors::register_assign_task_action());
        app.world_mut()
            .resource_mut::<crate::actions::ActionRegistry>()
            .register(crate::colony::survivors::register_unassign_task_action());

        // Register consume_shelter_resources system
        app.add_systems(
            bevy_app::Update,
            crate::colony::survivors::consume_shelter_resources.in_set(BdSet::Mutation),
        );

        // Register combat actions
        app.world_mut()
            .resource_mut::<crate::actions::ActionRegistry>()
            .register(crate::combat::register_aimed_attack_action());
        app.world_mut()
            .resource_mut::<crate::actions::ActionRegistry>()
            .register(crate::combat::register_quick_attack_action());
        app.world_mut()
            .resource_mut::<crate::actions::ActionRegistry>()
            .register(crate::combat::register_reload_action());
        app.world_mut()
            .resource_mut::<crate::actions::ActionRegistry>()
            .register(crate::combat::register_take_cover_action());

        // Register party actions
        app.world_mut()
            .resource_mut::<crate::actions::ActionRegistry>()
            .register(crate::party::register_add_to_party_action());
        app.world_mut()
            .resource_mut::<crate::actions::ActionRegistry>()
            .register(crate::party::register_remove_from_party_action());

        // Register travel action
        app.world_mut()
            .resource_mut::<crate::actions::ActionRegistry>()
            .register(crate::overworld::register_begin_travel_action());

        // Register colony resources
        app.insert_resource(crate::colony::production::ColonyResources::default());
        app.insert_resource(crate::party::PartyState::default());
        app.insert_resource(crate::overworld::OverworldState::default());
        app.insert_resource(crate::dialogue::DialogueLog::default());
        app.insert_resource(crate::gabriel::GabrielState::default());

        // Register production and raid systems
        app.add_systems(
            bevy_app::Update,
            crate::colony::production::process_production.in_set(BdSet::Mutation),
        );

        tracing::info!("BdCorePlugin initialized");
    }
}
