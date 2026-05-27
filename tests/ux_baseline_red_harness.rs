#[cfg(test)]
mod ux_baseline_red {
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::*;

    use broken_divinity::core::resources::WorldSeed;
    use broken_divinity::core::save::{SaveAppState, SaveGame, SaveGameTime, load_success_message};
    use broken_divinity::core::state::AppState;
    use broken_divinity::core::turn::GameTime;
    use broken_divinity::gamelog::GameLog;
    use broken_divinity::runtime_app::{app_state_for_flow, recap_for_flow};
    use broken_divinity::runtime_flow::{FlowAction, FlowNode, RuntimeFlow};
    use broken_divinity::save_recap::SaveRecapState;
    use broken_divinity::ui::help_panel::{HelpOpen, toggle_help};
    use broken_divinity::ui::menu::{MenuUiAction, MenuUiChoice, process_menu_action};
    use broken_divinity::ui::modal_priority::{ModalBlockers, ModalPriorityCoordinator, apply_modal_priority_policy};

    const EXPECTED_SCENARIO_COUNT: usize = 8;
    const MIN_EXECUTED_TESTS: usize = 1;
    const UX_RUN_ID: &str = "AXT00-20260527-REDUCED-RUNTIME-01";
    const OBSERVER_ID: &str = "copilot-gpt-5.4";
    const TEST_SEED: u64 = 424_242;
    const TEST_TURN: u32 = 17;

    fn scenario_ids() -> [&'static str; EXPECTED_SCENARIO_COUNT] {
        [
            "S01_MENU_TO_COLONY",
            "S02_COLONY_TO_OVERWORLD",
            "S03_OVERWORLD_TO_DUNGEON",
            "S04_DUNGEON_TO_COLONY_RETURN",
            "S05_SAVE_LOAD_COLONY",
            "S06_SAVE_LOAD_OVERWORLD",
            "S07_SAVE_LOAD_DUNGEON",
            "S08_MODAL_STRESS_TOGGLE",
        ]
    }

    fn non_zero_execution_gate(executed_tests: usize) -> bool {
        executed_tests >= MIN_EXECUTED_TESTS
    }

    fn log_scenario(scenario_id: &str, metric: &str, note: &str) {
        println!(
            "run_id={UX_RUN_ID} observer_id={OBSERVER_ID} scenario_id={scenario_id} metric={metric} notes={note}"
        );
    }

    fn progressed_flow_to(node: FlowNode) -> RuntimeFlow {
        let mut flow = RuntimeFlow::new();

        if matches!(node, FlowNode::Menu) {
            return flow;
        }

        let _ = flow.apply(FlowAction::StartRun);
        if matches!(node, FlowNode::Colony) {
            return flow;
        }

        let _ = flow.apply(FlowAction::TravelToOverworld);
        if matches!(node, FlowNode::Overworld) {
            return flow;
        }

        let _ = flow.apply(FlowAction::EnterDungeon);
        flow
    }

    #[test]
    fn scenario_ids_are_unique_and_non_empty() {
        let scenario_ids = scenario_ids();
        assert_eq!(scenario_ids.len(), EXPECTED_SCENARIO_COUNT);

        for id in &scenario_ids {
            assert!(!id.is_empty());
        }

        for (idx, id) in scenario_ids.iter().enumerate() {
            let duplicate = scenario_ids.iter().skip(idx + 1).any(|other| other == id);
            assert!(!duplicate, "duplicate scenario id: {id}");
        }
    }

    #[test]
    fn non_zero_execution_gate_rejects_zero() {
        let gate = non_zero_execution_gate(0);
        assert!(!gate);
    }

    #[test]
    fn non_zero_execution_gate_accepts_minimum() {
        let gate = non_zero_execution_gate(MIN_EXECUTED_TESTS);
        assert!(gate);
    }

    #[test]
    fn s01_menu_to_colony_executes_on_reduced_runtime() {
        let mut app = App::new();
        app.insert_resource(GameLog::default());
        app.insert_resource(GameTime { turn: TEST_TURN });
        app.insert_resource(MenuUiAction(Some(MenuUiChoice::NewGame { seed: TEST_SEED })));
        app.insert_resource(NextState::<AppState>::Unchanged);
        app.add_message::<AppExit>();
        app.add_systems(Update, process_menu_action);

        app.update();

        assert_eq!(app.world().resource::<WorldSeed>().0, TEST_SEED);
        assert!(matches!(
            *app.world().resource::<NextState<AppState>>(),
            NextState::Pending(AppState::Colony)
        ));

        log_scenario(
            "S01_MENU_TO_COLONY",
            "onboarding_clarity",
            "Reduced runtime menu flow reached colony with deterministic seed handoff.",
        );
    }

