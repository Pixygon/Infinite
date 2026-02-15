//! Item editor — list + create/edit form for server-backed items

use egui::{Color32, FontId, RichText, ScrollArea, Ui, Vec2};

use infinite_integration::IntegrationClient;
use infinite_integration::types::{ServerCharacterItem, ServerItemStats, GameItemCustomStats};
use infinite_integration::PendingRequest;

/// Item editor state
pub struct ItemEditor {
    /// List of items fetched from server
    items: Vec<ServerCharacterItem>,
    /// Currently selected item index (None = creating new)
    selected_index: Option<usize>,
    /// Editable form fields
    form: ItemForm,
    /// Whether we're in "create new" mode
    creating_new: bool,
    /// Pending list fetch
    pending_list: Option<PendingRequest<Vec<ServerCharacterItem>>>,
    /// Pending save
    pending_save: Option<PendingRequest<ServerCharacterItem>>,
    /// Pending delete
    pending_delete: Option<PendingRequest<serde_json::Value>>,
    /// Status message
    status: Option<(String, bool)>, // (message, is_error)
}

/// Editable form for an item
struct ItemForm {
    name: String,
    description: String,
    icon: String,
    category: String,
    subcategory: String,
    rarity: String,
    // Game stats
    max_hp: f32,
    attack: f32,
    defense: f32,
    speed: f32,
    crit_chance: f32,
    crit_multiplier: f32,
    element: String,
    // Weapon fields
    weapon_type: String,
    base_damage: f32,
    weapon_grip: String,
    // Economy
    price: f64,
    stackable: bool,
    max_stack: u32,
    // Meta
    required_level: u32,
    item_level: u32,
    is_available: bool,
    tags: String,
    // Server ID for updates
    item_id: String,
}

impl Default for ItemForm {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            icon: "📦".to_string(),
            category: "equipment".to_string(),
            subcategory: "weapon".to_string(),
            rarity: "common".to_string(),
            max_hp: 0.0,
            attack: 0.0,
            defense: 0.0,
            speed: 0.0,
            crit_chance: 0.0,
            crit_multiplier: 0.0,
            element: "Physical".to_string(),
            weapon_type: "Sword".to_string(),
            base_damage: 10.0,
            weapon_grip: "OneHanded".to_string(),
            price: 0.0,
            stackable: false,
            max_stack: 1,
            is_available: true,
            tags: String::new(),
            required_level: 1,
            item_level: 1,
            item_id: String::new(),
        }
    }
}

impl ItemForm {
    fn from_server_item(item: &ServerCharacterItem) -> Self {
        let custom = item.stats.custom.clone().unwrap_or_default();
        Self {
            name: item.name.clone(),
            description: item.description.clone(),
            icon: item.icon.clone(),
            category: item.category.clone(),
            subcategory: item.subcategory.clone(),
            rarity: item.rarity.clone(),
            max_hp: custom.stat_modifiers.as_ref().map(|s| s.max_hp).unwrap_or(0.0),
            attack: custom.stat_modifiers.as_ref().map(|s| s.attack).unwrap_or(0.0),
            defense: custom.stat_modifiers.as_ref().map(|s| s.defense).unwrap_or(0.0),
            speed: custom.stat_modifiers.as_ref().map(|s| s.speed).unwrap_or(0.0),
            crit_chance: custom.stat_modifiers.as_ref().map(|s| s.crit_chance).unwrap_or(0.0),
            crit_multiplier: custom.stat_modifiers.as_ref().map(|s| s.crit_multiplier).unwrap_or(0.0),
            element: custom.element.unwrap_or_else(|| "Physical".to_string()),
            weapon_type: custom.weapon_data.as_ref()
                .map(|w| w.weapon_type.clone())
                .unwrap_or_else(|| "Sword".to_string()),
            base_damage: custom.weapon_data.as_ref()
                .map(|w| w.base_damage)
                .unwrap_or(10.0),
            weapon_grip: custom.weapon_data.as_ref()
                .map(|w| w.weapon_grip.clone())
                .unwrap_or_else(|| "OneHanded".to_string()),
            price: item.price,
            stackable: item.stackable,
            max_stack: item.max_stack,
            is_available: item.is_available,
            tags: item.tags.join(", "),
            required_level: custom.required_level.unwrap_or(1),
            item_level: custom.item_level.unwrap_or(1),
            item_id: item.item_id.clone(),
        }
    }

