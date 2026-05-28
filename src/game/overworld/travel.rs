#![allow(clippy::too_many_arguments, clippy::type_complexity)]

//! Overworld travel — movement between nodes, resource consumption, encounters.

use bevy::prelude::*;
use rand::RngExt;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::core::abilities::SprintCooldown;
use crate::core::components::{Player, Position, TileKind};
use crate::core::gamelog::{GameLog, LogColor, UxMessage};
use crate::core::inventory::{Equipment, Inventory, RangedWeaponState};
use crate::core::movement::MapTiles;
use crate::core::perks::PlayerPerks;
use crate::core::resources::{ResourceKind, ShelterResources};
use crate::core::sanity::RaidExposure;
use crate::core::save;
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
    mut player_query: Query<&mut RaidExposure, With<Player>>,
) {
    // Roll new weather for the day
    travel.current_weather = weather::roll_weather(travel.world_seed, travel.day);

    if travel.current_weather.blocks_travel() {
        log.push(
            format!("{}! Travel blocked.", travel.current_weather.name()),
            LogColor::Status,
            travel.day,
        );
        travel.day += 1;
        return;
    }

    // Apply weather sanity pressure
    let pressure = travel.current_weather.sanity_pressure();
    if pressure > 0 {
        if let Ok(mut exposure) = player_query.single_mut() {
            exposure.add(pressure);
            log.push(
                format!(
                    "The {} wears on your mind. (+{} exposure)",
                    travel.current_weather.name(),
                    pressure
                ),
                LogColor::Status,
                travel.day,
            );
        }
    }

    // Consume food and water
    if !resources.try_consume(ResourceKind::Food, 1) {
        log.push(
            "No food for travel! Starving.",
            LogColor::EnemyHit,
            travel.day,
        );
    }
    if !resources.try_consume(ResourceKind::Water, 1) {
        log.push(
            "No water for travel! Dehydrating.",
            LogColor::EnemyHit,
            travel.day,
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
                travel.day,
            );
            resources.try_consume(ResourceKind::Food, 1);
            log.push(
                "Fought them off, but lost supplies.",
                LogColor::EnemyHit,
                travel.day,
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
                travel.day,
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
        travel.day,
    );
}

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
    log: Option<ResMut<GameLog>>,
    time: Option<Res<GameTime>>,
) {
    if !keyboard.just_pressed(KeyCode::Enter) {
        return;
    }

    let Ok((
        position,
        stats,
        inventory,
        equipment,
        ranged_state,
        sanity,
        perks,
        progression,
        name,
        sprint_cooldown,
    )) = player_q.single()
    else {
        return;
    };

    let Some(map) = map else {
        return;
    };

    if let Some(TileKind::StairsUp) = map.get_tile(position.x, position.y) {
        commands.insert_resource(save::PlayerSnapshot(Some(save::snapshot_player_state(
            position,
            stats,
            inventory,
            equipment,
            ranged_state,
            sanity,
            perks,
            progression,
            name,
            sprint_cooldown.remaining,
        ))));
        next_state.set(AppState::Overworld);
        return;
    }

    if let Some(mut log) = log {
        let turn = time.as_ref().map_or(0, |time| time.turn);
        log.push_ux_message(UxMessage::ColonyGateEnterHint, turn);
    }
}

/// Check if travel is complete.
pub fn check_travel_complete(travel: Option<Res<TravelState>>) -> bool {
    travel.is_some_and(|t| t.distance_remaining <= 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

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
}
