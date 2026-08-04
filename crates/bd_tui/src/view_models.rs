//! View models — plain data structs between ECS and rendering.

use bevy_app::App;
use bevy_ecs::{
    prelude::*,
    query::With,
    system::{Query, Res, ResMut, SystemParam},
};
use serde::{Deserialize, Serialize};

use bd_core::{
    BdSet,
    components::{BlocksMovement, ExitTile, Name, Player, Position, Tile},
    gamelog::{GameLog, LogLevel},
    inventory::Item,
    map::SmokeMap,
    pools::Pools,
    relationships::{ContainedIn, EquippedBy},
    signals::PoolKind,
};

/// Authoritative colony Supplies cap. Mirrors the colony Supplies pool bound
/// created by `bd_core::colony::production` (`Pool::new(Supplies, _, 0, 100)`)
/// and the `forecast_colony` clamp. Production always overrides the bound from
/// the live pool; this default only covers direct projection construction in
/// tests and is never consumed by the resource renderer.
pub const COLONY_SUPPLIES_CAP: i32 = 100;

/// Semantic resource pressure condition shared by every pool gauge.
///
/// The projection layer owns this threshold and emits a semantic token; the
/// renderer only formats the label and resolves the token. No fixture-specific
/// values live here.
pub fn resource_condition(current: i32, maximum: i32) -> (crate::visual::StyleToken, &'static str) {
    use crate::visual::StyleToken;
    if maximum <= 0 {
        (StyleToken::UiDanger, "UNKNOWN")
    } else if current <= 0 {
        (StyleToken::UiDanger, "CRITICAL")
    } else if current * 4 < maximum {
        (StyleToken::UiWarning, "LOW")
    } else {
        (StyleToken::UiPositive, "STABLE")
    }
}

/// Display-ready structured gauge for one colony pool. Populated by the
/// production projection from authoritative pool and forecast data; the
/// renderer consumes this and never recomputes pressure or parses prose.
#[derive(Debug, Clone)]
pub struct ResourceGaugeVm {
    /// Semantic label, e.g. "SUP".
    pub label: String,
    /// Exact current stock (authoritative).
    pub current: i32,
    /// Authoritative bound.
    pub maximum: i32,
    /// Display-ready condition label, e.g. "LOW"/"STABLE".
    pub condition: &'static str,
    /// Semantic fill token resolved by the theme layer.
    pub tone: crate::visual::StyleToken,
    /// Authoritative next-boundary delta.
    pub delta: i32,
    /// Resulting amount after the next boundary.
    pub result: i32,
}

#[derive(Resource, Debug, Clone)]
pub struct StatsViewModel {
    pub hp_current: i32,
    pub hp_max: i32,
    pub ap_current: i32,
    pub ap_max: i32,
    pub supplies: i32,
    /// Authoritative Supplies bound (legacy flat projection; the renderer uses
    /// [`StatsViewModel::supplies_gauge`] instead).
    pub supplies_max: i32,
    /// Structured display-ready Supplies gauge facts.
    pub supplies_gauge: Option<ResourceGaugeVm>,
    pub faith: i32,
    pub materials: i32,
    pub wild_plants: i32,
    pub stored_items: Vec<(String, u32)>,
    pub carried_loot: u32,
    pub extraction_ready: bool,
    pub save_available: bool,
    pub run_outcome: bd_core::session::RunOutcome,
    pub extracted_loot: u32,
    pub day: u64,
    pub party_names: Vec<String>,
    pub station_status: Vec<String>,
    pub next_day_forecast: String,
    pub latest_daily_summary: Vec<String>,
    pub management: Option<ManagementMenuVm>,
    /// Active nearby-target identity shown by the Context panel.
    pub context_target: Option<ContextTargetVm>,
    /// Compact faction standings: (label, value, status_text).
    pub faction_standings: Vec<(String, i32, String)>,
}

impl Default for StatsViewModel {
    fn default() -> Self {
        Self {
            supplies_max: COLONY_SUPPLIES_CAP,
            supplies_gauge: None,
            context_target: None,
            hp_current: 0,
            hp_max: 0,
            ap_current: 0,
            ap_max: 0,
            supplies: 0,
            faith: 0,
            materials: 0,
            wild_plants: 0,
            stored_items: Vec::new(),
            carried_loot: 0,
            extraction_ready: false,
            save_available: false,
            run_outcome: bd_core::session::RunOutcome::None,
            extracted_loot: 0,
            day: 0,
            party_names: Vec::new(),
            station_status: Vec::new(),
            next_day_forecast: String::new(),
            latest_daily_summary: Vec::new(),
            management: None,
            faction_standings: Vec::new(),
        }
    }
}

