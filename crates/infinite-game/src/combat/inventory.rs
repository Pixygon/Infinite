//! Inventory container
//!
//! Manages item storage with stacking, sorting, and capacity limits.

use serde::{Deserialize, Serialize};

use super::item::{Item, ItemCategory, ItemRarity};

/// Maximum default inventory size
pub const MAX_INVENTORY_SIZE: usize = 40;

/// Player inventory container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    pub items: Vec<Item>,
    pub capacity: usize,
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new()
    }
}

impl Inventory {
    /// Create a new empty inventory with default capacity
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            capacity: MAX_INVENTORY_SIZE,
        }
    }

    /// Add an item to the inventory. Stacks if stackable (same id + category).
    /// Returns `Err(item)` if inventory is full and item cannot be stacked.
    #[allow(clippy::result_large_err)]
    pub fn add_item(&mut self, item: Item) -> Result<(), Item> {
        // Try to stack with existing item
        if item.is_stackable() {
            for existing in &mut self.items {
                if existing.id == item.id && existing.category == item.category
                    && existing.stack_count < existing.max_stack
                {
                    let space = existing.max_stack - existing.stack_count;
                    let to_add = item.stack_count.min(space);
                    existing.stack_count += to_add;
                    if to_add >= item.stack_count {
                        return Ok(());
                    }
                    // Remaining goes to a new slot
                    let mut remainder = item;
                    remainder.stack_count -= to_add;
                    if self.items.len() < self.capacity {
                        self.items.push(remainder);
                        return Ok(());
                    } else {
                        return Err(remainder);
                    }
                }
            }
        }

        // No existing stack found, add to new slot
        if self.items.len() < self.capacity {
            self.items.push(item);
            Ok(())
        } else {
            Err(item)
        }
    }

    /// Remove and return the item at the given index
    pub fn remove_item(&mut self, index: usize) -> Option<Item> {
        if index < self.items.len() {
            Some(self.items.remove(index))
        } else {
            None
        }
    }

    /// Remove `count` from a stack at the given index, returning the removed portion.
    /// If count >= stack_count, removes the entire item.
    pub fn remove_item_stack(&mut self, index: usize, count: u32) -> Option<Item> {
        if index >= self.items.len() {
            return None;
        }

        if count >= self.items[index].stack_count {
            return self.remove_item(index);
        }

        let mut removed = self.items[index].clone();
        removed.stack_count = count;
        self.items[index].stack_count -= count;
        Some(removed)
    }

    /// Get a reference to the item at the given index
    pub fn get(&self, index: usize) -> Option<&Item> {
        self.items.get(index)
    }

    /// Number of occupied slots
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Whether the inventory has no items
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Whether the inventory is at capacity
    pub fn is_full(&self) -> bool {
        self.items.len() >= self.capacity
    }

    /// Sort items by category order: Weapon, Armor, Accessory, Consumable, Material, Gem, Rune
    pub fn sort_by_category(&mut self) {
        self.items.sort_by_key(|item| category_order(item.category));
    }

    /// Sort items by rarity descending (Legendary first)
    pub fn sort_by_rarity(&mut self) {
        self.items.sort_by_key(|item| std::cmp::Reverse(rarity_order(item.rarity)));
    }

    /// Get items filtered by category, with their original indices
    pub fn items_by_category(&self, category: ItemCategory) -> Vec<(usize, &Item)> {
        self.items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.category == category)
            .collect()
    }
}

fn category_order(cat: ItemCategory) -> u8 {
    match cat {
        ItemCategory::Weapon => 0,
        ItemCategory::Armor => 1,
        ItemCategory::Accessory => 2,
        ItemCategory::Consumable => 3,
        ItemCategory::Material => 4,
        ItemCategory::Gem => 5,
        ItemCategory::Rune => 6,
    }
}

