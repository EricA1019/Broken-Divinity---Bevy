//! Stations — buildable structures in the shelter (Stove, Altar, Workshop, etc.).
//!
//! Stations are spawned via the `"ability.build"` action, which requires
//! Supplies and a walkable tile.

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fmt};

use crate::{
    actions::{ActionDefinition, Effect, Requirement},
    components::Position,
    map::SmokeMap,
    pathfinding::{AStarPathfinder, Pathfinder},
    signals::PoolKind,
};

/// Explicit station selected by colony management for the next assignment.
#[derive(Resource, Debug, Default, Clone)]
pub struct PendingStationAssignment(pub Option<Entity>);

// ── Constants ──

/// Maximum number of stations allowed in the shelter.
pub const MAX_STATIONS: u32 = 20;

/// Supplies needed to build a station.
pub const STATION_BUILD_COST_SUPPLIES: i32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StationPlacementDenial {
    NotWalkable,
    Occupied,
    WouldBlockShelterEgress,
}

impl fmt::Display for StationPlacementDenial {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::NotWalkable => "Tile is not walkable",
            Self::Occupied => "Tile is occupied",
            Self::WouldBlockShelterEgress => "Would block shelter egress",
        })
    }
}

/// Validate one station footprint before payment or mutation.
///
/// Only permanent blockers belong in `permanent_blockers`; survivor motion is
/// deliberately excluded so a temporary worker position cannot make a legal
/// construction permanently invalid.
pub fn validate_station_placement(
    map: &SmokeMap,
    player: Position,
    gate: Position,
    permanent_blockers: &HashSet<Position>,
    candidate: Position,
) -> Result<(), StationPlacementDenial> {
    if !map.is_walkable(candidate.x, candidate.y) {
        return Err(StationPlacementDenial::NotWalkable);
    }
    if permanent_blockers.contains(&candidate) {
        return Err(StationPlacementDenial::Occupied);
    }

    let mut blockers = permanent_blockers.clone();
    blockers.insert(candidate);
    if AStarPathfinder
        .find_path(map, player, gate, &blockers)
        .is_none()
    {
        return Err(StationPlacementDenial::WouldBlockShelterEgress);
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildInteractionDenial {
    Placement(StationPlacementDenial),
    NotEnoughSupplies,
    StationUnavailable,
    UnknownSelection,
}

impl fmt::Display for BuildInteractionDenial {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Placement(reason) => reason.fmt(formatter),
            Self::NotEnoughSupplies => formatter.write_str("Not enough Supplies"),
            Self::StationUnavailable => formatter.write_str("Station is unavailable"),
            Self::UnknownSelection => formatter.write_str("Unknown station selection"),
        }
    }
}

/// Sole transient authority for the build selection, preview, and resolution
/// transaction. It is deliberately excluded from persistence.
#[derive(Resource, Debug, Clone, PartialEq, Eq, Default)]
pub enum BuildInteraction {
    #[default]
    Inactive,
    Selecting {
        selected_station: StationType,
    },
    Placing {
        selected_station: StationType,
        cursor: Position,
        validation: Result<(), BuildInteractionDenial>,
    },
    AwaitingResolution {
        selected_station: StationType,
        cursor: Position,
    },
}

impl BuildInteraction {
    pub fn selected_station(&self) -> Option<StationType> {
        match self {
            Self::Inactive => None,
            Self::Selecting { selected_station }
            | Self::Placing {
                selected_station, ..
            }
            | Self::AwaitingResolution {
                selected_station, ..
            } => Some(*selected_station),
        }
    }

    pub fn is_active(&self) -> bool {
        !matches!(self, Self::Inactive)
    }
}

// ── Components ──

/// Marker component for station entities.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Station;

/// Type of station, determining its function and production.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StationType {
    Stove,
    Altar,
    Workshop,
    Bed,
    Storage,
    /// Extension slot for data-defined station records.
    Custom(u16),
}

// ── Authoritative station content ──

/// Production and cost data for a station type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StationBlueprint {
    pub id: String,
    pub station_type: StationType,
    pub label: String,
    pub build_cost_supplies: i32,
    pub description: String,
    /// Monochrome glyph while no survivor is assigned.
    pub glyph: char,
    /// Explicit monochrome glyph while at least one survivor is assigned.
    pub staffed_glyph: char,
    pub effect: StationEffect,
    pub staffing_required: bool,
    pub buildable: bool,
    pub unavailable_reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StationEffect {
    Produce { kind: PoolKind, amount: i32 },
    RestoreWorkerMood { amount: i32 },
    Disabled,
}

