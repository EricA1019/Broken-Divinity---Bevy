use bevy::prelude::*;
use bevy_ecs_tilemap::TilemapPlugin;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
use std::env;
use broken_divinity::core::state::AppState;
use broken_divinity::core::turn::TurnPhase;

const HEADLESS_FLAG: &str = "--headless";
const QA_STANDARD_FLAG: &str = "--qa-standard";
const QA_DEEP_DIAGNOSTICS_FLAG: &str = "--qa-deep-diagnostics";
const DEFAULT_LOG_FILTER: &str = "info";
const QA_STANDARD_LOG_FILTER: &str = "warn,broken_divinity=info";
const QA_DEEP_DIAGNOSTICS_LOG_FILTER: &str = "info,broken_divinity=debug";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LogProfile {
    Default,
    QaStandard,
    QaDeepDiagnostics,
}

fn detect_log_profile(args: &[String]) -> LogProfile {
    if args.iter().any(|arg| arg == QA_DEEP_DIAGNOSTICS_FLAG) {
        return LogProfile::QaDeepDiagnostics;
    }

    if args.iter().any(|arg| arg == QA_STANDARD_FLAG) {
        return LogProfile::QaStandard;
    }

    LogProfile::Default
}

fn log_filter_for_profile(profile: LogProfile) -> &'static str {
    match profile {
        LogProfile::Default => DEFAULT_LOG_FILTER,
        LogProfile::QaStandard => QA_STANDARD_LOG_FILTER,
        LogProfile::QaDeepDiagnostics => QA_DEEP_DIAGNOSTICS_LOG_FILTER,
    }
}

