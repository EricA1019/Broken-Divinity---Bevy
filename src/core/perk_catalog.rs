//! Perk metadata loaded from RON config.
//!
//! Provides lookup functions for perk names, descriptions, lane labels,
//! and level requirements that were previously hardcoded in match arms.

use std::sync::OnceLock;

use serde::Deserialize;

// ---------------------------------------------------------------------------
// RON schema
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct RonPerkDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub lane_label: String,
    pub required_level: u32,
}

// ---------------------------------------------------------------------------
// Runtime types
// ---------------------------------------------------------------------------

/// Perk metadata loaded from config.
#[derive(Debug, Clone)]
pub struct PerkDef {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub lane_label: &'static str,
    pub required_level: u32,
}

impl PerkDef {
    /// Tier classification matching the original logic: 3→1, 6→2, 9→3.
    pub fn tier(&self) -> u8 {
        match self.required_level {
            3 => 1,
            6 => 2,
            _ => 3,
        }
    }
}

// ---------------------------------------------------------------------------
// Embedded source
// ---------------------------------------------------------------------------

const PERKS_RON_SOURCE: &str = include_str!("../../native/assets/data/perks.ron");

// ---------------------------------------------------------------------------
// Cached catalog
// ---------------------------------------------------------------------------

static PERKS: OnceLock<Vec<PerkDef>> = OnceLock::new();

pub fn all_perk_defs() -> &'static [PerkDef] {
    PERKS.get_or_init(|| {
        let ron_defs: Vec<RonPerkDef> = ron::from_str(PERKS_RON_SOURCE)
            .expect("Failed to parse embedded perks.ron");
        ron_defs
            .into_iter()
            .map(|r| PerkDef {
                id: Box::leak(r.id.into_boxed_str()),
                name: Box::leak(r.name.into_boxed_str()),
                description: Box::leak(r.description.into_boxed_str()),
                lane_label: Box::leak(r.lane_label.into_boxed_str()),
                required_level: r.required_level,
            })
            .collect()
    })
}

/// Look up perk metadata by string ID.
pub fn perk_def(id: &str) -> Option<&'static PerkDef> {
    all_perk_defs().iter().find(|d| d.id == id)
}

/// Get the lane label (virtue + proficiency) for a perk ID.
pub fn perk_lane(id: &str) -> &'static str {
    perk_def(id).map_or("Unknown Lane", |d| d.lane_label)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perks_loads_and_has_expected_count() {
        let defs = all_perk_defs();
        assert_eq!(defs.len(), 12, "All 12 perks must be defined in perks.ron");
    }

    #[test]
    fn test_all_perk_ids_are_unique() {
        let defs = all_perk_defs();
        let mut seen = std::collections::HashSet::new();
        for d in defs {
            assert!(seen.insert(d.id), "Duplicate perk ID: {}", d.id);
        }
    }

    #[test]
    fn test_heavyswing_tier1() {
        let def = perk_def("HeavySwing").expect("HeavySwing should exist");
        assert_eq!(def.name, "Heavy Swing");
        assert_eq!(def.required_level, 3);
        assert_eq!(def.tier(), 1);
    }

    #[test]
    fn test_ironwill_tier3() {
        let def = perk_def("IronWill").expect("IronWill should exist");
        assert_eq!(def.name, "Iron Will");
        assert_eq!(def.required_level, 9);
        assert_eq!(def.tier(), 3);
    }

    #[test]
    fn test_unknown_returns_none() {
        assert!(perk_def("NonexistentPerk").is_none());
    }
}