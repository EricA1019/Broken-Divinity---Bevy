//! Authoritative C4 developer-console mutation contracts.
//!
//! The observer runs after console dispatch and before the core debug resolver.
//! It proves that dispatch emitted one typed request while the complete C4
//! state was still unchanged; a decorative request after direct mutation does
//! not satisfy these contracts.

use bd_core::{
    BdSet,
    colony::{raids::RaidEnemy, survivors::Survivor},
    combat::Armor,
    components::{BlocksMovement, GodMode, Name, Player, Position},
    debug::{
        DebugMutation, DebugMutationGate, DebugMutationRequest, DebugMutationResult,
        DebugMutationSet,
    },
    factory::{BlueprintCatalog, EntityBlueprint, spawn_from_blueprint},
    pools::{Pool, Pools},
    relationships::FactionMember,
    signals::{DeltaTag, EntityDefeated, PoolDeltaApplied, PoolDeltaRequested, PoolKind},
    spatial::{EntityScope, GameMode, PersistentEntity, TransientEntity},
    statuses::Statuses,
    trace::SignalTrace,
};
use bevy_app::{App, Update};
use bevy_ecs::{
    message::{MessageCursor, Messages},
    prelude::*,
    schedule::IntoScheduleConfigs,
};
use bevy_ratatui::event::KeyMessage;

