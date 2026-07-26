//! Resource nodes — harvestable points on the shelter map.
//!
//! Resource nodes are spawned on the shelter map at generation time.
//! Survivors with the Gathering task produce resources when near a node.

use bevy_ecs::prelude::*;

use crate::components::{Name, Position, ResourceNode, ResourceNodeType};
use crate::map::SmokeMap;
use crate::signals::PoolKind;

// ── Constants ──

/// Minimum number of resource nodes to spawn on the shelter map.
pub const RESOURCE_NODE_COUNT_MIN: u32 = 4;

/// Maximum number of resource nodes to spawn on the shelter map.
pub const RESOURCE_NODE_COUNT_MAX: u32 = 6;

/// Manhattan distance within which a survivor can gather from a node.
pub const GATHERING_RANGE: i32 = 3;

/// Base resources produced per survivor per day when gathering.
pub const GATHERING_YIELD_PER_DAY: i32 = 1;

pub fn pool_for_node(kind: ResourceNodeType) -> PoolKind {
    match kind {
        ResourceNodeType::Trees => PoolKind::Materials,
        ResourceNodeType::WaterSource => PoolKind::Supplies,
        ResourceNodeType::WildPlants => PoolKind::WildPlants,
    }
}

// ── Spawning ──

/// Spawn resource nodes on the shelter map at walkable positions.
/// Returns the number of nodes spawned.
pub fn spawn_resource_nodes(commands: &mut Commands, map: &SmokeMap) -> u32 {
    // Use a simple deterministic seed derived from map dimensions
    let seed = (map.width as u64)
        .wrapping_mul(2654435761)
        .wrapping_add(map.height as u64);
    let count = RESOURCE_NODE_COUNT_MIN
        + (seed % (RESOURCE_NODE_COUNT_MAX - RESOURCE_NODE_COUNT_MIN + 1) as u64) as u32;

    let node_types = [
        ResourceNodeType::Trees,
        ResourceNodeType::WaterSource,
        ResourceNodeType::WildPlants,
    ];

    let mut spawned = 0u32;
    let mut attempt = 0u64;

    while spawned < count && attempt < 200 {
        let x =
            1 + ((seed.wrapping_add(attempt.wrapping_mul(127))) % (map.width as u64 - 2)) as i32;
        let y =
            1 + ((seed.wrapping_add(attempt.wrapping_mul(313))) % (map.height as u64 - 2)) as i32;

        if map.is_walkable(x, y) {
            let kind = node_types[(attempt % 3) as usize];
            let name = match kind {
                ResourceNodeType::Trees => "Trees",
                ResourceNodeType::WaterSource => "Water Source",
                ResourceNodeType::WildPlants => "Wild Plants",
            };
            commands.spawn((
                ResourceNode {
                    kind,
                    depleted: false,
                },
                Position { x, y },
                Name(name.into()),
                crate::spatial::EntityScope::ColonyPersistent,
            ));
            spawned += 1;
        }
        attempt += 1;
    }

    spawned
}

// ── Gathering system ──

/// Process gathering at day change: survivors with Gathering task near resource
/// nodes produce resources into ColonyResources.
pub(crate) fn process_survivor_gathering(
    survivors: Query<
        (
            &Position,
            &crate::colony::survivors::SurvivorTask,
            Option<&Name>,
        ),
        With<crate::colony::survivors::Survivor>,
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
        let count = spawn_resource_nodes(&mut app.world_mut().commands(), &map);
        assert!(
            count >= RESOURCE_NODE_COUNT_MIN,
            "Should spawn at least {} nodes, got {}",
            RESOURCE_NODE_COUNT_MIN,
            count
        );
        assert!(
            count <= RESOURCE_NODE_COUNT_MAX,
            "Should spawn at most {} nodes, got {}",
            RESOURCE_NODE_COUNT_MAX,
            count
        );
    }
}
