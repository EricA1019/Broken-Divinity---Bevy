//! Integration smoke tests for MVP readiness.
//!
//! These tests verify end-to-end flows work before handing to testers.
//! Tests use `World::new()` + `run_system_once` pattern with `MinimalPlugins`.

#[cfg(test)]
mod smoke {
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::*;
    use std::collections::HashMap;

    use crate::core::abilities::SprintCooldown;
    use crate::core::components::{Player, Position};
    use crate::core::gamelog::GameLog;
    use crate::core::inventory::Inventory;
    use crate::core::items::ItemStack;
    use crate::core::resources::ShelterResources;
    use crate::core::sanity::RaidExposure;
    use crate::core::save::{PendingSurvivorLoad, SaveSurvivor};
    use crate::core::stats::{CombatStats, EntityName};
    use crate::core::turn::{ActionBudget, GameTime, PendingAction, PlayerAction};
    use crate::game::colony::raids::{ActiveRaid, RaidChance, RaidPhase};
    use crate::game::colony::research::CompletedResearch;
    use crate::game::colony::spawn::{ShelterGateMarker, ShelterState, setup_shelter};
    use crate::game::colony::stations::{Station, StationType, station_production};
    use crate::game::colony::survivors::{
        Survivor, SurvivorNeeds, SurvivorTask, consume_shelter_resources, spawn_initial_survivors,
        survivor_ai, tick_survivor_needs,
    };
    use crate::game::overworld::travel::{TravelState, process_travel_day};

    // ── Test 1: Colony Bootstrap ──────────────────────────────────────

    #[test]
    fn test_colony_bootstrap_produces_working_shelter() {
        let mut world = World::new();

        // Insert required resources
        world.insert_resource(crate::core::resources::WorldSeed(42));
        world.insert_resource(GameLog::default());
        world.insert_resource(GameTime { turn: 1 });
        world.insert_resource(Assets::<Image>::default());
        let _ = world.run_system_once(crate::core::tilemap::init_placeholder_tile_atlas);

        // Run setup_shelter
        let _ = world.run_system_once(setup_shelter);

        // Verify ShelterState exists
        assert!(
            world.contains_resource::<ShelterState>(),
            "ShelterState should be created"
        );

        // Verify ShelterResources initialized
        let resources = world.get_resource::<ShelterResources>();
        assert!(
            resources.is_some(),
            "ShelterResources should be initialized"
        );
        let resources = resources.unwrap();
        assert!(resources.food > 0, "Starting food should be > 0");
        assert!(resources.water > 0, "Starting water should be > 0");
        assert!(resources.scrap > 0, "Starting scrap should be > 0");

        // Run spawn_initial_survivors
        let _ = world.run_system_once(spawn_initial_survivors);

        // Verify survivors spawned
        let survivor_count = world
            .query_filtered::<Entity, With<Survivor>>()
            .iter(&world)
            .count();
        assert_eq!(survivor_count, 3, "Should spawn 3 initial survivors");

        // Verify at least one station exists (spawned by setup_shelter)
        let station_count = world.query::<&Station>().iter(&world).count();
        assert!(station_count > 0, "At least one station should be spawned");

        let gate_marker_count = world.query::<&ShelterGateMarker>().iter(&world).count();
        assert_eq!(
            gate_marker_count, 1,
            "shelter should spawn a visible gate marker"
        );
    }

    // ── Test 2: Full Colony Tick ──────────────────────────────────────

    #[test]
    fn test_full_colony_tick() {
        let mut world = World::new();

        // Insert required resources
        world.insert_resource(GameLog::default());
        world.insert_resource(GameTime { turn: 1 });
        world.insert_resource(ShelterResources::new_game());
        world.insert_resource(CompletedResearch::default());

        // Spawn a Cook station with 1 worker
        world.spawn((
            Station {
                kind: StationType::Cook,
                tier: 1,
                worker_slots: 1,
                workers_assigned: 1,
            },
            Position { x: 5, y: 5 },
        ));

        // Spawn a survivor with high needs (won't trigger resource consumption)
        let survivor = world
            .spawn((
                Survivor,
                EntityName {
                    name: "TestSurvivor".to_string(),
                },
                SurvivorNeeds {
                    hunger: 80,
                    thirst: 80,
                    rest: 80,
                },
                SurvivorTask::Working(IVec2::new(5, 5)),
                Position { x: 5, y: 5 },
            ))
            .id();

        // Record initial state
        let initial_food = world.resource::<ShelterResources>().food;
        let initial_needs = world.get::<SurvivorNeeds>(survivor).unwrap().clone();

        // Run colony tick systems
        let _ = world.run_system_once(tick_survivor_needs);
        let _ = world.run_system_once(consume_shelter_resources);
        let _ = world.run_system_once(station_production);
        let _ = world.run_system_once(survivor_ai);

        // Verify needs changed (decay by 1)
        let final_needs = world.get::<SurvivorNeeds>(survivor).unwrap();
        assert!(
            final_needs.hunger < initial_needs.hunger,
            "Hunger should decay"
        );
        assert!(
            final_needs.thirst < initial_needs.thirst,
            "Thirst should decay"
        );

        // Verify food was produced (Cook station with 1 worker produces 1 food)
        let final_food = world.resource::<ShelterResources>().food;
        assert!(
            final_food > initial_food,
            "Food should be produced by staffed Cook station"
        );

        // No crashes = success
    }

    // ── Test 3: Save/Load Roundtrip ───────────────────────────────────

    #[test]
    fn test_save_load_roundtrip_with_survivors() {
        let mut world = World::new();

        // Create saved survivor data
        let saved_survivors = vec![
            SaveSurvivor {
                name: "LoadedSurvivor1".to_string(),
                x: 10,
                y: 15,
                hunger: 42,
                thirst: 58,
                rest: 73,
                task: crate::core::save::SaveSurvivorTask::Idle,
            },
            SaveSurvivor {
                name: "LoadedSurvivor2".to_string(),
                x: 12,
                y: 16,
                hunger: 65,
                thirst: 45,
                rest: 80,
                task: crate::core::save::SaveSurvivorTask::Working { x: 5, y: 5 },
            },
        ];

        // Insert pending load resource
        world.insert_resource(PendingSurvivorLoad(saved_survivors.clone()));

        // Run spawn_initial_survivors (which respects PendingSurvivorLoad)
        let _ = world.run_system_once(spawn_initial_survivors);

        // Verify survivors restored with correct needs
        let mut query =
            world.query_filtered::<(&EntityName, &SurvivorNeeds, &Position), With<Survivor>>();
        let survivors: Vec<_> = query.iter(&world).collect();

        assert_eq!(survivors.len(), 2, "Should restore 2 survivors");

        // Find survivor 1
        let s1 = survivors
            .iter()
            .find(|(name, _, _)| name.name == "LoadedSurvivor1")
            .expect("LoadedSurvivor1 should exist");
        assert_eq!(s1.1.hunger, 42, "Survivor 1 hunger should match");
        assert_eq!(s1.1.thirst, 58, "Survivor 1 thirst should match");
        assert_eq!(s1.1.rest, 73, "Survivor 1 rest should match");
        assert_eq!(s1.2.x, 10, "Survivor 1 x should match");
        assert_eq!(s1.2.y, 15, "Survivor 1 y should match");

        // Find survivor 2
        let s2 = survivors
            .iter()
            .find(|(name, _, _)| name.name == "LoadedSurvivor2")
            .expect("LoadedSurvivor2 should exist");
        assert_eq!(s2.1.hunger, 65, "Survivor 2 hunger should match");
        assert_eq!(s2.2.x, 12, "Survivor 2 x should match");

        // Verify task restored
        let s2_entity = world
            .query_filtered::<Entity, With<Survivor>>()
            .iter(&world)
            .nth(1)
            .unwrap();
        let task = world.get::<SurvivorTask>(s2_entity).unwrap();
        assert!(
            matches!(task, SurvivorTask::Working(_)),
            "Survivor 2 task should be Working"
        );
    }

