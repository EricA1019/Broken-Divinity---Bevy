use bevy::prelude::*;
use broken_divinity::core::state::AppState;
use broken_divinity::runtime_app::{
    app_state_for_flow,
    flow_node_for_app_state,
    flow_primary_action,
    flow_primary_label,
    flow_surface,
    recap_for_flow,
    setup_runtime_scene,
};
use broken_divinity::runtime_flow::{FlowAction, FlowNode};
use broken_divinity::save_recap::{RecapRisk, SaveRecapState};

#[test]
fn runtime_shell_primary_actions_follow_flow() {
    assert_eq!(flow_primary_action(FlowNode::Menu), Some(FlowAction::StartRun));
    assert_eq!(flow_primary_action(FlowNode::Colony), Some(FlowAction::TravelToOverworld));
    assert_eq!(flow_primary_action(FlowNode::Overworld), Some(FlowAction::EnterDungeon));
    assert_eq!(flow_primary_action(FlowNode::Dungeon), Some(FlowAction::ReturnToColony));
}

#[test]
fn runtime_shell_labels_match_surface_loop() {
    assert_eq!(flow_primary_label(FlowNode::Menu), "Start Run");
    assert_eq!(flow_primary_label(FlowNode::Colony), "Travel to Overworld");
    assert_eq!(flow_primary_label(FlowNode::Overworld), "Enter Dungeon");
    assert_eq!(flow_primary_label(FlowNode::Dungeon), "Return to Colony");
}

#[test]
fn runtime_shell_recap_follows_flow_state() {
    let colony = recap_for_flow(FlowNode::Colony).expect("colony recap");
    assert_eq!(colony.state, SaveRecapState::Colony);
    assert_eq!(colony.risk, RecapRisk::Low);

    let overworld = recap_for_flow(FlowNode::Overworld).expect("overworld recap");
    assert_eq!(overworld.state, SaveRecapState::Overworld);
    assert_eq!(overworld.risk, RecapRisk::Medium);

    let dungeon = recap_for_flow(FlowNode::Dungeon).expect("dungeon recap");
    assert_eq!(dungeon.state, SaveRecapState::Dungeon);
    assert_eq!(dungeon.risk, RecapRisk::High);

    assert!(recap_for_flow(FlowNode::Menu).is_none());
}

#[test]
fn runtime_shell_surface_stops_at_dungeon() {
    assert!(flow_surface(FlowNode::Menu).is_some());
    assert!(flow_surface(FlowNode::Colony).is_some());
    assert!(flow_surface(FlowNode::Overworld).is_some());
    assert!(flow_surface(FlowNode::Dungeon).is_none());
}

#[test]
fn runtime_shell_startup_spawns_camera_for_visibility() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Startup, setup_runtime_scene);

    app.update();

    let mut query = app.world_mut().query::<&Camera2d>();
    assert!(
        query.iter(app.world()).next().is_some(),
        "Expected startup scene to spawn a Camera2d for visible rendering"
    );
}

#[test]
fn flow_and_app_state_mappings_cover_runtime_loop() {
    assert_eq!(app_state_for_flow(FlowNode::Menu), AppState::Menu);
    assert_eq!(app_state_for_flow(FlowNode::Colony), AppState::Colony);
    assert_eq!(app_state_for_flow(FlowNode::Overworld), AppState::Overworld);
    assert_eq!(app_state_for_flow(FlowNode::Dungeon), AppState::Dungeon);

    assert_eq!(flow_node_for_app_state(AppState::Menu), FlowNode::Menu);
    assert_eq!(flow_node_for_app_state(AppState::Colony), FlowNode::Colony);
    assert_eq!(flow_node_for_app_state(AppState::Overworld), FlowNode::Overworld);
    assert_eq!(flow_node_for_app_state(AppState::Dungeon), FlowNode::Dungeon);
}