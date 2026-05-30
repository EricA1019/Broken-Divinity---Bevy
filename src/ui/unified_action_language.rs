use super::unified_continuity::UnifiedContinuityScreen;

pub struct UnifiedActionLanguage;

impl UnifiedActionLanguage {
    pub(crate) fn current_context_for(screen: UnifiedContinuityScreen) -> &'static str {
        match screen {
            UnifiedContinuityScreen::MainMenu => "Main menu",
            UnifiedContinuityScreen::Dungeon => "Dungeon",
            UnifiedContinuityScreen::Colony => "Colony",
            UnifiedContinuityScreen::Overworld => "Overworld",
            UnifiedContinuityScreen::Dossier => "Dossier",
            UnifiedContinuityScreen::InventoryEquipment => "Inventory and equipment",
        }
    }

    pub(crate) fn next_action_for(screen: UnifiedContinuityScreen) -> &'static str {
        match screen {
            UnifiedContinuityScreen::MainMenu => "Press N to start a run.",
            UnifiedContinuityScreen::Dungeon => "Use WASD to advance one tile.",
            UnifiedContinuityScreen::Colony => "Press O to review overworld routes.",
            UnifiedContinuityScreen::Overworld => "Press D to move into dungeon operations.",
            UnifiedContinuityScreen::Dossier => "Press I to inspect loadout details.",
            UnifiedContinuityScreen::InventoryEquipment => {
                "Press C to return to colony planning."
            }
        }
    }
}
