//! Gated developer-only mutation boundary.
//!
//! The public request/result vocabulary lets development tooling ask the core
//! for mutations without becoming a second gameplay owner. The core resolver
//! applies enabled requests and rejects disabled or invalid requests atomically.

use bevy_app::App;
use bevy_ecs::message::{MessageCursor, Messages};
use bevy_ecs::prelude::*;

use crate::{BdSet, components::Position, signals::PoolKind, trace::SignalTrace};

/// Explicit authority required before the core may apply debug mutations.
///
/// Core runtimes start disabled. A development-tool plugin must opt in.
#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DebugMutationGate {
    pub enabled: bool,
}

impl DebugMutationGate {
    pub const fn enabled() -> Self {
        Self { enabled: true }
    }
}

/// Typed survivor tasks exposed by the developer console.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugSurvivorTask {
    Idle,
    Defending,
    Resting,
}

/// Mutations accepted by the core debug boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DebugMutation {
    AddColonyResource {
        kind: PoolKind,
        amount: i32,
    },
    SetDay(u64),
    SetTurn(u64),
    SkipDay,
    TriggerEvent(String),
    EndEvent,
    SpawnSurvivor(String),
    AssignSurvivorTask {
        index: usize,
        task: DebugSurvivorTask,
    },
    TeleportPlayer(Position),
    TransitionToShelter,
    /// Defeat every non-player, non-survivor pooled entity.
    KillAllEnemies,
    /// Restore every player pool through canonical pool deltas.
    HealPlayer,
    /// Enable or disable the player's GodMode marker.
    SetGodMode(bool),
    /// Spawn through the canonical blueprint factory.
    SpawnBlueprint {
        blueprint_id: String,
        position: Position,
    },
}

/// One typed request emitted by development tooling.
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct DebugMutationRequest(pub DebugMutation);

/// One ordered result returned to the requesting development tool.
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct DebugMutationResult {
    pub accepted: bool,
    pub message: String,
}

/// Named core-owned stage for resolving debug mutations.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DebugMutationSet {
    Resolve,
}

/// One visible survivor target in the shared stable projection.
#[derive(Debug, Clone)]
pub struct SurvivorTarget {
    pub entity: Entity,
    pub name: String,
    pub position: Position,
}

/// Deterministic survivor projection shared by read-only listing and mutation
/// selection. Ordering is by visible (name, y, x); raw ECS iteration order and
/// raw `Entity` values are never used as selection identity.
#[derive(Debug, Clone)]
pub enum SurvivorProjection {
    Targets(Vec<SurvivorTarget>),
    Ambiguous { name: String, position: Position },
}

/// Derive the single shared survivor target projection.
pub fn project_survivors(world: &mut World) -> SurvivorProjection {
    let mut survivors = {
        let mut query = world.query_filtered::<(Entity, &crate::components::Name, &Position), With<crate::colony::survivors::Survivor>>();
        query
            .iter(world)
            .map(|(entity, name, position)| (entity, name.0.clone(), *position))
            .collect::<Vec<_>>()
    };
    survivors.sort_by(|left, right| {
        (&left.1, left.2.y, left.2.x).cmp(&(&right.1, right.2.y, right.2.x))
    });
    for window in survivors.windows(2) {
        if window[0].1 == window[1].1 && window[0].2 == window[1].2 {
            return SurvivorProjection::Ambiguous {
                name: window[0].1.clone(),
                position: window[0].2,
            };
        }
    }
    SurvivorProjection::Targets(
        survivors
            .into_iter()
            .map(|(entity, name, position)| SurvivorTarget {
                entity,
                name,
                position,
            })
            .collect(),
    )
}

fn to_survivor_task(task: DebugSurvivorTask) -> crate::colony::survivors::SurvivorTask {
    match task {
        DebugSurvivorTask::Idle => crate::colony::survivors::SurvivorTask::Idle,
        DebugSurvivorTask::Defending => crate::colony::survivors::SurvivorTask::Defending,
        DebugSurvivorTask::Resting => crate::colony::survivors::SurvivorTask::Resting,
    }
}

fn find_player(world: &mut World) -> Option<Entity> {
    let all: Vec<Entity> = world.query::<Entity>().iter(world).collect();
    all.into_iter()
        .find(|&entity| world.get::<crate::components::Player>(entity).is_some())
}

