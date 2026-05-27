use bevy::prelude::*;

use crate::help_panel::HelpOpen;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalKind {
    Inventory,
    Menu,
    Dialogue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalPriorityDecision {
    ModalFirst,
    GameplayFirst,
}

pub fn resolve_modal_priority(modal_open: bool, _modal_kind: ModalKind) -> ModalPriorityDecision {
    if modal_open {
        return ModalPriorityDecision::ModalFirst;
    }

    ModalPriorityDecision::GameplayFirst
}

#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ModalPriorityCoordinator;

#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ModalBlockers {
    pub critical_modal_active: bool,
}

pub fn apply_modal_priority_policy(
    _coordinator: Option<Res<ModalPriorityCoordinator>>,
    blockers: Option<Res<ModalBlockers>>,
    help_open: Option<ResMut<HelpOpen>>,
) {
    let Some(blockers) = blockers else {
        return;
    };

    if !blockers.critical_modal_active {
        return;
    }

    if let Some(mut help_open) = help_open {
        help_open.0 = false;
    }
}
