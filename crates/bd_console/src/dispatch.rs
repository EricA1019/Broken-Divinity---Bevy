//! Dispatch system — executes console commands from pending queue.
use bevy_ecs::prelude::*;
use crate::commands::{DebugCommand, parse};
use crate::state::ConsoleState;

pub fn execute_console_command(world: &mut World) {
    let pending = std::mem::take(&mut world.resource_mut::<ConsoleState>().pending);
    for raw in &pending {
        let cmd = parse(raw);
        if let Some(msg) = dispatch_one(world, cmd) {
            if !msg.is_empty() { world.resource_mut::<ConsoleState>().output.push(msg); }
        }
    }
}

fn dispatch_one(world: &mut World, cmd: DebugCommand) -> Option<String> {
    Some(match cmd {
        DebugCommand::AddResource(k, a) => add_resource(world, k, a),
        DebugCommand::SetDay(n) => set_day(world, n),
        DebugCommand::SetTurn(n) => set_turn(world, n),
        DebugCommand::SkipDay => skip_day(world),
        DebugCommand::TriggerEvent(id) => trigger_event(world, &id),
        DebugCommand::EndEvent => end_event(world),
        DebugCommand::KillAllEnemies => kill_all(world),
        DebugCommand::Heal => heal(world),
        DebugCommand::GodMode(on) => god_mode(world, on),
        DebugCommand::SpawnSurvivor(n) => spawn_survivor(world, &n),
        DebugCommand::AssignTask(i, t) => assign_task(world, i, &t),
        DebugCommand::SpawnEntity(b, x, y) => spawn_entity(world, &b, x, y),
        DebugCommand::Teleport(x, y) => teleport(world, x, y),
        DebugCommand::GotoShelter => goto_shelter(world),
        DebugCommand::ListBlueprints => list_blueprints(world),
        DebugCommand::ListEvents => list_events(world),
        DebugCommand::Stats => stats(world),
        DebugCommand::Help => help_text(),
        DebugCommand::Clear => { clear_output(world); return None; }
        DebugCommand::Unknown(m) => format!("ERROR: {}", m),
    })
}

fn find_player(world: &mut World) -> Option<Entity> {
    let all: Vec<Entity> = world.query::<Entity>().iter(world).collect();
    all.into_iter().find(|&e| world.get::<bd_core::components::Player>(e).is_some())
}

fn add_resource(world: &mut World, kind: bd_core::signals::PoolKind, amount: i32) -> String {
    match find_player(world) {
        Some(p) => {
            world.resource_mut::<bevy_ecs::message::Messages<bd_core::signals::PoolDeltaRequested>>()
                .write(bd_core::signals::PoolDeltaRequested { source: None, target: p, kind, amount, tags: vec![], reason: format!("console: add {:?} {}", kind, amount) });
            format!("OK: {:?} {}", kind, amount)
        }
        None => "ERROR: no player entity".into(),
    }
}

fn set_day(world: &mut World, n: u64) -> String { world.resource_mut::<bd_core::time::GameTime>().day = n; format!("OK: day {}", n) }
fn set_turn(world: &mut World, n: u64) -> String { world.resource_mut::<bd_core::time::GameTime>().turn = n; format!("OK: turn {}", n) }
fn skip_day(world: &mut World) -> String { let mut t = world.resource_mut::<bd_core::time::GameTime>(); t.day += 1; format!("OK: day {}", t.day) }

fn trigger_event(world: &mut World, id: &str) -> String {
    if world.resource::<bd_core::events::EventRegistry>().get(id).is_none() { return format!("ERROR: '{}' not registered", id); }
    match find_player(world) {
        Some(p) => { world.resource_mut::<bevy_ecs::message::Messages<bd_core::signals::EventTrigger>>().write(bd_core::signals::EventTrigger { actor: p, event_id: id.into() }); format!("OK: triggered '{}'", id) }
        None => "ERROR: no player".into(),
    }
}

fn end_event(world: &mut World) -> String {
    let mut ev = world.resource_mut::<bd_core::events::CurrentEvent>();
    if ev.active { ev.active = false; "OK: ended".into() } else { "ERROR: no active event".into() }
}

fn kill_all(world: &mut World) -> String {
    let all: Vec<Entity> = world.query::<Entity>().iter(world).collect();
    let pids: Vec<Entity> = all.iter().filter(|&&e| world.get::<bd_core::components::Player>(e).is_some()).copied().collect();
    let sids: Vec<Entity> = all.iter().filter(|&&e| world.get::<bd_core::colony::survivors::Survivor>(e).is_some()).copied().collect();
    let enemies: Vec<Entity> = all.into_iter().filter(|e| !pids.contains(e) && !sids.contains(e) && world.get::<bd_core::pools::Pools>(*e).is_some()).collect();
    if enemies.is_empty() { return "ERROR: no enemies".into(); }
    let n = enemies.len();
    let mut m = world.resource_mut::<bevy_ecs::message::Messages<bd_core::signals::EntityDefeated>>();
    for &e in &enemies { m.write(bd_core::signals::EntityDefeated { entity: e, kind: bd_core::signals::PoolKind::Health }); }
    format!("OK: {} enemies defeated", n)
}

