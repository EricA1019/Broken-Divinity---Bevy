//! Event system — player interruptions with choices and consequences.
//!
//! Events pause normal gameplay, present text + choices, apply effects on selection,
//! and resume play. Used for dialogues, encounters, moral dilemmas, etc.

use std::collections::HashMap;

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    BdSet,
    actions::Effect,
    components::Player,
    dialogue::{Choice, Condition},
    gamelog::GameLog,
    pools::Pools,
    signals::{EventSelected, EventTrigger, PoolDeltaRequested, PoolKind},
};

// ── Event definition types (data-driven, mirrors ActionRegistry) ──

/// A single node in an event tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventNode {
    pub speaker: String,
    pub text: String,
    pub choices: Vec<Choice>,
    /// Effects applied when entering this node (before choices are shown).
    #[serde(default)]
    pub on_enter_effects: Vec<Effect>,
    /// Effects applied when leaving this node (on transition or event end).
    #[serde(default)]
    pub on_exit_effects: Vec<Effect>,
}

/// A complete event with named nodes and a start node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDefinition {
    pub id: String,
    pub start_node: String,
    pub nodes: HashMap<String, EventNode>,
    /// Effects applied when the event starts (entity creation).
    #[serde(default)]
    pub spawn_on_enter: Vec<Effect>,
}

// ── Registry (data-driven, mirrors ActionRegistry) ──

/// Resource holding all event definitions.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventRegistry {
    definitions: Vec<EventDefinition>,
}

impl EventRegistry {
    pub fn get(&self, id: &str) -> Option<&EventDefinition> {
        self.definitions.iter().find(|d| d.id == id)
    }

    pub fn register(&mut self, def: EventDefinition) {
        self.definitions.push(def)
    }
}

// ── Current event state (driver for the TUI event screen) ──

/// Resource tracking the current active event, if any.
///
/// Two fields: the TUI reads `EventRegistry` to render the node text/choices.
/// No duplicated data — single source of truth.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct CurrentEvent {
    pub event_id: String,
    pub node_id: String,
    /// Screen to return to when the event ends.
    pub previous_screen: String,
    pub active: bool,
}

impl CurrentEvent {
    pub fn is_active(&self) -> bool {
        self.active
    }
}

// ── Default event registry (populated from content RON at startup) ──

/// Create an empty default event registry.
/// Events are loaded from `content/events/foundation.ron` and registered at startup.
pub fn default_event_registry() -> EventRegistry {
    EventRegistry::default()
}

// ── Systems ──

/// Evaluate whether a choice condition is met given the player's state.
fn evaluate_condition(condition: &Condition, _player_pools: Option<&Pools>) -> bool {
    match condition {
        Condition::Always => true,
        Condition::HasStatus(_status_id) => {
            // TODO: check player Statuses component
            true // for now, all status conditions pass (MVP)
        }
        Condition::PoolAbove(kind_str, threshold) => {
            // Match known pool kind strings (avoid needing FromStr on PoolKind)
            let kind = match kind_str.as_str() {
                "Health" => Some(PoolKind::Health),
                "ActionPoints" => Some(PoolKind::ActionPoints),
                "Sanity" => Some(PoolKind::Sanity),
                "Fortitude" => Some(PoolKind::Fortitude),
                "Supplies" => Some(PoolKind::Supplies),
                "Faith" => Some(PoolKind::Faith),
                "Morale" => Some(PoolKind::Morale),
                _ => None,
            };
            if let (Some(k), Some(pools)) = (kind, _player_pools) {
                if let Some(pool) = pools.get(k) {
                    return pool.current > *threshold;
                }
            }
            false
        }
        Condition::VirtueAbove(_, _) => true, // TODO: virtue check
        Condition::FactionReputationAbove(_, _) => true, // deferred
    }
}

