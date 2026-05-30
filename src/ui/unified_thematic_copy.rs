use super::unified_continuity::UnifiedContinuityScreen;

pub(crate) struct UnifiedThematicCopyCatalog;

impl UnifiedThematicCopyCatalog {
    pub(crate) fn line_for_screen(screen: UnifiedContinuityScreen) -> &'static str {
        match screen {
            UnifiedContinuityScreen::MainMenu => {
                "The shelter waits. Choose the next run with clean intent."
            }
            UnifiedContinuityScreen::Dungeon => {
                "The wall is warm. It should not be warm."
            }
            UnifiedContinuityScreen::Colony => {
                "Three checks first: food, water, and who can still stand."
            }
            UnifiedContinuityScreen::Overworld => {
                "Distance is a cost. Weather decides how much you pay."
            }
            UnifiedContinuityScreen::Dossier => {
                "Read the record like field notes, not prophecy."
            }
            UnifiedContinuityScreen::InventoryEquipment => {
                "Equipment is procedure. Procedure keeps people alive."
            }
        }
    }
}