fn log_level_for_profile(profile: LogProfile) -> bevy::log::Level {
    match profile {
        LogProfile::QaDeepDiagnostics => bevy::log::Level::DEBUG,
        LogProfile::Default | LogProfile::QaStandard => bevy::log::Level::INFO,
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let headless_mode = args.iter().any(|arg| arg == HEADLESS_FLAG);
    let log_profile = detect_log_profile(&args);
    
    let mut app = App::new();
    
    // Only add window plugins if not in headless mode
    let base_plugins = DefaultPlugins.set(bevy::log::LogPlugin {
        filter: log_filter_for_profile(log_profile).to_string(),
        level: log_level_for_profile(log_profile),
        ..default()
    });

    if !headless_mode {
        app.add_plugins(base_plugins);
    } else {
        // Add essential plugins for headless mode (no window)
        app.add_plugins(base_plugins.set(ImagePlugin::default_nearest()).set(WindowPlugin {
            primary_window: None,
            ..default()
        }));
    }
    
    app.add_plugins(TilemapPlugin)
        .add_plugins(EguiPlugin::default())
        .add_plugins(broken_divinity::core::plugin)
        .add_plugins(broken_divinity::game::colony::plugin)
        .add_plugins(broken_divinity::game::combat::plugin)
        .add_plugins(broken_divinity::game::dungeon::plugin)
        .add_plugins(broken_divinity::game::overworld::plugin)
        .init_state::<AppState>()
        // --- State wrapper registrations for BRP visibility ---
        .register_type::<State<AppState>>()
        .register_type::<NextState<AppState>>()
        .register_type::<State<TurnPhase>>()
        .register_type::<NextState<TurnPhase>>()
        // --- Action resources ---
        .init_resource::<broken_divinity::ui::gabriel_dialogue_panel::GabrielDialogueUiAction>()
        .init_resource::<broken_divinity::ui::help_panel::HelpOpen>()
        .init_resource::<broken_divinity::ui::inventory_panel::InventoryOpen>()
        .init_resource::<broken_divinity::ui::inventory_panel::InventoryUiAction>()
        .init_resource::<broken_divinity::ui::journal_panel::JournalOpen>()
        .init_resource::<broken_divinity::ui::menu::MenuUiAction>()
        .init_resource::<broken_divinity::ui::modal_priority::ModalBlockers>()
        .init_resource::<broken_divinity::ui::modal_priority::ModalPriorityCoordinator>()
        .init_resource::<broken_divinity::ui::objective_prompt::ColonyObjectivePromptState>()
        .init_resource::<broken_divinity::ui::overworld_panel::OverworldUiAction>()
        .init_resource::<broken_divinity::ui::colony_panel::ColonyUiAction>()
        .init_resource::<broken_divinity::game::colony::raids::RaidUiAction>()
        .init_resource::<broken_divinity::ui::gameover::GameOverUiAction>()
        .init_resource::<broken_divinity::ui::perk_choice_panel::PerkChoiceUiAction>()
        // --- State transitions ---
        .add_systems(
            OnEnter(AppState::Menu),
            broken_divinity::core::save::reset_run_state_for_menu,
        )
        .add_systems(
            OnEnter(AppState::Colony),
            broken_divinity::core::save::autosave
                .after(broken_divinity::game::colony::raids::deliver_pending_raid_report),
        )
        // --- Draw systems — EguiPrimaryContextPass (read-only) ---
        .add_systems(
            EguiPrimaryContextPass,
            broken_divinity::ui::menu::draw_main_menu.run_if(in_state(AppState::Menu)),
        )
        .add_systems(
            EguiPrimaryContextPass,
            broken_divinity::ui::overworld_panel::draw_overworld_panel
                .run_if(in_state(AppState::Overworld)),
        )
        .add_systems(
            EguiPrimaryContextPass,
            broken_divinity::ui::gabriel_dialogue_panel::draw_gabriel_dialogue_panel
                .run_if(in_state(AppState::Dungeon)),
        )
        .add_systems(
            EguiPrimaryContextPass,
            broken_divinity::ui::gamelog_panel::draw_gamelog_panel,
        )
        .add_systems(
            EguiPrimaryContextPass,
            broken_divinity::ui::help_panel::draw_help_panel,
        )
        .add_systems(
            EguiPrimaryContextPass,
            broken_divinity::ui::gameover::draw_gameover_screen,
        )
        .add_systems(
            EguiPrimaryContextPass,
            broken_divinity::ui::hud::draw_hud
                .run_if(in_state(AppState::Dungeon).or(in_state(AppState::Colony))),
        )
        .add_systems(
            EguiPrimaryContextPass,
            broken_divinity::ui::perk_choice_panel::draw_perk_choice_panel
                .run_if(in_state(AppState::Dungeon).or(in_state(AppState::Colony))),
        )
        .add_systems(
            EguiPrimaryContextPass,
            broken_divinity::ui::inventory_panel::draw_inventory_panel
                .run_if(in_state(AppState::Dungeon).or(in_state(AppState::Colony))),
        )
        .add_systems(
            EguiPrimaryContextPass,
            broken_divinity::ui::journal_panel::draw_journal_panel
                .run_if(in_state(AppState::Dungeon)),
        )
        .add_systems(
            EguiPrimaryContextPass,
            (
                broken_divinity::ui::colony_panel::draw_resource_bar,
                broken_divinity::ui::colony_panel::draw_survivor_panel,
                broken_divinity::ui::colony_panel::draw_build_panel,
                broken_divinity::ui::colony_panel::draw_research_panel,
                broken_divinity::game::colony::raids::draw_raid_modal,
            )
                .run_if(in_state(AppState::Colony)),
        )
        // --- Process systems — Update (mutations) ---
        .add_systems(
            Update,
            broken_divinity::ui::menu::process_menu_action.run_if(in_state(AppState::Menu)),
        )
        .add_systems(
            Update,
            broken_divinity::ui::overworld_panel::process_overworld_action
                .run_if(in_state(AppState::Overworld)),
        )
        .add_systems(
            Update,
            broken_divinity::ui::gabriel_dialogue_panel::process_gabriel_dialogue_action
                .run_if(in_state(AppState::Dungeon)),
        )
        .add_systems(
            Update,
            broken_divinity::ui::gameover::process_gameover_action,
        )
        .add_systems(
            Update,
            broken_divinity::ui::perk_choice_panel::process_perk_choice_action
                .run_if(in_state(AppState::Dungeon).or(in_state(AppState::Colony))),
        )
        .add_systems(
            Update,
            broken_divinity::ui::inventory_panel::process_inventory_action
                .run_if(in_state(AppState::Dungeon).or(in_state(AppState::Colony))),
        )
        .add_systems(
            Update,
            (
                broken_divinity::ui::colony_panel::process_colony_action,
                broken_divinity::game::colony::raids::process_raid_action,
            )
                .run_if(in_state(AppState::Colony)),
        )
        // --- Input handlers — Update ---
        .add_systems(
            Update,
            broken_divinity::ui::modal_priority::apply_modal_priority_policy
                .run_if(in_state(AppState::Colony))
                .before(broken_divinity::ui::help_panel::toggle_help)
                .before(broken_divinity::core::escape::handle_escape_to_menu),
        )
        .add_systems(Update, broken_divinity::ui::help_panel::toggle_help)
        .add_systems(
            Update,
            broken_divinity::ui::objective_prompt::refresh_colony_objective_prompt
                .run_if(in_state(AppState::Colony).or(in_state(AppState::Overworld))),
        )
        .add_systems(
            Update,
            broken_divinity::ui::inventory_panel::toggle_inventory
                .run_if(in_state(AppState::Dungeon).or(in_state(AppState::Colony))),
        )
        .add_systems(
            Update,
            broken_divinity::ui::journal_panel::toggle_journal.run_if(in_state(AppState::Dungeon)),
        )
        .add_systems(
            Update,
            broken_divinity::game::overworld::travel::enter_overworld_from_colony.run_if(in_state(AppState::Colony)),
        )
        .add_systems(
            Update,
            broken_divinity::core::save::handle_save_and_quit
                .run_if(in_state(AppState::Colony).or(in_state(AppState::Overworld))),
        );

    // BRP — Bevy Remote Protocol for live state inspection (dev builds only)
    #[cfg(feature = "dev")]
    {
        app.add_plugins(bevy::remote::RemotePlugin::default());
        app.add_plugins(bevy::remote::http::RemoteHttpPlugin::default());
        app.add_plugins(bevy_brp_extras::BrpExtrasPlugin);
    }

    app.run();
}

#[cfg(test)]
mod tests {
    use super::{detect_log_profile, log_filter_for_profile, LogProfile};

    #[test]
    fn detect_log_profile_defaults_when_no_flags() {
        let args = vec!["broken_divinity".to_string()];
        assert_eq!(detect_log_profile(&args), LogProfile::Default);
    }

    #[test]
    fn detect_log_profile_qa_standard_flag() {
        let args = vec!["broken_divinity".to_string(), "--qa-standard".to_string()];
        assert_eq!(detect_log_profile(&args), LogProfile::QaStandard);
    }

    #[test]
    fn detect_log_profile_qa_deep_flag() {
        let args = vec![
            "broken_divinity".to_string(),
            "--qa-deep-diagnostics".to_string(),
        ];
        assert_eq!(detect_log_profile(&args), LogProfile::QaDeepDiagnostics);
    }

    #[test]
    fn qa_standard_filter_reduces_noise_and_keeps_game_logs() {
        assert_eq!(
            log_filter_for_profile(LogProfile::QaStandard),
            "warn,broken_divinity=info"
        );
    }

    #[test]
    fn qa_deep_filter_enables_detailed_debugging() {
        assert_eq!(
            log_filter_for_profile(LogProfile::QaDeepDiagnostics),
            "info,broken_divinity=debug"
        );
    }
}
