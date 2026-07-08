const RUNTIME_COPY_MODULE_DECLARATION: &str = "pub mod runtime_copy;";
const RUNTIME_COPY_MARKER: &str = "RuntimeCopy";

const REQUIRED_RUNTIME_COPY_CONSTANTS: [&str; 16] = [
    "pub(crate) const MENU_TITLE: &str",
    "pub(crate) const MENU_SUBTITLE: &str",
    "pub(crate) const MENU_LOAD_GAME_LABEL: &str",
    "pub(crate) const MENU_QUIT_LABEL: &str",
    "pub(crate) const MENU_QUIT_CONFIRM_PROMPT: &str",
    "pub(crate) const MENU_QUIT_CONFIRM_LABEL: &str",
    "pub(crate) const MENU_QUIT_CANCEL_LABEL: &str",
    "pub(crate) const INVENTORY_WINDOW_TITLE: &str",
    "pub(crate) const JOURNAL_WINDOW_TITLE: &str",
    "pub(crate) const STATS_WINDOW_TITLE: &str",
    "pub(crate) const RESEARCH_WINDOW_TITLE: &str",
    "pub(crate) const PERK_WINDOW_TITLE: &str",
    "pub(crate) const GABRIEL_WINDOW_TITLE: &str",
    "pub(crate) const GAMELOG_PANEL_TITLE: &str",
    "pub(crate) const GENERIC_CLOSE_BUTTON_LABEL: &str",
    "pub(crate) const NO_SAVE_HELPER_TEXT: &str",
];

#[test]
fn ui_module_exports_runtime_copy_policy() {
    let ui_mod_source = include_str!("../src/ui/mod.rs");

    assert!(
        ui_mod_source.contains(RUNTIME_COPY_MODULE_DECLARATION),
        "ui module must export runtime copy policy module"
    );
}

#[test]
fn runtime_copy_module_declares_required_constants() {
    let runtime_copy_source = include_str!("../src/ui/runtime_copy.rs");

    for marker in REQUIRED_RUNTIME_COPY_CONSTANTS {
        assert!(
            runtime_copy_source.contains(marker),
            "runtime_copy.rs missing required constant declaration: {marker}"
        );
    }
}

#[test]
fn runtime_surfaces_use_shared_runtime_copy_constants() {
    let menu_source = include_str!("../src/ui/menu.rs");
    let inventory_source = include_str!("../src/ui/inventory_panel.rs");
    let journal_source = include_str!("../src/ui/journal_panel.rs");
    let stats_source = include_str!("../src/ui/stats_progression_panel.rs");
    let colony_source = include_str!("../src/ui/colony_panel.rs");
    let perk_source = include_str!("../src/ui/perk_choice_panel.rs");
    let gabriel_source = include_str!("../src/ui/gabriel_dialogue_panel.rs");
    let gamelog_source = include_str!("../src/ui/gamelog_panel.rs");

    assert!(menu_source.contains(RUNTIME_COPY_MARKER));
    assert!(inventory_source.contains(RUNTIME_COPY_MARKER));
    assert!(journal_source.contains(RUNTIME_COPY_MARKER));
    assert!(stats_source.contains(RUNTIME_COPY_MARKER));
    assert!(colony_source.contains(RUNTIME_COPY_MARKER));
    assert!(perk_source.contains(RUNTIME_COPY_MARKER));
    assert!(gabriel_source.contains(RUNTIME_COPY_MARKER));
    assert!(gamelog_source.contains(RUNTIME_COPY_MARKER));
}
