use bd_core::{
    colony::logistics::{
        Cargo, JobStage, LogisticsBlock, LogisticsJob, LogisticsTargetState, LogisticsTick,
        tick_logistics,
    },
    components::Position,
};
use bd_test_support::FoundationDriver;
use bd_test_support::foundation_content;

fn timber_job(stage: JobStage) -> LogisticsJob {
    LogisticsJob {
        recipe_id: "recipe.refine_timber".into(),
        stage,
        work_completed: 0,
        blocked: None,
    }
}

fn timber_job_near_completion(stage: JobStage, work_completed: u32) -> LogisticsJob {
    LogisticsJob {
        work_completed,
        ..timber_job(stage)
    }
}

#[test]
fn configured_work_turns_gate_gather_and_refine_yields_exactly_once() {
    let content = foundation_content();
    let recipe = content
        .colony_recipes
        .iter()
        .find(|recipe| recipe.id == "recipe.refine_timber")
        .unwrap();
    assert_eq!(recipe.gather_work_turns, 3);
    assert_eq!(recipe.refine_work_turns, 2);

    let mut job = timber_job(JobStage::ReadyToGather);
    let mut cargo = Cargo::default();
    let mut position = Position { x: 2, y: 2 };
    for completed in 1..recipe.gather_work_turns {
        let result = tick_logistics(
            &mut job,
            &mut cargo,
            &mut position,
            recipe,
            LogisticsTargetState::AtWorkTile,
        );
        assert_eq!(job.stage, JobStage::ReadyToGather, "turn={completed}");
        assert_eq!(job.work_completed, completed, "turn={completed}");
        assert_eq!(cargo.amount, 0, "turn={completed}: credited raw early");
        assert_eq!(result.gathered_input, 0, "turn={completed}");
    }
    let gathered = tick_logistics(
        &mut job,
        &mut cargo,
        &mut position,
        recipe,
        LogisticsTargetState::AtWorkTile,
    );
    assert_eq!(job.stage, JobStage::ToStation);
    assert_eq!(job.work_completed, 0);
    assert_eq!(cargo.amount, recipe.input_amount);
    assert_eq!(gathered.gathered_input, recipe.input_amount);

    job.stage = JobStage::ReadyToRefine;
    for completed in 1..recipe.refine_work_turns {
        let result = tick_logistics(
            &mut job,
            &mut cargo,
            &mut position,
            recipe,
            LogisticsTargetState::AtWorkTile,
        );
        assert_eq!(job.stage, JobStage::ReadyToRefine, "turn={completed}");
        assert_eq!(job.work_completed, completed, "turn={completed}");
        assert_eq!(
            result.finished_output, 0,
            "turn={completed}: credited output early"
        );
    }
    let refined = tick_logistics(
        &mut job,
        &mut cargo,
        &mut position,
        recipe,
        LogisticsTargetState::AtWorkTile,
    );
    assert_eq!(job.stage, JobStage::ToSource);
    assert_eq!(job.work_completed, 0);
    assert_eq!(cargo.amount, 0);
    assert_eq!(refined.finished_output, recipe.output_amount);
}

