//! Lore fragments — collectible text items found in dungeons.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::core::components::{Player, Position};
use crate::core::gamelog::{GameLog, LogColor};
use crate::core::turn::GameTime;
use crate::game::dungeon::bsp::Rect;

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// A single lore fragment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoreFragment {
    pub id: usize,
    pub title: String,
    pub text: String,
    pub category: LoreCategory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoreCategory {
    TheSundering,
    Factions,
    Thaumaturgy,
    PreWar,
    Personal,
}

impl LoreCategory {
    pub fn name(&self) -> &'static str {
        match self {
            Self::TheSundering => "The Sundering",
            Self::Factions => "Factions",
            Self::Thaumaturgy => "Thaumaturgy",
            Self::PreWar => "Pre-War",
            Self::Personal => "Personal",
        }
    }
}

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Resource tracking collected lore fragments.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct LoreJournal {
    pub fragments: Vec<LoreFragment>,
}

impl LoreJournal {
    pub fn add(&mut self, fragment: LoreFragment) {
        if !self.fragments.iter().any(|f| f.id == fragment.id) {
            self.fragments.push(fragment);
        }
    }

    pub fn has(&self, id: usize) -> bool {
        self.fragments.iter().any(|f| f.id == id)
    }
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Component for lore item drops in the dungeon.
#[derive(Component, Debug, Clone)]
pub struct LoreDrop {
    pub fragment_id: usize,
}

// ---------------------------------------------------------------------------
// Static data
// ---------------------------------------------------------------------------

/// Static lore fragments available in the game.
pub fn all_fragments() -> &'static [LoreFragment] {
    static FRAGMENTS: std::sync::OnceLock<Vec<LoreFragment>> = std::sync::OnceLock::new();
    FRAGMENTS.get_or_init(|| vec![
        LoreFragment {
            id: 1,
            title: "Shattered Sky".to_string(),
            text: "The sky cracked open on a Tuesday. Nobody agreed which Tuesday.".to_string(),
            category: LoreCategory::TheSundering,
        },
        LoreFragment {
            id: 2,
            title: "Michael's Decree".to_string(),
            text: "All thaumaturgy is heresy. All heresy will be purged.".to_string(),
            category: LoreCategory::Factions,
        },
        LoreFragment {
            id: 3,
            title: "Resonance Notes".to_string(),
            text: "The shimmer responds to intent, not words. Focus matters more than form.".to_string(),
            category: LoreCategory::Thaumaturgy,
        },
        LoreFragment {
            id: 4,
            title: "Lab Report 7-C".to_string(),
            text: "Subjects exposed to Veil energy showed 300% increase in cognitive function, followed by rapid deterioration.".to_string(),
            category: LoreCategory::PreWar,
        },
        LoreFragment {
            id: 5,
            title: "A Torn Letter".to_string(),
            text: "If you're reading this, I didn't make it. Take the medicine to Elena. She'll know what to do.".to_string(),
            category: LoreCategory::Personal,
        },
        LoreFragment {
            id: 6,
            title: "Fort Pershing Communique".to_string(),
            text: "All units: maintain perimeter integrity. Civilian interaction forbidden without clearance.".to_string(),
            category: LoreCategory::Factions,
        },
        LoreFragment {
            id: 7,
            title: "Veil Thickness Report".to_string(),
            text: "Measurements confirm Veil thinning accelerating. Projected full collapse: 18-24 months.".to_string(),
            category: LoreCategory::TheSundering,
        },
        LoreFragment {
            id: 8,
            title: "Scavenger's Journal".to_string(),
            text: "Day 47. Found another anomaly zone. The air tastes like copper and regret.".to_string(),
            category: LoreCategory::Personal,
        },
    ])
}

// ---------------------------------------------------------------------------
// Spawning
// ---------------------------------------------------------------------------

/// Spawn lore drops in dungeon rooms (10% chance per room, excluding room 0).
pub fn spawn_lore_drops(commands: &mut Commands, rooms: &[Rect], rng: &mut impl rand::Rng) {
    use rand::RngExt;
    let fragments = all_fragments();
    for room in rooms.iter().skip(1) {
        if rng.random_range(0..100u32) >= 10 {
            continue;
        }
        let idx = rng.random_range(0..fragments.len());
        let x = rng.random_range(room.x..(room.x + room.w));
        let y = rng.random_range(room.y..(room.y + room.h));
        commands.spawn((
            LoreDrop {
                fragment_id: fragments[idx].id,
            },
            Position { x, y },
        ));
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// System: pick up lore drops when player walks on them.
pub fn pickup_lore(
    mut commands: Commands,
    player_q: Query<&Position, With<Player>>,
    lore_q: Query<(Entity, &LoreDrop, &Position)>,
    mut journal: ResMut<LoreJournal>,
    mut log: ResMut<GameLog>,
    time: Res<GameTime>,
) {
    let Ok(player_pos) = player_q.single() else {
        return;
    };
    for (entity, drop, pos) in lore_q.iter() {
        if pos == player_pos {
            let fragments = all_fragments();
            if let Some(frag) = fragments.iter().find(|f| f.id == drop.fragment_id) {
                journal.add(frag.clone());
                log.push(
                    format!("Found lore: \"{}\"", frag.title),
                    LogColor::Critical,
                    time.turn,
                );
            }
            commands.entity(entity).despawn();
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_journal_add() {
        let mut journal = LoreJournal::default();
        let frag = LoreFragment {
            id: 1,
            title: "Test".to_string(),
            text: "Test text".to_string(),
            category: LoreCategory::TheSundering,
        };
        journal.add(frag);
        assert!(journal.has(1));
        assert_eq!(journal.fragments.len(), 1);
    }

    #[test]
    fn test_journal_no_duplicates() {
        let mut journal = LoreJournal::default();
        let frag = LoreFragment {
            id: 1,
            title: "Test".to_string(),
            text: "Test text".to_string(),
            category: LoreCategory::TheSundering,
        };
        journal.add(frag.clone());
        journal.add(frag);
        assert_eq!(journal.fragments.len(), 1);
    }

    #[test]
    fn test_all_fragments_unique() {
        let fragments = all_fragments();
        let mut ids: Vec<usize> = fragments.iter().map(|f| f.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(
            ids.len(),
            fragments.len(),
            "Duplicate fragment IDs detected"
        );
    }
}
