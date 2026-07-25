use bd_core::spatial::GameMode;
use bd_tui::commands::{
    ActionAvailability, CommandBindings, InteractionMode, TerminalLayout, UiCommand, action_panel,
    footer_text, help_entries, terminal_layout,
};
use crossterm::event::KeyCode;

#[test]
fn configured_binding_emits_expected_command() {
    let mut bindings = CommandBindings::default();
    bindings.bind(UiCommand::Attack, KeyCode::Char('x'));

    assert_eq!(
        bindings.command_for_key(&KeyCode::Char('x')),
        Some(UiCommand::Attack)
    );
    assert_ne!(
        bindings.command_for_key(&KeyCode::Char('f')),
        Some(UiCommand::Attack)
    );
}

#[test]
fn help_uses_configured_binding() {
    let mut bindings = CommandBindings::default();
    bindings.bind(UiCommand::Attack, KeyCode::Char('x'));

    let help = help_entries(&bindings, GameMode::Tactical, InteractionMode::Normal);
    assert!(
        help.iter()
            .any(|entry| entry.key == "x" && entry.label == "Attack")
    );
    assert!(
        !help
            .iter()
            .any(|entry| entry.key == "f" && entry.label == "Attack")
    );
}

#[test]
fn footer_uses_configured_binding() {
    let mut bindings = CommandBindings::default();
    bindings.bind(UiCommand::Attack, KeyCode::Char('x'));

    let footer = footer_text(&bindings, GameMode::Tactical, InteractionMode::Normal);
    assert!(footer.contains("Attack:x"));
    assert!(!footer.contains("Attack:f"));
}

#[test]
fn action_panel_uses_configured_binding() {
    let mut bindings = CommandBindings::default();
    bindings.bind(UiCommand::Attack, KeyCode::Char('x'));
    let availability = ActionAvailability::dungeon(true, true, true, false, false);

    let actions = action_panel(&bindings, availability);
    let attack = actions
        .iter()
        .find(|action| action.command == UiCommand::Attack)
        .unwrap();
    assert_eq!(attack.key, "x");
}

#[test]
fn colony_help_lists_only_colony_actions() {
    let help = help_entries(
        &CommandBindings::default(),
        GameMode::Outpost,
        InteractionMode::Normal,
    );

    assert!(help.iter().any(|entry| entry.command == UiCommand::Build));
    assert!(
        help.iter()
            .any(|entry| entry.command == UiCommand::AssignTask)
    );
    assert!(!help.iter().any(|entry| entry.command == UiCommand::Attack));
    assert!(!help.iter().any(|entry| entry.command == UiCommand::Extract));
}

#[test]
fn dungeon_help_lists_only_dungeon_actions() {
    let help = help_entries(
        &CommandBindings::default(),
        GameMode::Tactical,
        InteractionMode::Normal,
    );

    assert!(help.iter().any(|entry| entry.command == UiCommand::Attack));
    assert!(help.iter().any(|entry| entry.command == UiCommand::Extract));
    assert!(!help.iter().any(|entry| entry.command == UiCommand::Build));
    assert!(
        !help
            .iter()
            .any(|entry| entry.command == UiCommand::AssignTask)
    );
}

#[test]
fn save_and_load_are_discoverable() {
    let help = help_entries(
        &CommandBindings::default(),
        GameMode::Outpost,
        InteractionMode::Normal,
    );

    assert!(help.iter().any(|entry| entry.command == UiCommand::Save));
    assert!(help.iter().any(|entry| entry.command == UiCommand::Load));
}

#[test]
fn no_target_attack_displays_denial() {
    let actions = action_panel(
        &CommandBindings::default(),
        ActionAvailability::dungeon(true, false, true, false, false),
    );
    let attack = actions
        .iter()
        .find(|action| action.command == UiCommand::Attack)
        .unwrap();

    assert!(!attack.enabled);
    assert_eq!(attack.denial_reason.as_deref(), Some("No target in range"));
}

#[test]
fn actions_panel_matches_kernel_availability() {
    let actions = action_panel(
        &CommandBindings::default(),
        ActionAvailability::dungeon(false, true, false, true, true),
    );
    let move_action = actions
        .iter()
        .find(|action| action.command == UiCommand::MoveNorth)
        .unwrap();
    let attack = actions
        .iter()
        .find(|action| action.command == UiCommand::Attack)
        .unwrap();
    let pickup = actions
        .iter()
        .find(|action| action.command == UiCommand::Pickup)
        .unwrap();
    let use_item = actions
        .iter()
        .find(|action| action.command == UiCommand::UseItem)
        .unwrap();

    assert!(!move_action.enabled);
    assert_eq!(move_action.denial_reason.as_deref(), Some("No AP"));
    assert!(!attack.enabled);
    assert_eq!(attack.denial_reason.as_deref(), Some("No AP"));
    assert!(pickup.enabled);
    assert!(use_item.enabled);
}

#[test]
fn minimum_terminal_layout_preserves_required_controls() {
    assert_eq!(terminal_layout(80, 24), TerminalLayout::Full);
    assert_eq!(terminal_layout(60, 20), TerminalLayout::Compact);
    assert_eq!(terminal_layout(59, 20), TerminalLayout::TooSmall);

    let footer = footer_text(
        &CommandBindings::default(),
        GameMode::Outpost,
        InteractionMode::Normal,
    );
    for required in ["Help:", "Save:", "Load:", "Quit:"] {
        assert!(footer.contains(required), "missing {required} from footer");
    }
}
