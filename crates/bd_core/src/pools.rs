//! Pool system — unified signed delta pipeline for all stat pools.
//!
//! All pool mutation (HP, AP, stress, etc.) goes through `PoolDeltaRequested`
//! → `resolve_pool_deltas` → `PoolDeltaApplied`. No system mutates pools directly.

use std::collections::HashSet;

use bevy_app::App;
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    BdSet,
    combat::CombatRng,
    gamelog::{GameLog, LogLevel},
    signals::{
        DeltaTag, EntityDefeated, EntityMoved, PoolDeltaApplied, PoolDeltaRequested, PoolKind,
    },
    statuses::Statuses,
    time::TurnJustAdvanced,
    trace::SignalTrace,
};

// ── Component ──

/// A single pool with current/min/max bounds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pool {
    pub kind: PoolKind,
    pub current: i32,
    pub min: i32,
    pub max: i32,
}

impl Pool {
    pub fn new(kind: PoolKind, current: i32, min: i32, max: i32) -> Self {
        Self {
            kind,
            current,
            min,
            max,
        }
    }

    /// Apply a delta, returning the actual amount applied (after clamping).
    pub fn apply_delta(&mut self, delta: i32) -> i32 {
        let before = self.current;
        self.current = (self.current + delta).clamp(self.min, self.max);
        self.current - before
    }
}

/// Collection of pools on an entity.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Pools {
    pools: Vec<Pool>,
}

impl Pools {
    pub fn new(pools: Vec<Pool>) -> Self {
        Self { pools }
    }

    /// Get a pool by kind.
    pub fn get(&self, kind: PoolKind) -> Option<&Pool> {
        self.pools.iter().find(|p| p.kind == kind)
    }

    /// Iterate over all pools.
    pub fn iter(&self) -> impl Iterator<Item = &Pool> {
        self.pools.iter()
    }

    /// Iterate mutably over all pools.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Pool> {
        self.pools.iter_mut()
    }

    /// Get mutable access to a pool by kind.
    pub fn get_mut(&mut self, kind: PoolKind) -> Option<&mut Pool> {
        self.pools.iter_mut().find(|p| p.kind == kind)
    }
}

// ── Plugin registration ──

pub(crate) fn register_pools(app: &mut App) {
    app.add_message::<PoolDeltaRequested>();
    app.add_message::<PoolDeltaApplied>();
    app.add_message::<EntityDefeated>();

    app.add_systems(
        bevy_app::Update,
        resolve_pool_deltas.in_set(BdSet::Mutation),
    );

    app.add_systems(
        bevy_app::Update,
        log_combat_damage.in_set(BdSet::ResultEmission),
    );

    // P21: AP regeneration at turn start for ALL entities
    app.add_systems(
        bevy_app::Update,
        regenerate_action_points.in_set(BdSet::Input),
    );
}

// ── AP Regeneration ──

/// Restores ActionPoints to max for every entity when a turn advances.
/// Consumes the TurnJustAdvanced signal so it only fires once per turn.
fn regenerate_action_points(
    turn_signal: Option<Res<TurnJustAdvanced>>,
    mut commands: Commands,
    mut query: Query<&mut Pools>,
) {
    if turn_signal.is_none() {
        return;
    }
    for mut pools in &mut query {
        if let Some(ap) = pools.get_mut(PoolKind::ActionPoints) {
            ap.current = ap.max;
        }
    }
    commands.remove_resource::<TurnJustAdvanced>();
}

// ── Resolver ──