    fn to_server_item(&self, project_id: &str) -> ServerCharacterItem {
        let game_category = match (self.category.as_str(), self.subcategory.as_str()) {
            ("equipment", "weapon") => "Weapon",
            ("equipment", "armor") => "Armor",
            ("accessory", _) => "Accessory",
            ("consumable", _) => "Consumable",
            ("collectible", _) => "Material",
            _ => "Weapon",
        };

        let weapon_data = if self.category == "equipment" && self.subcategory == "weapon" {
            Some(infinite_integration::types::CustomWeaponData {
                weapon_type: self.weapon_type.clone(),
                base_damage: self.base_damage,
                weapon_grip: self.weapon_grip.clone(),
            })
        } else {
            None
        };

        let custom = GameItemCustomStats {
            stat_modifiers: Some(infinite_integration::types::CustomStatModifiers {
                max_hp: self.max_hp,
                attack: self.attack,
                defense: self.defense,
                speed: self.speed,
                crit_chance: self.crit_chance,
                crit_multiplier: self.crit_multiplier,
            }),
            element: Some(self.element.clone()),
            weapon_data,
            gem_sockets: None,
            item_level: Some(self.item_level),
            required_level: Some(self.required_level),
            game_category: Some(game_category.to_string()),
        };

        let tags: Vec<String> = self.tags
            .split(',')
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect();

        ServerCharacterItem {
            id: None,
            item_id: if self.item_id.is_empty() {
                format!("item_{}", uuid_simple())
            } else {
                self.item_id.clone()
            },
            project_id: Some(project_id.to_string()),
            name: self.name.clone(),
            description: self.description.clone(),
            icon: self.icon.clone(),
            category: self.category.clone(),
            subcategory: self.subcategory.clone(),
            rarity: self.rarity.clone(),
            tags,
            price: self.price,
            stackable: self.stackable,
            max_stack: self.max_stack,
            is_available: self.is_available,
            equip_slot: None,
            stats: ServerItemStats {
                custom: Some(custom),
            },
            effects: vec![],
            requirements: None,
        }
    }
}

/// Simple unique id generator (timestamp-based)
fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("{:x}", t)
}