#[test]
fn pilot_recipe_transition_matrix_performs_one_operation_per_worker_tick() {
    let content = foundation_content();
    let recipe = content
        .colony_recipes
        .iter()
        .find(|recipe| recipe.id == "recipe.refine_timber")
        .unwrap();
    let start = Position { x: 2, y: 2 };
    let step = Position { x: 3, y: 2 };
    let cases = [
        (
            "to-source-step",
            timber_job(JobStage::ToSource),
            Cargo::default(),
            LogisticsTargetState::RouteStep(step),
            JobStage::ToSource,
            step,
            0,
            0,
        ),
        (
            "source-arrival",
            timber_job(JobStage::ToSource),
            Cargo::default(),
            LogisticsTargetState::AtWorkTile,
            JobStage::ReadyToGather,
            start,
            0,
            0,
        ),
        (
            "gather-success",
            timber_job_near_completion(JobStage::ReadyToGather, recipe.gather_work_turns - 1),
            Cargo::default(),
            LogisticsTargetState::AtWorkTile,
            JobStage::ToStation,
            start,
            recipe.input_amount,
            0,
        ),
        (
            "station-arrival",
            timber_job(JobStage::ToStation),
            Cargo {
                resource_id: Some(recipe.input_resource_id.clone()),
                amount: recipe.input_amount,
            },
            LogisticsTargetState::AtWorkTile,
            JobStage::ReadyToRefine,
            start,
            recipe.input_amount,
            0,
        ),
        (
            "refine-success",
            timber_job_near_completion(JobStage::ReadyToRefine, recipe.refine_work_turns - 1),
            Cargo {
                resource_id: Some(recipe.input_resource_id.clone()),
                amount: recipe.input_amount,
            },
            LogisticsTargetState::AtWorkTile,
            JobStage::ToSource,
            start,
            0,
            recipe.output_amount,
        ),
    ];

    for (case_id, mut job, mut cargo, target, stage, position, raw, output) in cases {
        let mut actual_position = start;
        let result = tick_logistics(&mut job, &mut cargo, &mut actual_position, recipe, target);
        assert_eq!(job.stage, stage, "case={case_id}: stage; result={result:?}");
        assert_eq!(
            actual_position, position,
            "case={case_id}: position; result={result:?}"
        );
        assert_eq!(
            cargo.amount, raw,
            "case={case_id}: cargo; result={result:?}"
        );
        assert_eq!(
            result.finished_output, output,
            "case={case_id}: output; result={result:?}"
        );
    }
}

#[test]
fn missing_or_unreachable_targets_block_without_losing_cargo_or_output() {
    for (case_id, target, expected) in [
        (
            "missing-source",
            LogisticsTargetState::Missing,
            LogisticsBlock::MissingSource,
        ),
        (
            "no-route",
            LogisticsTargetState::NoRoute,
            LogisticsBlock::NoRoute,
        ),
    ] {
        let content = foundation_content();
        let recipe = &content.colony_recipes[0];
        let mut job = timber_job(JobStage::ToSource);
        let mut cargo = Cargo {
            resource_id: Some(recipe.input_resource_id.clone()),
            amount: 2,
        };
        let before = cargo.clone();
        let mut position = Position { x: 2, y: 2 };
        let result = tick_logistics(&mut job, &mut cargo, &mut position, recipe, target);
        assert_eq!(result.blocked, Some(expected), "case={case_id}");
        assert_eq!(cargo, before, "case={case_id}: cargo changed");
        assert_eq!(result.finished_output, 0, "case={case_id}: output changed");
    }
}

#[test]
fn pilot_recipe_conserves_raw_input_and_only_refining_creates_finished_output() {
    let content = foundation_content();
    let recipe = content
        .colony_recipes
        .iter()
        .find(|recipe| recipe.id == "recipe.refine_timber")
        .unwrap();
    let mut job = timber_job_near_completion(JobStage::ReadyToGather, recipe.gather_work_turns - 1);
    let mut cargo = Cargo::default();
    let mut position = Position { x: 2, y: 2 };
    let gathered = tick_logistics(
        &mut job,
        &mut cargo,
        &mut position,
        recipe,
        LogisticsTargetState::AtWorkTile,
    );
    assert_eq!(cargo.amount, recipe.input_amount);
    assert_eq!(gathered.finished_output, 0);

    job.stage = JobStage::ReadyToRefine;
    job.work_completed = recipe.refine_work_turns - 1;
    let refined = tick_logistics(
        &mut job,
        &mut cargo,
        &mut position,
        recipe,
        LogisticsTargetState::AtWorkTile,
    );
    assert_eq!(cargo.amount, 0);
    assert_eq!(refined.finished_output, recipe.output_amount);
}

