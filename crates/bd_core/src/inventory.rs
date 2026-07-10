//! Inventory, equipment, containers, and item interactions.
//!
//! All item operations go through the intent pipeline.
//! Items are entities with relationship components.

use bevy_app::App;
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    BdSet,
    components::{Name, Position},
    gamelog::{GameLog, LogLevel},
    relationships::{ContainedIn, EquippedBy},
    signals::{DeltaTag, PoolDeltaRequested, PoolKind},
};

// ── Components ──

/// Marks an entity as an item.
#[derive(Component, Debug, Clone, Default, Serialize, Deserialize)]
pub struct Item;

/// An entity that can hold items.
#[derive(Component, Debug, Clone, Default, Serialize, Deserialize)]
pub struct Container {
    pub capacity: Option<i32>,
    pub allowed_tags: Vec<String>,
}

/// Equipment slot kind.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlotKind {
    Weapon,
    Armor,
    Relic,
    Accessory,
}

/// An equipment slot on an entity.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct EquipmentSlot {
    pub kind: SlotKind,
    pub accepted_tags: Vec<String>,
}

/// An item can be used to produce effects.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Usable {
    pub consume_on_use: bool,
    pub effects: Vec<UseEffect>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UseEffect {
    Heal(i32),
    RestoreAp(i32),
    Log(String),
}

// ── Intents ──

#[derive(Message, Debug, Clone)]
pub struct PickupIntent {
    pub actor: Entity,
    pub item: Entity,
}

#[derive(Message, Debug, Clone)]
pub struct DropIntent {
    pub actor: Entity,
    pub item: Entity,
}

#[derive(Message, Debug, Clone)]
pub struct EquipIntent {
    pub actor: Entity,
    pub item: Entity,
    pub slot: SlotKind,
}

#[derive(Message, Debug, Clone)]
pub struct UnequipIntent {
    pub actor: Entity,
    pub item: Entity,
}

#[derive(Message, Debug, Clone)]
pub struct UseItemIntent {
    pub actor: Entity,
    pub item: Entity,
}

// ── Plugin ──

pub(crate) fn register_inventory(app: &mut App) {
    app.add_message::<PickupIntent>();
    app.add_message::<DropIntent>();
    app.add_message::<EquipIntent>();
    app.add_message::<UnequipIntent>();
    app.add_message::<UseItemIntent>();
    app.add_systems(
        bevy_app::Update,
        (
            process_pickup,
            process_drop,
            process_equip,
            process_unequip,
            process_use_item,
        )
            .in_set(BdSet::Mutation),
    );
}

// ── Systems ──

#[allow(clippy::type_complexity)]
fn process_pickup(
    mut commands: Commands,
    mut messages: bevy_ecs::message::MessageReader<PickupIntent>,
    mut game_log: ResMut<GameLog>,
    _actors: Query<&Name>,
    containers: Query<&Container>,
    #[allow(clippy::type_complexity)] items: Query<
        (Entity, &Name),
        (With<Item>, Without<Container>),
    >,
) {
    for intent in messages.read() {
        if !containers.contains(intent.actor) {
            continue;
        }
        if !items.contains(intent.item) {
            continue;
        }
        commands.entity(intent.item).remove::<Position>();
        commands
            .entity(intent.item)
            .insert(ContainedIn(intent.actor));
        if let Ok(name) = items.get(intent.item).map(|(_, n)| n) {
            game_log.push(format!("Picked up {}.", name.0), LogLevel::Info);
        }
    }
}

fn process_drop(
    mut commands: Commands,
    mut messages: bevy_ecs::message::MessageReader<DropIntent>,
    mut game_log: ResMut<GameLog>,
    actor_pos: Query<&Position>,
    items: Query<(Entity, &Name), With<ContainedIn>>,
) {
    for intent in messages.read() {
        let Ok(pos) = actor_pos.get(intent.actor) else {
            continue;
        };
        commands.entity(intent.item).remove::<ContainedIn>();
        commands.entity(intent.item).insert(Position {
            x: pos.x + 1,
            y: pos.y,
        });
        if let Ok((_, name)) = items.get(intent.item) {
            game_log.push(format!("Dropped {}.", name.0), LogLevel::Info);
        }
    }
}

