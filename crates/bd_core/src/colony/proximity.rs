//! Nearby interactable projection shared by Chronicle feedback, passive
//! context, and Interact availability.
//!
//! Cardinal adjacency is the Foundation interaction range. This module owns
//! the single read-only resolver: one stable, deterministic set of named
//! stations and resource nodes adjacent to the player. The Chronicle system
//! emits an edge-triggered `NEARBY` fact when a target newly enters range, and
//! the TUI action projection reads the same set to expose a semantic Interact
//! action. Rendering and view-model refreshes never write to this module.

use std::collections::HashMap;

use bevy_ecs::prelude::*;

use crate::{
    colony::{
        logistics::{Cargo, JobStage, LogisticsJob},
        resources::{DirectGatherProgress, pool_for_node},
        stations::{ConstructionSite, Station, StationType},
        survivors::{
            Survivor, SurvivorTask, WorkerActivity, WorkerBlockedReason, cardinally_adjacent,
        },
    },
    components::{ContentIdentity, Name, Player, Position, ResourceNode},
    content::FoundationContent,
    gamelog::{GameLog, LogLevel},
    session::FoundationRuntime,
    signals::{EntityMoved, PoolKind},
    spatial::{EntityScope, GameMode, entity_is_active},
};

/// Canonical display label for a resource node type. This matches the help
/// legend and the human-facing shelter map; the entity `Name` may carry a
/// shorter generator label, so the stable type label is the authoritative
/// identity for Chronicle and Context.
pub fn node_label(kind: crate::components::ResourceNodeType) -> &'static str {
    use crate::components::ResourceNodeType;
    match kind {
        ResourceNodeType::Trees => "Trees",
        ResourceNodeType::WaterSource => "Water Source",
        ResourceNodeType::WildPlants => "Wild Plants",
    }
}

/// Semantic category of a nearby interactable target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NearbyCategory {
    Station,
    ResourceNode,
    Colonist,
}

impl NearbyCategory {
    pub fn label(self) -> &'static str {
        match self {
            Self::Station => "Station",
            Self::ResourceNode => "Resource Node",
            Self::Colonist => "Colonist",
        }
    }
}

/// Display label for a survivor's current activity (concise status).
pub fn survivor_status(task: &SurvivorTask) -> &'static str {
    match task {
        SurvivorTask::Idle => "Idle",
        SurvivorTask::Gathering(_) => "Gathering",
        SurvivorTask::AssignedTo(_) => "Working",
        SurvivorTask::Resting => "Resting",
        SurvivorTask::Defending => "Defending",
    }
}

/// Display-ready label for the output pool a resource node feeds.
fn node_pool_label(kind: PoolKind) -> &'static str {
    match kind {
        PoolKind::Supplies => "Supplies",
        PoolKind::Materials => "Materials",
        PoolKind::WildPlants => "Medicine",
        _ => "Resources",
    }
}

/// Stable identity of a nearby target, independent of mutable status detail.
/// Deduplication compares this key so a staffing or activity change never
/// re-triggers an entry fact.
pub type NearbyIdentity = (NearbyCategory, String, Position);

/// Display-ready station row consumed by the nearby resolver.
pub struct StationRow {
    pub name: String,
    pub position: Position,
    pub station_type: StationType,
    pub staffed: bool,
    /// `Some((completed, required))` while the station is under construction.
    pub construction: Option<(u32, u32)>,
    /// Named worker currently staffing the station.
    pub worker: Option<String>,
    /// Active recipe display label (e.g. "Refine Water").
    pub recipe: Option<String>,
    /// Display-ready recipe progress (e.g. "1/2").
    pub progress: Option<String>,
}

/// Display-ready resource-node row consumed by the nearby resolver.
pub struct NodeRow {
    pub name: String,
    pub position: Position,
    pub pool: PoolKind,
    pub depleted: bool,
    /// Named gatherer currently assigned to this source.
    pub worker: Option<String>,
    /// Display-ready direct-gather progress (e.g. "1/3").
    pub progress: Option<String>,
}

