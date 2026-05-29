//! Toggle-able inventory panel — press I to open/close.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::core::components::Player;
use crate::core::inventory::{Equipment, Inventory};
use crate::core::items::{ItemKind, find_item};
use crate::core::state::AppState;
use crate::core::turn::{PendingAction, PlayerAction, TurnPhase};
use crate::ui::input_hints::{
    INVENTORY_TOGGLE_HINT_TEXT, INVENTORY_TOGGLE_PRIMARY_KEY,
    INVENTORY_TOGGLE_SECONDARY_KEY,
};
use crate::ui::inventory_rules::{
    EquipOutcome, EquipmentSlot, InventoryRuleError, equip_from_inventory_slot,
    resolve_equipment_slot, unequip_to_inventory_slot,
};

const INVENTORY_WINDOW_WIDTH: f32 = 280.0;
const STATUS_NEUTRAL_RGB: (u8, u8, u8) = (180, 180, 180);
const STATUS_SUCCESS_RGB: (u8, u8, u8) = (120, 210, 130);
const STATUS_WARNING_RGB: (u8, u8, u8) = (220, 170, 90);
const STATUS_EQUIP_OK: &str = "Item equipped.";
const STATUS_SWAP_OK: &str = "Item equipped; previous gear moved to inventory.";
const STATUS_UNEQUIP_OK: &str = "Item returned to inventory.";
const STATUS_NO_SPACE: &str = "Action blocked: inventory is full.";
const STATUS_NOT_EQUIPPABLE: &str = "Action blocked: item cannot be equipped.";
const STATUS_INVALID_SLOT: &str = "Action blocked: inventory slot is invalid.";
const STATUS_NOTHING_EQUIPPED: &str = "Action blocked: no item in that equipment slot.";
const STATUS_MISSING_EQUIPMENT: &str = "Action blocked: equipment data unavailable.";

/// Whether the inventory panel is currently open.
#[derive(Resource, Default)]
pub struct InventoryOpen(pub bool);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InventoryStatusTone {
    Neutral,
    Success,
    Warning,
}

#[derive(Resource, Default)]
pub struct InventoryUiStatus {
    message: Option<String>,
    tone: Option<InventoryStatusTone>,
}

impl InventoryUiStatus {
    fn clear(&mut self) {
        self.message = None;
        self.tone = None;
    }

    fn set(&mut self, message: impl Into<String>, tone: InventoryStatusTone) {
        self.message = Some(message.into());
        self.tone = Some(tone);
    }
}

// ---------------------------------------------------------------------------
// Action resource
// ---------------------------------------------------------------------------

#[derive(Resource, Default)]
pub struct InventoryUiAction(pub Option<InventoryUiChoice>);

#[derive(Clone, Copy, Debug)]
pub enum InventoryUiChoice {
    Close,
    UseItem(usize), // inventory slot index
    EquipFromInventory(usize),
    Unequip(EquipmentSlot),
}

