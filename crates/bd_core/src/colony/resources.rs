//! Resource nodes — harvestable points on the shelter map.
//!
//! Resource nodes are spawned on the shelter map at generation time.
//! Survivors with the Gathering task produce resources when near a node.

use std::collections::HashSet;

use bevy_ecs::prelude::*;

use crate::components::{
    BlocksMovement, ContentIdentity, Name, Position, ResourceNode, ResourceNodeType,
};
use crate::content::{ColonyPlacementProfile, ColonySourceDefinition};
use crate::map::SmokeMap;
use crate::pathfinding::{AStarPathfinder, Pathfinder};
use crate::signals::PoolKind;

// ── Constants ──

/// Legacy non-recipe gathering yield retained only for explicitly assigned
/// `SurvivorTask::Gathering` workers.
pub const GATHERING_YIELD_PER_DAY: i32 = 1;

pub fn pool_for_node(kind: ResourceNodeType) -> PoolKind {
    match kind {
        ResourceNodeType::Trees => PoolKind::Materials,
        ResourceNodeType::WaterSource => PoolKind::Supplies,
        ResourceNodeType::WildPlants => PoolKind::WildPlants,
    }
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

// ── Gathering system ──

/// Process gathering at day change: survivors with Gathering task near resource
/// nodes produce resources into ColonyResources.
#[allow(clippy::type_complexity)]
pub(crate) fn process_survivor_gathering(
    survivors: Query<
        (
            &Position,
            &crate::colony::survivors::SurvivorTask,
            Option<&Name>,
        ),
        (
            With<crate::colony::survivors::Survivor>,
            Without<crate::colony::logistics::LogisticsJob>,
        ),
    >,
    nodes: Query<(&Position, &ResourceNode)>,
    mut colony_res: ResMut<crate::colony::production::ColonyResources>,
    mut days: bevy_ecs::message::MessageReader<crate::time::DayAdvanced>,
    mut game_log: ResMut<crate::gamelog::GameLog>,
    mut draft: ResMut<crate::colony::production::DailyCycleDraft>,
) {
    if days.read().next().is_none() {
        return;
    }

    let node_snapshots = nodes
        .iter()
        .map(
            |(position, node)| crate::colony::production::ResourceWorkSnapshot {
                kind: node.kind,
                position: *position,
                depleted: node.depleted,
            },
        )
        .collect::<Vec<_>>();

    for (pos, task, name) in &survivors {
        let worker = crate::colony::production::SurvivorWorkSnapshot {
            task: task.clone(),
            position: *pos,
        };
        let crate::colony::production::PhysicalWorkEvaluation::Contributes(
            crate::colony::production::PhysicalWorkContribution::Resource(node),
        ) = crate::colony::production::evaluate_physical_work(&worker, &[], &node_snapshots)
        else {
            continue;
        };

        let pool_kind = pool_for_node(node.kind);
        let gathered = if let Some(pool) = colony_res.pools.get_mut(pool_kind) {
            let before = pool.current;
            pool.current = (pool.current + GATHERING_YIELD_PER_DAY).min(pool.max);
            pool.current - before
        } else {
            0
        };
        if let Some(summary) = draft.0.as_mut() {
            summary.gathering_units += 1;
            match pool_kind {
                PoolKind::Supplies => summary.gathered_supplies += gathered,
                PoolKind::Materials => summary.gathered_materials += gathered,
                PoolKind::WildPlants => summary.gathered_wild_plants += gathered,
                _ => {}
            }
        }
        let survivor_name = name.map_or("A survivor", |n| n.0.as_str());
        game_log.push(
            format!(
                "{} gathered 1 {:?} from {:?}.",
                survivor_name, pool_kind, node.kind
            ),
            crate::gamelog::LogLevel::Info,
        );
    }
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