/// Compact re-presentation of the authoritative dawn outlook embedded in the
/// colony forecast display line.
///
/// The forecast prose is produced by the colony projection from structured
/// `forecast_colony` data (e.g. `…stn=-3→7 M+0…`). This projection extracts the
/// already-computed supplies delta and resulting amount so a narrow resource
/// panel never clips the decisive outlook. It never computes colony pressure;
/// it only re-presents values the domain already produced.
pub fn dawn_outlook(forecast: &str) -> Option<(String, String)> {
    let marker = '→';
    let start = forecast.find(marker)?;
    let delta = forecast[..start]
        .rsplit(|character: char| character.is_whitespace() || character == '=')
        .next()?
        .to_string();
    let result = forecast[start + marker.len_utf8()..]
        .split_whitespace()
        .next()?
        .to_string();
    Some((delta, result))
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct SaveAvailability {
    pub manual_slot: bool,
}

#[derive(Debug, Clone)]
pub struct ManagementMenuVm {
    pub kind: ManagementMenuKind,
    pub survivors: Vec<String>,
    pub tasks: Vec<String>,
    pub selected_survivor: Option<usize>,
    pub selected_task: Option<usize>,
    pub resources: String,
    pub forecast: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManagementMenuKind {
    TaskAssignment,
    StationStaffing,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct LogViewModel {
    pub entries: Vec<LogEntryVm>,
}

#[derive(Debug, Clone)]
pub struct LogEntryVm {
    pub message: String,
    pub level: LogLevel,
}

#[derive(Debug, Clone)]
pub struct ActionItemVm {
    pub label: String,
    pub key_hint: String,
    pub enabled: bool,
    pub denial_reason: Option<String>,
}

/// Identity of the active context target shown by the Context panel.
#[derive(Debug, Clone)]
pub struct ContextTargetVm {
    pub name: String,
    /// Canonical category label (e.g. "Station", "Resource Node", "Colonist").
    pub category: String,
    /// Concise status phrase shown in the Context feed.
    pub status: String,
    /// Category heading shown in the Context panel title (e.g.
    /// "Operational Station", "Resource Node", "Colonist").
    pub title: String,
    /// 1-based index of the focused target within the nearby set.
    pub focus_index: usize,
    /// Total number of nearby targets (for the focus selector).
    pub target_count: usize,
    /// Player-facing position cue of the focused target.
    pub position: (i32, i32),
}

#[derive(Resource, Debug, Clone, Default)]
pub struct ActionListViewModel {
    pub actions: Vec<ActionItemVm>,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct MapViewModel {
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<Tile>,
    pub player_pos: Option<Position>,
    /// Unified semantic entity/fixture projection consumed by the map renderer.
    pub visuals: Vec<MapVisualVm>,
    /// Physical targets of active survivor assignments.
    pub assigned_targets: Vec<Position>,
    /// Display identity for active assignment targets.
    pub assigned_target_details: Vec<AssignedTargetVm>,
    /// Build ghost cursor position and glyph for outpost map rendering.
    pub build_ghost: Option<(Position, char)>,
    /// Typed production-domain reason why the current preview cannot be built.
    pub build_ghost_denial: Option<String>,
    /// Catalog-owned detail retained while choosing the placement tile.
    pub build_placement: Option<BuildPlacementVm>,
    /// Build menu entries with highlight index (None if menu closed).
    pub build_menu: Option<BuildMenuVm>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapVisualVm {
    pub position: Position,
    pub token: crate::visual::VisualToken,
    pub glyph: Option<char>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssignedTargetVm {
    pub position: Position,
    pub label: String,
    pub survivor: String,
}

#[derive(Debug, Clone)]
pub struct BuildPlacementVm {
    pub label: String,
    pub supply_cost: i32,
    pub effect: String,
}

/// Build menu data for the station selection popup.
#[derive(Debug, Clone)]
pub struct BuildMenuVm {
    pub options: Vec<(String, i32, String)>, // (label, supply_cost, effect)
    pub selected: usize,
    pub available_supplies: i32,
}

#[derive(Resource, Debug, Clone, Default)]
#[allow(dead_code)]
pub struct ActorPanelViewModel {
    pub entity_name: Option<String>,
    pub hp: Option<(i32, i32)>,
}

/// View model for inventory/container display.
#[derive(Resource, Debug, Clone, Default)]
pub struct ContainerViewModel {
    pub items: Vec<ItemEntryVm>,
}

#[derive(Debug, Clone)]
pub struct ItemEntryVm {
    pub name: String,
    pub equipped: bool,
    pub usable: bool,
}

// ── Event view model ──

#[derive(Resource, Debug, Default, Clone, Serialize, Deserialize)]
pub struct HelpViewModel {
    pub keys: Vec<(String, String)>,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct EventViewModel {
    pub speaker: String,
    pub text: String,
    pub choices: Vec<String>,
    pub active: bool,
}

pub(crate) fn register_view_models(app: &mut App) {
    app.insert_resource(StatsViewModel::default());
    app.init_resource::<SaveAvailability>();
    app.insert_resource(LogViewModel::default());
    app.insert_resource(ActionListViewModel::default());
    app.insert_resource(MapViewModel::default());
    app.insert_resource(ActorPanelViewModel::default());
    app.insert_resource(ContainerViewModel::default());
    app.insert_resource(HelpViewModel::default());
    app.insert_resource(EventViewModel::default());
    app.add_systems(
        bevy_app::Update,
        (
            build_stats_vm,
            build_log_vm,
            build_action_list_vm,
            build_map_vm,
            build_container_vm,
            build_party_vm,
            build_event_vm,
            build_help_vm,
        )
            .in_set(BdSet::ViewModelBuild),
    );
}

type SurvivorPartyItem<'a> = (
    Entity,
    &'a Name,
    &'a Position,
    &'a bd_core::colony::survivors::SurvivorTask,
    Option<&'a Pools>,
    Option<&'a bd_core::spatial::EntityScope>,
    Option<&'a bd_core::colony::survivors::WorkerActivity>,
    Option<&'a bd_core::colony::resources::DirectGatherProgress>,
);

fn pool_label(kind: PoolKind) -> &'static str {
    match kind {
        PoolKind::Supplies => "Supplies",
        PoolKind::Materials => "Materials",
        PoolKind::WildPlants => "Wild Plants",
        PoolKind::Faith => "Faith",
        _ => "Resource",
    }
}

fn content_resource_label(content: &bd_core::content::FoundationContent, id: &str) -> String {
    content
        .colony_resources
        .iter()
        .find(|resource| resource.id == id)
        .map_or_else(
            || "Unknown resource".into(),
            |resource| resource.label.clone(),
        )
}

fn activity_label(
    activity: Option<&bd_core::colony::survivors::WorkerActivity>,
    worker_position: Position,
) -> String {
    let distance_to = |target: Position| {
        (target.x - worker_position.x).unsigned_abs()
            + (target.y - worker_position.y).unsigned_abs()
    };
    activity.map_or_else(
        || "Assigned".into(),
        |activity| match activity {
            bd_core::colony::survivors::WorkerActivity::Idle => "Idle".into(),
            bd_core::colony::survivors::WorkerActivity::EnRoute {
                target,
                target_position,
                ..
            } => {
                format!("EnRoute {target} · {} tiles", distance_to(*target_position))
            }
            bd_core::colony::survivors::WorkerActivity::Working { target, .. } => {
                format!("Working {target}")
            }
            bd_core::colony::survivors::WorkerActivity::Blocked {
                target,
                target_position,
                reason,
            } => {
                let reason = reason.to_string();
                let mut characters = reason.chars();
                let readable_reason = characters.next().map_or_else(String::new, |first| {
                    first.to_uppercase().collect::<String>() + characters.as_str()
                });
                target_position.map_or_else(
                    || format!("Blocked {target}: {readable_reason}"),
                    |target_position| {
                        format!(
                            "Blocked {target} · {} tiles: {readable_reason}",
                            distance_to(target_position)
                        )
                    },
                )
            }
            bd_core::colony::survivors::WorkerActivity::Resting => "Resting".into(),
            bd_core::colony::survivors::WorkerActivity::Defending => "Defending".into(),
        },
    )
}

#[derive(SystemParam)]
struct StatsProjectionInputs<'w, 's> {
    player: Query<'w, 's, (Entity, &'static Pools, &'static Position), With<Player>>,
    carried_items: Query<
        'w,
        's,
        (
            &'static ContainedIn,
            Option<&'static bd_core::spatial::EntityScope>,
        ),
        With<Item>,
    >,
    exits: Query<
        'w,
        's,
        (
            &'static Position,
            Option<&'static bd_core::spatial::EntityScope>,
        ),
        With<ExitTile>,
    >,
    colony_res: Res<'w, bd_core::colony::production::ColonyResources>,
    colony_storage: Res<'w, bd_core::colony::production::ColonyStorage>,
    game_time: Res<'w, bd_core::time::GameTime>,
    last_completed: Res<'w, bd_core::session::LastCompletedRun>,
    faction_rep: Option<Res<'w, bd_core::factions::FactionReputation>>,
    mode: Option<Res<'w, bd_core::spatial::GameMode>>,
    save_availability: Option<Res<'w, SaveAvailability>>,
}

fn build_stats_vm(inputs: StatsProjectionInputs, mut vm: ResMut<StatsViewModel>) {
    let mode = inputs.mode.as_deref().copied().unwrap_or_default();
    if let Ok((player_entity, pools, player_position)) = inputs.player.single() {
        vm.hp_current = pools.get(PoolKind::Health).map_or(0, |p| p.current);
        vm.hp_max = pools.get(PoolKind::Health).map_or(0, |p| p.max);
        vm.ap_current = pools.get(PoolKind::ActionPoints).map_or(0, |p| p.current);
        vm.ap_max = pools.get(PoolKind::ActionPoints).map_or(0, |p| p.max);
        vm.carried_loot = inputs
            .carried_items
            .iter()
            .filter(|(contained, scope)| {
                contained.0 == player_entity
                    && matches!(scope, Some(bd_core::spatial::EntityScope::DungeonTransient))
            })
            .count() as u32;
        vm.extraction_ready = mode == bd_core::spatial::GameMode::Tactical
            && inputs.exits.iter().any(|(position, scope)| {
                bd_core::spatial::entity_is_active(scope, mode, true) && position == player_position
            });
    } else {
        vm.carried_loot = 0;
        vm.extraction_ready = false;
    }
    vm.save_available = inputs
        .save_availability
        .is_some_and(|availability| availability.manual_slot);
    vm.supplies = inputs
        .colony_res
        .pools
        .get(PoolKind::Supplies)
        .map_or(0, |p| p.current);
    vm.supplies_max = inputs
        .colony_res
        .pools
        .get(PoolKind::Supplies)
        .map_or(COLONY_SUPPLIES_CAP, |p| p.max);
    vm.faith = inputs
        .colony_res
        .pools
        .get(PoolKind::Faith)
        .map_or(0, |p| p.current);
    vm.materials = inputs
        .colony_res
        .pools
        .get(PoolKind::Materials)
        .map_or(0, |p| p.current);
    vm.wild_plants = inputs
        .colony_res
        .pools
        .get(PoolKind::WildPlants)
        .map_or(0, |p| p.current);
    vm.stored_items = inputs
        .colony_storage
        .items
        .iter()
        .map(|(id, count)| (id.clone(), *count))
        .collect::<Vec<_>>();
    vm.day = inputs.game_time.day;
    vm.run_outcome = inputs.last_completed.outcome;
    vm.extracted_loot = inputs.last_completed.extracted_loot;

    // P17-D: Faction standings
    vm.faction_standings.clear();
    let Some(faction_rep) = inputs.faction_rep else {
        return;
    };
    for faction in bd_core::factions::ALL_FACTIONS {
        let val = faction_rep.get(faction);
        let status = bd_core::factions::faction_status(val);
        let label = match faction {
            PoolKind::RepPuritans => "Puritans",
            PoolKind::RepWanderers => "Wanderers",
            PoolKind::RepBrokenChoir => "BrokenChoir",
            PoolKind::RepDemons => "Demons",
            PoolKind::RepHumanSettlements => "Settlements",
            _ => "???",
        };
        let status_text = match status {
            bd_core::factions::FactionStatus::Hostile => "H",
            bd_core::factions::FactionStatus::Neutral => "N",
            bd_core::factions::FactionStatus::Friendly => "F",
            bd_core::factions::FactionStatus::Allied => "A",
        };
        vm.faction_standings
            .push((label.to_string(), val, status_text.to_string()));
    }
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn build_party_vm(
    survivors: Query<SurvivorPartyItem, With<bd_core::colony::survivors::Survivor>>,
    stations: Query<
        (
            Entity,
            &Name,
            &Position,
            &bd_core::colony::stations::StationType,
            Option<&bd_core::spatial::EntityScope>,
        ),
        (
            With<bd_core::colony::stations::Station>,
            Without<bd_core::colony::stations::ConstructionSite>,
        ),
    >,
    work_survivors: Query<
        (&Position, &bd_core::colony::survivors::SurvivorTask),
        With<bd_core::colony::survivors::Survivor>,
    >,
    work_stations: Query<
        (Entity, &Position, &bd_core::colony::stations::StationType),
        (
            With<bd_core::colony::stations::Station>,
            Without<bd_core::colony::stations::ConstructionSite>,
        ),
    >,
    construction_sites: Query<
        (
            &Name,
            &bd_core::colony::stations::ConstructionSite,
            Option<&bd_core::spatial::EntityScope>,
        ),
        With<bd_core::colony::stations::Station>,
    >,
    work_nodes: Query<(&Position, &bd_core::components::ResourceNode)>,
    colony_resources: Res<bd_core::colony::production::ColonyResources>,
    latest_daily_summary: Res<bd_core::colony::production::LatestDailySummary>,
    station_catalog: Res<bd_core::colony::stations::StationCatalog>,
    management: Res<crate::ManagementMenuState>,
    foundation_content: Option<Res<bd_core::content::FoundationContent>>,
    logistics: Query<(
        &bd_core::colony::logistics::LogisticsJob,
        &bd_core::colony::logistics::Cargo,
        Option<&bd_core::colony::survivors::WorkerActivity>,
    )>,
    mode: Res<bd_core::spatial::GameMode>,
    mut vm: ResMut<StatsViewModel>,
) {
    let mut active_survivors: Vec<_> = survivors
        .iter()
        .filter(|(_, _, _, _, _, scope, _, _)| scope_active(*scope, *mode))
        .collect::<Vec<_>>();
    active_survivors.sort_by(|left, right| left.1.0.cmp(&right.1.0));
    let mut active_stations: Vec<_> = stations
        .iter()
        .filter(|(_, _, _, _, scope)| scope_active(*scope, *mode))
        .collect::<Vec<_>>();
    active_stations.sort_by(|left, right| {
        (&left.1.0, left.2.y, left.2.x).cmp(&(&right.1.0, right.2.y, right.2.x))
    });
    vm.party_names = active_survivors
        .iter()
        .map(
            |(entity, name, position, task, pools, _, activity, direct_progress)| {
                let task = if let Ok((job, cargo, activity)) = logistics.get(*entity) {
                    let activity = activity_label(activity, **position);
                    let recipe = foundation_content.as_ref().and_then(|content| {
                        content
                            .colony_recipes
                            .iter()
                            .find(|recipe| recipe.id == job.recipe_id)
                    });
                    let work_required = recipe
                        .map(|recipe| match job.stage {
                            bd_core::colony::logistics::JobStage::ReadyToGather => {
                                recipe.gather_work_turns
                            }
                            bd_core::colony::logistics::JobStage::ReadyToRefine => {
                                recipe.refine_work_turns
                            }
                            _ => 0,
                        })
                        .unwrap_or(0);
                    let recipe_label =
                        recipe.map_or(job.recipe_id.as_str(), |recipe| recipe.label.as_str());
                    let cargo_label = cargo.resource_id.as_deref().map_or("empty".into(), |id| {
                        foundation_content.as_ref().map_or_else(
                            || "Unknown resource".into(),
                            |content| content_resource_label(content, id),
                        )
                    });
                    format!(
                        "{} {:?} {}/{} | {} | cargo {} {}",
                        recipe_label,
                        job.stage,
                        job.work_completed,
                        work_required,
                        activity,
                        cargo.amount,
                        cargo_label
                    )
                } else {
                    match task {
                        bd_core::colony::survivors::SurvivorTask::Idle => "Idle".into(),
                        bd_core::colony::survivors::SurvivorTask::Gathering(kind) => {
                            let definition = foundation_content.as_ref().and_then(|content| {
                                bd_core::colony::resources::direct_gather_definition(content, *kind)
                            });
                            let source = definition
                                .and_then(|definition| {
                                    foundation_content.as_ref().and_then(|content| {
                                        content
                                            .colony_sources
                                            .iter()
                                            .find(|source| source.id == definition.source_id)
                                    })
                                })
                                .map_or("Unknown source", |source| source.label.as_str());
                            let completed = definition
                                .zip(*direct_progress)
                                .filter(|(definition, progress)| {
                                    progress.definition_id == definition.id
                                })
                                .map_or(0, |(_, progress)| progress.work_completed);
                            let required = definition.map_or(0, |definition| definition.work_turns);
                            let output =
                                definition.map_or(0, |definition| definition.output_amount);
                            let work_state = activity.map_or_else(
                                || format!("Assigned {source}"),
                                |_| activity_label(*activity, **position),
                            );
                            format!(
                                "Gather {} | {} | {}/{} → {} {}",
                                pool_label(*kind),
                                work_state,
                                completed,
                                required,
                                output,
                                pool_label(*kind)
                            )
                        }
                        bd_core::colony::survivors::SurvivorTask::Defending => "Defend".into(),
                        bd_core::colony::survivors::SurvivorTask::Resting => "Rest".into(),
                        bd_core::colony::survivors::SurvivorTask::AssignedTo(bits) => {
                            active_stations
                                .iter()
                                .find(|(entity, _, _, _, _)| entity.to_bits() == *bits)
                                .map_or_else(
                                    || "Invalid station".into(),
                                    |(_, name, _, _, _)| name.0.clone(),
                                )
                        }
                    }
                };
                let mood = pools
                    .and_then(|pools| pools.get(PoolKind::Mood))
                    .map_or(0, |mood| mood.current);
                format!("{} — {} (Mood {})", name.0, task, mood)
            },
        )
        .collect();
    vm.station_status = active_stations
        .iter()
        .map(|(entity, name, _, station_type, _)| {
            let effect = station_catalog
                .get(**station_type)
                .map_or_else(|| "Unknown effect".into(), |entry| entry.effect_label());
            let worker = active_survivors
                .iter()
                .find(|(_, _, _, task, _, _, _, _)| {
                    matches!(
                        task,
                        bd_core::colony::survivors::SurvivorTask::AssignedTo(bits)
                            if *bits == entity.to_bits()
                    )
                })
                .map_or("Unstaffed", |(_, name, _, _, _, _, _, _)| name.0.as_str());
            format!("{} — {} — {}", name.0, effect, worker)
        })
        .collect();
    let mut site_status = construction_sites
        .iter()
        .filter(|(_, _, scope)| scope_active(*scope, *mode))
        .map(|(name, site, _)| {
            format!(
                "{} construction — {}/{} work",
                name.0, site.work_completed, site.work_required
            )
        })
        .collect::<Vec<_>>();
    site_status.sort();
    vm.station_status.extend(site_status);
    if let Some(content) = foundation_content.as_ref() {
        let raw = colony_resources
            .raw
            .iter()
            .filter(|(_, count)| **count > 0)
            .map(|(id, count)| format!("{} {}", content_resource_label(content, id), count))
            .collect::<Vec<_>>();
        if !raw.is_empty() {
            vm.station_status
                .push(format!("Raw stockpile — {}", raw.join(", ")));
        }
    }
    let work_survivors = work_survivors
        .iter()
        .map(
            |(position, task)| bd_core::colony::production::SurvivorWorkSnapshot {
                task: task.clone(),
                position: *position,
            },
        )
        .collect::<Vec<_>>();
    let work_stations = work_stations
        .iter()
        .map(
            |(entity, position, station_type)| bd_core::colony::production::StationWorkSnapshot {
                entity_bits: entity.to_bits(),
                station_type: *station_type,
                position: *position,
            },
        )
        .collect::<Vec<_>>();
    let work_nodes = work_nodes
        .iter()
        .map(
            |(position, node)| bd_core::colony::production::ResourceWorkSnapshot {
                kind: node.kind,
                position: *position,
                depleted: node.depleted,
            },
        )
        .collect::<Vec<_>>();
    let forecast = bd_core::colony::production::forecast_colony(
        &colony_resources,
        &work_survivors,
        &work_stations,
        &work_nodes,
        &station_catalog,
    );
    let next_worker =
        active_survivors
            .iter()
            .find_map(|(_, _, _, task, _, _, _, direct_progress)| {
                let bd_core::colony::survivors::SurvivorTask::Gathering(kind) = task else {
                    return None;
                };
                let definition = foundation_content.as_ref().and_then(|content| {
                    bd_core::colony::resources::direct_gather_definition(content, *kind)
                })?;
                let completed = direct_progress
                    .filter(|progress| progress.definition_id == definition.id)
                    .map_or(0, |progress| progress.work_completed);
                Some(format!(
                    "{} +{} after {} work",
                    pool_label(*kind),
                    definition.output_amount,
                    definition.work_turns.saturating_sub(completed)
                ))
            });
    vm.next_day_forecast = format!(
        "Next worker: {} | Next day: Sup -{}food {:+}stn={:+}→{} M{:+} P{:+} F{:+}",
        next_worker.as_deref().unwrap_or("no direct completion"),
        forecast.food_consumed,
        forecast.station_supplies,
        forecast.supplies_net,
        forecast.supplies_after,
        forecast.materials_net,
        forecast.plants_net,
        forecast.faith_net,
    );

    // Structured resource gauge: authoritative current/bound, condition, and
    // next-day delta/result produced here so the renderer only formats facts.
    let supplies_pool = colony_resources.pools.get(PoolKind::Supplies);
    let gauge_current = supplies_pool.map_or(0, |pool| pool.current);
    let gauge_maximum = supplies_pool.map_or(COLONY_SUPPLIES_CAP, |pool| pool.max);
    let (gauge_tone, gauge_condition) = resource_condition(gauge_current, gauge_maximum);
    vm.supplies_gauge = Some(ResourceGaugeVm {
        label: "SUP".into(),
        current: gauge_current,
        maximum: gauge_maximum,
        condition: gauge_condition,
        tone: gauge_tone,
        delta: forecast.supplies_net,
        result: forecast.supplies_after,
    });
    vm.latest_daily_summary = latest_daily_summary.0.as_ref().map_or_else(
        Vec::new,
        bd_core::colony::production::DailySummary::display_lines,
    );

    vm.management = management.active.then(|| {
        let selected_processor = management.selected_choice.is_some_and(|choice| {
            matches!(
                choice,
                crate::ManagementChoice::Station(selected)
                    if active_stations.iter().any(|(entity, _, _, station_type, _)| {
                        *entity == selected
                            && **station_type
                                == bd_core::colony::stations::StationType::Custom(1)
                    })
            )
        });
        let selected_task = if selected_processor {
            management.selected_recipe
        } else {
            management.selected_choice.and_then(|choice| match choice {
                crate::ManagementChoice::Action("ability.assign_idle") => Some(0),
                crate::ManagementChoice::Action("ability.gather_supplies") => Some(1),
                crate::ManagementChoice::Action("ability.gather_materials") => Some(2),
                crate::ManagementChoice::Action("ability.gather_plants") => Some(3),
                crate::ManagementChoice::Action("ability.assign_resting") => Some(4),
                crate::ManagementChoice::Action(_) => None,
                crate::ManagementChoice::Station(selected) => active_stations
                    .iter()
                    .position(|(entity, _, _, _, _)| *entity == selected),
            })
        };
        let task_options = match management.kind {
            crate::ManagementMenuKind::TaskAssignment => vec![
                "1. Idle".into(),
                "2. Gather Supplies".into(),
                "3. Gather Materials".into(),
                "4. Gather Plants".into(),
                "5. Rest".into(),
            ],
            crate::ManagementMenuKind::StationStaffing if selected_processor => foundation_content
                .as_deref()
                .map(|content| {
                    content
                        .colony_recipes
                        .iter()
                        .enumerate()
                        .map(|(index, recipe)| {
                            let input = content_resource_label(content, &recipe.input_resource_id);
                            let output =
                                content_resource_label(content, &recipe.output_resource_id);
                            format!("{}. {} — {} → {}", index + 1, recipe.label, input, output)
                        })
                        .collect()
                })
                .unwrap_or_default(),
            crate::ManagementMenuKind::StationStaffing => active_stations
                .iter()
                .enumerate()
                .map(|(index, (entity, name, _, station_type, _))| {
                    let effect = station_catalog
                        .get(**station_type)
                        .map_or_else(|| "Unknown effect".into(), |entry| entry.effect_label());
                    let worker = active_survivors
                        .iter()
                        .find(|(_, _, _, task, _, _, _, _)| {
                            matches!(
                                task,
                                bd_core::colony::survivors::SurvivorTask::AssignedTo(bits)
                                    if *bits == entity.to_bits()
                            )
                        })
                        .map_or("Unstaffed", |(_, name, _, _, _, _, _, _)| name.0.as_str());
                    format!("{}. {} — {} — {}", index + 1, name.0, effect, worker)
                })
                .collect(),
        };
        ManagementMenuVm {
            kind: match management.kind {
                crate::ManagementMenuKind::TaskAssignment => ManagementMenuKind::TaskAssignment,
                crate::ManagementMenuKind::StationStaffing => ManagementMenuKind::StationStaffing,
            },
            survivors: vm.party_names.clone(),
            tasks: task_options,
            selected_survivor: management.selected_survivor,
            selected_task,
            resources: format!(
                "Sup {}  Mat {}  Plant {}  Faith {}",
                colony_resources
                    .pools
                    .get(PoolKind::Supplies)
                    .map_or(0, |pool| pool.current),
                colony_resources
                    .pools
                    .get(PoolKind::Materials)
                    .map_or(0, |pool| pool.current),
                colony_resources
                    .pools
                    .get(PoolKind::WildPlants)
                    .map_or(0, |pool| pool.current),
                colony_resources
                    .pools
                    .get(PoolKind::Faith)
                    .map_or(0, |pool| pool.current),
            ),
            forecast: vm.next_day_forecast.clone(),
        }
    });
}

fn build_log_vm(log: Res<GameLog>, mut vm: ResMut<LogViewModel>) {
    vm.entries = log
        .iter()
        .map(|e| LogEntryVm {
            message: e.message.clone(),
            level: e.level,
        })
        .collect();
    vm.entries.reverse();
}

#[cfg(test)]
mod stabilization_tests {
    use super::*;

    #[test]
    fn rendered_combat_log_is_chronological() {
        let mut app = App::new();
        app.insert_resource(GameLog::default());
        app.insert_resource(LogViewModel::default());
        app.add_systems(bevy_app::Update, build_log_vm);
        app.world_mut()
            .resource_mut::<GameLog>()
            .push("Player attacks.", LogLevel::Combat);
        app.world_mut()
            .resource_mut::<GameLog>()
            .push("Rat takes 3 damage.", LogLevel::Combat);

        app.update();

        let messages = app
            .world()
            .resource::<LogViewModel>()
            .entries
            .iter()
            .map(|entry| entry.message.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            messages,
            ["Player attacks.", "Rat takes 3 damage."],
            "the rendered window must show cause before result"
        );
    }
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)] // One read-only projection over distinct ECS owners.
fn build_action_list_vm(
    player: Query<
        (
            Entity,
            &Position,
            &Pools,
            Option<&bd_core::spatial::EntityScope>,
        ),
        With<Player>,
    >,
    enemies: Query<
        (&Position, Option<&bd_core::spatial::EntityScope>),
        (With<BlocksMovement>, Without<Player>),
    >,
    mode: Res<bd_core::spatial::GameMode>,
    map: Res<SmokeMap>,
    outpost: Res<bd_core::spatial::OutpostState>,
    items: Query<
        (
            Option<&Position>,
            Option<&bd_core::inventory::Usable>,
            Option<&ContainedIn>,
            Option<&bd_core::spatial::EntityScope>,
        ),
        With<Item>,
    >,
    survivors: Query<
        (
            &bd_core::colony::survivors::SurvivorTask,
            Option<&bd_core::spatial::EntityScope>,
        ),
        With<bd_core::colony::survivors::Survivor>,
    >,
    stations: Query<
        Option<&bd_core::spatial::EntityScope>,
        (
            With<bd_core::colony::stations::Station>,
            Without<bd_core::colony::stations::ConstructionSite>,
        ),
    >,
    exits: Query<(&Position, Option<&bd_core::spatial::EntityScope>), With<ExitTile>>,
    colony_resources: Res<bd_core::colony::production::ColonyResources>,
    game_time: Res<bd_core::time::GameTime>,
    bindings: Res<crate::commands::CommandBindings>,
    station_catalog: Res<bd_core::colony::stations::StationCatalog>,
    nearby: Res<bd_core::colony::proximity::NearbyInteractables>,
    mut vm: ResMut<ActionListViewModel>,
    mut stats_vm: ResMut<StatsViewModel>,
) {
    let Some((player_entity, pp, pools, _)) = player
        .iter()
        .find(|(_, _, _, scope)| scope_active(*scope, *mode))
    else {
        vm.actions.clear();
        return;
    };
    let ap = pools.get(PoolKind::ActionPoints).map_or(0, |p| p.current);
    let has_ap = ap >= 1;
    let enemy_near = enemies
        .iter()
        .filter(|(_, scope)| scope_active(*scope, *mode))
        .any(|(position, _)| {
            (position.x - pp.x).unsigned_abs() + (position.y - pp.y).unsigned_abs() <= 1
        });
    let active_map = if *mode == bd_core::spatial::GameMode::Outpost {
        &outpost.map
    } else {
        &map
    };
    let can_move = [
        (pp.x, pp.y - 1),
        (pp.x, pp.y + 1),
        (pp.x - 1, pp.y),
        (pp.x + 1, pp.y),
    ]
    .into_iter()
    .any(|(x, y)| {
        active_map.is_walkable(x, y)
            && !enemies.iter().any(|(position, scope)| {
                scope_active(scope, *mode) && position.x == x && position.y == y
            })
    });
    let item_here = items.iter().any(|(position, _, _, scope)| {
        scope_active(scope, *mode) && position.is_some_and(|position| *position == *pp)
    });
    let usable_item = items.iter().any(|(_, usable, contained, scope)| {
        scope_active(scope, *mode)
            && usable.is_some()
            && contained.is_some_and(|container| container.0 == player_entity)
    });
    let survivor_available = survivors
        .iter()
        .any(|(_, scope)| scope_active(scope, *mode));
    let idle_survivor_available = survivors.iter().any(|(task, scope)| {
        scope_active(scope, *mode) && matches!(task, bd_core::colony::survivors::SurvivorTask::Idle)
    });
    let station_available = stations.iter().any(|scope| scope_active(scope, *mode));
    let minimum_build_cost = station_catalog
        .buildable()
        .map(|blueprint| blueprint.build_cost_supplies)
        .min()
        .unwrap_or_default();
    let supplies = colony_resources
        .pools
        .get(PoolKind::Supplies)
        .map_or(0, |pool| pool.current);
    let at_exit = exits
        .iter()
        .any(|(position, scope)| scope_active(scope, *mode) && *position == *pp);
    let availability = crate::commands::ActionAvailability {
        mode: *mode,
        has_ap,
        enemy_in_range: enemy_near,
        can_move,
        item_here,
        usable_item,
        can_build: supplies >= minimum_build_cost,
        survivor_available,
        station_available,
        at_exit,
        can_travel: supplies >= bd_core::spatial::TRAVEL_SUPPLIES_COST,
        day: game_time.day,
        turn: game_time.turn,
    };
    let mut projections = crate::commands::action_panel(&bindings, availability);
    if let Some(staff) = projections
        .iter_mut()
        .find(|action| action.command == crate::commands::UiCommand::AssignStation)
        && !idle_survivor_available
    {
        staff.enabled = false;
        staff.denial_reason = Some("No idle survivor".into());
    }

    let mut items = projections
        .into_iter()
        .map(|action| ActionItemVm {
            label: action.label,
            key_hint: action.key,
            enabled: action.enabled,
            denial_reason: action.denial_reason,
        })
        .collect::<Vec<_>>();

    // The colony context owns the action feed while an interactable target is
    // nearby: one generic context projection (fed by the shared proximity
    // resolver) supplies Interact and the focused target's preview actions. No
    // normal-world actions are advertised while the context owns input, and no
    // second nearby target's actions are flattened into the focused feed.
    if *mode == bd_core::spatial::GameMode::Outpost && !nearby.is_empty() {
        let focused = &nearby.targets[0];
        let interact_key = bindings
            .key_for(crate::commands::UiCommand::Interact)
            .map(crate::commands::config_key_name);
        // Interact is a semantic command with no owner-approved Context reducer
        // route yet (UI9-D is not authorized). A configured binding alone does
        // not make it executable, so it stays disabled with a truthful route
        // reason; an unbound Interact keeps the "unbound" hint. UI9-C preview
        // actions are likewise disabled until the Context menu reducer exists.
        let interact = match &interact_key {
            Some(key) => ActionItemVm {
                label: "Interact".into(),
                key_hint: key.clone(),
                enabled: false,
                denial_reason: Some("No Context route".into()),
            },
            None => ActionItemVm {
                label: "Interact".into(),
                key_hint: "unbound".into(),
                enabled: false,
                denial_reason: Some("Interact is unbound".into()),
            },
        };
        let preview_reason = "Menu".to_string();
        let mut context_items = vec![interact];
        context_items.push(ActionItemVm {
            label: format!("Inspect {}", focused.name),
            key_hint: String::new(),
            enabled: false,
            denial_reason: Some(preview_reason.clone()),
        });
        match focused.category {
            bd_core::colony::proximity::NearbyCategory::Station
                if focused.construction.is_none() =>
            {
                // Set Production precedes Assign Worker so the wrapped Context
                // feed packs into the compact three-row profile without
                // clipping; both remain disabled previews.
                // UI-only placeholder: production recipes are owner-locked
                // for a later phase, so this stays visibly unavailable.
                context_items.push(ActionItemVm {
                    label: "Set Production".into(),
                    key_hint: String::new(),
                    enabled: false,
                    denial_reason: Some("Coming later".into()),
                });
                context_items.push(ActionItemVm {
                    label: "Assign Worker".into(),
                    key_hint: String::new(),
                    enabled: false,
                    denial_reason: Some(preview_reason),
                });
            }
            // Construction sites never inherit operational-only actions.
            bd_core::colony::proximity::NearbyCategory::Station => {}
            bd_core::colony::proximity::NearbyCategory::ResourceNode if !focused.depleted => {
                context_items.push(ActionItemVm {
                    label: "Assign Gatherer".into(),
                    key_hint: String::new(),
                    enabled: false,
                    denial_reason: Some(preview_reason),
                });
            }
            // Depleted nodes never advertise invalid gathering actions.
            bd_core::colony::proximity::NearbyCategory::ResourceNode => {}
            bd_core::colony::proximity::NearbyCategory::Colonist => {
                context_items.push(ActionItemVm {
                    label: "Assign Task".into(),
                    key_hint: String::new(),
                    enabled: false,
                    denial_reason: Some(preview_reason),
                });
            }
        }
        items = context_items;
        // The Context title names the category and the authoritative staffing
        // state only. It must never derive staffing from the legacy parallel
        // worker/recipe/progress fields, which an adversarial observer can
        // poison independently of the shared detail/status.
        let title = match focused.category {
            bd_core::colony::proximity::NearbyCategory::Station
                if focused.construction.is_some() =>
            {
                "Construction Station".to_string()
            }
            bd_core::colony::proximity::NearbyCategory::Station => {
                format!("Station {}", focused.status)
            }
            bd_core::colony::proximity::NearbyCategory::ResourceNode => "Resource Node".to_string(),
            bd_core::colony::proximity::NearbyCategory::Colonist => "Colonist".to_string(),
        };
        // The Context feed transports the authoritative shared detail
        // projection with every semantic segment intact; only the presentation
        // separator is normalized. Category adapters in the proximity resolver
        // own the domain wording, and this layer never rebuilds it from
        // parallel fields or strips semantic segments.
        let category_label = focused.category.label().to_string();
        let status = focused.detail.replace(" · ", " ");
        stats_vm.context_target = Some(ContextTargetVm {
            name: focused.name.clone(),
            category: category_label,
            status,
            title,
            focus_index: 1,
            target_count: nearby.targets.len(),
            position: (focused.position.x, focused.position.y),
        });
    } else {
        stats_vm.context_target = None;
    }

    vm.actions = items;
}

fn build_help_vm(
    bindings: Res<crate::commands::CommandBindings>,
    mode: Res<bd_core::spatial::GameMode>,
    build: Res<bd_core::colony::stations::BuildInteraction>,
    symbols: Res<crate::visual::SymbolRegistry>,
    stations: Res<bd_core::colony::stations::StationCatalog>,
    mut vm: ResMut<HelpViewModel>,
) {
    let interaction = if *mode == bd_core::spatial::GameMode::GameOver {
        crate::commands::InteractionMode::GameOver
    } else if build.is_active() {
        crate::commands::InteractionMode::Build
    } else {
        crate::commands::InteractionMode::Normal
    };
    vm.keys = crate::commands::help_entries_with_legend(
        &bindings,
        *mode,
        interaction,
        &symbols,
        &stations,
    )
    .into_iter()
    .map(|entry| (entry.key, entry.description))
    .collect();
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)] // Map projection reads independent scoped entity layers.
fn build_map_vm(
    map: Res<SmokeMap>,
    player_pos: Query<(&Position, Option<&bd_core::spatial::EntityScope>), With<Player>>,
    enemies: Query<
        (
            &Position,
            Option<&bd_core::components::Name>,
            Option<&bd_core::spatial::EntityScope>,
        ),
        (With<BlocksMovement>, Without<Player>),
    >,
    survivors: Query<
        (
            &Position,
            Option<&bd_core::components::Name>,
            &bd_core::colony::survivors::SurvivorTask,
            Option<&bd_core::colony::survivors::WorkerActivity>,
            Option<&bd_core::spatial::EntityScope>,
        ),
        With<bd_core::colony::survivors::Survivor>,
    >,
    stations: Query<(
        Entity,
        &Position,
        &bd_core::colony::stations::StationType,
        Option<&bd_core::colony::stations::ConstructionSite>,
        Option<&bd_core::spatial::EntityScope>,
    )>,
    gabriel_q: Query<
        (&Position, Option<&bd_core::spatial::EntityScope>),
        With<bd_core::components::Gabriel>,
    >,
    resource_nodes: Query<(
        &Position,
        &bd_core::components::ResourceNode,
        Option<&bd_core::spatial::EntityScope>,
    )>,
    items: Query<
        (
            &Position,
            Option<&ContainedIn>,
            Option<&bd_core::spatial::EntityScope>,
        ),
        With<Item>,
    >,
    exit_tiles: Query<
        (&Position, Option<&bd_core::spatial::EntityScope>),
        With<bd_core::components::ExitTile>,
    >,
    build: Res<bd_core::colony::stations::BuildInteraction>,
    colony_resources: Res<bd_core::colony::production::ColonyResources>,
    station_catalog: Res<bd_core::colony::stations::StationCatalog>,
    mut vm: ResMut<MapViewModel>,
    mode: Res<bd_core::spatial::GameMode>,
    outpost: Res<bd_core::spatial::OutpostState>,
) {
    // Use shelter map in outpost mode, dungeon map otherwise
    let active_map = if *mode == bd_core::spatial::GameMode::Outpost {
        &outpost.map
    } else {
        &map
    };

    vm.width = active_map.width;
    vm.height = active_map.height;
    vm.tiles.clear();
    for y in 0..active_map.height {
        for x in 0..active_map.width {
            vm.tiles.push(active_map.get(x, y).unwrap_or(Tile::Wall));
        }
    }
    vm.player_pos = player_pos
        .iter()
        .find(|(_, scope)| scope_active(*scope, *mode))
        .map(|(position, _)| *position);
    vm.visuals.clear();
    if let Some(position) = vm.player_pos {
        vm.visuals.push(MapVisualVm {
            position,
            token: crate::visual::VisualToken::Player,
            glyph: None,
        });
    }
    // Only collect enemies in tactical/dungeon mode — shelter has no enemies
    if *mode != bd_core::spatial::GameMode::Outpost {
        for (pos, name, scope) in enemies.iter() {
            if !scope_active(scope, *mode) {
                continue;
            }
            let glyph = name.map_or('E', |n| match n.0.as_str() {
                "Rat" => 'r',
                "Skeleton" => 'S',
                "Boss" => 'B',
                _ => 'E',
            });
            vm.visuals.push(MapVisualVm {
                position: *pos,
                token: crate::visual::VisualToken::Enemy,
                glyph: Some(glyph),
            });
        }
    }
    vm.assigned_targets.clear();
    vm.assigned_target_details.clear();
    for (pos, name, task, activity, scope) in survivors.iter() {
        if !scope_active(scope, *mode) {
            continue;
        }
        let token = match activity {
            Some(bd_core::colony::survivors::WorkerActivity::Idle) => {
                crate::visual::VisualToken::WorkerIdle
            }
            Some(bd_core::colony::survivors::WorkerActivity::EnRoute { .. }) => {
                crate::visual::VisualToken::WorkerEnRoute
            }
            Some(bd_core::colony::survivors::WorkerActivity::Working { .. }) => {
                crate::visual::VisualToken::WorkerWorking
            }
            Some(bd_core::colony::survivors::WorkerActivity::Blocked { .. }) => {
                crate::visual::VisualToken::WorkerBlocked
            }
            Some(bd_core::colony::survivors::WorkerActivity::Resting) => {
                crate::visual::VisualToken::WorkerResting
            }
            Some(bd_core::colony::survivors::WorkerActivity::Defending) => {
                crate::visual::VisualToken::WorkerDefending
            }
            None => match task {
                bd_core::colony::survivors::SurvivorTask::Idle => {
                    crate::visual::VisualToken::WorkerIdle
                }
                bd_core::colony::survivors::SurvivorTask::Gathering(_)
                | bd_core::colony::survivors::SurvivorTask::AssignedTo(_) => {
                    crate::visual::VisualToken::WorkerEnRoute
                }
                bd_core::colony::survivors::SurvivorTask::Defending => {
                    crate::visual::VisualToken::WorkerDefending
                }
                bd_core::colony::survivors::SurvivorTask::Resting => {
                    crate::visual::VisualToken::WorkerResting
                }
            },
        };
        vm.visuals.push(MapVisualVm {
            position: *pos,
            token,
            glyph: None,
        });
        let target_detail = match activity {
            Some(bd_core::colony::survivors::WorkerActivity::EnRoute {
                target,
                target_position,
                ..
            })
            | Some(bd_core::colony::survivors::WorkerActivity::Working {
                target,
                target_position,
            }) => Some((*target_position, target.clone())),
            Some(bd_core::colony::survivors::WorkerActivity::Blocked {
                target,
                target_position,
                ..
            }) => target_position.map(|position| (position, target.clone())),
            _ => None,
        };
        if let Some((target_position, target_label)) = target_detail {
            vm.assigned_targets.push(target_position);
            vm.assigned_target_details.push(AssignedTargetVm {
                position: target_position,
                label: target_label,
                survivor: name.map_or_else(|| "Survivor".into(), |name| name.0.clone()),
            });
        }
    }
    vm.assigned_targets
        .sort_by_key(|position| (position.y, position.x));
    vm.assigned_targets.dedup();
    vm.assigned_target_details.sort_by(|left, right| {
        (
            left.position.y,
            left.position.x,
            left.label.as_str(),
            left.survivor.as_str(),
        )
            .cmp(&(
                right.position.y,
                right.position.x,
                right.label.as_str(),
                right.survivor.as_str(),
            ))
    });
    vm.assigned_target_details
        .dedup_by(|left, right| left.position == right.position && left.label == right.label);
    let staffed = survivors
        .iter()
        .filter_map(|(_, _, task, _, scope)| {
            if !scope_active(scope, *mode) {
                return None;
            }
            match task {
                bd_core::colony::survivors::SurvivorTask::AssignedTo(bits) => Some(*bits),
                _ => None,
            }
        })
        .collect::<std::collections::HashSet<_>>();
    for (entity, pos, stype, construction, scope) in stations.iter() {
        if !scope_active(scope, *mode) {
            continue;
        }
        let glyph = station_catalog.get(*stype).map_or('?', |entry| {
            if construction.is_some() {
                '%'
            } else if staffed.contains(&entity.to_bits()) {
                entry.staffed_glyph
            } else {
                entry.glyph
            }
        });
        vm.visuals.push(MapVisualVm {
            position: *pos,
            token: crate::visual::VisualToken::Station,
            glyph: Some(glyph),
        });
    }

    // P15-C: Gabriel glyph on shelter map
    let gabriel_glyph = gabriel_q
        .iter()
        .find(|(_, scope)| scope_active(*scope, *mode))
        .map(|(position, _)| (*position, 'G'));
    if let Some((position, glyph)) = gabriel_glyph {
        vm.visuals.push(MapVisualVm {
            position,
            token: crate::visual::VisualToken::Ally,
            glyph: Some(glyph),
        });
    }

    // P22-D: Resource node glyphs on shelter map
    for (pos, node, scope) in resource_nodes.iter() {
        if !scope_active(scope, *mode) {
            continue;
        }
        let token = match node.kind {
            bd_core::components::ResourceNodeType::Trees => crate::visual::VisualToken::Trees,
            bd_core::components::ResourceNodeType::WaterSource => {
                crate::visual::VisualToken::WaterSource
            }
            bd_core::components::ResourceNodeType::WildPlants => {
                crate::visual::VisualToken::WildPlants
            }
        };
        vm.visuals.push(MapVisualVm {
            position: *pos,
            token,
            glyph: None,
        });
    }

    for (position, contained, scope) in items.iter() {
        if contained.is_none() && scope_active(scope, *mode) {
            vm.visuals.push(MapVisualVm {
                position: *position,
                token: crate::visual::VisualToken::Item,
                glyph: None,
            });
        }
    }
    // P3-A: Exit tile glyphs on the shelter map (gate, dungeon exits)
    for (pos, scope) in exit_tiles.iter() {
        if !scope_active(scope, *mode) {
            continue;
        }
        vm.visuals.push(MapVisualVm {
            position: *pos,
            token: crate::visual::VisualToken::Exit,
            glyph: None,
        });
    }
    vm.visuals.sort_by_key(|visual| {
        (
            visual.position.y,
            visual.position.x,
            visual.token as u8,
            visual.glyph,
        )
    });

    // P2-C: Build transaction projection on the shelter map.
    let placement = match &*build {
        bd_core::colony::stations::BuildInteraction::Placing {
            selected_station,
            cursor,
            validation,
        } => Some((*selected_station, *cursor, validation.as_ref().err())),
        bd_core::colony::stations::BuildInteraction::AwaitingResolution {
            selected_station,
            cursor,
        } => Some((*selected_station, *cursor, None)),
        _ => None,
    };
    vm.build_ghost = placement.map(|(station_type, cursor, _)| {
        let glyph = station_catalog
            .get(station_type)
            .map_or('?', |entry| entry.glyph);
        (cursor, glyph)
    });
    vm.build_ghost_denial = placement
        .and_then(|(_, _, denial)| denial)
        .map(ToString::to_string);
    vm.build_placement = placement
        .map(|(station_type, _, _)| station_type)
        .and_then(|station_type| station_catalog.get(station_type))
        .map(|entry| BuildPlacementVm {
            label: entry.label.clone(),
            supply_cost: entry.build_cost_supplies,
            effect: entry.effect_label(),
        });

    // P2: Build menu popup
    vm.build_menu =
        if let bd_core::colony::stations::BuildInteraction::Selecting { selected_station } = *build
        {
            let options: Vec<(String, i32, String)> = station_catalog
                .entries()
                .iter()
                .map(|bp| {
                    (
                        bp.label.to_string(),
                        bp.build_cost_supplies,
                        bp.effect_label(),
                    )
                })
                .collect();
            Some(BuildMenuVm {
                options,
                selected: station_catalog
                    .entries()
                    .iter()
                    .position(|entry| entry.station_type == selected_station)
                    .unwrap_or(0),
                available_supplies: colony_resources
                    .pools
                    .get(PoolKind::Supplies)
                    .map_or(0, |pool| pool.current),
            })
        } else {
            None
        };
}

/// Build the inventory container view model for the player.
#[allow(clippy::type_complexity)] // Inventory projection reads optional item facets in one query.
fn build_container_vm(
    player: Query<(Entity, Option<&bd_core::spatial::EntityScope>), With<Player>>,
    items: Query<(
        Entity,
        Option<&Name>,
        Option<&Item>,
        Option<&bd_core::spatial::EntityScope>,
    )>,
    contained_in: Query<&ContainedIn>,
    equipped_by: Query<&EquippedBy>,
    mode: Res<bd_core::spatial::GameMode>,
    mut vm: ResMut<ContainerViewModel>,
) {
    let Some((player_entity, _)) = player.iter().find(|(_, scope)| scope_active(*scope, *mode))
    else {
        vm.items.clear();
        return;
    };

    // Find items in player's inventory (ContainedIn → player)
    let mut entries: Vec<ItemEntryVm> = Vec::new();
    for (entity, name, _item, scope) in items.iter() {
        if !scope_active(scope, *mode) {
            continue;
        }
        // Check if this item belongs to the player
        let is_contained = contained_in
            .get(entity)
            .ok()
            .is_some_and(|c| c.0 == player_entity);
        let is_equipped = equipped_by
            .get(entity)
            .ok()
            .is_some_and(|e| e.0 == player_entity);

        if is_contained || is_equipped {
            entries.push(ItemEntryVm {
                name: name
                    .map(|n| n.0.clone())
                    .unwrap_or_else(|| "Unknown".into()),
                equipped: is_equipped,
                usable: is_contained, // contained items can be used
            });
        }
    }

    vm.items = entries;
}

fn scope_active(
    scope: Option<&bd_core::spatial::EntityScope>,
    mode: bd_core::spatial::GameMode,
) -> bool {
    scope.is_none_or(|scope| scope.is_active(mode))
}

/// Build the event view model from the CurrentEvent resource.
fn build_event_vm(
    current: Option<Res<bd_core::events::CurrentEvent>>,
    registry: Option<Res<bd_core::events::EventRegistry>>,
    mut vm: ResMut<EventViewModel>,
) {
    let (Some(current), Some(registry)) = (current, registry) else {
        vm.active = false;
        return;
    };
    if !current.is_active() {
        vm.active = false;
        return;
    }
    if let Some(event_def) = registry.get(&current.event_id) {
        if let Some(node) = event_def.nodes.get(&current.node_id) {
            vm.speaker = node.speaker.clone();
            vm.text = node.text.clone();
            vm.choices = node.choices.iter().map(|c| c.label.clone()).collect();
            vm.active = true;
            return;
        }
    }
    vm.active = false;
}

#[cfg(test)]
mod tests {
    use super::*;
    use bd_core::pools::Pool;
    use bevy_app::App;

    fn test_app() -> App {
        let mut app = App::new();
        // Minimal plugins needed for schedule execution
        app.add_plugins(bd_core::BdCorePlugin);
        app.insert_resource(bd_core::colony::production::ColonyResources::default());
        app.insert_resource(crate::commands::CommandBindings::default());
        *app.world_mut().resource_mut::<bd_core::spatial::GameMode>() =
            bd_core::spatial::GameMode::Tactical;
        // Insert all view model resources
        app.insert_resource(StatsViewModel::default());
        app.insert_resource(ActionListViewModel::default());
        app.insert_resource(MapViewModel::default());
        app.add_systems(
            bevy_app::Update,
            (build_stats_vm, build_action_list_vm, build_map_vm).in_set(BdSet::ViewModelBuild),
        );
        app
    }

    #[test]
    fn stats_view_model_contains_hp_ap() {
        let mut app = test_app();
        app.world_mut().spawn((
            Player,
            Position { x: 5, y: 5 },
            Pools::new(vec![
                Pool::new(PoolKind::Health, 15, 0, 20),
                Pool::new(PoolKind::ActionPoints, 2, 0, 3),
            ]),
        ));
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        app.update();
        let vm = app.world().resource::<StatsViewModel>();
        assert_eq!(vm.hp_current, 15);
        assert_eq!(vm.hp_max, 20);
        assert_eq!(vm.ap_current, 2);
        assert_eq!(vm.ap_max, 3);
    }

    #[test]
    fn action_list_contains_move_wait_attack_guard() {
        let mut app = test_app();
        app.world_mut().spawn((
            Player,
            Position { x: 5, y: 5 },
            Pools::new(vec![Pool::new(PoolKind::ActionPoints, 3, 0, 3)]),
        ));
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        app.update();
        let labels: Vec<&str> = app
            .world()
            .resource::<ActionListViewModel>()
            .actions
            .iter()
            .map(|a| a.label.as_str())
            .collect();
        assert!(labels.contains(&"Move"));
        assert!(labels.contains(&"Wait"));
        assert!(labels.contains(&"Attack"));
        assert!(labels.contains(&"Guard"));
    }

    #[test]
    fn disabled_action_contains_denial_reason() {
        let mut app = test_app();
        app.world_mut().spawn((
            Player,
            Position { x: 5, y: 5 },
            Pools::new(vec![Pool::new(PoolKind::ActionPoints, 0, 0, 3)]),
        ));
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        app.update();
        let vm = app.world().resource::<ActionListViewModel>();
        let attack = vm.actions.iter().find(|a| a.label == "Attack").unwrap();
        assert!(!attack.enabled);
        assert!(attack.denial_reason.is_some());
    }

    #[test]
    fn extraction_action_reflects_exit_location_without_writing_a_hint() {
        let mut app = test_app();
        let exit = Position { x: 5, y: 5 };
        app.world_mut().spawn((
            Player,
            exit,
            Pools::new(vec![Pool::new(PoolKind::ActionPoints, 3, 0, 3)]),
        ));
        app.world_mut().spawn((ExitTile, exit));
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        let log_before = app.world().resource::<GameLog>().iter().count();

        app.update();

        let extract = app
            .world()
            .resource::<ActionListViewModel>()
            .actions
            .iter()
            .find(|action| action.label == "Extract")
            .expect("tactical action list must expose extraction");
        assert!(extract.enabled);
        assert_eq!(app.world().resource::<GameLog>().iter().count(), log_before);
    }

    #[test]
    fn map_view_model_contains_tiles() {
        let mut app = test_app();
        app.world_mut().spawn((
            Player,
            Position { x: 5, y: 5 },
            Pools::new(vec![Pool::new(PoolKind::ActionPoints, 3, 0, 3)]),
        ));
        app.world_mut()
            .insert_resource(SmokeMap::default_smoke_map());
        app.update();
        let vm = app.world().resource::<MapViewModel>();
        // Tactical mode projects the active dungeon map.
        assert_eq!(vm.width, 20);
        assert_eq!(vm.player_pos, Some(Position { x: 5, y: 5 }));
    }

    #[test]
    fn widgets_can_render_from_view_models() {
        let mut app = test_app();
        app.world_mut().spawn((
            Player,
            Position { x: 5, y: 5 },
            Pools::new(vec![
                Pool::new(PoolKind::Health, 20, 0, 20),
                Pool::new(PoolKind::ActionPoints, 3, 0, 3),
            ]),
        ));
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        app.update();
        let stats = app.world().resource::<StatsViewModel>();
        assert!(stats.hp_max > 0);
        assert!(stats.ap_max > 0);
        let actions = app.world().resource::<ActionListViewModel>();
        assert!(!actions.actions.is_empty());
        let map = app.world().resource::<MapViewModel>();
        assert!(map.width > 0);
    }

    #[test]
    fn enemy_glyph_maps_by_name() {
        let mut app = test_app();
        // Spawn a Rat enemy at (3,3)
        let _rat = app
            .world_mut()
            .spawn((
                Position { x: 3, y: 3 },
                bd_core::components::BlocksMovement,
                bd_core::components::Name("Rat".into()),
            ))
            .id();
        // Spawn a Skeleton at (5,5)
        let _skeleton = app
            .world_mut()
            .spawn((
                Position { x: 5, y: 5 },
                bd_core::components::BlocksMovement,
                bd_core::components::Name("Skeleton".into()),
            ))
            .id();
        // Spawn an unnamed enemy at (7,7)
        let _unknown = app
            .world_mut()
            .spawn((Position { x: 7, y: 7 }, bd_core::components::BlocksMovement))
            .id();

        // Ensure map resource is set (test_app may have left default)
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));

        app.update();

        let vm = app.world().resource::<MapViewModel>();
        let enemies = vm
            .visuals
            .iter()
            .filter(|visual| visual.token == crate::visual::VisualToken::Enemy)
            .collect::<Vec<_>>();
        assert_eq!(
            enemies.len(),
            3,
            "Should find 3 enemy positions, got {:?}",
            enemies
        );
        // Find the glyph for the Rat at (3,3)
        let rat_glyph = enemies
            .iter()
            .find(|visual| visual.position == Position { x: 3, y: 3 })
            .and_then(|visual| visual.glyph);
        assert_eq!(rat_glyph, Some('r'), "Rat should map to glyph 'r'");
        // Find the glyph for the Skeleton at (5,5)
        let skel_glyph = enemies
            .iter()
            .find(|visual| visual.position == Position { x: 5, y: 5 })
            .and_then(|visual| visual.glyph);
        assert_eq!(skel_glyph, Some('S'), "Skeleton should map to glyph 'S'");
        // Unknown enemy should be 'E'
        let unknown_glyph = enemies
            .iter()
            .find(|visual| visual.position == Position { x: 7, y: 7 })
            .and_then(|visual| visual.glyph);
        assert_eq!(
            unknown_glyph,
            Some('E'),
            "Unknown enemy should default to 'E'"
        );
    }
}
