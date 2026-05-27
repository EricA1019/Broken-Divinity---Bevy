#[derive(Debug, Clone, Copy)]
pub struct EscapeContext {
    pub modal_open: bool,
    pub can_pause: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EscapeAction {
    CloseModal,
    PauseGame,
    NoOp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EscapeGuidanceEvent {
    ShowHint,
    Suppressed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EscapeHintContext {
    ModalOpen,
    PauseAvailable,
    None,
}

#[derive(Debug, Clone, Copy)]
pub struct EscapeGuidanceEngine {
    acknowledged: bool,
    last_context: EscapeHintContext,
}

impl EscapeGuidanceEngine {
    pub fn new() -> Self {
        Self {
            acknowledged: false,
            last_context: EscapeHintContext::None,
        }
    }

    pub fn guidance(&mut self, context: EscapeContext) -> EscapeGuidanceEvent {
        if self.acknowledged {
            return EscapeGuidanceEvent::Suppressed;
        }

        let hint_context = to_hint_context(context);
        if hint_context == EscapeHintContext::None {
            self.last_context = EscapeHintContext::None;
            return EscapeGuidanceEvent::Suppressed;
        }

        if self.last_context == hint_context {
            return EscapeGuidanceEvent::Suppressed;
        }

        self.last_context = hint_context;
        EscapeGuidanceEvent::ShowHint
    }

    pub fn acknowledge(&mut self) {
        self.acknowledged = true;
    }
}

impl Default for EscapeGuidanceEngine {
    fn default() -> Self {
        Self::new()
    }
}

fn to_hint_context(context: EscapeContext) -> EscapeHintContext {
    if context.modal_open {
        return EscapeHintContext::ModalOpen;
    }

    if context.can_pause {
        return EscapeHintContext::PauseAvailable;
    }

    EscapeHintContext::None
}

pub fn resolve_escape_action(context: EscapeContext) -> EscapeAction {
    if context.modal_open {
        return EscapeAction::CloseModal;
    }

    if context.can_pause {
        return EscapeAction::PauseGame;
    }

    EscapeAction::NoOp
}