/// The single system that applies all `PoolDeltaRequested` messages.
/// This is the ONLY system that mutates pool values.
fn resolve_pool_deltas(
    mut commands: Commands,
    mut combat_rng: Option<ResMut<CombatRng>>,
    mut requests: bevy_ecs::message::MessageReader<PoolDeltaRequested>,
    mut applied_writer: bevy_ecs::message::MessageWriter<PoolDeltaApplied>,
    mut defeated_writer: bevy_ecs::message::MessageWriter<EntityDefeated>,
    mut trace: ResMut<SignalTrace>,
    mut query: Query<(
        Entity,
        &mut Pools,
        Option<&Statuses>,
        Option<&mut crate::combat::Armor>,
    )>,
) {
    let mut defeated_this_resolution: HashSet<Entity> = HashSet::new();
    for req in requests.read() {
        if defeated_this_resolution.contains(&req.target) {
            continue;
        }
        trace.push(
            "Mutation",
            "PoolDelta",
            format!("{:?} {:?} {}", req.target, req.kind, req.amount),
        );
        let Ok((entity, mut pools, statuses, armor)) = query.get_mut(req.target) else {
            continue; // target has no Pools component — skip
        };

        let Some(pool) = pools.get_mut(req.kind) else {
            continue; // target doesn't have this pool kind
        };

        // Apply status modifiers (e.g., Guarded halves physical damage)
        let mut modified_amount = if let Some(statuses) = statuses {
            crate::statuses::apply_modifiers(entity, req.kind, req.amount, &req.tags, statuses)
        } else {
            req.amount
        };

        // P13-A: Apply d100 damage variance to Health deltas with combat damage tags
        if req.kind == PoolKind::Health
            && modified_amount < 0
            && req.tags.iter().any(|t| {
                matches!(
                    t,
                    DeltaTag::Physical | DeltaTag::Ballistic | DeltaTag::Slash
                )
            })
        {
            modified_amount =
                CombatRng::apply_damage_variance(modified_amount, combat_rng.as_deref_mut());
        }

        // P13-C: Armor reduces physical/ballistic/slash damage
        if modified_amount < 0
            && req.kind == PoolKind::Health
            && req.tags.iter().any(|t| {
                matches!(
                    t,
                    DeltaTag::Physical | DeltaTag::Ballistic | DeltaTag::Slash
                )
            })
        {
            if let Some(mut armor) = armor {
                if armor.durability > 0 && armor.reduction > 0 {
                    let absorbed = armor.reduction.min(-modified_amount);
                    modified_amount += absorbed;
                    armor.durability -= 1;
                    tracing::debug!(
                        "Armor absorbed {} damage (durability: {})",
                        absorbed,
                        armor.durability
                    );
                }
            }
        }

        let before = pool.current;
        let amount_applied = pool.apply_delta(modified_amount);
        let after = pool.current;

        // Emit applied event
        applied_writer.write(PoolDeltaApplied {
            source: req.source,
            target: req.target,
            kind: req.kind,
            before,
            after,
            amount_applied,
            tags: req.tags.clone(),
            reason: req.reason.clone(),
        });

        // Check threshold: entity defeated (health at min)
        if req.kind == PoolKind::Health && after <= pool.min {
            defeated_this_resolution.insert(req.target);
            defeated_writer.write(EntityDefeated {
                entity: req.target,
                kind: PoolKind::Health,
            });
        }

        // P13-E: Apply Wounded status when HP drops below threshold
        if req.kind == PoolKind::Health
            && after > pool.min
            && after <= pool.max * crate::combat::WOUND_THRESHOLD_PCT / 100
        {
            let defs = crate::statuses::default_status_definitions();
            crate::statuses::apply_status(entity, "status.wounded", 0, None, &mut commands, &defs);
        }

        tracing::debug!(
            "PoolDelta: {:?} {:?} {}→{} (applied {}) — {}",
            entity,
            req.kind,
            before,
            after,
            amount_applied,
            req.reason
        );
    }
}

// ── Entity cleanup ──

/// Observes EntityDefeated messages and despawns the defeated entities.
/// Generic: handles any entity regardless of how it was spawned.
fn cleanup_defeated_entities(
    mut defeated: bevy_ecs::message::MessageReader<EntityDefeated>,
    mut commands: Commands,
    mut game_log: ResMut<GameLog>,
    names: Query<&crate::components::Name>,
) {
    for msg in defeated.read() {
        // Log defeat with entity name
        let entity_name = names
            .get(msg.entity)
            .map(|n| n.0.as_str())
            .unwrap_or("An enemy");
        game_log.push(format!("{entity_name} is defeated!"), LogLevel::Combat);
        commands.entity(msg.entity).despawn();
    }
}