/// Display-ready colonist row consumed by the nearby resolver.
pub struct ColonistRow {
    pub name: String,
    pub position: Position,
    pub status: String,
    pub target_note: String,
    /// Named assignment target/destination when applicable.
    pub target: Option<String>,
    /// Display-ready work progress (e.g. "1/3").
    pub progress: Option<String>,
    /// Human-labeled cargo plus amount when carrying.
    pub cargo: Option<(String, u32)>,
    /// Actionable blocker reason when blocked.
    pub blocked_reason: Option<String>,
    /// True while the colonist is idle (no assignment target).
    pub idle: bool,
}

/// Resolve a worker activity target to the canonical display label. Worker
/// activities name their target from the entity `Name` (e.g. "Water"), while
/// the node row and help legend use the stable type label ("Water Source");
/// station names pass through unchanged.
fn canonical_target_label(target: &str, node_labels: &HashMap<String, &'static str>) -> String {
    node_labels
        .get(target)
        .copied()
        .unwrap_or(target)
        .to_string()
}

/// Concise, truthful display label for a worker blocker.
fn blocked_reason_label(reason: WorkerBlockedReason) -> &'static str {
    match reason {
        WorkerBlockedReason::MissingTarget => "Target gone",
        WorkerBlockedReason::TargetUnavailable => "Unavailable",
        WorkerBlockedReason::NoAdjacentWorkTile => "No work tile",
        WorkerBlockedReason::NoRoute => "No route",
        WorkerBlockedReason::DestinationReserved => "Reserved",
    }
}

/// One stable, named target currently in cardinal interaction range.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NearbyTarget {
    pub category: NearbyCategory,
    pub name: String,
    pub position: Position,
    /// Concise entity-specific status (e.g. "Unstaffed", "Supplies", "Idle").
    pub status: String,
    /// Display-ready detail line, e.g. "Station · Operational · Unstaffed".
    pub detail: String,
    /// `Some((completed, required))` while a station is under construction.
    pub construction: Option<(u32, u32)>,
    /// True while a resource node is depleted.
    pub depleted: bool,
    /// True while a colonist is idle (has no assignment target).
    pub idle: bool,
    /// Named worker staffing a station or gathering at a node.
    pub worker: Option<String>,
    /// Active recipe display label (staffed station).
    pub recipe: Option<String>,
    /// Display-ready progress (e.g. "1/2") for station/node/colonist work.
    pub progress: Option<String>,
    /// Named colonist target/destination (assigned/carrying/blocked).
    pub target: Option<String>,
    /// Human-labeled cargo plus amount (carrying colonist).
    pub cargo: Option<(String, u32)>,
    /// Actionable blocker reason (blocked colonist).
    pub blocked_reason: Option<String>,
}

impl NearbyTarget {
    /// Chronicle fact emitted exactly once when this target newly enters range.
    ///
    /// The fact stays compact (category + status + Interact) so the wrapped
    /// Chronicle at both supported profiles never clips the target name, the
    /// category, the status, or the Interact hint; the full detail line lives
    /// in the projection and the Context panel instead.
    pub fn fact_line(&self) -> String {
        let category = self.category.label();
        let status = if self.depleted {
            "Depleted"
        } else {
            self.status.as_str()
        };
        format!("NEARBY {} · {category} · {status} · Interact", self.name)
    }

    /// Stable key used to detect a genuinely new entry. Detail changes such as
    /// staffing flips must not count as a fresh entry.
    pub fn identity(&self) -> NearbyIdentity {
        (self.category, self.name.clone(), self.position)
    }
}

/// Current stable nearby-interactable projection.
///
/// `targets` is the read-only current set. `previous` is the last observed set
/// used by the Chronicle edge to emit exactly one fact per entry. The edge
/// itself fires only on an `EntityMoved` for the player, so builds, node
/// spawns, fixture relocation, and projection refreshes never write history.
#[derive(Resource, Debug, Clone, Default)]
pub struct NearbyInteractables {
    pub targets: Vec<NearbyTarget>,
    previous: Vec<NearbyTarget>,
}