#[derive(Debug, Clone, PartialEq, Eq)]
struct EntityRecord {
    entity: u64,
    name: Option<String>,
    position: Option<Position>,
    player: bool,
    survivor: bool,
    god: bool,
    blocks: bool,
    pools: Vec<(String, i32, i32, i32)>,
    statuses: Vec<(String, i32, i32, Option<u64>)>,
    raid_enemy: bool,
    faction: Option<String>,
    scope: Option<EntityScope>,
    legacy_persistent: bool,
    legacy_transient: bool,
    armor: Option<(i32, i32)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct C4Snapshot {
    mode: GameMode,
    entities: Vec<EntityRecord>,
    pool_requested: usize,
    pool_applied: usize,
    defeated: usize,
}

fn pools_record(pools: Option<&Pools>) -> Vec<(String, i32, i32, i32)> {
    let mut values = pools
        .into_iter()
        .flat_map(Pools::iter)
        .map(|pool| (format!("{:?}", pool.kind), pool.current, pool.min, pool.max))
        .collect::<Vec<_>>();
    values.sort();
    values
}

fn statuses_record(statuses: Option<&Statuses>) -> Vec<(String, i32, i32, Option<u64>)> {
    let mut values = statuses
        .into_iter()
        .flat_map(|statuses| statuses.instances.iter())
        .map(|status| {
            (
                status.status_id.clone(),
                status.remaining_duration,
                status.stacks,
                status.source.map(Entity::to_bits),
            )
        })
        .collect::<Vec<_>>();
    values.sort();
    values
}

fn entity_record(world: &World, entity: Entity) -> EntityRecord {
    EntityRecord {
        entity: entity.to_bits(),
        name: world.get::<Name>(entity).map(|name| name.0.clone()),
        position: world.get::<Position>(entity).copied(),
        player: world.get::<Player>(entity).is_some(),
        survivor: world.get::<Survivor>(entity).is_some(),
        god: world.get::<GodMode>(entity).is_some(),
        blocks: world.get::<BlocksMovement>(entity).is_some(),
        pools: pools_record(world.get::<Pools>(entity)),
        statuses: statuses_record(world.get::<Statuses>(entity)),
        raid_enemy: world.get::<RaidEnemy>(entity).is_some(),
        faction: world
            .get::<FactionMember>(entity)
            .map(|faction| faction.0.clone()),
        scope: world.get::<EntityScope>(entity).copied(),
        legacy_persistent: world.get::<PersistentEntity>(entity).is_some(),
        legacy_transient: world.get::<TransientEntity>(entity).is_some(),
        armor: world
            .get::<Armor>(entity)
            .map(|armor| (armor.reduction, armor.durability)),
    }
}

fn snapshot(world: &mut World) -> C4Snapshot {
    let mut query = world.query::<Entity>();
    let mut entities = query
        .iter(world)
        .map(|entity| entity_record(world, entity))
        .collect::<Vec<_>>();
    entities.sort_by_key(|entity| entity.entity);
    C4Snapshot {
        mode: *world.resource::<GameMode>(),
        entities,
        pool_requested: world.resource::<Messages<PoolDeltaRequested>>().len(),
        pool_applied: world.resource::<Messages<PoolDeltaApplied>>().len(),
        defeated: world.resource::<Messages<EntityDefeated>>().len(),
    }
}

#[derive(Resource, Default)]
struct C4BoundaryAudit {
    cursor: MessageCursor<DebugMutationRequest>,
    observations: Vec<(DebugMutation, C4Snapshot)>,
}

fn audit_c4_boundary(world: &mut World) {
    let mut audit = world
        .remove_resource::<C4BoundaryAudit>()
        .expect("C4 boundary audit must be installed");
    let requests = audit
        .cursor
        .read(world.resource::<Messages<DebugMutationRequest>>())
        .map(|request| request.0.clone())
        .collect::<Vec<_>>();
    if !requests.is_empty() {
        let checkpoint = snapshot(world);
        audit.observations.extend(
            requests
                .into_iter()
                .map(|request| (request, checkpoint.clone())),
        );
    }
    world.insert_resource(audit);
}

fn runtime() -> App {
    let mut app = App::new();
    app.add_plugins(bd_core::BdCorePlugin);
    app.add_message::<KeyMessage>();
    app.add_plugins(bd_console::BdConsolePlugin);
    app.init_resource::<C4BoundaryAudit>();
    app.add_systems(
        Update,
        audit_c4_boundary
            .after(bd_console::dispatch::execute_console_command)
            .before(DebugMutationSet::Resolve)
            .in_set(BdSet::Mutation),
    );
    app.world_mut().spawn((
        Player,
        Name("C4 Witness".into()),
        Position { x: 4, y: 4 },
        Pools::new(vec![
            Pool::new(PoolKind::Health, 7, 0, 30),
            Pool::new(PoolKind::ActionPoints, 1, 0, 4),
        ]),
    ));
    app
}

fn submit(app: &mut App, command: &str) {
    app.world_mut()
        .resource_mut::<Messages<bd_console::ConsoleCommand>>()
        .write(bd_console::ConsoleCommand(command.into()));
}

fn player(app: &mut App) -> Entity {
    let mut query = app.world_mut().query_filtered::<Entity, With<Player>>();
    query
        .iter(app.world())
        .find(|entity| {
            app.world()
                .get::<Name>(*entity)
                .is_some_and(|name| name.0 == "C4 Witness")
        })
        .expect("C4 fixture player must exist")
}

fn output_has(app: &App, token: &str) -> bool {
    app.world()
        .resource::<bd_console::ConsoleState>()
        .output
        .iter()
        .any(|line| line.contains(token))
}

fn debug_traces(app: &App, start: usize) -> Vec<String> {
    app.world().resource::<SignalTrace>().entries[start..]
        .iter()
        .filter(|entry| {
            entry.stage == "DebugMutation" && entry.signal_type == "DebugMutationResult"
        })
        .map(|entry| entry.summary.clone())
        .collect()
}

fn one_result(app: &App) -> Vec<DebugMutationResult> {
    let mut cursor = MessageCursor::<DebugMutationResult>::default();
    cursor
        .read(app.world().resource::<Messages<DebugMutationResult>>())
        .cloned()
        .collect()
}

#[test]
fn remaining_combat_and_spawn_commands_use_one_gated_typed_owner() {
    // Supporting DEBUG-GOD-001 / DEBUG-SPAWN-001.
    // Given: four fresh console runtimes, first enabled and then disabled.
    // When: kill_all, heal, god on, and spawn traverse production dispatch.
    // Then: each emits one typed request before mutation; enabled requests use
    // canonical effects, while disabled requests preserve all state.
    // Must not change: read-only commands and the accepted C3 routes.
    //
    // Implementation guidance:
    // - Reusable owner: extend the existing core DebugMutation resolver.
    // - Integration seam: dispatch parses/emits only; resolver owns validation,
    //   effects, result, and trace before ResultEmission.
    // - Preserve: one request/result/trace per command and same-frame healing.
    // - Invalid shortcuts: direct mutation plus a decorative request, a second
    //   resolver, direct Pools writes, or bypassing the disabled gate.
    // - Closing evidence: run this case in both gate states plus every primary.
    let rows = [
        ("kill-all", "kill_all", DebugMutation::KillAllEnemies),
        ("heal", "heal", DebugMutation::HealPlayer),
        ("god-on", "god on", DebugMutation::SetGodMode(true)),
        (
            "spawn",
            "spawn blueprint.c4.boundary 8 3",
            DebugMutation::SpawnBlueprint {
                blueprint_id: "blueprint.c4.boundary".into(),
                position: Position { x: 8, y: 3 },
            },
        ),
    ];
    let mut violations = Vec::new();

    for enabled in [true, false] {
        for (name, command, expected_request) in &rows {
            let mut app = runtime();
            app.world_mut().resource_mut::<DebugMutationGate>().enabled = enabled;
            app.world_mut()
                .insert_resource(BlueprintCatalog::new(vec![EntityBlueprint {
                    id: "blueprint.c4.boundary".into(),
                    label: "C4 Boundary Spawn".into(),
                    is_player: false,
                    blocks_movement: true,
                    pools: vec![(PoolKind::Health, 9, 0, 9)],
                    statuses: vec![],
                    visual: None,
                    markers: vec!["RaidEnemy".into()],
                }]));
            app.world_mut().spawn((
                Name("C4 Boundary Enemy".into()),
                Position { x: 6, y: 6 },
                Pools::new(vec![Pool::new(PoolKind::Health, 5, 0, 5)]),
            ));
            let before = snapshot(app.world_mut());
            let trace_start = app.world().resource::<SignalTrace>().entries.len();
            submit(&mut app, command);
            app.update();

            let observations = &app.world().resource::<C4BoundaryAudit>().observations;
            if observations.len() != 1
                || observations.first().map(|entry| &entry.0) != Some(expected_request)
                || observations.first().map(|entry| &entry.1) != Some(&before)
            {
                violations.push(format!(
                    "gate={enabled} case={name} checkpoint=typed-pre-mutation-boundary expected=({expected_request:?},{before:?}) actual={observations:?}"
                ));
            }
            let results = one_result(&app);
            let traces = debug_traces(&app, trace_start);
            if results.len() != 1 || results[0].accepted != enabled {
                violations.push(format!(
                    "gate={enabled} case={name} checkpoint=result expected=one accepted={enabled} actual={results:?}"
                ));
            }
            let trace_word = if enabled { "accepted" } else { "denied" };
            if traces.len() != 1 || !traces[0].contains(trace_word) {
                violations.push(format!(
                    "gate={enabled} case={name} checkpoint=trace expected=one-{trace_word} actual={traces:?}"
                ));
            }
            if !enabled {
                let after = snapshot(app.world_mut());
                if after != before || !output_has(&app, "disabled") {
                    violations.push(format!(
                        "gate=false case={name} checkpoint=atomic-denial expected={before:?}+disabled actual={after:?}+{:?}",
                        app.world().resource::<bd_console::ConsoleState>().output
                    ));
                }
                continue;
            }

            match *name {
                "kill-all" => {
                    if app.world().resource::<Messages<EntityDefeated>>().len() != 1 {
                        violations.push(format!(
                            "case=kill-all checkpoint=canonical-effect expected=one-EntityDefeated actual={}",
                            app.world().resource::<Messages<EntityDefeated>>().len()
                        ));
                    }
                }
                "heal" => {
                    let witness = player(&mut app);
                    let pools = app.world().get::<Pools>(witness).expect("player pools");
                    let requested = app.world().resource::<Messages<PoolDeltaRequested>>().len();
                    let applied = app.world().resource::<Messages<PoolDeltaApplied>>().len();
                    if pools.get(PoolKind::Health).map(|pool| pool.current) != Some(30)
                        || pools.get(PoolKind::ActionPoints).map(|pool| pool.current) != Some(4)
                        || requested != 2
                        || applied != 2
                    {
                        violations.push(format!(
                            "case=heal checkpoint=canonical-signed-deltas expected=(health=30,ap=4,requested=2,applied=2) actual=({:?},{:?},{requested},{applied})",
                            pools.get(PoolKind::Health).map(|pool| pool.current),
                            pools.get(PoolKind::ActionPoints).map(|pool| pool.current)
                        ));
                    }
                }
                "god-on" => {
                    let witness = player(&mut app);
                    if app.world().get::<GodMode>(witness).is_none() {
                        violations.push(
                            "case=god-on checkpoint=resolver-mutation expected=GodMode actual=absent"
                                .into(),
                        );
                    }
                }
                "spawn" => {
                    let found = {
                        let mut query = app.world_mut().query::<&Name>();
                        query
                            .iter(app.world())
                            .any(|name| name.0 == "C4 Boundary Spawn")
                    };
                    if !found {
                        violations.push(
                            "case=spawn checkpoint=resolver-mutation expected=spawned actual=absent"
                                .into(),
                        );
                    }
                }
                _ => unreachable!(),
            }
        }
    }

    assert!(
        violations.is_empty(),
        "C4 gated-boundary violations:\n{}",
        violations.join("\n")
    );
}

struct GodDeltaCase {
    name: &'static str,
    player: bool,
    god: bool,
    kind: PoolKind,
    amount: i32,
    tags: Vec<DeltaTag>,
    before: i32,
    expected_after: i32,
}

#[test]
fn god_mode_blocks_only_negative_player_health_deltas_inside_canonical_resolution() {
    // Contract: DEBUG-GOD-001.
    // Given: representative player/non-player, GodMode/plain, Health/AP, and
    // positive/negative signed-delta rows.
    // When: each request crosses the canonical PoolDelta resolver.
    // Then: only negative Health for a GodMode player becomes an observable
    // zero application; healing, costs, ordinary damage, and non-player damage
    // retain their established behavior.
    // Must not change: armor durability, Wounded status, or defeat on the
    // blocked row; every valid request still emits one PoolDeltaApplied record.
    //
    // Implementation guidance:
    // - Reusable owner: the sole signed-delta resolver owns this narrow rule.
    // - Integration seam: GodMode is toggled through the debug owner, while all
    //   damage/heal/cost sources continue to emit PoolDeltaRequested.
    // - Preserve: modifier/variance/armor ordering for unblocked damage.
    // - Invalid shortcuts: healing after damage, a console-only flag check,
    //   blocking AP costs/healing/non-player damage, or suppressing telemetry.
    // - Closing evidence: run every row independently, then core/app neighbors.
    let cases = vec![
        GodDeltaCase {
            name: "god-player-negative-health",
            player: true,
            god: true,
            kind: PoolKind::Health,
            amount: -50,
            tags: vec![DeltaTag::Physical],
            before: 12,
            expected_after: 12,
        },
        GodDeltaCase {
            name: "god-player-positive-health",
            player: true,
            god: true,
            kind: PoolKind::Health,
            amount: 5,
            tags: vec![DeltaTag::Recovery],
            before: 12,
            expected_after: 17,
        },
        GodDeltaCase {
            name: "god-player-negative-ap",
            player: true,
            god: true,
            kind: PoolKind::ActionPoints,
            amount: -2,
            tags: vec![DeltaTag::Action],
            before: 4,
            expected_after: 2,
        },
        GodDeltaCase {
            name: "god-non-player-negative-health",
            player: false,
            god: true,
            kind: PoolKind::Health,
            amount: -4,
            tags: vec![DeltaTag::Celestial],
            before: 12,
            expected_after: 8,
        },
        GodDeltaCase {
            name: "plain-player-negative-health",
            player: true,
            god: false,
            kind: PoolKind::Health,
            amount: -4,
            tags: vec![DeltaTag::Celestial],
            before: 12,
            expected_after: 8,
        },
    ];
    let mut violations = Vec::new();

    for case in cases {
        let mut app = runtime();
        let fixture_player = player(&mut app);
        app.world_mut().despawn(fixture_player);
        let mut entity = app.world_mut().spawn((
            Name(format!("C4 God Case {}", case.name)),
            Position { x: 3, y: 3 },
            Pools::new(vec![Pool::new(case.kind, case.before, 0, 30)]),
            Armor {
                reduction: 3,
                durability: 2,
            },
        ));
        if case.player {
            entity.insert(Player);
        }
        if case.god {
            entity.insert(GodMode);
        }
        let target = entity.id();

        if case.name == "god-player-negative-health" {
            app.world_mut().entity_mut(target).remove::<GodMode>();
            submit(&mut app, "god on");
            app.update();
            if app.world().get::<GodMode>(target).is_none() {
                violations.push(format!(
                    "case={} checkpoint=console-toggle expected=GodMode actual=absent",
                    case.name
                ));
            }
        }

        let defeated_before = app.world().resource::<Messages<EntityDefeated>>().len();
        app.world_mut()
            .resource_mut::<Messages<PoolDeltaRequested>>()
            .write(PoolDeltaRequested {
                source: None,
                target,
                kind: case.kind,
                amount: case.amount,
                tags: case.tags.clone(),
                reason: format!("C4 {}", case.name),
            });
        app.update();

        let after = app
            .world()
            .get::<Pools>(target)
            .and_then(|pools| pools.get(case.kind))
            .map(|pool| pool.current);
        let mut applied_cursor = MessageCursor::<PoolDeltaApplied>::default();
        let new_applied = applied_cursor
            .read(app.world().resource::<Messages<PoolDeltaApplied>>())
            .collect::<Vec<_>>();
        let expected_applied = case.expected_after - case.before;
        if after != Some(case.expected_after)
            || new_applied.len() != 1
            || new_applied[0].target != target
            || new_applied[0].kind != case.kind
            || new_applied[0].before != case.before
            || new_applied[0].after != case.expected_after
            || new_applied[0].amount_applied != expected_applied
            || new_applied[0].tags != case.tags
            || new_applied[0].reason != format!("C4 {}", case.name)
        {
            violations.push(format!(
                "case={} checkpoint=signed-delta expected=(after={},one-applied {}->{},amount={expected_applied},tags={:?}) actual=(after={after:?},applied={new_applied:?})",
                case.name,
                case.expected_after,
                case.before,
                case.expected_after,
                case.tags
            ));
        }
        if case.name == "god-player-negative-health" {
            let armor = app.world().get::<Armor>(target);
            let wounded = app.world().get::<Statuses>(target).is_some_and(|statuses| {
                statuses
                    .instances
                    .iter()
                    .any(|status| status.status_id == "status.wounded")
            });
            let defeated_after = app.world().resource::<Messages<EntityDefeated>>().len();
            if armor.map(|armor| armor.durability) != Some(2)
                || wounded
                || defeated_after != defeated_before
            {
                violations.push(format!(
                    "case={} checkpoint=forbidden-side-effects expected=(armor=2,wounded=false,defeat-delta=0) actual=({:?},{wounded},{})",
                    case.name,
                    armor.map(|armor| armor.durability),
                    defeated_after.saturating_sub(defeated_before)
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "contract=DEBUG-GOD-001 violations:\n{}",
        violations.join("\n")
    );
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FactoryFingerprint {
    name: Option<String>,
    position: Option<Position>,
    player: bool,
    blocks: bool,
    pools: Vec<(String, i32, i32, i32)>,
    statuses: Vec<(String, i32, i32, Option<u64>)>,
    raid_enemy: bool,
    faction: Option<String>,
}

fn factory_fingerprint(world: &World, entity: Entity) -> FactoryFingerprint {
    let record = entity_record(world, entity);
    FactoryFingerprint {
        name: record.name,
        position: record.position,
        player: record.player,
        blocks: record.blocks,
        pools: record.pools,
        statuses: record.statuses,
        raid_enemy: record.raid_enemy,
        faction: record.faction,
    }
}

fn canonical_fingerprint(blueprint: &EntityBlueprint, position: Position) -> FactoryFingerprint {
    let mut app = App::new();
    let entity = spawn_from_blueprint(
        blueprint,
        Some(position),
        &[],
        &mut app.world_mut().commands(),
    );
    app.world_mut().flush();
    factory_fingerprint(app.world(), entity)
}

fn blueprint_rows() -> Vec<(GameMode, Position, EntityBlueprint)> {
    vec![
        (
            GameMode::Tactical,
            Position { x: 8, y: 4 },
            EntityBlueprint {
                id: "blueprint.c4.tactical_sentinel".into(),
                label: "Tactical Sentinel".into(),
                is_player: false,
                blocks_movement: true,
                pools: vec![
                    (PoolKind::Health, 17, 0, 19),
                    (PoolKind::ActionPoints, 2, 0, 5),
                ],
                statuses: vec![("status.guarded".into(), 3)],
                visual: Some("Enemy".into()),
                markers: vec!["RaidEnemy".into()],
            },
        ),
        (
            GameMode::Outpost,
            Position { x: 2, y: 7 },
            EntityBlueprint {
                id: "blueprint.c4.outpost_envoy".into(),
                label: "Outpost Envoy".into(),
                is_player: true,
                blocks_movement: false,
                pools: vec![(PoolKind::Health, 23, 1, 25)],
                statuses: vec![("status.blessed".into(), 4), ("status.poisoned".into(), 2)],
                visual: Some("NPC".into()),
                markers: vec!["FactionMember:broken_choir".into()],
            },
        ),
    ]
}

fn find_named(world: &mut World, label: &str) -> Option<Entity> {
    let mut query = world.query::<(Entity, &Name)>();
    query
        .iter(world)
        .find_map(|(entity, name)| (name.0 == label).then_some(entity))
}

#[test]
fn console_spawn_matches_canonical_factory_fingerprint_for_unlike_blueprints() {
    // Contract: DEBUG-SPAWN-001.
    // Given: unlike Tactical and Outpost blueprints with different pool,
    // status, player/blocking, and marker shapes.
    // When: each is spawned by its production console command.
    // Then: its structured component fingerprint equals the canonical factory.
    // Must not change: catalog data or factory marker/status interpretation.
    //
    // Implementation guidance:
    // - Reusable owner: call `spawn_from_blueprint`; do not reproduce its bundle.
    // - Integration seam: the core debug resolver performs lookup and queues the
    //   canonical factory spawn after the typed request is accepted.
    // - Preserve: every generic catalog entry, not only a rat-shaped fixture.
    // - Invalid shortcuts: copying selected fields/markers or special-casing IDs.
    // - Closing evidence: both rows, the separate scope primary, and factory tests.
    let mut violations = Vec::new();
    for (mode, position, blueprint) in blueprint_rows() {
        let expected = canonical_fingerprint(&blueprint, position);
        let mut app = runtime();
        *app.world_mut().resource_mut::<GameMode>() = mode;
        app.world_mut()
            .insert_resource(BlueprintCatalog::new(vec![blueprint.clone()]));
        submit(
            &mut app,
            &format!("spawn {} {} {}", blueprint.id, position.x, position.y),
        );
        app.update();
        let actual = find_named(app.world_mut(), &blueprint.label)
            .map(|entity| factory_fingerprint(app.world(), entity));
        if actual.as_ref() != Some(&expected) {
            violations.push(format!(
                "case={mode:?}/{} checkpoint=factory-fingerprint expected={expected:?} actual={actual:?}",
                blueprint.id
            ));
        }
    }
    assert!(
        violations.is_empty(),
        "contract=DEBUG-SPAWN-001 violations:\n{}",
        violations.join("\n")
    );
}

#[test]
fn console_spawn_scope_follows_current_game_mode_without_legacy_markers() {
    // Contract: DEBUG-SPAWN-002.
    // Given: the same generic blueprint in Tactical and Outpost mode.
    // When: production console spawning resolves it.
    // Then: Tactical receives DungeonTransient and Outpost receives
    // ColonyPersistent, with no deprecated lifetime marker in either row.
    // Must not change: the canonical action-spawn mode-to-scope rule.
    //
    // Implementation guidance:
    // - Reusable owner: use the established canonical scope mapping.
    // - Integration seam: assign scope to the entity returned by the factory.
    // - Preserve: Foundation EntityScope is authoritative over legacy markers.
    // - Invalid shortcuts: always-persistent scope or dual legacy/new markers.
    // - Closing evidence: run both fresh-mode rows independently.
    let blueprint = EntityBlueprint {
        id: "blueprint.c4.scope".into(),
        label: "C4 Scope Probe".into(),
        is_player: false,
        blocks_movement: false,
        pools: vec![],
        statuses: vec![],
        visual: None,
        markers: vec![],
    };
    let rows = [
        (GameMode::Tactical, EntityScope::DungeonTransient),
        (GameMode::Outpost, EntityScope::ColonyPersistent),
    ];
    let mut violations = Vec::new();
    for (mode, expected_scope) in rows {
        let mut app = runtime();
        *app.world_mut().resource_mut::<GameMode>() = mode;
        app.world_mut()
            .insert_resource(BlueprintCatalog::new(vec![blueprint.clone()]));
        submit(&mut app, "spawn blueprint.c4.scope 9 5");
        app.update();
        let actual = find_named(app.world_mut(), &blueprint.label).map(|entity| {
            (
                app.world().get::<EntityScope>(entity).copied(),
                app.world().get::<PersistentEntity>(entity).is_some(),
                app.world().get::<TransientEntity>(entity).is_some(),
            )
        });
        if actual != Some((Some(expected_scope), false, false)) {
            violations.push(format!(
                "case={mode:?} checkpoint=scope expected=({expected_scope:?},legacy=false/false) actual={actual:?}"
            ));
        }
    }
    assert!(
        violations.is_empty(),
        "contract=DEBUG-SPAWN-002 violations:\n{}",
        violations.join("\n")
    );
}

#[test]
fn unknown_console_blueprint_is_atomic_and_reports_one_rejection() {
    // Contract: DEBUG-SPAWN-003.
    // Given: an enabled console and a catalog without the requested ID.
    // When: a syntactically valid spawn command crosses dispatch.
    // Then: the resolver rejects exactly once, names the ID, and preserves the
    // complete observed world/effect state.
    // Must not change: parsing succeeds and emits the typed request even when
    // domain lookup later rejects it.
    //
    // Implementation guidance:
    // - Reusable owner: catalog validation belongs in the core debug resolver.
    // - Integration seam: one result and one rejected trace return to console.
    // - Preserve: no placeholder entity or partial component bundle.
    // - Invalid shortcuts: rejecting in dispatch before the typed boundary.
    // - Closing evidence: boundary, atomic snapshot, result, trace, and output.
    let mut app = runtime();
    app.world_mut()
        .insert_resource(BlueprintCatalog::new(vec![]));
    let before = snapshot(app.world_mut());
    let trace_start = app.world().resource::<SignalTrace>().entries.len();
    let expected = DebugMutation::SpawnBlueprint {
        blueprint_id: "blueprint.c4.missing".into(),
        position: Position { x: 5, y: 6 },
    };
    submit(&mut app, "spawn blueprint.c4.missing 5 6");
    app.update();

    let observations = app
        .world()
        .resource::<C4BoundaryAudit>()
        .observations
        .clone();
    let after = snapshot(app.world_mut());
    let results = one_result(&app);
    let traces = debug_traces(&app, trace_start);
    let mut violations = Vec::new();
    if observations != [(expected, before.clone())] {
        violations.push(format!(
            "checkpoint=typed-boundary expected=one-request+prestate actual={observations:?}"
        ));
    }
    if after != before {
        violations.push(format!(
            "checkpoint=atomic-state expected={before:?} actual={after:?}"
        ));
    }
    if results.len() != 1
        || results[0].accepted
        || !results[0].message.contains("blueprint.c4.missing")
    {
        violations.push(format!(
            "checkpoint=result expected=one-readable-rejection actual={results:?}"
        ));
    }
    if traces.len() != 1
        || !traces[0].contains("rejected")
        || !traces[0].contains("blueprint.c4.missing")
    {
        violations.push(format!(
            "checkpoint=trace expected=one-readable-rejection actual={traces:?}"
        ));
    }
    if !output_has(&app, "ERROR") || !output_has(&app, "blueprint.c4.missing") {
        violations.push(format!(
            "checkpoint=console-output expected=ERROR+id actual={:?}",
            app.world().resource::<bd_console::ConsoleState>().output
        ));
    }
    assert!(
        violations.is_empty(),
        "contract=DEBUG-SPAWN-003 violations:\n{}",
        violations.join("\n")
    );
}
