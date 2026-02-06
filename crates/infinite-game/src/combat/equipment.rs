//! Equipment system with 14 slots
//!
//! Manages equipping/unequipping items, validates slot compatibility,
//! and computes total stat modifiers from all equipped items.

use serde::{Deserialize, Serialize};
use std::fmt;

use super::damage::StatModifiers;
use super::item::{Item, ItemCategory};
use super::weapon::{WeaponGrip, WeaponType};

/// The 14 equipment slots
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EquipmentSlot {
    Head,
    Shoulders,
    Chest,
    Bracers,
    Gloves,
    Belt,
    Legs,
    Boots,
    Cape,
    MainHand,
    OffHand,
    Ring1,
    Ring2,
    Amulet,
}

impl EquipmentSlot {
    /// All equipment slot variants
    pub fn all() -> &'static [EquipmentSlot] {
        &[
            Self::Head,
            Self::Shoulders,
            Self::Chest,
            Self::Bracers,
            Self::Gloves,
            Self::Belt,
            Self::Legs,
            Self::Boots,
            Self::Cape,
            Self::MainHand,
            Self::OffHand,
            Self::Ring1,
            Self::Ring2,
            Self::Amulet,
        ]
    }

    /// Display name
    pub fn name(self) -> &'static str {
        match self {
            Self::Head => "Head",
            Self::Shoulders => "Shoulders",
            Self::Chest => "Chest",
            Self::Bracers => "Bracers",
            Self::Gloves => "Gloves",
            Self::Belt => "Belt",
            Self::Legs => "Legs",
            Self::Boots => "Boots",
            Self::Cape => "Cape",
            Self::MainHand => "Main Hand",
            Self::OffHand => "Off Hand",
            Self::Ring1 => "Ring 1",
            Self::Ring2 => "Ring 2",
            Self::Amulet => "Amulet",
        }
    }

    /// Which item category is valid for this slot
    pub fn valid_category(self) -> ItemCategory {
        match self {
            Self::MainHand | Self::OffHand => ItemCategory::Weapon,
            Self::Ring1 | Self::Ring2 | Self::Amulet => ItemCategory::Accessory,
            _ => ItemCategory::Armor,
        }
    }
}

/// Error when equipping an item
#[derive(Debug, Clone)]
pub enum EquipError {
    /// Item category doesn't match slot
    WrongCategory { expected: ItemCategory, got: ItemCategory },
    /// Two-handed weapon conflicts with off-hand
    TwoHandedConflict,
    /// Can't equip in off-hand when main hand has a two-handed weapon
    MainHandIsTwoHanded,
}

impl fmt::Display for EquipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongCategory { expected, got } => {
                write!(f, "Slot requires {:?}, got {:?}", expected, got)
            }
            Self::TwoHandedConflict => {
                write!(f, "Two-handed weapon requires both hands")
            }
            Self::MainHandIsTwoHanded => {
                write!(f, "Cannot equip off-hand with a two-handed main weapon")
            }
        }
    }
}

/// The player's full set of equipped items
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EquipmentSet {
    pub head: Option<Item>,
    pub shoulders: Option<Item>,
    pub chest: Option<Item>,
    pub bracers: Option<Item>,
    pub gloves: Option<Item>,
    pub belt: Option<Item>,
    pub legs: Option<Item>,
    pub boots: Option<Item>,
    pub cape: Option<Item>,
    pub main_hand: Option<Item>,
    pub off_hand: Option<Item>,
    pub ring1: Option<Item>,
    pub ring2: Option<Item>,
    pub amulet: Option<Item>,
}

impl EquipmentSet {
    /// Create an empty equipment set
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a reference to the item in a slot
    pub fn get(&self, slot: EquipmentSlot) -> &Option<Item> {
        match slot {
            EquipmentSlot::Head => &self.head,
            EquipmentSlot::Shoulders => &self.shoulders,
            EquipmentSlot::Chest => &self.chest,
            EquipmentSlot::Bracers => &self.bracers,
            EquipmentSlot::Gloves => &self.gloves,
            EquipmentSlot::Belt => &self.belt,
            EquipmentSlot::Legs => &self.legs,
            EquipmentSlot::Boots => &self.boots,
            EquipmentSlot::Cape => &self.cape,
            EquipmentSlot::MainHand => &self.main_hand,
            EquipmentSlot::OffHand => &self.off_hand,
            EquipmentSlot::Ring1 => &self.ring1,
            EquipmentSlot::Ring2 => &self.ring2,
            EquipmentSlot::Amulet => &self.amulet,
        }
    }

