//! Overworld node graph generation.

use bevy::prelude::*;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

use crate::game::dungeon::theme::DungeonTheme;

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum NodeType {
    Shelter,
    Dungeon,
    Ruins,
    Crossroads,
    Landmark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum DungeonStoryTag {
    GabrielIntro,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverworldNode {
    pub id: usize,
    pub node_type: NodeType,
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub discovered: bool,
    pub dungeon_theme: Option<DungeonTheme>,
    #[serde(default)]
    pub story_tag: Option<DungeonStoryTag>,
}

/// An edge (road) between two nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Road {
    pub from: usize,
    pub to: usize,
    pub distance: f32,
}

/// The complete overworld graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverworldGraph {
    pub nodes: Vec<OverworldNode>,
    pub roads: Vec<Road>,
}

impl OverworldGraph {
    /// Get ids of nodes connected to `node_id`.
    pub fn neighbors(&self, node_id: usize) -> Vec<usize> {
        let mut out = Vec::new();
        for road in &self.roads {
            if road.from == node_id {
                out.push(road.to);
            } else if road.to == node_id {
                out.push(road.from);
            }
        }
        out
    }

    /// Get road between two nodes if it exists.
    pub fn road_between(&self, a: usize, b: usize) -> Option<&Road> {
        self.roads
            .iter()
            .find(|r| (r.from == a && r.to == b) || (r.from == b && r.to == a))
    }

    /// Get node by id.
    pub fn node(&self, id: usize) -> Option<&OverworldNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn ensure_story_tags(&mut self) {
        if self
            .nodes
            .iter()
            .any(|node| node.story_tag == Some(DungeonStoryTag::GabrielIntro))
        {
            return;
        }

        let Some(shelter) = self
            .nodes
            .iter()
            .find(|node| node.node_type == NodeType::Shelter)
        else {
            return;
        };

        let Some(node_id) = self
            .nodes
            .iter()
            .filter(|node| node.node_type == NodeType::Dungeon)
            .min_by(|left, right| {
                node_distance(left, shelter)
                    .partial_cmp(&node_distance(right, shelter))
                    .unwrap_or(Ordering::Equal)
            })
            .map(|node| node.id)
        else {
            return;
        };

        if let Some(node) = self.nodes.iter_mut().find(|node| node.id == node_id) {
            node.story_tag = Some(DungeonStoryTag::GabrielIntro);
        }
    }
}

// ---------------------------------------------------------------------------
// Generation
// ---------------------------------------------------------------------------

/// Generate a deterministic overworld graph from a seed.
pub fn generate_overworld(seed: u64) -> OverworldGraph {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let nodes = place_nodes(&mut rng);
    let roads = generate_roads(&nodes, &mut rng);
    let mut graph = OverworldGraph { nodes, roads };
    graph.ensure_story_tags();
    graph
}