impl ItemEditor {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected_index: None,
            form: ItemForm::default(),
            creating_new: false,
            pending_list: None,
            pending_save: None,
            pending_delete: None,
            status: None,
        }
    }

    /// Trigger a refresh of the item list from the server
    pub fn refresh_items(&mut self, client: &IntegrationClient) {
        self.pending_list = Some(client.list_project_items());
    }

    /// Main render
    pub fn render(&mut self, ui: &mut Ui, integration_client: Option<&IntegrationClient>) {
        // Poll pending operations
        self.poll_pending(integration_client);

        ui.horizontal(|ui| {
            // Left panel: item list
            let list_width = 250.0;
            ui.vertical(|ui| {
                ui.set_width(list_width);
                ui.set_min_height(ui.available_height());

                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Items")
                            .font(FontId::proportional(18.0))
                            .color(Color32::from_rgb(180, 180, 220)),
                    );

                    if let Some(client) = integration_client {
                        if ui.small_button("Refresh").clicked() {
                            self.refresh_items(client);
                        }
                    }
                });

                ui.add_space(5.0);

                // New item button
                let new_btn = egui::Button::new(
                    RichText::new("+ New Item")
                        .font(FontId::proportional(14.0))
                        .color(Color32::from_rgb(100, 255, 100)),
                )
                .min_size(Vec2::new(list_width - 10.0, 28.0))
                .fill(Color32::from_rgb(40, 60, 40))
                .stroke(egui::Stroke::new(1.0, Color32::from_rgb(60, 100, 60)));

                if ui.add(new_btn).clicked() {
                    self.form = ItemForm::default();
                    self.selected_index = None;
                    self.creating_new = true;
                }

                ui.add_space(5.0);
                ui.separator();

                // Item list
                ScrollArea::vertical()
                    .max_height(ui.available_height() - 10.0)
                    .show(ui, |ui| {
                        let mut clicked_index = None;
                        for (i, item) in self.items.iter().enumerate() {
                            let selected = self.selected_index == Some(i);
                            let bg = if selected {
                                Color32::from_rgb(60, 60, 90)
                            } else {
                                Color32::TRANSPARENT
                            };

                            let rarity_color = rarity_color(&item.rarity);

                            let response = ui.horizontal(|ui| {
                                egui::Frame::new()
                                    .fill(bg)
                                    .corner_radius(3.0)
                                    .inner_margin(4.0)
                                    .show(ui, |ui| {
                                        ui.set_min_width(list_width - 30.0);
                                        ui.horizontal(|ui| {
                                            ui.label(&item.icon);
                                            ui.vertical(|ui| {
                                                ui.label(
                                                    RichText::new(&item.name)
                                                        .font(FontId::proportional(13.0))
                                                        .color(rarity_color),
                                                );
                                                ui.label(
                                                    RichText::new(format!("{} / {}", item.category, item.subcategory))
                                                        .font(FontId::proportional(10.0))
                                                        .color(Color32::from_rgb(120, 120, 140)),
                                                );
                                            });
                                        });
                                    });
                            });
                            if response.response.interact(egui::Sense::click()).clicked() {
                                clicked_index = Some(i);
                            }
                        }

                        if let Some(i) = clicked_index {
                            self.selected_index = Some(i);
                            self.creating_new = false;
                            self.form = ItemForm::from_server_item(&self.items[i]);
                        }
                    });
            });

            ui.separator();

            // Right panel: edit form
            ui.vertical(|ui| {
                if self.creating_new || self.selected_index.is_some() {
                    self.render_form(ui, integration_client);
                } else {
                    ui.add_space(100.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("Select an item or create a new one")
                                .font(FontId::proportional(16.0))
                                .color(Color32::from_rgb(120, 120, 140)),
                        );
                    });
                }
            });
        });
    }

    fn render_form(&mut self, ui: &mut Ui, integration_client: Option<&IntegrationClient>) {
        let title = if self.creating_new { "New Item" } else { "Edit Item" };
        ui.label(
            RichText::new(title)
                .font(FontId::proportional(20.0))
                .color(Color32::from_rgb(200, 200, 255)),
        );

        // Status message
        if let Some((msg, is_err)) = &self.status {
            let color = if *is_err {
                Color32::from_rgb(255, 100, 100)
            } else {
                Color32::from_rgb(100, 255, 100)
            };
            ui.label(RichText::new(msg).font(FontId::proportional(13.0)).color(color));
        }

        ui.add_space(10.0);

        ScrollArea::vertical()
            .max_height(ui.available_height() - 50.0)
            .show(ui, |ui| {
                // Basic section
                section_header(ui, "Basic");
                form_field(ui, "Name", &mut self.form.name);
                form_field(ui, "Description", &mut self.form.description);
                form_field(ui, "Icon", &mut self.form.icon);

                ui.add_space(10.0);

                // Classification
                section_header(ui, "Classification");
                combo_field(ui, "Category", &mut self.form.category,
                    &["equipment", "accessory", "consumable", "collectible"]);
                combo_field(ui, "Subcategory", &mut self.form.subcategory,
                    &["weapon", "armor", "tool", "jewelry", "other"]);
                combo_field(ui, "Rarity", &mut self.form.rarity,
                    &["common", "uncommon", "rare", "epic", "legendary", "mythic"]);

                ui.add_space(10.0);

                // Game Stats
                section_header(ui, "Game Stats");
                stat_slider(ui, "Max HP", &mut self.form.max_hp, 0.0, 500.0);
                stat_slider(ui, "Attack", &mut self.form.attack, 0.0, 100.0);
                stat_slider(ui, "Defense", &mut self.form.defense, 0.0, 100.0);
                stat_slider(ui, "Speed", &mut self.form.speed, 0.0, 50.0);
                stat_slider(ui, "Crit Chance", &mut self.form.crit_chance, 0.0, 1.0);
                stat_slider(ui, "Crit Multiplier", &mut self.form.crit_multiplier, 0.0, 5.0);
                combo_field(ui, "Element", &mut self.form.element,
                    &["Physical", "Fire", "Earth", "Water", "Air", "Void", "Meta"]);

                // Weapon section (only if weapon)
                if self.form.category == "equipment" && self.form.subcategory == "weapon" {
                    ui.add_space(10.0);
                    section_header(ui, "Weapon");
                    combo_field(ui, "Type", &mut self.form.weapon_type,
                        &["Sword", "Axe", "Mace", "Dagger", "Spear", "Bow", "Staff",
                          "Wand", "Halberd", "Crossbow", "Greatsword", "DualBlades",
                          "Scythe", "Hammer", "Whip"]);
                    stat_slider(ui, "Base Damage", &mut self.form.base_damage, 1.0, 200.0);
                    combo_field(ui, "Grip", &mut self.form.weapon_grip,
                        &["OneHanded", "TwoHanded"]);
                }

                ui.add_space(10.0);

                // Economy
                section_header(ui, "Economy");
                ui.horizontal(|ui| {
                    ui.label("Price:");
                    let mut price_f32 = self.form.price as f32;
                    ui.add(egui::DragValue::new(&mut price_f32).range(0.0..=999999.0));
                    self.form.price = price_f32 as f64;
                });
                ui.checkbox(&mut self.form.stackable, "Stackable");
                if self.form.stackable {
                    ui.horizontal(|ui| {
                        ui.label("Max Stack:");
                        let mut ms = self.form.max_stack as i32;
                        ui.add(egui::DragValue::new(&mut ms).range(1..=9999));
                        self.form.max_stack = ms.max(1) as u32;
                    });
                }

                ui.add_space(10.0);

                // Meta
                section_header(ui, "Meta");
                ui.horizontal(|ui| {
                    ui.label("Required Level:");
                    let mut rl = self.form.required_level as i32;
                    ui.add(egui::DragValue::new(&mut rl).range(1..=100));
                    self.form.required_level = rl.max(1) as u32;
                });
                ui.horizontal(|ui| {
                    ui.label("Item Level:");
                    let mut il = self.form.item_level as i32;
                    ui.add(egui::DragValue::new(&mut il).range(1..=100));
                    self.form.item_level = il.max(1) as u32;
                });
                form_field(ui, "Tags (comma-separated)", &mut self.form.tags);
                ui.checkbox(&mut self.form.is_available, "Published");
            });

        // Action buttons
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            let is_busy = self.pending_save.is_some() || self.pending_delete.is_some();

            // Save
            let save_label = if is_busy { "Saving..." } else if self.creating_new { "Create" } else { "Save" };
            let save_btn = egui::Button::new(
                RichText::new(save_label)
                    .font(FontId::proportional(14.0))
                    .color(Color32::from_rgb(220, 220, 240)),
            )
            .min_size(Vec2::new(100.0, 32.0))
            .fill(Color32::from_rgb(40, 70, 40))
            .stroke(egui::Stroke::new(1.0, Color32::from_rgb(60, 100, 60)));

            if ui.add(save_btn).clicked() && !is_busy {
                if let Some(client) = integration_client {
                    let item = self.form.to_server_item("6981e8eda259e89734bd007a");
                    if self.creating_new {
                        self.pending_save = Some(client.create_item(item));
                    } else {
                        self.pending_save = Some(client.update_item(
                            &self.form.item_id,
                            item,
                        ));
                    }
                }
            }

            // Cancel
            if ui.button("Cancel").clicked() {
                self.selected_index = None;
                self.creating_new = false;
                self.status = None;
            }

            // Delete (only for existing items)
            if !self.creating_new && self.selected_index.is_some() {
                ui.add_space(20.0);
                let del_btn = egui::Button::new(
                    RichText::new("Delete")
                        .font(FontId::proportional(14.0))
                        .color(Color32::from_rgb(255, 150, 150)),
                )
                .min_size(Vec2::new(80.0, 32.0))
                .fill(Color32::from_rgb(80, 30, 30))
                .stroke(egui::Stroke::new(1.0, Color32::from_rgb(120, 50, 50)));

                if ui.add(del_btn).clicked() && !is_busy {
                    if let Some(client) = integration_client {
                        self.pending_delete = Some(client.delete_item(&self.form.item_id));
                    }
                }
            }
        });
    }

    fn poll_pending(&mut self, integration_client: Option<&IntegrationClient>) {
        // Poll list
        if let Some(pending) = &self.pending_list {
            if let Some(result) = pending.try_recv() {
                match result {
                    Ok(items) => {
                        self.items = items;
                        self.status = Some((format!("Loaded {} items", self.items.len()), false));
                    }
                    Err(e) => {
                        self.status = Some((format!("Failed to load items: {}", e), true));
                    }
                }
                self.pending_list = None;
            }
        }

        // Poll save
        if let Some(pending) = &self.pending_save {
            if let Some(result) = pending.try_recv() {
                match result {
                    Ok(_item) => {
                        self.status = Some(("Item saved!".to_string(), false));
                        self.creating_new = false;
                        self.selected_index = None;
                        // Refresh list
                        if let Some(client) = integration_client {
                            self.refresh_items(client);
                        }
                    }
                    Err(e) => {
                        self.status = Some((format!("Save failed: {}", e), true));
                    }
                }
                self.pending_save = None;
            }
        }

        // Poll delete
        if let Some(pending) = &self.pending_delete {
            if let Some(result) = pending.try_recv() {
                match result {
                    Ok(_) => {
                        self.status = Some(("Item deleted".to_string(), false));
                        self.selected_index = None;
                        self.creating_new = false;
                        if let Some(client) = integration_client {
                            self.refresh_items(client);
                        }
                    }
                    Err(e) => {
                        self.status = Some((format!("Delete failed: {}", e), true));
                    }
                }
                self.pending_delete = None;
            }
        }
    }
}

