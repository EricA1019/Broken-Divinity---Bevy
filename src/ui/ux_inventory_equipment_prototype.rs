//! Dedicated inventory + equipment (paper-doll) prototype.
//!
//! Run with: `cargo run --bin ux_inventory_equipment_prototype`
//!
//! Controls:
//!   I      — inventory pane
//!   E      — equipment pane
//!   Esc    — quit

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use super::ux_style_contract::{style_for, VariantStyle};

const INV_SIZE: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InventoryScreen {
    Inventory,
    Equipment,
}

impl InventoryScreen {
    fn label(self) -> &'static str {
        match self {
            Self::Inventory => "Inventory",
            Self::Equipment => "Equipment",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EquipSlot {
    Head,
    Chest,
    Weapon,
    Offhand,
    Hands,
    Legs,
    Feet,
    Accessory1,
    Accessory2,
    Accessory3,
}

impl EquipSlot {
    fn label(self) -> &'static str {
        match self {
            Self::Head => "Head",
            Self::Chest => "Chest",
            Self::Weapon => "Weapon",
            Self::Offhand => "Offhand",
            Self::Hands => "Hands",
            Self::Legs => "Legs",
            Self::Feet => "Feet",
            Self::Accessory1 => "Accessory I",
            Self::Accessory2 => "Accessory II",
            Self::Accessory3 => "Accessory III",
        }
    }
}

#[derive(Debug, Clone)]
struct Item {
    name: &'static str,
    quantity: u32,
    desc: &'static str,
    equippable: Option<EquipSlot>,
    rating: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DragSource {
    Inventory(usize),
    Equipment(EquipSlot),
}

#[derive(Debug, Clone, Copy)]
struct DragPayload {
    item_idx: usize,
    source: DragSource,
}

#[derive(Resource)]
pub(crate) struct InventoryProtoState {
    screen: InventoryScreen,
    selected_slot: usize,
    // Inventory slot -> optional item index.
    inventory_slots: [Option<usize>; INV_SIZE],
    // Equipment slot -> optional item index.
    eq_head: Option<usize>,
    eq_chest: Option<usize>,
    eq_weapon: Option<usize>,
    eq_offhand: Option<usize>,
    eq_hands: Option<usize>,
    eq_legs: Option<usize>,
    eq_feet: Option<usize>,
    eq_accessory_1: Option<usize>,
    eq_accessory_2: Option<usize>,
    eq_accessory_3: Option<usize>,
    dragging: Option<DragPayload>,
    items: Vec<Item>,
}

pub struct InventoryEquipmentPrototypePlugin;

impl Plugin for InventoryEquipmentPrototypePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(seed_state())
            .add_systems(Startup, setup_camera)
            .add_systems(Update, handle_input)
            .add_systems(EguiPrimaryContextPass, draw_inventory_equipment_prototype);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn seed_state() -> InventoryProtoState {
    let items = vec![
        Item {
            name: "Salvaged Rifle",
            quantity: 1,
            desc: "Reliable ballistic rifle. Loud, stable, and serviceable.",
            equippable: Some(EquipSlot::Weapon),
            rating: "8-15 dmg",
        },
        Item {
            name: "Scrap Vest",
            quantity: 1,
            desc: "Layered scrap plates over stitched cloth.",
            equippable: Some(EquipSlot::Chest),
            rating: "AR 4",
        },
        Item {
            name: "Field Helm",
            quantity: 1,
            desc: "Light head protection with cracked visor.",
            equippable: Some(EquipSlot::Head),
            rating: "AR 1",
        },
        Item {
            name: "Bandage Roll",
            quantity: 3,
            desc: "Stops bleeding and buys time.",
            equippable: None,
            rating: "Use: Stabilize",
        },
        Item {
            name: "Charm of Witness",
            quantity: 1,
            desc: "Minor relic that steadies exposure spikes.",
            equippable: Some(EquipSlot::Accessory1),
            rating: "+5 exposure resist",
        },
        Item {
            name: "Work Gloves",
            quantity: 1,
            desc: "Improves grip and repair handling.",
            equippable: Some(EquipSlot::Hands),
            rating: "+repair handling",
        },
        Item {
            name: "Trail Boots",
            quantity: 1,
            desc: "Sturdy boots for rough routes.",
            equippable: Some(EquipSlot::Feet),
            rating: "+travel safety",
        },
    ];

    let mut inventory_slots = [None; INV_SIZE];
    inventory_slots[0] = Some(0);
    inventory_slots[1] = Some(1);
    inventory_slots[2] = Some(2);
    inventory_slots[3] = Some(3);
    inventory_slots[4] = Some(4);
    inventory_slots[5] = Some(5);
    inventory_slots[6] = Some(6);

    InventoryProtoState {
        screen: InventoryScreen::Inventory,
        selected_slot: 0,
        inventory_slots,
        eq_head: Some(2),
        eq_chest: Some(1),
        eq_weapon: Some(0),
        eq_offhand: None,
        eq_hands: Some(5),
        eq_legs: None,
        eq_feet: Some(6),
        eq_accessory_1: Some(4),
        eq_accessory_2: None,
        eq_accessory_3: None,
        dragging: None,
        items,
    }
}

pub(crate) fn inventory_seed_state() -> InventoryProtoState {
    seed_state()
}

pub(crate) fn handle_inventory_equipment_input(
    keys: &ButtonInput<KeyCode>,
    state: &mut InventoryProtoState,
) {
    if keys.just_pressed(KeyCode::KeyI) {
        state.screen = InventoryScreen::Inventory;
    }
    if keys.just_pressed(KeyCode::KeyE) {
        state.screen = InventoryScreen::Equipment;
    }
}

fn handle_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<InventoryProtoState>,
    mut exit: MessageWriter<AppExit>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
        return;
    }
    if keys.just_pressed(KeyCode::KeyI) {
        state.screen = InventoryScreen::Inventory;
    }
    if keys.just_pressed(KeyCode::KeyE) {
        state.screen = InventoryScreen::Equipment;
    }
}

