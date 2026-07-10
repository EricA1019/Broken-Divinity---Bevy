//! Stations — buildable structures in the shelter (Stove, Altar, Workshop, etc.).
//!
//! Stations are spawned via the `"ability.build"` action, which requires
//! Supplies and a walkable tile.

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    actions::{ActionDefinition, Effect, Requirement},
    signals::{DeltaTag, PoolKind},
};

// ── Pending station build (set by UI, consumed by action system) ──

/// Pending station type to build, set by TUI when cycling station types.
/// Reset to `None` after the build action is processed.
#[derive(Resource, Debug, Clone)]
pub struct PendingStationBuild(pub Option<StationType>);

impl Default for PendingStationBuild {
    fn default() -> Self {
        Self(None)
    }
}

// ── Constants ──

/// Maximum number of stations allowed in the shelter.
pub const MAX_STATIONS: u32 = 20;

/// Supplies needed to build a station.
pub const STATION_BUILD_COST_SUPPLIES: i32 = 2;

// ── Components ──

/// Marker component for station entities.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Station;

/// Type of station, determining its function and production.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StationType {
    Stove,
    Altar,
    Workshop,
    Bed,
    Storage,
}

// ── Blueprint data (Rust fixture, RON later) ──

/// Production and cost data for a station type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StationBlueprint {
    pub station_type: StationType,
    pub label: &'static str,
    pub build_cost_supplies: i32,
    pub produces: Option<(PoolKind, i32)>, // (resource, amount_per_day)
}

/// Default station blueprints.
pub fn default_station_blueprints() -> Vec<StationBlueprint> {
    vec![
        StationBlueprint {
            station_type: StationType::Stove,
            label: "Stove",
            build_cost_supplies: STATION_BUILD_COST_SUPPLIES,
            produces: Some((PoolKind::Supplies, 3)),
        },
        StationBlueprint {
            station_type: StationType::Altar,
            label: "Altar",
            build_cost_supplies: STATION_BUILD_COST_SUPPLIES,
            produces: Some((PoolKind::Faith, 2)),
        },
        StationBlueprint {
            station_type: StationType::Workshop,
            label: "Workshop",
            build_cost_supplies: STATION_BUILD_COST_SUPPLIES,
            produces: Some((PoolKind::Supplies, 2)),
        },
        StationBlueprint {
            station_type: StationType::Bed,
            label: "Bed",
            build_cost_supplies: STATION_BUILD_COST_SUPPLIES,
            produces: None, // restores mood, not resources
        },
        StationBlueprint {
            station_type: StationType::Storage,
            label: "Storage",
            build_cost_supplies: STATION_BUILD_COST_SUPPLIES,
            produces: None, // increases capacity
        },
    ]
}

// ── Action registration ──

/// Register the build action and add station blueprints to the registry.
pub fn register_station_actions() -> ActionDefinition {
    ActionDefinition {
        id: "ability.build".into(),
        label: "Build".into(),
        requirements: vec![
            Requirement::ResourcePoolAbove(PoolKind::Supplies, STATION_BUILD_COST_SUPPLIES),
        ],
        cost_effects: vec![Effect::PoolDelta {
            kind: PoolKind::Supplies,
            amount: -STATION_BUILD_COST_SUPPLIES,
            tags: vec![DeltaTag::Action],
            reason: "build cost".into(),
        }],
        effects: vec![
            Effect::SpawnEntity("blueprint.station".into()),
            Effect::Log("You build a station.".into(), crate::gamelog::LogLevel::Info),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        components::{Player, Position, Tile},
        map::SmokeMap,
        pools::{Pool, Pools},
        signals::ActionIntent,
    };
    use bevy_app::App;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(crate::BdCorePlugin);
        app
    }

    fn spawn_player_with_supplies(app: &mut App, supplies: i32) -> Entity {
        app.world_mut()
            .spawn((
                Player,
                Position { x: 5, y: 5 },
                Pools::new(vec![
                    Pool::new(PoolKind::Supplies, supplies, 0, 50),
                    Pool::new(PoolKind::ActionPoints, 3, 0, 3),
                ]),
            ))
            .id()
    }

    fn send_action(app: &mut App, actor: Entity, action_id: &str, direction: Option<crate::direction::Direction>, target: Option<Entity>) {
        app.world_mut()
            .resource_mut::<bevy_ecs::message::Messages<ActionIntent>>()
            .write(ActionIntent {
                actor,
                action_id: action_id.into(),
                direction,
                target,
            });
    }

    #[test]
    fn station_blueprints_have_correct_defaults() {
        let blueprints = default_station_blueprints();
        assert_eq!(blueprints.len(), 5);
        assert_eq!(blueprints[0].station_type, StationType::Stove);
        assert_eq!(blueprints[0].build_cost_supplies, STATION_BUILD_COST_SUPPLIES);
    }

    #[test]
    fn build_action_has_correct_requirements() {
        let def = register_station_actions();
        assert_eq!(def.id, "ability.build");
        assert!(def.requirements.iter().any(|r| matches!(r, Requirement::ResourcePoolAbove(PoolKind::Supplies, _))));
    }

    #[test]
    fn station_rejected_without_supplies() {
        let mut app = test_app();
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        let p = spawn_player_with_supplies(&mut app, 0); // no supplies
        send_action(&mut app, p, "ability.build", Some(crate::direction::Direction::East), None);
        app.update();
        // Supplies should remain 0 (build was denied)
        let supplies = app.world().get::<Pools>(p).unwrap().get(PoolKind::Supplies).unwrap().current;
        assert_eq!(supplies, 0);
    }

    #[test]
    fn build_consumes_supplies_when_enough() {
        let mut app = test_app();
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        let p = spawn_player_with_supplies(&mut app, 10);
        send_action(&mut app, p, "ability.build", Some(crate::direction::Direction::East), None);
        app.update();
        let supplies = app.world().get::<Pools>(p).unwrap().get(PoolKind::Supplies).unwrap().current;
        // Cost should be deducted
        assert_eq!(supplies, 10 - STATION_BUILD_COST_SUPPLIES);
    }
}
