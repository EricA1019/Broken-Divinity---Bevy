#![allow(clippy::explicit_counter_loop, clippy::too_many_arguments)]

//! Raid system — enemy attacks on the shelter.
//!
//! After an opening grace window, raids trigger every `RAID_INTERVAL` shelter
//! ticks. Forecast warnings appear during the last 5 ticks before the raid.
//! For MVP, staying home can still use the placeholder narrated defense while
//! leaving the shelter causes the raid to auto-resolve off-screen and queue a
//! return report.
//!
//! ## Current state (Phase 2 — auto-resolve only)
//! - **Works:** raid chance accumulation, forecast warnings, auto-resolve
//!   based on shelter defense vs raider strength, resource theft, survivor
//!   casualty despawn, escalating difficulty.
//! - **Data ready:** `RaidPhase` enum, `CombatPreset` component, `RaidReport`.
//!
//! ## Phase 3 — playable shelter defense (not yet implemented)
//! - Transition to `AppState::Combat` when the player chooses to defend.
//! - Spawn raider entities on the shelter map with AI behaviors.
//! - Reuse `TurnPhase` loop (same as dungeon combat).
//! - Apply `CombatPreset` to position survivors at stations.
//! - On victory/defeat, transition back to `AppState::Colony`.

use bevy::prelude::*;
use bevy_egui;
use serde::{Deserialize, Serialize};

use crate::core::gamelog::{FeedbackEvent, GameLog, LogColor};
use crate::core::resources::ShelterResources;
use crate::core::turn::GameTime;
use crate::game::colony::stations::{Station, StationType};
use crate::game::colony::survivors::Survivor;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Shelter ticks between raids.
const RAID_INTERVAL: u32 = 50;
/// Extra breathing room before the first raid cycle begins.
const FIRST_RAID_GRACE_TICKS: u32 = 70;

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Tracks raid probability accumulation and tick counter.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Resource)]
pub struct RaidChance {
    pub accumulated: f32,
    pub base_chance: f32,
    pub ticks_since_last_raid: u32,
}

impl Default for RaidChance {
    fn default() -> Self {
        Self {
            accumulated: 0.0,
            base_chance: 0.02,
            ticks_since_last_raid: 0,
        }
    }
}