    // ── Test 4: Raid Flow Planning to Resolve ─────────────────────────

    #[test]
    fn test_raid_flow_planning_to_resolve() {
        let mut world = World::new();

        // Insert required resources
        world.insert_resource(GameLog::default());
        world.insert_resource(GameTime { turn: 1 });
        world.insert_resource(RaidChance {
            accumulated: 1.0,
            base_chance: 1.0, // 100% chance to trigger
            ticks_since_last_raid: 50,
        });

        // Run check_raid_trigger multiple times to trigger a raid
        for _ in 0..10 {
            let _ = world.run_system_once(crate::game::colony::raids::check_raid_trigger);
            if world.contains_resource::<ActiveRaid>() {
                break;
            }
        }

        // Verify raid triggered
        assert!(
            world.contains_resource::<ActiveRaid>(),
            "Raid should be triggered"
        );

        let raid = world.resource::<ActiveRaid>();
        assert!(
            raid.phase == RaidPhase::Planning || raid.phase == RaidPhase::Warning,
            "Raid should start in Warning or Planning phase, got {:?}",
            raid.phase
        );

        // Manually advance raid to InProgress phase
        world.resource_mut::<ActiveRaid>().phase = RaidPhase::InProgress;

        // Insert shelter resources and stations for resolve calculation
        world.insert_resource(ShelterResources::new_game());
        world.spawn(Station {
            kind: StationType::SecurityCheckpoint,
            tier: 1,
            worker_slots: 1,
            workers_assigned: 1,
        });

        // Run resolve_active_raid
        let _ = world.run_system_once(crate::game::colony::raids::resolve_active_raid);

        // Verify raid was resolved and removed
        assert!(
            !world.contains_resource::<ActiveRaid>(),
            "ActiveRaid should be removed after resolution"
        );
    }

    // ── Test 5: Station Build, Staff, Produce Cycle ───────────────────

    #[test]
    fn test_station_build_staff_produce_cycle() {
        let mut world = World::new();

        // Insert required resources
        world.insert_resource(GameLog::default());
        world.insert_resource(GameTime { turn: 1 });
        world.insert_resource(ShelterResources {
            food: 5,
            water: 5,
            scrap: 20,
            medicine: 5,
            ammo: 5,
        });
        world.insert_resource(CompletedResearch::default());

        let initial_food = world.resource::<ShelterResources>().food;

        // Manually build a Cook station (simulating process_colony_action)
        world.spawn((
            Station {
                kind: StationType::Cook,
                tier: 1,
                worker_slots: 1,
                workers_assigned: 1,
            },
            Position { x: 10, y: 10 },
        ));

        // Run station_production
        let _ = world.run_system_once(station_production);

        // Verify food increased
        let final_food = world.resource::<ShelterResources>().food;
        assert!(
            final_food > initial_food,
            "Food should increase from Cook station production"
        );
        assert_eq!(
            final_food,
            initial_food + 1,
            "Cook station should produce 1 food per tick"
        );
    }

    // ── Test 6: Loot Pickup and Consumable Use ────────────────────────

    #[test]
    fn test_loot_pickup_and_consumable_use() {
        let mut world = World::new();

        // Insert required resources
        world.insert_resource(GameLog::default());
        world.insert_resource(GameTime { turn: 1 });
        world.insert_resource(PlayerAction(Some(PendingAction::UseItem(0))));

        // Spawn player with inventory and damaged HP
        let mut inventory_slots: [Option<ItemStack>; 20] = Default::default();
        inventory_slots[0] = Some(ItemStack {
            item_id: "medicine".to_string(),
            quantity: 1,
        });

        let player = world
            .spawn((
                Player,
                Position { x: 5, y: 5 },
                CombatStats {
                    hp: 30,
                    hp_max: 100,
                    speed: 10,
                    ar: 50,
                    md: 20,
                    skills: HashMap::new(),
                },
                Inventory {
                    slots: inventory_slots,
                },
                ActionBudget::new(10),
            ))
            .id();

        let initial_hp = world.get::<CombatStats>(player).unwrap().hp;

        // Run resolve_consumable_use
        let _ = world.run_system_once(crate::game::dungeon::consumables::resolve_consumable_use);

        // Verify HP increased
        let final_hp = world.get::<CombatStats>(player).unwrap().hp;
        assert!(
            final_hp > initial_hp,
            "HP should increase after using medicine"
        );

        // Verify item removed
        let inventory = world.get::<Inventory>(player).unwrap();
        assert!(inventory.slots[0].is_none(), "Medicine should be consumed");

        // Verify action budget decreased
        let budget = world.get::<ActionBudget>(player).unwrap();
        assert_eq!(budget.remaining, 9, "Action budget should decrease by 1");
    }

    // ── Test 7: Travel Consumes Resources and Encounters ──────────────

    #[test]
    fn test_travel_consumes_resources_and_encounters() {
        let mut world = World::new();

        // Insert required resources
        world.insert_resource(GameLog::default());
        world.insert_resource(GameTime { turn: 1 });
        world.insert_resource(TravelState {
            from_node: 0,
            to_node: 3,
            distance_remaining: 10.0,
            day: 1,
            current_weather: crate::game::overworld::weather::Weather::Clear,
            world_seed: 42,
            encounters_seen: 0,
        });
        world.insert_resource(ShelterResources {
            food: 20,
            water: 20,
            scrap: 10,
            medicine: 5,
            ammo: 10,
        });

        // Spawn player with RaidExposure
        world.spawn((Player, Position { x: 0, y: 0 }, RaidExposure::default()));

        let initial_food = world.resource::<ShelterResources>().food;
        let initial_water = world.resource::<ShelterResources>().water;
        let initial_day = world.resource::<TravelState>().day;
        let initial_distance = world.resource::<TravelState>().distance_remaining;

        // Run process_travel_day for 3 days
        for _ in 0..3 {
            let _ = world.run_system_once(process_travel_day);
        }

        // Verify resources consumed
        let final_food = world.resource::<ShelterResources>().food;
        let final_water = world.resource::<ShelterResources>().water;
        assert!(
            final_food != initial_food || final_water != initial_water,
            "Travel should change supplies through consumption or encounters"
        );

        // Verify day incremented
        let final_day = world.resource::<TravelState>().day;
        assert!(
            final_day > initial_day,
            "Day should increment during travel"
        );

        // Verify distance decreased (if weather allowed travel)
        let final_distance = world.resource::<TravelState>().distance_remaining;
        assert!(
            final_distance <= initial_distance,
            "Distance should decrease or stay same (if blocked by weather)"
        );
    }

    // ── Test 8: Sprint and Cooldown Cycle ─────────────────────────────

