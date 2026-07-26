//! bd_app — Binary entry point for Broken Divinity Kernel.
//!
//! Initializes tracing, Bevy app with Bevy-Ratatui,
//! and spawns the Phase 1 minimal terminal slice.

use std::time::Duration;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use bevy_app::{AppExit, PanicHandlerPlugin, ScheduleRunnerPlugin};
use bevy_ecs::message::MessageWriter;
use bevy_ecs::schedule::IntoScheduleConfigs;
use bevy_ecs::system::ResMut;

use bd_core::gamelog::{GameLog, LogLevel};

mod config;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let validate_only = args.iter().any(|a| a == "--validate" || a == "-v");

    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "bd=info".into()),
        )
        .init();

    let result = if validate_only {
        run_validation()
    } else {
        run_application()
    };
    if let Err(error) = result {
        tracing::error!("Broken Divinity could not start: {error}");
        std::process::exit(1);
    }
}

fn run_application() -> Result<(), String> {
    tracing::info!("Broken Divinity Kernel starting");

    let data_dir = config::data_dir();
    let log_dir = data_dir.join("logs");
    if let Err(e) = std::fs::create_dir_all(&log_dir) {
        tracing::warn!("Failed to create log directory {log_dir:?}: {e}");
    }
    if let Err(e) = std::fs::create_dir_all(&data_dir) {
        tracing::warn!("Failed to create data directory {data_dir:?}: {e}");
    }

    let loaded = config::load_config().map_err(|error| format!("configuration error: {error}"))?;
    for warn in &loaded.warnings {
        tracing::warn!("{warn}");
    }
    tracing::info!(
        "Config source: {:?}",
        match &loaded.source {
            config::ConfigSource::Defaults => "built-in defaults",
            config::ConfigSource::File(p) => p.to_str().unwrap_or("unknown"),
        }
    );
    let save_dir = resolve_save_directory(&loaded.config, &data_dir);
    let command_bindings = loaded
        .config
        .keybindings
        .command_bindings()
        .map_err(|error| format!("configuration error: {error}"))?;
    let application_content = load_application_content(&content_dir())?;

    let frame_time = Duration::from_secs_f32(1.0 / 60.0);
    let mut app = bevy_app::App::new();
    app.add_plugins(ScheduleRunnerPlugin::run_loop(frame_time));
    app.add_plugins(PanicHandlerPlugin);
    app.add_plugins(bevy_ratatui::RatatuiPlugins::default());

    app.add_plugins(bd_core::BdFoundationPlugin);
    bd_data::loader::validate_runtime_action_links(&application_content.foundation, |action_id| {
        bd_core::foundation_action_is_registered(app.world(), action_id)
    })
    .map_err(|error| format!("content error: {error}"))?;
    app.insert_resource(bd_core::colony::stations::StationCatalog::new(
        application_content.foundation.stations.clone(),
    ));
    app.insert_resource(application_content.foundation);

    app.add_plugins(bd_tui::BdTuiPlugin);
    app.insert_resource(command_bindings);
    app.insert_resource(application_content.symbols);
    app.insert_resource(application_content.themes);
    app.insert_resource(ManualSaveDirectory(save_dir));

    configure_application_boundary_systems(&mut app);

    app.run();
    tracing::info!("Broken Divinity Kernel exited cleanly");
    Ok(())
}

#[derive(bevy_ecs::prelude::Resource)]
struct ManualSaveDirectory(std::path::PathBuf);

fn resolve_save_directory(config: &config::AppConfig, data_dir: &Path) -> PathBuf {
    config
        .save_dir_override
        .as_deref()
        .map(PathBuf::from)
        .unwrap_or_else(|| data_dir.join("saves"))
}

fn configure_application_boundary_systems(app: &mut bevy_app::App) {
    app.init_resource::<bd_tui::commands::ApplicationExitRequest>();
    app.add_message::<AppExit>();
    app.add_systems(
        bevy_app::Update,
        (process_persistence_requests, process_exit_request)
            .chain()
            .after(bd_core::BdSet::ResultEmission)
            .before(bd_core::BdSet::ViewModelBuild),
    );
}

fn process_exit_request(
    mut request: ResMut<bd_tui::commands::ApplicationExitRequest>,
    mut exits: MessageWriter<AppExit>,
) {
    if std::mem::take(&mut request.0) {
        exits.write(AppExit::Success);
    }
}

