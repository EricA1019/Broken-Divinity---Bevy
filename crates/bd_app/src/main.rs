//! bd_app — Binary entry point for Broken Divinity Kernel.
//!
//! Initializes tracing, Bevy app with Bevy-Ratatui,
//! and spawns the Phase 1 minimal terminal slice.

use std::path::Path;
use std::time::Duration;

use bevy_app::{PanicHandlerPlugin, ScheduleRunnerPlugin, Startup};
use bevy_ecs::system::{Commands, ResMut};

use bd_core::components::{BlocksMovement, ExitTile, Name, Position};
use bd_core::factory::{BlueprintRegistry, spawn_from_blueprint};
use bd_core::gamelog::{GameLog, LogLevel};
use bd_core::map::SmokeMap;
use bd_core::procgen::{LocationTemplate, generate_location};
use bd_core::HelpLine;

mod config;

fn main() {
    // Parse CLI args
    let args: Vec<String> = std::env::args().collect();
    let validate_only = args.iter().any(|a| a == "--validate" || a == "-v");

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "bd=info".into()),
        )
        .init();

    if validate_only {
        run_validation();
        return;
    }

    tracing::info!("Broken Divinity Kernel starting");

    // Create data and log directories
    let data_dir = config::data_dir();
    let log_dir = data_dir.join("logs");
    if let Err(e) = std::fs::create_dir_all(&log_dir) {
        tracing::warn!("Failed to create log directory {log_dir:?}: {e}");
    }
    if let Err(e) = std::fs::create_dir_all(&data_dir) {
        tracing::warn!("Failed to create data directory {data_dir:?}: {e}");
    }

    // Load config
    let loaded = config::load_config();
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

    let frame_time = Duration::from_secs_f32(1.0 / 60.0);

    let mut app = bevy_app::App::new();

    app.add_plugins(ScheduleRunnerPlugin::run_loop(frame_time));
    app.add_plugins(PanicHandlerPlugin);
    app.add_plugins(bevy_ratatui::RatatuiPlugins::default());

    // Core + TUI plugins (register default registries)
    app.add_plugins(bd_core::BdCorePlugin);
    app.add_plugins(bd_tui::BdTuiPlugin);

    // Register spatial/transition module
    bd_core::spatial::register_spatial(&mut app);

    // Override HelpLine with config-derived value
    let help_line = HelpLine(loaded.config.keybindings.help_line());
    app.insert_resource(help_line);

    // Override defaults with RON content at startup
    app.add_systems(Startup, apply_ron_content);

    // Spawn player in outpost at startup (no dungeon yet)
    app.add_systems(Startup, spawn_outpost_player);

    app.run();
    tracing::info!("Broken Divinity Kernel exited cleanly");
}

/// Load RON symbol/theme files and override the registries.
fn apply_ron_content(
    mut symbols: bevy_ecs::system::ResMut<bd_tui::visual::SymbolRegistry>,
    mut themes: bevy_ecs::system::ResMut<bd_tui::theme::ThemeRegistry>,
) {
    use bd_data::loader::load_ron;

    let content_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("content");

    // Load symbols
    let sym_path = content_dir.join("symbols/default.ron");
    if sym_path.exists() {
        match load_ron::<bd_tui::visual::SymbolDef>(&sym_path) {
            Ok(file) => {
                tracing::info!("Loaded {} symbols from {}", file.items.len(), file.path);
                *symbols = bd_tui::visual::SymbolRegistry::new(file.items);
            }
            Err(e) => tracing::warn!("Failed to load symbols: {e}"),
        }
    }

    // Load themes
    let theme_path = content_dir.join("themes/default.ron");
    if theme_path.exists() {
        match load_ron::<bd_tui::theme::ThemeDef>(&theme_path) {
            Ok(file) => {
                tracing::info!("Loaded {} themes from {}", file.items.len(), file.path);
                *themes = bd_tui::theme::ThemeRegistry::from_defs(file.items);
            }
            Err(e) => tracing::warn!("Failed to load themes: {e}"),
        }
    }
}

/// Spawn the player at the shelter outpost at startup.
fn spawn_outpost_player(
    mut commands: Commands,
    mut game_log: ResMut<GameLog>,
) {
    let registry = BlueprintRegistry::phase18_defaults();

    // Spawn player in the center of the shelter
    let player_pos = Position {
        x: bd_core::colony::shelter::SHELTER_WIDTH / 2,
        y: bd_core::colony::shelter::SHELTER_HEIGHT / 2,
    };

    if let Some(bp) = registry.get("blueprint.player") {
        let entity = spawn_from_blueprint(bp, Some(player_pos), &[], &mut commands);
        commands.entity(entity).insert(bd_core::spatial::PersistentEntity);
    }

    // Grant initial colony supplies via ColonyResources
    game_log.push("You survey the shelter. Survivors are gathering.", LogLevel::Info);
    game_log.push("b: build | t: travel | i: inventory", LogLevel::Info);

    game_log.push("You survey the shelter. Survivors are gathering.", LogLevel::Info);
    game_log.push("b: build | a: assign | t: travel | i: inventory", LogLevel::Info);
}


/// Run content validation and exit.
fn run_validation() {
    use bd_data::loader::load_ron;

    let content_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("content");
    let mut errors = 0;

    // Validate symbols
    let sym_path = content_dir.join("symbols/default.ron");
    if sym_path.exists() {
        match load_ron::<bd_tui::visual::SymbolDef>(&sym_path) {
            Ok(file) => {
                tracing::info!("Symbols: {} items loaded", file.items.len());
            }
            Err(e) => {
                tracing::error!("Symbol validation: {e}");
                errors += 1;
            }
        }
    } else {
        tracing::warn!("No symbols file found at {}", sym_path.display());
    }

    // Validate themes
    let theme_path = content_dir.join("themes/default.ron");
    if theme_path.exists() {
        match load_ron::<bd_tui::theme::ThemeDef>(&theme_path) {
            Ok(file) => {
                tracing::info!("Themes: {} items loaded", file.items.len());
            }
            Err(e) => {
                tracing::error!("Theme validation: {e}");
                errors += 1;
            }
        }
    } else {
        tracing::warn!("No themes file found at {}", theme_path.display());
    }

    // Validate blueprints
    let registry = BlueprintRegistry::phase18_defaults();
    let bp_count = registry.blueprints.len();
    let mut bp_errors = 0;
    for bp in &registry.blueprints {
        if bp.id.is_empty() {
            tracing::error!("Blueprint has empty id");
            bp_errors += 1;
        }
        if bp.label.is_empty() {
            tracing::error!("Blueprint '{}' has empty label", bp.id);
            bp_errors += 1;
        }
    }
    tracing::info!("Blueprints: {bp_count} items, {bp_errors} errors");

    errors += bp_errors;

    if errors == 0 {
        tracing::info!("Content validation PASSED");
    } else {
        tracing::error!("Content validation FAILED with {errors} error(s)");
    }
}