    #[test]
    fn test_sprint_and_cooldown_cycle() {
        let mut world = World::new();

        // Spawn player with SprintCooldown
        let player = world
            .spawn((
                Player,
                Position { x: 5, y: 5 },
                SprintCooldown { remaining: 0 },
            ))
            .id();

        // Verify cooldown starts at 0
        let cooldown = world.get::<SprintCooldown>(player).unwrap();
        assert_eq!(cooldown.remaining, 0, "Initial cooldown should be 0");

        // Simulate a sprint (set cooldown to 3)
        world.get_mut::<SprintCooldown>(player).unwrap().remaining = 3;

        // Tick cooldown 3 times
        for _ in 0..3 {
            let _ = world.run_system_once(crate::core::turn::tick_sprint_cooldown);
        }

        // Verify cooldown back to 0
        let final_cooldown = world.get::<SprintCooldown>(player).unwrap();
        assert_eq!(
            final_cooldown.remaining, 0,
            "Cooldown should be back to 0 after 3 ticks"
        );
    }

    // ── Test 9: Survivor Death on Critical Needs ──────────────────────

    #[test]
    fn test_survivor_death_on_critical_needs() {
        let mut world = World::new();

        // Insert required resources
        world.insert_resource(GameLog::default());
        world.insert_resource(GameTime { turn: 1 });

        // Spawn a survivor with critical needs (all at 0)
        let survivor = world
            .spawn((
                Survivor,
                EntityName {
                    name: "DyingSurvivor".to_string(),
                },
                SurvivorNeeds {
                    hunger: 0,
                    thirst: 0,
                    rest: 0,
                },
                Position { x: 5, y: 5 },
            ))
            .id();

        // Run survivor_death system
        let _ = world.run_system_once(crate::game::colony::survivors::survivor_death);

        // Verify survivor was despawned
        assert!(
            world.get_entity(survivor).is_err(),
            "Survivor with all needs at 0 should be despawned"
        );
    }

    // ── Test 10: Raid Chance Accumulation ─────────────────────────────

    #[test]
    fn test_raid_chance_accumulation() {
        let mut world = World::new();

        // Insert required resources
        world.insert_resource(GameLog::default());
        world.insert_resource(GameTime { turn: 1 });
        world.insert_resource(RaidChance {
            accumulated: 0.5,
            base_chance: 0.02,
            ticks_since_last_raid: 0,
        });

        // Run check_raid_trigger 60 times (enough to trigger a raid at RAID_INTERVAL=50)
        for _ in 0..60 {
            let _ = world.run_system_once(crate::game::colony::raids::check_raid_trigger);
            if world.contains_resource::<ActiveRaid>() {
                break;
            }
        }

        // Verify raid triggered (at tick 50)
        let raid_triggered = world.contains_resource::<ActiveRaid>();
        assert!(raid_triggered, "Raid should trigger after 50 ticks");

        // Verify accumulated increased when raid triggered
        if raid_triggered {
            let accumulated = world.resource::<RaidChance>().accumulated;
            assert!(
                accumulated > 0.0,
                "Accumulated should increase when raid triggers"
            );
        }

        // Verify ticks_since_last_raid was reset
        let ticks = world.resource::<RaidChance>().ticks_since_last_raid;
        assert_eq!(ticks, 0, "Ticks should reset to 0 after raid triggers");
    }
}

#[cfg(test)]
mod ux_baseline_red {
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::*;
    use std::collections::HashMap;

    use crate::core::abilities::SprintCooldown;
    use crate::core::brp_safety::validate_brp_entity_access;
    use crate::core::components::{Player, Position, TileKind};
    use crate::core::escape::handle_escape_to_menu;
    use crate::core::gamelog::GameLog;
    use crate::core::inventory::{Equipment, Inventory, RangedWeaponState};
    use crate::core::movement::MapTiles;
    use crate::core::perks::PlayerPerks;
    use crate::core::resources::WorldSeed;
    use crate::core::sanity::RaidExposure;
    use crate::core::save::SaveAndQuitRequested;
    use crate::core::save::{self, SaveAppState, SaveGame, SaveGameTime};
    use crate::core::state::AppState;
    use crate::core::stats::{CombatStats, PlayerProgression};
    use crate::core::tilemap::init_placeholder_tile_atlas;
    use crate::core::turn::GameTime;
    use crate::game::colony::raids::{ActiveRaid, RaidPhase};
    use crate::game::colony::spawn::cleanup_shelter;
    use crate::game::colony::spawn::{GateAffordanceConfig, ShelterGateMarker, setup_shelter};
    use crate::game::overworld::travel::enter_overworld_from_colony;
    use crate::ui::colony_panel::primary_colony_cta_label;
    use crate::ui::colony_panel::{
        colony_objective_indicator_text, colony_readability_snapshot,
        colony_top_bar_objective_presentation,
    };
    use crate::ui::help_panel::colony_help_shows_secondary_hints;
    use crate::ui::help_panel::{HelpOpen, toggle_help};
    use crate::ui::inventory_panel::{InventoryOpen, toggle_inventory};
    use crate::ui::menu::primary_menu_cta_label;
    use crate::ui::menu::{
        MenuUiAction, MenuUiChoice, load_affordance_for_save_state, menu_readability_snapshot,
        process_menu_action, seed_helper_text,
    };
    use crate::ui::modal_priority::{
        ModalBlockers, ModalPriorityCoordinator, apply_modal_priority_policy,
    };
    use crate::ui::objective_prompt::{
        ColonyObjectivePromptState, InstructionPriorityPolicy, refresh_colony_objective_prompt,
    };
    use crate::ui::overworld_panel::primary_overworld_cta_label;

    const TEST_TURN: u32 = 17;
    const TEST_MAP_SIZE: usize = 3;
    const TEST_PLAYER_X: i32 = 1;
    const TEST_PLAYER_Y: i32 = 1;
    const TEST_HP: i32 = 10;
    const TEST_SPEED: u8 = 1;
    const TEST_RAIDER_COUNT: u32 = 3;
    const TEST_RAIDER_STRENGTH: u32 = 30;
    const TEST_SAVE_SEED: u64 = 42;
    const TEST_BRP_OPERATION: &str = "qa.inspect_entity";
    const FT_01_SEED: u64 = 202_601;
    const FT_03_SEED: u64 = 202_603;
    const TEST_HELPER_SEED: u64 = 77;
    const KEYWORD_OBJECTIVE: &str = "objective";
    const KEYWORD_GATE: &str = "gate";
    const KEYWORD_TRAVEL: &str = "travel";
    const ST_01_RAPID_TOGGLE_CYCLES: usize = 3;

    fn log_contains_all_terms(log: &GameLog, terms: &[&str]) -> bool {
        log.entries().iter().any(|entry| {
            let text = entry.text.to_lowercase();
            terms.iter().all(|term| text.contains(term))
        })
    }

    fn off_gate_map() -> MapTiles {
        MapTiles::new(vec![vec![TileKind::Floor; TEST_MAP_SIZE]; TEST_MAP_SIZE])
    }

    fn baseline_player_bundle() -> (
        Player,
        Position,
        CombatStats,
        Inventory,
        Equipment,
        RangedWeaponState,
        RaidExposure,
        PlayerPerks,
        PlayerProgression,
        SprintCooldown,
    ) {
        (
            Player,
            Position::new(TEST_PLAYER_X, TEST_PLAYER_Y),
            CombatStats {
                hp: TEST_HP,
                hp_max: TEST_HP,
                speed: TEST_SPEED,
                ar: 0,
                md: 0,
                skills: HashMap::new(),
            },
            Inventory::default(),
            Equipment::default(),
            RangedWeaponState {
                clip_current: 0,
                clip_size: 1,
            },
            RaidExposure::default(),
            PlayerPerks::default(),
            PlayerProgression::default(),
            SprintCooldown { remaining: 0 },
        )
    }