/// Read `EventTrigger` messages, validate them, and initiate an event.
pub fn process_event_triggers(
    mut triggers: bevy_ecs::message::MessageReader<EventTrigger>,
    registry: Res<EventRegistry>,
    mut current: ResMut<CurrentEvent>,
    player: Query<&Pools, With<Player>>,
    player_entity: Query<Entity, With<Player>>,
    mut game_log: ResMut<GameLog>,
    mut commands: Commands,
    blueprint_catalog: Res<crate::factory::BlueprintCatalog>,
    mut delta_writer: bevy_ecs::message::MessageWriter<PoolDeltaRequested>,
) {
    let player_pools = player.single().ok();

    for trigger in triggers.read() {
        // Validate event_id
        let Some(event_def) = registry.get(&trigger.event_id) else {
            game_log.push(
                format!("Unknown event: {}", trigger.event_id),
                crate::gamelog::LogLevel::Warn,
            );
            continue;
        };

        // Find start node
        let Some(start_node) = event_def.nodes.get(&event_def.start_node) else {
            game_log.push(
                format!("Event '{}' has no start node", trigger.event_id),
                crate::gamelog::LogLevel::Warn,
            );
            continue;
        };

        // Filter choices by conditions
        let available_choices = start_node
            .choices
            .iter()
            .filter(|c| {
                c.conditions
                    .iter()
                    .all(|cond| evaluate_condition(cond, player_pools))
            })
            .count();

        if available_choices == 0 {
            game_log.push(
                format!("Event '{}': all choices filtered out", trigger.event_id),
                crate::gamelog::LogLevel::Warn,
            );
            continue;
        }

        // Initiate the event
        current.event_id = trigger.event_id.clone();
        current.node_id = event_def.start_node.clone();
        current.previous_screen = "combat".into(); // default, TUI refines in E-3
        current.active = true;
        game_log.push(
            format!("Event started: {}", trigger.event_id),
            crate::gamelog::LogLevel::Info,
        );

        // Apply on_enter_effects from the start node
        let player_entity = player_entity.single().ok();
        for effect in &start_node.on_enter_effects {
            resolve_event_effect(
                effect,
                player_entity,
                &mut game_log,
                &mut commands,
                &blueprint_catalog,
                &mut delta_writer,
            );
        }

        // Apply spawn_on_enter from the event definition
        for effect in &event_def.spawn_on_enter {
            resolve_event_effect(
                effect,
                player_entity,
                &mut game_log,
                &mut commands,
                &blueprint_catalog,
                &mut delta_writer,
            );
        }
    }
}

/// Resolve a single Effect in event context (shared by triggers and choices).
fn resolve_event_effect(
    effect: &Effect,
    player_entity: Option<Entity>,
    game_log: &mut GameLog,
    commands: &mut Commands,
    blueprint_catalog: &crate::factory::BlueprintCatalog,
    delta_writer: &mut bevy_ecs::message::MessageWriter<PoolDeltaRequested>,
) {
    match effect {
        Effect::PoolDelta {
            kind,
            amount,
            tags,
            reason,
        } => {
            if let Some(player) = player_entity {
                delta_writer.write(PoolDeltaRequested {
                    source: None,
                    target: player,
                    kind: *kind,
                    amount: *amount,
                    tags: tags.clone(),
                    reason: reason.clone(),
                });
            }
        }
        Effect::Log(msg, level) => {
            game_log.push(msg.clone(), *level);
        }
        Effect::ApplyStatus(status_id) => {
            if let Some(player) = player_entity {
                let defs = crate::statuses::default_status_definitions();
                crate::statuses::apply_status(player, status_id, 3, None, commands, &defs);
            }
        }
        Effect::Flag(name, value) => {
            game_log.push(
                format!("Flag '{}' set to {}", name, value),
                crate::gamelog::LogLevel::Info,
            );
        }
        Effect::SpawnBlueprintAt {
            blueprint_id,
            x,
            y,
            mutators,
        } => {
            let Some(blueprint) = blueprint_catalog.get(blueprint_id) else {
                game_log.push(
                    format!("Unknown blueprint: {blueprint_id}"),
                    crate::gamelog::LogLevel::Warn,
                );
                return;
            };
            let entity = crate::factory::spawn_from_blueprint(
                blueprint,
                Some(crate::components::Position { x: *x, y: *y }),
                mutators,
                commands,
            );
            commands
                .entity(entity)
                .insert(crate::spatial::EntityScope::ColonyPersistent);
        }
        _ => {} // MoveEntity, SpawnEntity, SetSurvivorTask: no-op in event context
    }
}

