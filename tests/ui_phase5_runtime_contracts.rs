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

const CUTOVER_MAIN_MARKERS: [&str; 3] = [
    "#[cfg(feature = \"dev\")]",
    "broken_divinity::ui::ux_unified_prototype::UnifiedPrototypePlugin",
    "#[cfg(not(feature = \"dev\"))]",
];

const DEV_FEATURE_MARKER: &str = "dev = [\"bevy/dynamic_linking\", \"dep:bevy_brp_extras\", \"ux-prototypes\"]";
const MENU_SHORTCUT_HINT_DECLARATION: &str = "pub const MENU_SHORTCUT_HINT_TEXT: &str";

#[test]
fn runtime_main_does_not_register_prototype_draw_paths() {
    let main_source = include_str!("../src/main.rs");
    let Some(legacy_section) = main_source
        .split("#[cfg(not(feature = \"dev\"))]\nfn main()")
        .nth(1)
    else {
        panic!("expected non-dev runtime branch in main.rs");
    };

    assert!(
        !legacy_section.contains("ui::ux_"),
        "legacy runtime branch should not directly register prototype UI modules"
    );
}

#[test]
fn inventory_toggle_hint_mentions_active_bindings() {
    assert!(INVENTORY_TOGGLE_HINT_TEXT.contains("I"));
    assert!(INVENTORY_TOGGLE_HINT_TEXT.contains("Tab"));
}

#[test]
fn overworld_return_hint_mentions_escape_binding() {
    assert!(OVERWORLD_RETURN_HINT_TEXT.contains("Esc"));
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
fn runtime_main_registers_migrated_runtime_panels() {
    let main_source = include_str!("../src/main.rs");

    for panel_path in MIGRATED_RUNTIME_PANELS {
        assert!(
            main_source.contains(panel_path),
            "runtime main must register migrated panel path: {panel_path}"
        );
    }
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

    assert!(inventory_panel_source.contains("INVENTORY_TOGGLE_PRIMARY_KEY"));
    assert!(inventory_panel_source.contains("INVENTORY_TOGGLE_SECONDARY_KEY"));
    assert!(journal_panel_source.contains("JOURNAL_TOGGLE_KEY"));
    assert!(stats_panel_source.contains("STATS_TOGGLE_KEY"));
    assert!(stats_panel_source.contains("STATS_TOGGLE_HINT_TEXT"));
    assert!(help_panel_source.contains("HELP_TOGGLE_KEY"));
    assert!(input_hints_source.contains(MENU_SHORTCUT_HINT_DECLARATION));
    assert!(menu_source.contains("MENU_SHORTCUT_HINT_TEXT"));
    assert!(menu_source.contains("use crate::ui::input_hints::MENU_SHORTCUT_HINT_TEXT;"));
}

#[test]
fn dev_cutover_entrypoint_is_explicit_about_unified_prototype_default() {
    let main_source = include_str!("../src/main.rs");

    for marker in CUTOVER_MAIN_MARKERS {
        assert!(
            main_source.contains(marker),
            "main entrypoint must declare cutover marker: {marker}"
        );
    }
}

#[test]
fn dev_feature_enables_unified_prototype_cutover() {
    let cargo_toml = include_str!("../Cargo.toml");

    assert!(
        cargo_toml.contains(DEV_FEATURE_MARKER),
        "dev feature must enable ux-prototypes for cutover launcher"
    );
}
