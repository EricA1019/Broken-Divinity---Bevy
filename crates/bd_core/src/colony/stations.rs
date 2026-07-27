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
/// Foundation fallback when validated station content is unavailable.
pub const STATION_CONSTRUCTION_WORK_TURNS: u32 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StationPlacementDenial {
    NotWalkable,
    Occupied,
    WouldBlockShelterEgress,
    NoReachableWorkTile,
}

impl fmt::Display for StationPlacementDenial {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::NotWalkable => "Tile is not walkable",
            Self::Occupied => "Tile is occupied",
            Self::WouldBlockShelterEgress => "Would block shelter egress",
            Self::NoReachableWorkTile => "No reachable adjacent work tile",
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
    let has_reachable_work_tile = [
        Position {
            x: candidate.x + 1,
            y: candidate.y,
        },
        Position {
            x: candidate.x - 1,
            y: candidate.y,
        },
        Position {
            x: candidate.x,
            y: candidate.y + 1,
        },
        Position {
            x: candidate.x,
            y: candidate.y - 1,
        },
    ]
    .into_iter()
    .filter(|work_tile| map.is_walkable(work_tile.x, work_tile.y) && !blockers.contains(work_tile))
    .any(|work_tile| {
        AStarPathfinder
            .find_path(map, player, work_tile, &blockers)
            .is_some()
    });
    if !has_reachable_work_tile {
        return Err(StationPlacementDenial::NoReachableWorkTile);
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

/// Paid station footprint that is not operational until idle workers finish it.
#[derive(Component, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstructionSite {
    pub work_completed: u32,
    pub work_required: u32,
}

impl ConstructionSite {
    pub fn new(work_required: u32) -> Self {
        Self {
            work_completed: 0,
            work_required,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.work_completed >= self.work_required
    }
}

/// Marks an idle survivor currently controlled by automatic construction.
#[derive(Component, Debug, Clone, Copy)]
pub struct AutoConstructing;

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
    pub construction_work_turns: u32,
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
    ProcessRecipes,
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
            StationEffect::ProcessRecipes => "Refines assigned colony recipes".into(),
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
            construction_work_turns: STATION_CONSTRUCTION_WORK_TURNS,
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
            construction_work_turns: STATION_CONSTRUCTION_WORK_TURNS,
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
            construction_work_turns: STATION_CONSTRUCTION_WORK_TURNS,
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
            construction_work_turns: STATION_CONSTRUCTION_WORK_TURNS,
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
            construction_work_turns: STATION_CONSTRUCTION_WORK_TURNS,
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

/// Route genuinely idle survivors to paid construction sites and apply one
/// construction work unit per worker tick while cardinally adjacent.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub(crate) fn process_idle_construction(
    mut commands: Commands,
    mut workers: Query<
        (
            Entity,
            Option<&crate::components::Name>,
            &mut Position,
            &crate::colony::survivors::SurvivorTask,
        ),
        (
            With<crate::colony::survivors::Survivor>,
            Without<crate::colony::logistics::LogisticsJob>,
        ),
    >,
    mut sites: Query<
        (
            Entity,
            &Position,
            Option<&crate::components::Name>,
            &mut ConstructionSite,
        ),
        (With<Station>, Without<crate::colony::survivors::Survivor>),
    >,
    stations: Query<&Position, (With<Station>, Without<crate::colony::survivors::Survivor>)>,
    nodes: Query<
        &Position,
        (
            With<crate::components::ResourceNode>,
            Without<crate::colony::survivors::Survivor>,
        ),
    >,
    player: Query<
        &Position,
        (
            With<crate::components::Player>,
            Without<crate::colony::survivors::Survivor>,
        ),
    >,
    map: Res<SmokeMap>,
    mode: Res<crate::spatial::GameMode>,
    time_plan: Res<crate::time::TimeAdvancePlan>,
    mut game_log: ResMut<crate::gamelog::GameLog>,
) {
    if *mode != crate::spatial::GameMode::Outpost {
        return;
    }

    let mut order = workers
        .iter()
        .map(|(entity, name, _, _)| {
            (
                name.map_or_else(
                    || format!("Survivor {}", entity.to_bits()),
                    |name| name.0.clone(),
                ),
                entity,
            )
        })
        .collect::<Vec<_>>();
    order.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));

    if time_plan.outpost_worker_steps == 0 {
        return;
    }

    for _ in 0..time_plan.outpost_worker_steps {
        let mut site_targets = sites
            .iter()
            .filter(|(_, _, _, site)| !site.is_complete())
            .map(|(entity, position, name, _)| {
                (
                    entity,
                    crate::colony::survivors::WorkTarget {
                        label: name.map_or_else(
                            || "Construction site".into(),
                            |name| format!("{} construction", name.0),
                        ),
                        position: *position,
                    },
                )
            })
            .collect::<Vec<_>>();
        site_targets.sort_by(|left, right| {
            (
                left.1.position.y,
                left.1.position.x,
                left.1.label.as_str(),
                left.0.to_bits(),
            )
                .cmp(&(
                    right.1.position.y,
                    right.1.position.x,
                    right.1.label.as_str(),
                    right.0.to_bits(),
                ))
        });

        if site_targets.is_empty() {
            for (_, entity) in &order {
                commands.entity(*entity).remove::<AutoConstructing>();
            }
            break;
        }

        let permanent = stations
            .iter()
            .copied()
            .chain(nodes.iter().copied())
            .chain(player.iter().copied())
            .collect::<HashSet<_>>();
        let mut occupied = workers
            .iter()
            .map(|(_, _, position, _)| *position)
            .collect::<HashSet<_>>();
        let targets = site_targets
            .iter()
            .map(|(_, target)| target.clone())
            .collect::<Vec<_>>();

        for (_, entity) in &order {
            let Ok((_, _, mut position, task)) = workers.get_mut(*entity) else {
                continue;
            };
            if *task != crate::colony::survivors::SurvivorTask::Idle {
                commands.entity(*entity).remove::<AutoConstructing>();
                continue;
            }

            let mut blocked = permanent.clone();
            blocked.extend(occupied.iter().copied());
            blocked.remove(&*position);
            match crate::colony::survivors::choose_worker_path(&map, *position, &targets, &blocked)
            {
                Ok((target, path)) if path.len() <= 1 => {
                    let Some((site_entity, _)) = site_targets
                        .iter()
                        .find(|(_, candidate)| candidate.position == target.position)
                    else {
                        continue;
                    };
                    if let Ok((_, _, _, mut site)) = sites.get_mut(*site_entity)
                        && !site.is_complete()
                    {
                        site.work_completed = site.work_completed.saturating_add(1);
                        if site.is_complete() {
                            site.work_completed = site.work_required;
                            commands.entity(*site_entity).remove::<ConstructionSite>();
                            game_log.push(
                                format!("{} is complete.", target.label),
                                crate::gamelog::LogLevel::Info,
                            );
                        }
                    }
                    commands.entity(*entity).insert((
                        AutoConstructing,
                        crate::colony::survivors::WorkerActivity::Working {
                            target: target.label,
                            target_position: target.position,
                        },
                    ));
                }
                Ok((target, path)) => {
                    let before = *position;
                    *position = path[1];
                    occupied.remove(&before);
                    occupied.insert(*position);
                    commands.entity(*entity).insert((
                        AutoConstructing,
                        crate::colony::survivors::WorkerActivity::EnRoute {
                            target: target.label,
                            target_position: target.position,
                            distance: i32::try_from(path.len().saturating_sub(2))
                                .unwrap_or(i32::MAX),
                        },
                    ));
                }
                Err(reason) => {
                    let target = targets.first();
                    commands.entity(*entity).insert((
                        AutoConstructing,
                        crate::colony::survivors::WorkerActivity::Blocked {
                            target: target.map_or_else(
                                || "Construction site".into(),
                                |target| target.label.clone(),
                            ),
                            target_position: target.map(|target| target.position),
                            reason,
                        },
                    ));
                }
            }
        }
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

    #[test]
    fn placement_rejects_station_without_a_reachable_adjacent_work_tile() {
        let mut map = SmokeMap::new(7, 5, Tile::Wall);
        for position in [
            Position { x: 1, y: 1 },
            Position { x: 2, y: 1 },
            Position { x: 3, y: 1 },
            Position { x: 5, y: 3 },
        ] {
            map.set(position.x, position.y, Tile::Floor);
        }
        let player = Position { x: 1, y: 1 };
        let gate = Position { x: 3, y: 1 };
        let candidate = Position { x: 5, y: 3 };

        assert_eq!(
            validate_station_placement(&map, player, gate, &HashSet::new(), candidate),
            Err(StationPlacementDenial::NoReachableWorkTile),
            "contract=COLONY-SPATIAL-002 case=isolated-candidate \
             fixture=colony_build_distant_invalid expected typed rejection \
             without changing the still-open player-to-gate route"
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