/// Read `EventSelected` messages, apply effects, advance or end the event.
pub fn process_event_choices(
    mut selections: bevy_ecs::message::MessageReader<EventSelected>,
    mut current: ResMut<CurrentEvent>,
    registry: Res<EventRegistry>,
    mut delta_writer: bevy_ecs::message::MessageWriter<PoolDeltaRequested>,
    mut game_log: ResMut<GameLog>,
    mut commands: Commands,
    player_query: Query<Entity, With<Player>>,
    blueprint_catalog: Res<crate::factory::BlueprintCatalog>,
) {
    let Ok(player_entity) = player_query.single() else {
        return;
    };

    for selection in selections.read() {
        if !current.is_active() {
            continue;
        }
        let Some(event_def) = registry.get(&current.event_id) else {
            continue;
        };
        let Some(node) = event_def.nodes.get(&current.node_id) else {
            continue;
        };

        // Filter available choices (same condition check as triggers)
        let available: Vec<&crate::dialogue::Choice> = node
            .choices
            .iter()
            .filter(|c| {
                c.conditions
                    .iter()
                    .all(|cond| evaluate_condition(cond, None))
            })
            .collect();

        if selection.choice_index >= available.len() {
            continue; // stale or invalid index, silently ignored
        }

        let choice = &available[selection.choice_index];

        // Apply the choice's effects via shared resolver
        for effect in &choice.effects {
            resolve_event_effect(
                effect,
                Some(player_entity),
                &mut game_log,
                &mut commands,
                &blueprint_catalog,
                &mut delta_writer,
            );
        }

        // Apply on_exit_effects from the current node (fires on both transition and event end)
        for effect in &node.on_exit_effects {
            resolve_event_effect(
                effect,
                Some(player_entity),
                &mut game_log,
                &mut commands,
                &blueprint_catalog,
                &mut delta_writer,
            );
        }

        // Advance or end
        if let Some(next_node_id) = &choice.next_node {
            if event_def.nodes.contains_key(next_node_id) {
                current.node_id.clone_from(next_node_id);
            }
        } else {
            // Event ends
            game_log.push("Event ended.".to_string(), crate::gamelog::LogLevel::Info);
            current.active = false;
            current.event_id.clear();
            current.node_id.clear();
        }
    }
}

/// Register event systems.
pub fn register_events(app: &mut bevy_app::App) {
    app.add_systems(
        bevy_app::Update,
        (
            process_event_triggers.in_set(BdSet::IntentCollection),
            process_event_choices.in_set(BdSet::Mutation),
        ),
    );
}

// ── Gabriel encounter trigger ──

pub const GABRIEL_EVENT_ID: &str = "gabriel.first_encounter";
pub const GABRIEL_TRIGGER_FLOOR: u32 = 1;

