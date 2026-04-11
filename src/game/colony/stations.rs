//! Station entities — workstations placed in shelter rooms.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::core::components::Position;
use crate::core::resources::{ResourceKind, ShelterResources};
use crate::core::turn::GameTime;

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct Station {
    pub kind: StationType,
    pub tier: u8,
    pub worker_slots: u8,
    pub workers_assigned: u8,
}

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum StationType {
    Workbench,
    Cook,
    Purifier,
    AmmoPress,
    Generator,
    ResearchTable,
    MedicalBay,
    Quarters,
    SecurityCheckpoint,
    MilitiaTraining,
}

impl StationType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Workbench => "Workbench",
            Self::Cook => "Cooking Station",
            Self::Purifier => "Water Purifier",
            Self::AmmoPress => "Ammo Press",
            Self::Generator => "Generator",
            Self::ResearchTable => "Research Table",
            Self::MedicalBay => "Medical Bay",
            Self::Quarters => "Quarters",
            Self::SecurityCheckpoint => "Security Checkpoint",
            Self::MilitiaTraining => "Militia Training",
        }
    }

    /// Resource cost to build this station from scratch.
    pub fn build_cost(&self) -> &[(ResourceKind, u32)] {
        match self {
            Self::Cook => &[(ResourceKind::Scrap, 5)],
            Self::Purifier => &[(ResourceKind::Scrap, 8)],
            Self::Workbench => &[(ResourceKind::Scrap, 10)],
            Self::AmmoPress => &[(ResourceKind::Scrap, 15), (ResourceKind::Ammo, 5)],
            Self::Generator => &[(ResourceKind::Scrap, 20), (ResourceKind::Ammo, 10)],
            Self::MedicalBay => &[(ResourceKind::Scrap, 10), (ResourceKind::Medicine, 5)],
            Self::Quarters => &[(ResourceKind::Scrap, 8)],
            Self::ResearchTable => &[(ResourceKind::Scrap, 25), (ResourceKind::Ammo, 10)],
            Self::SecurityCheckpoint => &[(ResourceKind::Scrap, 12)],
            Self::MilitiaTraining => &[(ResourceKind::Scrap, 15)],
        }
    }

    pub fn worker_slots(&self) -> u8 {
        match self {
            Self::Workbench => 2,
            Self::Quarters => 0,
            _ => 1,
        }
    }

    /// MVP production recipe. Returns `None` for stations with no resource IO.
    pub fn recipe(&self) -> Option<StationRecipe> {
        match self {
            Self::Cook => Some(StationRecipe {
                input: None,
                output: Some((ResourceKind::Food, 1)),
                tick_interval: 1,
            }),
            Self::Purifier => Some(StationRecipe {
                input: None,
                output: Some((ResourceKind::Water, 1)),
                tick_interval: 1,
            }),
            Self::AmmoPress => Some(StationRecipe {
                input: Some((ResourceKind::Scrap, 1)),
                output: Some((ResourceKind::Ammo, 2)),
                tick_interval: 1,
            }),
            Self::MedicalBay => Some(StationRecipe {
                input: None,
                output: Some((ResourceKind::Medicine, 1)),
                tick_interval: 3,
            }),
            // Generator, Workbench, Quarters, etc. — no direct resource IO at MVP.
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Recipe
// ---------------------------------------------------------------------------

/// What a station produces (and consumes) when staffed.
pub struct StationRecipe {
    pub input: Option<(ResourceKind, u32)>,
    pub output: Option<(ResourceKind, u32)>,
    /// How many shelter ticks between each production cycle.
    pub tick_interval: u32,
}

// ---------------------------------------------------------------------------
// Spawn helper
// ---------------------------------------------------------------------------

/// Spawn a station entity at the given grid position.
pub fn spawn_station(commands: &mut Commands, kind: StationType, x: i32, y: i32) -> Entity {
    commands
        .spawn((
            Station {
                kind,
                tier: 1,
                worker_slots: kind.worker_slots(),
                workers_assigned: 0,
            },
            Position::new(x, y),
        ))
        .id()
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Runs each shelter tick: staffed stations apply their recipe.
pub fn station_production(
    stations: Query<&Station>,
    mut resources: ResMut<ShelterResources>,
    time: Res<GameTime>,
) {
    for station in &stations {
        if station.workers_assigned == 0 {
            continue;
        }

        let Some(recipe) = station.kind.recipe() else {
            continue;
        };

        // Respect tick interval (e.g. MedicalBay produces every 3 ticks).
        if !time.turn.is_multiple_of(recipe.tick_interval) {
            continue;
        }

        // Consume input if required; skip production if stockpile insufficient.
        if let Some((res_kind, amount)) = recipe.input
            && !resources.try_consume(res_kind, amount) {
                continue;
            }

        // Produce output.
        if let Some((res_kind, amount)) = recipe.output {
            resources.add(res_kind, amount);
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use bevy::ecs::world::World;

    #[test]
    fn test_station_recipes() {
        // Cook produces 1 food, no input
        let cook = StationType::Cook.recipe().expect("Cook should have recipe");
        assert!(cook.input.is_none());
        assert_eq!(cook.output, Some((ResourceKind::Food, 1)));

        // AmmoPress consumes scrap, produces ammo
        let ammo = StationType::AmmoPress.recipe().expect("AmmoPress should have recipe");
        assert_eq!(ammo.input, Some((ResourceKind::Scrap, 1)));
        assert_eq!(ammo.output, Some((ResourceKind::Ammo, 2)));

        // MedicalBay produces every 3 ticks
        let med = StationType::MedicalBay.recipe().expect("MedicalBay should have recipe");
        assert_eq!(med.tick_interval, 3);
        assert_eq!(med.output, Some((ResourceKind::Medicine, 1)));
    }

    #[test]
    fn test_unstaffed_no_production() {
        let mut world = World::new();
        world.insert_resource(ShelterResources::new_game());
        world.insert_resource(GameTime { turn: 1 });

        // Spawn an unstaffed Cook station.
        world.spawn(Station {
            kind: StationType::Cook,
            tier: 1,
            worker_slots: 1,
            workers_assigned: 0,
        });

        world.run_system_once(station_production);

        let res = world.resource::<ShelterResources>();
        // Food should be unchanged from starting value (10).
        assert_eq!(res.food, 10);
    }

    #[test]
    fn test_staffed_cook_produces_food() {
        let mut world = World::new();
        world.insert_resource(ShelterResources::new_game());
        world.insert_resource(GameTime { turn: 1 });

        world.spawn(Station {
            kind: StationType::Cook,
            tier: 1,
            worker_slots: 1,
            workers_assigned: 1,
        });

        world.run_system_once(station_production);

        let res = world.resource::<ShelterResources>();
        assert_eq!(res.food, 11); // 10 + 1
    }

    #[test]
    fn test_ammo_press_consumes_scrap() {
        let mut world = World::new();
        world.insert_resource(ShelterResources::new_game());
        world.insert_resource(GameTime { turn: 1 });

        world.spawn(Station {
            kind: StationType::AmmoPress,
            tier: 1,
            worker_slots: 1,
            workers_assigned: 1,
        });

        world.run_system_once(station_production);

        let res = world.resource::<ShelterResources>();
        assert_eq!(res.scrap, 14); // 15 - 1
        assert_eq!(res.ammo, 12); // 10 + 2
    }

    #[test]
    fn test_assign_survivor_increments_workers_and_sets_task() {
        use crate::core::gamelog::GameLog;
        use crate::core::stats::EntityName;
        use crate::game::colony::survivors::{Survivor, SurvivorTask};
        use crate::ui::colony_panel::{ColonyUiAction, ColonyUiChoice};

        let mut world = World::new();
        world.insert_resource(GameLog::default());
        world.insert_resource(GameTime { turn: 1 });
        world.init_resource::<ColonyUiAction>();

        let station_entity = world
            .spawn((
                Station {
                    kind: StationType::Cook,
                    tier: 1,
                    worker_slots: 1,
                    workers_assigned: 0,
                },
                Position::new(3, 4),
            ))
            .id();

        let survivor_entity = world
            .spawn((
                Survivor,
                EntityName {
                    name: "Marcus".to_string(),
                },
                SurvivorTask::Idle,
            ))
            .id();

        // Set the action as if the UI wrote it
        world.resource_mut::<ColonyUiAction>().0 =
            Some(ColonyUiChoice::AssignToStation {
                survivor: survivor_entity,
                station: station_entity,
            });

        world.run_system_once(crate::ui::colony_panel::process_colony_action);

        let station = world.entity(station_entity).get::<Station>().unwrap();
        assert_eq!(station.workers_assigned, 1, "workers_assigned should be 1");

        let task = world.entity(survivor_entity).get::<SurvivorTask>().unwrap();
        assert!(
            matches!(task, SurvivorTask::Working(pos) if *pos == IVec2::new(3, 4)),
            "survivor should be Working at station position"
        );
    }

    #[test]
    fn test_unassign_survivor_decrements_workers_and_sets_idle() {
        use crate::core::gamelog::GameLog;
        use crate::core::stats::EntityName;
        use crate::game::colony::survivors::{Survivor, SurvivorTask};
        use crate::ui::colony_panel::{ColonyUiAction, ColonyUiChoice};

        let mut world = World::new();
        world.insert_resource(GameLog::default());
        world.insert_resource(GameTime { turn: 1 });
        world.init_resource::<ColonyUiAction>();

        let station_entity = world
            .spawn((
                Station {
                    kind: StationType::Cook,
                    tier: 1,
                    worker_slots: 1,
                    workers_assigned: 1,
                },
                Position::new(3, 4),
            ))
            .id();

        let survivor_entity = world
            .spawn((
                Survivor,
                EntityName {
                    name: "Elena".to_string(),
                },
                SurvivorTask::Working(IVec2::new(3, 4)),
            ))
            .id();

        world.resource_mut::<ColonyUiAction>().0 =
            Some(ColonyUiChoice::UnassignSurvivor {
                survivor: survivor_entity,
            });

        world.run_system_once(crate::ui::colony_panel::process_colony_action);

        let station = world.entity(station_entity).get::<Station>().unwrap();
        assert_eq!(station.workers_assigned, 0, "workers_assigned should be 0");

        let task = world.entity(survivor_entity).get::<SurvivorTask>().unwrap();
        assert!(
            matches!(task, SurvivorTask::Idle),
            "survivor should be Idle after unassign"
        );

        // Station entity should still exist (not despawned)
        let _ = station_entity;
    }

    #[test]
    fn test_build_cost_all_types() {
        // Every station type should have at least one cost entry
        let all = [
            StationType::Cook,
            StationType::Purifier,
            StationType::Workbench,
            StationType::AmmoPress,
            StationType::Generator,
            StationType::MedicalBay,
            StationType::Quarters,
            StationType::ResearchTable,
            StationType::SecurityCheckpoint,
            StationType::MilitiaTraining,
        ];
        for kind in all {
            assert!(
                !kind.build_cost().is_empty(),
                "{:?} should have at least one build cost",
                kind
            );
        }
    }

    #[test]
    fn test_build_station_consumes_resources_and_spawns() {
        use crate::core::gamelog::GameLog;
        use crate::ui::colony_panel::{ColonyUiAction, ColonyUiChoice};

        let mut world = World::new();
        world.insert_resource(GameLog::default());
        world.insert_resource(GameTime { turn: 1 });
        world.insert_resource(ShelterResources::new_game()); // scrap=15
        world.init_resource::<ColonyUiAction>();

        // Cook costs 5 scrap
        world.resource_mut::<ColonyUiAction>().0 =
            Some(ColonyUiChoice::BuildStation(StationType::Cook));

        world.run_system_once(crate::ui::colony_panel::process_colony_action);

        let res = world.resource::<ShelterResources>();
        assert_eq!(res.scrap, 10, "should consume 5 scrap to build Cook");

        // A new Station entity should exist
        let count = world
            .query::<&Station>()
            .iter(&world)
            .filter(|s| s.kind == StationType::Cook)
            .count();
        assert_eq!(count, 1, "should have spawned a Cook station");
    }

    #[test]
    fn test_build_station_fails_when_insufficient_resources() {
        use crate::core::gamelog::GameLog;
        use crate::ui::colony_panel::{ColonyUiAction, ColonyUiChoice};

        let mut world = World::new();
        world.insert_resource(GameLog::default());
        world.insert_resource(GameTime { turn: 1 });
        // Start with 0 resources
        world.insert_resource(ShelterResources {
            food: 0,
            water: 0,
            scrap: 0,
            medicine: 0,
            ammo: 0,
        });
        world.init_resource::<ColonyUiAction>();

        // Try to build a Cook (costs 5 scrap)
        world.resource_mut::<ColonyUiAction>().0 =
            Some(ColonyUiChoice::BuildStation(StationType::Cook));

        world.run_system_once(crate::ui::colony_panel::process_colony_action);

        let res = world.resource::<ShelterResources>();
        assert_eq!(res.scrap, 0, "scrap should remain 0 when can't afford");

        // No station should have been spawned
        let count = world.query::<&Station>().iter(&world).count();
        assert_eq!(count, 0, "no station should be spawned");

        // Log should contain the failure message
        let log = world.resource::<GameLog>();
        let last = log.entries().last().expect("should have a log entry");
        assert!(
            last.text.contains("Not enough resources"),
            "log should mention insufficient resources, got: {}",
            last.text
        );
    }
}
