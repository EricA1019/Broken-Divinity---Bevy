//! bd_app — Binary entry point for Broken Divinity Kernel.
//!
//! Initializes tracing, Bevy app with Bevy-Ratatui,
//! and spawns the Phase 1 minimal terminal slice.

use std::path::Path;
use std::time::Duration;

use bevy_app::{PanicHandlerPlugin, ScheduleRunnerPlugin, Startup};
use bevy_ecs::system::{Commands, ResMut};

use bd_core::components::Position;
use bd_core::factory::{BlueprintRegistry, Mutator, spawn_from_blueprint};
use bd_core::gamelog::{GameLog, LogLevel};
use bd_core::HelpLine;

mod config;

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "bd=info".into()),
        )
        .init();

    tracing::info!("Broken Divinity Kernel starting");

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

    // Override HelpLine with config-derived value
    let help_line = HelpLine(loaded.config.keybindings.help_line());
    app.insert_resource(help_line);

    // Override defaults with RON content at startup
    app.add_systems(Startup, apply_ron_content);

    // Spawn initial entities
    app.add_systems(Startup, spawn_world);

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

/// Spawn the player and some initial entities using the entity factory.
fn spawn_world(mut commands: Commands, mut game_log: ResMut<GameLog>) {
    let registry = BlueprintRegistry::phase10_defaults();

    // Spawn player via blueprint
    if let Some(bp) = registry.get("blueprint.player") {
        spawn_from_blueprint(bp, Some(Position { x: 10, y: 6 }), &[], &mut commands);
    }

    // Spawn a wounded training dummy
    if let Some(bp) = registry.get("blueprint.training_dummy") {
        spawn_from_blueprint(
            bp,
            Some(Position { x: 12, y: 6 }),
            &[Mutator::Wounded],
            &mut commands,
        );
    }

    game_log.push("You enter the smoke-filled chamber.", LogLevel::Info);
    game_log.push("WASD or arrow keys to move.", LogLevel::Info);
}