fn heal(world: &mut World) -> String {
    let p = match find_player(world) { Some(p) => p, None => return "ERROR: no player".into() };
    let kinds: Vec<bd_core::signals::PoolKind> = match world.get::<bd_core::pools::Pools>(p) { Some(pl) => pl.iter().map(|x| x.kind).collect(), None => return "ERROR: no pools".into() };
    let mut h = 0i32;
    let mut pools = world.get_mut::<bd_core::pools::Pools>(p).unwrap();
    for k in kinds { if let Some(po) = pools.get_mut(k) { let miss = po.max - po.current; if miss > 0 { po.current = po.max; h += miss; } } }
    format!("OK: healed {} points", h)
}

fn god_mode(world: &mut World, on: bool) -> String {
    let p = match find_player(world) { Some(p) => p, None => return "ERROR: no player".into() };
    if on {
        if world.entity(p).contains::<bd_core::components::GodMode>() { return "ERROR: already active".into(); }
        world.entity_mut(p).insert(bd_core::components::GodMode); "OK: god mode ON".into()
    } else {
        if !world.entity(p).contains::<bd_core::components::GodMode>() { return "ERROR: not active".into(); }
        world.entity_mut(p).remove::<bd_core::components::GodMode>(); "OK: god mode OFF".into()
    }
}

fn spawn_survivor(world: &mut World, name: &str) -> String {
    world.spawn((bd_core::colony::survivors::Survivor, bd_core::components::Name(name.into()), bd_core::components::Position { x: 1, y: 1 }, bd_core::colony::survivors::default_survivor_pools(), bd_core::colony::survivors::SurvivorTask::Idle));
    format!("OK: spawned '{}'", name)
}

fn assign_task(world: &mut World, idx: usize, task: &str) -> String {
    use bd_core::colony::survivors::SurvivorTask;
    let all: Vec<Entity> = world.query::<Entity>().iter(world).collect();
    let sv: Vec<Entity> = all.into_iter().filter(|&e| world.get::<bd_core::colony::survivors::Survivor>(e).is_some()).collect();
    if idx >= sv.len() { return format!("ERROR: index {} out of {}", idx, sv.len()); }
    let e = sv[idx];
    let nt = match task { "idle" => SurvivorTask::Idle, "defending" => SurvivorTask::Defending, "resting" => SurvivorTask::Resting, o => return format!("ERROR: unknown '{}'", o) };
    match world.get_mut::<SurvivorTask>(e) { Some(mut t) => { *t = nt; format!("OK: #{} -> {}", idx, task) } None => "ERROR: no SurvivorTask".into() }
}

fn spawn_entity(world: &mut World, id: &str, x: i32, y: i32) -> String {
    let bp = match world.resource::<bd_core::factory::BlueprintCatalog>().get(id) { Some(b) => b.clone(), None => return format!("ERROR: '{}' not found", id) };
    let mut e = world.spawn((bd_core::components::Name(bp.label.clone()), bd_core::components::Position { x, y }));
    if bp.is_player { e.insert(bd_core::components::Player); }
    if bp.blocks_movement { e.insert(bd_core::components::BlocksMovement); }
    if !bp.pools.is_empty() { e.insert(bd_core::pools::Pools::new(bp.pools.iter().map(|&(k,c,mn,mx)| bd_core::pools::Pool::new(k,c,mn,mx)).collect())); }
    format!("OK: spawned '{}' at ({},{})", id, x, y)
}

fn teleport(world: &mut World, x: i32, y: i32) -> String {
    let p = match find_player(world) { Some(p) => p, None => return "ERROR: no player".into() };
    let old = world.get::<bd_core::components::Position>(p).map(|po| (po.x, po.y)).unwrap_or((0,0));
    if let Some(mut pos) = world.get_mut::<bd_core::components::Position>(p) { pos.x = x; pos.y = y; }
    format!("OK: ({},{}) -> ({},{})", old.0, old.1, x, y)
}

fn goto_shelter(world: &mut World) -> String {
    world.resource_mut::<bevy_ecs::message::Messages<bd_core::spatial::TransitionIntent>>().write(bd_core::spatial::TransitionIntent { target: bd_core::spatial::GameMode::Outpost, node_id: None });
    "OK: shelter".into()
}