fn place_nodes(rng: &mut ChaCha8Rng) -> Vec<OverworldNode> {
    use rand::RngExt;
    let mut nodes = Vec::new();

    // Shelter at center
    nodes.push(OverworldNode {
        id: 0,
        node_type: NodeType::Shelter,
        name: "The Shelter".to_string(),
        x: 0.0,
        y: 0.0,
        discovered: true,
        dungeon_theme: None,
        story_tag: None,
    });

    // 4 dungeons in a ring around the shelter
    let themes = [
        DungeonTheme::UrbanDecay,
        DungeonTheme::Underground,
        DungeonTheme::Military,
        DungeonTheme::UrbanDecay,
    ];
    for (i, theme) in themes.iter().enumerate() {
        let base_angle = (i as f32) * std::f32::consts::TAU / 4.0;
        let angle = base_angle + rng.random_range(-0.3..0.3f32);
        let dist = rng.random_range(5.0..8.0f32);
        nodes.push(OverworldNode {
            id: nodes.len(),
            node_type: NodeType::Dungeon,
            name: format!("Sector {}", i + 1),
            x: angle.cos() * dist,
            y: angle.sin() * dist,
            discovered: false,
            dungeon_theme: Some(*theme),
            story_tag: None,
        });
    }

    // 2 ruins at medium distance
    for i in 0..2 {
        let angle = rng.random_range(0.0..std::f32::consts::TAU);
        let dist = rng.random_range(3.0..6.0f32);
        nodes.push(OverworldNode {
            id: nodes.len(),
            node_type: NodeType::Ruins,
            name: format!("Old Ruins {}", i + 1),
            x: angle.cos() * dist,
            y: angle.sin() * dist,
            discovered: false,
            dungeon_theme: None,
            story_tag: None,
        });
    }

    // 2 crossroads
    for i in 0..2 {
        let angle = rng.random_range(0.0..std::f32::consts::TAU);
        let dist = rng.random_range(2.0..5.0f32);
        nodes.push(OverworldNode {
            id: nodes.len(),
            node_type: NodeType::Crossroads,
            name: format!("Crossroads {}", i + 1),
            x: angle.cos() * dist,
            y: angle.sin() * dist,
            discovered: false,
            dungeon_theme: None,
            story_tag: None,
        });
    }

    // 1 landmark at far distance
    let angle = rng.random_range(0.0..std::f32::consts::TAU);
    let dist = rng.random_range(8.0..12.0f32);
    nodes.push(OverworldNode {
        id: nodes.len(),
        node_type: NodeType::Landmark,
        name: "The Monument".to_string(),
        x: angle.cos() * dist,
        y: angle.sin() * dist,
        discovered: false,
        dungeon_theme: None,
        story_tag: None,
    });

    nodes
}

/// Euclidean distance between two nodes.
fn node_distance(a: &OverworldNode, b: &OverworldNode) -> f32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    (dx * dx + dy * dy).sqrt()
}

