use bd_core::spatial::GameMode;
use bd_tui::commands::{
    ActionAvailability, CommandBindings, GameOverInput, InteractionMode, TerminalLayout,
    TitleInput, UiCommand, action_panel, footer_control_lines, footer_text, game_over_input,
    help_entries, inventory_toggle_destination, terminal_layout, title_input,
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
fn compact_footer_keeps_complete_context_and_global_tokens() {
    let lines = footer_control_lines(
        &CommandBindings::default(),
        GameMode::Outpost,
        InteractionMode::Normal,
        "outpost",
        60,
    );

    assert!(lines.contextual.len() <= 60);
    assert!(lines.global.len() <= 60);
    for required in ["Travel:t", "Move:wasd/arrows"] {
        assert!(lines.contextual.contains(required), "missing {required}");
    }
    for required in ["Help:?", "Save:F5", "Load:F9", "Quit:q"] {
        assert!(lines.global.contains(required), "missing {required}");
    }
}

#[test]
fn inventory_footer_labels_the_toggle_as_back() {
    let lines = footer_control_lines(
        &CommandBindings::default(),
        GameMode::Outpost,
        InteractionMode::Normal,
        "inventory",
        60,
    );

    for required in ["Back:i", "Use:u", "Save:F5", "Load:F9", "Quit:q"] {
        assert!(lines.global.contains(required), "missing {required}");
    }
    assert!(!lines.global.contains("Inventory:i"));
}

#[test]
fn inventory_toggle_returns_to_the_active_mode_screen() {
    assert_eq!(
        inventory_toggle_destination("outpost", GameMode::Outpost),
        "inventory"
    );
    assert_eq!(
        inventory_toggle_destination("inventory", GameMode::Outpost),
        "outpost"
    );
    assert_eq!(
        inventory_toggle_destination("inventory", GameMode::Tactical),
        "combat"
    );
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
fn extraction_context_is_visible_and_truthful() {
    let away_from_exit = action_panel(
        &CommandBindings::default(),
        ActionAvailability::dungeon(true, false, true, false, false).at_exit(false),
    );
    assert!(
        away_from_exit
            .iter()
            .all(|action| action.command != UiCommand::Extract),
        "Extract should only occupy the action panel when it is valid"
    );

    let at_exit = action_panel(
        &CommandBindings::default(),
        ActionAvailability::dungeon(true, false, true, false, false).at_exit(true),
    );
    let extract = at_exit
        .iter()
        .find(|action| action.command == UiCommand::Extract)
        .expect("the explicit extraction action must remain discoverable");
    assert!(extract.enabled);
    assert_eq!(extract.denial_reason, None);

    let footer = footer_control_lines(
        &CommandBindings::default(),
        GameMode::Tactical,
        InteractionMode::Normal,
        "combat",
        60,
    );
    assert!(
        !footer.contextual.contains("Extract:"),
        "location-dependent extraction belongs in the truthful action panel"
    );
}

#[test]
fn insufficient_travel_supplies_are_explained_before_input() {
    let actions = action_panel(
        &CommandBindings::default(),
        ActionAvailability::outpost(true, true, true, true, true).can_travel(false),
    );
    let travel = actions
        .iter()
        .find(|action| action.command == UiCommand::Travel)
        .expect("travel remains discoverable when unaffordable");

    assert!(!travel.enabled);
    assert_eq!(travel.denial_reason.as_deref(), Some("Need 2 Supplies"));
}

#[test]
fn shelter_travel_action_remains_discoverable() {
    let actions = action_panel(
        &CommandBindings::default(),
        ActionAvailability::outpost(true, true, true, true, true),
    );
    let travel = actions
        .iter()
        .find(|action| action.command == UiCommand::Travel)
        .expect("the shelter must expose the Foundation dungeon entry action");

    assert!(travel.enabled);
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
            .any(|entry| entry.command == UiCommand::RestUntilNextDay)
    );
    assert!(
        help.iter()
            .any(|entry| entry.command == UiCommand::AssignTask)
    );
    assert!(!help.iter().any(|entry| entry.command == UiCommand::Attack));
    assert!(!help.iter().any(|entry| entry.command == UiCommand::Extract));
}

#[test]
fn rest_guidance_is_outpost_only_and_names_the_exact_boundary() {
    let actions = action_panel(
        &CommandBindings::default(),
        ActionAvailability::outpost(true, true, true, true, true).time(3, 7),
    );
    let rest = actions
        .iter()
        .find(|action| action.command == UiCommand::RestUntilNextDay)
        .expect("Rest must be discoverable in the shelter");
    assert_eq!(rest.key, "n");
    assert_eq!(rest.label, "Rest to Day 4 (17 turns)");

    for (mode, interaction) in [
        (GameMode::Tactical, InteractionMode::Normal),
        (GameMode::Outpost, InteractionMode::Build),
        (GameMode::GameOver, InteractionMode::GameOver),
    ] {
        assert!(
            help_entries(&CommandBindings::default(), mode, interaction)
                .iter()
                .all(|entry| entry.command != UiCommand::RestUntilNextDay),
            "Rest must be unavailable in {mode:?}/{interaction:?}"
        );
    }
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
fn foundation_controls_do_not_advertise_redundant_z_command() {
    let bindings = CommandBindings::default();
    assert_eq!(
        bindings.command_for_key(&KeyCode::Char('z')),
        None,
        "Z has no distinct Foundation behavior and must not be bound"
    );
    for mode in [GameMode::Outpost, GameMode::Tactical, GameMode::GameOver] {
        let interaction = if mode == GameMode::GameOver {
            InteractionMode::GameOver
        } else {
            InteractionMode::Normal
        };
        assert!(
            help_entries(&bindings, mode, interaction)
                .iter()
                .all(|entry| entry.key != "z"),
            "{mode:?} help must not advertise the redundant screen command"
        );
    }
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
fn title_load_binding_requests_load_instead_of_starting_a_new_run() {
    let bindings = CommandBindings::default();

    assert_eq!(title_input(&bindings, &KeyCode::F(9)), TitleInput::Load);
    assert_eq!(
        title_input(&bindings, &KeyCode::Char('x')),
        TitleInput::Begin
    );
}

#[test]
fn title_footer_advertises_only_actions_that_work_on_title() {
    let footer = footer_text(
        &CommandBindings::default(),
        GameMode::Title,
        InteractionMode::Normal,
    );

    assert!(footer.contains("Load:F9"));
    assert!(footer.contains("Quit:q"));
    assert!(!footer.contains("Save:"));
    assert!(!footer.contains("Help:"));
}

#[test]
fn game_over_save_and_load_bindings_request_persistence_actions() {
    let bindings = CommandBindings::default();

    assert_eq!(
        game_over_input(&bindings, &KeyCode::F(5)),
        Some(GameOverInput::Save)
    );
    assert_eq!(
        game_over_input(&bindings, &KeyCode::F(9)),
        Some(GameOverInput::Load)
    );
}

#[test]
fn game_over_footer_advertises_persistence_and_exit_controls() {
    let footer = footer_text(
        &CommandBindings::default(),
        GameMode::GameOver,
        InteractionMode::GameOver,
    );

    for required in ["Restart:", "Save:F5", "Load:F9", "Quit:q"] {
        assert!(footer.contains(required), "missing {required} in {footer}");
    }
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
