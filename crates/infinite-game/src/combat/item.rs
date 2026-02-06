//! Item data model
//!
//! Items with rarity, gem sockets, stat modifiers, and categories.

use serde::{Deserialize, Serialize};

use super::damage::StatModifiers;
use super::element::Element;
use super::gem::{Gem, GemShape};
use super::weapon::WeaponData;

/// Unique item identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ItemId(pub u64);

/// Item rarity tier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

impl ItemRarity {
    /// Maximum gem sockets for this rarity (ordinal value)
    pub fn max_gem_sockets(self) -> usize {
        match self {
            Self::Common => 0,
            Self::Uncommon => 1,
            Self::Rare => 2,
            Self::Epic => 3,
            Self::Legendary => 4,
        }
    }

    /// Display name
    pub fn name(self) -> &'static str {
        match self {
            Self::Common => "Common",
            Self::Uncommon => "Uncommon",
            Self::Rare => "Rare",
            Self::Epic => "Epic",
            Self::Legendary => "Legendary",
        }
    }

    /// Color as [r, g, b] floats
    pub fn color(self) -> [f32; 3] {
        match self {
            Self::Common => [0.7, 0.7, 0.7],
            Self::Uncommon => [0.3, 0.8, 0.3],
            Self::Rare => [0.3, 0.5, 1.0],
            Self::Epic => [0.6, 0.2, 0.9],
            Self::Legendary => [1.0, 0.6, 0.0],
        }
    }
}

/// Item category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemCategory {
    Weapon,
    Armor,
    Accessory,
    Consumable,
    Material,
    Gem,
    Rune,
}

/// A socket on an item that can hold a gem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GemSocket {
    /// Shape of this socket
    pub shape: GemShape,
    /// Gem currently socketed (if any)
    pub gem: Option<Gem>,
}

impl GemSocket {
    /// Create a new empty socket
    pub fn new(shape: GemShape) -> Self {
        Self { shape, gem: None }
    }

    /// Check if a gem fits this socket
    pub fn accepts(&self, gem: &Gem) -> bool {
        gem.fits_socket(self.shape)
    }
}

/// A game item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: ItemId,
    pub name: String,
    pub description: String,
    pub category: ItemCategory,
    pub rarity: ItemRarity,
    /// Base stat modifiers on the item itself
    pub stat_modifiers: StatModifiers,
    /// Element affinity (for weapons/armor)
    pub element: Element,
    /// Weapon-specific data (only for weapons)
    pub weapon_data: Option<WeaponData>,
    /// Gem sockets
    pub gem_sockets: Vec<GemSocket>,
    /// Required player level to equip
    pub required_level: u32,
    /// Item level (determines stat scaling)
    pub item_level: u32,
    /// Current stack count (for stackable items)
    pub stack_count: u32,
    /// Maximum stack size
    pub max_stack: u32,
}

impl Item {
    /// Total stat modifiers including socketed gem bonuses
    pub fn total_modifiers(&self) -> StatModifiers {
        let mut total = self.stat_modifiers.clone();
        for socket in &self.gem_sockets {
            if let Some(gem) = &socket.gem {
                total.add(&gem.effective_modifiers());
            }
        }
        total
    }

    /// Whether this item is stackable
    pub fn is_stackable(&self) -> bool {
        self.max_stack > 1
    }

    /// Whether this item is a weapon
    pub fn is_weapon(&self) -> bool {
        self.category == ItemCategory::Weapon && self.weapon_data.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_item() -> Item {
        Item {
            id: ItemId(1),
            name: "Iron Sword".to_string(),
            description: "A basic sword".to_string(),
            category: ItemCategory::Weapon,
            rarity: ItemRarity::Uncommon,
            stat_modifiers: StatModifiers {
                attack: 5.0,
                ..Default::default()
            },
            element: Element::Physical,
            weapon_data: Some(WeaponData::new(
                super::super::weapon::WeaponType::Sword,
                10.0,
            )),
            gem_sockets: vec![GemSocket::new(GemShape::Circle)],
            required_level: 1,
            item_level: 5,
            stack_count: 1,
            max_stack: 1,
        }
    }

    #[test]
    fn test_item_is_weapon() {
        let item = test_item();
        assert!(item.is_weapon());
    }

    #[test]
    fn test_item_not_stackable() {
        let item = test_item();
        assert!(!item.is_stackable());
    }

    #[test]
    fn test_total_modifiers_no_gems() {
        let item = test_item();
        let mods = item.total_modifiers();
        assert_eq!(mods.attack, 5.0);
    }

    #[test]
    fn test_rarity_sockets() {
        assert_eq!(ItemRarity::Common.max_gem_sockets(), 0);
        assert_eq!(ItemRarity::Uncommon.max_gem_sockets(), 1);
        assert_eq!(ItemRarity::Rare.max_gem_sockets(), 2);
        assert_eq!(ItemRarity::Epic.max_gem_sockets(), 3);
        assert_eq!(ItemRarity::Legendary.max_gem_sockets(), 4);
    }
}
