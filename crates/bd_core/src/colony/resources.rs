//! Resource nodes — harvestable points on the shelter map.
//!
//! Resource nodes are spawned on the shelter map at generation time.
//! Survivors with the Gathering task produce resources when near a node.

use std::collections::HashSet;

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::components::{
    BlocksMovement, ContentIdentity, Name, Position, ResourceNode, ResourceNodeType,
};
use crate::content::{ColonyPlacementProfile, ColonySourceDefinition};
use crate::map::SmokeMap;
use crate::pathfinding::{AStarPathfinder, Pathfinder};
use crate::signals::PoolKind;

pub fn pool_for_node(kind: ResourceNodeType) -> PoolKind {
    match kind {
        ResourceNodeType::Trees => PoolKind::Materials,
        ResourceNodeType::WaterSource => PoolKind::Supplies,
        ResourceNodeType::WildPlants => PoolKind::WildPlants,
    }
}

#[derive(Component, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirectGatherProgress {
    pub definition_id: String,
    pub work_completed: u32,
}

pub fn direct_gather_definition(
    content: &crate::content::FoundationContent,
    output_pool: PoolKind,
) -> Option<&crate::content::DirectGatherDefinition> {
    content
        .colony_gather_tasks
        .iter()
        .find(|definition| definition.output_pool == output_pool)
}

// ── Spawning ──

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceNodePlacement {
    pub source_id: String,
    pub label: String,
    pub kind: ResourceNodeType,
    pub position: Position,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourcePlacementError {
    InvalidProfile(String),
    InvalidSource(String),
    NoCompleteLayout { requested: usize },
}

fn stable_score(seed: u64, salt: u64, source_id: &str, position: Position) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64 ^ seed ^ salt;
    for byte in source_id
        .bytes()
        .chain(position.x.to_le_bytes())
        .chain(position.y.to_le_bytes())
    {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn has_reachable_work_tile(
    map: &SmokeMap,
    origin: Position,
    target: Position,
    blockers: &HashSet<Position>,
) -> bool {
    [
        Position {
            x: target.x,
            y: target.y - 1,
        },
        Position {
            x: target.x,
            y: target.y + 1,
        },
        Position {
            x: target.x - 1,
            y: target.y,
        },
        Position {
            x: target.x + 1,
            y: target.y,
        },
    ]
    .into_iter()
    .any(|work_tile| {
        map.is_walkable(work_tile.x, work_tile.y)
            && !blockers.contains(&work_tile)
            && AStarPathfinder
                .find_path(map, origin, work_tile, blockers)
                .is_some()
    })
}

pub fn plan_resource_nodes(
    map: &SmokeMap,
    origin: Position,
    forbidden: &HashSet<Position>,
    sources: &[ColonySourceDefinition],
    profile: &ColonyPlacementProfile,
    seed: u64,
) -> Result<Vec<ResourceNodePlacement>, ResourcePlacementError> {
    if profile.minimum_manhattan_spacing == 0 {
        return Err(ResourcePlacementError::InvalidProfile(profile.id.clone()));
    }
    if sources
        .iter()
        .any(|source| source.spawn_count == 0 || source.id.trim().is_empty())
    {
        return Err(ResourcePlacementError::InvalidSource(
            sources
                .iter()
                .find(|source| source.spawn_count == 0 || source.id.trim().is_empty())
                .map_or_else(|| "<unknown>".into(), |source| source.id.clone()),
        ));
    }

    let mut source_order = sources.iter().collect::<Vec<_>>();
    source_order.sort_by(|left, right| left.id.cmp(&right.id));
    let requests = source_order
        .iter()
        .flat_map(|source| std::iter::repeat_n(*source, source.spawn_count as usize))
        .collect::<Vec<_>>();
    let mut candidates = (0..map.height)
        .flat_map(|y| (0..map.width).map(move |x| Position { x, y }))
        .filter(|position| map.is_walkable(position.x, position.y) && !forbidden.contains(position))
        .collect::<Vec<_>>();
    candidates.sort_by_key(|position| {
        (
            stable_score(seed, profile.seed_salt, "candidate", *position),
            position.y,
            position.x,
        )
    });

    let mut placements = Vec::with_capacity(requests.len());
    for source in &requests {
        let candidate = candidates.iter().copied().find(|candidate| {
            if placements.iter().any(|placed: &ResourceNodePlacement| {
                (placed.position.x - candidate.x).unsigned_abs()
                    + (placed.position.y - candidate.y).unsigned_abs()
                    < profile.minimum_manhattan_spacing
            }) {
                return false;
            }
            let mut blockers = forbidden.clone();
            blockers.extend(placements.iter().map(|placed| placed.position));
            blockers.insert(*candidate);
            has_reachable_work_tile(map, origin, *candidate, &blockers)
        });
        let Some(position) = candidate else {
            return Err(ResourcePlacementError::NoCompleteLayout {
                requested: requests.len(),
            });
        };
        placements.push(ResourceNodePlacement {
            source_id: source.id.clone(),
            label: source.label.clone(),
            kind: source.node_type,
            position,
        });
    }

    let blockers = forbidden
        .iter()
        .copied()
        .chain(placements.iter().map(|placement| placement.position))
        .collect::<HashSet<_>>();
    if placements
        .iter()
        .any(|placement| !has_reachable_work_tile(map, origin, placement.position, &blockers))
    {
        return Err(ResourcePlacementError::NoCompleteLayout {
            requested: requests.len(),
        });
    }
    placements.sort_by(|left, right| {
        (&left.source_id, left.position.y, left.position.x).cmp(&(
            &right.source_id,
            right.position.y,
            right.position.x,
        ))
    });
    Ok(placements)
}

pub fn spawn_resource_nodes(commands: &mut Commands, placements: &[ResourceNodePlacement]) -> u32 {
    for placement in placements {
        commands.spawn((
            ResourceNode {
                source_id: placement.source_id.clone(),
                kind: placement.kind,
                depleted: false,
            },
            placement.position,
            Name(placement.label.clone()),
            ContentIdentity(placement.source_id.clone()),
            BlocksMovement,
            crate::spatial::EntityScope::ColonyPersistent,
            crate::spatial::PersistentEntity,
        ));
    }
    u32::try_from(placements.len()).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Tile;
    use crate::map::SmokeMap;
    use bevy_app::App;

    #[test]
    fn resource_nodes_spawn_on_walkable_tiles() {
        let mut app = App::new();
        let map = SmokeMap::new(40, 30, Tile::Floor);
        let sources = vec![ColonySourceDefinition {
            id: "source.test".into(),
            label: "Test Source".into(),
            node_type: ResourceNodeType::Trees,
            raw_resource_id: "resource.raw_test".into(),
            spawn_count: 1,
            glyph: 'T',
        }];
        let profile = ColonyPlacementProfile {
            id: "placement.test".into(),
            minimum_manhattan_spacing: 2,
            seed_salt: 7,
        };
        let plan = plan_resource_nodes(
            &map,
            Position { x: 1, y: 1 },
            &HashSet::new(),
            &sources,
            &profile,
            42,
        )
        .unwrap();
        let count = spawn_resource_nodes(&mut app.world_mut().commands(), &plan);
        assert_eq!(count, 1);
        assert!(map.is_walkable(plan[0].position.x, plan[0].position.y));
    }
}