/// Toggle inventory visibility when I is pressed.
pub fn toggle_inventory(keys: Res<ButtonInput<KeyCode>>, mut open: ResMut<InventoryOpen>) {
    if keys.just_pressed(INVENTORY_TOGGLE_PRIMARY_KEY)
        || keys.just_pressed(INVENTORY_TOGGLE_SECONDARY_KEY)
    {
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
    status: Res<InventoryUiStatus>,
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
        .default_width(INVENTORY_WINDOW_WIDTH)
        .show(ctx, |ui| {
            if let Some(status_message) = status.message.as_deref() {
                ui.label(
                    egui::RichText::new(status_message).color(status_color(&status)),
                );
                ui.separator();
            }

            ui.label(
                egui::RichText::new(format!("{}/{} slots", used, max))
                    .color(rgb(STATUS_NEUTRAL_RGB)),
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

                    if resolve_equipment_slot(&stack.item_id).is_ok()
                        && ui.small_button("Equip").clicked()
                    {
                        action.0 = Some(InventoryUiChoice::EquipFromInventory(slot_idx));
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
                if equip.weapon.is_some() && ui.small_button("Unequip Weapon").clicked() {
                    action.0 = Some(InventoryUiChoice::Unequip(EquipmentSlot::Weapon));
                }
                ui.label(format!("🛡 Armor:  {}", armor_name));
                if equip.armor.is_some() && ui.small_button("Unequip Armor").clicked() {
                    action.0 = Some(InventoryUiChoice::Unequip(EquipmentSlot::Armor));
                }
                ui.label(format!("💎 Acc:    {}", accessory_name));
                if equip.accessory.is_some() && ui.small_button("Unequip Accessory").clicked() {
                    action.0 = Some(InventoryUiChoice::Unequip(EquipmentSlot::Accessory));
                }
            } else {
                ui.label("No equipment data");
            }

            ui.separator();
            ui.label(egui::RichText::new(INVENTORY_TOGGLE_HINT_TEXT).color(rgb(STATUS_NEUTRAL_RGB)));
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
    mut status: ResMut<InventoryUiStatus>,
    mut player_action: ResMut<PlayerAction>,
    app_state: Res<State<AppState>>,
    turn_phase: Res<State<TurnPhase>>,
    mut next_turn_phase: ResMut<NextState<TurnPhase>>,
    mut query: Query<(&mut Inventory, Option<&mut Equipment>), With<Player>>,
) {
    let Some(choice) = action.0.take() else {
        return;
    };

    match choice {
        InventoryUiChoice::Close => {
            open.0 = false;
            status.clear();
        }
        InventoryUiChoice::UseItem(slot_idx) => {
            handle_use_item_action(
                *app_state.get(),
                turn_phase.get().clone(),
                &mut player_action,
                &mut next_turn_phase,
                slot_idx,
            );
        }
        InventoryUiChoice::EquipFromInventory(slot_idx) => {
            let Ok((mut inventory, equipment)) = query.single_mut() else {
                return;
            };
            let Some(mut equipment) = equipment else {
                status.set(STATUS_MISSING_EQUIPMENT, InventoryStatusTone::Warning);
                return;
            };
            match equip_from_inventory_slot(&mut inventory, &mut equipment, slot_idx) {
                Ok(EquipOutcome::Equipped) => {
                    status.set(STATUS_EQUIP_OK, InventoryStatusTone::Success);
                }
                Ok(EquipOutcome::Swapped) => {
                    status.set(STATUS_SWAP_OK, InventoryStatusTone::Success);
                }
                Err(error) => {
                    status.set(status_message_for_error(error), InventoryStatusTone::Warning);
                }
            }
        }
        InventoryUiChoice::Unequip(slot) => {
            let Ok((mut inventory, equipment)) = query.single_mut() else {
                return;
            };
            let Some(mut equipment) = equipment else {
                status.set(STATUS_MISSING_EQUIPMENT, InventoryStatusTone::Warning);
                return;
            };
            match unequip_to_inventory_slot(&mut inventory, &mut equipment, slot) {
                Ok(()) => {
                    status.set(STATUS_UNEQUIP_OK, InventoryStatusTone::Success);
                }
                Err(error) => {
                    status.set(status_message_for_error(error), InventoryStatusTone::Warning);
                }
            }
        }
    }
}

fn handle_use_item_action(
    app_state: AppState,
    turn_phase: TurnPhase,
    player_action: &mut PlayerAction,
    next_turn_phase: &mut NextState<TurnPhase>,
    slot_index: usize,
) {
    if app_state == AppState::Dungeon && turn_phase == TurnPhase::AwaitingInput {
        player_action.0 = Some(PendingAction::UseItem(slot_index));
        next_turn_phase.set(TurnPhase::PlayerTurn);
    }
}

fn status_message_for_error(error: InventoryRuleError) -> &'static str {
    match error {
        InventoryRuleError::InvalidInventorySlot => STATUS_INVALID_SLOT,
        InventoryRuleError::NotEquippable => STATUS_NOT_EQUIPPABLE,
        InventoryRuleError::InventoryFull => STATUS_NO_SPACE,
        InventoryRuleError::NothingEquipped => STATUS_NOTHING_EQUIPPED,
    }
}

fn status_color(status: &InventoryUiStatus) -> egui::Color32 {
    match status.tone.unwrap_or(InventoryStatusTone::Neutral) {
        InventoryStatusTone::Neutral => rgb(STATUS_NEUTRAL_RGB),
        InventoryStatusTone::Success => rgb(STATUS_SUCCESS_RGB),
        InventoryStatusTone::Warning => rgb(STATUS_WARNING_RGB),
    }
}

fn rgb(rgb: (u8, u8, u8)) -> egui::Color32 {
    egui::Color32::from_rgb(rgb.0, rgb.1, rgb.2)
}