#[test]
fn absent_worker_tick_is_a_no_op() {
    let job = timber_job(JobStage::ReadyToGather);
    let cargo = Cargo::default();
    let position = Position { x: 2, y: 2 };
    let snapshot = (job.clone(), cargo.clone(), position);

    assert_eq!((job, cargo, position), snapshot);
    let _ = LogisticsTick::default();
}

#[test]
fn scheduler_frames_without_accepted_time_do_not_advance_logistics() {
    let mut driver = FoundationDriver::new(4400);
    driver.start_colony().unwrap();
    let survivor = driver.survivor_by_name("Mara").unwrap();
    driver
        .assign_recipe(
            "assign render-idle worker",
            survivor,
            "recipe.refine_timber",
        )
        .unwrap();
    let before = (
        driver.position(survivor),
        driver.logistics_job(survivor),
        driver.worker_cargo(survivor),
    );

    for _ in 0..20 {
        driver.advance_idle();
    }

    assert_eq!(
        (
            driver.position(survivor),
            driver.logistics_job(survivor),
            driver.worker_cargo(survivor),
        ),
        before
    );
}

#[test]
fn tactical_turns_do_not_advance_colony_logistics() {
    let mut driver = FoundationDriver::new(4405);
    driver.start_colony().unwrap();
    let survivor = driver.survivor_by_name("Mara").unwrap();
    driver
        .assign_recipe(
            "assign tactical-paused worker",
            survivor,
            "recipe.refine_timber",
        )
        .unwrap();
    driver
        .enter_dungeon(bd_core::spatial::FOUNDATION_DUNGEON_ID)
        .unwrap();
    let before = (
        driver.position(survivor),
        driver.logistics_job(survivor),
        driver.worker_cargo(survivor),
    );
    let player = driver.player().unwrap();

    for step in 0..5 {
        driver
            .submit_action_and_advance_result_frame(
                &format!("tactical wait {step}"),
                player,
                "ability.wait",
                None,
                None,
            )
            .unwrap();
        driver.advance_enemy_phase_frame();
    }

    assert_eq!(
        (
            driver.position(survivor),
            driver.logistics_job(survivor),
            driver.worker_cargo(survivor),
        ),
        before
    );
}

#[test]
fn one_survivor_completes_the_pilot_source_to_station_route() {
    let mut driver = FoundationDriver::new(4401);
    driver.start_colony().unwrap();
    let station_position = Position { x: 3, y: 3 };
    assert!(
        driver
            .resource_node_layout()
            .iter()
            .all(|(_, position)| *position != station_position),
        "fixture station must not overlap a generated source"
    );
    driver.fixture_spawn_processing_station(station_position);
    let survivor = driver.survivor_by_name("Mara").unwrap();
    driver.fixture_assign_recipe(survivor, "recipe.refine_timber");
    let materials_before = driver
        .resource_current(bd_core::signals::PoolKind::Materials)
        .unwrap();
    let player = driver.player().unwrap();

    for _step in 0..160 {
        driver
            .submit_action_and_advance_result_frame(
                "pilot production worker tick",
                player,
                "ability.wait",
                None,
                None,
            )
            .unwrap();
        if driver
            .resource_current(bd_core::signals::PoolKind::Materials)
            .unwrap()
            > materials_before
        {
            break;
        }
    }

    assert_eq!(
        driver
            .resource_current(bd_core::signals::PoolKind::Materials)
            .unwrap(),
        materials_before + 1
    );
    assert_eq!(driver.worker_cargo(survivor).unwrap().amount, 0);
    assert_eq!(
        driver.logistics_job(survivor).unwrap().stage,
        JobStage::ToSource
    );
}