fn list_blueprints(world: &mut World) -> String {
    let c = world.resource::<bd_core::factory::BlueprintCatalog>(); let ids = c.blueprint_ids();
    if ids.is_empty() { return "No blueprints.".into() }
    let mut l = vec![format!("{} blueprints:", ids.len())];
    for id in ids { if let Some(b) = c.get(id) { l.push(format!("  {} — {}", b.id, b.label)); } }
    l.join("\n")
}

fn list_events(world: &mut World) -> String {
    let r = world.resource::<bd_core::events::EventRegistry>(); let ids = r.all_ids();
    if ids.is_empty() { return "No events.".into() }
    let mut l = vec![format!("{} events:", ids.len())];
    for id in ids { l.push(format!("  {}", id)); }
    l.join("\n")
}

fn stats(world: &mut World) -> String {
    let t = world.resource::<bd_core::time::GameTime>();
    let c = world.resource::<bd_core::colony::production::ColonyResources>();
    let mut l = vec![format!("Day: {}  Turn: {}", t.day, t.turn)];
    for &k in &[bd_core::signals::PoolKind::Supplies, bd_core::signals::PoolKind::Materials, bd_core::signals::PoolKind::WildPlants, bd_core::signals::PoolKind::Faith] {
        if let Some(po) = c.pools.get(k) { l.push(format!("  {:?}: {}/{}", k, po.current, po.max)); }
    }
    l.join("\n")
}

fn help_text() -> String { "COMMANDS: s|supplies <n>  m|materials <n>  f|faith <n>  p|plants <n> | day/turn <n> | skip_day | event <id> | end_event | kill_all | heal | god on|off | survivor <name> | task <idx> idle|defending|resting | spawn <bp> <x> <y> | goto <x> <y> | shelter | blueprints | events | stats | help | clear".into() }

fn clear_output(world: &mut World) { world.resource_mut::<ConsoleState>().output.clear(); }

#[cfg(test)]
mod tests {
    use super::*;
    use bd_core::components::{Player, Position};
    use bd_core::events::{CurrentEvent, EventDefinition, EventRegistry};
    use bd_core::factory::{BlueprintCatalog, EntityBlueprint};
    use bd_core::signals::*;
    use bd_core::time::GameTime;

    fn w() -> World { let mut w = World::new(); w.init_resource::<ConsoleState>(); w.init_resource::<GameTime>(); w.init_resource::<CurrentEvent>(); w.init_resource::<EventRegistry>(); w.init_resource::<bd_core::colony::production::ColonyResources>(); w.init_resource::<BlueprintCatalog>(); w.insert_resource(bevy_ecs::message::Messages::<PoolDeltaRequested>::default()); w.insert_resource(bevy_ecs::message::Messages::<EventTrigger>::default()); w.insert_resource(bevy_ecs::message::Messages::<EntityDefeated>::default()); w.insert_resource(bevy_ecs::message::Messages::<bd_core::spatial::TransitionIntent>::default()); w }
    fn pl(world: &mut World) -> Entity { world.spawn((Player, Position { x: 5, y: 5 }, bd_core::pools::Pools::new(vec![]))).id() }
    fn c(world: &mut World, s: &str) { world.resource_mut::<ConsoleState>().pending.push(s.into()); }
    fn r(world: &mut World) { execute_console_command(world); }
    fn has(world: &World, n: &str) -> bool { world.resource::<ConsoleState>().output.iter().any(|l| l.contains(n)) }

