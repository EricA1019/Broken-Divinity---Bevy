//! bd_core — Core ECS components, resources, and systems for the BD Kernel.
//!
//! This crate defines the foundational ECS types and the BdCorePlugin.
//! Gameplay systems (pools, actions, statuses, etc.) are added in later phases.

use bevy_app::{App, Plugin};
use bevy_ecs::prelude::*;
use bevy_ecs::schedule::{IntoScheduleConfigs, SystemSet};

pub mod components;
pub mod content;
pub mod direction;
pub mod gamelog;
pub mod ids;
pub mod map;
pub mod pathfinding;
pub mod pools;
pub mod procgen;
pub mod save;
pub mod session;
pub mod spatial;
use spatial::register_spatial;
pub mod signals;
pub mod statuses;
pub mod time;
pub mod trace;

mod actions;
pub mod colony;
pub mod combat;
pub mod dialogue;
pub mod enemy_ai;
pub mod events;
pub mod factions;
pub mod factory;
use factions::register_factions;
pub mod gabriel;
pub mod inventory;
pub mod overworld;
pub mod party;
pub mod progression;
pub mod relationships;
pub mod sanity;
pub mod virtues;

use crate::trace::{SignalTrace, TriggerExecutionGuard};
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

/// Explicit order for request producers and consumers inside authoritative
/// mutation. This prevents a checkpoint from observing an emitted combat
/// delta before that delta has been applied.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum BdMutationSet {
    ActionEffects,
    PoolDeltas,
}

/// Full legacy plugin — registers the foundation plus deferred systems.
///
/// Existing module tests use this plugin because they exercise deferred
/// systems in isolation. The application and foundation acceptance tests must
/// use [`BdFoundationPlugin`] instead.
pub struct BdCorePlugin;

pub fn foundation_action_is_registered(world: &World, action_id: &str) -> bool {
    world
        .get_resource::<actions::ActionRegistry>()
        .is_some_and(|registry| registry.get(action_id).is_some())
}

impl Plugin for BdCorePlugin {
    fn build(&self, app: &mut App) {
        register_foundation(app, false);
        register_deferred(app);
        tracing::info!("BdCorePlugin initialized");
    }
}

/// Foundation-only plugin used by the application and MVP tests.
///
/// This intentionally excludes sanity, raids, events, Gabriel, overworld
/// travel, party expansion, and reputation systems. Those systems remain
/// available through [`BdCorePlugin`] for legacy tests and later products.
pub struct BdFoundationPlugin;

impl Plugin for BdFoundationPlugin {
    fn build(&self, app: &mut App) {
        register_foundation(app, true);
        tracing::info!("BdFoundationPlugin initialized");
    }
}