/// Observe PoolDeltaApplied messages and log combat damage amounts.
/// Does NOT modify resolve_pool_deltas — separate observer (SRP).
fn log_combat_damage(
    mut applied: bevy_ecs::message::MessageReader<PoolDeltaApplied>,
    mut game_log: ResMut<GameLog>,
    names: Query<&crate::components::Name>,
) {
    for msg in applied.read() {
        if msg.kind != PoolKind::Health {
            continue;
        }
        let amount = msg.amount_applied;
        if amount >= 0 {
            continue; // healing, not damage
        }
        let target_name = names
            .get(msg.target)
            .map(|n| n.0.as_str())
            .unwrap_or("target");
        game_log.push(
            format!("{target_name} takes {} damage!", -amount),
            LogLevel::Combat,
        );
    }
}

/// Register the cleanup system.
pub fn register_cleanup(app: &mut bevy_app::App) {
    app.add_systems(
        bevy_app::Update,
        (observe_player_defeat, cleanup_defeated_entities)
            .chain()
            .in_set(crate::BdSet::ResultEmission),
    );
}

/// Observes EntityDefeated messages and switches to GameOver when the player dies.
/// Runs before `cleanup_defeated_entities` (chained above) so the Player component
/// is still readable before the entity is despawned.
fn observe_player_defeat(
    mut defeated: bevy_ecs::message::MessageReader<EntityDefeated>,
    player: Query<(), With<crate::components::Player>>,
    mut mode: ResMut<crate::spatial::GameMode>,
    mut session: ResMut<crate::session::RunSession>,
) {
    for msg in defeated.read() {
        if player.get(msg.entity).is_ok() {
            *mode = crate::spatial::GameMode::GameOver;
            session.mark_defeated();
        }
    }
}

// ── Movement feedback ──

/// Consume MoveBlocked messages and push to the game log.
fn log_move_blocked(
    mut blocked: bevy_ecs::message::MessageReader<crate::signals::MoveBlocked>,
    mut game_log: ResMut<GameLog>,
) {
    for msg in blocked.read() {
        game_log.push(format!("Blocked: {:?}", msg.reason), LogLevel::Warn);
    }
}

/// Observe EntityMoved messages and log items at the destination.
/// Player walks onto an item tile → log the item name.
fn log_item_pickup(
    mut moved: bevy_ecs::message::MessageReader<EntityMoved>,
    mut game_log: ResMut<GameLog>,
    player: Query<(), With<crate::components::Player>>,
    items_at_pos: Query<
        (&crate::components::Name, &crate::components::Position),
        With<crate::inventory::Item>,
    >,
) {
    for msg in moved.read() {
        // Only log pickups for player movement
        if player.get(msg.entity).is_err() {
            continue;
        }
        // Check if any item is at the destination position
        for (item_name, _pos) in items_at_pos.iter() {
            if *_pos == msg.to {
                game_log.push(format!("You found a {}!", item_name.0), LogLevel::Info);
            }
        }
    }
}

