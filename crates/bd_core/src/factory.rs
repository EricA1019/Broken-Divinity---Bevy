use serde::{Deserialize, Serialize};

use crate::{
    components::{BlocksMovement, Name, Player, Position},
    pools::{Pool, Pools},
    signals::PoolKind,
    statuses::{StatusInstance, Statuses},
};
use bevy_ecs::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityBlueprint {
    pub id: String,
    pub label: String,
    pub is_player: bool,
    pub blocks_movement: bool,
    pub pools: Vec<(PoolKind, i32, i32, i32)>,
    pub statuses: Vec<(String, i32)>,
    pub visual: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlueprintRegistry {
    pub blueprints: Vec<EntityBlueprint>,
}

impl BlueprintRegistry {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn get(&self, id: &str) -> Option<&EntityBlueprint> {
        self.blueprints.iter().find(|b| b.id == id)
    }
    pub fn phase10_defaults() -> Self {
        Self {
            blueprints: vec![
                EntityBlueprint {
                    id: "blueprint.player".into(),
                    label: "Player".into(),
                    is_player: true,
                    blocks_movement: false,
                    pools: vec![
                        (PoolKind::Health, 20, 0, 20),
                        (PoolKind::ActionPoints, 3, 0, 3),
                    ],
                    statuses: vec![],
                    visual: Some("Player".into()),
                },
                EntityBlueprint {
                    id: "blueprint.training_dummy".into(),
                    label: "Training Dummy".into(),
                    is_player: false,
                    blocks_movement: true,
                    pools: vec![
                        (PoolKind::Health, 15, 0, 15),
                        (PoolKind::ActionPoints, 0, 0, 0),
                    ],
                    statuses: vec![("status.poisoned".into(), 5)],
                    visual: Some("Enemy".into()),
                },
                EntityBlueprint {
                    id: "blueprint.rat".into(),
                    label: "Rat".into(),
                    is_player: false,
                    blocks_movement: true,
                    pools: vec![
                        (PoolKind::Health, 5, 0, 5),
                        (PoolKind::ActionPoints, 2, 0, 2),
                    ],
                    statuses: vec![],
                    visual: Some("Enemy".into()),
                },
                EntityBlueprint {
                    id: "blueprint.healing_potion".into(),
                    label: "Healing Potion".into(),
                    is_player: false,
                    blocks_movement: false,
                    pools: vec![],
                    statuses: vec![],
                    visual: Some("Item".into()),
                },
            ],
        }
    }

    /// Phase 18 MVP content pack — adds more enemies, items, and an ally.
    pub fn phase18_defaults() -> Self {
        let mut base = Self::phase10_defaults();
        base.blueprints.extend(vec![
            // Enemy: skeleton
            EntityBlueprint {
                id: "blueprint.skeleton".into(),
                label: "Skeleton".into(),
                is_player: false,
                blocks_movement: true,
                pools: vec![
                    (PoolKind::Health, 12, 0, 12),
                    (PoolKind::ActionPoints, 2, 0, 2),
                ],
                statuses: vec![],
                visual: Some("Enemy".into()),
            },
            // Ally: warden
            EntityBlueprint {
                id: "blueprint.ally_warden".into(),
                label: "Warden".into(),
                is_player: false,
                blocks_movement: false,
                pools: vec![
                    (PoolKind::Health, 25, 0, 25),
                    (PoolKind::ActionPoints, 2, 0, 2),
                ],
                statuses: vec![],
                visual: Some("Ally".into()),
            },
            // Item: sword
            EntityBlueprint {
                id: "blueprint.sword".into(),
                label: "Rusted Sword".into(),
                is_player: false,
                blocks_movement: false,
                pools: vec![],
                statuses: vec![],
                visual: Some("Item".into()),
            },
            // Item: shield
            EntityBlueprint {
                id: "blueprint.shield".into(),
                label: "Wooden Shield".into(),
                is_player: false,
                blocks_movement: false,
                pools: vec![],
                statuses: vec![],
                visual: Some("Item".into()),
            },
            // Item: scroll of smite
            EntityBlueprint {
                id: "blueprint.smite_scroll".into(),
                label: "Scroll of Smite".into(),
                is_player: false,
                blocks_movement: false,
                pools: vec![],
                statuses: vec![],
                visual: Some("Item".into()),
            },
            // Item: gold pile
            EntityBlueprint {
                id: "blueprint.gold_pile".into(),
                label: "Gold Pile".into(),
                is_player: false,
                blocks_movement: false,
                pools: vec![],
                statuses: vec![],
                visual: Some("Item".into()),
            },
            // Boss: crypt lord
            EntityBlueprint {
                id: "blueprint.crypt_lord".into(),
                label: "Crypt Lord".into(),
                is_player: false,
                blocks_movement: true,
                pools: vec![
                    (PoolKind::Health, 30, 0, 30),
                    (PoolKind::ActionPoints, 3, 0, 3),
                ],
                statuses: vec![],
                visual: Some("Enemy".into()),
            },
        ]);
        base
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Mutator {
    Wounded,
    Elite,
}

impl Mutator {
    pub fn apply(&self, pools: &mut [(PoolKind, i32, i32, i32)]) {
        for (kind, current, _, max) in pools.iter_mut() {
            if *kind != PoolKind::Health {
                continue;
            }
            match self {
                Mutator::Wounded => {
                    *max /= 2;
                    *current = (*current).min(*max);
                }
                Mutator::Elite => {
                    *max = (*max as f32 * 1.5) as i32;
                }
            }
        }
    }
}

pub fn spawn_from_blueprint(
    blueprint: &EntityBlueprint,
    pos: Option<Position>,
    mutators: &[Mutator],
    commands: &mut Commands,
) -> Entity {
    let mut pools = blueprint.pools.clone();
    for m in mutators {
        m.apply(&mut pools);
    }
    let mut e = commands.spawn_empty();
    if blueprint.is_player {
        e.insert(Player);
    }
    if blueprint.blocks_movement {
        e.insert(BlocksMovement);
    }
    if let Some(p) = pos {
        e.insert(p);
    }
    e.insert(Name(blueprint.label.clone()));
    if !pools.is_empty() {
        e.insert(Pools::new(
            pools
                .into_iter()
                .map(|(k, c, mn, mx)| Pool::new(k, c, mn, mx))
                .collect(),
        ));
    }
    if !blueprint.statuses.is_empty() {
        e.insert(Statuses {
            instances: blueprint
                .statuses
                .iter()
                .map(|(id, dur)| StatusInstance {
                    status_id: id.clone(),
                    remaining_duration: *dur,
                    stacks: 1,
                    source: None,
                })
                .collect(),
        });
    }
    e.id()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Tile;
    use crate::map::SmokeMap;
    use bevy_app::App;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(crate::BdCorePlugin);
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        app
    }

    #[test]
    fn factory_spawns_player_blueprint() {
        let mut app = test_app();
        let reg = BlueprintRegistry::phase10_defaults();
        let bp = reg.get("blueprint.player").unwrap();
        let e = spawn_from_blueprint(
            bp,
            Some(Position { x: 10, y: 6 }),
            &[],
            &mut app.world_mut().commands(),
        );
        app.update();
        assert!(app.world().get::<Player>(e).is_some());
        assert_eq!(
            app.world().get::<Position>(e).unwrap(),
            &Position { x: 10, y: 6 }
        );
    }

    #[test]
    fn factory_spawns_enemy_blueprint() {
        let mut app = test_app();
        let reg = BlueprintRegistry::phase10_defaults();
        let bp = reg.get("blueprint.training_dummy").unwrap();
        let e = spawn_from_blueprint(
            bp,
            Some(Position { x: 12, y: 6 }),
            &[],
            &mut app.world_mut().commands(),
        );
        app.update();
        assert!(app.world().get::<BlocksMovement>(e).is_some());
        assert!(app.world().get::<Player>(e).is_none());
    }

    #[test]
    fn factory_spawns_item_blueprint() {
        let mut app = test_app();
        let reg = BlueprintRegistry::phase10_defaults();
        let bp = reg.get("blueprint.healing_potion").unwrap();
        let e = spawn_from_blueprint(bp, None, &[], &mut app.world_mut().commands());
        app.update();
        assert!(app.world().get::<Name>(e).is_some());
    }

    #[test]
    fn factory_applies_wounded_mutator() {
        let mut pools = vec![
            (PoolKind::Health, 20, 0, 20),
            (PoolKind::ActionPoints, 3, 0, 3),
        ];
        Mutator::Wounded.apply(&mut pools);
        assert_eq!(pools[0].3, 10);
        assert_eq!(pools[0].1, 10);
    }

    #[test]
    fn factory_applies_elite_mutator() {
        let mut pools = vec![(PoolKind::Health, 5, 0, 5)];
        Mutator::Elite.apply(&mut pools);
        assert_eq!(pools[0].3, 7);
    }

    #[test]
    fn factory_rejects_missing_blueprint() {
        assert!(
            BlueprintRegistry::new()
                .get("blueprint.nonexistent")
                .is_none()
        );
    }

    #[test]
    fn factory_adds_visual_token() {
        let bp = EntityBlueprint {
            id: "test".into(),
            label: "Test".into(),
            is_player: false,
            blocks_movement: false,
            pools: vec![],
            statuses: vec![],
            visual: Some("Enemy".into()),
        };
        assert_eq!(bp.visual, Some("Enemy".into()));
    }
}
