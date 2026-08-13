//! Dispatch system — parses console commands and routes them.

use crate::commands::{DebugCommand, parse};
use crate::state::ConsoleState;
use bd_core::debug::{DebugMutation, DebugMutationRequest, DebugSurvivorTask};
use bevy_ecs::prelude::*;

/// Reads [`ConsoleState::pending`], parses each line, and routes it. All
/// mutating commands cross the typed core boundary; read-only commands retain
/// console-local query ownership. Runs as an exclusive system in
/// `BdSet::Mutation`, before the named core resolver.
pub fn execute_console_command(world: &mut World) {
    let pending = std::mem::take(&mut world.resource_mut::<ConsoleState>().pending);
    for raw in &pending {
        let cmd = parse(raw);
        if let Some(msg) = dispatch_one(world, cmd) {
            if !msg.is_empty() {
                world.resource_mut::<ConsoleState>().output.push(msg);
            }
        }
    }
}

fn dispatch_one(world: &mut World, cmd: DebugCommand) -> Option<String> {
    match cmd {
        // ── C3 ordinary mutations: emit one typed core request ──
        DebugCommand::AddResource(kind, amount) => {
            emit_request(world, DebugMutation::AddColonyResource { kind, amount });
            None
        }
        DebugCommand::SetDay(n) => {
            emit_request(world, DebugMutation::SetDay(n));
            None
        }
        DebugCommand::SetTurn(n) => {
            emit_request(world, DebugMutation::SetTurn(n));
            None
        }
        DebugCommand::SkipDay => {
            emit_request(world, DebugMutation::SkipDay);
            None
        }
        DebugCommand::TriggerEvent(id) => {
            emit_request(world, DebugMutation::TriggerEvent(id));
            None
        }
        DebugCommand::EndEvent => {
            emit_request(world, DebugMutation::EndEvent);
            None
        }
        DebugCommand::SpawnSurvivor(name) => {
            emit_request(world, DebugMutation::SpawnSurvivor(name));
            None
        }
        DebugCommand::AssignTask(index, task) => match parse_debug_task(&task) {
            Some(task) => {
                emit_request(world, DebugMutation::AssignSurvivorTask { index, task });
                None
            }
            None => Some(format!("ERROR: unknown task '{task}'")),
        },
        DebugCommand::Teleport(x, y) => {
            emit_request(
                world,
                DebugMutation::TeleportPlayer(bd_core::components::Position { x, y }),
            );
            None
        }
        DebugCommand::GotoShelter => {
            emit_request(world, DebugMutation::TransitionToShelter);
            None
        }
        // ── C4 commands: emit one typed core request ──
        DebugCommand::KillAllEnemies => {
            emit_request(world, DebugMutation::KillAllEnemies);
            None
        }
        DebugCommand::Heal => {
            emit_request(world, DebugMutation::HealPlayer);
            None
        }
        DebugCommand::GodMode(on) => {
            emit_request(world, DebugMutation::SetGodMode(on));
            None
        }
        DebugCommand::SpawnEntity(id, x, y) => {
            emit_request(
                world,
                DebugMutation::SpawnBlueprint {
                    blueprint_id: id,
                    position: bd_core::components::Position { x, y },
                },
            );
            None
        }
        // ── Read-only and console-local commands ──
        DebugCommand::ListBlueprints => Some(list_blueprints(world)),
        DebugCommand::ListEvents => Some(list_events(world)),
        DebugCommand::Stats => Some(stats(world)),
        DebugCommand::Help => Some(help_text()),
        DebugCommand::Clear => {
            clear_output(world);
            None
        }
        DebugCommand::Unknown(message) => Some(format!("ERROR: {message}")),
    }
}

fn emit_request(world: &mut World, mutation: DebugMutation) {
    world
        .resource_mut::<bevy_ecs::message::Messages<DebugMutationRequest>>()
        .write(DebugMutationRequest(mutation));
}