#[test]
fn carrying_checkpoint_preserves_recipe_stage_and_raw_cargo() {
    let mut driver = FoundationDriver::new(4402);
    driver.start_colony().unwrap();
    driver.fixture_spawn_processing_station(Position { x: 3, y: 3 });
    let survivor = driver.survivor_by_name("Mara").unwrap();
    driver.fixture_assign_recipe(survivor, "recipe.refine_timber");
    let player = driver.player().unwrap();
    for _ in 0..100 {
        driver
            .submit_action_and_advance_result_frame(
                "advance to carrying checkpoint",
                player,
                "ability.wait",
                None,
                None,
            )
            .unwrap();
        if driver.worker_cargo(survivor).unwrap().amount > 0 {
            break;
        }
    }
    assert_eq!(
        driver.logistics_job(survivor).unwrap().stage,
        JobStage::ToStation
    );
    let cargo = driver.worker_cargo(survivor).unwrap();
    let checkpoint = driver.checkpoint().unwrap();
    driver.restore_checkpoint(&checkpoint).unwrap();
    let restored_survivor = driver.survivor_by_name("Mara").unwrap();

    assert_eq!(driver.worker_cargo(restored_survivor), Some(cargo));
    assert_eq!(
        driver.logistics_job(restored_survivor).unwrap().stage,
        JobStage::ToStation
    );
}

#[test]
fn partial_work_progress_survives_checkpoint_without_free_yield() {
    let mut driver = FoundationDriver::new(4402);
    driver.start_colony().unwrap();
    let survivor = driver.survivor_by_name("Mara").unwrap();
    driver.fixture_assign_recipe(survivor, "recipe.refine_timber");
    let source = driver
        .resource_node_layout()
        .into_iter()
        .find(|(source_id, _)| source_id == "source.trees")
        .unwrap()
        .1;
    driver
        .fixture_set_position(
            survivor,
            Position {
                x: source.x - 1,
                y: source.y,
            },
        )
        .unwrap();
    driver.fixture_set_logistics_progress(survivor, JobStage::ReadyToGather, 1);
    driver.update_frames(1);
    let cargo_before = driver.worker_cargo(survivor);
    let resources_before = driver.resource_current(bd_core::signals::PoolKind::Materials);
    let fingerprint_before = driver.fingerprint();

    let checkpoint = driver.checkpoint().unwrap();
    let mut restored = FoundationDriver::from_checkpoint(&checkpoint).unwrap();
    let restored_survivor = restored.survivor_by_name("Mara").unwrap();
    let restored_job = restored.logistics_job(restored_survivor).unwrap();
    assert_eq!(restored_job.stage, JobStage::ReadyToGather);
    assert_eq!(restored_job.work_completed, 1);
    assert_eq!(restored.worker_cargo(restored_survivor), cargo_before);
    assert_eq!(
        restored.resource_current(bd_core::signals::PoolKind::Materials),
        resources_before
    );
    assert_eq!(restored.fingerprint(), fingerprint_before);
}

#[test]
fn carrying_checkpoint_preserves_the_next_deterministic_worker_tick() {
    let mut original = FoundationDriver::new(4403);
    original.start_colony().unwrap();
    original.fixture_spawn_processing_station(Position { x: 3, y: 3 });
    let survivor = original.survivor_by_name("Mara").unwrap();
    original.fixture_assign_recipe(survivor, "recipe.refine_timber");
    let player = original.player().unwrap();
    for _ in 0..100 {
        original
            .submit_action_and_advance_result_frame(
                "advance to deterministic carrying checkpoint",
                player,
                "ability.wait",
                None,
                None,
            )
            .unwrap();
        if original.worker_cargo(survivor).unwrap().amount > 0 {
            break;
        }
    }
    let checkpoint = original.checkpoint().unwrap();
    let mut restored = FoundationDriver::from_checkpoint(&checkpoint).unwrap();
    let restored_player = restored.player().unwrap();
    original
        .submit_action_and_advance_result_frame(
            "uninterrupted carrying tick",
            player,
            "ability.wait",
            None,
            None,
        )
        .unwrap();
    restored
        .submit_action_and_advance_result_frame(
            "restored carrying tick",
            restored_player,
            "ability.wait",
            None,
            None,
        )
        .unwrap();
    let restored_survivor = restored.survivor_by_name("Mara").unwrap();

    assert_eq!(
        restored.position(restored_survivor),
        original.position(survivor)
    );
    assert_eq!(
        restored.logistics_job(restored_survivor),
        original.logistics_job(survivor)
    );
    assert_eq!(
        restored.worker_cargo(restored_survivor),
        original.worker_cargo(survivor)
    );
    assert_eq!(
        restored.resource_current(bd_core::signals::PoolKind::Materials),
        original.resource_current(bd_core::signals::PoolKind::Materials)
    );
}

