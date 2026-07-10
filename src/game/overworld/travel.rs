//! Overworld travel — movement between nodes, resource consumption, encounters.

use bevy::prelude::*;
use rand::RngExt;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::core::abilities::SprintCooldown;
use crate::core::components::{Player, Position, TileKind};
use crate::core::gamelog::{FeedbackEvent, GameLog, LogColor, UxMessage};
use crate::core::inventory::{Equipment, Inventory, RangedWeaponState};
use crate::core::movement::MapTiles;
use crate::core::perks::PlayerPerks;
use crate::core::resources::{ResourceKind, ShelterResources};
use crate::core::sanity::RaidExposure;
use crate::core::save::{self, PlayerSnapshot};
use crate::core::state::AppState;
use crate::core::stats::{CombatStats, EntityName, PlayerProgression};
use crate::core::turn::GameTime;

use super::weather::{self, Weather};

/// Resource tracking overworld travel state.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Resource)]
pub struct TravelState {
    pub from_node: usize,
    pub to_node: usize,
    pub distance_remaining: f32,
    pub day: u32,
    pub current_weather: Weather,
    pub world_seed: u64,
    #[serde(default)]
    pub encounters_seen: u32,
}

/// Encounter types during travel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncounterType {
    Hostile,
    Scavenge,
    Nothing,
}

pub const STARVATION_HP_DAMAGE: i32 = 2;
pub const STARVATION_EXPOSURE: u32 = 1;
pub const DEHYDRATION_HP_DAMAGE: i32 = 4;
pub const DEHYDRATION_EXPOSURE: u32 = 2;
const OFF_GATE_GUIDANCE_COOLDOWN_TURNS: u32 = 2;

fn should_emit_off_gate_guidance(last_turn: Option<u32>, current_turn: u32) -> bool {
    match last_turn {
        None => true,
        Some(last_turn) => {
            current_turn.saturating_sub(last_turn) >= OFF_GATE_GUIDANCE_COOLDOWN_TURNS
        }
    }
}

fn emit_off_gate_guidance_if_needed(
    log: &mut GameLog,
    current_turn: u32,
    last_guidance_turn: &mut Option<u32>,
) {
    if !should_emit_off_gate_guidance(*last_guidance_turn, current_turn) {
        return;
    }

    log.push_ux_message(UxMessage::ColonyGateEnterHint, current_turn);
    *last_guidance_turn = Some(current_turn);
}

/// Roll encounter based on distance from shelter.
pub fn roll_encounter(distance_from_shelter: f32, rng: &mut impl rand::Rng) -> EncounterType {
    let chance = if distance_from_shelter < 4.0 {
        5 // 5% near shelter
    } else if distance_from_shelter < 8.0 {
        15 // 15% mid-range
    } else {
        25 // 25% far
    };

    let roll = rng.random_range(0..100u32);
    if roll < chance {
        if rng.random_range(0..2u32) == 0 {
            EncounterType::Hostile
        } else {
            EncounterType::Scavenge
        }
    } else {
        EncounterType::Nothing
    }
}