fn draw_inventory_equipment_prototype(mut contexts: EguiContexts, mut state: ResMut<InventoryProtoState>) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let s = style_for();

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(s.panel_bg))
        .show(ctx, |ui| {
            ui.label(styled(
                &s,
                &format!(
                    " Inventory & Equipment  |  Screen: {}  |  I inventory  E equipment  Esc quit",
                    state.screen.label()
                ),
                11.0,
                s.subtitle_color,
            ));
            ui.separator();

            draw_inventory_equipment_content(ui, &s, &mut state);

            ui.add_space(6.0 * s.spacing);
            ui.separator();
            ui.label(styled(
                &s,
                " Right-click slots for context menu. Hover any slot for tooltip.",
                s.small_size,
                s.subtitle_color,
            ));
        });
}

pub(crate) fn draw_inventory_equipment_content(
    ui: &mut egui::Ui,
    s: &VariantStyle,
    state: &mut InventoryProtoState,
) {
    ui.columns(2, |cols| {
        draw_inventory_panel(&mut cols[0], s, state);
        draw_equipment_panel(&mut cols[1], s, state);
    });
}

fn draw_inventory_panel(ui: &mut egui::Ui, s: &VariantStyle, state: &mut InventoryProtoState) {
    ui.label(styled(s, " Inventory List ", s.heading_size, s.title_color));
    ui.label(styled(s, " Classic pack rendered as readable list with context actions.", s.small_size, s.subtitle_color));
    ui.add_space(4.0 * s.spacing);

    egui::ScrollArea::vertical().max_height(330.0).show(ui, |ui| {
        for idx in 0..INV_SIZE {
            let item_idx = state.inventory_slots[idx];
            let is_selected = state.selected_slot == idx;

            let fill = if item_idx.is_some() {
                if is_selected {
                    s.accent_color.gamma_multiply(0.55)
                } else {
                    s.info_color.gamma_multiply(0.28)
                }
            } else {
                s.subtitle_color.gamma_multiply(0.10)
            };

            let row_text = if let Some(it) = item_idx {
                let item = &state.items[it];
                format!(
                    "[{idx:02}]  {:<26}  x{}  {}",
                    item.name,
                    item.quantity,
                    item.rating
                )
            } else {
                format!("[{idx:02}]  [ ]  Empty slot")
            };

            let resp = ui.add_sized(
                [ui.available_width(), 28.0],
                egui::Button::new(styled(s, &row_text, s.small_size, s.title_color)).fill(fill),
            );

            let resp = if let Some(it) = item_idx {
                let item = &state.items[it];
                resp.on_hover_text(format!(
                    "{}\n{}\n{}",
                    item.name,
                    item.rating,
                    item.desc
                ))
            } else {
                resp.on_hover_text("Empty slot")
            };

            if resp.clicked() {
                state.selected_slot = idx;
            }

            if resp.drag_started() {
                if let Some(it) = item_idx {
                    state.dragging = Some(DragPayload {
                        item_idx: it,
                        source: DragSource::Inventory(idx),
                    });
                }
            }

            if resp.double_clicked() {
                if let Some(it) = item_idx {
                    if let Some(slot) = preferred_equip_slot(state, it) {
                        equip_item_to_slot(state, it, slot);
                    }
                }
            }

            let dropped_here = resp.hovered()
                && state.dragging.is_some()
                && ui.ctx().input(|i| i.pointer.any_released());
            if dropped_here {
                if let Some(payload) = state.dragging.take() {
                    drop_to_inventory_slot(state, payload, idx);
                }
            }

            resp.context_menu(|ui| {
                if let Some(it) = item_idx {
                    let item = &state.items[it];
                    ui.label(styled(s, item.name, s.small_size, s.title_color));
                    if let Some(slot) = item.equippable {
                        if is_accessory_slot(slot) {
                            for acc_slot in [EquipSlot::Accessory1, EquipSlot::Accessory2, EquipSlot::Accessory3] {
                                if ui.button(format!("Equip to {}", acc_slot.label())).clicked() {
                                    equip_item_to_slot(state, it, acc_slot);
                                    ui.close();
                                }
                            }
                        } else if ui.button(format!("Equip to {}", slot.label())).clicked() {
                            equip_item_to_slot(state, it, slot);
                            ui.close();
                        }
                    }
                    if ui.button("Use").clicked() {
                        ui.close();
                    }
                    if ui.button("Drop").clicked() {
                        state.inventory_slots[idx] = None;
                        if state.selected_slot == idx {
                            state.selected_slot = 0;
                        }
                        ui.close();
                    }
                } else {
                    ui.label(styled(s, "Empty slot", s.small_size, s.subtitle_color));
                }
            });
        }
    });

    ui.add_space(6.0 * s.spacing);
    draw_selected_item_card(ui, s, state);
}

