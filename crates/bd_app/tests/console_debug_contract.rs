//! Authoritative C3 developer-console mutation contracts.
//!
//! These tests submit the existing typed `ConsoleCommand` through the
//! registered plugins. A dispatch-boundary observer proves that parsing has
//! emitted a typed core request before any gameplay state changes.

use bd_core::{
    BdSet,
    colony::{
        production::ColonyResources,
        survivors::{Survivor, SurvivorTask},
    },
    components::{BlocksMovement, GodMode, Name, Player, Position},
    debug::{
        DebugMutation, DebugMutationGate, DebugMutationRequest, DebugMutationResult,
        DebugMutationSet, DebugSurvivorTask,
    },
    events::{CurrentEvent, EventDefinition, EventRegistry},
    factory::{BlueprintCatalog, EntityBlueprint},
    pools::{Pool, Pools},
    signals::{EntityDefeated, EventTrigger, PoolKind},
    spatial::{GameMode, TransitionIntent},
    time::GameTime,
    trace::SignalTrace,
};
use bevy_app::{App, Update};
use bevy_ecs::{
    message::{MessageCursor, Messages},
    prelude::*,
    schedule::{IntoScheduleConfigs, IntoSystemSet, NodeId, SystemSet},
    system::SystemParam,
};
use bevy_ratatui::event::KeyMessage;

#[derive(Debug, Clone, PartialEq, Eq)]
struct WorldSnapshot {
    day: u64,
    turn: u64,
    supplies: i32,
    materials: i32,
    wild_plants: i32,
    faith: i32,
    event: (bool, String, String, String),
    mode: GameMode,
    player_position: Option<Position>,
    survivors: Vec<(String, Position, String)>,
    event_trigger_count: usize,
    transition_count: usize,
}

fn pool(resources: &ColonyResources, kind: PoolKind) -> i32 {
    resources
        .pools
        .get(kind)
        .unwrap_or_else(|| panic!("fixture must initialize {kind:?}"))
        .current
}

fn snapshot_world(world: &mut World) -> WorldSnapshot {
    let time = world.resource::<GameTime>().clone();
    let resources = world.resource::<ColonyResources>();
    let supplies = pool(resources, PoolKind::Supplies);
    let materials = pool(resources, PoolKind::Materials);
    let wild_plants = pool(resources, PoolKind::WildPlants);
    let faith = pool(resources, PoolKind::Faith);
    let current = world.resource::<CurrentEvent>().clone();
    let mode = *world.resource::<GameMode>();
    let event_trigger_count = world.resource::<Messages<EventTrigger>>().len();
    let transition_count = world.resource::<Messages<TransitionIntent>>().len();
    let player_position = {
        let mut query = world.query_filtered::<&Position, With<Player>>();
        query.iter(world).next().copied()
    };
    let mut survivors = {
        let mut query = world.query_filtered::<(&Name, &Position, &SurvivorTask), With<Survivor>>();
        query
            .iter(world)
            .map(|(name, position, task)| (name.0.clone(), *position, format!("{task:?}")))
            .collect::<Vec<_>>()
    };
    survivors.sort_by(|left, right| {
        (&left.0, left.1.y, left.1.x, &left.2).cmp(&(&right.0, right.1.y, right.1.x, &right.2))
    });

    WorldSnapshot {
        day: time.day,
        turn: time.turn,
        supplies,
        materials,
        wild_plants,
        faith,
        event: (
            current.active,
            current.event_id,
            current.node_id,
            current.previous_screen,
        ),
        mode,
        player_position,
        survivors,
        event_trigger_count,
        transition_count,
    }
}

