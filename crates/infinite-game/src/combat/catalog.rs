//! Item catalog — cached game items loaded from the server

use super::item::{Item, ItemCategory};
use super::item_conversion::server_to_game_item;
use infinite_integration::types::ServerCharacterItem;

/// A catalog of items loaded from the server, with prices.
pub struct ItemCatalog {
    /// Converted game items
    items: Vec<Item>,
    /// Price per item (index-aligned with items)
    prices: Vec<u64>,
    /// Server item_id for each item (index-aligned)
    server_item_ids: Vec<String>,
}

impl ItemCatalog {
    /// Build a catalog from server items.
    /// Items that fail conversion are silently skipped.
    pub fn load_from_server(server_items: Vec<ServerCharacterItem>) -> Self {
        let mut items = Vec::new();
        let mut prices = Vec::new();
        let mut server_item_ids = Vec::new();

        for si in &server_items {
            if let Some(game_item) = server_to_game_item(si) {
                // Price comes from server; round to u64, minimum 1
                let price = (si.price as u64).max(1);
                server_item_ids.push(si.item_id.clone());
                items.push(game_item);
                prices.push(price);
            }
        }

        Self {
            items,
            prices,
            server_item_ids,
        }
    }

    /// All items in the catalog.
    pub fn items(&self) -> &[Item] {
        &self.items
    }

    /// Price of the item at `index`.
    pub fn price(&self, index: usize) -> u64 {
        self.prices.get(index).copied().unwrap_or(0)
    }

    /// Server item_id of the item at `index`.
    #[allow(dead_code)]
    pub fn server_item_id(&self, index: usize) -> Option<&str> {
        self.server_item_ids.get(index).map(|s| s.as_str())
    }

    /// Filter items by category, returning `(catalog_index, &Item)` pairs.
    pub fn items_by_category(&self, cat: ItemCategory) -> Vec<(usize, &Item)> {
        self.items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.category == cat)
            .collect()
    }

    /// Number of items in the catalog.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Whether the catalog is empty.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use infinite_integration::types::*;

    fn make_server_item(name: &str, price: f64) -> ServerCharacterItem {
        ServerCharacterItem {
            id: None,
            item_id: format!("test_{}", name.to_lowercase().replace(' ', "_")),
            project_id: Some("proj".to_string()),
            name: name.to_string(),
            description: "A test item".to_string(),
            icon: "📦".to_string(),
            category: "equipment".to_string(),
            subcategory: "weapon".to_string(),
            rarity: "common".to_string(),
            tags: vec![],
            price,
            stackable: false,
            max_stack: 1,
            is_available: true,
            equip_slot: None,
            stats: ServerItemStats {
                custom: Some(GameItemCustomStats {
                    stat_modifiers: Some(CustomStatModifiers {
                        max_hp: 0.0,
                        attack: 10.0,
                        defense: 0.0,
                        speed: 0.0,
                        crit_chance: 0.0,
                        crit_multiplier: 0.0,
                    }),
                    element: Some("Physical".to_string()),
                    weapon_data: Some(CustomWeaponData {
                        weapon_type: "Sword".to_string(),
                        base_damage: 15.0,
                        weapon_grip: "OneHanded".to_string(),
                    }),
                    gem_sockets: Some(0),
                    item_level: Some(1),
                    required_level: Some(1),
                    game_category: Some("Weapon".to_string()),
                }),
            },
            effects: vec![],
            requirements: None,
        }
    }

    #[test]
    fn test_load_from_server() {
        let server_items = vec![
            make_server_item("Iron Sword", 50.0),
            make_server_item("Steel Axe", 120.0),
        ];
        let catalog = ItemCatalog::load_from_server(server_items);
        assert_eq!(catalog.len(), 2);
        assert_eq!(catalog.price(0), 50);
        assert_eq!(catalog.price(1), 120);
        assert_eq!(catalog.items()[0].name, "Iron Sword");
    }

    #[test]
    fn test_items_by_category() {
        let server_items = vec![
            make_server_item("Iron Sword", 50.0),
        ];
        let catalog = ItemCatalog::load_from_server(server_items);
        let weapons = catalog.items_by_category(ItemCategory::Weapon);
        assert_eq!(weapons.len(), 1);
        assert_eq!(weapons[0].0, 0);
    }

    #[test]
    fn test_minimum_price() {
        let server_items = vec![make_server_item("Free Sword", 0.0)];
        let catalog = ItemCatalog::load_from_server(server_items);
        assert_eq!(catalog.price(0), 1); // minimum 1 gold
    }
}
