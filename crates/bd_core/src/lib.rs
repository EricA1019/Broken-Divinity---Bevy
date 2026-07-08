//! bd_core — Core ECS components, resources, and systems for the BD Kernel.
//!
//! This crate defines the foundational ECS types and the BdCorePlugin.
//! Gameplay systems (pools, actions, statuses, etc.) are added in later phases.

use bevy_app::{App, Plugin};
use bevy_ecs::schedule::{IntoScheduleConfigs, SystemSet};

pub mod components;
pub mod direction;
pub mod gamelog;
pub mod map;
mod movement;
pub mod pools;
pub mod signals;

use gamelog::GameLog;
use map::SmokeMap;

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

        // Register movement systems
        movement::register_movement(app);

        // Register pool delta pipeline
        pools::register_pools(app);

        tracing::info!("BdCorePlugin initialized");
    }
}