fn draw_selected_item_card(ui: &mut egui::Ui, s: &VariantStyle, state: &InventoryProtoState) {
    ui.separator();
    ui.label(styled(s, " Selected Item ", s.body_size, s.accent_color));

    if let Some(item_idx) = state.inventory_slots[state.selected_slot] {
        let item = &state.items[item_idx];
        ui.label(styled(s, item.name, s.body_size, s.title_color));
        ui.label(styled(s, item.rating, s.small_size, s.info_color));
        ui.label(styled(s, item.desc, s.small_size, s.subtitle_color));
        if let Some(slot) = item.equippable {
            ui.label(styled(
                s,
                &format!("Equippable: {}", slot.label()),
                s.small_size,
                s.success_color,
            ));
        } else {
            ui.label(styled(s, "Not equippable", s.small_size, s.warn_color));
        }
    } else {
        ui.label(styled(s, "No item selected.", s.small_size, s.subtitle_color));
    }
}

fn draw_equipment_panel(ui: &mut egui::Ui, s: &VariantStyle, state: &mut InventoryProtoState) {
    ui.label(styled(s, " Equipment Paper Doll ", s.heading_size, s.title_color));
    ui.label(styled(
        s,
        " Empty slots are dark boxes. Equipped slots are highlighted.",
        s.small_size,
        s.subtitle_color,
    ));
    ui.add_space(4.0 * s.spacing);

    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            egui::Grid::new("paper_doll_main")
                .spacing(egui::vec2(10.0, 8.0))
                .show(ui, |ui| {
                    ui.label("");
                    draw_slot(ui, s, state, EquipSlot::Head);
                    ui.label("");
                    ui.end_row();

                    draw_slot(ui, s, state, EquipSlot::Weapon);
                    draw_slot(ui, s, state, EquipSlot::Chest);
                    draw_slot(ui, s, state, EquipSlot::Offhand);
                    ui.end_row();

                    ui.label("");
                    draw_slot(ui, s, state, EquipSlot::Hands);
                    ui.label("");
                    ui.end_row();

                    ui.label("");
                    draw_slot(ui, s, state, EquipSlot::Legs);
                    ui.label("");
                    ui.end_row();

                    ui.label("");
                    draw_slot(ui, s, state, EquipSlot::Feet);
                    ui.label("");
                    ui.end_row();
                });
        });

        ui.add_space(12.0);

        ui.vertical(|ui| {
            ui.label(styled(s, " Accessories ", s.body_size, s.accent_color));
            ui.add_space(4.0 * s.spacing);
            draw_slot(ui, s, state, EquipSlot::Accessory1);
            ui.add_space(4.0);
            draw_slot(ui, s, state, EquipSlot::Accessory2);
            ui.add_space(4.0);
            draw_slot(ui, s, state, EquipSlot::Accessory3);
        });
    });
}

