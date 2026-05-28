//! Faction system — named groups with territories, dispositions, and themed NPCs.

use bevy::prelude::*;
use rand::RngExt;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// Resource holding all factions in the world.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Resource)]
pub struct Factions(pub Vec<Faction>);

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct Faction {
    pub id: usize,
    pub name: String,
    pub archetype: FactionArchetype,
    pub disposition: FactionDisposition,
    pub home_node: Option<usize>,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum FactionArchetype {
    Puritan,
    Military,
    Commune,
    Cult,
    Traders,
}

impl FactionArchetype {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Puritan => "Puritan",
            Self::Military => "Military",
            Self::Commune => "Commune",
            Self::Cult => "Cult",
            Self::Traders => "Traders",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum FactionDisposition {
    Hostile,
    Neutral,
    Friendly,
}

// ---------------------------------------------------------------------------
// Generation
// ---------------------------------------------------------------------------

const ADJECTIVES: &[&str] = &["Iron", "Crimson", "Silent", "Burning", "Ashen"];
const NOUNS: &[&str] = &["Order", "Pact", "Circle", "Front", "Covenant"];

/// Generate deterministic factions: 3 hardcoded + 2 procedural.
pub fn generate_factions(seed: u64, num_overworld_nodes: usize) -> Factions {
    let mut factions = vec![
        Faction {
            id: 0,
            name: "Michael's Host".to_string(),
            archetype: FactionArchetype::Puritan,
            disposition: FactionDisposition::Hostile,
            home_node: Some(1),
            description: "Religious zealots who see all thaumaturgy as heresy.".to_string(),
        },
        Faction {
            id: 1,
            name: "Fort Pershing".to_string(),
            archetype: FactionArchetype::Military,
            disposition: FactionDisposition::Neutral,
            home_node: Some(2),
            description: "Disciplined soldiers holding a pre-war military installation."
                .to_string(),
        },
        Faction {
            id: 2,
            name: "The Collective".to_string(),
            archetype: FactionArchetype::Commune,
            disposition: FactionDisposition::Friendly,
            home_node: Some(3),
            description: "A cooperative of survivors sharing resources and knowledge.".to_string(),
        },
    ];

    let reserved_nodes: Vec<usize> = vec![0, 1, 2, 3]; // 0=shelter, 1-3=hardcoded factions
    let available_nodes: Vec<usize> = (0..num_overworld_nodes)
        .filter(|n| !reserved_nodes.contains(n))
        .collect();

    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let proc_archetypes = [FactionArchetype::Cult, FactionArchetype::Traders];
    let proc_dispositions = [FactionDisposition::Hostile, FactionDisposition::Neutral];

    for i in 0..2 {
        let adj = ADJECTIVES[rng.random_range(0..ADJECTIVES.len())];
        let noun = NOUNS[rng.random_range(0..NOUNS.len())];
        let name = format!("{adj} {noun}");

        let archetype = proc_archetypes[rng.random_range(0..proc_archetypes.len())];
        let disposition = proc_dispositions[rng.random_range(0..proc_dispositions.len())];

        let home_node = if available_nodes.is_empty() {
            None
        } else {
            Some(available_nodes[rng.random_range(0..available_nodes.len())])
        };

        factions.push(Faction {
            id: 3 + i,
            name,
            archetype,
            disposition,
            description: format!(
                "A {} faction driven by {}.",
                archetype.name().to_lowercase(),
                match archetype {
                    FactionArchetype::Cult => "obsession with thaumic power",
                    FactionArchetype::Traders => "commerce and barter",
                    _ => "unknown motives",
                }
            ),
            home_node,
        });
    }

    Factions(factions)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardcoded_factions() {
        let factions = generate_factions(12345, 10);
        let names: Vec<&str> = factions.0.iter().map(|f| f.name.as_str()).collect();
        assert!(names.contains(&"Michael's Host"));
        assert!(names.contains(&"Fort Pershing"));
        assert!(names.contains(&"The Collective"));
    }

    #[test]
    fn test_faction_count() {
        let factions = generate_factions(99999, 10);
        assert_eq!(factions.0.len(), 5);
    }

    #[test]
    fn test_determinism() {
        let a = generate_factions(42, 10);
        let b = generate_factions(42, 10);
        assert_eq!(a.0.len(), b.0.len());
        for (fa, fb) in a.0.iter().zip(b.0.iter()) {
            assert_eq!(fa.name, fb.name);
            assert_eq!(fa.archetype, fb.archetype);
            assert_eq!(fa.disposition, fb.disposition);
            assert_eq!(fa.home_node, fb.home_node);
        }
    }
}