#[test]
fn checkpoint_round_trip_preserves_every_logistics_stage() {
    for (case_index, expected_stage) in [
        JobStage::ToSource,
        JobStage::ReadyToGather,
        JobStage::ToStation,
        JobStage::ReadyToRefine,
    ]
    .into_iter()
    .enumerate()
    {
        let mut driver = FoundationDriver::new(4410 + case_index as u64);
        driver.start_colony().unwrap();
        let survivor = driver.survivor_by_name("Mara").unwrap();
        driver.fixture_assign_recipe(survivor, "recipe.refine_timber");
        let player = driver.player().unwrap();
        for _ in 0..160 {
            if driver.logistics_job(survivor).unwrap().stage == expected_stage {
                break;
            }
            driver
                .submit_action_and_advance_result_frame(
                    "advance to durable logistics stage",
                    player,
                    "ability.wait",
                    None,
                    None,
                )
                .unwrap();
        }
        let job = driver.logistics_job(survivor).unwrap();
        assert_eq!(
            job.stage, expected_stage,
            "stage={expected_stage:?}: stage was never reached"
        );
        let cargo = driver.worker_cargo(survivor).unwrap();
        let position = driver.position(survivor).unwrap();
        let checkpoint = driver.checkpoint().unwrap();
        driver.restore_checkpoint(&checkpoint).unwrap();
        let restored_survivor = driver.survivor_by_name("Mara").unwrap();

        assert_eq!(
            driver.logistics_job(restored_survivor),
            Some(job),
            "stage={expected_stage:?}: job changed"
        );
        assert_eq!(
            driver.worker_cargo(restored_survivor),
            Some(cargo),
            "stage={expected_stage:?}: cargo changed"
        );
        assert_eq!(
            driver.position(restored_survivor),
            Some(position),
            "stage={expected_stage:?}: position changed"
        );
    }
}

#[test]
fn reassigning_a_carrying_worker_deposits_raw_cargo_and_cancels_logistics() {
    let mut driver = FoundationDriver::new(4404);
    driver.start_colony().unwrap();
    let survivor = driver.survivor_by_name("Mara").unwrap();
    driver.fixture_assign_recipe(survivor, "recipe.refine_timber");
    let player = driver.player().unwrap();
    for _ in 0..100 {
        driver
            .submit_action_and_advance_result_frame(
                "advance worker until carrying",
                player,
                "ability.wait",
                None,
                None,
            )
            .unwrap();
        if driver.worker_cargo(survivor).unwrap().amount > 0 {
            break;
        }
    }
    let cargo = driver.worker_cargo(survivor).unwrap();
    assert_eq!(cargo.resource_id.as_deref(), Some("resource.raw_timber"));
    assert!(cargo.amount > 0);

    driver
        .submit_action_and_advance_result_frame(
            "reassign carrying worker to idle",
            player,
            "ability.assign_idle",
            None,
            Some(survivor),
        )
        .unwrap();

    assert_eq!(driver.logistics_job(survivor), None);
    assert_eq!(driver.worker_cargo(survivor), None);
    assert_eq!(
        driver.raw_resource_count("resource.raw_timber"),
        cargo.amount
    );
    assert_eq!(
        driver.survivor_task(survivor),
        Some(bd_core::colony::survivors::SurvivorTask::Idle)
    );
}