/// The single real core-owned resolver. Gate off denies; gate on applies or
/// rejects atomically. Exactly one `DebugMutationResult` and one trace entry
/// are produced per request.
fn resolve_debug_mutations(
    world: &mut World,
    mut request_cursor: Local<MessageCursor<DebugMutationRequest>>,
) {
    let gate = *world.resource::<DebugMutationGate>();
    let requests = {
        let messages = world.resource::<Messages<DebugMutationRequest>>();
        request_cursor.read(messages).cloned().collect::<Vec<_>>()
    };
    for request in requests {
        let (accepted, message, summary) = if !gate.enabled {
            (
                false,
                "ERROR: debug mutation disabled".to_string(),
                "denied",
            )
        } else {
            let (accepted, message) = apply_debug_mutation(world, &request.0);
            let summary = if accepted { "accepted" } else { "rejected" };
            (accepted, message, summary)
        };
        world
            .resource_mut::<Messages<DebugMutationResult>>()
            .write(DebugMutationResult {
                accepted,
                message: message.clone(),
            });
        world.resource_mut::<SignalTrace>().push(
            "DebugMutation",
            "DebugMutationResult",
            format!("{summary} {:?}: {message}", request.0),
        );
    }
}

fn apply_debug_mutation(world: &mut World, mutation: &DebugMutation) -> (bool, String) {
    match mutation {
        DebugMutation::AddColonyResource { kind, amount } => {
            let mut resources = world.resource_mut::<crate::colony::production::ColonyResources>();
            match resources.pools.get_mut(*kind) {
                Some(pool) => {
                    let applied = pool.apply_delta(*amount);
                    (
                        true,
                        format!("OK: {:?} +{} ({applied} applied)", kind, amount),
                    )
                }
                None => (false, format!("ERROR: unknown resource {:?}", kind)),
            }
        }
        DebugMutation::SetDay(day) => {
            world.resource_mut::<crate::time::GameTime>().day = *day;
            (true, format!("OK: day set to {day}"))
        }
        DebugMutation::SetTurn(turn) => {
            world.resource_mut::<crate::time::GameTime>().turn = *turn;
            (true, format!("OK: turn set to {turn}"))
        }
        DebugMutation::SkipDay => {
            let mut time = world.resource_mut::<crate::time::GameTime>();
            time.day += 1;
            (true, format!("OK: day {}", time.day))
        }
        DebugMutation::TriggerEvent(event_id) => {
            if world
                .resource::<crate::events::EventRegistry>()
                .get(event_id)
                .is_none()
            {
                return (false, format!("ERROR: event '{event_id}' not registered"));
            }
            let Some(player) = find_player(world) else {
                return (false, "ERROR: no player entity".to_string());
            };
            world
                .resource_mut::<Messages<crate::signals::EventTrigger>>()
                .write(crate::signals::EventTrigger {
                    actor: player,
                    event_id: event_id.clone(),
                });
            (true, format!("OK: triggered '{event_id}'"))
        }
        DebugMutation::EndEvent => {
            let mut event = world.resource_mut::<crate::events::CurrentEvent>();
            if event.active {
                event.active = false;
                (true, "OK: event ended".to_string())
            } else {
                (false, "ERROR: no active event".to_string())
            }
        }
        DebugMutation::SpawnSurvivor(name) => {
            let pools = crate::colony::survivors::default_survivor_pools();
            world.spawn((
                crate::colony::survivors::Survivor,
                crate::components::Name(name.clone()),
                Position { x: 1, y: 1 },
                pools,
                crate::colony::survivors::SurvivorTask::Idle,
                crate::spatial::EntityScope::ColonyPersistent,
                crate::spatial::PersistentEntity,
            ));
            (true, format!("OK: spawned '{name}'"))
        }
        DebugMutation::AssignSurvivorTask { index, task } => match project_survivors(world) {
            SurvivorProjection::Ambiguous { name, position } => (
                false,
                format!(
                    "ERROR: ambiguous survivor '{name}' at ({},{})",
                    position.x, position.y
                ),
            ),
            SurvivorProjection::Targets(targets) => {
                let Some(target) = targets.get(*index) else {
                    return (false, format!("ERROR: index {index} >= {}", targets.len()));
                };
                let Some(mut current) =
                    world.get_mut::<crate::colony::survivors::SurvivorTask>(target.entity)
                else {
                    return (false, "ERROR: entity has no SurvivorTask".to_string());
                };
                *current = to_survivor_task(*task);
                (
                    true,
                    format!(
                        "OK: survivor #{} {} ({},{}) -> {:?}",
                        index, target.name, target.position.x, target.position.y, task
                    ),
                )
            }
        },
        DebugMutation::TeleportPlayer(position) => {
            let Some(player) = find_player(world) else {
                return (false, "ERROR: no player".to_string());
            };
            let old = world
                .get::<Position>(player)
                .copied()
                .unwrap_or(Position { x: 0, y: 0 });
            if let Some(mut current) = world.get_mut::<Position>(player) {
                current.x = position.x;
                current.y = position.y;
            }
            (
                true,
                format!(
                    "OK: ({},{}) -> ({},{})",
                    old.x, old.y, position.x, position.y
                ),
            )
        }
        DebugMutation::TransitionToShelter => {
            world
                .resource_mut::<Messages<crate::spatial::TransitionIntent>>()
                .write(crate::spatial::TransitionIntent {
                    target: crate::spatial::GameMode::Outpost,
                    node_id: None,
                });
            (true, "OK: transitioning to shelter".to_string())
        }
        DebugMutation::KillAllEnemies => {
            let all: Vec<Entity> = world.query::<Entity>().iter(world).collect();
            let enemies: Vec<Entity> = all
                .into_iter()
                .filter(|&entity| {
                    world.get::<crate::components::Player>(entity).is_none()
                        && world
                            .get::<crate::colony::survivors::Survivor>(entity)
                            .is_none()
                        && world.get::<crate::pools::Pools>(entity).is_some()
                })
                .collect();
            if enemies.is_empty() {
                return (false, "ERROR: no enemies to kill".to_string());
            }
            let count = enemies.len();
            let mut defeated = world.resource_mut::<Messages<crate::signals::EntityDefeated>>();
            for enemy in enemies {
                defeated.write(crate::signals::EntityDefeated {
                    entity: enemy,
                    kind: PoolKind::Health,
                });
            }
            (true, format!("OK: {count} enemies defeated"))
        }
        DebugMutation::HealPlayer => {
            let Some(player) = find_player(world) else {
                return (false, "ERROR: no player".to_string());
            };
            let deltas: Vec<(PoolKind, i32)> = {
                let Some(pools) = world.get::<crate::pools::Pools>(player) else {
                    return (false, "ERROR: player has no pools".to_string());
                };
                pools
                    .iter()
                    .filter_map(|pool| {
                        let missing = pool.max - pool.current;
                        (missing > 0).then_some((pool.kind, missing))
                    })
                    .collect()
            };
            let healed: i32 = deltas.iter().map(|&(_, amount)| amount).sum();
            let mut requested =
                world.resource_mut::<Messages<crate::signals::PoolDeltaRequested>>();
            for (kind, amount) in deltas {
                requested.write(crate::signals::PoolDeltaRequested {
                    source: None,
                    target: player,
                    kind,
                    amount,
                    tags: vec![],
                    reason: format!("console heal {kind:?}"),
                });
            }
            (true, format!("OK: healed {healed} points"))
        }
        DebugMutation::SetGodMode(on) => {
            let Some(player) = find_player(world) else {
                return (false, "ERROR: no player".to_string());
            };
            if *on {
                if world
                    .entity(player)
                    .contains::<crate::components::GodMode>()
                {
                    (false, "ERROR: god mode already active".to_string())
                } else {
                    world.entity_mut(player).insert(crate::components::GodMode);
                    (true, "OK: god mode ON".to_string())
                }
            } else if world
                .entity(player)
                .contains::<crate::components::GodMode>()
            {
                world
                    .entity_mut(player)
                    .remove::<crate::components::GodMode>();
                (true, "OK: god mode OFF".to_string())
            } else {
                (false, "ERROR: god mode not active".to_string())
            }
        }
        DebugMutation::SpawnBlueprint {
            blueprint_id,
            position,
        } => {
            let Some(blueprint) = world
                .resource::<crate::factory::BlueprintCatalog>()
                .get(blueprint_id)
                .cloned()
            else {
                return (false, format!("ERROR: '{blueprint_id}' not found"));
            };
            let mode = *world.resource::<crate::spatial::GameMode>();
            crate::factory::spawn_from_blueprint_scoped(
                &blueprint,
                Some(*position),
                &[],
                mode,
                &mut world.commands(),
            );
            world.flush();
            (
                true,
                format!(
                    "OK: spawned '{blueprint_id}' at ({},{})",
                    position.x, position.y
                ),
            )
        }
    }
}

pub(crate) fn register_debug(app: &mut App) {
    app.init_resource::<DebugMutationGate>();
    app.add_message::<DebugMutationRequest>();
    app.add_message::<DebugMutationResult>();
    app.configure_sets(
        bevy_app::Update,
        DebugMutationSet::Resolve
            .before(crate::BdMutationSet::PoolDeltas)
            .in_set(BdSet::Mutation),
    );
    app.add_systems(
        bevy_app::Update,
        resolve_debug_mutations.in_set(DebugMutationSet::Resolve),
    );
}