fn process_equip(
    mut commands: Commands,
    mut messages: bevy_ecs::message::MessageReader<EquipIntent>,
    mut game_log: ResMut<GameLog>,
    items: Query<(Entity, &Name), With<ContainedIn>>,
) {
    for intent in messages.read() {
        if !items.contains(intent.item) {
            continue;
        }
        commands.entity(intent.item).remove::<ContainedIn>();
        commands
            .entity(intent.item)
            .insert(EquippedBy(intent.actor));
        if let Ok((_, name)) = items.get(intent.item) {
            game_log.push(format!("Equipped {}.", name.0), LogLevel::Info);
        }
    }
}

fn process_unequip(
    mut commands: Commands,
    mut messages: bevy_ecs::message::MessageReader<UnequipIntent>,
    mut game_log: ResMut<GameLog>,
    items: Query<(Entity, &Name), With<EquippedBy>>,
) {
    for intent in messages.read() {
        if !items.contains(intent.item) {
            continue;
        }
        commands.entity(intent.item).remove::<EquippedBy>();
        commands
            .entity(intent.item)
            .insert(ContainedIn(intent.actor));
        if let Ok((_, name)) = items.get(intent.item) {
            game_log.push(format!("Unequipped {}.", name.0), LogLevel::Info);
        }
    }
}