/// System: process one day of travel, consuming resources and checking encounters.
pub fn process_travel_day(
    mut travel: ResMut<TravelState>,
    mut resources: ResMut<ShelterResources>,
    mut log: ResMut<GameLog>,
    mut player_query: Query<(Option<&mut CombatStats>, Option<&mut RaidExposure>), With<Player>>,
    time: Res<GameTime>,
) {
    // Roll new weather for the day
    travel.current_weather = weather::roll_weather(travel.world_seed, travel.day);

    if travel.current_weather.blocks_travel() {
        log.push(
            format!("{}! Travel blocked.", travel.current_weather.name()),
            LogColor::Status,
            time.turn,
        );
        travel.day += 1;
        return;
    }

    // Apply weather sanity pressure
    let pressure = travel.current_weather.sanity_pressure();
    if pressure > 0
        && let Ok((_, exposure)) = player_query.single_mut() {
            if let Some(mut exposure) = exposure {
                exposure.add(pressure);
            }
            log.push(
                format!(
                    "The {} wears on your mind. (+{} exposure)",
                    travel.current_weather.name(),
                    pressure
                ),
                LogColor::Status,
                time.turn,
            );
        }

    // Consume food and water
    if !resources.try_consume(ResourceKind::Food, 1) {
        let mut hp_loss = 0;
        let mut exposure_gain = 0;
        if let Ok((stats, exposure)) = player_query.single_mut() {
            if let Some(mut stats) = stats {
                stats.hp = (stats.hp - STARVATION_HP_DAMAGE).max(0);
                hp_loss = STARVATION_HP_DAMAGE;
            }
            if let Some(mut exposure) = exposure {
                exposure.add(STARVATION_EXPOSURE);
                exposure_gain = STARVATION_EXPOSURE;
            }
        }
        log.push_feedback(FeedbackEvent::TravelNoFood, time.turn);
        log.push(
            format!("Travel penalties: -{hp_loss} HP, +{exposure_gain} exposure."),
            LogColor::System,
            time.turn,
        );
    }
    if !resources.try_consume(ResourceKind::Water, 1) {
        let mut hp_loss = 0;
        let mut exposure_gain = 0;
        if let Ok((stats, exposure)) = player_query.single_mut() {
            if let Some(mut stats) = stats {
                stats.hp = (stats.hp - DEHYDRATION_HP_DAMAGE).max(0);
                hp_loss = DEHYDRATION_HP_DAMAGE;
            }
            if let Some(mut exposure) = exposure {
                exposure.add(DEHYDRATION_EXPOSURE);
                exposure_gain = DEHYDRATION_EXPOSURE;
            }
        }
        log.push_feedback(FeedbackEvent::TravelNoWater, time.turn);
        log.push(
            format!("Travel penalties: -{hp_loss} HP, +{exposure_gain} exposure."),
            LogColor::System,
            time.turn,
        );
    }

    // Advance distance
    let speed = travel.current_weather.travel_speed();
    travel.distance_remaining -= speed;
    travel.day += 1;

    // Roll encounter with deterministic RNG seeded from world_seed + day
    let distance_from_shelter = travel.day as f32 * 2.0;
    let encounter_seed = travel
        .world_seed
        .wrapping_add(travel.day as u64)
        .wrapping_mul(7919);
    let mut rng = ChaCha8Rng::seed_from_u64(encounter_seed);
    let encounter = roll_encounter(distance_from_shelter, &mut rng);

    match encounter {
        EncounterType::Hostile => {
            travel.encounters_seen += 1;
            log.push(
                "Ambush! Hostile survivors attack!",
                LogColor::EnemyHit,
                time.turn,
            );
            resources.try_consume(ResourceKind::Food, 1);
            log.push(
                "Fought them off, but lost supplies.",
                LogColor::EnemyHit,
                time.turn,
            );
        }
        EncounterType::Scavenge => {
            travel.encounters_seen += 1;
            let gain = rng.random_range(1..=2u32);
            let kind = if rng.random_range(0..2u32) == 0 {
                ResourceKind::Food
            } else {
                ResourceKind::Water
            };
            resources.add(kind, gain);
            log.push(
                format!("Found an abandoned cache! +{gain} {kind:?}."),
                LogColor::PlayerHit,
                time.turn,
            );
        }
        EncounterType::Nothing => {}
    }

    log.push(
        format!(
            "Day {} — {} ({})",
            travel.day,
            travel.current_weather.name(),
            if travel.distance_remaining > 0.0 {
                "traveling..."
            } else {
                "arrived!"
            }
        ),
        LogColor::System,
        time.turn,
    );
}