    #[test] fn supplies_ok() { let mut w = w(); pl(&mut w); c(&mut w, "supplies 50"); r(&mut w); assert!(has(&w, "Supplies 50")); }
    #[test] fn supplies_no_player() { let mut w = w(); c(&mut w, "supplies 50"); r(&mut w); assert!(has(&w, "ERROR")); }
    #[test] fn day_10() { let mut w = w(); c(&mut w, "day 10"); r(&mut w); assert_eq!(w.resource::<GameTime>().day, 10); }
    #[test] fn turn_42() { let mut w = w(); c(&mut w, "turn 42"); r(&mut w); assert_eq!(w.resource::<GameTime>().turn, 42); }
    #[test] fn skip_day_ok() { let mut w = w(); w.resource_mut::<GameTime>().day = 5; c(&mut w, "skip_day"); r(&mut w); assert_eq!(w.resource::<GameTime>().day, 6); }
    #[test] fn trigger_ok() { let mut w = w(); pl(&mut w); w.resource_mut::<EventRegistry>().register(EventDefinition { id: "t.e".into(), start_node: "s".into(), nodes: std::collections::HashMap::new(), spawn_on_enter: vec![] }); c(&mut w, "event t.e"); r(&mut w); assert!(w.resource::<bevy_ecs::message::Messages<EventTrigger>>().len() >= 1); }
    #[test] fn trigger_miss() { let mut w = w(); pl(&mut w); c(&mut w, "event x"); r(&mut w); assert!(has(&w, "ERROR")); }
    #[test] fn end_ev() { let mut w = w(); w.resource_mut::<CurrentEvent>().active = true; c(&mut w, "end_event"); r(&mut w); assert!(!w.resource::<CurrentEvent>().active); }
    #[test] fn end_ev_none() { let mut w = w(); c(&mut w, "end_event"); r(&mut w); assert!(has(&w, "no active")); }
    #[test] fn kill() { let mut w = w(); pl(&mut w); w.spawn((bd_core::components::Name("R".into()), Position { x: 3, y: 3 }, bd_core::pools::Pools::new(vec![]))); c(&mut w, "kill_all"); r(&mut w); assert!(w.resource::<bevy_ecs::message::Messages<EntityDefeated>>().len() >= 1); }
    #[test] fn kill_skips() { let mut w = w(); pl(&mut w); w.spawn((bd_core::colony::survivors::Survivor, Position { x: 1, y: 1 }, bd_core::pools::Pools::new(vec![]))); w.spawn((bd_core::components::Name("S".into()), Position { x: 5, y: 5 }, bd_core::pools::Pools::new(vec![]))); c(&mut w, "kill_all"); r(&mut w); assert_eq!(w.resource::<bevy_ecs::message::Messages<EntityDefeated>>().len(), 1); }
    #[test] fn heal_ok() { let mut w = w(); let p = w.spawn((Player, Position { x: 5, y: 5 }, bd_core::pools::Pools::new(vec![bd_core::pools::Pool::new(PoolKind::Health, 30, 0, 30)]))).id(); w.get_mut::<bd_core::pools::Pools>(p).unwrap().get_mut(PoolKind::Health).unwrap().current = 5; c(&mut w, "heal"); r(&mut w); assert_eq!(w.get::<bd_core::pools::Pools>(p).unwrap().get(PoolKind::Health).unwrap().current, 30); }
    #[test] fn surv_ok() { let mut w = w(); c(&mut w, "survivor Bob"); r(&mut w); let all: Vec<Entity> = w.query::<Entity>().iter(&w).collect(); assert!(all.iter().any(|&e| w.get::<bd_core::colony::survivors::Survivor>(e).is_some())); }
    #[test] fn task_ok() { let mut w = w(); w.spawn((bd_core::colony::survivors::Survivor, bd_core::colony::survivors::SurvivorTask::Idle)); c(&mut w, "task 0 defending"); r(&mut w); let all: Vec<Entity> = w.query::<Entity>().iter(&w).collect(); let e = all.into_iter().find(|&x| w.get::<bd_core::colony::survivors::Survivor>(x).is_some()).unwrap(); assert_eq!(*w.get::<bd_core::colony::survivors::SurvivorTask>(e).unwrap(), bd_core::colony::survivors::SurvivorTask::Defending); }
    #[test] fn task_oob() { let mut w = w(); c(&mut w, "task 0 idle"); r(&mut w); assert!(has(&w, "ERROR")); }
    #[test] fn spawn_ok() { let mut w = w(); w.insert_resource(BlueprintCatalog::new(vec![EntityBlueprint { id: "bp.r".into(), label: "R".into(), is_player: false, blocks_movement: true, pools: vec![(PoolKind::Health, 11, 0, 11)], statuses: vec![], visual: None, markers: vec![] }])); c(&mut w, "spawn bp.r 8 4"); r(&mut w); let all: Vec<Entity> = w.query::<Entity>().iter(&w).collect(); assert!(all.iter().any(|&e| w.get::<Position>(e).map_or(false, |p| p.x == 8 && p.y == 4))); }
    #[test] fn spawn_miss() { let mut w = w(); c(&mut w, "spawn x 0 0"); r(&mut w); assert!(has(&w, "ERROR")); }
    #[test] fn tp_ok() { let mut w = w(); let p = pl(&mut w); c(&mut w, "goto 10 20"); r(&mut w); assert_eq!((w.get::<Position>(p).unwrap().x, w.get::<Position>(p).unwrap().y), (10, 20)); }
    #[test] fn shelter() { let mut w = w(); c(&mut w, "shelter"); r(&mut w); assert!(w.resource::<bevy_ecs::message::Messages<bd_core::spatial::TransitionIntent>>().len() >= 1); }
    #[test] fn help_ok() { let mut w = w(); c(&mut w, "help"); r(&mut w); assert!(has(&w, "COMMANDS")); }
    #[test] fn clear_ok() { let mut w = w(); w.resource_mut::<ConsoleState>().output = vec!["a".into()]; c(&mut w, "clear"); r(&mut w); assert!(w.resource::<ConsoleState>().output.is_empty()); }
    #[test] fn unknown() { let mut w = w(); c(&mut w, "xyz"); r(&mut w); assert!(has(&w, "unknown command")); }
}
