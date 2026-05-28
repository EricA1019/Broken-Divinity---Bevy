use bevy::prelude::*;

use crate::game::colony::raids::ActiveRaid;
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
    blockers: Option<ResMut<ModalBlockers>>,
    active_raid: Option<Res<ActiveRaid>>,
    help_open: Option<ResMut<HelpOpen>>,
) {
    let blocker_was_active = blockers
        .as_ref()
        .is_some_and(|blockers| blockers.critical_modal_active);
    let raid_active = active_raid.is_some();
    let critical_modal_active = raid_active || blocker_was_active;

    if let Some(mut blockers) = blockers {
        blockers.critical_modal_active = raid_active;
    }

    if !critical_modal_active {
        return;
    }

    if let Some(mut help_open) = help_open {
        help_open.0 = false;
    }
}