fn parse_debug_task(task: &str) -> Option<DebugSurvivorTask> {
    match task {
        "idle" => Some(DebugSurvivorTask::Idle),
        "defending" => Some(DebugSurvivorTask::Defending),
        "resting" => Some(DebugSurvivorTask::Resting),
        _ => None,
    }
}

// ── Read-only helpers ──

fn list_blueprints(world: &mut World) -> String {
    let catalog = world.resource::<bd_core::factory::BlueprintCatalog>();
    let ids = catalog.blueprint_ids();
    if ids.is_empty() {
        return "No blueprints.".into();
    }
    let mut lines = vec![format!("{} blueprints:", ids.len())];
    for id in ids {
        if let Some(blueprint) = catalog.get(id) {
            lines.push(format!("  {} — {}", blueprint.id, blueprint.label));
        }
    }
    lines.join("\n")
}

fn list_events(world: &mut World) -> String {
    let registry = world.resource::<bd_core::events::EventRegistry>();
    let ids = registry.all_ids();
    if ids.is_empty() {
        return "No events.".into();
    }
    let mut lines = vec![format!("{} events:", ids.len())];
    for id in ids {
        lines.push(format!("  {id}"));
    }
    lines.join("\n")
}

fn stats(world: &mut World) -> String {
    let (day, turn) = {
        let time = world.resource::<bd_core::time::GameTime>();
        (time.day, time.turn)
    };
    let pool_lines = {
        let resources = world.resource::<bd_core::colony::production::ColonyResources>();
        [
            bd_core::signals::PoolKind::Supplies,
            bd_core::signals::PoolKind::Materials,
            bd_core::signals::PoolKind::WildPlants,
            bd_core::signals::PoolKind::Faith,
        ]
        .into_iter()
        .filter_map(|kind| {
            resources
                .pools
                .get(kind)
                .map(|pool| format!("  {kind:?}: {}/{}", pool.current, pool.max))
        })
        .collect::<Vec<_>>()
    };
    let mut lines = vec![format!("Day: {day}  Turn: {turn}")];
    lines.extend(pool_lines);
    match bd_core::debug::project_survivors(world) {
        bd_core::debug::SurvivorProjection::Targets(targets) => {
            if !targets.is_empty() {
                lines.push("Survivors:".into());
                for (index, target) in targets.iter().enumerate() {
                    lines.push(format!(
                        "  #{index} {} ({},{})",
                        target.name, target.position.x, target.position.y
                    ));
                }
            }
        }
        bd_core::debug::SurvivorProjection::Ambiguous { name, position } => {
            lines.push(format!(
                "  ambiguous: {name} ({},{})",
                position.x, position.y
            ));
        }
    }
    lines.join("\n")
}

fn help_text() -> String {
    "COMMANDS: s|supplies <n>  m|materials <n>  f|faith <n>  p|plants <n> | day/turn <n> | skip_day | event <id> | end_event | kill_all | heal | god on|off | survivor <name> | task <idx> idle|defending|resting | spawn <bp> <x> <y> | goto <x> <y> | shelter | blueprints | events | stats | help | clear".into()
}

fn clear_output(world: &mut World) {
    world.resource_mut::<ConsoleState>().output.clear();
}

#[cfg(test)]
mod tests {
    use super::*;
    use bd_core::components::{Player, Position};
    use bd_core::debug::{DebugMutation, DebugMutationRequest, DebugSurvivorTask};
    use bd_core::events::{CurrentEvent, EventDefinition, EventRegistry};
    use bd_core::factory::{BlueprintCatalog, EntityBlueprint};
    use bd_core::signals::*;
    use bd_core::time::GameTime;

