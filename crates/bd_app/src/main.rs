//! bd_app — Binary entry point for Broken Divinity Kernel.
//!
//! Initializes tracing, Bevy app with Bevy-Ratatui,
//! and spawns the Phase 1 minimal terminal slice.

use std::time::Duration;

use bevy_app::{PanicHandlerPlugin, ScheduleRunnerPlugin, Startup};
use bevy_ecs::system::{Commands, ResMut};

use bd_core::components::{BlocksMovement, Name, Player, Position};
use bd_core::gamelog::{GameLog, LogLevel};
use bd_core::pools::{Pool, Pools};
use bd_core::signals::PoolKind;

fn main() {
    // Initialize tracing (color-eyre is set up by Bevy internally)
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "bd=info".into()),
        )
        .init();

    tracing::info!("Broken Divinity Kernel starting");

    let frame_time = Duration::from_secs_f32(1.0 / 60.0);

    let mut app = bevy_app::App::new();

    // Schedule runner keeps the app alive (required for headless/terminal apps)
    app.add_plugins(ScheduleRunnerPlugin::run_loop(frame_time));

    // Panic handler ensures terminal cleanup on crash
    app.add_plugins(PanicHandlerPlugin);

    // Bevy-Ratatui terminal plugin (must be after MinimalPlugins)
    app.add_plugins(bevy_ratatui::RatatuiPlugins::default());

    // Core plugin
    app.add_plugins(bd_core::BdCorePlugin);

    // TUI plugin
    app.add_plugins(bd_tui::BdTuiPlugin);

    // Spawn initial entities
    app.add_systems(Startup, spawn_world);

    // Run the app (loops until AppExit)
    app.run();

    tracing::info!("Broken Divinity Kernel exited cleanly");
}

/// Spawn the player and some initial entities.
fn spawn_world(mut commands: Commands, mut game_log: ResMut<GameLog>) {
    // Player at center of the map
    commands.spawn((
        Player,
        Position { x: 10, y: 6 },
        Name("Player".into()),
        Pools::new(vec![
            Pool::new(PoolKind::Health, 20, 0, 20),
            Pool::new(PoolKind::ActionPoints, 3, 0, 3),
        ]),
    ));

    // A training dummy to test attack
    commands.spawn((
        BlocksMovement,
        Position { x: 12, y: 6 },
        Name("Training Dummy".into()),
        Pools::new(vec![
            Pool::new(PoolKind::Health, 15, 0, 15),
            Pool::new(PoolKind::ActionPoints, 0, 0, 0),
        ]),
    ));

    game_log.push("You enter the smoke-filled chamber.", LogLevel::Info);
    game_log.push("WASD or arrow keys to move.", LogLevel::Info);
}
