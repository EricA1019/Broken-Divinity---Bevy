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

/// Generate a procedural location and spawn the MVP entities.
fn spawn_world(
    mut commands: Commands,
    mut game_log: ResMut<GameLog>,
    mut map: ResMut<SmokeMap>,
) {
    let registry = BlueprintRegistry::phase18_defaults();

    // Generate a procedural ruin
    let seed = 42; // could be read from config later
    let template = LocationTemplate::ruin();
    let plan = generate_location(&template, seed);

    // Replace the default smoke map with the generated one
    *map = SmokeMap::from_tiles(plan.width, plan.height, &plan.tiles);

    // Set entrance tile as Door
    map.set(plan.entrance.x, plan.entrance.y, bd_core::components::Tile::Door);

    // Spawn player at the entrance
    if let Some(bp) = registry.get("blueprint.player") {
        spawn_from_blueprint(bp, Some(plan.entrance), &[], &mut commands);
    }

    // Spawn enemies on spawn zones
    let enemy_blueprints = ["blueprint.rat", "blueprint.skeleton"];
    for (i, zone) in plan.spawn_zones.iter().enumerate() {
        let bp_id = enemy_blueprints[i % enemy_blueprints.len()];
        if let Some(bp) = registry.get(bp_id) {
            spawn_from_blueprint(bp, Some(*zone), &[], &mut commands);
        }
    }

    // Spawn items scattered in rooms
    let item_bps = [
        "blueprint.healing_potion",
        "blueprint.sword",
        "blueprint.shield",
        "blueprint.smite_scroll",
        "blueprint.gold_pile",
    ];
    for (i, bp_id) in item_bps.iter().enumerate() {
        if let Some(room) = plan.rooms.get((i + 1) % plan.rooms.len()) {
            let pos = Position {
                x: room.x + 1,
                y: room.y + 1 + i as i32 % 2,
            };
            if map.is_walkable(pos.x, pos.y) {
                if let Some(bp) = registry.get(bp_id) {
                    let entity = spawn_from_blueprint(bp, Some(pos), &[], &mut commands);
                    commands.entity(entity).insert(BlocksMovement);
                    // Remove BlocksMovement for items (they shouldn't block)
                    commands.entity(entity).remove::<BlocksMovement>();
                }
            }
        }
    }

    // Place exit marker on the first exit position
    if let Some(exit_pos) = plan.exits.first() {
        map.set(exit_pos.x, exit_pos.y, bd_core::components::Tile::Door);
        // Spawn an exit entity marker
        commands.spawn((
            ExitTile,
            *exit_pos,
            Name("Exit".into()),
        ));
    }

    game_log.push("You enter a crumbling ruin...", LogLevel::Info);
    game_log.push("WASD: move | f: attack | g: guard | .: wait | i: inventory",
        LogLevel::Info,
    );
    game_log.push("Find your way to the exit >", LogLevel::Info);
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