/// Register the move-blocked logging system.
pub fn register_move_feedback(app: &mut bevy_app::App) {
    app.add_systems(
        bevy_app::Update,
        (log_move_blocked, log_item_pickup).in_set(crate::BdSet::ResultEmission),
    );
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Player;
    use bevy_app::App;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(crate::BdCorePlugin);
        app
    }

    fn send_delta(app: &mut App, target: Entity, kind: PoolKind, amount: i32) {
        app.world_mut()
            .resource_mut::<bevy_ecs::message::Messages<PoolDeltaRequested>>()
            .write(PoolDeltaRequested {
                source: None,
                target,
                kind,
                amount,
                tags: vec![],
                reason: "test".into(),
            });
    }

    fn spawn_with_pools(app: &mut App, hp: i32, ap: i32) -> Entity {
        app.world_mut()
            .spawn((
                Player,
                Pools::new(vec![
                    Pool::new(PoolKind::Health, hp, 0, 20),
                    Pool::new(PoolKind::ActionPoints, ap, 0, 3),
                ]),
            ))
            .id()
    }

    fn get_pool(app: &App, entity: Entity, kind: PoolKind) -> i32 {
        app.world()
            .get::<Pools>(entity)
            .unwrap()
            .get(kind)
            .unwrap()
            .current
    }

    // ── Tests ──

    #[test]
    fn health_negative_delta_reduces_health() {
        let mut app = test_app();
        let e = spawn_with_pools(&mut app, 20, 3);
        send_delta(&mut app, e, PoolKind::Health, -8);
        app.update();
        assert_eq!(get_pool(&app, e, PoolKind::Health), 12);
    }

    #[test]
    fn health_positive_delta_increases_health() {
        let mut app = test_app();
        let e = spawn_with_pools(&mut app, 10, 3);
        send_delta(&mut app, e, PoolKind::Health, 5);
        app.update();
        assert_eq!(get_pool(&app, e, PoolKind::Health), 15);
    }

    #[test]
    fn ap_negative_delta_spends_ap() {
        let mut app = test_app();
        let e = spawn_with_pools(&mut app, 20, 3);
        send_delta(&mut app, e, PoolKind::ActionPoints, -1);
        app.update();
        assert_eq!(get_pool(&app, e, PoolKind::ActionPoints), 2);
    }

    #[test]
    fn ap_positive_delta_restores_ap() {
        let mut app = test_app();
        let e = spawn_with_pools(&mut app, 20, 1);
        send_delta(&mut app, e, PoolKind::ActionPoints, 2);
        app.update();
        assert_eq!(get_pool(&app, e, PoolKind::ActionPoints), 3);
    }

    #[test]
    fn pool_delta_clamps_to_min() {
        let mut app = test_app();
        let e = spawn_with_pools(&mut app, 2, 3);
        send_delta(&mut app, e, PoolKind::Health, -10);
        app.update();
        // Health hit min which triggers EntityDefeated, entity cleaned up
        assert!(
            !app.world().entities().contains(e),
            "Entity should be despawned after hitting 0 health"
        );
    }

    #[test]
    fn pool_delta_clamps_to_max() {
        let mut app = test_app();
        let e = spawn_with_pools(&mut app, 18, 3);
        send_delta(&mut app, e, PoolKind::Health, 10);
        app.update();
        assert_eq!(get_pool(&app, e, PoolKind::Health), 20); // clamped to max
    }

    #[test]
    fn zero_delta_does_not_emit_threshold() {
        let mut app = test_app();
        let e = spawn_with_pools(&mut app, 20, 3);
        send_delta(&mut app, e, PoolKind::Health, 0);
        app.update();
        // Entity should still be alive, no defeated message
        let defeated_count = app
            .world()
            .resource::<bevy_ecs::message::Messages<EntityDefeated>>()
            .len();
        assert_eq!(defeated_count, 0);
    }

    #[test]
    fn player_defeat_triggers_game_over() {
        let mut app = test_app();
        let player = spawn_with_pools(&mut app, 2, 3); // HP=2
        // Set a gameplay mode so the test isn't starting from Title
        app.world_mut()
            .insert_resource(crate::spatial::GameMode::Tactical);

        // Deal fatal damage
        send_delta(&mut app, player, PoolKind::Health, -10);

        app.update();

        // Player should be despawned and mode should be GameOver
        assert!(
            !app.world().entities().contains(player),
            "player should be despawned on defeat"
        );
        let mode = *app.world().resource::<crate::spatial::GameMode>();
        assert_eq!(
            mode,
            crate::spatial::GameMode::GameOver,
            "defeating the player should switch mode to GameOver"
        );
    }

    #[test]
    fn fatal_health_emits_entity_defeated() {
        let mut app = test_app();
        let e = spawn_with_pools(&mut app, 5, 3);
        send_delta(&mut app, e, PoolKind::Health, -100);
        app.update();
        // Defeated message should have been emitted
        let defeated_count = app
            .world()
            .resource::<bevy_ecs::message::Messages<EntityDefeated>>()
            .len();
        assert!(defeated_count > 0, "Expected EntityDefeated message");
        // Entity should have been cleaned up
        assert!(
            !app.world().entities().contains(e),
            "Entity should be despawned after fatal damage"
        );
    }

    #[test]
    fn move_blocked_logs_message() {
        let mut app = test_app();
        use crate::components::{Player, Position, Tile};
        use crate::map::SmokeMap;
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        app.world_mut()
            .resource_scope(|world, mut map: Mut<SmokeMap>| {
                map.set(6, 5, Tile::Wall);
            });
        let p = app
            .world_mut()
            .spawn((
                Player,
                Position { x: 5, y: 5 },
                Pools::new(vec![Pool::new(PoolKind::ActionPoints, 3, 0, 3)]),
            ))
            .id();
        app.world_mut()
            .resource_mut::<bevy_ecs::message::Messages<crate::signals::ActionIntent>>()
            .write(crate::signals::ActionIntent {
                actor: p,
                action_id: "ability.move".into(),
                direction: Some(crate::direction::Direction::East),
                target: None,
            });
        app.update();
        let log = app.world().resource::<GameLog>();
        let has_blocked = log.iter().any(|e| e.message.contains("Blocked"));
        assert!(has_blocked, "Expected 'Blocked' in log");
    }

    #[test]
    fn damage_is_logged_to_combat_channel() {
        let mut app = test_app();
        use crate::components::Name;
        // Spawn a target that will take damage
        let target = app
            .world_mut()
            .spawn((
                Name("Rat".into()),
                Pools::new(vec![Pool::new(PoolKind::Health, 10, 0, 10)]),
            ))
            .id();
        // Deal 5 damage
        send_delta(&mut app, target, PoolKind::Health, -5);
        app.update();

        let log = app.world().resource::<GameLog>();
        let has_damage = log
            .iter()
            .any(|e| e.message.contains("5") && e.message.contains("Rat"));
        assert!(
            has_damage,
            "Damage log should contain amount and target name, got: {:?}",
            log.iter().map(|e| &e.message).collect::<Vec<_>>()
        );
    }

    #[test]
    fn defeat_is_logged_to_combat_channel() {
        let mut app = test_app();
        use crate::components::Name;
        // Spawn enemy with Name and minimal Health
        let e = app
            .world_mut()
            .spawn((
                Name("Rat".into()),
                Pools::new(vec![Pool::new(PoolKind::Health, 1, 0, 5)]),
            ))
            .id();
        // Deal lethal damage
        send_delta(&mut app, e, PoolKind::Health, -5);
        app.update();
        let log = app.world().resource::<GameLog>();
        let has_defeat = log
            .iter()
            .any(|e| e.message.contains("Rat") && e.message.contains("defeated"));
        assert!(
            has_defeat,
            "Defeat log should mention 'Rat' and 'defeated', got: {:?}",
            log.iter().map(|e| &e.message).collect::<Vec<_>>()
        );
        // Entity should be despawned
        assert!(
            !app.world().entities().contains(e),
            "Entity should be despawned after defeat"
        );
    }

    #[test]
    fn lethal_damage_blocks_later_queued_damage_and_defeat_results() {
        let mut app = test_app();
        use crate::components::Name;
        let e = app
            .world_mut()
            .spawn((
                Name("Rat".into()),
                Pools::new(vec![Pool::new(PoolKind::Health, 5, 0, 5)]),
            ))
            .id();

        send_delta(&mut app, e, PoolKind::Health, -5);
        send_delta(&mut app, e, PoolKind::Health, -5);
        app.update();

        let defeat_count = app
            .world()
            .resource::<bevy_ecs::message::Messages<EntityDefeated>>()
            .len();
        assert_eq!(
            defeat_count, 1,
            "lethal damage should emit one defeat result"
        );
        let defeat_logs = app
            .world()
            .resource::<GameLog>()
            .iter()
            .filter(|entry| entry.message.contains("defeated"))
            .count();
        assert_eq!(
            defeat_logs, 1,
            "lethal damage should produce one defeat log"
        );
        assert!(!app.world().entities().contains(e));
    }
}
