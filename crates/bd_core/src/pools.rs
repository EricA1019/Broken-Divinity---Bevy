//! Pool system — unified signed delta pipeline for all stat pools.
//!
//! All pool mutation (HP, AP, stress, etc.) goes through `PoolDeltaRequested`
//! → `resolve_pool_deltas` → `PoolDeltaApplied`. No system mutates pools directly.

use bevy_app::App;
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    BdSet,
    gamelog::{GameLog, LogLevel},
    signals::{EntityDefeated, PoolDeltaApplied, PoolDeltaRequested, PoolKind},
    statuses::Statuses,
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
}

// ── Resolver ──

/// The single system that applies all `PoolDeltaRequested` messages.
/// This is the ONLY system that mutates pool values.
fn resolve_pool_deltas(
    _commands: Commands,
    mut requests: bevy_ecs::message::MessageReader<PoolDeltaRequested>,
    mut applied_writer: bevy_ecs::message::MessageWriter<PoolDeltaApplied>,
    mut defeated_writer: bevy_ecs::message::MessageWriter<EntityDefeated>,
    mut game_log: ResMut<GameLog>,
    mut trace: ResMut<SignalTrace>,
    mut query: Query<(Entity, &mut Pools, Option<&Statuses>)>,
) {
    for req in requests.read() {
        trace.push(
            "Mutation",
            "PoolDelta",
            format!("{:?} {:?} {}", req.target, req.kind, req.amount),
        );
        let Ok((entity, mut pools, statuses)) = query.get_mut(req.target) else {
            continue; // target has no Pools component — skip
        };

        let Some(pool) = pools.get_mut(req.kind) else {
            continue; // target doesn't have this pool kind
        };

        // Apply status modifiers (e.g., Guarded halves physical damage)
        let modified_amount = if let Some(statuses) = statuses {
            crate::statuses::apply_modifiers(entity, req.kind, req.amount, &req.tags, statuses)
        } else {
            req.amount
        };

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
            defeated_writer.write(EntityDefeated {
                entity: req.target,
                kind: PoolKind::Health,
            });
            game_log.push(
                format!("Entity {entity:?} has been defeated!"),
                LogLevel::Combat,
            );
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
) {
    for msg in defeated.read() {
        commands.entity(msg.entity).despawn();
    }
}

/// Register the cleanup system.
pub fn register_cleanup(app: &mut bevy_app::App) {
    app.add_systems(
        bevy_app::Update,
        cleanup_defeated_entities.in_set(crate::BdSet::ResultEmission),
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
        assert!(!app.world().entities().contains(e),
            "Entity should be despawned after hitting 0 health");
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
        assert!(!app.world().entities().contains(e),
            "Entity should be despawned after fatal damage");
    }
}
