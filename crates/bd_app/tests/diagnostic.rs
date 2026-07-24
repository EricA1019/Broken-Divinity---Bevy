// Auto-generated integration test — DO NOT COMMIT
// Purpose: diagnose what the outpost state looks like after title→outpost transition

#[cfg(test)]
mod integration_diagnostic {
    use bd_core::components::{Player, Position};
    use bd_core::gamelog::GameLog;
    use bd_core::map::SmokeMap;
    use bd_core::pools::Pools;
    use bd_core::signals::PoolKind;
    use bd_core::spatial::{GameMode, OutpostState};
    use bevy_app::App;
    use bevy_ecs::prelude::*;

    #[test]
    #[ignore = "diagnostic-only snapshot; not part of the foundation regression gate"]
    fn diagnose_title_to_outpost_state() {
        let mut app = App::new();
        app.add_plugins(bd_core::BdCorePlugin);
        app.add_plugins(bd_tui::BdTuiPlugin);

        // Simulate title screen keypress: set mode to Outpost
        app.world_mut().insert_resource(GameMode::Outpost);

        // Run one frame to process the transition
        app.update();

        // -- ECS State Diagnostics --

        // 1. GameMode
        let mode = *app.world().resource::<GameMode>();
        eprintln!("DIAG: GameMode = {:?}", mode);

        // 2. Player entity
        let players: Vec<(Entity, Position)> = {
            let world = app.world_mut();
            let mut query = world.query::<(Entity, &Position, Option<&Player>)>();
            query
                .iter(world)
                .filter(|(_, _, player)| player.is_some())
                .map(|(e, p, _)| (e, *p))
                .collect()
        };
        eprintln!("DIAG: Player count = {}", players.len());
        for (e, pos) in &players {
            let hp = app
                .world()
                .get::<Pools>(*e)
                .and_then(|p| p.get(PoolKind::Health))
                .map(|p| p.current)
                .unwrap_or(-1);
            let ap = app
                .world()
                .get::<Pools>(*e)
                .and_then(|p| p.get(PoolKind::ActionPoints))
                .map(|p| p.current)
                .unwrap_or(-1);
            eprintln!(
                "DIAG: Player {:?} at ({},{}), HP={}, AP={}",
                e, pos.x, pos.y, hp, ap
            );
        }

        // 3. Survivor count
        let survivor_count = {
            let world = app.world_mut();
            let mut query = world.query::<&bd_core::components::Name>();
            query.iter(world).count()
        };
        eprintln!("DIAG: Named entities = {}", survivor_count);

        // 4. Outpost map dimensions
        let outpost = app.world().resource::<OutpostState>();
        eprintln!(
            "DIAG: Outpost map = {}x{}",
            outpost.map.width, outpost.map.height
        );

        // 5. Global SmokeMap dimensions
        let map = app.world().resource::<SmokeMap>();
        eprintln!("DIAG: Global SmokeMap = {}x{}", map.width, map.height);

        // 6. GameLog messages
        let log = app.world().resource::<GameLog>();
        eprintln!("DIAG: GameLog entries = {}", log.iter().count());
        for (i, entry) in log.iter().enumerate() {
            eprintln!("DIAG: Log[{}] = [{:?}] {}", i, entry.level, entry.message);
        }

        // 7. Total entities
        let total = {
            let world = app.world_mut();
            let mut query = world.query::<Entity>();
            query.iter(world).count()
        };
        eprintln!("DIAG: Total entities = {}", total);

        // 8. Screen state
        let screen = app.world().resource::<bd_tui::screens::ScreenState>();
        eprintln!("DIAG: ScreenState = {}", screen.current);
    }
}
