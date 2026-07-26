//! Architecture tests — verify schedule discipline and signal boundaries.
//!
//! These tests validate that the kernel's architectural rules are enforced,
//! not just that individual systems behave correctly.

#[cfg(test)]
mod tests {
    use bd_core::BdSet;
    use bevy_app::{App, Update};
    use bevy_ecs::prelude::{IntoScheduleConfigs, ResMut, Resource};

    #[derive(Resource, Default)]
    struct ScheduleProbe(Vec<&'static str>);

    /// Keep the public set vocabulary stable. This does not claim that the
    /// Bevy schedule executed the sets in this order.
    #[test]
    fn bd_set_declaration_names_remain_stable() {
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

    fn required_stage_positions(stages: &[&str]) -> Result<[usize; 3], String> {
        let required = ["Validation", "CostResolution", "Mutation"];
        let mut positions = [0; 3];

        for (required_index, required_stage) in required.iter().enumerate() {
            let matches: Vec<usize> = stages
                .iter()
                .enumerate()
                .filter_map(|(index, stage)| (*stage == *required_stage).then_some(index))
                .collect();
            if matches.len() != 1 {
                return Err(format!(
                    "{required_stage} must occur exactly once, found {}",
                    matches.len()
                ));
            }
            positions[required_index] = matches[0];
        }

        if !(positions[0] < positions[1] && positions[1] < positions[2]) {
            return Err(format!(
                "required order is Validation < CostResolution < Mutation, positions={positions:?}"
            ));
        }
        Ok(positions)
    }

    #[test]
    fn stage_sequence_validator_rejects_missing_duplicate_and_reordered_stages() {
        assert!(required_stage_positions(&["Validation", "CostResolution", "Mutation"]).is_ok());
        assert!(required_stage_positions(&["Validation", "Mutation"]).is_err());
        assert!(
            required_stage_positions(&[
                "Validation",
                "CostResolution",
                "CostResolution",
                "Mutation",
            ])
            .is_err()
        );
        assert!(required_stage_positions(&["Mutation", "Validation", "CostResolution"]).is_err());
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
    fn denied_move_never_reaches_cost_or_mutation() {
        use bd_core::components::{Player, Position, Tile};
        use bd_core::map::SmokeMap;
        use bd_core::pools::{Pool, Pools};
        use bd_core::signals::{ActionIntent, PoolKind};
        use bd_core::trace::SignalTrace;
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

        let trace = app.world().resource::<SignalTrace>();
        let stages: Vec<&str> = trace.entries.iter().map(|entry| entry.stage).collect();
        assert_eq!(
            stages
                .iter()
                .filter(|stage| **stage == "Validation")
                .count(),
            1,
            "a denied move must be validated exactly once"
        );
        assert!(
            !stages.contains(&"CostResolution"),
            "a denied move reached cost resolution: {stages:?}"
        );
        assert!(
            !stages.contains(&"Mutation"),
            "a denied move reached mutation: {stages:?}"
        );
    }

    /// Execute one accepted action through the production plugin and prove
    /// that each action-pipeline signal appears exactly once in stage order.
    #[test]
    fn accepted_move_emits_each_required_trace_stage_exactly_once() {
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
        let required_signals = [
            ("Validation", "ActionIntent"),
            ("CostResolution", "CostCompile"),
            ("Mutation", "EffectResolve"),
        ];
        let mut positions = [0; 3];
        for (required_index, required) in required_signals.iter().enumerate() {
            let matches: Vec<usize> = trace
                .entries
                .iter()
                .enumerate()
                .filter_map(|(index, entry)| {
                    ((entry.stage, entry.signal_type) == *required).then_some(index)
                })
                .collect();
            assert_eq!(
                matches.len(),
                1,
                "{required:?} must appear exactly once; trace={:?}",
                trace.entries
            );
            positions[required_index] = matches[0];
        }
        assert!(
            positions[0] < positions[1] && positions[1] < positions[2],
            "required pipeline signal order drifted; positions={positions:?}, trace={:?}",
            trace.entries
        );
    }

    #[test]
    fn production_schedule_executes_required_sets_in_declared_order() {
        let mut app = App::new();
        app.add_plugins(bd_core::BdCorePlugin);
        app.init_resource::<ScheduleProbe>();
        app.add_systems(
            Update,
            (
                (|mut probe: ResMut<ScheduleProbe>| probe.0.push("Validation"))
                    .in_set(BdSet::Validation),
                (|mut probe: ResMut<ScheduleProbe>| probe.0.push("CostResolution"))
                    .in_set(BdSet::CostResolution),
                (|mut probe: ResMut<ScheduleProbe>| probe.0.push("Mutation"))
                    .in_set(BdSet::Mutation),
            ),
        );

        app.update();

        let stages = &app.world().resource::<ScheduleProbe>().0;
        required_stage_positions(stages)
            .unwrap_or_else(|error| panic!("{error}; actual probe stages={stages:?}"));
    }
}
