//! Canonical foundation run/session state.
//!
//! The session owns foundation phase, run identity, extraction state, outcome,
//! and replay metadata. UI state and legacy `GameMode` access are projections
//! of this resource; they are not independent authorities.

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::spatial::GameMode;

/// Marker resource identifying the foundation runtime profile.
#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct FoundationRuntime;

/// Outcome of the current foundation run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunOutcome {
    None,
    Extracted,
    Defeated,
}

impl Default for RunOutcome {
    fn default() -> Self {
        Self::None
    }
}

/// Canonical state for a single foundation run.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct RunSession {
    pub phase: GameMode,
    pub seed: u64,
    pub day: u64,
    pub turn: u64,
    pub dungeon_id: Option<String>,
    pub extraction_applied: bool,
    pub extracted_loot: u32,
    pub outcome: RunOutcome,
    pub replay_intents: Vec<String>,
}

impl Default for RunSession {
    fn default() -> Self {
        Self::new(0)
    }
}

impl RunSession {
    pub fn new(seed: u64) -> Self {
        Self {
            phase: GameMode::Title,
            seed,
            day: 0,
            turn: 0,
            dungeon_id: None,
            extraction_applied: false,
            extracted_loot: 0,
            outcome: RunOutcome::None,
            replay_intents: Vec::new(),
        }
    }

    /// Foundation transition policy. Legacy travel is intentionally excluded.
    pub fn allows_foundation_transition(from: GameMode, to: GameMode) -> bool {
        matches!(
            (from, to),
            (GameMode::Title, GameMode::Outpost)
                | (GameMode::Outpost, GameMode::Tactical)
                | (GameMode::Tactical, GameMode::Outpost)
                | (GameMode::Tactical, GameMode::GameOver)
                | (GameMode::GameOver, GameMode::Title)
        )
    }

    pub fn record_intent(&mut self, action_id: impl Into<String>) {
        self.replay_intents.push(action_id.into());
    }

    pub fn begin_dungeon(&mut self, dungeon_id: impl Into<String>) {
        self.phase = GameMode::Tactical;
        self.dungeon_id = Some(dungeon_id.into());
        self.extraction_applied = false;
        self.extracted_loot = 0;
        self.outcome = RunOutcome::None;
    }

    pub fn mark_extracted(&mut self) -> bool {
        if self.extraction_applied {
            return false;
        }
        self.extraction_applied = true;
        self.outcome = RunOutcome::Extracted;
        true
    }

    pub fn mark_extracted_with_loot(&mut self, loot: u32) -> bool {
        if !self.mark_extracted() {
            return false;
        }
        self.extracted_loot = loot;
        true
    }

    pub fn mark_defeated(&mut self) {
        self.outcome = RunOutcome::Defeated;
        self.phase = GameMode::GameOver;
    }
}

/// Register session state for the foundation runtime.
pub(crate) fn register_session(app: &mut bevy_app::App, foundation: bool) {
    app.init_resource::<RunSession>();
    app.add_systems(
        bevy_app::Update,
        synchronize_rng.in_set(crate::BdSet::Input),
    );
    if foundation {
        app.init_resource::<FoundationRuntime>();
    }
}

fn synchronize_rng(
    session: Res<RunSession>,
    mut combat_rng: ResMut<crate::combat::CombatRng>,
    mut last_seed: Local<Option<u64>>,
) {
    if *last_seed != Some(session.seed) {
        *combat_rng = crate::combat::CombatRng::from_seed(session.seed);
        *last_seed = Some(session.seed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;
    use bevy_ecs::message::Messages;

    #[test]
    fn foundation_transition_table_accepts_only_mvp_edges() {
        assert!(RunSession::allows_foundation_transition(
            GameMode::Title,
            GameMode::Outpost
        ));
        assert!(RunSession::allows_foundation_transition(
            GameMode::Outpost,
            GameMode::Tactical
        ));
        assert!(RunSession::allows_foundation_transition(
            GameMode::Tactical,
            GameMode::Outpost
        ));
        assert!(!RunSession::allows_foundation_transition(
            GameMode::Outpost,
            GameMode::Travel
        ));
    }

    #[test]
    fn extraction_is_idempotent() {
        let mut session = RunSession::new(42);
        session.begin_dungeon("foundation.dungeon");
        assert!(session.mark_extracted());
        assert!(!session.mark_extracted());
        assert_eq!(session.outcome, RunOutcome::Extracted);
    }

    #[test]
    fn session_serializes_seed_and_phase() {
        let mut session = RunSession::new(42);
        session.begin_dungeon("foundation.dungeon");
        session.record_intent("ability.move");

        let encoded = ron::ser::to_string(&session).unwrap();
        let restored: RunSession = ron::de::from_str(&encoded).unwrap();

        assert_eq!(restored.seed, 42);
        assert_eq!(restored.phase, GameMode::Tactical);
        assert_eq!(restored.dungeon_id.as_deref(), Some("foundation.dungeon"));
        assert_eq!(restored.replay_intents, vec!["ability.move"]);
    }

    #[test]
    fn foundation_transition_system_updates_canonical_session() {
        let mut app = App::new();
        app.add_plugins(crate::BdFoundationPlugin);

        app.world_mut()
            .resource_mut::<Messages<crate::spatial::TransitionIntent>>()
            .write(crate::spatial::TransitionIntent {
                target: GameMode::Outpost,
                node_id: None,
            });
        app.update();

        assert_eq!(
            app.world().resource::<RunSession>().phase,
            GameMode::Outpost
        );
        assert_eq!(*app.world().resource::<GameMode>(), GameMode::Outpost);

        app.world_mut()
            .resource_mut::<Messages<crate::spatial::TransitionIntent>>()
            .write(crate::spatial::TransitionIntent {
                target: GameMode::Tactical,
                node_id: Some("foundation.dungeon".into()),
            });
        app.update();

        let session = app.world().resource::<RunSession>();
        assert_eq!(session.phase, GameMode::Tactical);
        assert_eq!(session.dungeon_id.as_deref(), Some("foundation.dungeon"));
    }

    #[test]
    fn foundation_rejects_travel_without_mutating_session() {
        let mut app = App::new();
        app.add_plugins(crate::BdFoundationPlugin);

        app.world_mut()
            .resource_mut::<Messages<crate::spatial::TransitionIntent>>()
            .write(crate::spatial::TransitionIntent {
                target: GameMode::Travel,
                node_id: Some("legacy-node".into()),
            });
        app.update();

        assert_eq!(app.world().resource::<RunSession>().phase, GameMode::Title);
        assert_eq!(*app.world().resource::<GameMode>(), GameMode::Title);
    }
}
