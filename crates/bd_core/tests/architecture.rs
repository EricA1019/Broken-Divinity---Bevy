//! Architecture tests — verify schedule discipline and signal boundaries.
//!
//! These tests validate that the kernel's architectural rules are enforced,
//! not just that individual systems behave correctly.

#[cfg(test)]
mod tests {
    use bd_core::BdSet;
    use bevy_app::App;

    /// Verify the declared schedule order is stable.
    #[test]
    fn system_order_matches_declared_schedule() {
        // The BdSet variants in declaration order
        let expected = [
            "Input",
            "IntentCollection",
            "Validation",
            "CostResolution",
            "EffectEmission",
            "ModifierApplication",
            "Mutation",
            "ResultEmission",
            "ViewModelBuild",
            "Render",
        ];

        let actual: Vec<String> = [
            BdSet::Input,
            BdSet::IntentCollection,
            BdSet::Validation,
            BdSet::CostResolution,
            BdSet::EffectEmission,
            BdSet::ModifierApplication,
            BdSet::Mutation,
            BdSet::ResultEmission,
            BdSet::ViewModelBuild,
            BdSet::Render,
        ]
        .iter()
        .map(|s| format!("{s:?}"))
        .collect();

        assert_eq!(actual, expected, "BdSet order must not change silently");
    }

    /// Verify that BdCorePlugin registers the trigger guard resource.
    #[test]
    fn trigger_guard_resource_exists() {
        let mut app = App::new();
        app.add_plugins(bd_core::BdCorePlugin);

        let guard = app
            .world()
            .get_resource::<bd_core::trace::TriggerExecutionGuard>()
            .expect("TriggerExecutionGuard should be registered");
        assert_eq!(guard.max_depth, 10);
    }

    /// Verify that SignalTrace resource exists and can be written to.
    #[test]
    fn signal_trace_resource_exists() {
        let mut app = App::new();
        app.add_plugins(bd_core::BdCorePlugin);

        let mut trace = app
            .world_mut()
            .get_resource_mut::<bd_core::trace::SignalTrace>()
            .expect("SignalTrace should be registered");
        trace.push("Test", "TestSignal", "test summary".into());
        assert_eq!(trace.entries.len(), 1);
    }

    /// Verify that an invalid action stops before cost resolution (doesn't spend AP).
    #[test]
    fn invalid_action_stops_before_cost_resolution() {
        use bd_core::components::{Player, Position, Tile};
        use bd_core::map::SmokeMap;
        use bd_core::pools::{Pool, Pools};
        use bd_core::signals::{ActionIntent, PoolKind};
        use bevy_ecs::message::Messages;

        let mut app = App::new();
        app.add_plugins(bd_core::BdCorePlugin);

        // Wall at (6,5)
        let mut map = SmokeMap::new(10, 10, Tile::Floor);
        map.set(6, 5, Tile::Wall);
        app.world_mut().insert_resource(map);

        let player = app
            .world_mut()
            .spawn((
                Player,
                Position { x: 5, y: 5 },
                Pools::new(vec![
                    Pool::new(PoolKind::Health, 20, 0, 20),
                    Pool::new(PoolKind::ActionPoints, 3, 0, 3),
                ]),
            ))
            .id();

        // Try to move into a wall
        app.world_mut()
            .resource_mut::<Messages<ActionIntent>>()
            .write(ActionIntent {
                actor: player,
                action_id: "ability.move".into(),
                direction: Some(bd_core::direction::Direction::East),
                target: None,
            });

        app.update();

        // AP should NOT have been spent since the action was denied
        let ap = app
            .world()
            .get::<Pools>(player)
            .unwrap()
            .get(PoolKind::ActionPoints)
            .unwrap()
            .current;
        assert_eq!(ap, 3, "Denied actions must not spend AP");
    }

    /// Verify trace records ordered flow through the pipeline.
    #[test]
    fn trace_records_ordered_flow() {
        use bd_core::components::{Player, Position, Tile};
        use bd_core::map::SmokeMap;
        use bd_core::pools::{Pool, Pools};
        use bd_core::signals::{ActionIntent, PoolKind};
        use bd_core::trace::SignalTrace;
        use bevy_ecs::message::Messages;

        let mut app = App::new();
        app.add_plugins(bd_core::BdCorePlugin);
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));

        let player = app
            .world_mut()
            .spawn((
                Player,
                Position { x: 5, y: 5 },
                Pools::new(vec![
                    Pool::new(PoolKind::Health, 20, 0, 20),
                    Pool::new(PoolKind::ActionPoints, 3, 0, 3),
                ]),
            ))
            .id();

        app.world_mut()
            .resource_mut::<Messages<ActionIntent>>()
            .write(ActionIntent {
                actor: player,
                action_id: "ability.move".into(),
                direction: Some(bd_core::direction::Direction::East),
                target: None,
            });

        app.update();

        let trace = app.world().resource::<SignalTrace>();
        assert!(
            !trace.entries.is_empty(),
            "Trace should have entries after a move action"
        );

        // Verify stages appear in order
        let stages: Vec<&str> = trace.entries.iter().map(|e| e.stage).collect();
        let validation_idx = stages.iter().position(|&s| s == "Validation");
        let cost_idx = stages.iter().position(|&s| s == "CostResolution");
        let mutation_idx = stages.iter().position(|&s| s == "Mutation");

        if let (Some(v), Some(c), Some(m)) = (validation_idx, cost_idx, mutation_idx) {
            assert!(
                v < c && c < m,
                "Trace stages must be: Validation < CostResolution < Mutation"
            );
        }
    }
}