impl NearbyInteractables {
    pub fn is_empty(&self) -> bool {
        self.targets.is_empty()
    }
}

/// Compute the deterministic nearby set from domain snapshots.
///
/// Station rows carry their display name, position, staffed state, optional
/// construction progress, and authoritative worker/recipe/progress facts; node
/// rows carry name, position, output pool, depletion, and gatherer progress;
/// colonist rows carry name, position, activity status, target note, target,
/// progress, cargo, and blocker. The result is ordered by category, name, then
/// position so two frames with the same targets always produce the same
/// projection (never ECS query order or raw entity bits).
pub fn resolve_nearby(
    player_position: Position,
    stations: &[StationRow],
    colonists: &[ColonistRow],
    nodes: &[NodeRow],
) -> Vec<NearbyTarget> {
    let mut targets = Vec::new();
    for station in stations {
        if !cardinally_adjacent(player_position, station.position) {
            continue;
        }
        let (status, detail) = station.construction.map_or_else(
            || {
                let status = if station.staffed {
                    "Staffed".to_string()
                } else {
                    "Unstaffed".to_string()
                };
                // Compact shared detail: category then staffing state, with the
                // active worker/recipe/progress facts immediately after. The
                // Context title names the category and staffing state, so the
                // downstream status selects the active facts from this one
                // projection instead of rebuilding them. The unstaffed default
                // row keeps "Operational", which its walk-by contract names.
                let mut parts = vec!["Station".to_string(), status.clone()];
                if station.staffed {
                    if let Some(worker) = &station.worker {
                        parts.push(worker.clone());
                    }
                    if let Some(recipe) = &station.recipe {
                        parts.push(recipe.clone());
                    }
                    if let Some(progress) = &station.progress {
                        parts.push(progress.clone());
                    }
                } else {
                    parts.push("Operational".to_string());
                }
                (status, parts.join(" · "))
            },
            |(completed, required)| {
                let status = format!("{completed}/{required}");
                (status.clone(), format!("Construction · {status} work"))
            },
        );
        targets.push(NearbyTarget {
            category: NearbyCategory::Station,
            name: station.name.clone(),
            position: station.position,
            status,
            detail,
            construction: station.construction,
            depleted: false,
            idle: false,
            worker: station.worker.clone(),
            recipe: station.recipe.clone(),
            progress: station.progress.clone(),
            target: None,
            cargo: None,
            blocked_reason: None,
        });
    }
    for colonist in colonists {
        if !cardinally_adjacent(player_position, colonist.position) {
            continue;
        }
        let mut parts = vec![format!("Colonist · {}", colonist.status)];
        if !colonist.target_note.is_empty() {
            parts.push(colonist.target_note.clone());
        }
        if let Some(target) = &colonist.target {
            if colonist.blocked_reason.is_some() {
                parts.push(format!("Blocked {target}"));
            } else if colonist.cargo.is_some() {
                parts.push(format!("To {target}"));
            } else {
                parts.push(target.clone());
            }
        }
        if let Some(progress) = &colonist.progress {
            if colonist.blocked_reason.is_none() {
                parts.push(progress.clone());
            }
        }
        if let Some((label, amount)) = &colonist.cargo {
            parts.push(format!("Cargo {label} {amount}"));
        }
        if let Some(blocked) = &colonist.blocked_reason {
            parts.push(blocked.clone());
        }
        targets.push(NearbyTarget {
            category: NearbyCategory::Colonist,
            name: colonist.name.clone(),
            position: colonist.position,
            status: colonist.status.clone(),
            detail: parts.join(" · "),
            construction: None,
            depleted: false,
            idle: colonist.idle,
            worker: None,
            recipe: None,
            progress: colonist.progress.clone(),
            target: colonist.target.clone(),
            cargo: colonist.cargo.clone(),
            blocked_reason: colonist.blocked_reason.clone(),
        });
    }
    for node in nodes {
        if !cardinally_adjacent(player_position, node.position) {
            continue;
        }
        let (status, detail) = if node.depleted {
            (
                "Depleted".to_string(),
                "Resource Node · Depleted".to_string(),
            )
        } else {
            let label = node_pool_label(node.pool);
            let mut parts = vec![
                "Resource Node".to_string(),
                label.to_string(),
                "Renewable".to_string(),
            ];
            if let Some(worker) = &node.worker {
                parts.push(worker.clone());
            }
            if let Some(progress) = &node.progress {
                parts.push(format!("Gather {progress}"));
            }
            (label.to_string(), parts.join(" · "))
        };
        targets.push(NearbyTarget {
            category: NearbyCategory::ResourceNode,
            name: node.name.clone(),
            position: node.position,
            status,
            detail,
            construction: None,
            depleted: node.depleted,
            idle: false,
            worker: node.worker.clone(),
            recipe: None,
            progress: node.progress.clone(),
            target: None,
            cargo: None,
            blocked_reason: None,
        });
    }
    targets.sort_by(|left, right| {
        (
            left.category,
            left.name.as_str(),
            left.position.y,
            left.position.x,
        )
            .cmp(&(
                right.category,
                right.name.as_str(),
                right.position.y,
                right.position.x,
            ))
    });
    targets
}