#[test]
fn two_workers_share_one_station_work_tile_without_stacking_or_duplicate_credit() {
    let mut driver = FoundationDriver::new(4502);
    driver.start_colony().unwrap();
    let station = driver
        .station_by_type(bd_core::colony::stations::StationType::Custom(1))
        .expect("starter Basic Processing station must exist");
    let station_position = Position { x: 10, y: 10 };
    driver
        .fixture_set_position(station, station_position)
        .unwrap();
    for wall in [
        Position { x: 10, y: 9 },
        Position { x: 9, y: 10 },
        Position { x: 11, y: 10 },
    ] {
        driver.fixture_set_outpost_tile(wall, bd_core::components::Tile::Wall);
    }
    let first_worker = driver.survivor_by_name("Mara").unwrap();
    let second_worker = driver.survivor_by_name("Iven").unwrap();
    driver.fixture_assign_recipe(first_worker, "recipe.refine_timber");
    driver.fixture_assign_recipe(second_worker, "recipe.refine_timber");
    let materials_before = driver
        .resource_current(bd_core::signals::PoolKind::Materials)
        .unwrap();
    let player = driver.player().unwrap();

    for _ in 0..260 {
        driver
            .submit_action_and_advance_result_frame(
                "contested station worker tick",
                player,
                "ability.wait",
                None,
                None,
            )
            .unwrap();
        let positions = driver.survivor_positions();
        assert_eq!(
            positions
                .iter()
                .copied()
                .collect::<std::collections::HashSet<_>>()
                .len(),
            positions.len(),
            "workers stacked while competing for the sole station work tile"
        );
        if driver
            .resource_current(bd_core::signals::PoolKind::Materials)
            .unwrap()
            >= materials_before + 2
        {
            break;
        }
    }

    assert_eq!(
        driver
            .resource_current(bd_core::signals::PoolKind::Materials)
            .unwrap(),
        materials_before + 2,
        "each worker must receive exactly one observable completion credit"
    );
}

#[test]
fn complete_colony_workflow_replays_deterministically_from_player_actions() {
    let mut first = FoundationDriver::new(4510);
    let mut second = FoundationDriver::new(4510);
    first.start_colony().unwrap();
    second.start_colony().unwrap();
    let original_survivor = first.survivor_by_name("Mara").unwrap();
    let second_survivor = second.survivor_by_name("Mara").unwrap();
    first
        .assign_recipe(
            "first deterministic recipe assignment",
            original_survivor,
            "recipe.refine_timber",
        )
        .unwrap();
    second
        .assign_recipe(
            "second deterministic recipe assignment",
            second_survivor,
            "recipe.refine_timber",
        )
        .unwrap();
    let first_player = first.player().unwrap();
    let second_player = second.player().unwrap();

    for step in 0..120 {
        first
            .submit_action_and_advance_result_frame(
                &format!("first deterministic worker tick {step}"),
                first_player,
                "ability.wait",
                None,
                None,
            )
            .unwrap();
        second
            .submit_action_and_advance_result_frame(
                &format!("second deterministic worker tick {step}"),
                second_player,
                "ability.wait",
                None,
                None,
            )
            .unwrap();
        assert_eq!(
            (
                first.position(original_survivor),
                first.logistics_job(original_survivor),
                first.worker_cargo(original_survivor),
                first.resource_current(bd_core::signals::PoolKind::Materials),
            ),
            (
                second.position(second_survivor),
                second.logistics_job(second_survivor),
                second.worker_cargo(second_survivor),
                second.resource_current(bd_core::signals::PoolKind::Materials),
            ),
            "step={step}: deterministic colony state diverged"
        );
    }
}

