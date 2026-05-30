use broken_divinity::ui::input_hints::{
    HELP_TOGGLE_KEY, INVENTORY_TOGGLE_HINT_TEXT, INVENTORY_TOGGLE_PRIMARY_KEY,
    INVENTORY_TOGGLE_SECONDARY_KEY, JOURNAL_TOGGLE_KEY, OVERWORLD_RETURN_HINT_TEXT,
    OVERWORLD_RETURN_KEY, STATS_TOGGLE_HINT_TEXT, STATS_TOGGLE_KEY,
};

const PROTOTYPE_FEATURE_ATTR: &str = "#[cfg(feature = \"ux-prototypes\")]";
const PROTOTYPE_BIN_REQUIRED_FEATURE: &str = "required-features = [\"ux-prototypes\"]";

const PROTOTYPE_MODULE_DECLARATIONS: [&str; 7] = [
    "pub mod ux_colony_prototype;",
    "pub mod ux_dungeon_map_prototypes;",
    "pub mod ux_dungeon_style_prototype;",
    "pub mod ux_inventory_equipment_prototype;",
    "pub mod ux_overworld_prototype;",
    "pub mod ux_unified_prototype;",
    "pub mod ux_prototypes;",
];

const PROTOTYPE_BIN_NAMES: [&str; 7] = [
    "name = \"ux_prototypes\"",
    "name = \"ux_inventory_equipment_prototype\"",
    "name = \"ux_unified_prototype\"",
    "name = \"ux_overworld_prototype\"",
    "name = \"ux_dungeon_style_prototype\"",
    "name = \"ux_dungeon_map_prototypes\"",
    "name = \"ux_colony_prototype\"",
];

const MIGRATED_RUNTIME_PANELS: [&str; 7] = [
    "ui::hud::draw_hud",
    "ui::inventory_panel::draw_inventory_panel",
    "ui::overworld_panel::draw_overworld_panel",
    "ui::colony_panel::draw_resource_bar",
    "ui::colony_panel::draw_survivor_panel",
    "ui::colony_panel::draw_build_panel",
    "ui::journal_panel::draw_journal_panel",
];

const RUNTIME_AUTHORITY_MARKERS: [&str; 3] = [
    "fn configure_runtime_authority_app(app: &mut App)",
    "fn prototype_mode_enabled() -> bool",
    "Runtime UI authority default",
];

const MENU_SHORTCUT_HINT_DECLARATION: &str = "pub const MENU_SHORTCUT_HINT_TEXT: &str";
const SAVE_AND_QUIT_LABEL_DECLARATION: &str = "pub const SAVE_AND_QUIT_LABEL: &str";
const SAVE_AND_QUIT_HINT_DECLARATION: &str = "pub const SAVE_AND_QUIT_HINT_TEXT: &str";
const JOURNAL_TOGGLE_HINT_DECLARATION: &str = "pub const JOURNAL_TOGGLE_HINT_TEXT: &str";
const PROTOTYPE_DEPRECATION_MARKER: &str = "DEPRECATED: prototype-only binary";
const RUNTIME_ACTION_LANGUAGE_MODULE_DECLARATION: &str =
    "pub mod runtime_action_language;";
const RUNTIME_ACTION_LANGUAGE_MARKER: &str = "RuntimeActionLanguage";

#[test]
fn runtime_main_does_not_register_prototype_draw_paths() {
    let main_source = include_str!("../src/main.rs");
    let Some(runtime_section) = main_source
        .split("fn configure_runtime_authority_app(app: &mut App)")
        .nth(1)
    else {
        panic!("expected runtime authority helper in main.rs");
    };

    assert!(
        !runtime_section.contains("ui::ux_"),
        "runtime authority branch should not directly register prototype UI modules"
    );
}

