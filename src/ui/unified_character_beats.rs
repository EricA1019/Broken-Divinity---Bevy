use bevy::prelude::Resource;

use super::unified_continuity::UnifiedContinuityScreen;

const GABRIEL_DUNGEON_BEAT: &str =
    "Gabriel: Keep low. The corridor teaches by killing the loud.";
const GABRIEL_COLONY_BEAT: &str =
    "Gabriel: Count supplies first; promises are cheaper than bullets.";
const GABRIEL_OVERWORLD_BEAT: &str =
    "Gabriel: Pick the route with an exit, not the route with pride.";
const GABRIEL_DOSSIER_BEAT: &str =
    "Gabriel: Read your wounds like orders. They tell you what to stop.";
const GABRIEL_INVENTORY_BEAT: &str =
    "Gabriel: If the loadout lies, the first turn tells the truth.";

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct UnifiedCharacterBeatState {
    active_beat: Option<&'static str>,
}

impl Default for UnifiedCharacterBeatState {
    fn default() -> Self {
        Self { active_beat: None }
    }
}

impl UnifiedCharacterBeatState {
    pub(crate) fn on_transition(
        &mut self,
        previous: UnifiedContinuityScreen,
        current: UnifiedContinuityScreen,
    ) {
        if previous == current {
            return;
        }

        self.active_beat = beat_for_screen(current);
    }

    pub(crate) fn active_character_beat(&self) -> Option<&'static str> {
        self.active_beat
    }
}

fn beat_for_screen(screen: UnifiedContinuityScreen) -> Option<&'static str> {
    match screen {
        UnifiedContinuityScreen::MainMenu => None,
        UnifiedContinuityScreen::Dungeon => Some(GABRIEL_DUNGEON_BEAT),
        UnifiedContinuityScreen::Colony => Some(GABRIEL_COLONY_BEAT),
        UnifiedContinuityScreen::Overworld => Some(GABRIEL_OVERWORLD_BEAT),
        UnifiedContinuityScreen::Dossier => Some(GABRIEL_DOSSIER_BEAT),
        UnifiedContinuityScreen::InventoryEquipment => Some(GABRIEL_INVENTORY_BEAT),
    }
}