/// When entering Tactical mode on the first dungeon trip and Gabriel hasn't
/// appeared yet, OR when building an Altar in the Outpost, fire the Gabriel
/// encounter event.
pub fn trigger_gabriel_encounter(
    mode: Res<crate::spatial::GameMode>,
    mut gabriel: ResMut<crate::gabriel::GabrielState>,
    mut trigger_writer: bevy_ecs::message::MessageWriter<EventTrigger>,
    player: Query<Entity, With<Player>>,
    stations: Query<Entity, With<crate::colony::stations::Station>>,
    station_types: Query<&crate::colony::stations::StationType>,
) {
    if gabriel.appeared {
        return;
    }
    let Some(player_entity) = player.iter().next() else {
        return;
    };

    let should_trigger = match *mode {
        crate::spatial::GameMode::Tactical => {
            // Trigger on first dungeon entry
            true
        }
        crate::spatial::GameMode::Outpost => {
            // Trigger only when an Altar station exists (not any station)
            stations.iter().any(|e| {
                station_types
                    .get(e)
                    .is_ok_and(|t| matches!(t, crate::colony::stations::StationType::Altar))
            })
        }
        _ => false,
    };

    if !should_trigger {
        return;
    }

    gabriel.appeared = true;
    trigger_writer.write(EventTrigger {
        actor: player_entity,
        event_id: GABRIEL_EVENT_ID.into(),
    });
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::Effect;

    #[test]
    fn event_registry_lookup() {
        let mut reg = EventRegistry::default();
        let ev = EventDefinition {
            id: "test.event".into(),
            start_node: "start".into(),
            nodes: HashMap::from([(
                "start".into(),
                EventNode {
                    speaker: "NPC".into(),
                    text: "Hello.".into(),
                    choices: vec![],
                    on_enter_effects: vec![],
                    on_exit_effects: vec![],
                },
            )]),
            spawn_on_enter: vec![],
        };
        reg.register(ev);
        let found = reg.get("test.event");
        assert!(found.is_some());
        assert_eq!(found.unwrap().start_node, "start");
    }

    #[test]
    fn event_registry_unknown_returns_none() {
        let reg = EventRegistry::default();
        assert!(reg.get("nonexistent").is_none());
    }

    #[test]
    fn current_event_default_is_not_active() {
        let ev = CurrentEvent::default();
        assert!(!ev.is_active());
    }

    #[test]
    fn flag_effect_is_defined() {
        let flag = Effect::Flag("test.flag".into(), true);
        assert!(matches!(flag, Effect::Flag(_, _)));
    }

    #[test]
    fn event_node_with_enter_effects_roundtrips() {
        let node = EventNode {
            speaker: "Test".into(),
            text: "Text".into(),
            choices: vec![],
            on_enter_effects: vec![Effect::Log(
                "entered".into(),
                crate::gamelog::LogLevel::Info,
            )],
            on_exit_effects: vec![],
        };
        assert_eq!(node.on_enter_effects.len(), 1);
    }

    // ── Session 2: EVT-RON-001 — events load from RON ──

    #[test]
    fn ron_loads_all_required_events() {
        let content_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("content");
        let ron_path = content_root.join("events").join("foundation.ron");
        let raw = std::fs::read_to_string(&ron_path)
            .expect("events/foundation.ron must exist and be readable");
        let events: Vec<EventDefinition> =
            ron::from_str(&raw).expect("events/foundation.ron must be valid RON");

        assert!(
            events.len() >= 3,
            "must have at least 3 events, found {}",
            events.len()
        );

        // All events must have non-empty id and valid start_node
        for ev in &events {
            assert!(!ev.id.is_empty(), "event must have non-empty id");
            assert!(
                ev.nodes.contains_key(&ev.start_node),
                "event '{}' start_node '{}' must exist in nodes",
                ev.id,
                ev.start_node
            );
        }

        // Required events
        let find = |id: &str| events.iter().find(|ev| ev.id == id);

        let small = find("event.raid.small").expect("must have event.raid.small");
        assert_eq!(
            small.spawn_on_enter.len(),
            2,
            "small raid must spawn 2 rats"
        );

        let medium = find("event.raid.medium").expect("must have event.raid.medium");
        assert_eq!(
            medium.spawn_on_enter.len(),
            3,
            "medium raid must spawn 3 rats"
        );

        let gabriel = find("gabriel.first_encounter").expect("must have gabriel.first_encounter");
        let node = gabriel
            .nodes
            .get(&gabriel.start_node)
            .expect("gabriel must have start node");
        assert_eq!(
            node.choices.len(),
            3,
            "gabriel must have 3 choices (Accept, Reject, Defer)"
        );
    }

    // ── E-2 tests: process_event_triggers ──

    use crate::{
        components::{Name, Player, Position, Tile},
        gamelog::GameLog,
        map::SmokeMap,
        pools::Pools,
        signals::EventTrigger,
    };
    use bevy_app::App;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(crate::BdCorePlugin);
        app
    }

    fn register_test_event(app: &mut App) {
        app.world_mut()
            .resource_mut::<EventRegistry>()
            .register(EventDefinition {
                id: "test.event".into(),
                start_node: "start".into(),
                nodes: HashMap::from([(
                    "start".into(),
                    EventNode {
                        speaker: "Test".into(),
                        text: "Hello world.".into(),
                        choices: vec![Choice {
                            label: "Option A".into(),
                            conditions: vec![Condition::Always],
                            effects: vec![],
                            next_node: None,
                        }],
                        on_enter_effects: vec![],
                        on_exit_effects: vec![],
                    },
                )]),
                spawn_on_enter: vec![],
            });
    }

    #[test]
    fn trigger_starts_event() {
        let mut app = test_app();
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        let player = app
            .world_mut()
            .spawn((Player, Position { x: 5, y: 5 }, Pools::new(vec![])))
            .id();
        register_test_event(&mut app);

        app.world_mut()
            .resource_mut::<bevy_ecs::message::Messages<EventTrigger>>()
            .write(EventTrigger {
                actor: player,
                event_id: "test.event".into(),
            });

        app.update();
        let ev = app.world_mut().resource::<CurrentEvent>();
        assert!(ev.is_active());
        assert_eq!(ev.event_id, "test.event");
    }

    #[test]
    fn trigger_with_unknown_id_ignored() {
        let mut app = test_app();
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        let player = app
            .world_mut()
            .spawn((Player, Position { x: 5, y: 5 }, Pools::new(vec![])))
            .id();

        app.world_mut()
            .resource_mut::<bevy_ecs::message::Messages<EventTrigger>>()
            .write(EventTrigger {
                actor: player,
                event_id: "bogus.nope".into(),
            });

        app.update();
        let ev = app.world_mut().resource::<CurrentEvent>();
        assert!(!ev.is_active());
        let log = app.world_mut().resource::<GameLog>();
        assert!(log.iter().any(|e| e.message.contains("Unknown event")));
    }

    // ── E-4 tests: process_event_choices ──

    use crate::signals::EventSelected;

    #[test]
    fn choice_ends_event() {
        let mut app = test_app();
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        let player = app
            .world_mut()
            .spawn((Player, Position { x: 5, y: 5 }, Pools::new(vec![])))
            .id();
        register_test_event(&mut app);

        // Trigger the event
        app.world_mut()
            .resource_mut::<bevy_ecs::message::Messages<EventTrigger>>()
            .write(EventTrigger {
                actor: player,
                event_id: "test.event".into(),
            });
        app.update();
        assert!(app.world_mut().resource::<CurrentEvent>().is_active());

        // Select the only choice (which has next_node=None → event ends)
        app.world_mut()
            .resource_mut::<bevy_ecs::message::Messages<EventSelected>>()
            .write(EventSelected {
                actor: player,
                choice_index: 0,
            });
        app.update();

        let ev = app.world_mut().resource::<CurrentEvent>();
        assert!(!ev.is_active());
    }

    #[test]
    fn invalid_choice_index_ignored() {
        let mut app = test_app();
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        let player = app
            .world_mut()
            .spawn((Player, Position { x: 5, y: 5 }, Pools::new(vec![])))
            .id();
        register_test_event(&mut app);

        // Trigger the event
        app.world_mut()
            .resource_mut::<bevy_ecs::message::Messages<EventTrigger>>()
            .write(EventTrigger {
                actor: player,
                event_id: "test.event".into(),
            });
        app.update();

        // Select an out-of-range index
        app.world_mut()
            .resource_mut::<bevy_ecs::message::Messages<EventSelected>>()
            .write(EventSelected {
                actor: player,
                choice_index: 99,
            });
        app.update();

        // Event should still be active (invalid index was silently ignored)
        let ev = app.world_mut().resource::<CurrentEvent>();
        assert!(ev.is_active());

        let log = app.world_mut().resource::<GameLog>();
        assert!(!log.iter().any(|e| e.message.contains("Event ended")));
    }

    // ── E-5 tests: Gabriel encounter ──

    use crate::gabriel::GabrielState;
    use crate::spatial::GameMode;

    /// Register a minimal Gabriel event so tests work without default_event_registry().
    fn register_gabriel_event(app: &mut App) {
        app.world_mut()
            .resource_mut::<EventRegistry>()
            .register(EventDefinition {
                id: GABRIEL_EVENT_ID.into(),
                start_node: "start".into(),
                nodes: HashMap::from([(
                    "start".into(),
                    EventNode {
                        speaker: "Gabriel".into(),
                        text: "I am Gabriel.".into(),
                        choices: vec![Choice {
                            label: "Acknowledge".into(),
                            conditions: vec![Condition::Always],
                            effects: vec![],
                            next_node: None,
                        }],
                        on_enter_effects: vec![],
                        on_exit_effects: vec![],
                    },
                )]),
                spawn_on_enter: vec![],
            });
    }

    #[test]
    fn gabriel_triggers_on_tactical_entry() {
        let mut app = test_app();
        register_gabriel_event(&mut app);
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        app.world_mut()
            .spawn((Player, Position { x: 5, y: 5 }, Pools::new(vec![])));
        // Modify existing GameMode resource rather than re-inserting
        *app.world_mut().resource_mut::<GameMode>() = GameMode::Tactical;

        // Verify state before update
        assert!(
            !app.world_mut().resource::<GabrielState>().appeared,
            "Gabriel should not have appeared yet"
        );
        assert_eq!(*app.world_mut().resource::<GameMode>(), GameMode::Tactical);

        app.update();
        // After first update, trigger_gabriel_encounter should have fired
        assert!(
            app.world_mut().resource::<GabrielState>().appeared,
            "Gabriel should have appeared after first update"
        );

        app.update(); // second frame: process_event_triggers reads the message

        // Gabriel event should have been triggered
        let ev = app.world_mut().resource::<CurrentEvent>();
        assert!(ev.is_active(), "Event should be active");
        assert_eq!(ev.event_id, GABRIEL_EVENT_ID);
    }

    #[test]
    fn gabriel_does_not_trigger_twice() {
        let mut app = test_app();
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        app.world_mut()
            .spawn((Player, Position { x: 5, y: 5 }, Pools::new(vec![])));
        *app.world_mut().resource_mut::<GameMode>() = GameMode::Tactical;
        app.world_mut().resource_mut::<GabrielState>().appeared = true; // already seen

        app.update();

        // Gabriel should NOT trigger again
        let ev = app.world_mut().resource::<CurrentEvent>();
        assert!(!ev.is_active());
    }

    // ── Phase 3: spawn_on_enter + on_exit_effects tests ──

    use crate::factory::BlueprintCatalog;
    use crate::factory::EntityBlueprint;
    use crate::signals::PoolKind;

    /// App with BlueprintCatalog containing a rat blueprint.
    fn spawn_event_test_app() -> App {
        let mut app = test_app();
        app.world_mut().insert_resource(BlueprintCatalog::new(vec![
            EntityBlueprint {
                id: "blueprint.raid_rat".into(),
                label: "Rat".into(),
                is_player: false,
                blocks_movement: true,
                pools: vec![(PoolKind::Health, 11, 0, 11), (PoolKind::ActionPoints, 2, 0, 2)],
                statuses: vec![],
                visual: Some("Enemy".into()),
                markers: vec![],
            },
        ]));
        app
    }

    /// Register an event with spawn_on_enter + optional on_exit on the start node.
    fn register_spawn_event(
        app: &mut App,
        event_id: &str,
        spawn_effects: Vec<Effect>,
        on_exit_effects: Vec<Effect>,
        has_second_node: bool,
    ) {
        let mut nodes = HashMap::from([(
            "start".into(),
            EventNode {
                speaker: "Test".into(),
                text: "Spawn event.".into(),
                choices: vec![Choice {
                    label: "Done".into(),
                    conditions: vec![Condition::Always],
                    effects: vec![],
                    next_node: if has_second_node {
                        Some("node2".into())
                    } else {
                        None
                    },
                }],
                on_enter_effects: vec![],
                on_exit_effects,
            },
        )]);

        if has_second_node {
            nodes.insert(
                "node2".into(),
                EventNode {
                    speaker: "Test".into(),
                    text: "Second node.".into(),
                    choices: vec![Choice {
                        label: "End".into(),
                        conditions: vec![Condition::Always],
                        effects: vec![],
                        next_node: None,
                    }],
                    on_enter_effects: vec![],
                    on_exit_effects: vec![],
                },
            );
        }

        app.world_mut()
            .resource_mut::<EventRegistry>()
            .register(EventDefinition {
                id: event_id.into(),
                start_node: "start".into(),
                nodes,
                spawn_on_enter: spawn_effects,
            });
    }

    /// Trigger an event and return a reference to the player entity.
    /// Returns the player entity so tests can submit choice selections.
    fn trigger_event_in(app: &mut App, event_id: &str) -> Entity {
        let player = app
            .world_mut()
            .spawn((
                Player,
                Position { x: 1, y: 1 },
                Pools::new(vec![
                    crate::pools::Pool::new(PoolKind::Health, 20, 0, 20),
                    crate::pools::Pool::new(PoolKind::ActionPoints, 3, 0, 3),
                ]),
            ))
            .id();
        app.world_mut()
            .resource_mut::<bevy_ecs::message::Messages<EventTrigger>>()
            .write(EventTrigger {
                actor: player,
                event_id: event_id.into(),
            });
        app.update();
        player
    }

    #[test]
    fn spawn_on_enter_creates_entities() {
        let mut app = spawn_event_test_app();
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));

        register_spawn_event(
            &mut app,
            "test.spawn",
            vec![Effect::SpawnBlueprintAt {
                blueprint_id: "blueprint.raid_rat".into(),
                x: 2,
                y: 2,
                mutators: vec![],
            }],
            vec![],
            false,
        );
        // Spawn player + trigger event (inline to avoid borrow issues)
        let player = app
            .world_mut()
            .spawn((Player, Position { x: 1, y: 1 }, Pools::new(vec![])))
            .id();
        app.world_mut()
            .resource_mut::<bevy_ecs::message::Messages<EventTrigger>>()
            .write(EventTrigger {
                actor: player,
                event_id: "test.spawn".into(),
            });
        app.update();

        let ev = app.world_mut().resource::<CurrentEvent>();
        assert!(ev.is_active(), "event must be active after spawn_on_enter");

        // Entity at (2,2) with Name "Rat"
        let rat = app
            .world_mut()
            .query_filtered::<(Entity, &Position), Without<Player>>()
            .iter(app.world_mut())
            .find(|(_, pos)| pos.x == 2 && pos.y == 2);

        assert!(rat.is_some(), "spawn_on_enter must create entity at (2,2)");
        let (rat_entity, _) = rat.unwrap();
        let name = app.world_mut().get::<Name>(rat_entity).unwrap();
        assert_eq!(name.0, "Rat");
    }

    #[test]
    fn on_exit_fires_on_event_end() {
        let mut app = spawn_event_test_app();
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));

        register_spawn_event(
            &mut app,
            "test.onexit_end",
            vec![],
            vec![Effect::SpawnBlueprintAt {
                blueprint_id: "blueprint.raid_rat".into(),
                x: 3,
                y: 3,
                mutators: vec![],
            }],
            false, // no second node → choice ends event
        );

        let player = trigger_event_in(&mut app, "test.onexit_end");
        assert!(app.world_mut().resource::<CurrentEvent>().is_active());

        // Select choice to end event
        app.world_mut()
            .resource_mut::<bevy_ecs::message::Messages<EventSelected>>()
            .write(EventSelected {
                actor: player,
                choice_index: 0,
            });
        app.update();

        let ev = app.world_mut().resource::<CurrentEvent>();
        assert!(!ev.is_active(), "event must end after choice");

        // on_exit should have spawned entity at (3,3)
        let rat = app
            .world_mut()
            .query_filtered::<(Entity, &Position), Without<Player>>()
            .iter(app.world_mut())
            .find(|(_, pos)| pos.x == 3 && pos.y == 3);

        assert!(rat.is_some(), "on_exit_effects must spawn entity at (3,3)");
    }

    #[test]
    fn on_exit_fires_on_node_transition() {
        let mut app = spawn_event_test_app();
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));

        register_spawn_event(
            &mut app,
            "test.onexit_trans",
            vec![],
            vec![Effect::SpawnBlueprintAt {
                blueprint_id: "blueprint.raid_rat".into(),
                x: 4,
                y: 4,
                mutators: vec![],
            }],
            true, // has second node → choice advances to node2
        );

        let player = trigger_event_in(&mut app, "test.onexit_trans");
        assert_eq!(
            app.world_mut().resource::<CurrentEvent>().node_id,
            "start"
        );

        // Select choice → transition to node2
        app.world_mut()
            .resource_mut::<bevy_ecs::message::Messages<EventSelected>>()
            .write(EventSelected {
                actor: player,
                choice_index: 0,
            });
        app.update();

        // on_exit from start node should have spawned entity at (4,4)
        let rat = app
            .world_mut()
            .query_filtered::<(Entity, &Position), Without<Player>>()
            .iter(app.world_mut())
            .find(|(_, pos)| pos.x == 4 && pos.y == 4);

        assert!(
            rat.is_some(),
            "on_exit must fire on node transition, entity at (4,4)"
        );

        // Event should still be active, now on node2
        let ev = app.world_mut().resource::<CurrentEvent>();
        assert!(ev.is_active(), "event must still be active after transition");
        assert_eq!(ev.node_id, "node2");
    }

    #[test]
    fn mixed_effects_in_spawn_on_enter() {
        let mut app = spawn_event_test_app();
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));

        register_spawn_event(
            &mut app,
            "test.mixed",
            vec![
                Effect::PoolDelta {
                    kind: PoolKind::Health,
                    amount: -1,
                    tags: vec![],
                    reason: "test".into(),
                },
                Effect::SpawnBlueprintAt {
                    blueprint_id: "blueprint.raid_rat".into(),
                    x: 5,
                    y: 5,
                    mutators: vec![],
                },
            ],
            vec![],
            false,
        );

        let player = trigger_event_in(&mut app, "test.mixed");

        // PoolDelta effect must have applied (actor takes 1 Health damage)
        let pools = app.world_mut().get::<Pools>(player).unwrap();
        let hp = pools.get(PoolKind::Health).unwrap();
        assert_eq!(
            hp.current, 19,
            "PoolDelta from spawn_on_enter must apply (20 - 1 = 19)"
        );

        let rat = app
            .world_mut()
            .query_filtered::<(Entity, &Position), Without<Player>>()
            .iter(app.world_mut())
            .find(|(_, pos)| pos.x == 5 && pos.y == 5);
        assert!(
            rat.is_some(),
            "SpawnBlueprintAt must be applied from spawn_on_enter alongside Log"
        );
    }

    #[test]
    fn invalid_blueprint_in_event_does_not_crash() {
        let mut app = spawn_event_test_app();
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));

        register_spawn_event(
            &mut app,
            "test.bad_bp",
            vec![Effect::SpawnBlueprintAt {
                blueprint_id: "blueprint.nonexistent".into(),
                x: 9,
                y: 9,
                mutators: vec![],
            }],
            vec![],
            false,
        );

        trigger_event_in(&mut app, "test.bad_bp");

        // Event must still be active (missing blueprint doesn't block event)
        let ev = app.world_mut().resource::<CurrentEvent>();
        assert!(
            ev.is_active(),
            "event must be active despite invalid blueprint"
        );

        // No entity at (9,9)
        let rat = app
            .world_mut()
            .query_filtered::<(Entity, &Position), Without<Player>>()
            .iter(app.world_mut())
            .find(|(_, pos)| pos.x == 9 && pos.y == 9);
        assert!(rat.is_none(), "no entity must be spawned for invalid blueprint");

        // Warning logged about the invalid blueprint
        let log = app.world_mut().resource::<GameLog>();
        assert!(
            log.iter().any(|e| e.level == crate::gamelog::LogLevel::Warn
                && e.message.to_lowercase().contains("blueprint")),
            "warning must be logged for invalid blueprint"
        );
    }
}