/// Refresh the nearby projection and emit exactly one `NEARBY` Chronicle fact
/// per accepted player movement that newly brings station/node targets into
/// range. Multiple simultaneous entries aggregate into one focused fact naming
/// the deterministic first target plus a semantic count.
///
/// The Chronicle edge fires only when the player entity actually moved this
/// frame (`EntityMoved`), never on a bare position change, build, node spawn,
/// fixture relocation, rendering, resizing, Help, save, or load. Colonists are
/// nearby context targets but never write a `NEARBY` fact.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn update_nearby_interactables(
    player: Query<(Entity, &Position, Option<&EntityScope>), With<Player>>,
    mut moved: bevy_ecs::message::MessageReader<EntityMoved>,
    stations: Query<
        (
            Entity,
            &Name,
            &Position,
            &StationType,
            Option<&ConstructionSite>,
            Option<&ContentIdentity>,
            Option<&EntityScope>,
        ),
        With<Station>,
    >,
    survivors: Query<
        (
            &Name,
            &Position,
            &SurvivorTask,
            Option<&WorkerActivity>,
            Option<&LogisticsJob>,
            Option<&Cargo>,
            Option<&DirectGatherProgress>,
            Option<&EntityScope>,
        ),
        With<Survivor>,
    >,
    nodes: Query<(&Name, &Position, &ResourceNode, Option<&EntityScope>)>,
    mode: Res<GameMode>,
    foundation: Option<Res<FoundationRuntime>>,
    content: Option<Res<FoundationContent>>,
    mut nearby: ResMut<NearbyInteractables>,
    mut game_log: ResMut<GameLog>,
) {
    let foundation_runtime = foundation.is_some();
    if *mode != GameMode::Outpost {
        nearby.targets.clear();
        return;
    }
    let Some((player_entity, player_position, _)) = player
        .iter()
        .find(|(_, _, scope)| entity_is_active(*scope, *mode, foundation_runtime))
        .map(|(entity, position, scope)| (entity, *position, scope))
    else {
        nearby.targets.clear();
        return;
    };

    // Canonical node identity map: entity Name -> stable type label, so a
    // worker activity target written from the node Name resolves to the same
    // label the node row and the help legend use.
    let node_labels: HashMap<String, &'static str> = nodes
        .iter()
        .filter(|(_, _, _, scope)| entity_is_active(*scope, *mode, foundation_runtime))
        .map(|(name, _, node, _)| (name.0.clone(), node_label(node.kind)))
        .collect();

    // Station identity index: entity bits -> content identity. Production
    // logistics jobs name their station by recipe `station_id` (content
    // identity), while pre-recipe staffing names it by entity, so both resolve
    // to one stable station key here.
    let mut station_identity: HashMap<u64, String> = HashMap::new();
    for (entity, _, _, _, _, identity, _) in &stations {
        if let Some(identity) = identity {
            station_identity.insert(entity.to_bits(), identity.0.clone());
        }
    }

    // Authoritative active-state facts resolved once from domain components and
    // content: staffed stations (worker/recipe/progress), assigned node
    // gatherers (worker/progress), and colonist activity facts. Staffing keys
    // by the station content identity so a real logistics job (which sets the
    // worker task to Idle) and a pre-recipe AssignedTo both staff the same row.
    let mut staffed: HashMap<String, (String, Option<String>, Option<String>)> = HashMap::new();
    let mut gatherers: HashMap<String, (String, String)> = HashMap::new();
    let mut colonist_rows: Vec<ColonistRow> = Vec::new();
    for (name, position, task, activity, job, cargo, gather, scope) in &survivors {
        if !entity_is_active(scope, *mode, foundation_runtime) {
            continue;
        }
        // A durable logistics job staffs its station with the assigned worker,
        // active recipe label, and display-ready progress. The recipe workflow
        // sets the worker task to Idle, so staffing derives from the job and
        // content rather than the task marker.
        if let Some(job) = job
            && let Some(recipe) = content.as_ref().and_then(|content| {
                content
                    .colony_recipes
                    .iter()
                    .find(|recipe| recipe.id == job.recipe_id)
            })
        {
            let required = match job.stage {
                JobStage::ToSource | JobStage::ReadyToGather => recipe.gather_work_turns,
                JobStage::ToStation | JobStage::ReadyToRefine => recipe.refine_work_turns,
            };
            staffed.insert(
                recipe.station_id.clone(),
                (
                    name.0.clone(),
                    Some(recipe.label.clone()),
                    Some(format!("{}/{}", job.work_completed, required)),
                ),
            );
        }
        // Station staffing before a recipe is assigned names the station by
        // entity; resolve it to the same identity key when available.
        if let SurvivorTask::AssignedTo(bits) = task {
            let key = station_identity
                .get(bits)
                .cloned()
                .unwrap_or_else(|| bits.to_string());
            staffed
                .entry(key)
                .or_insert_with(|| (name.0.clone(), None, None));
        }
        if let SurvivorTask::Gathering(_) = task
            && let Some(gather) = gather
            && let Some(definition) = content.as_ref().and_then(|content| {
                content
                    .colony_gather_tasks
                    .iter()
                    .find(|definition| definition.id == gather.definition_id)
            })
        {
            gatherers.insert(
                definition.source_id.clone(),
                (
                    name.0.clone(),
                    format!("{}/{}", gather.work_completed, definition.work_turns),
                ),
            );
        }

        let status = survivor_status(task).to_string();
        let mut row_target = None;
        let mut row_progress = None;
        let mut row_cargo = None;
        let mut row_blocked = None;
        let mut target_note = String::new();
        let mut idle = false;
        match activity {
            Some(WorkerActivity::EnRoute { target, .. })
            | Some(WorkerActivity::Working { target, .. }) => {
                row_target = Some(canonical_target_label(target, &node_labels));
            }
            Some(WorkerActivity::Blocked { target, reason, .. }) => {
                row_target = Some(canonical_target_label(target, &node_labels));
                row_blocked = Some(blocked_reason_label(*reason).to_string());
            }
            Some(WorkerActivity::Idle) | None
                if matches!(task, SurvivorTask::Idle) && job.is_none() && cargo.is_none() =>
            {
                idle = true;
                target_note = "No target".to_string();
            }
            _ => {}
        }
        if let Some(cargo) = cargo
            && cargo.amount > 0
        {
            let label = cargo
                .resource_id
                .as_deref()
                .and_then(|id| {
                    content.as_ref().and_then(|content| {
                        content
                            .colony_resources
                            .iter()
                            .find(|resource| resource.id == id)
                    })
                })
                .map_or("Cargo".to_string(), |resource| resource.label.clone());
            row_cargo = Some((label, cargo.amount));
        }
        if let Some(gather) = gather
            && let Some(definition) = content.as_ref().and_then(|content| {
                content
                    .colony_gather_tasks
                    .iter()
                    .find(|definition| definition.id == gather.definition_id)
            })
        {
            row_progress = Some(format!(
                "{}/{}",
                gather.work_completed, definition.work_turns
            ));
        }
        if let Some(job) = job
            && let Some(recipe) = content.as_ref().and_then(|content| {
                content
                    .colony_recipes
                    .iter()
                    .find(|recipe| recipe.id == job.recipe_id)
            })
        {
            let required = match job.stage {
                JobStage::ToSource | JobStage::ReadyToGather => recipe.gather_work_turns,
                JobStage::ToStation | JobStage::ReadyToRefine => recipe.refine_work_turns,
            };
            row_progress = Some(format!("{}/{}", job.work_completed, required));
        }
        colonist_rows.push(ColonistRow {
            name: name.0.clone(),
            position: *position,
            status,
            target_note,
            target: row_target,
            progress: row_progress,
            cargo: row_cargo,
            blocked_reason: row_blocked,
            idle,
        });
    }

    let station_rows = stations
        .iter()
        .filter(|(_, _, _, _, _, _, scope)| entity_is_active(*scope, *mode, foundation_runtime))
        .map(
            |(entity, name, position, station_type, site, identity, _)| {
                let key = identity
                    .as_ref()
                    .map(|identity| identity.0.clone())
                    .unwrap_or_else(|| entity.to_bits().to_string());
                let facts = staffed.get(&key);
                StationRow {
                    name: name.0.clone(),
                    position: *position,
                    station_type: *station_type,
                    staffed: facts.is_some(),
                    construction: site.map(|site| (site.work_completed, site.work_required)),
                    worker: facts.map(|facts| facts.0.clone()),
                    recipe: facts.and_then(|facts| facts.1.clone()),
                    progress: facts.and_then(|facts| facts.2.clone()),
                }
            },
        )
        .collect::<Vec<_>>();
    let node_rows = nodes
        .iter()
        .filter(|(_, _, _, scope)| entity_is_active(*scope, *mode, foundation_runtime))
        .map(|(_, position, node, _)| {
            let gatherer = gatherers.get(&node.source_id);
            NodeRow {
                name: node_label(node.kind).to_string(),
                position: *position,
                pool: pool_for_node(node.kind),
                depleted: node.depleted,
                worker: gatherer.map(|gatherer| gatherer.0.clone()),
                progress: gatherer.map(|gatherer| gatherer.1.clone()),
            }
        })
        .collect::<Vec<_>>();

    let current = resolve_nearby(player_position, &station_rows, &colonist_rows, &node_rows);
    let player_moved = moved
        .read()
        .any(|movement| movement.entity == player_entity);
    if player_moved {
        // Only stations and resource nodes are Chronicle feedback targets.
        // Deduplicate by stable identity against the previous set: a target
        // that remains adjacent (even when its staffing/activity detail
        // changes) never re-emits, and a silent exit rearms re-entry.
        let entered = current
            .iter()
            .filter(|target| {
                target.category != NearbyCategory::Colonist
                    && !nearby
                        .previous
                        .iter()
                        .any(|previous| previous.identity() == target.identity())
            })
            .collect::<Vec<_>>();
        if let Some(first) = entered.first() {
            // Simultaneous entry stays one focused fact: the deterministic
            // first target (category/name/position order) plus a semantic
            // count of the additional targets. The current projection below
            // still retains the complete list.
            let fact = if entered.len() == 1 {
                first.fact_line()
            } else {
                format!(
                    "NEARBY {} · {} · Interact · +{}",
                    first.name,
                    first.detail,
                    entered.len() - 1
                )
            };
            game_log.push(fact, LogLevel::Info);
        }
    }
    nearby.previous = current.clone();
    nearby.targets = current;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn station_row(name: &str, position: Position) -> StationRow {
        StationRow {
            name: name.into(),
            position,
            station_type: StationType::Custom(1),
            staffed: false,
            construction: None,
            worker: None,
            recipe: None,
            progress: None,
        }
    }

    fn colonist_row(name: &str, position: Position) -> ColonistRow {
        ColonistRow {
            name: name.into(),
            position,
            status: "Idle".into(),
            target_note: "No target".into(),
            target: None,
            progress: None,
            cargo: None,
            blocked_reason: None,
            idle: true,
        }
    }

    fn node_row(name: &str, position: Position) -> NodeRow {
        NodeRow {
            name: name.into(),
            position,
            pool: PoolKind::Supplies,
            depleted: false,
            worker: None,
            progress: None,
        }
    }

    #[test]
    fn resolver_orders_by_category_then_name_then_position() {
        let player = Position { x: 5, y: 5 };
        let targets = resolve_nearby(
            player,
            &[station_row("Basic Processing", Position { x: 5, y: 4 })],
            &[colonist_row("Mara", Position { x: 5, y: 6 })],
            &[node_row("Water Source", Position { x: 6, y: 5 })],
        );
        assert_eq!(targets.len(), 3);
        assert_eq!(targets[0].category, NearbyCategory::Station);
        assert_eq!(targets[1].category, NearbyCategory::ResourceNode);
        assert_eq!(targets[2].category, NearbyCategory::Colonist);
        assert!(targets[0].fact_line().contains("Interact"));
        assert!(targets[0].fact_line().contains("Basic Processing"));
        assert!(targets[0].detail.contains("Station"));
        assert!(targets[0].detail.contains("Unstaffed"));
    }

    #[test]
    fn resolver_ignores_targets_outside_cardinal_range() {
        let player = Position { x: 5, y: 5 };
        let targets = resolve_nearby(
            player,
            &[station_row("Far", Position { x: 8, y: 5 })],
            &[colonist_row("Mara", Position { x: 8, y: 6 })],
            &[node_row("Diagonal", Position { x: 6, y: 6 })],
        );
        assert!(targets.is_empty());
    }

    #[test]
    fn construction_site_renders_progress_detail() {
        let player = Position { x: 5, y: 5 };
        let targets = resolve_nearby(
            player,
            &[StationRow {
                name: "Stove".into(),
                position: Position { x: 6, y: 5 },
                station_type: StationType::Stove,
                staffed: false,
                construction: Some((1, 4)),
                worker: None,
                recipe: None,
                progress: None,
            }],
            &[],
            &[],
        );
        assert_eq!(targets.len(), 1);
        assert!(targets[0].detail.contains("Construction · 1/4 work"));
        assert!(targets[0].construction == Some((1, 4)));
        assert!(targets[0].fact_line().contains("Interact"));
        assert!(targets[0].fact_line().contains("Station · 1/4"));
    }

    #[test]
    fn staffed_station_row_composes_worker_recipe_and_progress() {
        let player = Position { x: 5, y: 5 };
        let targets = resolve_nearby(
            player,
            &[StationRow {
                name: "Basic Processing".into(),
                position: Position { x: 5, y: 4 },
                station_type: StationType::Custom(1),
                staffed: true,
                construction: None,
                worker: Some("Mara".into()),
                recipe: Some("Refine Water".into()),
                progress: Some("1/2".into()),
            }],
            &[],
            &[],
        );
        assert_eq!(targets.len(), 1);
        let detail = format!("{} {}", targets[0].status, targets[0].detail).to_ascii_lowercase();
        assert!(detail.contains("mara"));
        assert!(detail.contains("refine water"));
        assert!(detail.contains("1/2"));
        assert!(targets[0].worker.as_deref() == Some("Mara"));
    }
}