/// Build roads using Prim's MST for connectivity, then add ~30% random extra edges.
fn generate_roads(nodes: &[OverworldNode], rng: &mut ChaCha8Rng) -> Vec<Road> {
    use rand::RngExt;
    let n = nodes.len();
    if n < 2 {
        return Vec::new();
    }

    // Pre-compute pairwise distances
    let mut dist = vec![vec![0.0f32; n]; n];
    for i in 0..n {
        for j in (i + 1)..n {
            let d = node_distance(&nodes[i], &nodes[j]);
            dist[i][j] = d;
            dist[j][i] = d;
        }
    }

    // --- Prim's MST starting from node 0 (Shelter) ---
    let mut in_tree = vec![false; n];
    let mut min_cost = vec![f32::INFINITY; n];
    let mut min_edge = vec![0usize; n]; // which tree-node connects cheapest

    in_tree[0] = true;
    for j in 1..n {
        min_cost[j] = dist[0][j];
        min_edge[j] = 0;
    }

    let mut mst_edges: Vec<(usize, usize)> = Vec::with_capacity(n - 1);

    for _ in 0..(n - 1) {
        // Find cheapest fringe node
        let mut best = usize::MAX;
        let mut best_cost = f32::INFINITY;
        for j in 0..n {
            if !in_tree[j] && min_cost[j] < best_cost {
                best_cost = min_cost[j];
                best = j;
            }
        }
        if best == usize::MAX {
            break; // disconnected (shouldn't happen)
        }

        mst_edges.push((min_edge[best], best));
        in_tree[best] = true;

        // Update fringe costs
        for j in 0..n {
            if !in_tree[j] && dist[best][j] < min_cost[j] {
                min_cost[j] = dist[best][j];
                min_edge[j] = best;
            }
        }
    }

    // Collect MST edges into a set for deduplication
    let mut edge_set: std::collections::HashSet<(usize, usize)> = mst_edges
        .iter()
        .map(|&(a, b)| (a.min(b), a.max(b)))
        .collect();

    // --- Add ~30% of remaining possible edges ---
    let mut extra_candidates: Vec<(usize, usize)> = Vec::new();
    for i in 0..n {
        for j in (i + 1)..n {
            let key = (i, j);
            if !edge_set.contains(&key) {
                extra_candidates.push(key);
            }
        }
    }

    // Shuffle and take ~30%
    // Fisher-Yates shuffle
    for i in (1..extra_candidates.len()).rev() {
        let j = rng.random_range(0..=i);
        extra_candidates.swap(i, j);
    }
    let extra_count = (extra_candidates.len() as f32 * 0.3).round() as usize;
    for &(a, b) in extra_candidates.iter().take(extra_count) {
        edge_set.insert((a, b));
    }

    // Build Road structs
    edge_set
        .into_iter()
        .map(|(a, b)| Road {
            from: a,
            to: b,
            distance: dist[a][b],
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    #[test]
    fn test_determinism() {
        let g1 = generate_overworld(42);
        let g2 = generate_overworld(42);

        assert_eq!(g1.nodes.len(), g2.nodes.len());
        assert_eq!(g1.roads.len(), g2.roads.len());

        for (a, b) in g1.nodes.iter().zip(g2.nodes.iter()) {
            assert_eq!(a.id, b.id);
            assert_eq!(a.node_type, b.node_type);
            assert_eq!(a.name, b.name);
            assert!((a.x - b.x).abs() < f32::EPSILON);
            assert!((a.y - b.y).abs() < f32::EPSILON);
            assert_eq!(a.story_tag, b.story_tag);
        }
    }

    #[test]
    fn test_gabriel_intro_marks_closest_dungeon() {
        let graph = generate_overworld(42);
        let shelter = graph
            .nodes
            .iter()
            .find(|node| node.node_type == NodeType::Shelter)
            .expect("shelter node");
        let tagged: Vec<_> = graph
            .nodes
            .iter()
            .filter(|node| node.story_tag == Some(DungeonStoryTag::GabrielIntro))
            .collect();

        assert_eq!(tagged.len(), 1);
        assert_eq!(tagged[0].node_type, NodeType::Dungeon);

        let closest = graph
            .nodes
            .iter()
            .filter(|node| node.node_type == NodeType::Dungeon)
            .min_by(|left, right| {
                node_distance(left, shelter)
                    .partial_cmp(&node_distance(right, shelter))
                    .unwrap_or(Ordering::Equal)
            })
            .expect("closest dungeon");

        assert_eq!(tagged[0].id, closest.id);
    }

    #[test]
    fn test_connectivity() {
        let graph = generate_overworld(123);

        // BFS from shelter (node 0) — every node must be reachable.
        let mut visited = vec![false; graph.nodes.len()];
        let mut queue = VecDeque::new();
        visited[0] = true;
        queue.push_back(0);

        while let Some(current) = queue.pop_front() {
            for neighbor in graph.neighbors(current) {
                if !visited[neighbor] {
                    visited[neighbor] = true;
                    queue.push_back(neighbor);
                }
            }
        }

        assert!(
            visited.iter().all(|&v| v),
            "Not all nodes reachable from shelter"
        );
    }

    #[test]
    fn test_node_count() {
        let graph = generate_overworld(999);

        assert_eq!(graph.nodes.len(), 10);

        let shelters = graph
            .nodes
            .iter()
            .filter(|n| n.node_type == NodeType::Shelter)
            .count();
        let dungeons = graph
            .nodes
            .iter()
            .filter(|n| n.node_type == NodeType::Dungeon)
            .count();
        let ruins = graph
            .nodes
            .iter()
            .filter(|n| n.node_type == NodeType::Ruins)
            .count();
        let crossroads = graph
            .nodes
            .iter()
            .filter(|n| n.node_type == NodeType::Crossroads)
            .count();
        let landmarks = graph
            .nodes
            .iter()
            .filter(|n| n.node_type == NodeType::Landmark)
            .count();

        assert_eq!(shelters, 1);
        assert_eq!(dungeons, 4);
        assert_eq!(ruins, 2);
        assert_eq!(crossroads, 2);
        assert_eq!(landmarks, 1);
    }
}
