use bevy::prelude::Resource;

use super::unified_action_language::UnifiedActionLanguage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UnifiedContinuityScreen {
    MainMenu,
    Dungeon,
    Colony,
    Overworld,
    Dossier,
    InventoryEquipment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct UnifiedContinuityCue {
    pub current_context: &'static str,
    pub next_action: &'static str,
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct UnifiedContinuityState {
    previous_context: &'static str,
    current_context: &'static str,
    next_action: &'static str,
}

impl Default for UnifiedContinuityState {
    fn default() -> Self {
        let cue = cue_for_screen(UnifiedContinuityScreen::MainMenu);
        Self {
            previous_context: cue.current_context,
            current_context: cue.current_context,
            next_action: cue.next_action,
        }
    }
}

impl UnifiedContinuityState {
    pub(crate) fn update(&mut self, screen: UnifiedContinuityScreen) {
        let cue = cue_for_screen(screen);
        self.previous_context = self.current_context;
        self.current_context = cue.current_context;
        self.next_action = cue.next_action;
    }

    pub(crate) fn previous_context(&self) -> &'static str {
        self.previous_context
    }

    pub(crate) fn current_context(&self) -> &'static str {
        self.current_context
    }

    pub(crate) fn next_action(&self) -> &'static str {
        self.next_action
    }
}

pub(crate) fn cue_for_screen(screen: UnifiedContinuityScreen) -> UnifiedContinuityCue {
    match screen {
        UnifiedContinuityScreen::MainMenu => UnifiedContinuityCue {
            current_context: UnifiedActionLanguage::current_context_for(screen),
            next_action: UnifiedActionLanguage::next_action_for(screen),
        },
        UnifiedContinuityScreen::Dungeon => UnifiedContinuityCue {
            current_context: UnifiedActionLanguage::current_context_for(screen),
            next_action: UnifiedActionLanguage::next_action_for(screen),
        },
        UnifiedContinuityScreen::Colony => UnifiedContinuityCue {
            current_context: UnifiedActionLanguage::current_context_for(screen),
            next_action: UnifiedActionLanguage::next_action_for(screen),
        },
        UnifiedContinuityScreen::Overworld => UnifiedContinuityCue {
            current_context: UnifiedActionLanguage::current_context_for(screen),
            next_action: UnifiedActionLanguage::next_action_for(screen),
        },
        UnifiedContinuityScreen::Dossier => UnifiedContinuityCue {
            current_context: UnifiedActionLanguage::current_context_for(screen),
            next_action: UnifiedActionLanguage::next_action_for(screen),
        },
        UnifiedContinuityScreen::InventoryEquipment => UnifiedContinuityCue {
            current_context: UnifiedActionLanguage::current_context_for(screen),
            next_action: UnifiedActionLanguage::next_action_for(screen),
        },
    }
}