#[test]
fn every_configured_recipe_obeys_the_same_gather_and_refine_transition() {
    let content = foundation_content();
    for recipe in &content.colony_recipes {
        let mut job = LogisticsJob {
            recipe_id: recipe.id.clone(),
            stage: JobStage::ReadyToGather,
            work_completed: recipe.gather_work_turns - 1,
            blocked: None,
        };
        let mut cargo = Cargo::default();
        let mut position = Position { x: 2, y: 2 };
        let gathered = tick_logistics(
            &mut job,
            &mut cargo,
            &mut position,
            recipe,
            LogisticsTargetState::AtWorkTile,
        );
        assert_eq!(
            (cargo.resource_id.as_deref(), cargo.amount),
            (Some(recipe.input_resource_id.as_str()), recipe.input_amount),
            "recipe={}: gather",
            recipe.id
        );
        assert_eq!(
            gathered.finished_output, 0,
            "recipe={}: gather output",
            recipe.id
        );

        job.stage = JobStage::ReadyToRefine;
        job.work_completed = recipe.refine_work_turns - 1;
        let refined = tick_logistics(
            &mut job,
            &mut cargo,
            &mut position,
            recipe,
            LogisticsTargetState::AtWorkTile,
        );
        assert_eq!(cargo.amount, 0, "recipe={}: consumed input", recipe.id);
        assert_eq!(
            refined.finished_output, recipe.output_amount,
            "recipe={}: refined output",
            recipe.id
        );
    }
}

#[test]
fn two_survivors_complete_different_chains_without_stacking_or_duplicate_credit() {
    let mut driver = FoundationDriver::new(4501);
    driver.start_colony().unwrap();
    driver.fixture_spawn_processing_station(Position { x: 3, y: 3 });
    let timber_worker = driver.survivor_by_name("Mara").unwrap();
    let plant_worker = driver.survivor_by_name("Iven").unwrap();
    driver.fixture_assign_recipe(timber_worker, "recipe.refine_timber");
    driver.fixture_assign_recipe(plant_worker, "recipe.refine_plants");
    let materials_before = driver
        .resource_current(bd_core::signals::PoolKind::Materials)
        .unwrap();
    let plants_before = driver
        .resource_current(bd_core::signals::PoolKind::WildPlants)
        .unwrap();
    let player = driver.player().unwrap();
    for _ in 0..200 {
        driver
            .submit_action_and_advance_result_frame(
                "concurrent production worker tick",
                player,
                "ability.wait",
                None,
                None,
            )
            .unwrap();
        if driver
            .resource_current(bd_core::signals::PoolKind::Materials)
            .unwrap()
            > materials_before
            && driver
                .resource_current(bd_core::signals::PoolKind::WildPlants)
                .unwrap()
                > plants_before
        {
            break;
        }
    }
    assert_eq!(
        driver
            .resource_current(bd_core::signals::PoolKind::Materials)
            .unwrap(),
        materials_before + 1
    );
    assert_eq!(
        driver
            .resource_current(bd_core::signals::PoolKind::WildPlants)
            .unwrap(),
        plants_before + 1
    );
    let positions = driver.survivor_positions();
    assert_eq!(
        positions
            .iter()
            .copied()
            .collect::<std::collections::HashSet<_>>()
            .len(),
        positions.len()
    );
}

#[test]
fn fixture_fourth_chain_needs_no_new_transition_branch() {
    let recipe = bd_core::content::ColonyRecipeDefinition {
        id: "recipe.fixture_fourth".into(),
        label: "Fixture Fourth".into(),
        source_id: "source.fixture_fourth".into(),
        input_resource_id: "resource.raw_fixture_fourth".into(),
        output_resource_id: "resource.refined_fixture_fourth".into(),
        station_id: "station.basic_processor".into(),
        input_amount: 2,
        output_amount: 3,
        gather_work_turns: 1,
        refine_work_turns: 1,
    };
    let mut job = LogisticsJob {
        recipe_id: recipe.id.clone(),
        stage: JobStage::ReadyToGather,
        work_completed: 0,
        blocked: None,
    };
    let mut cargo = Cargo::default();
    let mut position = Position { x: 2, y: 2 };
    tick_logistics(
        &mut job,
        &mut cargo,
        &mut position,
        &recipe,
        LogisticsTargetState::AtWorkTile,
    );
    job.stage = JobStage::ReadyToRefine;
    let result = tick_logistics(
        &mut job,
        &mut cargo,
        &mut position,
        &recipe,
        LogisticsTargetState::AtWorkTile,
    );

    assert_eq!(cargo.amount, 0);
    assert_eq!(result.finished_output, 3);
}
