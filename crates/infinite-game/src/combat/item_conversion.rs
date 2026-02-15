//! Conversion between server CharacterItem and game Item types

use infinite_integration::types::{
    CustomStatModifiers, CustomWeaponData, GameItemCustomStats, ServerCharacterItem,
    ServerItemStats,
};

use super::damage::StatModifiers;
use super::element::Element;
use super::gem::GemShape;
use super::item::{GemSocket, Item, ItemCategory, ItemId, ItemRarity};
use super::weapon::{WeaponData, WeaponGrip, WeaponType};

// WeaponGrip is read from WeaponType::grip() — no grip field on WeaponData

/// Convert a server CharacterItem to a game Item
pub fn server_to_game_item(server: &ServerCharacterItem) -> Option<Item> {
    let custom = server.stats.custom.as_ref()?;

    let category = match custom.game_category.as_deref() {
        Some("Weapon") => ItemCategory::Weapon,
        Some("Armor") => ItemCategory::Armor,
        Some("Accessory") => ItemCategory::Accessory,
        Some("Consumable") => ItemCategory::Consumable,
        Some("Material") => ItemCategory::Material,
        Some("Gem") => ItemCategory::Gem,
        Some("Rune") => ItemCategory::Rune,
        _ => match (server.category.as_str(), server.subcategory.as_str()) {
            ("equipment", "weapon") => ItemCategory::Weapon,
            ("equipment", "armor") => ItemCategory::Armor,
            ("accessory", _) => ItemCategory::Accessory,
            ("consumable", _) => ItemCategory::Consumable,
            ("collectible", _) => ItemCategory::Material,
            _ => ItemCategory::Material,
        },
    };

    let rarity = match server.rarity.as_str() {
        "common" => ItemRarity::Common,
        "uncommon" => ItemRarity::Uncommon,
        "rare" => ItemRarity::Rare,
        "epic" => ItemRarity::Epic,
        "legendary" | "mythic" => ItemRarity::Legendary,
        _ => ItemRarity::Common,
    };

    let stat_modifiers = if let Some(mods) = &custom.stat_modifiers {
        StatModifiers {
            max_hp: mods.max_hp,
            attack: mods.attack,
            defense: mods.defense,
            speed: mods.speed,
            crit_chance: mods.crit_chance,
            crit_multiplier: mods.crit_multiplier,
            ..Default::default()
        }
    } else {
        StatModifiers::default()
    };

    let element = parse_element(custom.element.as_deref().unwrap_or("Physical"));

    let weapon_data = custom.weapon_data.as_ref().map(|w| {
        let weapon_type = parse_weapon_type(&w.weapon_type);
        WeaponData::new(weapon_type, w.base_damage)
    });

    let gem_sockets = (0..custom.gem_sockets.unwrap_or(0))
        .map(|_| GemSocket::new(GemShape::Circle))
        .collect();

    // Use a hash of the item_id string as the numeric ItemId
    let id_hash = hash_string(&server.item_id);

    Some(Item {
        id: ItemId(id_hash),
        name: server.name.clone(),
        description: server.description.clone(),
        category,
        rarity,
        stat_modifiers,
        element,
        weapon_data,
        gem_sockets,
        required_level: custom.required_level.unwrap_or(1),
        item_level: custom.item_level.unwrap_or(1),
        stack_count: 1,
        max_stack: server.max_stack,
    })
}