fn draw_slot(ui: &mut egui::Ui, s: &VariantStyle, state: &mut InventoryProtoState, slot: EquipSlot) {
    let item_idx = get_equipped(state, slot);
    let is_filled = item_idx.is_some();

    let fill = if is_filled {
        s.success_color.gamma_multiply(0.30)
    } else {
        s.subtitle_color.gamma_multiply(0.12)
    };

    let text = if let Some(it) = item_idx {
        let item = &state.items[it];
        format!("{}\n[{}]", slot.label(), item.name)
    } else {
        format!("{}\n[ ]", slot.label())
    };

    let resp = ui.add_sized(
        [96.0, 74.0],
        egui::Button::new(styled(s, &text, s.small_size, s.title_color)).fill(fill),
    );

    let resp = if let Some(it) = item_idx {
        let item = &state.items[it];
        resp.on_hover_text(format!("{}\n{}\n{}", item.name, item.rating, item.desc))
    } else {
        resp.on_hover_text(format!("{} slot is empty", slot.label()))
    };

    if resp.drag_started() {
        if let Some(it) = item_idx {
            state.dragging = Some(DragPayload {
                item_idx: it,
                source: DragSource::Equipment(slot),
            });
        }
    }

    if resp.double_clicked() {
        if item_idx.is_some() {
            unequip_to_inventory(state, slot);
        }
    }

    let dropped_here = resp.hovered()
        && state.dragging.is_some()
        && ui.ctx().input(|i| i.pointer.any_released());
    if dropped_here {
        if let Some(payload) = state.dragging.take() {
            drop_to_equipment_slot(state, payload, slot);
        }
    }

    resp.context_menu(|ui| {
        if item_idx.is_some() {
            if ui.button("Unequip").clicked() {
                unequip_to_inventory(state, slot);
                ui.close();
            }
        }
        ui.label(styled(s, "Auto-equip from inventory menu", s.small_size, s.subtitle_color));
    });
}

fn get_equipped(state: &InventoryProtoState, slot: EquipSlot) -> Option<usize> {
    match slot {
        EquipSlot::Head => state.eq_head,
        EquipSlot::Chest => state.eq_chest,
        EquipSlot::Weapon => state.eq_weapon,
        EquipSlot::Offhand => state.eq_offhand,
        EquipSlot::Hands => state.eq_hands,
        EquipSlot::Legs => state.eq_legs,
        EquipSlot::Feet => state.eq_feet,
        EquipSlot::Accessory1 => state.eq_accessory_1,
        EquipSlot::Accessory2 => state.eq_accessory_2,
        EquipSlot::Accessory3 => state.eq_accessory_3,
    }
}

