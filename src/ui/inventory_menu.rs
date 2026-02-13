//! Inventory and equipment UI

use egui::{Color32, FontId, RichText, ScrollArea, Ui, Vec2};

use infinite_game::combat::equipment::{EquipmentSet, EquipmentSlot};
use infinite_game::combat::inventory::Inventory;
use infinite_game::combat::item::{Item, ItemCategory, ItemRarity};
use infinite_game::player::stats::CharacterStats;

use crate::state::StateTransition;

/// Active tab in the inventory screen
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventoryTab {
    Equipment,
    Inventory,
}

/// Action to perform after rendering the inventory
#[derive(Debug, Clone)]
pub enum InventoryAction {
    None,
    EquipItem { inventory_index: usize, slot: EquipmentSlot },
    UnequipItem { slot: EquipmentSlot },
    UseItem { inventory_index: usize },
}

/// Inventory menu state
pub struct InventoryMenu {
    pub active_tab: InventoryTab,
    pub selected_item: Option<usize>,
    pub selected_slot: Option<EquipmentSlot>,
}

impl Default for InventoryMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl InventoryMenu {
    pub fn new() -> Self {
        Self {
            active_tab: InventoryTab::Equipment,
            selected_item: None,
            selected_slot: None,
        }
    }

    pub fn render(
        &mut self,
        ui: &mut Ui,
        equipment: &EquipmentSet,
        inventory: &Inventory,
        stats: &CharacterStats,
    ) -> (StateTransition, InventoryAction) {
        let mut transition = StateTransition::None;
        let mut action = InventoryAction::None;

        // Semi-transparent background overlay
        let painter = ui.painter();
        painter.rect_filled(
            ui.max_rect(),
            0.0,
            Color32::from_rgba_unmultiplied(0, 0, 0, 180),
        );

        let available = ui.available_size();

        ui.vertical_centered(|ui| {
            ui.add_space(available.y * 0.05);

            // Title
            ui.label(
                RichText::new("INVENTORY")
                    .font(FontId::proportional(48.0))
                    .color(Color32::from_rgb(200, 200, 255)),
            );

            ui.add_space(15.0);

            // Tab buttons
            ui.horizontal(|ui| {
                ui.add_space(available.x * 0.3);
                if tab_button(ui, "Equipment", self.active_tab == InventoryTab::Equipment) {
                    self.active_tab = InventoryTab::Equipment;
                    self.selected_item = None;
                    self.selected_slot = None;
                }
                ui.add_space(10.0);
                if tab_button(ui, "Inventory", self.active_tab == InventoryTab::Inventory) {
                    self.active_tab = InventoryTab::Inventory;
                    self.selected_item = None;
                    self.selected_slot = None;
                }
            });

            ui.add_space(15.0);

            // Content area
            let content_width = available.x * 0.7;
            let content_height = available.y * 0.65;

            ui.allocate_ui(Vec2::new(content_width, content_height), |ui| {
                match self.active_tab {
                    InventoryTab::Equipment => {
                        action = self.render_equipment_tab(ui, equipment, inventory, stats);
                    }
                    InventoryTab::Inventory => {
                        action = self.render_inventory_tab(ui, equipment, inventory, stats);
                    }
                }
            });

            ui.add_space(20.0);

            // Close button
            let button_size = Vec2::new(120.0, 36.0);
            if inv_button(ui, "Close", button_size) {
                transition = StateTransition::Pop;
            }
        });

        (transition, action)
    }