    fn w() -> World {
        let mut w = World::new();
        w.init_resource::<ConsoleState>();
        w.init_resource::<GameTime>();
        w.init_resource::<CurrentEvent>();
        w.init_resource::<EventRegistry>();
        w.init_resource::<bd_core::colony::production::ColonyResources>();
        w.init_resource::<BlueprintCatalog>();
        w.insert_resource(bevy_ecs::message::Messages::<PoolDeltaRequested>::default());
        w.insert_resource(bevy_ecs::message::Messages::<EventTrigger>::default());
        w.insert_resource(bevy_ecs::message::Messages::<EntityDefeated>::default());
        w.insert_resource(bevy_ecs::message::Messages::<
            bd_core::spatial::TransitionIntent,
        >::default());
        w.insert_resource(bevy_ecs::message::Messages::<DebugMutationRequest>::default());
        w
    }
    fn pl(world: &mut World) -> Entity {
        world
            .spawn((
                Player,
                Position { x: 5, y: 5 },
                bd_core::pools::Pools::new(vec![]),
            ))
            .id()
    }
    fn c(world: &mut World, s: &str) {
        world.resource_mut::<ConsoleState>().pending.push(s.into());
    }
    fn r(world: &mut World) {
        execute_console_command(world);
    }
    fn has(world: &World, n: &str) -> bool {
        world
            .resource::<ConsoleState>()
            .output
            .iter()
            .any(|l| l.contains(n))
    }
    fn emitted(world: &mut World) -> Vec<DebugMutation> {
        world
            .resource_mut::<bevy_ecs::message::Messages<DebugMutationRequest>>()
            .drain()
            .map(|request| request.0)
            .collect()
    }