/// Execute persistence requests emitted by the TUI at the application
/// boundary. The kernel owns snapshot contents; the application owns paths
/// and user-facing success/failure reporting.
fn process_persistence_requests(world: &mut bevy_ecs::world::World) {
    let save_dir = world.resource::<ManualSaveDirectory>().0.clone();
    let save_requested = world.resource_mut::<bd_core::save::SaveRequest>().0;
    world.resource_mut::<bd_core::save::SaveRequest>().0 = false;
    if save_requested {
        match bd_core::save::save_manual_slot(world, &save_dir) {
            Ok(path) => world
                .resource_mut::<GameLog>()
                .push(format!("Game saved to {}.", path.display()), LogLevel::Info),
            Err(error) => {
                tracing::warn!("Manual save failed: {error}");
                world
                    .resource_mut::<GameLog>()
                    .push(player_facing_save_error(&error, false), LogLevel::Warn);
            }
        }
    }

    let load_requested = world.resource_mut::<bd_core::save::LoadRequest>().0;
    world.resource_mut::<bd_core::save::LoadRequest>().0 = false;
    if load_requested {
        let path = bd_core::save::manual_slot_path(&save_dir);
        match bd_core::save::load_manual_slot(&save_dir).and_then(|snapshot| {
            bd_core::save::restore_snapshot_into(world, &snapshot, &HashMap::new())
                .map(|_| snapshot)
        }) {
            Ok(snapshot) => {
                request_screen_for_restored_mode(world);
                world.resource_mut::<GameLog>().push(
                    format!(
                        "Loaded save from {} (turn {}).",
                        path.display(),
                        snapshot.turn
                    ),
                    LogLevel::Info,
                );
            }
            Err(error) => {
                tracing::warn!("Manual load failed: {error}");
                world
                    .resource_mut::<GameLog>()
                    .push(player_facing_save_error(&error, true), LogLevel::Warn);
            }
        }
    }
}

fn player_facing_save_error(error: &bd_core::save::SaveError, loading: bool) -> String {
    use bd_core::save::SaveError;
    match error {
        SaveError::Io(io) if loading && io.kind() == std::io::ErrorKind::NotFound => {
            "No manual save exists yet.".into()
        }
        SaveError::Corrupt(_) => "The manual save is corrupt and could not be loaded.".into(),
        SaveError::VersionMismatch { .. } | SaveError::ContentMismatch { .. } => {
            "The manual save is incompatible with this build.".into()
        }
        SaveError::MissingBlueprint(_) => {
            "The manual save references content unavailable in this build.".into()
        }
        SaveError::Io(_) if loading => "The manual save could not be accessed.".into(),
        SaveError::Io(_) => "The game could not write the manual save.".into(),
    }
}

fn request_screen_for_restored_mode(world: &mut bevy_ecs::world::World) {
    let mode = *world.resource::<bd_core::spatial::GameMode>();
    let screen_id = match mode {
        bd_core::spatial::GameMode::Title => "title",
        bd_core::spatial::GameMode::Outpost => "outpost",
        bd_core::spatial::GameMode::Tactical => "combat",
        bd_core::spatial::GameMode::GameOver => "game_over",
        bd_core::spatial::GameMode::Travel => return,
    };
    world
        .resource_mut::<bevy_ecs::message::Messages<bd_tui::screens::ScreenIntent>>()
        .write(bd_tui::screens::ScreenIntent {
            screen_id: screen_id.into(),
        });
}

#[derive(Debug)]
struct ApplicationContent {
    foundation: bd_core::content::FoundationContent,
    symbols: bd_tui::visual::SymbolRegistry,
    themes: bd_tui::theme::ThemeRegistry,
}

fn load_application_content(content_dir: &Path) -> Result<ApplicationContent, String> {
    use bd_data::loader::load_ron;

    let foundation = bd_data::loader::load_foundation_content(content_dir)
        .map_err(|error| format!("content error: {error}"))?;
    let sym_path = content_dir.join("symbols/default.ron");
    let symbol_file = load_ron::<bd_tui::visual::SymbolDef>(&sym_path)
        .map_err(|error| format!("content error: {error}"))?;
    let symbols = bd_tui::visual::SymbolRegistry::new(symbol_file.items);
    let symbol_errors = symbols.validate();
    if !symbol_errors.is_empty() {
        return Err(format!(
            "content error in {}: {}",
            sym_path.display(),
            symbol_errors.join("; ")
        ));
    }

    let theme_path = content_dir.join("themes/default.ron");
    let theme_file = load_ron::<bd_tui::theme::ThemeDef>(&theme_path)
        .map_err(|error| format!("content error: {error}"))?;
    let themes = bd_tui::theme::ThemeRegistry::from_defs(theme_file.items);
    let theme_errors = themes.validate();
    if !theme_errors.is_empty() {
        return Err(format!(
            "content error in {}: {}",
            theme_path.display(),
            theme_errors.join("; ")
        ));
    }

    Ok(ApplicationContent {
        foundation,
        symbols,
        themes,
    })
}

/// Run the same content validation used by normal startup without entering
/// terminal mode.
fn run_validation() -> Result<(), String> {
    let content_dir = content_dir();
    let content = load_application_content(&content_dir)?;
    let mut app = bevy_app::App::new();
    app.add_plugins(bd_core::BdFoundationPlugin);
    bd_data::loader::validate_runtime_action_links(&content.foundation, |action_id| {
        bd_core::foundation_action_is_registered(app.world(), action_id)
    })
    .map_err(|error| format!("content error: {error}"))?;
    tracing::info!(
        "Foundation: {} dungeons, {} items, {} skills, {} factions, {} actions, {} stations, {} blueprints",
        content.foundation.dungeons.len(),
        content.foundation.items.len(),
        content.foundation.skills.len(),
        content.foundation.factions.len(),
        content.foundation.actions.len(),
        content.foundation.stations.len(),
        content.foundation.blueprints.len()
    );
    tracing::info!("Content validation PASSED");
    Ok(())
}

