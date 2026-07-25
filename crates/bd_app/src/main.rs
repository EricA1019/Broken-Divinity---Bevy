//! bd_app — Binary entry point for Broken Divinity Kernel.
//!
//! Initializes tracing, Bevy app with Bevy-Ratatui,
//! and spawns the Phase 1 minimal terminal slice.

use std::time::Duration;
use std::{collections::HashMap, path::Path};

use bevy_app::{AppExit, PanicHandlerPlugin, ScheduleRunnerPlugin};
use bevy_ecs::message::MessageWriter;
use bevy_ecs::schedule::IntoScheduleConfigs;
use bevy_ecs::system::{Commands, Query, Res, ResMut};

use bd_core::components::{Player, Position};
use bd_core::factory::spawn_from_blueprint;
use bd_core::gamelog::{GameLog, LogLevel};
use bevy_ecs::entity::Entity;
use bevy_ecs::query::With;

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
    app.insert_resource(application_content.foundation);

    app.add_plugins(bd_tui::BdTuiPlugin);
    app.insert_resource(command_bindings);
    app.insert_resource(application_content.symbols);
    app.insert_resource(application_content.themes);

    app.add_systems(
        bevy_app::Update,
        spawn_outpost_player.in_set(bd_core::BdSet::IntentCollection),
    );
    app.add_systems(
        bevy_app::Update,
        (process_persistence_requests, process_exit_request)
            .chain()
            .in_set(bd_core::BdSet::ResultEmission),
    );

    app.run();
    tracing::info!("Broken Divinity Kernel exited cleanly");
    Ok(())
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
    let save_requested = world.resource_mut::<bd_core::save::SaveRequest>().0;
    world.resource_mut::<bd_core::save::SaveRequest>().0 = false;
    if save_requested {
        let save_dir = config::data_dir().join("saves");
        match bd_core::save::save_manual_slot(world, &save_dir) {
            Ok(path) => world
                .resource_mut::<GameLog>()
                .push(format!("Game saved to {}.", path.display()), LogLevel::Info),
            Err(error) => world
                .resource_mut::<GameLog>()
                .push(format!("Save failed: {error}"), LogLevel::Warn),
        }
    }

    let load_requested = world.resource_mut::<bd_core::save::LoadRequest>().0;
    world.resource_mut::<bd_core::save::LoadRequest>().0 = false;
    if load_requested {
        let save_dir = config::data_dir().join("saves");
        let path = bd_core::save::manual_slot_path(&save_dir);
        match bd_core::save::load_manual_slot(&save_dir).and_then(|snapshot| {
            bd_core::save::restore_snapshot_into(world, &snapshot, &HashMap::new())
                .map(|_| snapshot)
        }) {
            Ok(snapshot) => world.resource_mut::<GameLog>().push(
                format!(
                    "Loaded save from {} (turn {}).",
                    path.display(),
                    snapshot.turn
                ),
                LogLevel::Info,
            ),
            Err(error) => world
                .resource_mut::<GameLog>()
                .push(format!("Load failed: {error}"), LogLevel::Warn),
        }
    }
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

/// Spawn the player at the shelter outpost at startup.
fn spawn_outpost_player(
    mut commands: Commands,
    mut game_log: ResMut<GameLog>,
    mode: Res<bd_core::spatial::GameMode>,
    content: Res<bd_core::content::FoundationContent>,
    player: Query<Entity, With<Player>>,
    bindings: Res<bd_tui::commands::CommandBindings>,
) {
    // Only spawn when the game is in Outpost mode (not Title)
    if *mode != bd_core::spatial::GameMode::Outpost {
        return;
    }
    // Once-only guard — player already spawned
    if !player.is_empty() {
        return;
    }

    // Spawn player in the center of the shelter
    let player_pos = Position {
        x: bd_core::colony::shelter::SHELTER_WIDTH / 2,
        y: bd_core::colony::shelter::SHELTER_HEIGHT / 2,
    };

    if let Some(bp) = content
        .blueprints
        .iter()
        .find(|bp| bp.id == "blueprint.player")
    {
        let entity = spawn_from_blueprint(bp, Some(player_pos), &[], &mut commands);
        commands.entity(entity).insert((
            bd_core::spatial::PersistentEntity,
            bd_core::inventory::Container::default(),
        ));
    }

    // Grant initial colony supplies via ColonyResources
    game_log.push(
        "You survey the shelter. Survivors are gathering.",
        LogLevel::Info,
    );
    game_log.push(
        bd_tui::commands::footer_text(
            &bindings,
            bd_core::spatial::GameMode::Outpost,
            bd_tui::commands::InteractionMode::Normal,
        ),
        LogLevel::Info,
    );
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
        "Foundation: {} dungeons, {} items, {} skills, {} factions, {} actions, {} blueprints",
        content.foundation.dungeons.len(),
        content.foundation.items.len(),
        content.foundation.skills.len(),
        content.foundation.factions.len(),
        content.foundation.actions.len(),
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

    #[test]
    fn invalid_content_returns_readable_application_error() {
        let missing = std::env::temp_dir().join("bd-missing-foundation-content");
        let error = load_application_content(&missing).unwrap_err();
        assert!(error.contains("content error"));
        assert!(error.contains("dungeons/foundation.ron"));
    }
}