/// Register the systems and resources required by the current MVP foundation.
fn register_foundation(app: &mut App, foundation: bool) {
    session::register_session(app, foundation);

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
    app.configure_sets(
        bevy_app::Update,
        (BdMutationSet::ActionEffects, BdMutationSet::PoolDeltas)
            .chain()
            .in_set(BdSet::Mutation),
    );

    // Register resources
    app.insert_resource(SmokeMap::default_smoke_map());
    app.insert_resource(GameLog::default());
    app.insert_resource(SignalTrace::default());
    app.insert_resource(TriggerExecutionGuard::default());
    app.init_resource::<crate::save::SaveRequest>();
    app.init_resource::<crate::save::LoadRequest>();

    // Register CombatRng for d100 damage variance
    app.init_resource::<crate::combat::CombatRng>();

    // Register spatial systems (transitions, game mode)
    register_spatial(app);

    // Register movement-related message types (used by actions)
    app.add_message::<crate::signals::MoveIntent>();
    app.add_message::<crate::signals::MoveBlocked>();
    app.add_message::<crate::signals::EntityMoved>();

    // The TUI can be present in the foundation runtime even though event
    // content is deferred. Keep its optional event-input channel safe.
    if foundation {
        app.add_message::<crate::signals::EventSelected>();
    }

    // Register colony assignment message
    app.add_message::<crate::signals::AssignToStation>();
    app.add_message::<crate::signals::AssignRecipe>();
    app.init_resource::<crate::colony::logistics::PendingRecipeAssignment>();

    // Register time system (observes existing messages)
    time::register_time(app);

    // Register pool delta pipeline
    pools::register_pools(app);

    // Register entity cleanup on defeat
    pools::register_cleanup(app);

    // Register movement feedback (blocked log)
    pools::register_move_feedback(app);

    // Register colony resources (must exist before actions validate them)
    app.init_resource::<crate::colony::stations::PendingStationAssignment>();
    app.init_resource::<crate::colony::stations::BuildInteraction>();
    app.init_resource::<crate::colony::stations::StationCatalog>();
    app.init_resource::<crate::factory::BlueprintCatalog>();
    app.insert_resource(crate::colony::production::ColonyResources::default());
    app.init_resource::<crate::colony::production::ColonyStorage>();
    app.init_resource::<crate::colony::production::DailyCycleDraft>();
    app.init_resource::<crate::colony::production::LatestDailySummary>();
    app.init_resource::<crate::colony::proximity::NearbyInteractables>();
    app.add_message::<crate::colony::production::DailySummary>();

    // Register action system (replaces direct movement systems)
    actions::register_actions(app);
    app.world_mut()
        .resource_mut::<crate::actions::ActionRegistry>()
        .register(crate::time::register_rest_until_next_day_action());
    progression::register_progression(app);

    // Register status/trigger/modifier system
    statuses::register_statuses(app);
    // Register virtue gain systems
    virtues::register_virtues(app);
    inventory::register_inventory(app);

    // Register station build action
    app.world_mut()
        .resource_mut::<crate::actions::ActionRegistry>()
        .register(crate::colony::stations::register_station_actions());
    app.world_mut()
        .resource_mut::<crate::actions::ActionRegistry>()
        .register(crate::spatial::register_foundation_entry_action());

    // Register survivor actions
    app.world_mut()
        .resource_mut::<crate::actions::ActionRegistry>()
        .register(crate::colony::survivors::register_assign_gathering_action());
    app.world_mut()
        .resource_mut::<crate::actions::ActionRegistry>()
        .register(crate::colony::survivors::register_gather_supplies_action());
    app.world_mut()
        .resource_mut::<crate::actions::ActionRegistry>()
        .register(crate::colony::survivors::register_gather_materials_action());
    app.world_mut()
        .resource_mut::<crate::actions::ActionRegistry>()
        .register(crate::colony::survivors::register_gather_plants_action());
    app.world_mut()
        .resource_mut::<crate::actions::ActionRegistry>()
        .register(crate::colony::survivors::register_unassign_task_action());
    app.world_mut()
        .resource_mut::<crate::actions::ActionRegistry>()
        .register(crate::colony::survivors::register_assign_defending_action());
    app.world_mut()
        .resource_mut::<crate::actions::ActionRegistry>()
        .register(crate::colony::survivors::register_assign_resting_action());
    app.world_mut()
        .resource_mut::<crate::actions::ActionRegistry>()
        .register(crate::colony::survivors::register_assign_idle_action());
    app.world_mut()
        .resource_mut::<crate::actions::ActionRegistry>()
        .register(crate::colony::logistics::register_assign_recipe_action());

    // Register consume_shelter_resources system
    app.add_systems(
        bevy_app::Update,
        crate::colony::survivors::consume_shelter_resources
            .after(crate::colony::production::process_production)
            .before(crate::pools::resolve_pool_deltas)
            .in_set(BdSet::Mutation),
    );
    app.add_systems(
        bevy_app::Update,
        crate::colony::logistics::process_recipe_assignments
            .after(crate::actions::resolve_action_effects)
            .in_set(BdSet::Mutation),
    );

    // Register station assignment system (reads AssignToStation messages)
    app.add_systems(
        bevy_app::Update,
        crate::colony::survivors::process_station_assignments
            .after(crate::actions::resolve_action_effects)
            .in_set(BdSet::Mutation),
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

    // Register enemy AI (process_enemy_turns in BdSet::Input)
    enemy_ai::register_enemy_ai(app);

    // Register production and raid systems
    app.add_systems(
        bevy_app::Update,
        crate::colony::production::process_production.in_set(BdSet::Mutation),
    );

    app.add_systems(
        bevy_app::Update,
        crate::colony::production::finalize_daily_cycle
            .after(crate::colony::survivors::consume_shelter_resources)
            .in_set(BdSet::Mutation),
    );

    // P3: Register survivor movement system
    app.add_systems(
        bevy_app::Update,
        crate::colony::logistics::process_logistics_workers
            .after(crate::actions::resolve_action_effects)
            .after(crate::colony::survivors::process_station_assignments)
            .before(crate::colony::survivors::process_survivor_movement)
            .in_set(BdSet::Mutation),
    );
    app.add_systems(
        bevy_app::Update,
        crate::colony::stations::process_idle_construction
            .after(crate::actions::resolve_action_effects)
            .after(crate::colony::logistics::process_logistics_workers)
            .before(crate::colony::survivors::process_survivor_movement)
            .in_set(BdSet::Mutation),
    );
    app.add_systems(
        bevy_app::Update,
        crate::colony::survivors::process_survivor_movement
            .after(crate::actions::resolve_action_effects)
            .after(crate::colony::survivors::process_station_assignments)
            .after(crate::colony::stations::process_idle_construction)
            .in_set(BdSet::Mutation),
    );
    app.add_systems(
        bevy_app::Update,
        crate::colony::survivors::report_assignment_feedback
            .after(crate::colony::survivors::process_survivor_movement)
            .in_set(BdSet::Mutation),
    );

    // Nearby interactable projection: refresh after accepted movement and emit
    // one Chronicle fact per newly entered target. Runs after mutation so it
    // observes the accepted destination; rendering never writes here.
    app.add_systems(
        bevy_app::Update,
        crate::colony::proximity::update_nearby_interactables.in_set(BdSet::ResultEmission),
    );
}

/// Register systems intentionally excluded from the MVP foundation.
fn register_deferred(app: &mut App) {
    // Event and Gabriel systems
    app.insert_resource(crate::events::default_event_registry());
    app.insert_resource(crate::events::CurrentEvent::default());
    app.add_message::<crate::signals::EventTrigger>();
    app.add_message::<crate::signals::EventSelected>();
    crate::events::register_events(app);
    app.add_systems(
        bevy_app::Update,
        crate::events::trigger_gabriel_encounter.in_set(crate::BdSet::Mutation),
    );

    // Sanity systems
    sanity::register_sanity(app);

    // Party and overworld systems
    app.insert_resource(crate::party::PartyState::default());
    app.insert_resource(crate::overworld::OverworldState::default());
    app.insert_resource(crate::overworld::TravelContext::default());
    app.insert_resource(crate::dialogue::DialogueLog::default());
    app.insert_resource(crate::gabriel::GabrielState::default());
    app.world_mut()
        .resource_mut::<crate::actions::ActionRegistry>()
        .register(crate::party::register_add_to_party_action());
    app.world_mut()
        .resource_mut::<crate::actions::ActionRegistry>()
        .register(crate::party::register_remove_from_party_action());
    app.world_mut()
        .resource_mut::<crate::actions::ActionRegistry>()
        .register(crate::overworld::register_begin_travel_action());
    crate::overworld::register_travel(app);

    // Faction reputation and raid systems
    register_factions(app);
    crate::colony::raids::register_raids(app);
}