fn process_use_item(
    mut commands: Commands,
    mut messages: bevy_ecs::message::MessageReader<UseItemIntent>,
    mut game_log: ResMut<GameLog>,
    mut delta_writer: bevy_ecs::message::MessageWriter<PoolDeltaRequested>,
    items: Query<(Entity, &Name, &Usable, &ContainedIn)>,
) {
    for intent in messages.read() {
        let Ok((_, name, usable, contained)) = items.get(intent.item) else {
            continue;
        };
        if contained.0 != intent.actor {
            continue;
        }
        for effect in &usable.effects {
            match effect {
                UseEffect::Heal(amount) => {
                    delta_writer.write(PoolDeltaRequested {
                        source: Some(intent.item),
                        target: intent.actor,
                        kind: PoolKind::Health,
                        amount: *amount,
                        tags: vec![DeltaTag::Recovery],
                        reason: format!("used {}", name.0),
                    });
                }
                UseEffect::RestoreAp(amount) => {
                    delta_writer.write(PoolDeltaRequested {
                        source: Some(intent.item),
                        target: intent.actor,
                        kind: PoolKind::ActionPoints,
                        amount: *amount,
                        tags: vec![DeltaTag::Recovery],
                        reason: format!("used {}", name.0),
                    });
                }
                UseEffect::Log(msg) => {
                    game_log.push(msg.clone(), LogLevel::Info);
                }
            }
        }
        if usable.consume_on_use {
            game_log.push(format!("Used {}. It's gone.", name.0), LogLevel::Info);
            commands.entity(intent.item).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Tile;
    use crate::map::SmokeMap;
    use crate::pools::{Pool, Pools};
    use bevy_app::App;
    use bevy_ecs::message::Messages;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(crate::BdCorePlugin);
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        app
    }

    fn send_pickup(app: &mut App, actor: Entity, item: Entity) {
        app.world_mut()
            .resource_mut::<Messages<PickupIntent>>()
            .write(PickupIntent { actor, item });
    }

    #[test]
    fn pickup_moves_item_into_container() {
        let mut app = test_app();
        let player = app
            .world_mut()
            .spawn((Container::default(), Name("Player".into())))
            .id();
        let potion = app
            .world_mut()
            .spawn((Item, Name("Potion".into()), Position { x: 5, y: 5 }))
            .id();
        send_pickup(&mut app, player, potion);
        app.update();
        assert!(app.world().get::<ContainedIn>(potion).is_some());
        assert!(app.world().get::<Position>(potion).is_none());
    }

    #[test]
    fn drop_moves_item_to_map() {
        let mut app = test_app();
        let player = app
            .world_mut()
            .spawn((
                Container::default(),
                Name("Player".into()),
                Position { x: 5, y: 5 },
            ))
            .id();
        let potion = app
            .world_mut()
            .spawn((Item, Name("Potion".into()), ContainedIn(player)))
            .id();
        app.world_mut()
            .resource_mut::<Messages<DropIntent>>()
            .write(DropIntent {
                actor: player,
                item: potion,
            });
        app.update();
        assert!(app.world().get::<ContainedIn>(potion).is_none());
        assert!(app.world().get::<Position>(potion).is_some());
    }

    #[test]
    fn equip_moves_item_to_slot() {
        let mut app = test_app();
        let player = app
            .world_mut()
            .spawn((Container::default(), Name("Player".into())))
            .id();
        let sword = app
            .world_mut()
            .spawn((Item, Name("Sword".into()), ContainedIn(player)))
            .id();
        app.world_mut()
            .resource_mut::<Messages<EquipIntent>>()
            .write(EquipIntent {
                actor: player,
                item: sword,
                slot: SlotKind::Weapon,
            });
        app.update();
        assert!(app.world().get::<EquippedBy>(sword).is_some());
        assert!(app.world().get::<ContainedIn>(sword).is_none());
    }

    #[test]
    fn unequip_returns_item_to_inventory() {
        let mut app = test_app();
        let player = app
            .world_mut()
            .spawn((Container::default(), Name("Player".into())))
            .id();
        let sword = app
            .world_mut()
            .spawn((Item, Name("Sword".into()), EquippedBy(player)))
            .id();
        app.world_mut()
            .resource_mut::<Messages<UnequipIntent>>()
            .write(UnequipIntent {
                actor: player,
                item: sword,
            });
        app.update();
        assert!(app.world().get::<ContainedIn>(sword).is_some());
        assert!(app.world().get::<EquippedBy>(sword).is_none());
    }

    #[test]
    fn use_item_emits_effects() {
        let mut app = test_app();
        let player = app
            .world_mut()
            .spawn((
                Container::default(),
                Name("Player".into()),
                Pools::new(vec![Pool::new(PoolKind::Health, 10, 0, 20)]),
            ))
            .id();
        let potion = app
            .world_mut()
            .spawn((
                Item,
                Name("Potion".into()),
                ContainedIn(player),
                Usable {
                    consume_on_use: true,
                    effects: vec![UseEffect::Heal(5)],
                },
            ))
            .id();
        app.world_mut()
            .resource_mut::<Messages<UseItemIntent>>()
            .write(UseItemIntent {
                actor: player,
                item: potion,
            });
        app.update();
        app.update(); // second frame to process pool deltas
        let hp = app
            .world()
            .get::<Pools>(player)
            .unwrap()
            .get(PoolKind::Health)
            .unwrap()
            .current;
        assert_eq!(hp, 15);
    }

    #[test]
    fn consumable_item_is_removed_after_use() {
        let mut app = test_app();
        let player = app
            .world_mut()
            .spawn((
                Container::default(),
                Name("Player".into()),
                Pools::new(vec![Pool::new(PoolKind::Health, 10, 0, 20)]),
            ))
            .id();
        let potion = app
            .world_mut()
            .spawn((
                Item,
                Name("Potion".into()),
                ContainedIn(player),
                Usable {
                    consume_on_use: true,
                    effects: vec![UseEffect::Heal(5)],
                },
            ))
            .id();
        app.world_mut()
            .resource_mut::<Messages<UseItemIntent>>()
            .write(UseItemIntent {
                actor: player,
                item: potion,
            });
        app.update();
        assert!(app.world().get_entity(potion).is_err());
    }

    #[test]
    fn container_view_model_lists_items() {
        let mut app = test_app();
        let player = app
            .world_mut()
            .spawn((Container::default(), Name("Player".into())))
            .id();
        app.world_mut()
            .spawn((Item, Name("Sword".into()), ContainedIn(player)));
        app.world_mut()
            .spawn((Item, Name("Shield".into()), ContainedIn(player)));
        let contained = crate::relationships::items_in_container(player, app.world_mut());
        assert_eq!(contained.len(), 2);
    }
}