    /// Get a mutable reference to the item slot
    fn get_mut(&mut self, slot: EquipmentSlot) -> &mut Option<Item> {
        match slot {
            EquipmentSlot::Head => &mut self.head,
            EquipmentSlot::Shoulders => &mut self.shoulders,
            EquipmentSlot::Chest => &mut self.chest,
            EquipmentSlot::Bracers => &mut self.bracers,
            EquipmentSlot::Gloves => &mut self.gloves,
            EquipmentSlot::Belt => &mut self.belt,
            EquipmentSlot::Legs => &mut self.legs,
            EquipmentSlot::Boots => &mut self.boots,
            EquipmentSlot::Cape => &mut self.cape,
            EquipmentSlot::MainHand => &mut self.main_hand,
            EquipmentSlot::OffHand => &mut self.off_hand,
            EquipmentSlot::Ring1 => &mut self.ring1,
            EquipmentSlot::Ring2 => &mut self.ring2,
            EquipmentSlot::Amulet => &mut self.amulet,
        }
    }

    /// Equip an item in a slot. Returns the previously equipped item (if any).
    /// Returns error if the item is incompatible with the slot.
    pub fn equip(&mut self, slot: EquipmentSlot, item: Item) -> Result<Option<Item>, EquipError> {
        // Validate category
        let expected_cat = slot.valid_category();
        // Allow weapons in main hand / off hand, accessories in ring/amulet
        if item.category != expected_cat {
            return Err(EquipError::WrongCategory {
                expected: expected_cat,
                got: item.category,
            });
        }

        // Two-handed weapon validation
        if slot == EquipmentSlot::MainHand {
            if let Some(wd) = &item.weapon_data {
                if wd.weapon_type.grip() == WeaponGrip::TwoHanded && self.off_hand.is_some() {
                    return Err(EquipError::TwoHandedConflict);
                }
            }
        }

        if slot == EquipmentSlot::OffHand {
            if let Some(main) = &self.main_hand {
                if let Some(wd) = &main.weapon_data {
                    if wd.weapon_type.grip() == WeaponGrip::TwoHanded {
                        return Err(EquipError::MainHandIsTwoHanded);
                    }
                }
            }
        }

        let prev = self.get_mut(slot).take();
        *self.get_mut(slot) = Some(item);
        Ok(prev)
    }

    /// Unequip the item from a slot, returning it
    pub fn unequip(&mut self, slot: EquipmentSlot) -> Option<Item> {
        self.get_mut(slot).take()
    }

    /// Total stat modifiers from all equipped items
    pub fn total_modifiers(&self) -> StatModifiers {
        let mut total = StatModifiers::default();
        for &slot in EquipmentSlot::all() {
            if let Some(item) = self.get(slot) {
                total.add(&item.total_modifiers());
            }
        }
        total
    }

    /// Get the weapon type of the main hand weapon (if any)
    pub fn main_weapon_type(&self) -> Option<WeaponType> {
        self.main_hand
            .as_ref()
            .and_then(|item| item.weapon_data.as_ref())
            .map(|wd| wd.weapon_type)
    }

