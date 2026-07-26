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
}

/// A complete event with named nodes and a start node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDefinition {
    pub id: String,
    pub start_node: String,
    pub nodes: HashMap<String, EventNode>,
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

// ── Default event registry (empty — populated in E-5 for Gabriel) ──

/// Create the default event registry.
pub fn default_event_registry() -> EventRegistry {
    let mut reg = EventRegistry::default();

    // ── Gabriel first encounter ──
    reg.register(EventDefinition {
        id: "gabriel.first_encounter".into(),
        start_node: "start".into(),
        nodes: HashMap::from([(
            "start".into(),
            EventNode {
                speaker: "Gabriel".into(),
                text: "You have walked far, mortal. I have watched you since the Sundering.\n\nI offer you my witness. Carry it, and you shall never face the dark alone.\n\nDo you accept?".into(),
                choices: vec![
                    Choice {
                        label: "Accept the witness".into(),
                        conditions: vec![Condition::Always],
                        effects: vec![
                            Effect::ApplyStatus("status.gabriels_witness".into()),
                            Effect::Flag("gabriel.accepted".into(), true),
                            Effect::PoolDelta {
                                kind: PoolKind::Faith,
                                amount: 10,
                                tags: vec![],
                                reason: "accepted Gabriel's witness".into(),
                            },
                            Effect::PoolDelta {
                                kind: PoolKind::Temperance,
                                amount: 2,
                                tags: vec![],
                                reason: "accepted Gabriel's witness".into(),
                            },
                            Effect::PoolDelta {
                                kind: PoolKind::RepPuritans,
                                amount: -5,
                                tags: vec![],
                                reason: "consorting with the mysterious".into(),
                            },
                            Effect::Log("Gabriel's presence settles over you like a mantle. You are no longer alone.".into(), crate::gamelog::LogLevel::Info),
                        ],
                        next_node: None,
                    },
                    Choice {
                        label: "Reject the offer".into(),
                        conditions: vec![Condition::Always],
                        effects: vec![
                            Effect::PoolDelta {
                                kind: PoolKind::Prudence,
                                amount: 2,
                                tags: vec![],
                                reason: "rejected the unknown".into(),
                            },
                            Effect::PoolDelta {
                                kind: PoolKind::Fortitude,
                                amount: 1,
                                tags: vec![],
                                reason: "stood firm against temptation".into(),
                            },
                            Effect::PoolDelta {
                                kind: PoolKind::RepPuritans,
                                amount: 3,
                                tags: vec![],
                                reason: "rejected mysterious entity".into(),
                            },
                            Effect::Log("Gabriel's form flickers with something like disappointment. 'As you wish.' He fades.".into(), crate::gamelog::LogLevel::Info),
                        ],
                        next_node: None,
                    },
                    Choice {
                        label: "Defer — ask for time".into(),
                        conditions: vec![Condition::Always],
                        effects: vec![
                            Effect::PoolDelta {
                                kind: PoolKind::Justice,
                                amount: 1,
                                tags: vec![],
                                reason: "sought wisdom before deciding".into(),
                            },
                            Effect::PoolDelta {
                                kind: PoolKind::Metis,
                                amount: 1,
                                tags: vec![],
                                reason: "displayed caution".into(),
                            },
                            Effect::Log("'Time is the one thing I have in abundance.' Gabriel lingers, waiting.".into(), crate::gamelog::LogLevel::Info),
                        ],
                        next_node: None,
                    },
                ],
                on_enter_effects: vec![],
            },
        )]),
    });

    reg
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
    mut game_log: ResMut<GameLog>,
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

        // Apply the choice's effects
        for effect in &choice.effects {
            match effect {
                Effect::PoolDelta {
                    kind,
                    amount,
                    tags: _,
                    reason: _,
                } => {
                    delta_writer.write(PoolDeltaRequested {
                        source: None,
                        target: player_entity,
                        kind: *kind,
                        amount: *amount,
                        tags: vec![],
                        reason: "event choice".into(),
                    });
                }
                Effect::Log(msg, level) => {
                    game_log.push(msg.clone(), *level);
                }
                Effect::ApplyStatus(status_id) => {
                    let defs = crate::statuses::default_status_definitions();
                    crate::statuses::apply_status(
                        player_entity,
                        status_id,
                        3,
                        None,
                        &mut commands,
                        &defs,
                    );
                }
                Effect::Flag(name, value) => {
                    game_log.push(
                        format!("Flag '{}' set to {}", name, value),
                        crate::gamelog::LogLevel::Info,
                    );
                }
                _ => {} // MoveEntity, SpawnEntity, SetSurvivorTask: no-op in event context
            }
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
                },
            )]),
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
        };
        assert_eq!(node.on_enter_effects.len(), 1);
    }

    // ── E-2 tests: process_event_triggers ──

    use crate::{
        components::{Player, Position, Tile},
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
                    },
                )]),
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
        let ev = app.world().resource::<CurrentEvent>();
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
        let ev = app.world().resource::<CurrentEvent>();
        assert!(!ev.is_active());
        let log = app.world().resource::<GameLog>();
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
        assert!(app.world().resource::<CurrentEvent>().is_active());

        // Select the only choice (which has next_node=None → event ends)
        app.world_mut()
            .resource_mut::<bevy_ecs::message::Messages<EventSelected>>()
            .write(EventSelected {
                actor: player,
                choice_index: 0,
            });
        app.update();

        let ev = app.world().resource::<CurrentEvent>();
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
        let ev = app.world().resource::<CurrentEvent>();
        assert!(ev.is_active());

        let log = app.world().resource::<GameLog>();
        assert!(!log.iter().any(|e| e.message.contains("Event ended")));
    }

    // ── E-5 tests: Gabriel encounter ──

    use crate::gabriel::GabrielState;
    use crate::spatial::GameMode;

    #[test]
    fn gabriel_triggers_on_tactical_entry() {
        let mut app = test_app();
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        app.world_mut()
            .spawn((Player, Position { x: 5, y: 5 }, Pools::new(vec![])));
        // Modify existing GameMode resource rather than re-inserting
        *app.world_mut().resource_mut::<GameMode>() = GameMode::Tactical;

        // Verify state before update
        assert!(
            !app.world().resource::<GabrielState>().appeared,
            "Gabriel should not have appeared yet"
        );
        assert_eq!(*app.world().resource::<GameMode>(), GameMode::Tactical);

        app.update();
        // After first update, trigger_gabriel_encounter should have fired
        assert!(
            app.world().resource::<GabrielState>().appeared,
            "Gabriel should have appeared after first update"
        );

        app.update(); // second frame: process_event_triggers reads the message

        // Gabriel event should have been triggered
        let ev = app.world().resource::<CurrentEvent>();
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
        let ev = app.world().resource::<CurrentEvent>();
        assert!(!ev.is_active());
    }
}
