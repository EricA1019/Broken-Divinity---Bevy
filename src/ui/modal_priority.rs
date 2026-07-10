use bevy::prelude::*;

use crate::game::colony::raids::{ActiveRaid, RaidPhase};
use crate::ui::help_panel::HelpOpen;

#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct ModalPriorityCoordinator {
    pub block_help_when_critical: bool,
}

impl Default for ModalPriorityCoordinator {
    fn default() -> Self {
        Self {
            block_help_when_critical: true,
        }
    }
}

#[derive(Resource, Debug, Clone, Reflect, Default)]
#[reflect(Resource)]
pub struct ModalBlockers {
    pub critical_modal_active: bool,
}

fn critical_modal_active(active_raid: Option<&ActiveRaid>) -> bool {
    active_raid.is_some_and(|raid| matches!(raid.phase, RaidPhase::Planning | RaidPhase::InProgress))
}

pub fn can_open_help_panel(
    active_raid: Option<&ActiveRaid>,
    coordinator: &ModalPriorityCoordinator,
) -> bool {
    if !coordinator.block_help_when_critical {
        return true;
    }

    !critical_modal_active(active_raid)
}

pub fn apply_modal_priority_policy(
    mut help_open: ResMut<HelpOpen>,
    blockers: Option<ResMut<ModalBlockers>>,
    active_raid: Option<Res<ActiveRaid>>,
    coordinator: Option<Res<ModalPriorityCoordinator>>,
) {
    let critical_active = critical_modal_active(active_raid.as_deref());
    if let Some(mut blockers) = blockers {
        blockers.critical_modal_active = critical_active;
    }

    let coordinator = coordinator
        .as_deref()
        .cloned()
        .unwrap_or_default();
    if help_open.0 && !can_open_help_panel(active_raid.as_deref(), &coordinator) {
        help_open.0 = false;
    }
}
