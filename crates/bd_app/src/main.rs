//! bd_app — Binary entry point for Broken Divinity Kernel.
//!
//! Initializes tracing, Bevy app with Bevy-Ratatui,
//! and spawns the Phase 1 minimal terminal slice.

use std::time::Duration;
use std::{collections::HashMap, path::Path};

use bevy_app::{PanicHandlerPlugin, ScheduleRunnerPlugin, Startup};
use bevy_ecs::schedule::IntoScheduleConfigs;
use bevy_ecs::system::{Commands, Query, Res, ResMut};

use bd_core::HelpLine;
use bd_core::components::{Player, Position};
use bd_core::factory::spawn_from_blueprint;
use bd_core::gamelog::{GameLog, LogLevel};
use bevy_ecs::entity::Entity;
use bevy_ecs::query::With;

mod config;

fn main() {
    // Parse CLI args
    let args: Vec<String> = std::env::args().collect();
    let validate_only = args.iter().any(|a| a == "--validate" || a == "-v");

    // Initialize tracing — write to stderr so TUI stdout stays clean
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "bd=info".into()),
        )
        .init();

    if validate_only {
        if !run_validation() {
            std::process::exit(1);
        }
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
    app.add_plugins(bd_core::BdFoundationPlugin);
    app.add_plugins(bd_tui::BdTuiPlugin);

    // Override HelpLine with config-derived value
    let help_line = HelpLine(loaded.config.keybindings.help_line());
    app.insert_resource(help_line);

    // Override defaults with RON content at startup
    app.add_systems(Startup, apply_ron_content);
    app.add_systems(Startup, load_foundation_content);

    // Spawn player when entering outpost mode (Startup + transitions)
    app.add_systems(
        bevy_app::Update,
        spawn_outpost_player.in_set(bd_core::BdSet::IntentCollection),
    );
    app.add_systems(
        bevy_app::Update,
        process_persistence_requests.in_set(bd_core::BdSet::ResultEmission),
    );

    app.run();
    tracing::info!("Broken Divinity Kernel exited cleanly");
}

/// Execute persistence requests emitted by the TUI at the application
/// boundary. The kernel owns snapshot contents; the application owns paths
/// and user-facing success/failure reporting.
fn process_persistence_requests(world: &mut bevy_ecs::world::World) {
    let save_requested = world.resource_mut::<bd_core::save::SaveRequest>().0;
    world.resource_mut::<bd_core::save::SaveRequest>().0 = false;
    if save_requested {
        let session = world.resource::<bd_core::session::RunSession>();
        let seed = session.seed;
        let turn = session.turn;
        let save_dir = config::data_dir().join("saves");
        match bd_core::save::save_world(world, seed, turn, &save_dir) {
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
        let latest = std::fs::read_dir(&save_dir).ok().and_then(|entries| {
            entries
                .filter_map(Result::ok)
                .filter_map(|entry| {
                    let path = entry.path();
                    let turn = path
                        .file_stem()?
                        .to_str()?
                        .strip_prefix("save-turn-")?
                        .parse::<u64>()
                        .ok()?;
                    Some((turn, path))
                })
                .max_by_key(|(turn, _)| *turn)
                .map(|(_, path)| path)
        });
        let Some(path) = latest else {
            world.resource_mut::<GameLog>().push(
                "Load failed: no save file exists.".to_string(),
                LogLevel::Warn,
            );
            return;
        };
        match bd_core::save::load_snapshot(&path).and_then(|snapshot| {
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

/// Load the required foundation bundle. A runtime without its foundation
/// content is invalid and must not silently fall back to Rust fixtures.
fn load_foundation_content(mut commands: Commands) {
    let content_dir = content_dir();
    match bd_data::loader::load_foundation_content(&content_dir) {
        Ok(content) => {
            tracing::info!(
                "Loaded foundation content: {} dungeons, {} items, {} skills, {} factions",
                content.dungeons.len(),
                content.items.len(),
                content.skills.len(),
                content.factions.len()
            );
            commands.insert_resource(content);
        }
        Err(error) => panic!("Foundation content failed validation: {error}"),
    }
}

/// Spawn the player at the shelter outpost at startup.
fn spawn_outpost_player(
    mut commands: Commands,
    mut game_log: ResMut<GameLog>,
    mode: Res<bd_core::spatial::GameMode>,
    content: Res<bd_core::content::FoundationContent>,
    player: Query<Entity, With<Player>>,
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
        "b: build | a: assign | t: travel | i: inventory | p: pickup | r: extract",
        LogLevel::Info,
    );
}

/// Run content validation and exit.
fn run_validation() -> bool {
    use bd_data::loader::load_ron;

    let content_dir = content_dir();
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

    match bd_data::loader::load_foundation_content(&content_dir) {
        Ok(bundle) => tracing::info!(
            "Foundation: {} dungeons, {} items, {} skills, {} factions, {} actions, {} blueprints",
            bundle.dungeons.len(),
            bundle.items.len(),
            bundle.skills.len(),
            bundle.factions.len(),
            bundle.actions.len(),
            bundle.blueprints.len()
        ),
        Err(error) => {
            tracing::error!("Foundation validation: {error}");
            errors += 1;
        }
    }

    if errors == 0 {
        tracing::info!("Content validation PASSED");
        true
    } else {
        tracing::error!("Content validation FAILED with {errors} error(s)");
        false
    }
}

fn content_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("content")
}