    fn render_equipment_tab(
        &mut self,
        ui: &mut Ui,
        equipment: &EquipmentSet,
        _inventory: &Inventory,
        _stats: &CharacterStats,
    ) -> InventoryAction {
        let mut action = InventoryAction::None;

        ui.horizontal(|ui| {
            // Left: Armor slots
            ui.vertical(|ui| {
                ui.label(
                    RichText::new("Armor")
                        .font(FontId::proportional(16.0))
                        .color(Color32::from_rgb(200, 200, 255)),
                );
                ui.add_space(5.0);

                let armor_slots = [
                    EquipmentSlot::Head,
                    EquipmentSlot::Shoulders,
                    EquipmentSlot::Chest,
                    EquipmentSlot::Bracers,
                    EquipmentSlot::Gloves,
                    EquipmentSlot::Belt,
                    EquipmentSlot::Legs,
                    EquipmentSlot::Boots,
                    EquipmentSlot::Cape,
                ];

                for &slot in &armor_slots {
                    let item = equipment.get(slot);
                    let is_selected = self.selected_slot == Some(slot);
                    if equipment_slot_button(ui, slot, item, is_selected) {
                        if item.is_some() {
                            if self.selected_slot == Some(slot) {
                                // Double-click to unequip
                                action = InventoryAction::UnequipItem { slot };
                                self.selected_slot = None;
                            } else {
                                self.selected_slot = Some(slot);
                                self.selected_item = None;
                            }
                        } else {
                            self.selected_slot = Some(slot);
                            self.selected_item = None;
                        }
                    }
                }
            });

            ui.add_space(30.0);

            // Right: Weapon and accessory slots
            ui.vertical(|ui| {
                ui.label(
                    RichText::new("Weapons & Accessories")
                        .font(FontId::proportional(16.0))
                        .color(Color32::from_rgb(200, 200, 255)),
                );
                ui.add_space(5.0);

                let other_slots = [
                    EquipmentSlot::MainHand,
                    EquipmentSlot::OffHand,
                    EquipmentSlot::Ring1,
                    EquipmentSlot::Ring2,
                    EquipmentSlot::Amulet,
                ];

                for &slot in &other_slots {
                    let item = equipment.get(slot);
                    let is_selected = self.selected_slot == Some(slot);
                    if equipment_slot_button(ui, slot, item, is_selected) {
                        if item.is_some() {
                            if self.selected_slot == Some(slot) {
                                action = InventoryAction::UnequipItem { slot };
                                self.selected_slot = None;
                            } else {
                                self.selected_slot = Some(slot);
                                self.selected_item = None;
                            }
                        } else {
                            self.selected_slot = Some(slot);
                            self.selected_item = None;
                        }
                    }
                }

                // Detail panel for selected slot
                if let Some(slot) = self.selected_slot {
                    ui.add_space(15.0);
                    ui.separator();
                    ui.add_space(5.0);
                    if let Some(item) = equipment.get(slot) {
                        render_item_detail(ui, item);
                        ui.add_space(5.0);
                        if inv_button(ui, "Unequip", Vec2::new(100.0, 28.0)) {
                            action = InventoryAction::UnequipItem { slot };
                            self.selected_slot = None;
                        }
                    } else {
                        ui.label(
                            RichText::new(format!("{} - Empty", slot.name()))
                                .color(Color32::from_rgb(140, 140, 160)),
                        );
                    }
                }
            });
        });

        action
    }

    fn render_inventory_tab(
        &mut self,
        ui: &mut Ui,
        _equipment: &EquipmentSet,
        inventory: &Inventory,
        _stats: &CharacterStats,
    ) -> InventoryAction {
        let mut action = InventoryAction::None;

        ui.horizontal(|ui| {
            // Left: Item grid
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(format!("Items ({}/{})", inventory.len(), inventory.capacity))
                        .font(FontId::proportional(14.0))
                        .color(Color32::from_rgb(200, 200, 255)),
                );
                ui.add_space(5.0);

                ScrollArea::vertical()
                    .max_height(ui.available_height())
                    .show(ui, |ui| {
                        let cols = 5;
                        let mut col = 0;
                        ui.horizontal_wrapped(|ui| {
                            for (idx, item) in inventory.items.iter().enumerate() {
                                let is_selected = self.selected_item == Some(idx);
                                if inventory_item_button(ui, item, is_selected) {
                                    if self.selected_item == Some(idx) {
                                        self.selected_item = None;
                                    } else {
                                        self.selected_item = Some(idx);
                                        self.selected_slot = None;
                                    }
                                }
                                col += 1;
                                if col >= cols {
                                    col = 0;
                                    ui.end_row();
                                }
                            }
                        });
                    });
            });

