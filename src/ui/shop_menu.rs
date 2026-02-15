//! Shop UI — buy and sell items from a catalog

use egui::{Color32, FontId, RichText, ScrollArea, Ui, Vec2};

use infinite_game::combat::catalog::ItemCatalog;
use infinite_game::combat::inventory::Inventory;
use infinite_game::combat::item::{Item, ItemCategory, ItemRarity};
use infinite_game::Element;

/// Active tab in the shop
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShopTab {
    Buy,
    Sell,
}

/// Category filter for the buy tab
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CategoryFilter {
    All,
    Category(ItemCategory),
}

/// Action returned by the shop after rendering
#[derive(Debug, Clone)]
pub enum ShopAction {
    None,
    Buy { catalog_index: usize },
    Sell { inventory_index: usize },
    Close,
}

/// Shop menu state
pub struct ShopMenu {
    pub active_tab: ShopTab,
    selected_buy_item: Option<usize>,
    selected_sell_item: Option<usize>,
    category_filter: CategoryFilter,
}

impl Default for ShopMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl ShopMenu {
    pub fn new() -> Self {
        Self {
            active_tab: ShopTab::Buy,
            selected_buy_item: None,
            selected_sell_item: None,
            category_filter: CategoryFilter::All,
        }
    }

    /// Reset sell item selection (called after selling an item)
    pub fn selected_sell_item_reset(&mut self) {
        self.selected_sell_item = None;
    }

    pub fn render(
        &mut self,
        ui: &mut Ui,
        catalog: &ItemCatalog,
        inventory: &Inventory,
        gold: u64,
    ) -> ShopAction {
        let mut action = ShopAction::None;

        // Semi-transparent background overlay
        let painter = ui.painter();
        painter.rect_filled(
            ui.max_rect(),
            0.0,
            Color32::from_rgba_unmultiplied(0, 0, 0, 200),
        );

        let available = ui.available_size();

        ui.vertical_centered(|ui| {
            ui.add_space(available.y * 0.03);

            // Title
            ui.label(
                RichText::new("SHOP")
                    .font(FontId::proportional(48.0))
                    .color(Color32::from_rgb(255, 215, 0)),
            );

            ui.add_space(5.0);

            // Gold display
            ui.label(
                RichText::new(format!("Gold: {}", format_gold(gold)))
                    .font(FontId::proportional(18.0))
                    .color(Color32::from_rgb(255, 215, 0)),
            );

            ui.add_space(10.0);

            // Tab buttons
            ui.horizontal(|ui| {
                ui.add_space(available.x * 0.35);
                if tab_button(ui, "Buy", self.active_tab == ShopTab::Buy) {
                    self.active_tab = ShopTab::Buy;
                    self.selected_buy_item = None;
                    self.selected_sell_item = None;
                }
                ui.add_space(10.0);
                if tab_button(ui, "Sell", self.active_tab == ShopTab::Sell) {
                    self.active_tab = ShopTab::Sell;
                    self.selected_buy_item = None;
                    self.selected_sell_item = None;
                }
            });

            ui.add_space(10.0);

            // Content area
            let content_width = available.x * 0.8;
            let content_height = available.y * 0.6;

            ui.allocate_ui(Vec2::new(content_width, content_height), |ui| {
                match self.active_tab {
                    ShopTab::Buy => {
                        action = self.render_buy_tab(ui, catalog, gold, inventory);
                    }
                    ShopTab::Sell => {
                        action = self.render_sell_tab(ui, catalog, inventory);
                    }
                }
            });

            ui.add_space(15.0);

            // Close button
            if shop_button(ui, "Close", Vec2::new(120.0, 36.0)) {
                action = ShopAction::Close;
            }
        });

        action
    }