/// Check if travel is complete.
pub fn check_travel_complete(travel: Option<Res<TravelState>>) -> bool {
    travel.is_some_and(|t| t.distance_remaining <= 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::stats::{SkillId, SkillState};
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use std::collections::HashMap;

    #[test]
    fn test_encounter_rates() {
        let mut rng = ChaCha8Rng::seed_from_u64(99);
        let samples = 10_000u32;

        let mut near_encounters = 0u32;
        let mut mid_encounters = 0u32;
        let mut far_encounters = 0u32;

        for _ in 0..samples {
            if roll_encounter(2.0, &mut rng) != EncounterType::Nothing {
                near_encounters += 1;
            }
            if roll_encounter(6.0, &mut rng) != EncounterType::Nothing {
                mid_encounters += 1;
            }
            if roll_encounter(10.0, &mut rng) != EncounterType::Nothing {
                far_encounters += 1;
            }
        }

        let near_pct = near_encounters as f64 / samples as f64 * 100.0;
        let mid_pct = mid_encounters as f64 / samples as f64 * 100.0;
        let far_pct = far_encounters as f64 / samples as f64 * 100.0;

        assert!(
            (2.0..=8.0).contains(&near_pct),
            "Near ~5% but was {near_pct:.1}%"
        );
        assert!(
            (10.0..=20.0).contains(&mid_pct),
            "Mid ~15% but was {mid_pct:.1}%"
        );
        assert!(
            (20.0..=30.0).contains(&far_pct),
            "Far ~25% but was {far_pct:.1}%"
        );
    }

    #[test]
    fn test_resource_consumption() {
        let mut app = App::new();
        app.insert_resource(ShelterResources::new_game());
        app.insert_resource(GameLog::default());
        app.insert_resource(GameTime { turn: 1 });
        app.insert_resource(TravelState {
            from_node: 0,
            to_node: 1,
            distance_remaining: 3.0,
            day: 0,
            current_weather: Weather::Clear,
            world_seed: 1,
            encounters_seen: 0,
        });

        app.add_systems(Update, process_travel_day);
        app.update();

        let res = app.world().resource::<ShelterResources>();
        assert_eq!(res.food, 9, "Should consume 1 food per travel day");
        assert_eq!(res.water, 9, "Should consume 1 water per travel day");
    }

    #[test]
    fn test_encounters_modify_resources_over_many_days() {
        // Run many travel days with a far-travel seed to trigger encounters.
        // With high day numbers, distance_from_shelter is large → 25% encounter rate.
        let mut app = App::new();
        app.insert_resource(ShelterResources {
            food: 200,
            water: 200,
            scrap: 0,
            medicine: 0,
            ammo: 0,
        });
        app.insert_resource(GameLog::default());
        app.insert_resource(GameTime { turn: 50 });
        app.insert_resource(TravelState {
            from_node: 0,
            to_node: 1,
            distance_remaining: 500.0,
            day: 50, // far from shelter → high encounter chance
            current_weather: Weather::Clear,
            world_seed: 42,
            encounters_seen: 0,
        });

        app.add_systems(Update, process_travel_day);

        // Run 100 travel days
        for _ in 0..100 {
            app.update();
        }

        let res = app.world().resource::<ShelterResources>();
        let travel = app.world().resource::<TravelState>();

        // Base consumption: 100 food + 100 water from travel alone.
        // Hostile encounters consume 1 extra food each.
        // Scavenge encounters add 1-2 food or water.
        // With 25% encounter rate over 100 days, we should see some encounters.
        assert!(
            travel.encounters_seen > 0,
            "Should have seen at least one encounter over 100 days at far distance"
        );

        // Verify resources diverged from simple 200 - 100 = 100 baseline.
        // Either food < 100 (hostile took extra) or food/water > 100 (scavenge added).
        let food_changed = res.food != 100;
        let water_changed = res.water != 100;
        assert!(
            food_changed || water_changed,
            "Encounters should modify resources: food={}, water={}",
            res.food,
            res.water,
        );
    }

    #[test]
    fn test_ashfall_increases_raid_exposure() {
        // Use a seed+day combo that produces Ashfall weather.
        // Ashfall has sanity_pressure() == 1.
        // Brute-force find a (seed, day) that rolls Ashfall.
        let (seed, start_day) = (0..1000u64)
            .flat_map(|s| (1..100u32).map(move |d| (s, d)))
            .find(|&(s, d)| weather::roll_weather(s, d) == Weather::Ashfall)
            .expect("should find an Ashfall day");

        let mut app = App::new();
        app.insert_resource(ShelterResources::new_game());
        app.insert_resource(GameLog::default());
        app.insert_resource(GameTime { turn: start_day });
        app.insert_resource(TravelState {
            from_node: 0,
            to_node: 1,
            distance_remaining: 10.0,
            day: start_day,
            current_weather: Weather::Clear,
            world_seed: seed,
            encounters_seen: 0,
        });

        // Spawn a player with RaidExposure
        app.world_mut().spawn((Player, RaidExposure::default()));

        app.add_systems(Update, process_travel_day);
        app.update();

        let exposure = app
            .world_mut()
            .query_filtered::<&RaidExposure, With<Player>>()
            .single(app.world())
            .expect("player should exist");
        assert!(
            exposure.current >= 1,
            "Ashfall should add ≥1 exposure, got {}",
            exposure.current
        );
    }

    #[test]
    fn test_process_travel_day_logs_use_universal_game_time() {
        let mut app = App::new();
        app.insert_resource(ShelterResources::new_game());
        app.insert_resource(GameLog::default());
        app.insert_resource(GameTime { turn: 42 });
        app.insert_resource(TravelState {
            from_node: 0,
            to_node: 1,
            distance_remaining: 3.0,
            day: 1,
            current_weather: Weather::Clear,
            world_seed: 1,
            encounters_seen: 0,
        });

        app.add_systems(Update, process_travel_day);
        app.update();

        let log = app.world().resource::<GameLog>();
        let last = log.last_n(1).first().expect("travel log should exist");
        assert_eq!(last.turn, 42);
    }

    #[test]
    fn test_missing_supplies_apply_real_attrition() {
        let (seed, start_day) = (0..1000u64)
            .flat_map(|s| (1..100u32).map(move |d| (s, d)))
            .find(|&(s, d)| weather::roll_weather(s, d) == Weather::Clear)
            .expect("should find a clear travel day");

        let mut app = App::new();
        app.insert_resource(ShelterResources {
            food: 0,
            water: 0,
            scrap: 0,
            medicine: 0,
            ammo: 0,
        });
        app.insert_resource(GameLog::default());
        app.insert_resource(GameTime { turn: start_day });
        app.insert_resource(TravelState {
            from_node: 0,
            to_node: 1,
            distance_remaining: 3.0,
            day: start_day,
            current_weather: Weather::Clear,
            world_seed: seed,
            encounters_seen: 0,
        });
        app.world_mut().spawn((
            Player,
            CombatStats {
                hp: 20,
                hp_max: 20,
                speed: 1,
                ar: 0,
                md: 0,
                skills: HashMap::<SkillId, SkillState>::new(),
            },
            RaidExposure::default(),
        ));

        app.add_systems(Update, process_travel_day);
        app.update();

        let mut query = app
            .world_mut()
            .query_filtered::<(&CombatStats, &RaidExposure), With<Player>>();
        let (stats, exposure) = query.single(app.world()).expect("player should exist");
        assert_eq!(stats.hp, 20 - STARVATION_HP_DAMAGE - DEHYDRATION_HP_DAMAGE);
        assert_eq!(exposure.current, STARVATION_EXPOSURE + DEHYDRATION_EXPOSURE);

        let log = app.world().resource::<GameLog>();
        assert!(
            log.entries()
                .iter()
                .any(|entry| entry.text.contains("No food for travel! Starving.")),
            "starvation warning should be logged"
        );
        assert!(
            log.entries()
                .iter()
                .any(|entry| entry.text.contains("No water for travel! Dehydrating.")),
            "dehydration warning should be logged"
        );
    }
}

/// Transition from Colony → Overworld when the player is on the gate tile and presses Enter.
/// Moved here from main.rs for SRP (Phase E.4).
pub fn enter_overworld_from_colony(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
    player_q: Query<
        (
            &Position,
            &CombatStats,
            &Inventory,
            &Equipment,
            &RangedWeaponState,
            &RaidExposure,
            &PlayerPerks,
            &PlayerProgression,
            Option<&EntityName>,
            &SprintCooldown,
        ),
        With<Player>,
    >,
    map: Option<Res<MapTiles>>,
    active_raid: Option<Res<crate::game::colony::raids::ActiveRaid>>,
    mut shelter_resources: Option<ResMut<ShelterResources>>,
    survivors: Query<Entity, With<crate::game::colony::survivors::Survivor>>,
    stations: Query<&crate::game::colony::stations::Station>,
    mut log: ResMut<GameLog>,
    time: Res<GameTime>,
    mut last_off_gate_guidance_turn: Local<Option<u32>>,
) {
    if !keyboard.just_pressed(KeyCode::Enter) {
        return;
    }
    let Ok((
        pos,
        stats,
        inventory,
        equipment,
        ranged_state,
        sanity,
        perks,
        progression,
        name,
        sprint_cd,
    )) = player_q.single()
    else {
        return;
    };
    let Some(map) = map else { return };
    if let Some(TileKind::StairsUp) = map.get_tile(pos.x, pos.y) {
        commands.insert_resource(PlayerSnapshot(Some(
            save::snapshot_player_state(
                pos,
                stats,
                inventory,
                equipment,
                ranged_state,
                sanity,
                perks,
                progression,
                name,
                sprint_cd.remaining,
            ),
        )));
        if let (Some(raid), Some(resources)) = (active_raid, shelter_resources.as_mut()) {
            crate::game::colony::raids::resolve_raid_away_from_shelter(
                &mut commands,
                raid.as_ref(),
                resources.as_mut(),
                &survivors,
                &stations,
                log.as_mut(),
                time.as_ref(),
            );
        }
        next_state.set(AppState::Overworld);
        return;
    }

    emit_off_gate_guidance_if_needed(
        log.as_mut(),
        time.turn,
        &mut last_off_gate_guidance_turn,
    );
}