/// Active raid state — inserted when a raid begins, removed on resolution.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Resource)]
pub struct ActiveRaid {
    pub raider_count: u32,
    pub raider_strength: u32,
    pub casualties: u32,
    pub resources_stolen: u32,
    pub phase: RaidPhase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum RaidPhase {
    Warning,
    Planning,
    InProgress,
    Complete,
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Survivor combat preset for raids.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CombatPreset {
    Flee,
    #[default]
    Defend,
    Support,
    HoldGate,
}

// ---------------------------------------------------------------------------
// Report
// ---------------------------------------------------------------------------

/// Post-raid outcome summary.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct RaidReport {
    pub survivors_lost: u32,
    pub raiders_killed: u32,
    pub resources_stolen: u32,
    pub stations_damaged: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum RaidReportOrigin {
    AwayAutoResolve,
}

/// Deferred raid summary delivered when the player returns to the shelter.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Resource)]
pub struct PendingRaidReport {
    pub origin: RaidReportOrigin,
    pub report: RaidReport,
}

// ---------------------------------------------------------------------------
// Pure helpers
// ---------------------------------------------------------------------------

/// Deterministic auto-resolve: compare shelter defense vs raider strength.
pub fn auto_resolve_raid(shelter_defense: u32, raider_strength: u32) -> RaidReport {
    let ratio = shelter_defense as f32 / raider_strength.max(1) as f32;
    let survivors_lost = if ratio > 0.8 { 0 } else { 1 };
    let resources_stolen = if ratio > 1.0 {
        0
    } else {
        (10.0 * (1.0 - ratio)) as u32
    };
    let raiders_killed = (raider_strength as f32 * ratio.min(1.0)) as u32;
    let stations_damaged = if ratio < 0.5 { 1 } else { 0 };

    RaidReport {
        survivors_lost,
        raiders_killed,
        resources_stolen,
        stations_damaged,
    }
}

fn raid_interval(raid_chance: &RaidChance) -> u32 {
    RAID_INTERVAL
        + if raid_chance.accumulated <= f32::EPSILON {
            FIRST_RAID_GRACE_TICKS
        } else {
            0
        }
}

fn forecast_start(raid_chance: &RaidChance) -> u32 {
    raid_interval(raid_chance).saturating_sub(FORECAST_WARNINGS.len() as u32)
}

fn shelter_defense_rating(
    survivors: &Query<Entity, With<Survivor>>,
    stations: &Query<&Station>,
) -> u32 {
    let survivor_count = survivors.iter().count() as u32;
    let station_defense: u32 = stations
        .iter()
        .filter(|s| {
            matches!(
                s.kind,
                StationType::SecurityCheckpoint | StationType::MilitiaTraining
            )
        })
        .map(|s| s.tier as u32 * 10)
        .sum();

    survivor_count * 10 + station_defense
}

fn apply_raid_report(
    commands: &mut Commands,
    resources: &mut ShelterResources,
    survivors: &Query<Entity, With<Survivor>>,
    report: &RaidReport,
) {
    if report.resources_stolen > 0 {
        let steal_each = report.resources_stolen / 3;
        resources.food = resources.food.saturating_sub(steal_each);
        resources.water = resources.water.saturating_sub(steal_each);
        resources.scrap = resources.scrap.saturating_sub(steal_each);
    }

    let mut killed = 0u32;
    for entity in survivors.iter() {
        if killed >= report.survivors_lost {
            break;
        }
        commands.entity(entity).despawn();
        killed += 1;
    }
}

fn raid_result_summary(prefix: &str, report: &RaidReport) -> String {
    format!(
        "{prefix} — Raiders killed: {}, Resources stolen: {}, Survivors lost: {}",
        report.raiders_killed, report.resources_stolen, report.survivors_lost
    )
}

/// Shelter simulation pauses while a raid is awaiting a player-facing resolution.
pub fn colony_sim_unpaused(active_raid: Option<Res<ActiveRaid>>) -> bool {
    active_raid.is_none()
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

const FORECAST_WARNINGS: [&str; 5] = [
    "Scout tracks spotted near perimeter.",
    "Hostile movement detected to the north.",
    "Raiders appear to be mobilizing.",
    "Perimeter breach imminent!",
    "RAID INCOMING!",
];

/// Increments the tick counter, emits forecast warnings, and inserts an
/// `ActiveRaid` resource when the interval is reached.
pub fn check_raid_trigger(
    mut commands: Commands,
    mut raid_chance: ResMut<RaidChance>,
    mut log: ResMut<GameLog>,
    active_raid: Option<Res<ActiveRaid>>,
    time: Res<GameTime>,
) {
    if active_raid.is_some() {
        return;
    }

    raid_chance.ticks_since_last_raid += 1;
    let tick = raid_chance.ticks_since_last_raid;
    let interval = raid_interval(&raid_chance);
    let forecast_start = forecast_start(&raid_chance);

    // Forecast warnings during the last few ticks before the raid.
    if (forecast_start..interval).contains(&tick) {
        let idx = (tick - forecast_start) as usize;
        if let Some(msg) = FORECAST_WARNINGS.get(idx) {
            log.push_feedback(FeedbackEvent::RaidForecast { message: msg }, time.turn);
        }
    }

    if tick >= interval {
        raid_chance.ticks_since_last_raid = 0;
        // Each raid slightly escalates future strength.
        let strength = 30 + (raid_chance.accumulated * 10.0) as u32;
        raid_chance.accumulated += 0.5;

        commands.insert_resource(ActiveRaid {
            raider_count: 3 + strength / 15,
            raider_strength: strength,
            casualties: 0,
            resources_stolen: 0,
            phase: RaidPhase::Planning,
        });

        log.push("RAIDERS ATTACK THE SHELTER!", LogColor::EnemyHit, time.turn);
    }
}

/// Auto-resolves an active raid, applies losses, and removes the resource.
pub fn resolve_active_raid(
    mut commands: Commands,
    active_raid: Option<Res<ActiveRaid>>,
    mut resources: ResMut<ShelterResources>,
    survivors: Query<Entity, With<Survivor>>,
    stations: Query<&Station>,
    mut log: ResMut<GameLog>,
    time: Res<GameTime>,
) {
    let Some(raid) = active_raid else { return };
    if raid.phase != RaidPhase::InProgress {
        return;
    }

    let report = auto_resolve_raid(
        shelter_defense_rating(&survivors, &stations),
        raid.raider_strength,
    );
    apply_raid_report(&mut commands, &mut resources, &survivors, &report);

    log.push(
        raid_result_summary("Raid resolved", &report),
        LogColor::System,
        time.turn,
    );

    commands.remove_resource::<ActiveRaid>();
}

/// Resolve an active raid because the player left the shelter before facing it.
pub fn resolve_raid_away_from_shelter(
    commands: &mut Commands,
    raid: &ActiveRaid,
    resources: &mut ShelterResources,
    survivors: &Query<Entity, With<Survivor>>,
    stations: &Query<&Station>,
    log: &mut GameLog,
    time: &GameTime,
) {
    let report = auto_resolve_raid(
        shelter_defense_rating(survivors, stations),
        raid.raider_strength,
    );
    apply_raid_report(commands, resources, survivors, &report);
    commands.insert_resource(PendingRaidReport {
        origin: RaidReportOrigin::AwayAutoResolve,
        report,
    });
    commands.remove_resource::<ActiveRaid>();
    log.push(
        "You leave the shelter as the raid closes in. The colony must fend for itself.",
        LogColor::EnemyHit,
        time.turn,
    );
}

/// Deliver a deferred raid summary when the player returns home.
pub fn deliver_pending_raid_report(
    mut commands: Commands,
    pending: Option<Res<PendingRaidReport>>,
    mut log: ResMut<GameLog>,
    time: Res<GameTime>,
) {
    let Some(pending) = pending else { return };
    match pending.origin {
        RaidReportOrigin::AwayAutoResolve => log.push(
            raid_result_summary(
                "While you were away, the shelter weathered a raid",
                &pending.report,
            ),
            LogColor::System,
            time.turn,
        ),
    }
    commands.remove_resource::<PendingRaidReport>();
}

// ---------------------------------------------------------------------------
// UI Action Resource
// ---------------------------------------------------------------------------

/// Action resource for raid UI draw/process split pattern.
#[derive(Resource, Default)]
pub struct RaidUiAction(pub Option<RaidUiChoice>);

#[derive(Clone, Copy, Debug)]
pub enum RaidUiChoice {
    AutoResolve,
    DefendShelter,
}

// ---------------------------------------------------------------------------
// UI Draw System
// ---------------------------------------------------------------------------

/// Draw pre-raid modal when in Planning phase.
pub fn draw_raid_modal(
    mut contexts: bevy_egui::EguiContexts,
    active_raid: Option<Res<ActiveRaid>>,
    mut action: ResMut<RaidUiAction>,
) {
    let Some(raid) = active_raid else { return };
    if raid.phase != RaidPhase::Planning {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };

    bevy_egui::egui::Window::new("RAID INCOMING!")
        .collapsible(false)
        .resizable(false)
        .anchor(bevy_egui::egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("RAID INCOMING!");
                ui.separator();
                ui.label(format!("Raiders: {}", raid.raider_count));
                ui.label(format!("Strength: {}", raid.raider_strength));
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button("Auto-resolve").clicked() {
                        action.0 = Some(RaidUiChoice::AutoResolve);
                    }
                    if ui.button("Defend Shelter").clicked() {
                        action.0 = Some(RaidUiChoice::DefendShelter);
                    }
                });
            });
        });
}