fn set_equipped(state: &mut InventoryProtoState, slot: EquipSlot, item: Option<usize>) {
    match slot {
        EquipSlot::Head => state.eq_head = item,
        EquipSlot::Chest => state.eq_chest = item,
        EquipSlot::Weapon => state.eq_weapon = item,
        EquipSlot::Offhand => state.eq_offhand = item,
        EquipSlot::Hands => state.eq_hands = item,
        EquipSlot::Legs => state.eq_legs = item,
        EquipSlot::Feet => state.eq_feet = item,
        EquipSlot::Accessory1 => state.eq_accessory_1 = item,
        EquipSlot::Accessory2 => state.eq_accessory_2 = item,
        EquipSlot::Accessory3 => state.eq_accessory_3 = item,
    }
}

fn equip_item_to_slot(state: &mut InventoryProtoState, item_idx: usize, slot: EquipSlot) {
    set_equipped(state, slot, Some(item_idx));
}

fn is_accessory_slot(slot: EquipSlot) -> bool {
    matches!(slot, EquipSlot::Accessory1 | EquipSlot::Accessory2 | EquipSlot::Accessory3)
}

fn preferred_equip_slot(state: &InventoryProtoState, item_idx: usize) -> Option<EquipSlot> {
    let slot = state.items[item_idx].equippable?;
    if is_accessory_slot(slot) {
        if state.eq_accessory_1.is_none() {
            return Some(EquipSlot::Accessory1);
        }
        if state.eq_accessory_2.is_none() {
            return Some(EquipSlot::Accessory2);
        }
        if state.eq_accessory_3.is_none() {
            return Some(EquipSlot::Accessory3);
        }
        return Some(EquipSlot::Accessory1);
    }
    Some(slot)
}

fn first_empty_inventory_slot(state: &InventoryProtoState) -> Option<usize> {
    (0..INV_SIZE).find(|&i| state.inventory_slots[i].is_none())
}

fn unequip_to_inventory(state: &mut InventoryProtoState, slot: EquipSlot) {
    if let Some(item_idx) = get_equipped(state, slot) {
        if let Some(dest) = first_empty_inventory_slot(state) {
            state.inventory_slots[dest] = Some(item_idx);
            set_equipped(state, slot, None);
            state.selected_slot = dest;
        }
    }
}

fn drop_to_inventory_slot(state: &mut InventoryProtoState, payload: DragPayload, target_idx: usize) {
    match payload.source {
        DragSource::Inventory(src_idx) => {
            if src_idx == target_idx {
                return;
            }
            state.inventory_slots.swap(src_idx, target_idx);
            state.selected_slot = target_idx;
        }
        DragSource::Equipment(src_slot) => {
            if state.inventory_slots[target_idx].is_none() {
                state.inventory_slots[target_idx] = Some(payload.item_idx);
                set_equipped(state, src_slot, None);
                state.selected_slot = target_idx;
            }
        }
    }
}

fn drop_to_equipment_slot(state: &mut InventoryProtoState, payload: DragPayload, target_slot: EquipSlot) {
    let item = &state.items[payload.item_idx];
    let can_equip = match item.equippable {
        Some(base) if is_accessory_slot(base) && is_accessory_slot(target_slot) => true,
        Some(base) => base == target_slot,
        None => false,
    };
    if !can_equip {
        return;
    }

    match payload.source {
        DragSource::Inventory(src_idx) => {
            set_equipped(state, target_slot, Some(payload.item_idx));
            state.inventory_slots[src_idx] = None;
        }
        DragSource::Equipment(src_slot) => {
            if src_slot != target_slot {
                set_equipped(state, src_slot, None);
            }
            set_equipped(state, target_slot, Some(payload.item_idx));
        }
    }
}

fn styled(style: &VariantStyle, text: &str, size: f32, color: egui::Color32) -> egui::RichText {
    let mut rt = egui::RichText::new(text).size(size).color(color);
    if style.mono_all {
        rt = rt.monospace();
    }
    rt
}
