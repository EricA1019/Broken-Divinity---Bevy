//! Station definitions loaded from RON config.
//!
//! Provides a lookup table for StationType properties that were previously
//! hardcoded in match arms. The StationType enum stays as the variant tag;
//! all per-variant data (build cost, recipe, color, name) is loaded from
//! stations.ron at compile time.

use std::sync::OnceLock;

use bevy::prelude::Color;
use serde::Deserialize;

use super::stations::StationType;
use crate::core::resources::ResourceKind;

// ---------------------------------------------------------------------------
// Loadable schema
// ---------------------------------------------------------------------------

/// A single cost entry for station building.
#[derive(Debug, Clone, Deserialize)]
pub struct CostEntry {
    pub kind: ResourceKind,
    pub amount: u32,
}

/// Input or output resource for a station recipe.
#[derive(Debug, Clone, Deserialize)]
pub struct RecipeIO {
    pub kind: ResourceKind,
    pub amount: u32,
}

/// RON-serializable station recipe. Input is optional (Some = consume, None = free).
#[derive(Debug, Clone, Deserialize)]
pub struct RonStationRecipe {
    pub input: Option<RecipeIO>,
    pub output: RecipeIO,
    pub tick_interval: u32,
}

/// RON-serializable station definition.
#[derive(Debug, Clone, Deserialize)]
pub struct RonStationDef {
    pub kind: String,
    pub name: String,
    pub tier: u8,
    pub worker_slots: u8,
    pub build_cost: Vec<CostEntry>,
    pub recipe: Option<RonStationRecipe>,
    /// RGB tuple (r, g, b) each in 0..1 range
    pub color: (f32, f32, f32),
}

// ---------------------------------------------------------------------------
// Runtime lookup types
// ---------------------------------------------------------------------------

/// Runtime station metadata loaded from config.
#[derive(Debug, Clone)]
pub struct StationDef {
    pub kind: StationType,
    pub name: &'static str,
    pub tier: u8,
    pub worker_slots: u8,
    pub build_cost: Vec<(ResourceKind, u32)>,
    pub recipe: Option<StationRecipeDef>,
    pub color: Color,
}

#[derive(Debug, Clone)]
pub struct StationRecipeDef {
    pub input: Option<(ResourceKind, u32)>,
    pub output: (ResourceKind, u32),
    pub tick_interval: u32,
}

// ---------------------------------------------------------------------------
// Embedded source
// ---------------------------------------------------------------------------

const STATIONS_RON_SOURCE: &str = include_str!("../../../native/assets/data/stations.ron");

// ---------------------------------------------------------------------------
// Cached catalog
// ---------------------------------------------------------------------------

static STATIONS: OnceLock<Vec<StationDef>> = OnceLock::new();

fn parse_station(kind_str: &str) -> StationType {
    match kind_str {
        "Workbench" => StationType::Workbench,
        "Cook" => StationType::Cook,
        "Purifier" => StationType::Purifier,
        "AmmoPress" => StationType::AmmoPress,
        "Generator" => StationType::Generator,
        "ResearchTable" => StationType::ResearchTable,
        "MedicalBay" => StationType::MedicalBay,
        "Quarters" => StationType::Quarters,
        "SecurityCheckpoint" => StationType::SecurityCheckpoint,
        "MilitiaTraining" => StationType::MilitiaTraining,
        other => panic!("Unknown StationType in RON config: {other}"),
    }
}

pub fn all_station_defs() -> &'static [StationDef] {
    STATIONS.get_or_init(|| {
        let ron_defs: Vec<RonStationDef> =
            ron::from_str(STATIONS_RON_SOURCE).expect("Failed to parse embedded stations.ron");
        ron_defs
            .into_iter()
            .map(|r| StationDef {
                kind: parse_station(&r.kind),
                name: Box::leak(r.name.into_boxed_str()),
                tier: r.tier,
                worker_slots: r.worker_slots,
                build_cost: r
                    .build_cost
                    .into_iter()
                    .map(|c| (c.kind, c.amount))
                    .collect(),
                recipe: r.recipe.map(|rec| StationRecipeDef {
                    input: rec.input.map(|io| (io.kind, io.amount)),
                    output: (rec.output.kind, rec.output.amount),
                    tick_interval: rec.tick_interval,
                }),
                color: Color::srgb(r.color.0, r.color.1, r.color.2),
            })
            .collect()
    })
}

/// Look up a station definition by type.
pub fn station_def(kind: StationType) -> Option<&'static StationDef> {
    all_station_defs().iter().find(|d| d.kind == kind)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stations_loads_and_has_expected_count() {
        let defs = all_station_defs();
        assert_eq!(
            defs.len(),
            10,
            "All 10 station types must be defined in stations.ron"
        );
    }

    #[test]
    fn test_all_station_types_have_unique_kinds() {
        let defs = all_station_defs();
        let mut seen = std::collections::HashSet::new();
        for d in defs {
            assert!(seen.insert(d.kind), "Duplicate station kind: {:?}", d.kind);
        }
    }

    #[test]
    fn test_cook_station_config() {
        let def = station_def(StationType::Cook).expect("Cook station should exist");
        assert_eq!(def.name, "Cooking Station");
        assert_eq!(def.worker_slots, 1);
        assert!(def.recipe.is_some());
        let recipe = def.recipe.as_ref().unwrap();
        assert!(recipe.input.is_none());
        assert_eq!(recipe.output.0, ResourceKind::Food);
        assert_eq!(recipe.output.1, 1);
        assert_eq!(recipe.tick_interval, 1);
    }

    #[test]
    fn test_ammo_press_consumes_scrap() {
        let def = station_def(StationType::AmmoPress).expect("AmmoPress should exist");
        let recipe = def.recipe.as_ref().expect("AmmoPress should have a recipe");
        let input = recipe
            .input
            .as_ref()
            .expect("AmmoPress should consume scrap");
        assert_eq!(input.0, ResourceKind::Scrap);
        assert_eq!(input.1, 1);
        assert_eq!(recipe.output.0, ResourceKind::Ammo);
        assert_eq!(recipe.output.1, 2);
    }
}