// ---------------------------------------------------------------------------
// Process System
// ---------------------------------------------------------------------------

/// Process raid UI action — auto-resolve or narrated defense.
pub fn process_raid_action(
    mut commands: Commands,
    mut action: ResMut<RaidUiAction>,
    active_raid: Option<ResMut<ActiveRaid>>,
    mut resources: ResMut<ShelterResources>,
    survivors: Query<Entity, With<Survivor>>,
    stations: Query<&Station>,
    mut log: ResMut<GameLog>,
    time: Res<crate::core::turn::GameTime>,
) {
    let Some(choice) = action.0.take() else {
        return;
    };
    let Some(mut raid) = active_raid else { return };
    if raid.phase != RaidPhase::Planning {
        return;
    }

    match choice {
        RaidUiChoice::AutoResolve => {
            raid.phase = RaidPhase::InProgress;
            log.push("Auto-resolving raid...", LogColor::System, time.turn);
        }
        RaidUiChoice::DefendShelter => {
            // Run narrated defense (3-5 rounds based on strength)
            let rounds = 3 + (raid.raider_strength / 30).min(2);
            log.push(
                "You stay to direct the shelter defense.",
                LogColor::System,
                time.turn,
            );

            // Example narration for each round
            let round_events = [
                (
                    "Marcus holds the gate! Raider takes damage.",
                    LogColor::PlayerHit,
                ),
                ("Elena treats the wounded.", LogColor::Status),
                ("Raiders breach the east wall!", LogColor::EnemyHit),
                (
                    "Survivors rally at the security checkpoint!",
                    LogColor::PlayerHit,
                ),
                ("A raider falls to concentrated fire.", LogColor::PlayerHit),
            ];

            for i in 0..rounds as usize {
                if let Some((msg, color)) = round_events.get(i) {
                    log.push(format!("Round {}: {}", i + 1, msg), *color, time.turn);
                }
            }

            // Use the same outcome oracle as auto-resolve
            let report = auto_resolve_raid(
                shelter_defense_rating(&survivors, &stations),
                raid.raider_strength,
            );
            apply_raid_report(&mut commands, &mut resources, &survivors, &report);

            log.push(
                raid_result_summary("Defense complete", &report),
                LogColor::System,
                time.turn,
            );

            commands.remove_resource::<ActiveRaid>();
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use bevy::ecs::world::World;

    #[test]
    fn test_auto_resolve_strong_defense() {
        let report = auto_resolve_raid(100, 30);
        assert_eq!(report.survivors_lost, 0);
        assert_eq!(report.resources_stolen, 0);
        assert_eq!(report.stations_damaged, 0);
    }

    #[test]
    fn test_auto_resolve_weak_defense() {
        let report = auto_resolve_raid(10, 100);
        assert_eq!(report.survivors_lost, 1);
        assert!(report.resources_stolen > 0);
        assert_eq!(report.stations_damaged, 1);
    }

    #[test]
    fn test_auto_resolve_zero_raiders() {
        let report = auto_resolve_raid(50, 0);
        assert_eq!(report.survivors_lost, 0);
        assert_eq!(report.resources_stolen, 0);
        assert_eq!(report.raiders_killed, 0);
    }

    #[test]
    fn test_raid_interval_triggers() {
        let mut app = App::new();
        app.insert_resource(RaidChance {
            accumulated: 0.5,
            ..default()
        });
        app.init_resource::<GameLog>();
        app.init_resource::<GameTime>();
        app.add_systems(Update, check_raid_trigger);

        // Run 49 ticks — no raid yet.
        for _ in 0..49 {
            app.update();
        }
        assert!(app.world().get_resource::<ActiveRaid>().is_none());

        // Tick 50 triggers the raid.
        app.update();
        assert!(app.world().get_resource::<ActiveRaid>().is_some());
    }

    #[test]
    fn test_first_raid_grace_delays_trigger() {
        let mut app = App::new();
        app.init_resource::<RaidChance>();
        app.init_resource::<GameLog>();
        app.init_resource::<GameTime>();
        app.add_systems(Update, check_raid_trigger);

        for _ in 0..(RAID_INTERVAL + FIRST_RAID_GRACE_TICKS - 1) {
            app.update();
        }
        assert!(app.world().get_resource::<ActiveRaid>().is_none());

        app.update();
        assert!(app.world().get_resource::<ActiveRaid>().is_some());
    }

    #[test]
    fn test_forecast_warnings() {
        let mut app = App::new();
        app.insert_resource(RaidChance {
            accumulated: 0.5,
            ..default()
        });
        app.init_resource::<GameLog>();
        app.init_resource::<GameTime>();
        app.add_systems(Update, check_raid_trigger);

        // Run 44 ticks — no warnings yet (ticks_since_last_raid 1..44).
        for _ in 0..44 {
            app.update();
        }
        let log = app.world().resource::<GameLog>();
        assert!(log.entries().is_empty(), "No warnings before tick 45");

        // Ticks 45..49 produce warnings.
        app.update(); // tick 45
        let log = app.world().resource::<GameLog>();
        assert_eq!(log.entries().len(), 1);
        assert!(log.entries()[0].text.contains("Scout tracks"));

        app.update(); // tick 46
        let log = app.world().resource::<GameLog>();
        assert_eq!(log.entries().len(), 2);
        assert!(log.entries()[1].text.contains("Hostile movement"));
    }

    #[test]
    fn test_raid_starts_in_planning_phase() {
        let mut app = App::new();
        app.insert_resource(RaidChance {
            accumulated: 0.5,
            ..default()
        });
        app.init_resource::<GameLog>();
        app.init_resource::<GameTime>();
        app.add_systems(Update, check_raid_trigger);

        // Run 50 ticks to trigger raid.
        for _ in 0..50 {
            app.update();
        }

        let raid = app.world().resource::<ActiveRaid>();
        assert_eq!(
            raid.phase,
            RaidPhase::Planning,
            "Raid should start in Planning phase"
        );
    }

    #[test]
    fn test_raid_warning_logs_use_universal_game_time() {
        let mut app = App::new();
        app.insert_resource(RaidChance {
            accumulated: 0.5,
            ticks_since_last_raid: 44,
            ..default()
        });
        app.insert_resource(GameLog::default());
        app.insert_resource(GameTime { turn: 77 });
        app.add_systems(Update, check_raid_trigger);

        app.update();

        let log = app.world().resource::<GameLog>();
        assert_eq!(log.entries().len(), 1);
        assert_eq!(log.entries()[0].turn, 77);
    }

    #[test]
    fn test_auto_resolve_from_planning() {
        use crate::core::turn::GameTime;

        let mut app = App::new();
        app.init_resource::<GameLog>();
        app.init_resource::<ShelterResources>();
        app.init_resource::<GameTime>();
        app.init_resource::<RaidUiAction>();
        app.insert_resource(ActiveRaid {
            raider_count: 5,
            raider_strength: 30,
            casualties: 0,
            resources_stolen: 0,
            phase: RaidPhase::Planning,
        });
        app.add_systems(Update, process_raid_action);

        // Spawn test survivor.
        app.world_mut().spawn(Survivor);

        // Process auto-resolve action.
        app.world_mut().resource_mut::<RaidUiAction>().0 = Some(RaidUiChoice::AutoResolve);
        app.update();

        // Raid should now be in InProgress phase.
        let raid = app.world().resource::<ActiveRaid>();
        assert_eq!(raid.phase, RaidPhase::InProgress);

        // Add resolve system and verify it removes the raid.
        app.add_systems(Update, resolve_active_raid);
        app.update();
        assert!(
            app.world().get_resource::<ActiveRaid>().is_none(),
            "Raid should be removed after resolution"
        );
    }

    #[test]
    fn test_colony_sim_unpaused_only_without_active_raid() {
        fn check(active_raid: Option<Res<ActiveRaid>>) -> bool {
            colony_sim_unpaused(active_raid)
        }

        let mut world = World::new();
        assert!(world.run_system_once(check).expect("predicate should run"));

        world.insert_resource(ActiveRaid {
            raider_count: 4,
            raider_strength: 30,
            casualties: 0,
            resources_stolen: 0,
            phase: RaidPhase::Planning,
        });
        assert!(!world.run_system_once(check).expect("predicate should run"));
    }

    #[test]
    fn test_narrated_defense_produces_logs() {
        use crate::core::turn::GameTime;

        let mut app = App::new();
        app.init_resource::<GameLog>();
        app.init_resource::<ShelterResources>();
        app.init_resource::<GameTime>();
        app.init_resource::<RaidUiAction>();
        app.insert_resource(ActiveRaid {
            raider_count: 5,
            raider_strength: 30,
            casualties: 0,
            resources_stolen: 0,
            phase: RaidPhase::Planning,
        });
        app.add_systems(Update, process_raid_action);

        // Spawn test survivors.
        for _ in 0..3 {
            app.world_mut().spawn(Survivor);
        }

        // Initial log count.
        let initial_count = app.world().resource::<GameLog>().entries().len();

        // Process defend action.
        app.world_mut().resource_mut::<RaidUiAction>().0 = Some(RaidUiChoice::DefendShelter);
        app.update();

        // Verify logs were added.
        let log = app.world().resource::<GameLog>();
        assert!(
            log.entries().len() > initial_count + 3,
            "Should have at least 3+ new log entries (prep + rounds + result)"
        );

        // Check for round-based narrative.
        let has_round = log.entries().iter().any(|e| e.text.contains("Round"));
        assert!(has_round, "Should have round-based narrative entries");

        // Check defense completion message.
        let has_complete = log
            .entries()
            .iter()
            .any(|e| e.text.contains("Defense complete"));
        assert!(has_complete, "Should have defense completion message");

        // Raid should be removed.
        assert!(
            app.world().get_resource::<ActiveRaid>().is_none(),
            "Raid should be removed after defense"
        );
    }

    #[test]
    fn test_resolve_raid_away_from_shelter_queues_pending_report() {
        let mut app = App::new();
        app.insert_resource(GameLog::default());
        app.insert_resource(GameTime { turn: 42 });
        app.insert_resource(ShelterResources::new_game());

        let raid = ActiveRaid {
            raider_count: 5,
            raider_strength: 80,
            casualties: 0,
            resources_stolen: 0,
            phase: RaidPhase::Planning,
        };
        app.insert_resource(raid.clone());
        for _ in 0..2 {
            app.world_mut().spawn(Survivor);
        }
        app.world_mut().spawn(Station {
            kind: StationType::SecurityCheckpoint,
            tier: 1,
            worker_slots: 1,
            workers_assigned: 1,
        });

        let _ = app.world_mut().run_system_once(
            |mut commands: Commands,
             active_raid: Res<ActiveRaid>,
             mut resources: ResMut<ShelterResources>,
             survivors: Query<Entity, With<Survivor>>,
             stations: Query<&Station>,
             mut log: ResMut<GameLog>,
             time: Res<GameTime>| {
                resolve_raid_away_from_shelter(
                    &mut commands,
                    &active_raid,
                    &mut resources,
                    &survivors,
                    &stations,
                    &mut log,
                    &time,
                );
            },
        );

        assert!(app.world().get_resource::<ActiveRaid>().is_none());
        let pending = app.world().resource::<PendingRaidReport>();
        assert_eq!(pending.origin, RaidReportOrigin::AwayAutoResolve);
    }

    #[test]
    fn test_deliver_pending_raid_report_logs_once_and_clears_resource() {
        let mut app = App::new();
        app.insert_resource(GameLog::default());
        app.insert_resource(GameTime { turn: 99 });
        app.insert_resource(PendingRaidReport {
            origin: RaidReportOrigin::AwayAutoResolve,
            report: RaidReport {
                survivors_lost: 1,
                raiders_killed: 2,
                resources_stolen: 3,
                stations_damaged: 0,
            },
        });
        app.add_systems(Update, deliver_pending_raid_report);

        app.update();

        let log = app.world().resource::<GameLog>();
        assert!(log.entries().iter().any(|entry| {
            entry
                .text
                .contains("While you were away, the shelter weathered a raid")
        }));
        assert!(app.world().get_resource::<PendingRaidReport>().is_none());
    }
}