/// Convert a game Item back to a server CharacterItem
pub fn game_item_to_server(item: &Item, project_id: &str) -> ServerCharacterItem {
    let game_category = match item.category {
        ItemCategory::Weapon => "Weapon",
        ItemCategory::Armor => "Armor",
        ItemCategory::Accessory => "Accessory",
        ItemCategory::Consumable => "Consumable",
        ItemCategory::Material => "Material",
        ItemCategory::Gem => "Gem",
        ItemCategory::Rune => "Rune",
    };

    let (server_category, server_subcategory) = match item.category {
        ItemCategory::Weapon => ("equipment", "weapon"),
        ItemCategory::Armor => ("equipment", "armor"),
        ItemCategory::Accessory => ("accessory", "jewelry"),
        ItemCategory::Consumable => ("consumable", "other"),
        ItemCategory::Material => ("collectible", "other"),
        ItemCategory::Gem => ("equipment", "tool"),
        ItemCategory::Rune => ("equipment", "tool"),
    };

    let rarity = match item.rarity {
        ItemRarity::Common => "common",
        ItemRarity::Uncommon => "uncommon",
        ItemRarity::Rare => "rare",
        ItemRarity::Epic => "epic",
        ItemRarity::Legendary => "legendary",
    };

    let weapon_data = item.weapon_data.as_ref().map(|w| CustomWeaponData {
        weapon_type: format!("{:?}", w.weapon_type),
        base_damage: w.base_damage,
        weapon_grip: match w.weapon_type.grip() {
            WeaponGrip::OneHanded => "OneHanded".to_string(),
            WeaponGrip::TwoHanded => "TwoHanded".to_string(),
        },
    });

    let custom = GameItemCustomStats {
        stat_modifiers: Some(CustomStatModifiers {
            max_hp: item.stat_modifiers.max_hp,
            attack: item.stat_modifiers.attack,
            defense: item.stat_modifiers.defense,
            speed: item.stat_modifiers.speed,
            crit_chance: item.stat_modifiers.crit_chance,
            crit_multiplier: item.stat_modifiers.crit_multiplier,
        }),
        element: Some(format!("{:?}", item.element)),
        weapon_data,
        gem_sockets: Some(item.gem_sockets.len() as u32),
        item_level: Some(item.item_level),
        required_level: Some(item.required_level),
        game_category: Some(game_category.to_string()),
    };

    ServerCharacterItem {
        id: None,
        item_id: format!("item_{}", item.id.0),
        project_id: Some(project_id.to_string()),
        name: item.name.clone(),
        description: item.description.clone(),
        icon: "📦".to_string(),
        category: server_category.to_string(),
        subcategory: server_subcategory.to_string(),
        rarity: rarity.to_string(),
        tags: Vec::new(),
        price: 0.0,
        stackable: item.max_stack > 1,
        max_stack: item.max_stack,
        is_available: true,
        equip_slot: None,
        stats: ServerItemStats {
            custom: Some(custom),
        },
        effects: Vec::new(),
        requirements: None,
    }
}

fn parse_element(s: &str) -> Element {
    match s {
        "Fire" => Element::Fire,
        "Earth" => Element::Earth,
        "Water" => Element::Water,
        "Air" => Element::Air,
        "Void" => Element::Void,
        "Meta" => Element::Meta,
        _ => Element::Physical,
    }
}

fn parse_weapon_type(s: &str) -> WeaponType {
    match s {
        "Sword" => WeaponType::Sword,
        "Axe" => WeaponType::Axe,
        "Mace" => WeaponType::Mace,
        "Dagger" => WeaponType::Dagger,
        "Spear" => WeaponType::Spear,
        "Bow" => WeaponType::Bow,
        "Staff" => WeaponType::Staff,
        "Wand" => WeaponType::Wand,
        "Halberd" => WeaponType::Halberd,
        "Crossbow" => WeaponType::Crossbow,
        "Greatsword" => WeaponType::Greatsword,
        "DualBlades" => WeaponType::DualBlades,
        "Scythe" => WeaponType::Scythe,
        "Hammer" => WeaponType::Hammer,
        "Whip" => WeaponType::Whip,
        _ => WeaponType::Sword,
    }
}

fn hash_string(s: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_to_game_roundtrip() {
        let server = ServerCharacterItem {
            id: None,
            item_id: "test_sword".to_string(),
            project_id: Some("proj".to_string()),
            name: "Test Sword".to_string(),
            description: "A test".to_string(),
            icon: "🗡️".to_string(),
            category: "equipment".to_string(),
            subcategory: "weapon".to_string(),
            rarity: "rare".to_string(),
            tags: vec![],
            price: 100.0,
            stackable: false,
            max_stack: 1,
            is_available: true,
            equip_slot: None,
            stats: ServerItemStats {
                custom: Some(GameItemCustomStats {
                    stat_modifiers: Some(CustomStatModifiers {
                        max_hp: 0.0,
                        attack: 15.0,
                        defense: 0.0,
                        speed: 5.0,
                        crit_chance: 0.1,
                        crit_multiplier: 1.5,
                    }),
                    element: Some("Fire".to_string()),
                    weapon_data: Some(CustomWeaponData {
                        weapon_type: "Sword".to_string(),
                        base_damage: 20.0,
                        weapon_grip: "OneHanded".to_string(),
                    }),
                    gem_sockets: Some(2),
                    item_level: Some(10),
                    required_level: Some(5),
                    game_category: Some("Weapon".to_string()),
                }),
            },
            effects: vec![],
            requirements: None,
        };

        let game_item = server_to_game_item(&server).unwrap();
        assert_eq!(game_item.name, "Test Sword");
        assert_eq!(game_item.category, ItemCategory::Weapon);
        assert_eq!(game_item.rarity, ItemRarity::Rare);
        assert_eq!(game_item.element, Element::Fire);
        assert_eq!(game_item.stat_modifiers.attack, 15.0);
        assert!(game_item.weapon_data.is_some());
        assert_eq!(game_item.gem_sockets.len(), 2);
    }
}
