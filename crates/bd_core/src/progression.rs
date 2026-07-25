//! Data-driven skill progression and virtue expressions.

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    BdSet,
    components::Player,
    content::FoundationContent,
    signals::{PoolDeltaRequested, PoolKind},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillId {
    Melee,
    Ranged,
    Repair,
    Medicine,
}

impl SkillId {
    pub fn parse(id: &str) -> Option<Self> {
        match id {
            "skill.melee" => Some(Self::Melee),
            "skill.ranged" => Some(Self::Ranged),
            "skill.repair" => Some(Self::Repair),
            "skill.medicine" => Some(Self::Medicine),
            _ => None,
        }
    }

    pub fn value(self, state: &SkillProgression) -> i32 {
        match self {
            Self::Melee => state.melee,
            Self::Ranged => state.ranged,
            Self::Repair => state.repair,
            Self::Medicine => state.medicine,
        }
    }

    fn add(self, state: &mut SkillProgression, amount: i32) {
        match self {
            Self::Melee => state.melee += amount,
            Self::Ranged => state.ranged += amount,
            Self::Repair => state.repair += amount,
            Self::Medicine => state.medicine += amount,
        }
    }
}

/// Progression owned by the player entity, never by the presentation layer.
#[derive(Component, Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillProgression {
    pub melee: i32,
    pub ranged: i32,
    pub repair: i32,
    pub medicine: i32,
}

#[derive(Message, Debug, Clone)]
pub struct ActionResolved {
    pub actor: Entity,
    pub action_id: String,
}

/// Ensure players created by old factories receive the new progression state.
fn ensure_player_progression(
    mut commands: Commands,
    players: Query<(Entity, Option<&SkillProgression>), With<Player>>,
) {
    for (entity, progression) in players.iter() {
        if progression.is_none() {
            commands.entity(entity).insert(SkillProgression::default());
        }
    }
}

fn apply_action_progression(
    mut messages: MessageReader<ActionResolved>,
    content: Option<Res<FoundationContent>>,
    mut action_deltas: MessageWriter<PoolDeltaRequested>,
    mut players: Query<&mut SkillProgression>,
    mut game_log: ResMut<crate::gamelog::GameLog>,
) {
    let Some(content) = content else {
        return;
    };

    for result in messages.read() {
        let Some(metadata) = content
            .actions
            .iter()
            .find(|action| action.id == result.action_id)
        else {
            continue;
        };
        let Some(skill_name) = metadata.skill_id.as_deref() else {
            continue;
        };
        let Some(skill_id) = SkillId::parse(skill_name) else {
            continue;
        };
        let Ok(mut progression) = players.get_mut(result.actor) else {
            continue;
        };

        let before = skill_id.value(&progression);
        skill_id.add(&mut progression, metadata.skill_gain);
        tracing::debug!(
            actor = ?result.actor,
            action = %result.action_id,
            skill = ?skill_id,
            before,
            after = skill_id.value(&progression),
            "skill progression applied"
        );
        game_log.push(
            format!(
                "{:?} skill improves to {}.",
                skill_id,
                skill_id.value(&progression)
            ),
            crate::gamelog::LogLevel::Info,
        );

        let Some(virtue_name) = metadata.virtue_expression.as_deref() else {
            continue;
        };
        let Some(virtue) = virtue_pool(virtue_name) else {
            continue;
        };
        action_deltas.write(PoolDeltaRequested {
            source: Some(result.actor),
            target: result.actor,
            kind: virtue,
            amount: metadata.virtue_gain,
            tags: vec![],
            reason: format!("{} expression", virtue_name),
        });
        game_log.push(
            format!("You express {}.", virtue_name),
            crate::gamelog::LogLevel::Info,
        );
    }
}

fn virtue_pool(id: &str) -> Option<PoolKind> {
    match id {
        "virtue.temperance" => Some(PoolKind::Temperance),
        "virtue.prudence" => Some(PoolKind::Prudence),
        "virtue.thumos" => Some(PoolKind::Thumos),
        "virtue.metis" => Some(PoolKind::Metis),
        _ => None,
    }
}

