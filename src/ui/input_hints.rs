use bevy::prelude::KeyCode;

pub const INVENTORY_TOGGLE_PRIMARY_KEY: KeyCode = KeyCode::KeyI;
pub const INVENTORY_TOGGLE_SECONDARY_KEY: KeyCode = KeyCode::Tab;
pub const JOURNAL_TOGGLE_KEY: KeyCode = KeyCode::KeyJ;
pub const STATS_TOGGLE_KEY: KeyCode = KeyCode::KeyK;
pub const HELP_TOGGLE_KEY: KeyCode = KeyCode::F1;
pub const OVERWORLD_RETURN_KEY: KeyCode = KeyCode::Escape;

pub const MENU_SHORTCUT_HINT_TEXT: &str =
	"Shortcuts: Enter/N new game | L load | Q quit | Y confirm | Esc cancel";
pub const SAVE_AND_QUIT_LABEL: &str = "Save & Quit";
pub const SAVE_AND_QUIT_HINT_TEXT: &str = "Save progress and return to the menu.";

pub const INVENTORY_TOGGLE_HINT_TEXT: &str = concat!(
	"Press ",
	"I",
	" or ",
	"Tab",
	" to toggle inventory.",
);

pub const JOURNAL_TOGGLE_HINT_TEXT: &str = concat!(
	"Press ",
	"J",
	" to open or close the lore journal.",
);

pub const STATS_TOGGLE_HINT_TEXT: &str = concat!(
	"[",
	"K",
	"] toggle  |  Core model from progression docs",
);

pub const OVERWORLD_RETURN_HINT_TEXT: &str = concat!(
	"Press ",
	"Esc",
	" to return to colony shelter.",
);

pub const UNIFIED_SCREEN_SWITCH_HINT_TEXT: &str =
	"M menu | D dungeon | C colony | O overworld | P dossier | I inventory";
pub const UNIFIED_CONTROL_CLUSTER_HINT_TEXT: &str =
	"1-6 layout tabs | Tab cycle | R reset | WASD move | Esc quit";
