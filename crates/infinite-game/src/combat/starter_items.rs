//! Per-archetype starting gear
//!
//! Creates starter weapons, armor, consumables, and skills for each character archetype.

use super::damage::StatModifiers;
use super::element::Element;
use super::item::{Item, ItemCategory, ItemId, ItemRarity};
use super::skill::{ActiveSkill, Skill, SkillId, SkillShape, SkillSlot, SkillTarget};
use super::status::StatusEffectType;
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

/// Create starter skill slots for a given archetype.
/// Returns 4 skill slots with the first slot populated with a starter skill.
pub fn create_starter_skills(archetype_name: &str) -> Vec<SkillSlot> {
    let skill = match archetype_name {
        "Chronomancer" => ActiveSkill {
            id: SkillId(4001),
            name: "Time Bolt".to_string(),
            description: "Hurls a bolt of temporal energy.".to_string(),
            element: Element::Void,
            shape: SkillShape::Bolt,
            target: SkillTarget::Projectile { speed: 25.0, range: 15.0 },
            base_damage: 15.0,
            damage_multiplier: 1.5,
            cooldown: 3.0,
            cost: 20.0,
            applies_status: None,
            status_duration: 0.0,
        },
        "TemporalHunter" => ActiveSkill {
            id: SkillId(4002),
            name: "Shadow Strike".to_string(),
            description: "A swift strike from the shadows of time.".to_string(),
            element: Element::Air,
            shape: SkillShape::Wave,
            target: SkillTarget::SingleTarget,
            base_damage: 20.0,
            damage_multiplier: 1.8,
            cooldown: 4.0,
            cost: 15.0,
            applies_status: None,
            status_duration: 0.0,
        },
        "Vanguard" => ActiveSkill {
            id: SkillId(4003),
            name: "Shield Bash".to_string(),
            description: "Bash enemies with your shield, stunning them.".to_string(),
            element: Element::Earth,
            shape: SkillShape::Blast,
            target: SkillTarget::Cone { angle: 90.0, range: 3.0 },
            base_damage: 10.0,
            damage_multiplier: 1.2,
            cooldown: 5.0,
            cost: 25.0,
            applies_status: Some(StatusEffectType::Stunned),
            status_duration: 2.0,
        },
        "Technomage" => ActiveSkill {
            id: SkillId(4004),
            name: "Fire Blast".to_string(),
            description: "Unleashes a blast of techno-magical fire.".to_string(),
            element: Element::Fire,
            shape: SkillShape::Blast,
            target: SkillTarget::AreaAroundSelf { radius: 5.0 },
            base_damage: 18.0,
            damage_multiplier: 1.6,
            cooldown: 3.5,
            cost: 22.0,
            applies_status: Some(StatusEffectType::Burning),
            status_duration: 3.0,
        },
        "ParadoxWeaver" => ActiveSkill {
            id: SkillId(4005),
            name: "Paradox Bolt".to_string(),
            description: "A bolt of contradictory energy that tears at reality.".to_string(),
            element: Element::Meta,
            shape: SkillShape::Bolt,
            target: SkillTarget::Projectile { speed: 20.0, range: 20.0 },
            base_damage: 25.0,
            damage_multiplier: 2.0,
            cooldown: 6.0,
            cost: 30.0,
            applies_status: None,
            status_duration: 0.0,
        },
        _ => return vec![SkillSlot::empty(); 4],
    };

    let mut slots = vec![SkillSlot::with_skill(Skill::Active(skill))];
    // Fill remaining 3 slots as empty
    for _ in 0..3 {
        slots.push(SkillSlot::empty());
    }
    slots
}