pub(crate) fn register_progression(app: &mut bevy_app::App) {
    app.add_message::<ActionResolved>();
    app.add_systems(
        bevy_app::Update,
        ensure_player_progression.in_set(BdSet::IntentCollection),
    );
    app.add_systems(
        bevy_app::Update,
        apply_action_progression.in_set(BdSet::ResultEmission),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        components::{BlocksMovement, Name, Position},
        content::SkillDefinition,
        map::SmokeMap,
        pools::{Pool, Pools},
        signals::ActionIntent,
    };
    use bevy_app::App;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(crate::BdFoundationPlugin);
        app.insert_resource(FoundationContent {
            actions: vec![crate::content::ActionReference {
                id: "ability.quick_attack".into(),
                label: "Quick Attack".into(),
                skill_id: Some("skill.melee".into()),
                skill_gain: 1,
                virtue_expression: Some("virtue.thumos".into()),
                virtue_gain: 1,
            }],
            skills: vec![SkillDefinition {
                id: "skill.melee".into(),
                label: "Melee".into(),
                action_id: "ability.quick_attack".into(),
                governing_virtue: "virtue.thumos".into(),
                progression_rate: 1,
            }],
            factions: vec![crate::content::FactionDefinition {
                id: "faction.test_hostile".into(),
                label: "Test Hostile".into(),
                identity_key: "test_hostile".into(),
                disposition: crate::content::FoundationDisposition::Hostile,
            }],
            ..Default::default()
        });
        app.world_mut()
            .insert_resource(crate::spatial::GameMode::Tactical);
        app.world_mut()
            .resource_mut::<crate::session::RunSession>()
            .phase = crate::spatial::GameMode::Tactical;
        app
    }

    #[test]
    fn action_uses_declared_skill_and_expression() {
        let mut app = test_app();
        app.insert_resource(SmokeMap::new(10, 10, crate::components::Tile::Floor));
        let actor = app
            .world_mut()
            .spawn((
                Player,
                crate::spatial::EntityScope::RunPersistent,
                Name("Player".into()),
                Position { x: 5, y: 5 },
                SkillProgression::default(),
                Pools::new(vec![
                    Pool::new(PoolKind::Health, 20, 0, 20),
                    Pool::new(PoolKind::ActionPoints, 3, 0, 3),
                    Pool::new(PoolKind::Thumos, 0, 0, 100),
                ]),
            ))
            .id();
        let target = app
            .world_mut()
            .spawn((
                BlocksMovement,
                crate::relationships::FactionMember("faction.test_hostile".into()),
                crate::spatial::EntityScope::DungeonTransient,
                Position { x: 6, y: 5 },
                Pools::new(vec![Pool::new(PoolKind::Health, 20, 0, 20)]),
            ))
            .id();

        app.world_mut()
            .resource_mut::<bevy_ecs::message::Messages<ActionIntent>>()
            .write(ActionIntent {
                actor,
                action_id: "ability.quick_attack".into(),
                direction: None,
                target: Some(target),
            });
        app.update();
        app.update();

        let progression = app.world().get::<SkillProgression>(actor).unwrap();
        assert_eq!(progression.melee, 1);
        let thumos = app
            .world()
            .get::<Pools>(actor)
            .unwrap()
            .get(PoolKind::Thumos)
            .unwrap();
        assert_eq!(thumos.current, 1);
    }

    #[test]
    fn rejected_action_does_not_grant_progression() {
        let mut app = test_app();
        app.insert_resource(SmokeMap::new(10, 10, crate::components::Tile::Floor));
        let actor = app
            .world_mut()
            .spawn((
                Player,
                crate::spatial::EntityScope::RunPersistent,
                Position { x: 5, y: 5 },
                SkillProgression::default(),
                Pools::new(vec![Pool::new(PoolKind::ActionPoints, 3, 0, 3)]),
            ))
            .id();
        app.world_mut()
            .resource_mut::<bevy_ecs::message::Messages<ActionIntent>>()
            .write(ActionIntent {
                actor,
                action_id: "ability.quick_attack".into(),
                direction: None,
                target: None,
            });
        app.update();
        app.update();
        assert_eq!(app.world().get::<SkillProgression>(actor).unwrap().melee, 0);
    }
}
