use broken_divinity::runtime_flow::{
    FlowAction,
    FlowError,
    FlowNode,
    RuntimeFlow,
};

#[test]
fn menu_to_return_flow_is_reachable() {
    let mut flow = RuntimeFlow::new();

    let start = flow.apply(FlowAction::StartRun);
    assert!(start.is_ok());
    assert_eq!(flow.current(), FlowNode::Colony);

    let to_overworld = flow.apply(FlowAction::TravelToOverworld);
    assert!(to_overworld.is_ok());
    assert_eq!(flow.current(), FlowNode::Overworld);

    let to_dungeon = flow.apply(FlowAction::EnterDungeon);
    assert!(to_dungeon.is_ok());
    assert_eq!(flow.current(), FlowNode::Dungeon);

    let return_to_colony = flow.apply(FlowAction::ReturnToColony);
    assert!(return_to_colony.is_ok());
    assert_eq!(flow.current(), FlowNode::Colony);
}

#[test]
fn invalid_transition_is_rejected() {
    let mut flow = RuntimeFlow::new();

    let direct_dungeon = flow.apply(FlowAction::EnterDungeon);
    assert_eq!(direct_dungeon, Err(FlowError::InvalidTransition));
    assert_eq!(flow.current(), FlowNode::Menu);
}

#[test]
fn snapshot_roundtrip_preserves_runtime_node() {
    let mut flow = RuntimeFlow::new();
    let _ = flow.apply(FlowAction::StartRun);
    let _ = flow.apply(FlowAction::TravelToOverworld);

    let snapshot = flow.snapshot();
    let restored = RuntimeFlow::from_snapshot(snapshot);

    assert_eq!(restored.current(), FlowNode::Overworld);
}

#[test]
fn each_reachable_state_supports_snapshot_roundtrip() {
    let menu_snapshot = RuntimeFlow::new().snapshot();
    assert_eq!(RuntimeFlow::from_snapshot(menu_snapshot).current(), FlowNode::Menu);

    let mut colony = RuntimeFlow::new();
    let _ = colony.apply(FlowAction::StartRun);
    let colony_snapshot = colony.snapshot();
    assert_eq!(RuntimeFlow::from_snapshot(colony_snapshot).current(), FlowNode::Colony);

    let mut overworld = RuntimeFlow::new();
    let _ = overworld.apply(FlowAction::StartRun);
    let _ = overworld.apply(FlowAction::TravelToOverworld);
    let overworld_snapshot = overworld.snapshot();
    assert_eq!(
        RuntimeFlow::from_snapshot(overworld_snapshot).current(),
        FlowNode::Overworld
    );

    let mut dungeon = RuntimeFlow::new();
    let _ = dungeon.apply(FlowAction::StartRun);
    let _ = dungeon.apply(FlowAction::TravelToOverworld);
    let _ = dungeon.apply(FlowAction::EnterDungeon);
    let dungeon_snapshot = dungeon.snapshot();
    assert_eq!(RuntimeFlow::from_snapshot(dungeon_snapshot).current(), FlowNode::Dungeon);
}