fn rarity_order(rarity: ItemRarity) -> u8 {
    match rarity {
        ItemRarity::Common => 0,
        ItemRarity::Uncommon => 1,
        ItemRarity::Rare => 2,
        ItemRarity::Epic => 3,
        ItemRarity::Legendary => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::damage::StatModifiers;
    use crate::combat::element::Element;
    use crate::combat::item::ItemId;

    fn make_weapon(id: u64, name: &str) -> Item {
        Item {
            id: ItemId(id),
            name: name.to_string(),
            description: "A test weapon".to_string(),
            category: ItemCategory::Weapon,
            rarity: ItemRarity::Common,
            stat_modifiers: StatModifiers::default(),
            element: Element::Physical,
            weapon_data: None,
            gem_sockets: vec![],
            required_level: 1,
            item_level: 1,
            stack_count: 1,
            max_stack: 1,
        }
    }

    fn make_potion(id: u64, count: u32) -> Item {
        Item {
            id: ItemId(id),
            name: "Health Potion".to_string(),
            description: "Restores health".to_string(),
            category: ItemCategory::Consumable,
            rarity: ItemRarity::Common,
            stat_modifiers: StatModifiers::default(),
            element: Element::Physical,
            weapon_data: None,
            gem_sockets: vec![],
            required_level: 1,
            item_level: 1,
            stack_count: count,
            max_stack: 10,
        }
    }

    fn make_armor(id: u64, name: &str, rarity: ItemRarity) -> Item {
        Item {
            id: ItemId(id),
            name: name.to_string(),
            description: "Test armor".to_string(),
            category: ItemCategory::Armor,
            rarity,
            stat_modifiers: StatModifiers::default(),
            element: Element::Physical,
            weapon_data: None,
            gem_sockets: vec![],
            required_level: 1,
            item_level: 1,
            stack_count: 1,
            max_stack: 1,
        }
    }

    #[test]
    fn test_new_inventory_empty() {
        let inv = Inventory::new();
        assert!(inv.is_empty());
        assert_eq!(inv.len(), 0);
        assert!(!inv.is_full());
        assert_eq!(inv.capacity, MAX_INVENTORY_SIZE);
    }

    #[test]
    fn test_add_and_get() {
        let mut inv = Inventory::new();
        let sword = make_weapon(1, "Sword");
        inv.add_item(sword).unwrap();
        assert_eq!(inv.len(), 1);
        assert_eq!(inv.get(0).unwrap().name, "Sword");
    }

    #[test]
    fn test_remove_item() {
        let mut inv = Inventory::new();
        inv.add_item(make_weapon(1, "Sword")).unwrap();
        inv.add_item(make_weapon(2, "Axe")).unwrap();
        let removed = inv.remove_item(0).unwrap();
        assert_eq!(removed.name, "Sword");
        assert_eq!(inv.len(), 1);
        assert_eq!(inv.get(0).unwrap().name, "Axe");
    }

    #[test]
    fn test_remove_out_of_bounds() {
        let mut inv = Inventory::new();
        assert!(inv.remove_item(0).is_none());
    }

    #[test]
    fn test_stacking() {
        let mut inv = Inventory::new();
        inv.add_item(make_potion(100, 3)).unwrap();
        inv.add_item(make_potion(100, 2)).unwrap();
        // Should stack into one slot
        assert_eq!(inv.len(), 1);
        assert_eq!(inv.get(0).unwrap().stack_count, 5);
    }

    #[test]
    fn test_stacking_overflow() {
        let mut inv = Inventory::new();
        inv.add_item(make_potion(100, 8)).unwrap();
        inv.add_item(make_potion(100, 5)).unwrap();
        // 8 + 5 = 13, max stack 10, so 10 in first slot, 3 in second
        assert_eq!(inv.len(), 2);
        assert_eq!(inv.get(0).unwrap().stack_count, 10);
        assert_eq!(inv.get(1).unwrap().stack_count, 3);
    }

    #[test]
    fn test_capacity_overflow() {
        let mut inv = Inventory {
            items: Vec::new(),
            capacity: 2,
        };
        inv.add_item(make_weapon(1, "A")).unwrap();
        inv.add_item(make_weapon(2, "B")).unwrap();
        let result = inv.add_item(make_weapon(3, "C"));
        assert!(result.is_err());
        assert_eq!(inv.len(), 2);
    }

    #[test]
    fn test_remove_item_stack_partial() {
        let mut inv = Inventory::new();
        inv.add_item(make_potion(100, 5)).unwrap();
        let removed = inv.remove_item_stack(0, 2).unwrap();
        assert_eq!(removed.stack_count, 2);
        assert_eq!(inv.get(0).unwrap().stack_count, 3);
    }

    #[test]
    fn test_remove_item_stack_all() {
        let mut inv = Inventory::new();
        inv.add_item(make_potion(100, 5)).unwrap();
        let removed = inv.remove_item_stack(0, 5).unwrap();
        assert_eq!(removed.stack_count, 5);
        assert!(inv.is_empty());
    }

    #[test]
    fn test_sort_by_category() {
        let mut inv = Inventory::new();
        inv.add_item(make_potion(100, 1)).unwrap(); // Consumable
        inv.add_item(make_weapon(1, "Sword")).unwrap(); // Weapon
        inv.add_item(make_armor(2, "Helm", ItemRarity::Common)).unwrap(); // Armor
        inv.sort_by_category();
        assert_eq!(inv.get(0).unwrap().category, ItemCategory::Weapon);
        assert_eq!(inv.get(1).unwrap().category, ItemCategory::Armor);
        assert_eq!(inv.get(2).unwrap().category, ItemCategory::Consumable);
    }

    #[test]
    fn test_sort_by_rarity() {
        let mut inv = Inventory::new();
        inv.add_item(make_armor(1, "Common", ItemRarity::Common)).unwrap();
        inv.add_item(make_armor(2, "Legendary", ItemRarity::Legendary)).unwrap();
        inv.add_item(make_armor(3, "Rare", ItemRarity::Rare)).unwrap();
        inv.sort_by_rarity();
        assert_eq!(inv.get(0).unwrap().rarity, ItemRarity::Legendary);
        assert_eq!(inv.get(1).unwrap().rarity, ItemRarity::Rare);
        assert_eq!(inv.get(2).unwrap().rarity, ItemRarity::Common);
    }

    #[test]
    fn test_items_by_category() {
        let mut inv = Inventory::new();
        inv.add_item(make_weapon(1, "Sword")).unwrap();
        inv.add_item(make_potion(100, 3)).unwrap();
        inv.add_item(make_weapon(2, "Axe")).unwrap();
        let weapons = inv.items_by_category(ItemCategory::Weapon);
        assert_eq!(weapons.len(), 2);
        assert_eq!(weapons[0].0, 0); // original index
        assert_eq!(weapons[1].0, 2);
    }
}