#[test]
fn ui_module_gates_prototype_exports_behind_feature() {
    let ui_mod_source = include_str!("../src/ui/mod.rs");

    for declaration in PROTOTYPE_MODULE_DECLARATIONS {
        let expected = format!("{PROTOTYPE_FEATURE_ATTR}\n{declaration}");
        assert!(
            ui_mod_source.contains(&expected),
            "prototype module declaration must be feature-gated: {declaration}"
        );
    }
}

#[test]
fn cargo_toml_requires_feature_for_prototype_bins() {
    let cargo_toml = include_str!("../Cargo.toml");
    assert!(
        cargo_toml.contains("ux-prototypes = []"),
        "Cargo feature ux-prototypes must exist"
    );

    for bin_name in PROTOTYPE_BIN_NAMES {
        let Some(bin_start) = cargo_toml.find(bin_name) else {
            panic!("missing prototype bin entry: {bin_name}");
        };
        let bin_section = &cargo_toml[bin_start..];
        assert!(
            bin_section.contains(PROTOTYPE_BIN_REQUIRED_FEATURE),
            "prototype bin must require ux-prototypes feature: {bin_name}"
        );
    }
}

#[test]
fn default_feature_set_does_not_enable_prototypes() {
    let cargo_toml = include_str!("../Cargo.toml");
    assert!(
        cargo_toml.contains("default = []"),
        "default feature set must not enable ux-prototypes"
    );
}

#[test]
fn dev_feature_set_does_not_force_prototypes() {
    let cargo_toml = include_str!("../Cargo.toml");
    assert!(
        cargo_toml.contains("dev = [\"bevy/dynamic_linking\", \"dep:bevy_brp_extras\"]"),
        "dev feature should not force prototype feature"
    );
}

#[test]
fn runtime_launch_path_declares_runtime_authority() {
    let main_source = include_str!("../src/main.rs");

    for marker in RUNTIME_AUTHORITY_MARKERS {
        assert!(
            main_source.contains(marker),
            "main entrypoint must declare runtime authority marker: {marker}"
        );
    }
}

#[test]
fn runtime_main_registers_migrated_runtime_panels() {
    let main_source = include_str!("../src/main.rs");

    for panel_path in MIGRATED_RUNTIME_PANELS {
        assert!(
            main_source.contains(panel_path),
            "runtime authority branch must register panel path: {panel_path}"
        );
    }
}

#[test]
fn prototype_bins_are_marked_deprecated_for_production_flow() {
    let prototype_bins = [
        include_str!("../src/bin/ux_prototypes.rs"),
        include_str!("../src/bin/ux_inventory_equipment_prototype.rs"),
        include_str!("../src/bin/ux_unified_prototype.rs"),
        include_str!("../src/bin/ux_overworld_prototype.rs"),
        include_str!("../src/bin/ux_dungeon_style_prototype.rs"),
        include_str!("../src/bin/ux_dungeon_map_prototypes.rs"),
        include_str!("../src/bin/ux_colony_prototype.rs"),
    ];

    for source in prototype_bins {
        assert!(
            source.contains(PROTOTYPE_DEPRECATION_MARKER),
            "prototype binaries must carry explicit deprecation marker"
        );
    }
}

#[test]
fn ui_module_exports_runtime_action_language_policy() {
    let ui_mod_source = include_str!("../src/ui/mod.rs");

    assert!(
        ui_mod_source.contains(RUNTIME_ACTION_LANGUAGE_MODULE_DECLARATION),
        "ui module must export runtime action language policy module"
    );
}

#[test]
fn runtime_panels_use_shared_runtime_action_language() {
    let runtime_app_source = include_str!("../src/runtime_app.rs");
    let menu_source = include_str!("../src/ui/menu.rs");
    let colony_source = include_str!("../src/ui/colony_panel.rs");
    let overworld_source = include_str!("../src/ui/overworld_panel.rs");

    assert!(
        runtime_app_source.contains(RUNTIME_ACTION_LANGUAGE_MARKER),
        "runtime app must use shared runtime action language policy"
    );
    assert!(
        menu_source.contains(RUNTIME_ACTION_LANGUAGE_MARKER),
        "menu surface must use shared runtime action language policy"
    );
    assert!(
        colony_source.contains(RUNTIME_ACTION_LANGUAGE_MARKER),
        "colony surface must use shared runtime action language policy"
    );
    assert!(
        overworld_source.contains(RUNTIME_ACTION_LANGUAGE_MARKER),
        "overworld surface must use shared runtime action language policy"
    );
}