// -- UI helpers --

fn section_header(ui: &mut Ui, text: &str) {
    ui.label(
        RichText::new(text)
            .font(FontId::proportional(16.0))
            .color(Color32::from_rgb(180, 180, 220)),
    );
    ui.add_space(3.0);
}

fn form_field(ui: &mut Ui, label: &str, value: &mut String) {
    ui.horizontal(|ui| {
        ui.label(format!("{}:", label));
        ui.text_edit_singleline(value);
    });
}

fn combo_field(ui: &mut Ui, label: &str, value: &mut String, options: &[&str]) {
    ui.horizontal(|ui| {
        ui.label(format!("{}:", label));
        egui::ComboBox::from_id_salt(label)
            .selected_text(value.as_str())
            .show_ui(ui, |ui| {
                for &opt in options {
                    ui.selectable_value(value, opt.to_string(), opt);
                }
            });
    });
}

fn stat_slider(ui: &mut Ui, label: &str, value: &mut f32, min: f32, max: f32) {
    ui.horizontal(|ui| {
        ui.label(format!("{}:", label));
        ui.add(egui::Slider::new(value, min..=max));
    });
}

fn rarity_color(rarity: &str) -> Color32 {
    match rarity {
        "common" => Color32::from_rgb(180, 180, 180),
        "uncommon" => Color32::from_rgb(80, 200, 80),
        "rare" => Color32::from_rgb(80, 130, 255),
        "epic" => Color32::from_rgb(160, 60, 230),
        "legendary" => Color32::from_rgb(255, 160, 0),
        "mythic" => Color32::from_rgb(255, 50, 50),
        _ => Color32::from_rgb(180, 180, 180),
    }
}