    #[test]
    fn enter_off_gate_emits_guidance() {
        let mut app = App::new();
        app.insert_resource(GameLog::default());
        app.insert_resource(GameTime { turn: TEST_TURN });
        app.insert_resource(MapTiles::new(off_gate_map().tiles));
        app.insert_resource(ButtonInput::<KeyCode>::default());
        app.insert_resource(State::new(AppState::Colony));
        app.insert_resource(NextState::<AppState>::Unchanged);
        app.world_mut().spawn(baseline_player_bundle());
        app.add_systems(Update, enter_overworld_from_colony);

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Enter);
        app.update();

        let next_state = app.world().resource::<NextState<AppState>>();
        assert!(
            matches!(*next_state, NextState::Unchanged),
            "Expected no AppState transition when Enter is pressed off-gate"
        );

        let log = app.world().resource::<GameLog>();
        let guidance_logged = log
            .entries()
            .iter()
            .any(|entry| entry.text.to_lowercase().contains("gate"));

        assert!(
            guidance_logged,
            "Expected explicit off-gate guidance when Enter is pressed away from the gate"
        );
    }

    #[test]
    fn enter_on_gate_transitions_to_overworld() {
        let mut app = App::new();
        app.insert_resource(GameLog::default());
        app.insert_resource(GameTime { turn: TEST_TURN });
        app.insert_resource(MapTiles::new(vec![
            vec![TileKind::StairsUp; TEST_MAP_SIZE];
            TEST_MAP_SIZE
        ]));
        app.insert_resource(ButtonInput::<KeyCode>::default());
        app.insert_resource(State::new(AppState::Colony));
        app.insert_resource(NextState::<AppState>::Unchanged);
        app.world_mut().spawn(baseline_player_bundle());
        app.add_systems(Update, enter_overworld_from_colony);

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Enter);
        app.update();

        let next_state = app.world().resource::<NextState<AppState>>();
        assert!(
            matches!(*next_state, NextState::Pending(AppState::Overworld)),
            "Expected Enter on gate to queue transition to Overworld"
        );
    }

    #[test]
    fn off_gate_guidance_is_throttled_within_same_turn() {
        let mut app = App::new();
        app.insert_resource(GameLog::default());
        app.insert_resource(GameTime { turn: TEST_TURN });
        app.insert_resource(MapTiles::new(off_gate_map().tiles));
        app.insert_resource(ButtonInput::<KeyCode>::default());
        app.insert_resource(State::new(AppState::Colony));
        app.insert_resource(NextState::<AppState>::Unchanged);
        app.world_mut().spawn(baseline_player_bundle());
        app.add_systems(Update, enter_overworld_from_colony);

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Enter);
        app.update();

        {
            let mut keyboard = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keyboard.release(KeyCode::Enter);
            keyboard.press(KeyCode::Enter);
        }
        app.update();

        let log = app.world().resource::<GameLog>();
        let guidance_count = log
            .entries()
            .iter()
            .filter(|entry| entry.text.to_lowercase().contains("gate"))
            .count();

        assert_eq!(
            guidance_count, 1,
            "Expected repeated off-gate Enter in the same turn to emit one guidance message"
        );
    }

    #[test]
    fn help_does_not_overlap_raid_modal() {
        let mut world = World::new();
        let mut keyboard = ButtonInput::<KeyCode>::default();
        keyboard.press(KeyCode::F1);

        world.insert_resource(keyboard);
        world.insert_resource(HelpOpen(false));
        world.insert_resource(ActiveRaid {
            raider_count: TEST_RAIDER_COUNT,
            raider_strength: TEST_RAIDER_STRENGTH,
            casualties: 0,
            resources_stolen: 0,
            phase: RaidPhase::Planning,
        });

        let _ = world.run_system_once(toggle_help);

        let help_open = world.resource::<HelpOpen>();
        assert!(
            !help_open.0,
            "Expected help panel to stay closed while a blocking raid modal is active"
        );
    }

    #[test]
    fn critical_modal_has_priority_over_help() {
        let mut world = World::new();
        world.insert_resource(HelpOpen(true));
        world.insert_resource(ModalPriorityCoordinator);
        world.insert_resource(ActiveRaid {
            raider_count: TEST_RAIDER_COUNT,
            raider_strength: TEST_RAIDER_STRENGTH,
            casualties: 0,
            resources_stolen: 0,
            phase: RaidPhase::Planning,
        });

        let _ = world.run_system_once(apply_modal_priority_policy);

        let help_open = world.resource::<HelpOpen>();
        assert!(
            !help_open.0,
            "Expected critical raid modal to force-close help via priority coordinator"
        );
    }

    #[test]
    fn escape_closes_topmost_blocking_layer_first() {
        let mut world = World::new();
        let mut keyboard = ButtonInput::<KeyCode>::default();
        keyboard.press(KeyCode::Escape);

        world.insert_resource(keyboard);
        world.insert_resource(HelpOpen(true));
        world.insert_resource(State::new(AppState::Colony));
        world.insert_resource(NextState::<AppState>::Unchanged);

        let _ = world.run_system_once(handle_escape_to_menu);

        let help_open = world.resource::<HelpOpen>();
        assert!(
            !help_open.0,
            "Expected Esc to close the topmost blocking layer before any state transition"
        );
    }

    #[test]
    fn escape_does_not_bypass_critical_modal_after_help_closes() {
        let mut world = World::new();
        let mut keyboard = ButtonInput::<KeyCode>::default();
        keyboard.press(KeyCode::Escape);

        world.insert_resource(keyboard);
        world.insert_resource(HelpOpen(true));
        world.insert_resource(ModalBlockers {
            critical_modal_active: true,
        });
        world.insert_resource(State::new(AppState::Colony));
        world.insert_resource(NextState::<AppState>::Unchanged);

        let _ = world.run_system_once(handle_escape_to_menu);

        let help_open = world.resource::<HelpOpen>();
        assert!(
            !help_open.0,
            "Expected first Esc to close help before any other action"
        );

        let next_state = world.resource::<NextState<AppState>>();
        assert!(
            matches!(*next_state, NextState::Unchanged),
            "Expected critical modal to block Esc-driven AppState transition"
        );
    }

    #[test]
    fn escape_in_overworld_returns_to_colony() {
        let mut world = World::new();
        let mut keyboard = ButtonInput::<KeyCode>::default();
        keyboard.press(KeyCode::Escape);

        world.insert_resource(keyboard);
        world.insert_resource(HelpOpen(false));
        world.insert_resource(State::new(AppState::Overworld));
        world.insert_resource(NextState::<AppState>::Unchanged);

        let _ = world.run_system_once(handle_escape_to_menu);

        let next_state = world.resource::<NextState<AppState>>();
        assert!(
            matches!(*next_state, NextState::Pending(AppState::Colony)),
            "Expected Esc in Overworld to return to Colony"
        );
    }

    #[test]
    fn tab_toggles_inventory_open() {
        let mut world = World::new();
        let mut keyboard = ButtonInput::<KeyCode>::default();
        keyboard.press(KeyCode::Tab);

        world.insert_resource(keyboard);
        world.insert_resource(InventoryOpen(false));

        let _ = world.run_system_once(toggle_inventory);

        let inventory_open = world.resource::<InventoryOpen>();
        assert!(
            inventory_open.0,
            "Expected Tab key press to toggle inventory open"
        );
    }

    #[test]
    fn save_and_quit_emits_confirmation_feedback() {
        let mut app = App::new();
        app.insert_resource(GameLog::default());
        app.insert_resource(GameTime { turn: TEST_TURN });
        app.insert_resource(SaveAndQuitRequested(true));
        app.insert_resource(State::new(AppState::Colony));
        app.insert_resource(NextState::<AppState>::Unchanged);
        app.world_mut().spawn(baseline_player_bundle());
        app.add_systems(Update, save::handle_save_and_quit);

        app.update();

        let next_state = app.world().resource::<NextState<AppState>>();
        assert!(
            matches!(*next_state, NextState::Pending(AppState::Menu)),
            "Expected Save & Quit to queue transition to Menu"
        );

        let log = app.world().resource::<GameLog>();
        let has_confirmation = log.entries().iter().any(|entry| {
            let text = entry.text.to_lowercase();
            text.contains("saved") || text.contains("save complete")
        });
        assert!(
            has_confirmation,
            "Expected Save & Quit to emit explicit save confirmation feedback"
        );
    }

    #[test]
    fn new_game_emits_exact_seed_feedback() {
        const TEST_SEED_VALUE: u64 = 123_456;

        let mut app = App::new();
        app.insert_resource(GameLog::default());
        app.insert_resource(GameTime { turn: TEST_TURN });
        app.insert_resource(MenuUiAction(Some(MenuUiChoice::NewGame {
            seed: TEST_SEED_VALUE,
        })));
        app.insert_resource(NextState::<AppState>::Unchanged);
        app.add_message::<AppExit>();
        app.add_systems(Update, process_menu_action);

        app.update();

        let next_state = app.world().resource::<NextState<AppState>>();
        assert!(
            matches!(*next_state, NextState::Pending(AppState::Colony)),
            "Expected New Game to queue transition to Colony"
        );

        let world_seed = app.world().resource::<WorldSeed>();
        assert_eq!(world_seed.0, TEST_SEED_VALUE);

        let log = app.world().resource::<GameLog>();
        let has_seed_feedback = log.entries().iter().any(|entry| {
            let text = entry.text.to_lowercase();
            text.contains("seed") && text.contains("123456")
        });
        assert!(
            has_seed_feedback,
            "Expected New Game flow to emit the exact seed value used for this run"
        );
    }

    #[test]
    fn menu_load_affordance_shows_disabled_reason_when_no_save_exists() {
        let affordance = load_affordance_for_save_state(false);

        assert!(
            !affordance.is_enabled,
            "Expected load affordance to be disabled when no save exists"
        );

        let helper = affordance.helper_text.to_lowercase();
        assert!(
            helper.contains("no save") && helper.contains("new game"),
            "Expected no-save helper text to explain why load is disabled and suggest next action"
        );
    }

    #[test]
    fn menu_load_affordance_enables_load_when_save_exists() {
        let affordance = load_affordance_for_save_state(true);

        assert!(
            affordance.is_enabled,
            "Expected load affordance to be enabled when a save exists"
        );

        assert!(
            affordance.helper_text.is_empty(),
            "Expected no helper text when load is available"
        );
    }

    #[test]
    fn seed_helper_text_is_player_facing_and_non_technical() {
        let empty_text = seed_helper_text(None);
        let empty_lower = empty_text.to_lowercase();
        assert!(
            empty_lower.contains("leave blank") || empty_lower.contains("random"),
            "Expected empty-seed helper text to use player-facing guidance"
        );
        assert!(
            !empty_lower.contains("resolved") && !empty_lower.contains("auto-generated"),
            "Expected empty-seed helper text to avoid technical wording"
        );

        let seeded_text = seed_helper_text(Some(TEST_HELPER_SEED));
        let seeded_lower = seeded_text.to_lowercase();
        assert!(
            seeded_lower.contains("world") && seeded_text.contains("77"),
            "Expected seeded helper text to communicate the world seed in player-facing terms"
        );
    }

    #[test]
    fn menu_cancel_quit_keeps_state_and_emits_no_exit() {
        let mut app = App::new();
        app.insert_resource(GameLog::default());
        app.insert_resource(GameTime { turn: TEST_TURN });
        app.insert_resource(MenuUiAction(Some(MenuUiChoice::CancelQuit)));
        app.insert_resource(NextState::<AppState>::Unchanged);
        app.add_message::<AppExit>();
        app.add_systems(Update, process_menu_action);

        app.update();

        let next_state = app.world().resource::<NextState<AppState>>();
        assert!(
            matches!(*next_state, NextState::Unchanged),
            "Expected cancel quit action to preserve current menu state"
        );

        let app_exit_messages = app.world().resource::<Messages<AppExit>>();
        assert!(
            app_exit_messages.is_empty(),
            "Expected cancel quit action to emit no app-exit messages"
        );
    }

    #[test]
    fn menu_confirm_quit_emits_exit_message() {
        let mut app = App::new();
        app.insert_resource(GameLog::default());
        app.insert_resource(GameTime { turn: TEST_TURN });
        app.insert_resource(MenuUiAction(Some(MenuUiChoice::ConfirmQuit)));
        app.insert_resource(NextState::<AppState>::Unchanged);
        app.add_message::<AppExit>();
        app.add_systems(Update, process_menu_action);

        app.update();

        let app_exit_messages = app.world().resource::<Messages<AppExit>>();
        assert!(
            !app_exit_messages.is_empty(),
            "Expected confirm quit action to emit an app-exit message"
        );
    }

    #[test]
    fn colony_objective_indicator_visible_when_prompt_active() {
        let inactive = ColonyObjectivePromptState {
            has_reached_overworld: false,
            visible_in_colony: false,
        };
        assert!(
            colony_objective_indicator_text(Some(&inactive)).is_none(),
            "Expected no objective indicator text when prompt is inactive"
        );

        let active = ColonyObjectivePromptState {
            has_reached_overworld: false,
            visible_in_colony: true,
        };
        assert!(
            colony_objective_indicator_text(Some(&active)).is_some(),
            "Expected objective indicator text when prompt is active"
        );
    }

    #[test]
    fn colony_top_bar_collapses_secondary_objective_detail_by_default() {
        let active = ColonyObjectivePromptState {
            has_reached_overworld: false,
            visible_in_colony: true,
        };

        let presentation = colony_top_bar_objective_presentation(Some(&active))
            .expect("Expected objective top-bar presentation when prompt is active");

        assert!(
            presentation.inline_detail.is_none(),
            "Expected top bar to keep secondary objective detail collapsed by default"
        );
        assert!(
            presentation.hover_detail.is_some(),
            "Expected collapsed objective detail to remain discoverable via hover detail"
        );
    }

    #[test]
    fn colony_top_bar_keeps_primary_objective_guidance_visible_when_collapsed() {
        let active = ColonyObjectivePromptState {
            has_reached_overworld: false,
            visible_in_colony: true,
        };

        let presentation = colony_top_bar_objective_presentation(Some(&active))
            .expect("Expected objective top-bar presentation when prompt is active");

        assert!(
            presentation
                .primary_label
                .to_lowercase()
                .contains("primary action"),
            "Expected top bar to keep primary objective guidance visible"
        );
    }

    #[test]
    fn load_emits_concise_recap() {
        let mut world = World::new();
        world.insert_resource(GameLog::default());

        let mut save = SaveGame::default();
        save.seed = TEST_SAVE_SEED;
        save.game_time = SaveGameTime { turn: TEST_TURN };

        let _ = world.run_system_once(move |mut commands: Commands| {
            save::restore_persistent_run_resources(&mut commands, &save);
        });

        let log = world.resource::<GameLog>();
        let has_recap = log.entries().iter().any(|entry| {
            entry.text.to_lowercase().contains("load")
                || entry.text.to_lowercase().contains("recap")
        });

        assert!(
            has_recap,
            "Expected load flow to emit a concise post-load recap"
        );
    }

    #[test]
    fn load_recap_includes_immediate_objective_when_pressure_is_high() {
        let mut world = World::new();
        world.insert_resource(GameLog::default());

        let mut save = SaveGame::default();
        save.seed = TEST_SAVE_SEED;
        save.game_time = SaveGameTime { turn: TEST_TURN };
        save.colony.resources.food = 0;
        save.colony.resources.water = 0;

        let _ = world.run_system_once(move |mut commands: Commands| {
            save::restore_persistent_run_resources(&mut commands, &save);
        });

        let log = world.resource::<GameLog>();
        let has_pressure_objective = log.entries().iter().any(|entry| {
            entry.text.to_lowercase().contains("objective")
                && entry.text.to_lowercase().contains("food")
                && entry.text.to_lowercase().contains("water")
        });

        assert!(
            has_pressure_objective,
            "Expected high-pressure load recap to include immediate food/water objective"
        );
    }

    #[test]
    fn load_recap_overworld_state_mentions_travel_objective() {
        let mut world = World::new();
        world.insert_resource(GameLog::default());

        let mut save = SaveGame::default();
        save.seed = TEST_SAVE_SEED;
        save.app_state = SaveAppState::Overworld;
        save.game_time = SaveGameTime { turn: TEST_TURN };
        save.colony.resources.food = 10;
        save.colony.resources.water = 10;

        let _ = world.run_system_once(move |mut commands: Commands| {
            save::restore_persistent_run_resources(&mut commands, &save);
        });

        let log = world.resource::<GameLog>();
        let mentions_travel_objective = log.entries().iter().any(|entry| {
            let text = entry.text.to_lowercase();
            text.contains("objective") && text.contains("travel")
        });

        assert!(
            mentions_travel_objective,
            "Expected overworld load recap to include travel objective guidance"
        );
    }

    #[test]
    fn load_recap_colony_state_mentions_gate_objective() {
        let mut world = World::new();
        world.insert_resource(GameLog::default());

        let mut save = SaveGame::default();
        save.seed = TEST_SAVE_SEED;
        save.app_state = SaveAppState::Colony;
        save.game_time = SaveGameTime { turn: TEST_TURN };
        save.colony.resources.food = 10;
        save.colony.resources.water = 10;

        let _ = world.run_system_once(move |mut commands: Commands| {
            save::restore_persistent_run_resources(&mut commands, &save);
        });

        let log = world.resource::<GameLog>();
        let mentions_gate_objective = log.entries().iter().any(|entry| {
            let text = entry.text.to_lowercase();
            text.contains("objective") && text.contains("gate")
        });

        assert!(
            mentions_gate_objective,
            "Expected colony load recap to include shelter-gate objective guidance"
        );
    }

    #[test]
    fn load_recap_dungeon_state_mentions_progression_objective() {
        let mut world = World::new();
        world.insert_resource(GameLog::default());

        let mut save = SaveGame::default();
        save.seed = TEST_SAVE_SEED;
        save.app_state = SaveAppState::Dungeon;
        save.game_time = SaveGameTime { turn: TEST_TURN };
        save.colony.resources.food = 10;
        save.colony.resources.water = 10;

        let _ = world.run_system_once(move |mut commands: Commands| {
            save::restore_persistent_run_resources(&mut commands, &save);
        });

        let log = world.resource::<GameLog>();
        let mentions_progression_objective = log.entries().iter().any(|entry| {
            let text = entry.text.to_lowercase();
            text.contains("objective") && text.contains("continue progression")
        });

        assert!(
            mentions_progression_objective,
            "Expected dungeon load recap to include continuation objective guidance"
        );
    }

    #[test]
    fn stale_entity_brp_request_is_graceful() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let _ = world.despawn(entity);

        let result = validate_brp_entity_access(&world, entity, TEST_BRP_OPERATION);
        let diagnostics =
            result.expect_err("Expected stale BRP entity request to return structured diagnostics");

        assert!(
            diagnostics.code.contains("stale_entity"),
            "Expected stale-entity diagnostics to expose a stable stale-entity code"
        );
        assert_eq!(diagnostics.operation, TEST_BRP_OPERATION);
        assert!(
            diagnostics.to_log_message().contains(TEST_BRP_OPERATION),
            "Expected diagnostic log output to include operation context"
        );
    }

    #[test]
    fn first_time_flow_readability_baseline_not_met() {
        let menu_snapshot = menu_readability_snapshot();
        let colony_snapshot = colony_readability_snapshot();

        assert!(
            menu_snapshot.seed_row_contrast_ratio >= menu_snapshot.minimum_contrast_ratio,
            "Expected menu seed row to meet readability contrast baseline"
        );
        assert!(
            menu_snapshot.subtitle_contrast_ratio >= menu_snapshot.minimum_contrast_ratio,
            "Expected menu subtitle to meet readability contrast baseline"
        );
        assert!(
            colony_snapshot.urgency_banner_contrast_ratio
                >= colony_snapshot.minimum_banner_contrast_ratio,
            "Expected colony urgency banner to meet readability contrast baseline"
        );
    }

    #[test]
    fn objective_prompt_visible_on_fresh_colony_entry() {
        let mut world = World::new();
        world.insert_resource(State::new(AppState::Colony));
        world.insert_resource(ColonyObjectivePromptState::default());

        let _ = world.run_system_once(refresh_colony_objective_prompt);

        let prompt = world.resource::<ColonyObjectivePromptState>();
        assert!(
            prompt.visible_in_colony,
            "Expected objective prompt to be visible on fresh colony entry"
        );
    }

    #[test]
    fn objective_prompt_clears_after_reaching_overworld() {
        let mut world = World::new();
        world.insert_resource(State::new(AppState::Overworld));
        world.insert_resource(ColonyObjectivePromptState::default());

        let _ = world.run_system_once(refresh_colony_objective_prompt);

        {
            let prompt = world.resource::<ColonyObjectivePromptState>();
            assert!(
                prompt.has_reached_overworld,
                "Expected objective prompt state to mark overworld milestone"
            );
            assert!(
                !prompt.visible_in_colony,
                "Expected objective prompt to hide while not in colony"
            );
        }

        world.insert_resource(State::new(AppState::Colony));
        let _ = world.run_system_once(refresh_colony_objective_prompt);

        let prompt = world.resource::<ColonyObjectivePromptState>();
        assert!(
            !prompt.visible_in_colony,
            "Expected objective prompt to stay cleared after overworld milestone"
        );
    }

    #[test]
    fn gate_affordance_present_on_colony_entry() {
        let mut world = World::new();
        world.insert_resource(WorldSeed(TEST_SAVE_SEED));
        world.insert_resource(Assets::<Image>::default());

        let _ = world.run_system_once(init_placeholder_tile_atlas);
        let _ = world.run_system_once(setup_shelter);

        let gate_marker_count = world
            .query_filtered::<Entity, With<ShelterGateMarker>>()
            .iter(&world)
            .count();
        assert_eq!(
            gate_marker_count, 1,
            "Expected one gate affordance marker on fresh colony entry"
        );

        let affordance_config = world.resource::<GateAffordanceConfig>();
        assert!(
            affordance_config.enabled,
            "Expected gate affordance to be enabled by default"
        );
    }

    #[test]
    fn gate_affordance_persists_across_colony_reentry() {
        let mut world = World::new();
        world.insert_resource(WorldSeed(TEST_SAVE_SEED));
        world.insert_resource(Assets::<Image>::default());

        let _ = world.run_system_once(init_placeholder_tile_atlas);
        let _ = world.run_system_once(setup_shelter);

        let first_marker_count = world
            .query_filtered::<Entity, With<ShelterGateMarker>>()
            .iter(&world)
            .count();
        assert_eq!(
            first_marker_count, 1,
            "Expected gate affordance marker on first colony entry"
        );

        let _ = world.run_system_once(cleanup_shelter);
        let _ = world.run_system_once(setup_shelter);

        let second_marker_count = world
            .query_filtered::<Entity, With<ShelterGateMarker>>()
            .iter(&world)
            .count();
        assert_eq!(
            second_marker_count, 1,
            "Expected gate affordance marker after colony re-entry"
        );
    }

    #[test]
    fn objective_priority_suppresses_secondary_help_hints_when_active() {
        let policy = InstructionPriorityPolicy::default();
        let objective_prompt = ColonyObjectivePromptState {
            has_reached_overworld: false,
            visible_in_colony: true,
        };

        let show_secondary = colony_help_shows_secondary_hints(Some(&objective_prompt), &policy);
        assert!(
            !show_secondary,
            "Expected secondary colony help hints to be suppressed while primary objective is active"
        );
    }

    #[test]
    fn objective_priority_allows_secondary_help_hints_after_progress() {
        let policy = InstructionPriorityPolicy::default();
        let objective_prompt = ColonyObjectivePromptState {
            has_reached_overworld: true,
            visible_in_colony: false,
        };

        let show_secondary = colony_help_shows_secondary_hints(Some(&objective_prompt), &policy);
        assert!(
            show_secondary,
            "Expected secondary colony help hints after primary objective milestone is complete"
        );
    }

    #[test]
    fn esc_overworld_reinforcement_emits_once_per_run() {
        let mut app = App::new();
        app.insert_resource(GameLog::default());
        app.insert_resource(GameTime { turn: TEST_TURN });
        app.insert_resource(HelpOpen(false));
        app.insert_resource(State::new(AppState::Overworld));
        app.insert_resource(NextState::<AppState>::Unchanged);
        app.insert_resource(ButtonInput::<KeyCode>::default());
        app.add_systems(Update, handle_escape_to_menu);

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.update();

        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.release(KeyCode::Escape);
            keys.press(KeyCode::Escape);
        }
        app.world_mut().resource_mut::<GameTime>().turn += 1;
        app.update();

        let log = app.world().resource::<GameLog>();
        let reinforce_count = log
            .entries()
            .iter()
            .filter(|entry| entry.text.contains("Esc returns you to shelter"))
            .count();
        assert_eq!(
            reinforce_count, 1,
            "Expected Esc overworld reinforcement message to emit once per run"
        );
    }

    #[test]
    fn esc_help_close_reinforcement_emits_once_per_run() {
        let mut app = App::new();
        app.insert_resource(GameLog::default());
        app.insert_resource(GameTime { turn: TEST_TURN });
        app.insert_resource(HelpOpen(true));
        app.insert_resource(State::new(AppState::Colony));
        app.insert_resource(NextState::<AppState>::Unchanged);
        app.insert_resource(ButtonInput::<KeyCode>::default());
        app.add_systems(Update, handle_escape_to_menu);

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.update();

        app.world_mut().resource_mut::<HelpOpen>().0 = true;
        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.release(KeyCode::Escape);
            keys.press(KeyCode::Escape);
        }
        app.world_mut().resource_mut::<GameTime>().turn += 1;
        app.update();

        let log = app.world().resource::<GameLog>();
        let reinforce_count = log
            .entries()
            .iter()
            .filter(|entry| entry.text.contains("Esc closed Help"))
            .count();
        assert_eq!(
            reinforce_count, 1,
            "Expected Esc help-close reinforcement message to emit once per run"
        );
    }

    #[test]
    fn primary_cta_labels_are_explicit_and_state_specific() {
        assert_eq!(primary_menu_cta_label(), "New Game");

        let colony_prompt = ColonyObjectivePromptState {
            has_reached_overworld: false,
            visible_in_colony: true,
        };
        assert!(
            primary_colony_cta_label(Some(&colony_prompt)).contains("shelter gate"),
            "Expected colony primary CTA to emphasize shelter gate while onboarding objective is active"
        );

        assert!(
            primary_overworld_cta_label().contains("connected node"),
            "Expected overworld primary CTA to emphasize connected node travel"
        );
    }

    #[test]
    fn ft_01_menu_new_game_colony_onboarding_overworld_transition() {
        let mut app = App::new();
        app.insert_resource(GameLog::default());
        app.insert_resource(GameTime { turn: TEST_TURN });
        app.insert_resource(MenuUiAction(Some(MenuUiChoice::NewGame {
            seed: FT_01_SEED,
        })));
        app.insert_resource(NextState::<AppState>::Unchanged);
        app.add_message::<AppExit>();
        app.add_systems(Update, process_menu_action);

        app.update();

        let next_state = app.world().resource::<NextState<AppState>>();
        assert!(
            matches!(*next_state, NextState::Pending(AppState::Colony)),
            "Expected New Game flow to queue transition to Colony"
        );

        let world_seed = app.world().resource::<WorldSeed>();
        assert_eq!(world_seed.0, FT_01_SEED);

        app.insert_resource(State::new(AppState::Colony));
        app.insert_resource(NextState::<AppState>::Unchanged);
        app.insert_resource(MapTiles::new(vec![
            vec![TileKind::StairsUp; TEST_MAP_SIZE];
            TEST_MAP_SIZE
        ]));
        app.insert_resource(ButtonInput::<KeyCode>::default());
        app.insert_resource(ColonyObjectivePromptState::default());
        app.world_mut().spawn(baseline_player_bundle());
        app.add_systems(
            Update,
            (refresh_colony_objective_prompt, enter_overworld_from_colony),
        );

        app.update();

        let prompt = app.world().resource::<ColonyObjectivePromptState>();
        assert!(
            prompt.visible_in_colony,
            "Expected colony onboarding objective prompt to be visible before first transition"
        );

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Enter);
        app.update();

        let next_state = app.world().resource::<NextState<AppState>>();
        assert!(
            matches!(*next_state, NextState::Pending(AppState::Overworld)),
            "Expected Enter on gate to queue Overworld transition in FT-01 flow"
        );
    }

    #[test]
    fn ft_02_load_recall_and_first_meaningful_action() {
        let mut world = World::new();
        world.insert_resource(GameLog::default());

        let mut save = SaveGame::default();
        save.seed = TEST_SAVE_SEED;
        save.app_state = SaveAppState::Colony;
        save.game_time = SaveGameTime { turn: TEST_TURN };
        save.colony.resources.food = 10;
        save.colony.resources.water = 10;

        let _ = world.run_system_once(move |mut commands: Commands| {
            save::restore_persistent_run_resources(&mut commands, &save);
        });

        let log = world.resource::<GameLog>();
        assert!(
            log_contains_all_terms(log, &[KEYWORD_OBJECTIVE, KEYWORD_GATE]),
            "Expected load recap to include immediate colony objective recall"
        );

        world.insert_resource(MapTiles::new(vec![
            vec![TileKind::StairsUp; TEST_MAP_SIZE];
            TEST_MAP_SIZE
        ]));
        world.insert_resource(ButtonInput::<KeyCode>::default());
        world.insert_resource(State::new(AppState::Colony));
        world.insert_resource(NextState::<AppState>::Unchanged);
        world.insert_resource(GameTime { turn: TEST_TURN });
        world.spawn(baseline_player_bundle());

        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Enter);
        let _ = world.run_system_once(enter_overworld_from_colony);

        let next_state = world.resource::<NextState<AppState>>();
        assert!(
            matches!(*next_state, NextState::Pending(AppState::Overworld)),
            "Expected first meaningful post-load colony action to reach Overworld transition"
        );
    }

    #[test]
    fn ft_03_new_game_help_esc_and_progression_continuity() {
        let mut app = App::new();
        app.insert_resource(GameLog::default());
        app.insert_resource(GameTime { turn: TEST_TURN });
        app.insert_resource(MenuUiAction(Some(MenuUiChoice::NewGame {
            seed: FT_03_SEED,
        })));
        app.insert_resource(NextState::<AppState>::Unchanged);
        app.add_message::<AppExit>();
        app.add_systems(Update, process_menu_action);

        app.update();

        let next_state = app.world().resource::<NextState<AppState>>();
        assert!(
            matches!(*next_state, NextState::Pending(AppState::Colony)),
            "Expected New Game flow to enter Colony in FT-03"
        );

        app.insert_resource(State::new(AppState::Colony));
        app.insert_resource(NextState::<AppState>::Unchanged);
        app.insert_resource(HelpOpen(false));
        app.insert_resource(ButtonInput::<KeyCode>::default());
        app.insert_resource(MapTiles::new(vec![
            vec![TileKind::StairsUp; TEST_MAP_SIZE];
            TEST_MAP_SIZE
        ]));
        app.world_mut().spawn(baseline_player_bundle());
        app.add_systems(
            Update,
            (
                toggle_help,
                handle_escape_to_menu,
                enter_overworld_from_colony,
            ),
        );

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::F1);
        app.update();
        {
            let help_open = app.world().resource::<HelpOpen>();
            assert!(help_open.0, "Expected help panel to open in FT-03");
        }

        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.release(KeyCode::F1);
            keys.press(KeyCode::Escape);
        }
        app.update();
        {
            let help_open = app.world().resource::<HelpOpen>();
            assert!(!help_open.0, "Expected Esc to close help panel in FT-03");
        }

        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.release(KeyCode::Escape);
            keys.press(KeyCode::Enter);
        }
        app.update();

        let next_state = app.world().resource::<NextState<AppState>>();
        assert!(
            matches!(*next_state, NextState::Pending(AppState::Overworld)),
            "Expected FT-03 flow to retain progression continuity after help/Esc interactions"
        );
    }

    #[test]
    fn rs_01_colony_save_load_recap_and_immediate_objective() {
        let mut world = World::new();
        world.insert_resource(GameLog::default());

        let mut save = SaveGame::default();
        save.seed = TEST_SAVE_SEED;
        save.app_state = SaveAppState::Colony;
        save.game_time = SaveGameTime { turn: TEST_TURN };
        save.colony.resources.food = 10;
        save.colony.resources.water = 10;

        let _ = world.run_system_once(move |mut commands: Commands| {
            save::restore_persistent_run_resources(&mut commands, &save);
        });

        let log = world.resource::<GameLog>();
        assert!(
            log_contains_all_terms(log, &[KEYWORD_OBJECTIVE, KEYWORD_GATE]),
            "Expected colony resume recap to identify immediate gate objective"
        );
    }

    #[test]
    fn rs_02_overworld_save_load_recap_and_travel_action() {
        let mut world = World::new();
        world.insert_resource(GameLog::default());

        let mut save = SaveGame::default();
        save.seed = TEST_SAVE_SEED;
        save.app_state = SaveAppState::Overworld;
        save.game_time = SaveGameTime { turn: TEST_TURN };
        save.colony.resources.food = 10;
        save.colony.resources.water = 10;

        let _ = world.run_system_once(move |mut commands: Commands| {
            save::restore_persistent_run_resources(&mut commands, &save);
        });

        let log = world.resource::<GameLog>();
        assert!(
            log_contains_all_terms(log, &[KEYWORD_OBJECTIVE, KEYWORD_TRAVEL]),
            "Expected overworld resume recap to identify travel objective"
        );

        assert!(
            primary_overworld_cta_label().contains("connected node"),
            "Expected overworld CTA guidance to provide a clear first travel action"
        );
    }

    #[test]
    fn st_01_rapid_modal_toggles_preserve_priority_and_state_transitions() {
        let mut app = App::new();
        app.insert_resource(GameLog::default());
        app.insert_resource(GameTime { turn: TEST_TURN });
        app.insert_resource(HelpOpen(false));
        app.insert_resource(InventoryOpen(false));
        app.insert_resource(ModalPriorityCoordinator);
        app.insert_resource(ModalBlockers::default());
        app.insert_resource(ActiveRaid {
            raider_count: TEST_RAIDER_COUNT,
            raider_strength: TEST_RAIDER_STRENGTH,
            casualties: 0,
            resources_stolen: 0,
            phase: RaidPhase::Planning,
        });
        app.insert_resource(State::new(AppState::Colony));
        app.insert_resource(NextState::<AppState>::Unchanged);
        app.insert_resource(ButtonInput::<KeyCode>::default());
        app.add_systems(
            Update,
            (
                toggle_help,
                toggle_inventory,
                apply_modal_priority_policy,
                handle_escape_to_menu,
            )
                .chain(),
        );

        for _ in 0..ST_01_RAPID_TOGGLE_CYCLES {
            {
                let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
                keys.release(KeyCode::F1);
                keys.release(KeyCode::Tab);
                keys.release(KeyCode::Escape);
                keys.press(KeyCode::F1);
                keys.press(KeyCode::Tab);
                keys.press(KeyCode::Escape);
            }
            app.update();
        }

        {
            let help_open = app.world().resource::<HelpOpen>();
            assert!(
                !help_open.0,
                "Expected critical modal priority to keep help closed during rapid toggles"
            );
        }
        {
            let blockers = app.world().resource::<ModalBlockers>();
            assert!(
                blockers.critical_modal_active,
                "Expected critical modal blocker to remain active while raid modal is present"
            );
        }
        {
            let next_state = app.world().resource::<NextState<AppState>>();
            assert!(
                matches!(*next_state, NextState::Unchanged),
                "Expected Esc not to transition app state while critical modal is active"
            );
        }

        app.world_mut().remove_resource::<ActiveRaid>();
        app.world_mut().resource_mut::<GameTime>().turn += 1;
        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.release(KeyCode::F1);
            keys.release(KeyCode::Tab);
            keys.release(KeyCode::Escape);
            keys.press(KeyCode::Escape);
        }
        let _ = app.world_mut().run_system_once(apply_modal_priority_policy);
        let _ = app.world_mut().run_system_once(handle_escape_to_menu);

        {
            let blockers = app.world().resource::<ModalBlockers>();
            assert!(
                !blockers.critical_modal_active,
                "Expected blocker flag to clear once critical modal resource is removed"
            );
        }
        let next_state = app.world().resource::<NextState<AppState>>();
        assert!(
            matches!(*next_state, NextState::Pending(AppState::Menu)),
            "Expected Esc to transition from Colony to Menu once critical modal is cleared"
        );
    }
}
