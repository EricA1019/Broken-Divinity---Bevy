//! Research system — tech tree unlocked via the Research Table station.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::core::gamelog::{GameLog, LogColor};
use crate::core::resources::ResourceKind;
use crate::core::turn::GameTime;
use crate::game::colony::stations::{Station, StationType};

// ---------------------------------------------------------------------------
// Data
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResearchProject {
    ImprovedCooking,
    WaterFiltration,
    ReinforcedDefenses,
    AdvancedMedicine,
}

impl ResearchProject {
    pub fn name(&self) -> &'static str {
        match self {
            Self::ImprovedCooking => "Improved Cooking",
            Self::WaterFiltration => "Water Filtration",
            Self::ReinforcedDefenses => "Reinforced Defenses",
            Self::AdvancedMedicine => "Advanced Medicine",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::ImprovedCooking => "Cook station produces 2 food instead of 1",
            Self::WaterFiltration => "Purifier produces 2 water instead of 1",
            Self::ReinforcedDefenses => "Security Checkpoint defense value doubles",
            Self::AdvancedMedicine => "Medical Bay produces 2 medicine per cycle",
        }
    }

    pub fn cost(&self) -> (ResourceKind, u32) {
        (ResourceKind::Scrap, self.scrap_cost())
    }

    pub fn scrap_cost(&self) -> u32 {
        match self {
            Self::ImprovedCooking => 15,
            Self::WaterFiltration => 15,
            Self::ReinforcedDefenses => 20,
            Self::AdvancedMedicine => 25,
        }
    }

    pub fn ticks_to_complete(&self) -> u32 {
        match self {
            Self::ImprovedCooking => 10,
            Self::WaterFiltration => 10,
            Self::ReinforcedDefenses => 15,
            Self::AdvancedMedicine => 20,
        }
    }

    /// All projects in display order.
    pub const ALL: &'static [ResearchProject] = &[
        ResearchProject::ImprovedCooking,
        ResearchProject::WaterFiltration,
        ResearchProject::ReinforcedDefenses,
        ResearchProject::AdvancedMedicine,
    ];
}

// ---------------------------------------------------------------------------
// Resource
// ---------------------------------------------------------------------------

/// Tracks completed and in-progress research for the colony.
#[derive(Resource, Default, Debug, Clone, Serialize, Deserialize)]
pub struct CompletedResearch {
    #[serde(default)]
    pub completed: Vec<ResearchProject>,
    #[serde(default)]
    pub active: Option<(ResearchProject, u32)>,
}

impl CompletedResearch {
    pub fn is_completed(&self, project: ResearchProject) -> bool {
        self.completed.contains(&project)
    }

    pub fn is_available(&self, project: ResearchProject) -> bool {
        !self.is_completed(project) && self.active.is_none_or(|(active, _)| active != project)
    }