#[derive(SystemParam)]
struct BoundaryView<'w, 's> {
    time: Res<'w, GameTime>,
    resources: Res<'w, ColonyResources>,
    current: Res<'w, CurrentEvent>,
    mode: Res<'w, GameMode>,
    players: Query<'w, 's, &'static Position, With<Player>>,
    survivors:
        Query<'w, 's, (&'static Name, &'static Position, &'static SurvivorTask), With<Survivor>>,
    event_triggers: Res<'w, Messages<EventTrigger>>,
    transitions: Res<'w, Messages<TransitionIntent>>,
}

impl BoundaryView<'_, '_> {
    fn snapshot(&self) -> WorldSnapshot {
        let mut survivors = self
            .survivors
            .iter()
            .map(|(name, position, task)| (name.0.clone(), *position, format!("{task:?}")))
            .collect::<Vec<_>>();
        survivors.sort_by(|left, right| {
            (&left.0, left.1.y, left.1.x, &left.2).cmp(&(&right.0, right.1.y, right.1.x, &right.2))
        });
        WorldSnapshot {
            day: self.time.day,
            turn: self.time.turn,
            supplies: pool(&self.resources, PoolKind::Supplies),
            materials: pool(&self.resources, PoolKind::Materials),
            wild_plants: pool(&self.resources, PoolKind::WildPlants),
            faith: pool(&self.resources, PoolKind::Faith),
            event: (
                self.current.active,
                self.current.event_id.clone(),
                self.current.node_id.clone(),
                self.current.previous_screen.clone(),
            ),
            mode: *self.mode,
            player_position: self.players.iter().next().copied(),
            survivors,
            event_trigger_count: self.event_triggers.len(),
            transition_count: self.transitions.len(),
        }
    }
}

#[derive(Resource, Debug, Default)]
struct DispatchBoundaryAudit {
    observations: Vec<(DebugMutation, WorldSnapshot)>,
}

fn audit_dispatch_boundary(
    mut requests: MessageReader<DebugMutationRequest>,
    view: BoundaryView,
    mut audit: ResMut<DispatchBoundaryAudit>,
) {
    let requests = requests
        .read()
        .map(|request| request.0.clone())
        .collect::<Vec<_>>();
    if requests.is_empty() {
        return;
    }
    let snapshot = view.snapshot();
    audit.observations.extend(
        requests
            .into_iter()
            .map(|request| (request, snapshot.clone())),
    );
}

fn runtime() -> App {
    let mut app = App::new();
    app.add_plugins(bd_core::BdCorePlugin);
    app.add_message::<KeyMessage>();
    app.add_plugins(bd_console::BdConsolePlugin);
    app.init_resource::<DispatchBoundaryAudit>();
    app.add_systems(
        Update,
        audit_dispatch_boundary
            .after(bd_console::dispatch::execute_console_command)
            .before(DebugMutationSet::Resolve)
            .in_set(BdSet::Mutation),
    );
    app.world_mut().spawn((
        Player,
        Name("Witness".into()),
        Position { x: 4, y: 4 },
        bd_core::pools::Pools::new(vec![]),
    ));
    app.world_mut()
        .resource_mut::<EventRegistry>()
        .register(EventDefinition {
            id: "test.c3.event".into(),
            start_node: "start".into(),
            nodes: Default::default(),
            spawn_on_enter: vec![],
        });
    app
}

fn submit(app: &mut App, command: &str) {
    app.world_mut()
        .resource_mut::<Messages<bd_console::ConsoleCommand>>()
        .write(bd_console::ConsoleCommand(command.into()));
}

fn output_contains(app: &App, needle: &str) -> bool {
    app.world()
        .resource::<bd_console::ConsoleState>()
        .output
        .iter()
        .any(|line| line.contains(needle))
}

fn spawn_survivor(app: &mut App, name: &str, position: Position, task: SurvivorTask) {
    app.world_mut().spawn((
        Survivor,
        Name(name.into()),
        position,
        task,
        bd_core::colony::survivors::default_survivor_pools(),
    ));
}

struct MutationCase {
    name: &'static str,
    command: &'static str,
    request: DebugMutation,
    prepare: fn(&mut App),
    expected: fn(&mut WorldSnapshot),
}

fn no_prepare(_: &mut App) {}

fn prepare_active_event(app: &mut App) {
    let mut event = app.world_mut().resource_mut::<CurrentEvent>();
    event.active = true;
    event.event_id = "test.c3.event".into();
    event.node_id = "start".into();
    event.previous_screen = "outpost".into();
}

fn prepare_task_target(app: &mut App) {
    spawn_survivor(app, "Ari", Position { x: 2, y: 1 }, SurvivorTask::Idle);
}

fn expect_supplies(snapshot: &mut WorldSnapshot) {
    snapshot.supplies += 7;
}
fn expect_materials(snapshot: &mut WorldSnapshot) {
    snapshot.materials += 6;
}
fn expect_faith(snapshot: &mut WorldSnapshot) {
    snapshot.faith += 5;
}
fn expect_wild_plants(snapshot: &mut WorldSnapshot) {
    snapshot.wild_plants += 4;
}
fn expect_day(snapshot: &mut WorldSnapshot) {
    snapshot.day = 7;
}
fn expect_turn(snapshot: &mut WorldSnapshot) {
    snapshot.turn = 9;
}
fn expect_skip_day(snapshot: &mut WorldSnapshot) {
    snapshot.day += 1;
}
fn expect_event_trigger(snapshot: &mut WorldSnapshot) {
    snapshot.event_trigger_count += 1;
}
fn expect_event_ended(snapshot: &mut WorldSnapshot) {
    snapshot.event.0 = false;
}
fn expect_survivor_spawn(snapshot: &mut WorldSnapshot) {
    snapshot
        .survivors
        .push(("Nia Vale".into(), Position { x: 1, y: 1 }, "Idle".into()));
    snapshot.survivors.sort_by(|left, right| {
        (&left.0, left.1.y, left.1.x, &left.2).cmp(&(&right.0, right.1.y, right.1.x, &right.2))
    });
}
fn expect_task(snapshot: &mut WorldSnapshot) {
    snapshot.survivors[0].2 = "Defending".into();
}
fn expect_teleport(snapshot: &mut WorldSnapshot) {
    snapshot.player_position = Some(Position { x: 8, y: 6 });
}
fn expect_transition(snapshot: &mut WorldSnapshot) {
    snapshot.transition_count += 1;
}

fn mutation_cases() -> Vec<MutationCase> {
    vec![
        MutationCase {
            name: "add-colony-resource",
            command: "supplies 7",
            request: DebugMutation::AddColonyResource {
                kind: PoolKind::Supplies,
                amount: 7,
            },
            prepare: no_prepare,
            expected: expect_supplies,
        },
        MutationCase {
            name: "add-materials",
            command: "materials 6",
            request: DebugMutation::AddColonyResource {
                kind: PoolKind::Materials,
                amount: 6,
            },
            prepare: no_prepare,
            expected: expect_materials,
        },
        MutationCase {
            name: "add-faith",
            command: "faith 5",
            request: DebugMutation::AddColonyResource {
                kind: PoolKind::Faith,
                amount: 5,
            },
            prepare: no_prepare,
            expected: expect_faith,
        },
        MutationCase {
            name: "add-wild-plants",
            command: "plants 4",
            request: DebugMutation::AddColonyResource {
                kind: PoolKind::WildPlants,
                amount: 4,
            },
            prepare: no_prepare,
            expected: expect_wild_plants,
        },
        MutationCase {
            name: "set-day",
            command: "day 7",
            request: DebugMutation::SetDay(7),
            prepare: no_prepare,
            expected: expect_day,
        },
        MutationCase {
            name: "set-turn",
            command: "turn 9",
            request: DebugMutation::SetTurn(9),
            prepare: no_prepare,
            expected: expect_turn,
        },
        MutationCase {
            name: "skip-day",
            command: "skip_day",
            request: DebugMutation::SkipDay,
            prepare: no_prepare,
            expected: expect_skip_day,
        },
        MutationCase {
            name: "trigger-event",
            command: "event test.c3.event",
            request: DebugMutation::TriggerEvent("test.c3.event".into()),
            prepare: no_prepare,
            expected: expect_event_trigger,
        },
        MutationCase {
            name: "end-event",
            command: "end_event",
            request: DebugMutation::EndEvent,
            prepare: prepare_active_event,
            expected: expect_event_ended,
        },
        MutationCase {
            name: "spawn-survivor",
            command: "survivor Nia Vale",
            request: DebugMutation::SpawnSurvivor("Nia Vale".into()),
            prepare: no_prepare,
            expected: expect_survivor_spawn,
        },
        MutationCase {
            name: "assign-survivor-task",
            command: "task 0 defending",
            request: DebugMutation::AssignSurvivorTask {
                index: 0,
                task: DebugSurvivorTask::Defending,
            },
            prepare: prepare_task_target,
            expected: expect_task,
        },
        MutationCase {
            name: "teleport-player",
            command: "goto 8 6",
            request: DebugMutation::TeleportPlayer(Position { x: 8, y: 6 }),
            prepare: no_prepare,
            expected: expect_teleport,
        },
        MutationCase {
            name: "transition-to-shelter",
            command: "shelter",
            request: DebugMutation::TransitionToShelter,
            prepare: no_prepare,
            expected: expect_transition,
        },
    ]
}

#[test]
fn core_debug_gate_defaults_disabled_and_denies_direct_requests() {
    // Supporting DEBUG-GATE-001: reviewer preparation is safe before C3.
    let mut app = App::new();
    app.add_plugins(bd_core::BdCorePlugin);
    assert_eq!(
        *app.world().resource::<DebugMutationGate>(),
        DebugMutationGate::default(),
        "contract=DEBUG-GATE-001 case=core-default expected=disabled"
    );
    let before = app.world().resource::<GameTime>().day;
    app.world_mut()
        .resource_mut::<Messages<DebugMutationRequest>>()
        .write(DebugMutationRequest(DebugMutation::SetDay(77)));
    app.update();

    let mut cursor = MessageCursor::<DebugMutationResult>::default();
    let results = cursor
        .read(app.world().resource::<Messages<DebugMutationResult>>())
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(app.world().resource::<GameTime>().day, before);
    assert_eq!(results.len(), 1);
    assert!(!results[0].accepted);
    assert!(results[0].message.contains("disabled"));
    let trace = &app.world().resource::<SignalTrace>().entries;
    assert_eq!(trace.len(), 1);
    assert_eq!(trace[0].stage, "DebugMutation");
    assert_eq!(trace[0].signal_type, "DebugMutationResult");
    assert!(trace[0].summary.contains("denied"));
}

#[test]
fn console_plugin_explicitly_enables_the_debug_gate() {
    // Contract: DEBUG-GATE-001 (primary integration seam).
    // Given: core begins with debug mutation disabled.
    // When: the development console plugin is deliberately installed.
    // Then: that app explicitly gains debug-mutation authority.
    // Must not change: core-only runtimes remain disabled.
    //
    // Implementation guidance:
    // - Reusable owner: `DebugMutationGate` remains core-owned and defaults off.
    // - Integration seam: `BdConsolePlugin` is the explicit opt-in for this app.
    // - Preserve: no global/release default-on switch and no implicit mutation.
    // - Invalid shortcuts: making the core default true or bypassing the gate in
    //   dispatch is not green.
    // - Closing evidence: run this, both gate tables, core/console/app neighbors,
    //   and the signed candidate gate independently.
    let app = runtime();
    assert!(
        app.world().resource::<DebugMutationGate>().enabled,
        "contract=DEBUG-GATE-001 case=console-opt-in expected=enabled actual=disabled; \
         BdConsolePlugin must deliberately enable the core-owned gate"
    );
}

#[test]
fn disabled_gate_blocks_every_c3_mutation_and_reports_each_denial() {
    // Contract: DEBUG-GATE-001 (completion-critical command table).
    // Every row runs in a fresh production schedule. The boundary snapshot
    // catches direct dispatch mutation; the post-resolver snapshot catches a
    // missing gate check. Failures accumulate so no later row hides.
    let mut violations = Vec::new();
    for case in mutation_cases() {
        let mut app = runtime();
        (case.prepare)(&mut app);
        app.world_mut().resource_mut::<DebugMutationGate>().enabled = false;
        let before = snapshot_world(app.world_mut());
        let trace_before = app.world().resource::<SignalTrace>().entries.len();

        submit(&mut app, case.command);
        app.update();

        let observations = &app.world().resource::<DispatchBoundaryAudit>().observations;
        if observations.len() != 1
            || observations.first().map(|item| &item.0) != Some(&case.request)
            || observations.first().map(|item| &item.1) != Some(&before)
        {
            violations.push(format!(
                "case={} checkpoint=typed-pre-mutation-boundary expected=request={:?},snapshot={:?} actual={:?}",
                case.name, case.request, before, observations
            ));
        }
        let after = snapshot_world(app.world_mut());
        if after != before {
            violations.push(format!(
                "case={} checkpoint=disabled-atomicity expected={:?} actual={:?}",
                case.name, before, after
            ));
        }
        let trace = &app.world().resource::<SignalTrace>().entries[trace_before..];
        if trace.len() != 1
            || trace[0].stage != "DebugMutation"
            || trace[0].signal_type != "DebugMutationResult"
            || !trace[0].summary.contains("denied")
        {
            violations.push(format!(
                "case={} checkpoint=denial-trace expected=one-DebugMutation-denied actual={trace:?}",
                case.name
            ));
        }
        app.update();
        if !output_contains(&app, "ERROR") || !output_contains(&app, "disabled") {
            violations.push(format!(
                "case={} checkpoint=console-denial expected=ERROR+disabled actual={:?}",
                case.name,
                app.world().resource::<bd_console::ConsoleState>().output
            ));
        }
    }
    assert!(
        violations.is_empty(),
        "contract=DEBUG-GATE-001 violations=\n{}",
        violations.join("\n")
    );
}

#[test]
fn every_ordinary_mutation_crosses_one_typed_boundary_then_applies_exactly_one_delta() {
    // Contract: DEBUG-INTENT-001 (primary completion-critical command table).
    // Given: the console-enabled gate and one controlled world per command.
    // When: the command traverses ConsoleCommand -> dispatch -> core resolver.
    // Then: dispatch emits exactly one typed request before mutation, the core
    // applies only the authorized delta/effect, and returns one result + trace.
    // Must not change: read-only commands and C4 combat/factory commands retain
    // their current owners until their separately sealed batches.
    //
    // Implementation guidance:
    // - Reusable owner: one core resolver owns all ten ordinary mutations.
    // - Integration seam: dispatch parses/reports only; event and transition
    //   cases emit their existing canonical typed effects from the resolver.
    // - Preserve: parser aliases, clamping, current component bundles, event and
    //   transition consumers, and all C1/C2 behavior.
    // - Invalid shortcuts: direct mutation plus a decorative request, one
    //   command-specific path, duplicate resolver systems, or prose-parsed
    //   results are not green.
    // - Closing evidence: every row, disabled table, read-only preservation,
    //   named schedule test, target matrix, neighbors, and signed gate.
    let mut violations = Vec::new();
    for case in mutation_cases() {
        let mut app = runtime();
        (case.prepare)(&mut app);
        app.world_mut().resource_mut::<DebugMutationGate>().enabled = true;
        let before = snapshot_world(app.world_mut());
        let trace_before = app.world().resource::<SignalTrace>().entries.len();
        let mut expected = before.clone();
        (case.expected)(&mut expected);

        submit(&mut app, case.command);
        app.update();

        let observations = &app.world().resource::<DispatchBoundaryAudit>().observations;
        if observations.len() != 1
            || observations.first().map(|item| &item.0) != Some(&case.request)
            || observations.first().map(|item| &item.1) != Some(&before)
        {
            violations.push(format!(
                "case={} checkpoint=typed-pre-mutation-boundary expected=request={:?},snapshot={:?} actual={:?}",
                case.name, case.request, before, observations
            ));
        }
        let after = snapshot_world(app.world_mut());
        if after != expected {
            violations.push(format!(
                "case={} checkpoint=exact-delta expected={:?} actual={:?}",
                case.name, expected, after
            ));
        }
        let trace = &app.world().resource::<SignalTrace>().entries[trace_before..];
        if trace.len() != 1
            || trace[0].stage != "DebugMutation"
            || trace[0].signal_type != "DebugMutationResult"
            || !trace[0].summary.contains("accepted")
        {
            violations.push(format!(
                "case={} checkpoint=acceptance-trace expected=one-DebugMutation-accepted actual={trace:?}",
                case.name
            ));
        }
        app.update();
        if !output_contains(&app, "OK:") {
            violations.push(format!(
                "case={} checkpoint=console-result expected=OK actual={:?}",
                case.name,
                app.world().resource::<bd_console::ConsoleState>().output
            ));
        }
    }
    assert!(
        violations.is_empty(),
        "contract=DEBUG-INTENT-001 violations=\n{}",
        violations.join("\n")
    );
}

fn remove_player(app: &mut App) {
    let player = {
        let mut query = app.world_mut().query_filtered::<Entity, With<Player>>();
        query
            .iter(app.world())
            .next()
            .expect("fixture player must exist")
    };
    assert!(app.world_mut().despawn(player));
}

struct RejectionCase {
    name: &'static str,
    command: &'static str,
    request: DebugMutation,
    prepare: fn(&mut App),
    output_token: &'static str,
}

#[test]
fn enabled_invalid_mutations_are_atomic_and_return_one_rejection_trace() {
    // Supporting DEBUG-INTENT-001: an enabled gate grants authority to ask,
    // not authority to bypass ordinary preconditions. Each rejection crosses
    // the same typed boundary and leaves the full observed world unchanged.
    let cases = [
        RejectionCase {
            name: "unknown-event",
            command: "event missing.c3.event",
            request: DebugMutation::TriggerEvent("missing.c3.event".into()),
            prepare: no_prepare,
            output_token: "missing.c3.event",
        },
        RejectionCase {
            name: "event-without-player",
            command: "event test.c3.event",
            request: DebugMutation::TriggerEvent("test.c3.event".into()),
            prepare: remove_player,
            output_token: "player",
        },
        RejectionCase {
            name: "inactive-event-end",
            command: "end_event",
            request: DebugMutation::EndEvent,
            prepare: no_prepare,
            output_token: "active event",
        },
        RejectionCase {
            name: "task-index-out-of-bounds",
            command: "task 4 resting",
            request: DebugMutation::AssignSurvivorTask {
                index: 4,
                task: DebugSurvivorTask::Resting,
            },
            prepare: no_prepare,
            output_token: "index",
        },
        RejectionCase {
            name: "teleport-without-player",
            command: "goto 8 6",
            request: DebugMutation::TeleportPlayer(Position { x: 8, y: 6 }),
            prepare: remove_player,
            output_token: "player",
        },
    ];
    let mut violations = Vec::new();
    for case in cases {
        let mut app = runtime();
        app.world_mut().resource_mut::<DebugMutationGate>().enabled = true;
        (case.prepare)(&mut app);
        let before = snapshot_world(app.world_mut());
        let trace_before = app.world().resource::<SignalTrace>().entries.len();
        submit(&mut app, case.command);
        app.update();

        let observations = &app.world().resource::<DispatchBoundaryAudit>().observations;
        if observations.len() != 1
            || observations.first().map(|item| &item.0) != Some(&case.request)
            || observations.first().map(|item| &item.1) != Some(&before)
        {
            violations.push(format!(
                "case={} checkpoint=typed-rejection-boundary expected=request={:?},snapshot={:?} actual={:?}",
                case.name, case.request, before, observations
            ));
        }
        let after = snapshot_world(app.world_mut());
        if after != before {
            violations.push(format!(
                "case={} checkpoint=rejection-atomicity expected={:?} actual={:?}",
                case.name, before, after
            ));
        }
        let trace = &app.world().resource::<SignalTrace>().entries[trace_before..];
        if trace.len() != 1
            || trace[0].stage != "DebugMutation"
            || trace[0].signal_type != "DebugMutationResult"
            || !trace[0].summary.contains("rejected")
        {
            violations.push(format!(
                "case={} checkpoint=rejection-trace expected=one-DebugMutation-rejected actual={trace:?}",
                case.name
            ));
        }
        app.update();
        if !output_contains(&app, "ERROR") || !output_contains(&app, case.output_token) {
            violations.push(format!(
                "case={} checkpoint=rejection-result expected=ERROR+{} actual={:?}",
                case.name,
                case.output_token,
                app.world().resource::<bd_console::ConsoleState>().output
            ));
        }
    }
    assert!(
        violations.is_empty(),
        "contract=DEBUG-INTENT-001 rejection-violations=\n{}",
        violations.join("\n")
    );
}

#[test]
fn read_only_and_console_local_commands_emit_no_debug_mutation() {
    // Supporting DEBUG-INTENT-001: queries remain queries, and clear remains a
    // ConsoleState-only operation rather than a fake core mutation.
    let cases = ["help", "stats", "blueprints", "events", "clear"];
    let mut violations = Vec::new();
    for command in cases {
        let mut app = runtime();
        app.world_mut().resource_mut::<DebugMutationGate>().enabled = true;
        app.world_mut()
            .resource_mut::<bd_console::ConsoleState>()
            .output
            .push("seed".into());
        let before = snapshot_world(app.world_mut());
        let trace_before = app.world().resource::<SignalTrace>().entries.len();
        submit(&mut app, command);
        app.update();
        let after = snapshot_world(app.world_mut());
        let observations = &app.world().resource::<DispatchBoundaryAudit>().observations;
        let trace_after = app.world().resource::<SignalTrace>().entries.len();
        let output = &app.world().resource::<bd_console::ConsoleState>().output;
        let output_ok = if command == "clear" {
            output.is_empty()
        } else {
            !output.is_empty()
        };
        if after != before || !observations.is_empty() || trace_after != trace_before || !output_ok
        {
            violations.push(format!(
                "case={command} expected=(same-world,no-request,no-trace,truthful-output) \
                 actual=(before={before:?},after={after:?},requests={observations:?},trace_delta={},output={output:?})",
                trace_after.saturating_sub(trace_before)
            ));
        }
    }
    assert!(
        violations.is_empty(),
        "contract=DEBUG-INTENT-001 preservation-violations=\n{}",
        violations.join("\n")
    );
}

#[test]
fn c4_combat_god_and_blueprint_commands_preserve_their_existing_behavior() {
    // C3 preservation only. C4 owns migration and semantic strengthening for
    // kill_all, heal, GodMode, and blueprint spawning; this batch may neither
    // delete those routes nor claim their later contracts.
    let mut violations = Vec::new();

    let mut kill = runtime();
    kill.world_mut().spawn((
        Name("C3 Preservation Enemy".into()),
        Position { x: 7, y: 7 },
        Pools::new(vec![Pool::new(PoolKind::Health, 5, 0, 5)]),
    ));
    submit(&mut kill, "kill_all");
    kill.update();
    if kill.world().resource::<Messages<EntityDefeated>>().len() != 1
        || !output_contains(&kill, "OK:")
    {
        violations.push(format!(
            "case=kill-all expected=one-defeat+OK actual=(defeats={},output={:?})",
            kill.world().resource::<Messages<EntityDefeated>>().len(),
            kill.world().resource::<bd_console::ConsoleState>().output
        ));
    }

    let mut heal = runtime();
    let healer = {
        let mut query = heal.world_mut().query_filtered::<Entity, With<Player>>();
        query.iter(heal.world()).next().expect("player must exist")
    };
    heal.world_mut()
        .entity_mut(healer)
        .insert(Pools::new(vec![Pool::new(PoolKind::Health, 5, 0, 30)]));
    submit(&mut heal, "heal");
    heal.update();
    let health = heal
        .world()
        .get::<Pools>(healer)
        .and_then(|pools| pools.get(PoolKind::Health))
        .map(|pool| pool.current);
    if health != Some(30) || !output_contains(&heal, "OK:") {
        violations.push(format!(
            "case=heal expected=(health=30,OK) actual=(health={health:?},output={:?})",
            heal.world().resource::<bd_console::ConsoleState>().output
        ));
    }

    let mut god = runtime();
    let god_player = {
        let mut query = god.world_mut().query_filtered::<Entity, With<Player>>();
        query.iter(god.world()).next().expect("player must exist")
    };
    submit(&mut god, "god on");
    god.update();
    if !god.world().entity(god_player).contains::<GodMode>() || !output_contains(&god, "ON") {
        violations.push(format!(
            "case=god-on expected=(marker=true,ON) actual=(marker={},output={:?})",
            god.world().entity(god_player).contains::<GodMode>(),
            god.world().resource::<bd_console::ConsoleState>().output
        ));
    }

    let mut spawn = runtime();
    spawn
        .world_mut()
        .insert_resource(BlueprintCatalog::new(vec![EntityBlueprint {
            id: "blueprint.c3_preservation".into(),
            label: "C3 Preservation Spawn".into(),
            is_player: false,
            blocks_movement: true,
            pools: vec![(PoolKind::Health, 11, 0, 11)],
            statuses: vec![],
            visual: None,
            markers: vec![],
        }]));
    submit(&mut spawn, "spawn blueprint.c3_preservation 8 4");
    spawn.update();
    let spawned = {
        let mut query = spawn.world_mut().query::<(
            Entity,
            &Name,
            &Position,
            Option<&BlocksMovement>,
            Option<&Pools>,
        )>();
        query
            .iter(spawn.world())
            .find(|(_, name, _, _, _)| name.0 == "C3 Preservation Spawn")
            .map(|(entity, _, position, blocks, pools)| {
                (
                    entity,
                    *position,
                    blocks.is_some(),
                    pools
                        .and_then(|pools| pools.get(PoolKind::Health))
                        .map(|pool| pool.current),
                )
            })
    };
    if spawned
        .as_ref()
        .map(|(_, position, blocks, health)| (*position, *blocks, *health))
        != Some((Position { x: 8, y: 4 }, true, Some(11)))
        || !output_contains(&spawn, "OK:")
    {
        violations.push(format!(
            "case=blueprint-spawn expected=((8,4),blocking,health=11,OK) actual=({spawned:?},{:?})",
            spawn.world().resource::<bd_console::ConsoleState>().output
        ));
    }

    assert!(
        violations.is_empty(),
        "C3 C4-preservation violations=\n{}",
        violations.join("\n")
    );
}

#[test]
fn debug_dispatch_precedes_exactly_one_named_core_resolver() {
    // Supporting DEBUG-INTENT-001.
    // The behavioral seam observer above deliberately adds its own edge, so
    // this separate production-only schedule proves the real registration.
    let mut app = App::new();
    app.add_plugins(bd_core::BdCorePlugin);
    app.add_message::<KeyMessage>();
    app.add_plugins(bd_console::BdConsolePlugin);
    app.update();

    let schedules = app.world().resource::<bevy_ecs::schedule::Schedules>();
    let schedule = schedules.get(Update).expect("Update schedule must exist");
    let graph = schedule.graph();
    let dispatch_keys = graph
        .systems_in_set(
            bd_console::dispatch::execute_console_command
                .into_system_set()
                .intern(),
        )
        .expect("console dispatcher must be registered");
    let resolver_set = DebugMutationSet::Resolve.intern();
    let resolver_set_key = graph
        .system_sets
        .get_key(resolver_set)
        .expect("named debug resolver set must have a schedule key");
    let resolver_keys = graph
        .systems_in_set(resolver_set)
        .expect("named debug resolver set must be registered");
    let ordered = dispatch_keys.iter().any(|dispatch| {
        graph
            .dependency()
            .contains_edge(NodeId::System(*dispatch), NodeId::Set(resolver_set_key))
    });

    assert_eq!(
        (dispatch_keys.len(), resolver_keys.len(), ordered),
        (1, 1, true),
        "contract=DEBUG-INTENT-001 case=named-resolver-order \
         expected=(dispatchers=1,resolvers=1,dispatch-before-resolver=true) \
         actual=(dispatchers={},resolvers={},dispatch-before-resolver={}); \
         insertion order, ambiguity suppression, or a second resolver is not an explicit owner",
        dispatch_keys.len(),
        resolver_keys.len(),
        ordered
    );
}

#[derive(Resource, Debug, Default)]
struct PostResolverRequestAudit(Vec<DebugMutation>);

fn audit_debug_requests_after_resolver(
    mut requests: MessageReader<DebugMutationRequest>,
    mut audit: ResMut<PostResolverRequestAudit>,
) {
    audit
        .0
        .extend(requests.read().map(|request| request.0.clone()));
}

#[test]
fn debug_request_channel_remains_observable_after_core_resolution() {
    // Supporting DEBUG-INTENT-001.
    // Given: the console's typed request channel has a second independent
    // reader scheduled after the core resolver.
    // When: one ordinary mutation crosses dispatch and resolves.
    // Then: the core applies it once while the independent reader still sees
    // the same request once.
    // Must not change: one resolver owns mutation; the observer is read-only.
    //
    // Implementation guidance:
    // - Reusable owner: the named core resolver owns mutation and maintains its
    //   own request cursor without consuming the shared message collection.
    // - Integration seam: `DebugMutationSet::Resolve` in `BdSet::Mutation`.
    // - Preserve: the accepted/rejected/denied matrices, result order, trace
    //   order, and explicit dispatch-before-resolver edge.
    // - Invalid shortcuts: moving this observer before the resolver, emitting a
    //   duplicate request, adding a second resolver, or reconstructing the
    //   request from result/trace prose is not green.
    // - Closing evidence: run this case independently with all nine existing
    //   C3 cases, the console/core/app neighbors, and the signed v2 gate.
    let mut app = runtime();
    app.init_resource::<PostResolverRequestAudit>();
    app.add_systems(
        Update,
        audit_debug_requests_after_resolver
            .after(DebugMutationSet::Resolve)
            .in_set(BdSet::ResultEmission),
    );

    submit(&mut app, "day 23");
    app.update();

    assert_eq!(
        app.world().resource::<GameTime>().day,
        23,
        "contract=DEBUG-INTENT-001 case=post-resolver-fanout checkpoint=resolved-delta expected=day-23 actual={}",
        app.world().resource::<GameTime>().day
    );
    assert_eq!(
        app.world().resource::<PostResolverRequestAudit>().0,
        vec![DebugMutation::SetDay(23)],
        "contract=DEBUG-INTENT-001 case=post-resolver-fanout checkpoint=independent-reader expected=[SetDay(23)] actual={:?}; the resolver must not drain the shared request channel",
        app.world().resource::<PostResolverRequestAudit>().0
    );

    // A cursor created inside the resolver body would satisfy the first-frame
    // fan-out assertion but replay the retained request on the next update.
    // Start a fresh result cursor after the accepted result, then prove an idle
    // update creates no second result, trace, or observer delivery.
    let mut later_results = app
        .world()
        .resource::<Messages<DebugMutationResult>>()
        .get_cursor_current();
    let trace_count = app.world().resource::<SignalTrace>().entries.len();
    app.update();
    let replayed_results = later_results
        .read(app.world().resource::<Messages<DebugMutationResult>>())
        .cloned()
        .collect::<Vec<_>>();
    let actual = (
        app.world().resource::<PostResolverRequestAudit>().0.len(),
        replayed_results,
        app.world().resource::<SignalTrace>().entries.len(),
    );
    let expected = (1, Vec::<DebugMutationResult>::new(), trace_count);
    assert_eq!(
        actual, expected,
        "contract=DEBUG-INTENT-001 case=post-resolver-fanout checkpoint=no-second-frame-replay expected={expected:?} actual={actual:?}; the resolver cursor must persist across frames",
    );
}

fn survivor_task_at(app: &mut App, position: Position) -> SurvivorTask {
    let mut query = app
        .world_mut()
        .query_filtered::<(&Position, &SurvivorTask), With<Survivor>>();
    query
        .iter(app.world())
        .find(|(candidate, _)| **candidate == position)
        .map(|(_, task)| task.clone())
        .unwrap_or_else(|| panic!("fixture survivor missing at {position:?}"))
}

fn stable_target_runtime(reverse: bool) -> App {
    let mut app = runtime();
    app.world_mut().resource_mut::<DebugMutationGate>().enabled = true;
    let rows = [
        ("Alex", Position { x: 6, y: 2 }, SurvivorTask::Resting),
        ("Alex", Position { x: 2, y: 2 }, SurvivorTask::Idle),
        ("Bryn", Position { x: 1, y: 1 }, SurvivorTask::Idle),
    ];
    if reverse {
        for (name, position, task) in rows.into_iter().rev() {
            spawn_survivor(&mut app, name, position, task);
        }
    } else {
        for (name, position, task) in rows {
            spawn_survivor(&mut app, name, position, task);
        }
    }
    app
}

#[test]
fn survivor_indices_share_one_visible_stable_order_and_reject_indistinguishable_duplicates() {
    // Contract: DEBUG-TARGET-001 (primary).
    // Given: unlike spawn orders, duplicate names at distinct positions, and
    // a second case whose visible identity is exactly duplicated.
    // When: stats advertises indices and `task 0` uses one.
    // Then: both runtimes show the same ordered rows and mutate the same
    // visible target; an indistinguishable tie is rejected atomically.
    // Must not change: no raw Entity value appears in output.
    //
    // Implementation guidance:
    // - Reusable owner: derive the survivor target projection once in core and
    //   reuse it for read-only listing and mutation selection.
    // - Integration seam: stats exposes index + name + position; task results
    //   repeat the chosen visible identity.
    // - Preserve: existing task vocabulary and non-target survivor state.
    // - Invalid shortcuts: sorting separately in dispatch/resolver, falling
    //   back to Entity order for ties, hiding duplicates, or displaying raw
    //   entity IDs is not green.
    // - Closing evidence: both insertion orders and the exact-duplicate denial
    //   must pass with the ordinary mutation table and signed gate.
    let mut forward = stable_target_runtime(false);
    let mut reverse = stable_target_runtime(true);
    let mut violations = Vec::new();

    submit(&mut forward, "stats");
    submit(&mut reverse, "stats");
    forward.update();
    reverse.update();
    let forward_stats = forward
        .world()
        .resource::<bd_console::ConsoleState>()
        .output
        .last()
        .cloned()
        .unwrap_or_default();
    let reverse_stats = reverse
        .world()
        .resource::<bd_console::ConsoleState>()
        .output
        .last()
        .cloned()
        .unwrap_or_default();
    if forward_stats != reverse_stats {
        violations.push(format!(
            "case=spawn-order-independent-list expected=same-output actual=(forward={forward_stats:?},reverse={reverse_stats:?})"
        ));
    }
    if !(forward_stats.contains("#0")
        && forward_stats.contains("Alex")
        && forward_stats.contains("(2,2)"))
    {
        violations.push(format!(
            "case=visible-index expected=#0+Alex+(2,2) actual={forward_stats:?}"
        ));
    }
    if forward_stats.contains("Entity") || forward_stats.contains("429496") {
        violations.push(format!(
            "case=no-raw-ecs-identity forbidden=raw-entity actual={forward_stats:?}"
        ));
    }

    submit(&mut forward, "task 0 defending");
    submit(&mut reverse, "task 0 defending");
    forward.update();
    reverse.update();
    for (label, app) in [("forward", &mut forward), ("reverse", &mut reverse)] {
        let selected = survivor_task_at(app, Position { x: 2, y: 2 });
        let neighbor = survivor_task_at(app, Position { x: 6, y: 2 });
        if selected != SurvivorTask::Defending || neighbor != SurvivorTask::Resting {
            violations.push(format!(
                "case={label}-selected-target expected=(2,2)=Defending,(6,2)=Resting actual=({selected:?},{neighbor:?})"
            ));
        }
    }
    forward.update();
    reverse.update();
    for (label, app) in [("forward", &forward), ("reverse", &reverse)] {
        if !(output_contains(app, "Alex") && output_contains(app, "(2,2)")) {
            violations.push(format!(
                "case={label}-result-identity expected=Alex+(2,2) actual={:?}",
                app.world().resource::<bd_console::ConsoleState>().output
            ));
        }
    }

    let mut ambiguous = runtime();
    ambiguous
        .world_mut()
        .resource_mut::<DebugMutationGate>()
        .enabled = true;
    spawn_survivor(
        &mut ambiguous,
        "Alex",
        Position { x: 3, y: 3 },
        SurvivorTask::Idle,
    );
    spawn_survivor(
        &mut ambiguous,
        "Alex",
        Position { x: 3, y: 3 },
        SurvivorTask::Resting,
    );
    submit(&mut ambiguous, "task 0 defending");
    ambiguous.update();
    let mut query = ambiguous
        .world_mut()
        .query_filtered::<&SurvivorTask, With<Survivor>>();
    let tasks = query.iter(ambiguous.world()).cloned().collect::<Vec<_>>();
    if !(tasks.contains(&SurvivorTask::Idle)
        && tasks.contains(&SurvivorTask::Resting)
        && !tasks.contains(&SurvivorTask::Defending))
    {
        violations.push(format!(
            "case=ambiguous-tie-atomicity expected=[Idle,Resting] actual={tasks:?}"
        ));
    }
    ambiguous.update();
    if !(output_contains(&ambiguous, "ERROR")
        && output_contains(&ambiguous, "ambiguous")
        && output_contains(&ambiguous, "Alex")
        && output_contains(&ambiguous, "(3,3)"))
    {
        violations.push(format!(
            "case=ambiguous-visible-denial expected=ERROR+ambiguous+Alex+(3,3) actual={:?}",
            ambiguous
                .world()
                .resource::<bd_console::ConsoleState>()
                .output
        ));
    }
    assert!(
        violations.is_empty(),
        "contract=DEBUG-TARGET-001 violations=\n{}",
        violations.join("\n")
    );
}