            ui.add_space(20.0);

            // Right: Detail panel
            ui.vertical(|ui| {
                ui.set_min_width(200.0);
                if let Some(idx) = self.selected_item {
                    if let Some(item) = inventory.get(idx) {
                        render_item_detail(ui, item);
                        ui.add_space(10.0);

                        // Equip button for equippable items
                        if item.category == ItemCategory::Weapon
                            || item.category == ItemCategory::Armor
                            || item.category == ItemCategory::Accessory
                        {
                            let target_slot = suggested_slot(item);
                            if let Some(slot) = target_slot {
                                if inv_button(ui, &format!("Equip ({})", slot.name()), Vec2::new(160.0, 28.0)) {
                                    action = InventoryAction::EquipItem {
                                        inventory_index: idx,
                                        slot,
                                    };
                                    self.selected_item = None;
                                }
                            }
                        }

                        // Use button for consumable items
                        if item.category == ItemCategory::Consumable
                            && inv_button(ui, "Use", Vec2::new(100.0, 28.0))
                        {
                            action = InventoryAction::UseItem {
                                inventory_index: idx,
                            };
                            self.selected_item = None;
                        }
                    }
                } else {
                    ui.label(
                        RichText::new("Select an item to view details")
                            .color(Color32::from_rgb(140, 140, 160)),
                    );
                }
            });
        });

        action
    }
}

/// Suggest the best equipment slot for an item
fn suggested_slot(item: &Item) -> Option<EquipmentSlot> {
    match item.category {
        ItemCategory::Weapon => Some(EquipmentSlot::MainHand),
        ItemCategory::Armor => Some(EquipmentSlot::Chest), // default to chest
        ItemCategory::Accessory => Some(EquipmentSlot::Ring1),
        _ => None,
    }
}

fn rarity_color(rarity: ItemRarity) -> Color32 {
    let c = rarity.color();
    Color32::from_rgb(
        (c[0] * 255.0) as u8,
        (c[1] * 255.0) as u8,
        (c[2] * 255.0) as u8,
    )
}

fn tab_button(ui: &mut Ui, text: &str, active: bool) -> bool {
    let fill = if active {
        Color32::from_rgba_unmultiplied(70, 70, 100, 220)
    } else {
        Color32::from_rgba_unmultiplied(50, 50, 70, 220)
    };
    let stroke_color = if active {
        Color32::from_rgb(120, 120, 180)
    } else {
        Color32::from_rgb(80, 80, 100)
    };

    ui.add(
        egui::Button::new(
            RichText::new(text)
                .font(FontId::proportional(16.0))
                .color(Color32::from_rgb(220, 220, 240)),
        )
        .min_size(Vec2::new(120.0, 32.0))
        .fill(fill)
        .stroke(egui::Stroke::new(1.0, stroke_color)),
    )
    .clicked()
}

fn inv_button(ui: &mut Ui, text: &str, size: Vec2) -> bool {
    ui.add(
        egui::Button::new(
            RichText::new(text)
                .font(FontId::proportional(14.0))
                .color(Color32::from_rgb(220, 220, 240)),
        )
        .min_size(size)
        .fill(Color32::from_rgba_unmultiplied(50, 50, 70, 220))
        .stroke(egui::Stroke::new(1.0, Color32::from_rgb(80, 80, 100))),
    )
    .clicked()
}

fn equipment_slot_button(
    ui: &mut Ui,
    slot: EquipmentSlot,
    item: &Option<Item>,
    selected: bool,
) -> bool {
    let fill = if selected {
        Color32::from_rgba_unmultiplied(70, 70, 100, 220)
    } else {
        Color32::from_rgba_unmultiplied(40, 40, 55, 220)
    };
    let stroke_color = if selected {
        Color32::from_rgb(120, 120, 180)
    } else {
        Color32::from_rgb(70, 70, 90)
    };

    let label = if let Some(item) = item {
        format!("{}: {}", slot.name(), item.name)
    } else {
        format!("{}: Empty", slot.name())
    };

    let text_color = if let Some(item) = item {
        rarity_color(item.rarity)
    } else {
        Color32::from_rgb(140, 140, 160)
    };

    ui.add(
        egui::Button::new(
            RichText::new(label)
                .font(FontId::proportional(13.0))
                .color(text_color),
        )
        .min_size(Vec2::new(220.0, 24.0))
        .fill(fill)
        .stroke(egui::Stroke::new(1.0, stroke_color)),
    )
    .clicked()
}