    #[test]
    fn s02_colony_to_overworld_executes_on_reduced_runtime() {
        let mut flow = progressed_flow_to(FlowNode::Colony);

        assert_eq!(app_state_for_flow(flow.current()), AppState::Colony);
        assert_eq!(flow.apply(FlowAction::TravelToOverworld), Ok(()));
        assert_eq!(flow.current(), FlowNode::Overworld);

        log_scenario(
            "S02_COLONY_TO_OVERWORLD",
            "control_discoverability",
            "Primary colony action advanced the reduced runtime into overworld.",
        );
    }

    #[test]
    fn s03_overworld_to_dungeon_executes_on_reduced_runtime() {
        let mut flow = progressed_flow_to(FlowNode::Overworld);

        assert_eq!(flow.apply(FlowAction::EnterDungeon), Ok(()));
        assert_eq!(flow.current(), FlowNode::Dungeon);
        assert_eq!(
            recap_for_flow(flow.current()).expect("dungeon recap").state,
            SaveRecapState::Dungeon
        );

        log_scenario(
            "S03_OVERWORLD_TO_DUNGEON",
            "navigation_predictability",
            "Overworld primary action reached dungeon and exposed the high-risk recap.",
        );
    }

    #[test]
    fn s04_dungeon_to_colony_return_executes_on_reduced_runtime() {
        let mut flow = progressed_flow_to(FlowNode::Dungeon);

        assert_eq!(flow.apply(FlowAction::ReturnToColony), Ok(()));
        assert_eq!(flow.current(), FlowNode::Colony);
        assert_eq!(
            recap_for_flow(flow.current()).expect("colony recap").state,
            SaveRecapState::Colony
        );

        log_scenario(
            "S04_DUNGEON_TO_COLONY_RETURN",
            "overall_first_session_confidence",
            "Dungeon return loop reached colony on the active reduced runtime path.",
        );
    }

    #[test]
    fn s05_save_load_colony_has_runtime_recap_surface() {
        let mut save = SaveGame::default();
        save.seed = TEST_SEED;
        save.app_state = SaveAppState::Colony;
        save.game_time = SaveGameTime { turn: TEST_TURN };

        assert_eq!(save.app_state.into_runtime_state(), AppState::Colony);
        assert_eq!(
            recap_for_flow(FlowNode::Colony).expect("colony recap").state,
            SaveRecapState::Colony
        );
        assert!(load_success_message().to_lowercase().contains("load"));

        log_scenario(
            "S05_SAVE_LOAD_COLONY",
            "save_load_continuity",
            "Colony save state maps back onto the reduced runtime with load recap copy available.",
        );
    }

    #[test]
    fn s06_save_load_overworld_has_runtime_recap_surface() {
        let mut save = SaveGame::default();
        save.seed = TEST_SEED;
        save.app_state = SaveAppState::Overworld;
        save.game_time = SaveGameTime { turn: TEST_TURN };

        assert_eq!(save.app_state.into_runtime_state(), AppState::Overworld);
        assert_eq!(
            recap_for_flow(FlowNode::Overworld)
                .expect("overworld recap")
                .state,
            SaveRecapState::Overworld
        );

        log_scenario(
            "S06_SAVE_LOAD_OVERWORLD",
            "save_load_continuity",
            "Overworld save state maps onto the reduced runtime and exposes the medium-risk recap.",
        );
    }

    #[test]
    fn s07_save_load_dungeon_has_runtime_recap_surface() {
        let mut save = SaveGame::default();
        save.seed = TEST_SEED;
        save.app_state = SaveAppState::Dungeon;
        save.game_time = SaveGameTime { turn: TEST_TURN };

        assert_eq!(save.app_state.into_runtime_state(), AppState::Dungeon);
        assert_eq!(
            recap_for_flow(FlowNode::Dungeon).expect("dungeon recap").state,
            SaveRecapState::Dungeon
        );

        log_scenario(
            "S07_SAVE_LOAD_DUNGEON",
            "save_load_continuity",
            "Dungeon save state maps onto the reduced runtime and preserves the high-risk recap.",
        );
    }

    #[test]
    fn s08_modal_stress_toggle_preserves_priority_on_reduced_runtime() {
        let mut world = World::new();
        world.insert_resource(HelpOpen(false));
        world.insert_resource(ModalPriorityCoordinator);
        world.insert_resource(ModalBlockers {
            critical_modal_active: true,
        });
        world.insert_resource(ButtonInput::<KeyCode>::default());

        for _ in 0..3 {
            {
                let mut keyboard = world.resource_mut::<ButtonInput<KeyCode>>();
                keyboard.release(KeyCode::F1);
                keyboard.press(KeyCode::F1);
            }
            let _ = world.run_system_once(toggle_help);
            let _ = world.run_system_once(apply_modal_priority_policy);
        }

        assert!(!world.resource::<HelpOpen>().0);

        log_scenario(
            "S08_MODAL_STRESS_TOGGLE",
            "error_edge_case_trust",
            "Critical modal priority kept help closed during rapid toggle stress.",
        );
    }
}