    // C3: dispatch crosses the typed boundary without mutating gameplay.
    #[test]
    fn supplies_ok() {
        let mut w = w();
        c(&mut w, "supplies 50");
        r(&mut w);
        assert_eq!(
            emitted(&mut w),
            vec![DebugMutation::AddColonyResource {
                kind: PoolKind::Supplies,
                amount: 50
            }]
        );
        assert_eq!(
            w.resource::<bd_core::colony::production::ColonyResources>()
                .pools
                .get(PoolKind::Supplies)
                .unwrap()
                .current,
            10
        );
    }
    #[test]
    fn supplies_works_without_player() {
        let mut w = w();
        c(&mut w, "supplies 50");
        r(&mut w);
        assert_eq!(emitted(&mut w).len(), 1);
    } // Colony resources don't need player entity
    #[test]
    fn day_10() {
        let mut w = w();
        c(&mut w, "day 10");
        r(&mut w);
        assert_eq!(emitted(&mut w), vec![DebugMutation::SetDay(10)]);
        assert_eq!(w.resource::<GameTime>().day, 0);
    }
    #[test]
    fn turn_42() {
        let mut w = w();
        c(&mut w, "turn 42");
        r(&mut w);
        assert_eq!(emitted(&mut w), vec![DebugMutation::SetTurn(42)]);
        assert_eq!(w.resource::<GameTime>().turn, 0);
    }
    #[test]
    fn skip_day_ok() {
        let mut w = w();
        w.resource_mut::<GameTime>().day = 5;
        c(&mut w, "skip_day");
        r(&mut w);
        assert_eq!(emitted(&mut w), vec![DebugMutation::SkipDay]);
        assert_eq!(w.resource::<GameTime>().day, 5);
    }
    #[test]
    fn trigger_ok() {
        let mut w = w();
        pl(&mut w);
        w.resource_mut::<EventRegistry>().register(EventDefinition {
            id: "t.e".into(),
            start_node: "s".into(),
            nodes: std::collections::HashMap::new(),
            spawn_on_enter: vec![],
        });
        c(&mut w, "event t.e");
        r(&mut w);
        assert_eq!(
            emitted(&mut w),
            vec![DebugMutation::TriggerEvent("t.e".into())]
        );
        assert!(
            w.resource::<bevy_ecs::message::Messages<EventTrigger>>()
                .is_empty()
        );
    }
    #[test]
    fn trigger_miss() {
        let mut w = w();
        pl(&mut w);
        c(&mut w, "event x");
        r(&mut w);
        assert_eq!(
            emitted(&mut w),
            vec![DebugMutation::TriggerEvent("x".into())]
        );
    }
    #[test]
    fn end_ev() {
        let mut w = w();
        w.resource_mut::<CurrentEvent>().active = true;
        c(&mut w, "end_event");
        r(&mut w);
        assert_eq!(emitted(&mut w), vec![DebugMutation::EndEvent]);
        assert!(w.resource::<CurrentEvent>().active);
    }
    #[test]
    fn end_ev_none() {
        let mut w = w();
        c(&mut w, "end_event");
        r(&mut w);
        assert_eq!(emitted(&mut w), vec![DebugMutation::EndEvent]);
    }
    #[test]
    fn kill() {
        let mut w = w();
        pl(&mut w);
        w.spawn((
            bd_core::components::Name("R".into()),
            Position { x: 3, y: 3 },
            bd_core::pools::Pools::new(vec![]),
        ));
        c(&mut w, "kill_all");
        r(&mut w);
        assert_eq!(emitted(&mut w), vec![DebugMutation::KillAllEnemies]);
        assert!(
            w.resource::<bevy_ecs::message::Messages<EntityDefeated>>()
                .is_empty(),
            "dispatch must not emit defeat before the gated core resolver"
        );
    }
    #[test]
    fn kill_skips() {
        let mut w = w();
        pl(&mut w);
        w.spawn((
            bd_core::colony::survivors::Survivor,
            Position { x: 1, y: 1 },
            bd_core::pools::Pools::new(vec![]),
        ));
        w.spawn((
            bd_core::components::Name("S".into()),
            Position { x: 5, y: 5 },
            bd_core::pools::Pools::new(vec![]),
        ));
        c(&mut w, "kill_all");
        r(&mut w);
        assert_eq!(emitted(&mut w), vec![DebugMutation::KillAllEnemies]);
        assert_eq!(
            w.resource::<bevy_ecs::message::Messages<EntityDefeated>>()
                .len(),
            0,
            "dispatch must not inspect or mutate candidate targets"
        );
    }
    #[test]
    fn heal_ok() {
        let mut w = w();
        let p = w
            .spawn((
                Player,
                Position { x: 5, y: 5 },
                bd_core::pools::Pools::new(vec![bd_core::pools::Pool::new(
                    PoolKind::Health,
                    30,
                    0,
                    30,
                )]),
            ))
            .id();
        w.get_mut::<bd_core::pools::Pools>(p)
            .unwrap()
            .get_mut(PoolKind::Health)
            .unwrap()
            .current = 5;
        c(&mut w, "heal");
        r(&mut w);
        assert_eq!(emitted(&mut w), vec![DebugMutation::HealPlayer]);
        assert_eq!(
            w.get::<bd_core::pools::Pools>(p)
                .unwrap()
                .get(PoolKind::Health)
                .unwrap()
                .current,
            5,
            "dispatch must leave pool mutation to the canonical resolver"
        );
        assert!(
            w.resource::<bevy_ecs::message::Messages<PoolDeltaRequested>>()
                .is_empty(),
            "dispatch must not emit pool effects before gate resolution"
        );
    }
    #[test]
    fn surv_ok() {
        let mut w = w();
        c(&mut w, "survivor Bob");
        r(&mut w);
        assert_eq!(
            emitted(&mut w),
            vec![DebugMutation::SpawnSurvivor("Bob".into())]
        );
        let all: Vec<Entity> = w.query::<Entity>().iter(&w).collect();
        assert!(
            !all.iter()
                .any(|&e| w.get::<bd_core::colony::survivors::Survivor>(e).is_some())
        );
    }
    #[test]
    fn task_ok() {
        let mut w = w();
        w.spawn((
            bd_core::colony::survivors::Survivor,
            bd_core::colony::survivors::SurvivorTask::Idle,
        ));
        c(&mut w, "task 0 defending");
        r(&mut w);
        assert_eq!(
            emitted(&mut w),
            vec![DebugMutation::AssignSurvivorTask {
                index: 0,
                task: DebugSurvivorTask::Defending
            }]
        );
        let all: Vec<Entity> = w.query::<Entity>().iter(&w).collect();
        let e = all
            .into_iter()
            .find(|&x| w.get::<bd_core::colony::survivors::Survivor>(x).is_some())
            .unwrap();
        assert_eq!(
            *w.get::<bd_core::colony::survivors::SurvivorTask>(e)
                .unwrap(),
            bd_core::colony::survivors::SurvivorTask::Idle
        );
    }
    #[test]
    fn task_oob() {
        let mut w = w();
        c(&mut w, "task 0 idle");
        r(&mut w);
        assert_eq!(
            emitted(&mut w),
            vec![DebugMutation::AssignSurvivorTask {
                index: 0,
                task: DebugSurvivorTask::Idle
            }]
        );
    }
    #[test]
    fn spawn_ok() {
        let mut w = w();
        w.insert_resource(BlueprintCatalog::new(vec![EntityBlueprint {
            id: "bp.r".into(),
            label: "R".into(),
            is_player: false,
            blocks_movement: true,
            pools: vec![(PoolKind::Health, 11, 0, 11)],
            statuses: vec![],
            visual: None,
            markers: vec![],
        }]));
        c(&mut w, "spawn bp.r 8 4");
        r(&mut w);
        assert_eq!(
            emitted(&mut w),
            vec![DebugMutation::SpawnBlueprint {
                blueprint_id: "bp.r".into(),
                position: Position { x: 8, y: 4 }
            }]
        );
        let all: Vec<Entity> = w.query::<Entity>().iter(&w).collect();
        assert!(
            !all.iter()
                .any(|&e| w.get::<Position>(e).is_some_and(|p| p.x == 8 && p.y == 4)),
            "dispatch must not copy the canonical factory"
        );
    }
    #[test]
    fn spawn_miss() {
        let mut w = w();
        c(&mut w, "spawn x 0 0");
        r(&mut w);
        assert_eq!(
            emitted(&mut w),
            vec![DebugMutation::SpawnBlueprint {
                blueprint_id: "x".into(),
                position: Position { x: 0, y: 0 }
            }]
        );
        assert!(
            !has(&w, "ERROR"),
            "catalog validation and readable rejection belong to the resolver"
        );
    }
    #[test]
    fn tp_ok() {
        let mut w = w();
        let p = pl(&mut w);
        c(&mut w, "goto 10 20");
        r(&mut w);
        assert_eq!(
            emitted(&mut w),
            vec![DebugMutation::TeleportPlayer(Position { x: 10, y: 20 })]
        );
        assert_eq!(
            (
                w.get::<Position>(p).unwrap().x,
                w.get::<Position>(p).unwrap().y
            ),
            (5, 5)
        );
    }
    #[test]
    fn shelter() {
        let mut w = w();
        c(&mut w, "shelter");
        r(&mut w);
        assert_eq!(emitted(&mut w), vec![DebugMutation::TransitionToShelter]);
        assert!(
            w.resource::<bevy_ecs::message::Messages<bd_core::spatial::TransitionIntent>>()
                .is_empty()
        );
    }
    #[test]
    fn help_ok() {
        let mut w = w();
        c(&mut w, "help");
        r(&mut w);
        assert!(has(&w, "COMMANDS"));
    }
    #[test]
    fn clear_ok() {
        let mut w = w();
        w.resource_mut::<ConsoleState>().output = vec!["a".into()];
        c(&mut w, "clear");
        r(&mut w);
        assert!(w.resource::<ConsoleState>().output.is_empty());
    }
    #[test]
    fn unknown() {
        let mut w = w();
        c(&mut w, "xyz");
        r(&mut w);
        assert!(has(&w, "unknown command"));
    }
}