    /// Get the main hand weapon damage (if any)
    pub fn main_weapon_damage(&self) -> f32 {
        self.main_hand
            .as_ref()
            .and_then(|item| item.weapon_data.as_ref())
            .map(|wd| wd.base_damage)
            .unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::element::Element;
    use crate::combat::item::{ItemId, ItemRarity};
    use crate::combat::weapon::WeaponData;

    fn make_weapon(weapon_type: WeaponType) -> Item {
        Item {
            id: ItemId(1),
            name: format!("Test {}", weapon_type.name()),
            description: "A test weapon".to_string(),
            category: ItemCategory::Weapon,
            rarity: ItemRarity::Common,
            stat_modifiers: StatModifiers::default(),
            element: Element::Physical,
            weapon_data: Some(WeaponData::new(weapon_type, 10.0)),
            gem_sockets: vec![],
            required_level: 1,
            item_level: 1,
            stack_count: 1,
            max_stack: 1,
        }
    }

    fn make_armor() -> Item {
        Item {
            id: ItemId(2),
            name: "Test Helmet".to_string(),
            description: "A test helmet".to_string(),
            category: ItemCategory::Armor,
            rarity: ItemRarity::Rare,
            stat_modifiers: StatModifiers {
                defense: 5.0,
                max_hp: 20.0,
                ..Default::default()
            },
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
    fn test_equip_slot_count() {
        assert_eq!(EquipmentSlot::all().len(), 14);
    }

    #[test]
    fn test_equip_weapon_main_hand() {
        let mut set = EquipmentSet::new();
        let sword = make_weapon(WeaponType::Sword);
        let result = set.equip(EquipmentSlot::MainHand, sword);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none()); // no previous item
        assert_eq!(set.main_weapon_type(), Some(WeaponType::Sword));
    }

    #[test]
    fn test_equip_replaces_existing() {
        let mut set = EquipmentSet::new();
        let sword = make_weapon(WeaponType::Sword);
        let axe = make_weapon(WeaponType::Axe);
        set.equip(EquipmentSlot::MainHand, sword).unwrap();
        let prev = set.equip(EquipmentSlot::MainHand, axe).unwrap();
        assert!(prev.is_some());
        assert_eq!(set.main_weapon_type(), Some(WeaponType::Axe));
    }

    #[test]
    fn test_equip_wrong_category() {
        let mut set = EquipmentSet::new();
        let weapon = make_weapon(WeaponType::Sword);
        let result = set.equip(EquipmentSlot::Head, weapon); // weapon in armor slot
        assert!(matches!(result, Err(EquipError::WrongCategory { .. })));
    }

    #[test]
    fn test_two_handed_blocks_offhand() {
        let mut set = EquipmentSet::new();
        let greatsword = make_weapon(WeaponType::Greatsword);
        set.equip(EquipmentSlot::MainHand, greatsword).unwrap();
        let dagger = make_weapon(WeaponType::Dagger);
        let result = set.equip(EquipmentSlot::OffHand, dagger);
        assert!(matches!(result, Err(EquipError::MainHandIsTwoHanded)));
    }

    #[test]
    fn test_two_handed_conflicts_with_existing_offhand() {
        let mut set = EquipmentSet::new();
        let dagger = make_weapon(WeaponType::Dagger);
        set.equip(EquipmentSlot::OffHand, dagger).unwrap();
        let greatsword = make_weapon(WeaponType::Greatsword);
        let result = set.equip(EquipmentSlot::MainHand, greatsword);
        assert!(matches!(result, Err(EquipError::TwoHandedConflict)));
    }

    #[test]
    fn test_unequip_roundtrip() {
        let mut set = EquipmentSet::new();
        let armor = make_armor();
        set.equip(EquipmentSlot::Head, armor).unwrap();
        assert!(set.head.is_some());
        let removed = set.unequip(EquipmentSlot::Head);
        assert!(removed.is_some());
        assert!(set.head.is_none());
    }

    #[test]
    fn test_total_modifiers() {
        let mut set = EquipmentSet::new();
        let armor = make_armor(); // +5 defense, +20 max_hp
        set.equip(EquipmentSlot::Head, armor).unwrap();
        let mods = set.total_modifiers();
        assert_eq!(mods.defense, 5.0);
        assert_eq!(mods.max_hp, 20.0);
    }

    #[test]
    fn test_empty_equipment_zero_modifiers() {
        let set = EquipmentSet::new();
        let mods = set.total_modifiers();
        assert_eq!(mods.attack, 0.0);
        assert_eq!(mods.defense, 0.0);
        assert_eq!(mods.max_hp, 0.0);
    }
}
