use bevy::prelude::*;
use rand::RngExt;

use crate::core::components::{Player, Position};
use crate::core::gamelog::{GameLog, LogColor};
use crate::core::inventory::Inventory;
use crate::core::items::{ItemDrop, find_item};
use crate::game::dungeon::bsp::Rect;

// ── Tiered Loot Table ─────────────────────────────────────────

struct LootEntry {
    item_id: &'static str,
    qty_min: u8,
    qty_max: u8,
    weight: u32,
    tier: u8,
}

const LOOT_TABLE: &[LootEntry] = &[
    // Tier 0 — common basics
    LootEntry {
        item_id: "ammo",
        qty_min: 2,
        qty_max: 5,
        weight: 30,
        tier: 0,
    },
    LootEntry {
        item_id: "scrap",
        qty_min: 1,
        qty_max: 3,
        weight: 25,
        tier: 0,
    },
    LootEntry {
        item_id: "raw_meat",
        qty_min: 1,
        qty_max: 1,
        weight: 15,
        tier: 0,
    },
    LootEntry {
        item_id: "dirty_water",
        qty_min: 1,
        qty_max: 1,
        weight: 15,
        tier: 0,
    },
    // Tier 1 — better consumables
    LootEntry {
        item_id: "medicine",
        qty_min: 1,
        qty_max: 1,
        weight: 10,
        tier: 1,
    },
    LootEntry {
        item_id: "ammo",
        qty_min: 5,
        qty_max: 10,
        weight: 12,
        tier: 1,
    },
    // Tier 2 — mid weapons / gear
    LootEntry {
        item_id: "hunting_knife",
        qty_min: 1,
        qty_max: 1,
        weight: 3,
        tier: 2,
    },
    LootEntry {
        item_id: "scrap_vest",
        qty_min: 1,
        qty_max: 1,
        weight: 3,
        tier: 2,
    },
    // Tier 3 — rare weapons
    LootEntry {
        item_id: "makeshift_pistol",
        qty_min: 1,
        qty_max: 1,
        weight: 2,
        tier: 3,
    },
    LootEntry {
        item_id: "military_rifle",
        qty_min: 1,
        qty_max: 1,
        weight: 1,
        tier: 3,
    },
];

/// Returns (min_tier, max_tier) for a given floor number.
fn tier_range_for_floor(floor_number: u32) -> (u8, u8) {
    match floor_number {
        1..=2 => (0, 1),
        3..=4 => (1, 2),
        _ => (2, 3),
    }
}

fn pick_loot(rng: &mut impl rand::Rng, floor_number: u32) -> (&'static str, u8) {
    let (min_tier, max_tier) = tier_range_for_floor(floor_number);

    let total: u32 = LOOT_TABLE
        .iter()
        .filter(|e| e.tier >= min_tier && e.tier <= max_tier)
        .map(|e| e.weight)
        .sum();

    if total == 0 {
        return (LOOT_TABLE[0].item_id, LOOT_TABLE[0].qty_min);
    }

    let mut roll = rng.random_range(0..total);
    for entry in LOOT_TABLE
        .iter()
        .filter(|e| e.tier >= min_tier && e.tier <= max_tier)
    {
        if roll < entry.weight {
            let qty = if entry.qty_min == entry.qty_max {
                entry.qty_min
            } else {
                rng.random_range(entry.qty_min..=entry.qty_max)
            };
            return (entry.item_id, qty);
        }
        roll -= entry.weight;
    }
    // Fallback (shouldn't happen)
    (LOOT_TABLE[0].item_id, LOOT_TABLE[0].qty_min)
}

// ── Spawn Helper ─────────────────────────────────────────────

/// Spawns loot items in dungeon rooms. Called from `dungeon/spawn.rs` during setup.
/// Room 0 is the player spawn and is skipped.
pub fn spawn_loot_in_rooms(
    commands: &mut Commands,
    rooms: &[Rect],
    rng: &mut impl rand::Rng,
    floor_number: u32,
) {
    for room in rooms.iter().skip(1) {
        // 60% chance to spawn loot in this room
        if rng.random_range(0..100) >= 60 {
            continue;
        }

        let item_count = rng.random_range(1..=2u8);
        for _ in 0..item_count {
            let x = rng.random_range(room.x..(room.x + room.w));
            let y = rng.random_range(room.y..(room.y + room.h));
            let (item_id, quantity) = pick_loot(rng, floor_number);

            commands.spawn((
                ItemDrop {
                    item_id: item_id.to_string(),
                    quantity,
                },
                Position { x, y },
            ));
        }
    }
}

// ── Pickup System ────────────────────────────────────────────

pub fn pickup_items(
    mut commands: Commands,
    mut player_query: Query<(&Position, &mut Inventory), With<Player>>,
    items_query: Query<(Entity, &ItemDrop, &Position), Without<Player>>,
    mut log: ResMut<GameLog>,
    time: Res<crate::core::turn::GameTime>,
) {
    let Ok((player_pos, mut inventory)) = player_query.single_mut() else {
        return;
    };

    let mut inventory_full_warned = false;

    for (entity, drop, drop_pos) in items_query.iter() {
        if drop_pos != player_pos {
            continue;
        }

        if inventory.try_add(&drop.item_id, drop.quantity).is_ok() {
            let item_name = find_item(&drop.item_id)
                .map(|d| d.name)
                .unwrap_or(&drop.item_id);
            log.push(
                format!("Picked up {}x {}", drop.quantity, item_name),
                LogColor::Critical, // yellow — loot
                time.turn,
            );
            commands.entity(entity).despawn();
        } else if !inventory_full_warned {
            log.push("Inventory full!", LogColor::EnemyHit, time.turn); // orange-ish — warning
            inventory_full_warned = true;
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_tier_range() {
        assert_eq!(tier_range_for_floor(1), (0, 1));
        assert_eq!(tier_range_for_floor(2), (0, 1));
        assert_eq!(tier_range_for_floor(3), (1, 2));
        assert_eq!(tier_range_for_floor(4), (1, 2));
        assert_eq!(tier_range_for_floor(5), (2, 3));
    }

    #[test]
    fn test_pick_loot_floor_1() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        for _ in 0..100 {
            let (item_id, _qty) = pick_loot(&mut rng, 1);
            let entry = LOOT_TABLE.iter().find(|e| e.item_id == item_id).unwrap();
            assert!(
                entry.tier <= 1,
                "Floor 1 produced tier {} item '{}'",
                entry.tier,
                item_id
            );
        }
    }

    #[test]
    fn test_pick_loot_floor_5() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        for _ in 0..100 {
            let (item_id, _qty) = pick_loot(&mut rng, 5);
            let entry = LOOT_TABLE.iter().find(|e| e.item_id == item_id).unwrap();
            assert!(
                entry.tier >= 2,
                "Floor 5 produced tier {} item '{}'",
                entry.tier,
                item_id
            );
        }
    }

    #[test]
    fn test_all_loot_ids_valid() {
        for entry in LOOT_TABLE {
            assert!(
                crate::core::items::find_item(entry.item_id).is_some(),
                "Loot table references unknown item '{}'",
                entry.item_id
            );
        }
    }
}
