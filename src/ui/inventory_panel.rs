//! Toggle-able inventory panel — press I to open/close.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::core::components::Player;
use crate::core::inventory::{Equipment, Inventory};
use crate::core::items::{find_item, ItemKind};
use crate::core::state::AppState;
use crate::core::turn::{PendingAction, PlayerAction, TurnPhase};

/// Whether the inventory panel is currently open.
#[derive(Resource, Default)]
pub struct InventoryOpen(pub bool);

// ---------------------------------------------------------------------------
// Action resource
// ---------------------------------------------------------------------------

#[derive(Resource, Default)]
pub struct InventoryUiAction(pub Option<InventoryUiChoice>);

#[derive(Clone, Copy, Debug)]
pub enum InventoryUiChoice {
    Close,
    UseItem(usize), // inventory slot index
}

/// Toggle inventory visibility when I is pressed.
pub fn toggle_inventory(keys: Res<ButtonInput<KeyCode>>, mut open: ResMut<InventoryOpen>) {
    if keys.just_pressed(KeyCode::KeyI) {
        open.0 = !open.0;
    }
}

// ---------------------------------------------------------------------------
// Draw — EguiPrimaryContextPass (read-only)
// ---------------------------------------------------------------------------

/// Draw the inventory window when open.
pub fn draw_inventory_panel(
    mut contexts: EguiContexts,
    state: Res<State<AppState>>,
    open: Res<InventoryOpen>,
    query: Query<(&Inventory, Option<&Equipment>), With<Player>>,
    mut action: ResMut<InventoryUiAction>,
) {
    if *state.get() != AppState::Dungeon || !open.0 {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let Ok((inventory, equipment)) = query.single() else {
        return;
    };

    let used = inventory.slots.iter().filter(|s| s.is_some()).count();
    let max = inventory.slots.len();

    egui::Window::new("Inventory")
        .collapsible(false)
        .resizable(false)
        .default_width(280.0)
        .show(ctx, |ui| {
            ui.label(
                egui::RichText::new(format!("{}/{} slots", used, max))
                    .color(egui::Color32::from_rgb(180, 180, 180)),
            );
            ui.separator();

            // Item list
            for (slot_idx, slot) in inventory.slots.iter().enumerate() {
                let Some(stack) = slot else { continue };
                let (prefix, name, is_consumable) = if let Some(def) = find_item(&stack.item_id) {
                    let icon = match def.kind {
                        ItemKind::Weapon => "⚔ ",
                        ItemKind::Armor => "🛡 ",
                        ItemKind::Consumable => "💊 ",
                        ItemKind::Resource => "📦 ",
                    };
                    (icon, def.name, def.consumable.is_some())
                } else {
                    ("? ", stack.item_id.as_str(), false)
                };

                ui.horizontal(|ui| {
                    if stack.quantity > 1 {
                        ui.label(format!("{}{} x{}", prefix, name, stack.quantity));
                    } else {
                        ui.label(format!("{}{}", prefix, name));
                    }
                    if is_consumable && ui.small_button("Use").clicked() {
                        action.0 = Some(InventoryUiChoice::UseItem(slot_idx));
                    }
                });
            }

            if used == 0 {
                ui.label(
                    egui::RichText::new("Empty")
                        .color(egui::Color32::from_rgb(120, 120, 120))
                        .italics(),
                );
            }

            // Equipment section
            ui.separator();
            ui.label(egui::RichText::new("Equipment").strong());

            if let Some(equip) = equipment {
                let weapon_name = equip
                    .weapon
                    .as_ref()
                    .and_then(|id| find_item(id))
                    .map(|d| d.name)
                    .unwrap_or("—");
                let armor_name = equip
                    .armor
                    .as_ref()
                    .and_then(|id| find_item(id))
                    .map(|d| d.name)
                    .unwrap_or("—");
                let accessory_name = equip
                    .accessory
                    .as_ref()
                    .and_then(|id| find_item(id))
                    .map(|d| d.name)
                    .unwrap_or("—");

                ui.label(format!("⚔ Weapon: {}", weapon_name));
                ui.label(format!("🛡 Armor:  {}", armor_name));
                ui.label(format!("💎 Acc:    {}", accessory_name));
            } else {
                ui.label("No equipment data");
            }

            ui.separator();
            if ui.button("Close").clicked() {
                action.0 = Some(InventoryUiChoice::Close);
            }
        });
}

// ---------------------------------------------------------------------------
// Process — Update (mutations)
// ---------------------------------------------------------------------------

pub fn process_inventory_action(
    mut action: ResMut<InventoryUiAction>,
    mut open: ResMut<InventoryOpen>,
    mut player_action: ResMut<PlayerAction>,
    app_state: Res<State<AppState>>,
    turn_phase: Res<State<TurnPhase>>,
    mut next_turn_phase: ResMut<NextState<TurnPhase>>,
) {
    let Some(choice) = action.0.take() else { return; };

    match choice {
        InventoryUiChoice::Close => {
            open.0 = false;
        }
        InventoryUiChoice::UseItem(slot_idx) => {
            if *app_state.get() == AppState::Dungeon
                && *turn_phase.get() == TurnPhase::AwaitingInput
            {
                player_action.0 = Some(PendingAction::UseItem(slot_idx));
                next_turn_phase.set(TurnPhase::PlayerTurn);
            }
        }
    }
}
