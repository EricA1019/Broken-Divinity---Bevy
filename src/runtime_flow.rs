#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowNode {
    Menu,
    Colony,
    Overworld,
    Dungeon,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowAction {
    StartRun,
    TravelToOverworld,
    EnterDungeon,
    ReturnToColony,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowError {
    InvalidTransition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlowSnapshot {
    node: FlowNode,
}

use bevy::prelude::Resource;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Resource)]
pub struct RuntimeFlow {
    node: FlowNode,
}

impl RuntimeFlow {
    const INITIAL_NODE: FlowNode = FlowNode::Menu;

    pub fn new() -> Self {
        Self {
            node: Self::INITIAL_NODE,
        }
    }

    pub fn current(&self) -> FlowNode {
        self.node
    }

    pub fn set_current(&mut self, node: FlowNode) {
        self.node = node;
    }

    pub fn apply(&mut self, action: FlowAction) -> Result<(), FlowError> {
        let next = next_node(self.node, action)?;
        self.node = next;
        Ok(())
    }

    pub fn snapshot(&self) -> FlowSnapshot {
        FlowSnapshot { node: self.node }
    }

    pub fn from_snapshot(snapshot: FlowSnapshot) -> Self {
        Self { node: snapshot.node }
    }
}

impl Default for RuntimeFlow {
    fn default() -> Self {
        Self::new()
    }
}

fn next_node(current: FlowNode, action: FlowAction) -> Result<FlowNode, FlowError> {
    match (current, action) {
        (FlowNode::Menu, FlowAction::StartRun) => Ok(FlowNode::Colony),
        (FlowNode::Colony, FlowAction::TravelToOverworld) => Ok(FlowNode::Overworld),
        (FlowNode::Overworld, FlowAction::EnterDungeon) => Ok(FlowNode::Dungeon),
        (FlowNode::Dungeon, FlowAction::ReturnToColony) => Ok(FlowNode::Colony),
        _ => Err(FlowError::InvalidTransition),
    }
}