    fn render_buy_tab(
        &mut self,
        ui: &mut Ui,
        catalog: &ItemCatalog,
        gold: u64,
        inventory: &Inventory,
    ) -> ShopAction {
        let mut action = ShopAction::None;

        ui.horizontal(|ui| {
            // Left: category filter buttons
            ui.vertical(|ui| {
                ui.set_min_width(100.0);
                ui.label(
                    RichText::new("Filter")
                        .font(FontId::proportional(14.0))
                        .color(Color32::from_rgb(200, 200, 255)),
                );
                ui.add_space(5.0);

                let filters: &[(&str, CategoryFilter)] = &[
                    ("All", CategoryFilter::All),
                    ("Weapons", CategoryFilter::Category(ItemCategory::Weapon)),
                    ("Armor", CategoryFilter::Category(ItemCategory::Armor)),
                    ("Accessories", CategoryFilter::Category(ItemCategory::Accessory)),
                    ("Consumables", CategoryFilter::Category(ItemCategory::Consumable)),
                    ("Materials", CategoryFilter::Category(ItemCategory::Material)),
                    ("Gems", CategoryFilter::Category(ItemCategory::Gem)),
                    ("Runes", CategoryFilter::Category(ItemCategory::Rune)),
                ];

                for &(label, filter) in filters {
                    let is_active = self.category_filter == filter;
                    if filter_button(ui, label, is_active) {
                        self.category_filter = filter;
                        self.selected_buy_item = None;
                    }
                }
            });

            ui.add_space(10.0);

            // Center: item grid
            ui.vertical(|ui| {
                ui.set_min_width(250.0);
                ui.label(
                    RichText::new(format!("Items ({})", catalog.len()))
                        .font(FontId::proportional(14.0))
                        .color(Color32::from_rgb(200, 200, 255)),
                );
                ui.add_space(5.0);

                ScrollArea::vertical()
                    .max_height(ui.available_height())
                    .show(ui, |ui| {
                        let filtered: Vec<(usize, &Item)> = match self.category_filter {
                            CategoryFilter::All => catalog.items().iter().enumerate().collect(),
                            CategoryFilter::Category(cat) => catalog.items_by_category(cat),
                        };

                        for (catalog_idx, item) in &filtered {
                            let price = catalog.price(*catalog_idx);
                            let is_selected = self.selected_buy_item == Some(*catalog_idx);
                            let can_afford = gold >= price;
                            if catalog_item_button(ui, item, price, is_selected, can_afford) {
                                if self.selected_buy_item == Some(*catalog_idx) {
                                    self.selected_buy_item = None;
                                } else {
                                    self.selected_buy_item = Some(*catalog_idx);
                                }
                            }
                        }
                    });
            });

            ui.add_space(15.0);

            // Right: detail panel
            ui.vertical(|ui| {
                ui.set_min_width(200.0);
                if let Some(idx) = self.selected_buy_item {
                    if let Some(item) = catalog.items().get(idx) {
                        let price = catalog.price(idx);
                        render_item_detail(ui, item);

                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(format!("Price: {} gold", format_gold(price)))
                                .font(FontId::proportional(14.0))
                                .color(Color32::from_rgb(255, 215, 0)),
                        );

                        ui.add_space(8.0);

                        let can_afford = gold >= price;
                        let inv_full = inventory.is_full();

                        if !can_afford {
                            ui.label(
                                RichText::new("Not enough gold")
                                    .font(FontId::proportional(12.0))
                                    .color(Color32::from_rgb(220, 100, 100)),
                            );
                        } else if inv_full {
                            ui.label(
                                RichText::new("Inventory full")
                                    .font(FontId::proportional(12.0))
                                    .color(Color32::from_rgb(220, 100, 100)),
                            );
                        }

                        let enabled = can_afford && !inv_full;
                        if buy_sell_button(ui, "Buy", enabled) {
                            action = ShopAction::Buy { catalog_index: idx };
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

    fn render_sell_tab(
        &mut self,
        ui: &mut Ui,
        catalog: &ItemCatalog,
        inventory: &Inventory,
    ) -> ShopAction {
        let mut action = ShopAction::None;

        ui.horizontal(|ui| {
            // Left: inventory item grid
            ui.vertical(|ui| {
                ui.set_min_width(300.0);
                ui.label(
                    RichText::new(format!("Your Items ({}/{})", inventory.len(), inventory.capacity))
                        .font(FontId::proportional(14.0))
                        .color(Color32::from_rgb(200, 200, 255)),
                );
                ui.add_space(5.0);

                ScrollArea::vertical()
                    .max_height(ui.available_height())
                    .show(ui, |ui| {
                        for (idx, item) in inventory.items.iter().enumerate() {
                            let sell_price = sell_price_for(item, catalog);
                            let is_selected = self.selected_sell_item == Some(idx);
                            if sell_item_button(ui, item, sell_price, is_selected) {
                                if self.selected_sell_item == Some(idx) {
                                    self.selected_sell_item = None;
                                } else {
                                    self.selected_sell_item = Some(idx);
                                }
                            }
                        }

                        if inventory.items.is_empty() {
                            ui.label(
                                RichText::new("No items to sell")
                                    .color(Color32::from_rgb(140, 140, 160)),
                            );
                        }
                    });
            });

            ui.add_space(15.0);

            // Right: detail panel
            ui.vertical(|ui| {
                ui.set_min_width(200.0);
                if let Some(idx) = self.selected_sell_item {
                    if let Some(item) = inventory.get(idx) {
                        let sell = sell_price_for(item, catalog);
                        render_item_detail(ui, item);

                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(format!("Sell for: {} gold", format_gold(sell)))
                                .font(FontId::proportional(14.0))
                                .color(Color32::from_rgb(255, 215, 0)),
                        );

                        ui.add_space(8.0);
                        if buy_sell_button(ui, "Sell", true) {
                            action = ShopAction::Sell { inventory_index: idx };
                        }
                    }
                } else {
                    ui.label(
                        RichText::new("Select an item to sell")
                            .color(Color32::from_rgb(140, 140, 160)),
                    );
                }
            });
        });

        action
    }
}

/// Calculate sell price: 50% of catalog price if found, else rarity-based fallback, min 1
pub fn sell_price_for(item: &Item, catalog: &ItemCatalog) -> u64 {
    // Try to find this item in the catalog by name
    for (idx, cat_item) in catalog.items().iter().enumerate() {
        if cat_item.name == item.name {
            return (catalog.price(idx) / 2).max(1);
        }
    }
    // Fallback: rarity-based
    let base = match item.rarity {
        ItemRarity::Common => 2,
        ItemRarity::Uncommon => 5,
        ItemRarity::Rare => 15,
        ItemRarity::Epic => 50,
        ItemRarity::Legendary => 200,
    };
    base
}

fn format_gold(amount: u64) -> String {
    if amount >= 1_000_000 {
        format!("{:.1}M", amount as f64 / 1_000_000.0)
    } else if amount >= 1_000 {
        format!("{},{:03}", amount / 1000, amount % 1000)
    } else {
        amount.to_string()
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
        .min_size(Vec2::new(100.0, 32.0))
        .fill(fill)
        .stroke(egui::Stroke::new(1.0, stroke_color)),
    )
    .clicked()
}

fn filter_button(ui: &mut Ui, text: &str, active: bool) -> bool {
    let fill = if active {
        Color32::from_rgba_unmultiplied(60, 60, 90, 220)
    } else {
        Color32::from_rgba_unmultiplied(40, 40, 55, 200)
    };
    let stroke_color = if active {
        Color32::from_rgb(100, 100, 160)
    } else {
        Color32::from_rgb(60, 60, 80)
    };

    ui.add(
        egui::Button::new(
            RichText::new(text)
                .font(FontId::proportional(12.0))
                .color(if active {
                    Color32::from_rgb(220, 220, 240)
                } else {
                    Color32::from_rgb(160, 160, 180)
                }),
        )
        .min_size(Vec2::new(90.0, 24.0))
        .fill(fill)
        .stroke(egui::Stroke::new(1.0, stroke_color)),
    )
    .clicked()
}

fn shop_button(ui: &mut Ui, text: &str, size: Vec2) -> bool {
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

fn buy_sell_button(ui: &mut Ui, text: &str, enabled: bool) -> bool {
    let fill = if enabled {
        Color32::from_rgba_unmultiplied(50, 80, 50, 220)
    } else {
        Color32::from_rgba_unmultiplied(50, 50, 50, 180)
    };
    let text_color = if enabled {
        Color32::from_rgb(100, 220, 100)
    } else {
        Color32::from_rgb(100, 100, 100)
    };

    let response = ui.add(
        egui::Button::new(
            RichText::new(text)
                .font(FontId::proportional(14.0))
                .color(text_color),
        )
        .min_size(Vec2::new(120.0, 32.0))
        .fill(fill)
        .stroke(egui::Stroke::new(
            1.0,
            if enabled {
                Color32::from_rgb(80, 140, 80)
            } else {
                Color32::from_rgb(60, 60, 60)
            },
        )),
    );

    enabled && response.clicked()
}

fn catalog_item_button(
    ui: &mut Ui,
    item: &Item,
    price: u64,
    selected: bool,
    can_afford: bool,
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

    let name_color = if can_afford {
        rarity_color(item.rarity)
    } else {
        Color32::from_rgb(100, 100, 110)
    };

    let price_color = if can_afford {
        Color32::from_rgb(255, 215, 0)
    } else {
        Color32::from_rgb(140, 110, 60)
    };

    ui.horizontal(|ui| {
        let response = ui.add(
            egui::Button::new(
                RichText::new(&item.name)
                    .font(FontId::proportional(12.0))
                    .color(name_color),
            )
            .min_size(Vec2::new(180.0, 24.0))
            .fill(fill)
            .stroke(egui::Stroke::new(1.0, stroke_color)),
        );
        ui.label(
            RichText::new(format!("{}g", price))
                .font(FontId::proportional(11.0))
                .color(price_color),
        );
        response.clicked()
    })
    .inner
}

fn sell_item_button(
    ui: &mut Ui,
    item: &Item,
    sell_price: u64,
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

    let label = if item.stack_count > 1 {
        format!("{} x{}", item.name, item.stack_count)
    } else {
        item.name.clone()
    };

    ui.horizontal(|ui| {
        let response = ui.add(
            egui::Button::new(
                RichText::new(&label)
                    .font(FontId::proportional(12.0))
                    .color(rarity_color(item.rarity)),
            )
            .min_size(Vec2::new(200.0, 24.0))
            .fill(fill)
            .stroke(egui::Stroke::new(1.0, stroke_color)),
        );
        ui.label(
            RichText::new(format!("{}g", sell_price))
                .font(FontId::proportional(11.0))
                .color(Color32::from_rgb(255, 215, 0)),
        );
        response.clicked()
    })
    .inner
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
    if item.element != Element::Physical {
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
    if mods.max_hp != 0.0 { stat_line(ui, "Max HP", mods.max_hp); }
    if mods.attack != 0.0 { stat_line(ui, "Attack", mods.attack); }
    if mods.defense != 0.0 { stat_line(ui, "Defense", mods.defense); }
    if mods.speed != 0.0 { stat_line(ui, "Speed", mods.speed); }
    if mods.crit_chance != 0.0 { stat_line(ui, "Crit Chance", mods.crit_chance * 100.0); }
    if mods.crit_multiplier != 0.0 { stat_line(ui, "Crit Damage", mods.crit_multiplier); }

    // Weapon data
    if let Some(wd) = &item.weapon_data {
        ui.add_space(4.0);
        ui.label(
            RichText::new(format!("Damage: {:.0}  ({})", wd.base_damage, wd.weapon_type.name()))
                .font(FontId::proportional(12.0))
                .color(Color32::from_rgb(200, 180, 140)),
        );
    }

    if item.required_level > 1 {
        ui.label(
            RichText::new(format!("Requires Level {}", item.required_level))
                .font(FontId::proportional(11.0))
                .color(Color32::from_rgb(200, 160, 100)),
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