    /// Production multiplier for a station kind based on completed research.
    pub fn production_multiplier(&self, kind: StationType) -> u32 {
        match kind {
            StationType::Cook if self.is_completed(ResearchProject::ImprovedCooking) => 2,
            StationType::Purifier if self.is_completed(ResearchProject::WaterFiltration) => 2,
            StationType::MedicalBay if self.is_completed(ResearchProject::AdvancedMedicine) => 2,
            _ => 1,
        }
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Each colony tick: if a ResearchTable is staffed and research is active,
/// decrement ticks_remaining. When it hits 0, complete the research.
pub fn tick_research(
    stations: Query<&Station>,
    mut research: ResMut<CompletedResearch>,
    mut log: ResMut<GameLog>,
    time: Res<GameTime>,
) {
    // Check that at least one ResearchTable is staffed.
    let has_staffed_table = stations
        .iter()
        .any(|s| s.kind == StationType::ResearchTable && s.workers_assigned > 0);

    if !has_staffed_table {
        return;
    }

    let Some((project, ticks_remaining)) = research.active.as_mut() else {
        return;
    };

    if *ticks_remaining <= 1 {
        let project = *project;
        let name = project.name();
        research.completed.push(project);
        research.active = None;
        log.push(
            format!("Research complete: {name}!"),
            LogColor::PlayerHit,
            time.turn,
        );
    } else {
        *ticks_remaining -= 1;
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

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(CompletedResearch::default());
        world.insert_resource(GameLog::default());
        world.insert_resource(GameTime { turn: 1 });
        world
    }

    #[test]
    fn test_research_ticks_down_when_staffed() {
        let mut world = setup_world();

        // Start a research project with 3 ticks remaining
        world.resource_mut::<CompletedResearch>().active =
            Some((ResearchProject::ImprovedCooking, 3));

        // Spawn a staffed ResearchTable
        world.spawn(Station {
            kind: StationType::ResearchTable,
            tier: 1,
            worker_slots: 1,
            workers_assigned: 1,
        });

        let _ = world.run_system_once(tick_research);

        let research = world.resource::<CompletedResearch>();
        assert_eq!(
            research.active,
            Some((ResearchProject::ImprovedCooking, 2)),
            "ticks_remaining should decrement from 3 to 2"
        );
        assert!(research.completed.is_empty(), "should not be completed yet");
    }

    #[test]
    fn test_research_completes_at_one_tick() {
        let mut world = setup_world();

        world.resource_mut::<CompletedResearch>().active =
            Some((ResearchProject::WaterFiltration, 1));

        world.spawn(Station {
            kind: StationType::ResearchTable,
            tier: 1,
            worker_slots: 1,
            workers_assigned: 1,
        });

        let _ = world.run_system_once(tick_research);

        let research = world.resource::<CompletedResearch>();
        assert!(research.active.is_none(), "active should be cleared");
        assert!(
            research.is_completed(ResearchProject::WaterFiltration),
            "project should be in completed list"
        );

        let log = world.resource::<GameLog>();
        let last = log.entries().last().expect("should have log entry");
        assert!(
            last.text.contains("Research complete"),
            "log should mention completion, got: {}",
            last.text
        );
    }

    #[test]
    fn test_research_no_progress_when_unstaffed() {
        let mut world = setup_world();

        world.resource_mut::<CompletedResearch>().active =
            Some((ResearchProject::ImprovedCooking, 5));

        // Spawn an unstaffed ResearchTable
        world.spawn(Station {
            kind: StationType::ResearchTable,
            tier: 1,
            worker_slots: 1,
            workers_assigned: 0,
        });

        let _ = world.run_system_once(tick_research);

        let research = world.resource::<CompletedResearch>();
        assert_eq!(
            research.active,
            Some((ResearchProject::ImprovedCooking, 5)),
            "ticks should not change when unstaffed"
        );
    }

    #[test]
    fn test_research_no_progress_without_table() {
        let mut world = setup_world();

        world.resource_mut::<CompletedResearch>().active =
            Some((ResearchProject::ImprovedCooking, 5));

        // No ResearchTable at all — just a Cook
        world.spawn(Station {
            kind: StationType::Cook,
            tier: 1,
            worker_slots: 1,
            workers_assigned: 1,
        });

        let _ = world.run_system_once(tick_research);

        let research = world.resource::<CompletedResearch>();
        assert_eq!(research.active, Some((ResearchProject::ImprovedCooking, 5)),);
    }

    #[test]
    fn test_completed_research_boosts_production() {
        use crate::core::resources::ShelterResources;
        use crate::game::colony::stations::station_production;

        let mut world = World::new();
        world.insert_resource(ShelterResources::new_game());
        world.insert_resource(GameTime { turn: 1 });

        let mut research = CompletedResearch::default();
        research.completed.push(ResearchProject::ImprovedCooking);
        world.insert_resource(research);

        // Spawn a staffed Cook
        world.spawn(Station {
            kind: StationType::Cook,
            tier: 1,
            worker_slots: 1,
            workers_assigned: 1,
        });

        let _ = world.run_system_once(station_production);

        let res = world.resource::<ShelterResources>();
        assert_eq!(
            res.food, 12,
            "Cook should produce 2 food with ImprovedCooking (10 + 2)"
        );
    }

    #[test]
    fn test_production_multiplier() {
        let mut r = CompletedResearch::default();
        assert_eq!(r.production_multiplier(StationType::Cook), 1);

        r.completed.push(ResearchProject::ImprovedCooking);
        assert_eq!(r.production_multiplier(StationType::Cook), 2);
        assert_eq!(r.production_multiplier(StationType::Purifier), 1);

        r.completed.push(ResearchProject::WaterFiltration);
        assert_eq!(r.production_multiplier(StationType::Purifier), 2);
    }

    #[test]
    fn test_is_available() {
        let mut r = CompletedResearch::default();
        assert!(r.is_available(ResearchProject::ImprovedCooking));

        // Mark active
        r.active = Some((ResearchProject::ImprovedCooking, 5));
        assert!(!r.is_available(ResearchProject::ImprovedCooking));
        assert!(r.is_available(ResearchProject::WaterFiltration));

        // Complete it
        r.completed.push(ResearchProject::ImprovedCooking);
        r.active = None;
        assert!(!r.is_available(ResearchProject::ImprovedCooking));
    }

    #[test]
    fn test_serde_roundtrip() {
        let mut r = CompletedResearch::default();
        r.completed.push(ResearchProject::ImprovedCooking);
        r.active = Some((ResearchProject::WaterFiltration, 7));

        let json = serde_json::to_string(&r).unwrap();
        let loaded: CompletedResearch = serde_json::from_str(&json).unwrap();

        assert_eq!(loaded.completed.len(), 1);
        assert!(loaded.is_completed(ResearchProject::ImprovedCooking));
        assert_eq!(loaded.active, Some((ResearchProject::WaterFiltration, 7)));
    }
}
