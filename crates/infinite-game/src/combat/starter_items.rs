//! Per-archetype starting gear
//!
//! Creates starter weapons, armor, and consumables for each character archetype.

use super::damage::StatModifiers;
use super::element::Element;
use super::item::{Item, ItemCategory, ItemId, ItemRarity};
use super::weapon::{WeaponData, WeaponType};

/// Create starter items for a given archetype.
/// Returns `(inventory_items, main_hand_weapon)`.
pub fn create_starter_items(
    archetype_name: &str,
    weapon_type: WeaponType,
    element: Element,
) -> (Vec<Item>, Item) {
    let (weapon_name, armor_name) = match archetype_name {
        "Chronomancer" => ("Chrono Staff", "Apprentice Robes"),
        "TemporalHunter" => ("Temporal Blades", "Scout's Leathers"),
        "Vanguard" => ("Guardian Sword", "Iron Plate"),
        "Technomage" => ("Techno Wand", "Tech Vest"),
        "ParadoxWeaver" => ("Paradox Scythe", "Weaver's Cloak"),
        _ => ("Starter Weapon", "Starter Armor"),
    };

    let weapon = create_weapon(weapon_name, weapon_type, element, 8.0);
    let armor = create_armor(armor_name, element);
    let potions = create_health_potion(3);

    let inventory_items = vec![armor, potions];
    (inventory_items, weapon)
}

fn create_weapon(name: &str, weapon_type: WeaponType, element: Element, base_damage: f32) -> Item {
    Item {
        id: ItemId(1000 + weapon_type as u64),
        name: name.to_string(),
        description: format!("A starter {} infused with {} energy.", weapon_type.name(), element.name()),
        category: ItemCategory::Weapon,
        rarity: ItemRarity::Common,
        stat_modifiers: StatModifiers::default(),
        element,
        weapon_data: Some(WeaponData::new(weapon_type, base_damage)),
        gem_sockets: vec![],
        required_level: 1,
        item_level: 1,
        stack_count: 1,
        max_stack: 1,
    }
}

fn create_armor(name: &str, element: Element) -> Item {
    Item {
        id: ItemId(2000),
        name: name.to_string(),
        description: "Basic starter armor.".to_string(),
        category: ItemCategory::Armor,
        rarity: ItemRarity::Common,
        stat_modifiers: StatModifiers {
            max_hp: 10.0,
            defense: 2.0,
            ..Default::default()
        },
        element,
        weapon_data: None,
        gem_sockets: vec![],
        required_level: 1,
        item_level: 1,
        stack_count: 1,
        max_stack: 1,
    }
}

fn create_health_potion(count: u32) -> Item {
    Item {
        id: ItemId(3000),
        name: "Minor Health Potion".to_string(),
        description: "Restores a small amount of health.".to_string(),
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