fn content_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("content")
}

#[cfg(test)]
mod application_tests {
    use super::*;
    use bd_core::components::Player;
    use bd_core::session::RunOutcome;
    use bevy_ecs::entity::Entity;
    use bevy_ecs::message::Messages;
    use bevy_ecs::query::With;

    #[test]
    fn invalid_content_returns_readable_application_error() {
        let missing = std::env::temp_dir().join("bd-missing-foundation-content");
        let error = load_application_content(&missing).unwrap_err();
        assert!(error.contains("content error"));
        assert!(error.contains("dungeons/foundation.ron"));
    }

    #[test]
    fn persistence_errors_are_classified_without_internal_diagnostics() {
        use bd_core::save::SaveError;

        let cases = [
            (
                SaveError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "raw missing path",
                )),
                true,
                "No manual save exists yet.",
            ),
            (
                SaveError::Corrupt("raw parser detail".into()),
                true,
                "The manual save is corrupt and could not be loaded.",
            ),
            (
                SaveError::VersionMismatch {
                    expected: 2,
                    found: 1,
                },
                true,
                "The manual save is incompatible with this build.",
            ),
            (
                SaveError::MissingBlueprint("blueprint.raw".into()),
                true,
                "The manual save references content unavailable in this build.",
            ),
            (
                SaveError::Io(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "raw denied path",
                )),
                true,
                "The manual save could not be accessed.",
            ),
            (
                SaveError::Io(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "raw denied path",
                )),
                false,
                "The game could not write the manual save.",
            ),
        ];

        for (error, loading, expected) in cases {
            let message = player_facing_save_error(&error, loading);
            assert_eq!(message, expected);
            assert!(!message.contains("raw"));
        }
    }

    #[test]
    fn configured_save_directory_override_is_honored() {
        let default_data_dir = Path::new("/default/data");
        let mut config = config::AppConfig::default();
        assert_eq!(
            resolve_save_directory(&config, default_data_dir),
            default_data_dir.join("saves")
        );

        config.save_dir_override = Some("/explicit/foundation-saves".into());
        assert_eq!(
            resolve_save_directory(&config, default_data_dir),
            Path::new("/explicit/foundation-saves")
        );
    }

    #[test]
    fn restored_dungeon_requests_the_combat_screen() {
        let mut world = bevy_ecs::world::World::new();
        world.insert_resource(bd_core::spatial::GameMode::Tactical);
        world.insert_resource(Messages::<bd_tui::screens::ScreenIntent>::default());

        request_screen_for_restored_mode(&mut world);

        let intents = world
            .resource_mut::<Messages<bd_tui::screens::ScreenIntent>>()
            .drain()
            .collect::<Vec<_>>();
        assert_eq!(intents.len(), 1);
        assert_eq!(intents[0].screen_id, "combat");
    }

    #[test]
    fn persistence_runs_after_terminal_results_are_committed() {
        fn request_save(mut request: ResMut<bd_core::save::SaveRequest>) {
            request.0 = true;
        }

        fn commit_defeat(
            mut mode: ResMut<bd_core::spatial::GameMode>,
            mut session: ResMut<bd_core::session::RunSession>,
        ) {
            *mode = bd_core::spatial::GameMode::GameOver;
            session.mark_defeated();
        }

        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let save_dir = std::env::temp_dir().join(format!(
            "bd-app-result-order-{}-{unique}",
            std::process::id()
        ));

        let mut app = bevy_app::App::new();
        app.add_plugins(bd_core::BdFoundationPlugin);
        app.insert_resource(ManualSaveDirectory(save_dir.clone()));
        configure_application_boundary_systems(&mut app);
        app.add_systems(
            bevy_app::Update,
            request_save.in_set(bd_core::BdSet::Mutation),
        );
        app.add_systems(
            bevy_app::Update,
            commit_defeat.in_set(bd_core::BdSet::ResultEmission),
        );

        app.update();

        let snapshot = bd_core::save::load_manual_slot(&save_dir).unwrap();
        assert_eq!(snapshot.session.outcome, RunOutcome::Defeated);
        assert_eq!(snapshot.session.phase, bd_core::spatial::GameMode::GameOver);
    }

    #[test]
    fn application_startup_has_exactly_one_player_authority() {
        let content = load_application_content(&content_dir()).unwrap();
        let mut app = bevy_app::App::new();
        app.add_plugins(bd_core::BdFoundationPlugin);
        app.insert_resource(content.foundation);
        *app.world_mut().resource_mut::<bd_core::spatial::GameMode>() =
            bd_core::spatial::GameMode::Outpost;
        app.world_mut()
            .resource_mut::<bd_core::session::RunSession>()
            .phase = bd_core::spatial::GameMode::Outpost;

        app.update();
        app.update();

        let player_count = app
            .world_mut()
            .query_filtered::<Entity, With<Player>>()
            .iter(app.world())
            .count();
        assert_eq!(player_count, 1);
    }
}
