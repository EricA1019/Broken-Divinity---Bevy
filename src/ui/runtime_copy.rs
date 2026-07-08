pub(crate) const MENU_TITLE: &str = "BROKEN DIVINITY";
pub(crate) const MENU_SUBTITLE: &str = "A post-apocalyptic roguelike";
pub(crate) const MENU_LOAD_GAME_LABEL: &str = "Load Game";
pub(crate) const MENU_QUIT_LABEL: &str = "Quit";
pub(crate) const MENU_QUIT_CONFIRM_PROMPT: &str = "Quit the game?";
pub(crate) const MENU_QUIT_CONFIRM_LABEL: &str = "Confirm";
pub(crate) const MENU_QUIT_CANCEL_LABEL: &str = "Cancel";
pub(crate) const NO_SAVE_HELPER_TEXT: &str = "No save found yet. Start a New Game to create one.";

pub(crate) const INVENTORY_WINDOW_TITLE: &str = "Inventory";
pub(crate) const JOURNAL_WINDOW_TITLE: &str = "Lore Journal";
pub(crate) const STATS_WINDOW_TITLE: &str = "Stats & Progression";
pub(crate) const RESEARCH_WINDOW_TITLE: &str = "Research";
pub(crate) const PERK_WINDOW_TITLE: &str = "Perk Unlocked";
pub(crate) const GABRIEL_WINDOW_TITLE: &str = "Gabriel";
pub(crate) const GAMELOG_PANEL_TITLE: &str = "Game Log";
pub(crate) const GENERIC_CLOSE_BUTTON_LABEL: &str = "Close";

pub(crate) struct RuntimeCopy;

impl RuntimeCopy {
    pub(crate) fn menu_title() -> &'static str {
        MENU_TITLE
    }

    pub(crate) fn menu_subtitle() -> &'static str {
        MENU_SUBTITLE
    }

    pub(crate) fn menu_load_game_label() -> &'static str {
        MENU_LOAD_GAME_LABEL
    }

    pub(crate) fn menu_quit_label() -> &'static str {
        MENU_QUIT_LABEL
    }

    pub(crate) fn menu_quit_confirm_prompt() -> &'static str {
        MENU_QUIT_CONFIRM_PROMPT
    }

    pub(crate) fn menu_quit_confirm_label() -> &'static str {
        MENU_QUIT_CONFIRM_LABEL
    }

    pub(crate) fn menu_quit_cancel_label() -> &'static str {
        MENU_QUIT_CANCEL_LABEL
    }

    pub(crate) fn no_save_helper_text() -> &'static str {
        NO_SAVE_HELPER_TEXT
    }

    pub(crate) fn inventory_window_title() -> &'static str {
        INVENTORY_WINDOW_TITLE
    }

    pub(crate) fn journal_window_title() -> &'static str {
        JOURNAL_WINDOW_TITLE
    }

    pub(crate) fn stats_window_title() -> &'static str {
        STATS_WINDOW_TITLE
    }

    pub(crate) fn research_window_title() -> &'static str {
        RESEARCH_WINDOW_TITLE
    }

    pub(crate) fn perk_window_title() -> &'static str {
        PERK_WINDOW_TITLE
    }

    pub(crate) fn gabriel_window_title() -> &'static str {
        GABRIEL_WINDOW_TITLE
    }

    pub(crate) fn gamelog_panel_title() -> &'static str {
        GAMELOG_PANEL_TITLE
    }

    pub(crate) fn generic_close_button_label() -> &'static str {
        GENERIC_CLOSE_BUTTON_LABEL
    }
}
