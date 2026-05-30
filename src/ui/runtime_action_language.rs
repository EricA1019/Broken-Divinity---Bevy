use crate::runtime_flow::FlowNode;

pub(crate) struct RuntimeActionLanguage;

impl RuntimeActionLanguage {
    pub(crate) fn menu_primary_cta_label() -> &'static str {
        "New Game"
    }

    pub(crate) fn colony_primary_cta_label() -> &'static str {
        "Primary action: Manage survivors and station output."
    }

    pub(crate) fn overworld_primary_cta_label() -> &'static str {
        "Click a connected node to travel."
    }

    pub(crate) fn dungeon_primary_cta_label() -> &'static str {
        "Return to Colony"
    }

    pub(crate) fn flow_primary_label(node: FlowNode) -> &'static str {
        match node {
            FlowNode::Menu => "Start Run",
            FlowNode::Colony => "Travel to Overworld",
            FlowNode::Overworld => "Enter Dungeon",
            FlowNode::Dungeon => Self::dungeon_primary_cta_label(),
        }
    }
}