impl StationBlueprint {
    pub fn effect_label(&self) -> String {
        match self.effect {
            StationEffect::Produce { kind, amount } => {
                format!("+{amount} {kind:?}/day when staffed")
            }
            StationEffect::RestoreWorkerMood { amount } => {
                format!("+{amount} Mood/day to assigned worker")
            }
            StationEffect::Disabled => format!(
                "Disabled — {}",
                self.unavailable_reason
                    .as_deref()
                    .unwrap_or("No Foundation effect yet")
            ),
        }
    }
}

/// Runtime owner for all station simulation and presentation facts.
#[derive(Resource, Debug, Clone)]
pub struct StationCatalog {
    entries: Vec<StationBlueprint>,
}

impl StationCatalog {
    pub fn new(entries: Vec<StationBlueprint>) -> Self {
        Self { entries }
    }

    pub fn entries(&self) -> &[StationBlueprint] {
        &self.entries
    }

    pub fn get(&self, station_type: StationType) -> Option<&StationBlueprint> {
        self.entries
            .iter()
            .find(|entry| entry.station_type == station_type)
    }

    pub fn buildable(&self) -> impl Iterator<Item = &StationBlueprint> {
        self.entries.iter().filter(|entry| entry.buildable)
    }
}

impl Default for StationCatalog {
    fn default() -> Self {
        Self::new(default_station_blueprints())
    }
}

/// Test/legacy fixture used when no application content has been installed.
///
/// The shipping application replaces this resource with validated RON data.
pub fn default_station_blueprints() -> Vec<StationBlueprint> {
    vec![
        StationBlueprint {
            id: "station.stove".into(),
            station_type: StationType::Stove,
            label: "Stove".into(),
            build_cost_supplies: STATION_BUILD_COST_SUPPLIES,
            description: "Prepares daily shelter Supplies.".into(),
            glyph: 'f',
            staffed_glyph: 'F',
            effect: StationEffect::Produce {
                kind: PoolKind::Supplies,
                amount: 3,
            },
            staffing_required: true,
            buildable: true,
            unavailable_reason: None,
        },
        StationBlueprint {
            id: "station.altar".into(),
            station_type: StationType::Altar,
            label: "Altar".into(),
            build_cost_supplies: STATION_BUILD_COST_SUPPLIES,
            description: "Builds daily Faith through observance.".into(),
            glyph: 'a',
            staffed_glyph: 'A',
            effect: StationEffect::Produce {
                kind: PoolKind::Faith,
                amount: 2,
            },
            staffing_required: true,
            buildable: true,
            unavailable_reason: None,
        },
        StationBlueprint {
            id: "station.workshop".into(),
            station_type: StationType::Workshop,
            label: "Workshop".into(),
            build_cost_supplies: STATION_BUILD_COST_SUPPLIES,
            description: "Produces repair and construction Materials.".into(),
            glyph: 'w',
            staffed_glyph: 'W',
            effect: StationEffect::Produce {
                kind: PoolKind::Materials,
                amount: 2,
            },
            staffing_required: true,
            buildable: true,
            unavailable_reason: None,
        },
        StationBlueprint {
            id: "station.bed".into(),
            station_type: StationType::Bed,
            label: "Bed".into(),
            build_cost_supplies: STATION_BUILD_COST_SUPPLIES,
            description: "Restores the assigned survivor's Mood.".into(),
            glyph: 'b',
            staffed_glyph: 'B',
            effect: StationEffect::RestoreWorkerMood {
                amount: crate::colony::survivors::MOOD_REST_BONUS,
            },
            staffing_required: true,
            buildable: true,
            unavailable_reason: None,
        },
        StationBlueprint {
            id: "station.storage".into(),
            station_type: StationType::Storage,
            label: "Storage".into(),
            build_cost_supplies: STATION_BUILD_COST_SUPPLIES,
            description: "Reserved for later storage-capacity rules.".into(),
            glyph: 's',
            staffed_glyph: 'S',
            effect: StationEffect::Disabled,
            staffing_required: false,
            buildable: false,
            unavailable_reason: Some("No Foundation effect yet".into()),
        },
    ]
}

pub fn station_catalog() -> Vec<StationBlueprint> {
    default_station_blueprints()
}

// ── Action registration ──