fn inventory_item_button(ui: &mut Ui, item: &Item, selected: bool) -> bool {
    let fill = if selected {
        Color32::from_rgba_unmultiplied(70, 70, 100, 220)
    } else {
        Color32::from_rgba_unmultiplied(40, 40, 55, 220)
    };
    let stroke_color = if selected {
        Color32::from_rgb(120, 120, 180)
    } else {
        Color32::from_rgb(70, 70, 90)
    };

    let label = if item.stack_count > 1 {
        format!("{} x{}", item.name, item.stack_count)
    } else {
        item.name.clone()
    };

    ui.add(
        egui::Button::new(
            RichText::new(label)
                .font(FontId::proportional(12.0))
                .color(rarity_color(item.rarity)),
        )
        .min_size(Vec2::new(110.0, 28.0))
        .fill(fill)
        .stroke(egui::Stroke::new(1.0, stroke_color)),
    )
    .clicked()
}

fn render_item_detail(ui: &mut Ui, item: &Item) {
    ui.label(
        RichText::new(&item.name)
            .font(FontId::proportional(18.0))
            .color(rarity_color(item.rarity)),
    );
    ui.label(
        RichText::new(format!("{} {:?}", item.rarity.name(), item.category))
            .font(FontId::proportional(12.0))
            .color(Color32::from_rgb(180, 180, 200)),
    );
    if item.element != infinite_game::Element::Physical {
        ui.label(
            RichText::new(format!("Element: {}", item.element.name()))
                .font(FontId::proportional(12.0))
                .color(Color32::from_rgb(160, 180, 220)),
        );
    }
    ui.add_space(4.0);
    ui.label(
        RichText::new(&item.description)
            .font(FontId::proportional(12.0))
            .color(Color32::from_rgb(180, 180, 200)),
    );

    // Stat modifiers
    let mods = item.total_modifiers();
    ui.add_space(4.0);
    if mods.max_hp != 0.0 {
        stat_line(ui, "Max HP", mods.max_hp);
    }
    if mods.attack != 0.0 {
        stat_line(ui, "Attack", mods.attack);
    }
    if mods.defense != 0.0 {
        stat_line(ui, "Defense", mods.defense);
    }
    if mods.speed != 0.0 {
        stat_line(ui, "Speed", mods.speed);
    }
    if mods.crit_chance != 0.0 {
        stat_line(ui, "Crit Chance", mods.crit_chance * 100.0);
    }
    if mods.crit_multiplier != 0.0 {
        stat_line(ui, "Crit Damage", mods.crit_multiplier);
    }

    // Weapon data
    if let Some(wd) = &item.weapon_data {
        ui.add_space(4.0);
        ui.label(
            RichText::new(format!("Damage: {:.0}  ({})", wd.base_damage, wd.weapon_type.name()))
                .font(FontId::proportional(12.0))
                .color(Color32::from_rgb(200, 180, 140)),
        );
    }

    if item.stack_count > 1 {
        ui.label(
            RichText::new(format!("Stack: {}/{}", item.stack_count, item.max_stack))
                .font(FontId::proportional(11.0))
                .color(Color32::from_rgb(160, 160, 180)),
        );
    }
}

fn stat_line(ui: &mut Ui, name: &str, value: f32) {
    let sign = if value > 0.0 { "+" } else { "" };
    let color = if value > 0.0 {
        Color32::from_rgb(100, 220, 100)
    } else {
        Color32::from_rgb(220, 100, 100)
    };
    ui.label(
        RichText::new(format!("{}{:.0} {}", sign, value, name))
            .font(FontId::proportional(12.0))
            .color(color),
    );
}