#[test]
fn key_binding_tokens_and_hint_copy_are_canonical() {
    assert_eq!(INVENTORY_TOGGLE_PRIMARY_KEY, bevy::prelude::KeyCode::KeyI);
    assert_eq!(INVENTORY_TOGGLE_SECONDARY_KEY, bevy::prelude::KeyCode::Tab);
    assert_eq!(JOURNAL_TOGGLE_KEY, bevy::prelude::KeyCode::KeyJ);
    assert_eq!(STATS_TOGGLE_KEY, bevy::prelude::KeyCode::KeyK);
    assert_eq!(HELP_TOGGLE_KEY, bevy::prelude::KeyCode::F1);
    assert_eq!(OVERWORLD_RETURN_KEY, bevy::prelude::KeyCode::Escape);

    assert!(INVENTORY_TOGGLE_HINT_TEXT.contains("I"));
    assert!(INVENTORY_TOGGLE_HINT_TEXT.contains("Tab"));
    assert!(OVERWORLD_RETURN_HINT_TEXT.contains("Esc"));
    assert!(STATS_TOGGLE_HINT_TEXT.contains("K"));
}

#[test]
fn migrated_panels_use_shared_input_hint_tokens() {
    let inventory_panel_source = include_str!("../src/ui/inventory_panel.rs");
    let journal_panel_source = include_str!("../src/ui/journal_panel.rs");
    let stats_panel_source = include_str!("../src/ui/stats_progression_panel.rs");
    let help_panel_source = include_str!("../src/ui/help_panel.rs");
    let input_hints_source = include_str!("../src/ui/input_hints.rs");
    let menu_source = include_str!("../src/ui/menu.rs");
    let colony_panel_source = include_str!("../src/ui/colony_panel.rs");
    let overworld_panel_source = include_str!("../src/ui/overworld_panel.rs");

    assert!(inventory_panel_source.contains("INVENTORY_TOGGLE_PRIMARY_KEY"));
    assert!(inventory_panel_source.contains("INVENTORY_TOGGLE_SECONDARY_KEY"));
    assert!(journal_panel_source.contains("JOURNAL_TOGGLE_KEY"));
    assert!(stats_panel_source.contains("STATS_TOGGLE_KEY"));
    assert!(stats_panel_source.contains("STATS_TOGGLE_HINT_TEXT"));
    assert!(help_panel_source.contains("HELP_TOGGLE_KEY"));
    assert!(input_hints_source.contains(MENU_SHORTCUT_HINT_DECLARATION));
    assert!(input_hints_source.contains(SAVE_AND_QUIT_LABEL_DECLARATION));
    assert!(input_hints_source.contains(SAVE_AND_QUIT_HINT_DECLARATION));
    assert!(input_hints_source.contains(JOURNAL_TOGGLE_HINT_DECLARATION));
    assert!(menu_source.contains("MENU_SHORTCUT_HINT_TEXT"));
    assert!(menu_source.contains("use crate::ui::input_hints::MENU_SHORTCUT_HINT_TEXT;"));
    assert!(journal_panel_source.contains("JOURNAL_TOGGLE_HINT_TEXT"));
    assert!(colony_panel_source.contains("SAVE_AND_QUIT_LABEL"));
    assert!(colony_panel_source.contains("SAVE_AND_QUIT_HINT_TEXT"));
    assert!(overworld_panel_source.contains("SAVE_AND_QUIT_LABEL"));
    assert!(overworld_panel_source.contains("SAVE_AND_QUIT_HINT_TEXT"));
}