/// Register the build action and add station blueprints to the registry.
pub fn register_station_actions() -> ActionDefinition {
    ActionDefinition {
        id: "ability.build".into(),
        label: "Build".into(),
        requirements: vec![Requirement::TileVacant],
        cost_effects: vec![],
        effects: vec![Effect::SpawnEntity("blueprint.station".into())],
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
    use std::collections::HashSet;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(crate::BdCorePlugin);
        app
    }

    fn spawn_player_with_colony_supplies(app: &mut App, supplies: i32) -> Entity {
        app.world_mut()
            .resource_mut::<crate::colony::production::ColonyResources>()
            .pools
            .get_mut(PoolKind::Supplies)
            .expect("test colony supplies must exist")
            .current = supplies;
        app.world_mut()
            .spawn((
                Player,
                Position { x: 5, y: 5 },
                Pools::new(vec![Pool::new(PoolKind::ActionPoints, 3, 0, 3)]),
            ))
            .id()
    }

    #[test]
    fn placement_rejects_candidate_that_removes_last_gate_path() {
        let map = crate::colony::shelter::create_shelter_map();
        let player = crate::colony::shelter::SHELTER_RETURN_SPAWN;
        let gate = Position {
            x: crate::colony::shelter::SHELTER_WIDTH / 2,
            y: 1,
        };
        let blockers = HashSet::from([Position { x: 1, y: 2 }]);

        let result =
            validate_station_placement(&map, player, gate, &blockers, Position { x: 2, y: 1 });

        assert_eq!(result, Err(StationPlacementDenial::WouldBlockShelterEgress));
    }

    #[test]
    fn placement_accepts_candidate_when_another_gate_path_remains() {
        let map = crate::colony::shelter::create_shelter_map();
        let player = crate::colony::shelter::SHELTER_RETURN_SPAWN;
        let gate = Position {
            x: crate::colony::shelter::SHELTER_WIDTH / 2,
            y: 1,
        };

        let result = validate_station_placement(
            &map,
            player,
            gate,
            &HashSet::new(),
            Position { x: 2, y: 1 },
        );

        assert_eq!(result, Ok(()));
    }

    #[test]
    fn placement_rejects_non_walkable_and_occupied_candidates_with_typed_reasons() {
        let map = crate::colony::shelter::create_shelter_map();
        let player = crate::colony::shelter::SHELTER_RETURN_SPAWN;
        let gate = Position {
            x: crate::colony::shelter::SHELTER_WIDTH / 2,
            y: 1,
        };
        let occupied = HashSet::from([Position { x: 2, y: 1 }]);

        assert_eq!(
            validate_station_placement(&map, player, gate, &occupied, Position { x: 2, y: 1 },),
            Err(StationPlacementDenial::Occupied)
        );
        assert_eq!(
            validate_station_placement(
                &map,
                player,
                gate,
                &HashSet::new(),
                Position { x: 0, y: 1 },
            ),
            Err(StationPlacementDenial::NotWalkable)
        );
    }

    fn send_action(
        app: &mut App,
        actor: Entity,
        action_id: &str,
        direction: Option<crate::direction::Direction>,
        target: Option<Entity>,
    ) {
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
        assert_eq!(
            blueprints[0].build_cost_supplies,
            STATION_BUILD_COST_SUPPLIES
        );
        let storage = station_catalog()
            .into_iter()
            .find(|entry| entry.station_type == StationType::Storage)
            .unwrap();
        assert!(!storage.buildable);
        assert_eq!(storage.effect, StationEffect::Disabled);
    }

    #[test]
    fn build_action_leaves_catalog_cost_to_the_build_validator() {
        let def = register_station_actions();
        assert_eq!(def.id, "ability.build");
        assert!(def.cost_effects.is_empty());
        assert_eq!(def.requirements.len(), 1);
        assert!(matches!(def.requirements[0], Requirement::TileVacant));
    }

    #[test]
    fn station_rejected_without_supplies() {
        let mut app = test_app();
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        let p = spawn_player_with_colony_supplies(&mut app, 0);
        send_action(
            &mut app,
            p,
            "ability.build",
            Some(crate::direction::Direction::East),
            None,
        );
        app.update();
        // Supplies should remain 0 (build was denied)
        let supplies = app
            .world()
            .resource::<crate::colony::production::ColonyResources>()
            .pools
            .get(PoolKind::Supplies)
            .unwrap()
            .current;
        assert_eq!(supplies, 0);
    }

    #[test]
    fn build_consumes_supplies_when_enough() {
        let mut app = test_app();
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        let p = spawn_player_with_colony_supplies(&mut app, 10);
        send_action(
            &mut app,
            p,
            "ability.build",
            Some(crate::direction::Direction::East),
            None,
        );
        app.update();
        let supplies = app
            .world()
            .resource::<crate::colony::production::ColonyResources>()
            .pools
            .get(PoolKind::Supplies)
            .unwrap()
            .current;
        // Cost should be deducted
        assert_eq!(supplies, 10 - STATION_BUILD_COST_SUPPLIES);
    }

    #[test]
    fn build_interaction_default_is_inactive() {
        let state = BuildInteraction::default();
        assert_eq!(state, BuildInteraction::Inactive);
        assert!(!state.is_active());
        assert_eq!(state.selected_station(), None);
    }

    #[test]
    fn build_interaction_placing_retains_selection_cursor_and_validation() {
        let state = BuildInteraction::Placing {
            selected_station: StationType::Stove,
            cursor: Position { x: 10, y: 5 },
            validation: Ok(()),
        };
        assert!(state.is_active());
        assert_eq!(state.selected_station(), Some(StationType::Stove));
    }
}
