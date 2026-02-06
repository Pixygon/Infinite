//! Damage calculation pipeline
//!
//! Attack types (Light/Heavy), stat modifiers, and the full damage pipeline.

use serde::{Deserialize, Serialize};

use super::element::{Element, ELEMENT_COUNT};
use super::weapon::WeaponType;

/// Attack type: light or heavy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AttackType {
    /// Fast attack: 1.0x damage, 0.4s cooldown, 0.1s windup
    Light,
    /// Slow attack: 1.8x damage, 1.2s cooldown, 0.5s windup
    Heavy,
}

impl AttackType {
    /// Damage multiplier for this attack type
    pub fn damage_multiplier(self) -> f32 {
        match self {
            Self::Light => 1.0,
            Self::Heavy => 1.8,
        }
    }

    /// Base cooldown in seconds
    pub fn base_cooldown(self) -> f32 {
        match self {
            Self::Light => 0.4,
            Self::Heavy => 1.2,
        }
    }

    /// Windup time in seconds before damage is dealt
    pub fn windup(self) -> f32 {
        match self {
            Self::Light => 0.1,
            Self::Heavy => 0.5,
        }
    }
}

/// Flat stat bonuses from equipment, gems, buffs, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatModifiers {
    pub max_hp: f32,
    pub attack: f32,
    pub defense: f32,
    pub speed: f32,
    pub crit_chance: f32,
    pub crit_multiplier: f32,
    /// Per-element damage bonus (indexed by Element::index())
    pub elemental_damage_bonus: [f32; ELEMENT_COUNT],
    /// Per-element resistance (indexed by Element::index())
    pub elemental_resistance: [f32; ELEMENT_COUNT],
}

impl Default for StatModifiers {
    fn default() -> Self {
        Self {
            max_hp: 0.0,
            attack: 0.0,
            defense: 0.0,
            speed: 0.0,
            crit_chance: 0.0,
            crit_multiplier: 0.0,
            elemental_damage_bonus: [0.0; ELEMENT_COUNT],
            elemental_resistance: [0.0; ELEMENT_COUNT],
        }
    }
}

impl StatModifiers {
    /// Add another set of modifiers onto this one
    pub fn add(&mut self, other: &StatModifiers) {
        self.max_hp += other.max_hp;
        self.attack += other.attack;
        self.defense += other.defense;
        self.speed += other.speed;
        self.crit_chance += other.crit_chance;
        self.crit_multiplier += other.crit_multiplier;
        for i in 0..ELEMENT_COUNT {
            self.elemental_damage_bonus[i] += other.elemental_damage_bonus[i];
            self.elemental_resistance[i] += other.elemental_resistance[i];
        }
    }

    /// Combined total of two modifier sets (non-mutating)
    pub fn combined(&self, other: &StatModifiers) -> StatModifiers {
        let mut result = self.clone();
        result.add(other);
        result
    }
}

/// Result of a damage calculation
#[derive(Debug, Clone)]
pub struct DamageEvent {
    /// Damage before defense
    pub base_amount: f32,
    /// Damage after all multipliers and defense
    pub final_amount: f32,
    /// Element of the attack
    pub element: Element,
    /// Attack type used
    pub attack_type: AttackType,
    /// Whether this was a critical hit
    pub is_crit: bool,
    /// Element multiplier applied
    pub element_multiplier: f32,
    /// Weapon type multiplier applied
    pub weapon_multiplier: f32,
}

/// Full damage calculation pipeline
///
/// Pipeline: base_attack + weapon_damage -> attack_type_mult -> weapon_type_mult
///           -> element_mult -> elemental_bonus -> crit -> minus defense -> floor at 1.0
#[allow(clippy::too_many_arguments)]
pub fn calculate_combat_damage(
    base_attack: f32,
    weapon_damage: f32,
    weapon_type: Option<WeaponType>,
    attack_type: AttackType,
    element: Element,
    crit_chance: f32,
    crit_multiplier: f32,
    elemental_damage_bonus: f32,
    target_defense: f32,
    target_element: Element,
    target_elemental_resistance: f32,
    weapon_weakness: Option<WeaponType>,
) -> DamageEvent {
    // Base damage
    let mut damage = base_attack + weapon_damage;

    // Attack type multiplier
    damage *= attack_type.damage_multiplier();

    // Weapon type multiplier
    let weapon_mult = weapon_type.map(|w| w.damage_multiplier()).unwrap_or(1.0);
    damage *= weapon_mult;

    // Weapon weakness bonus (1.5x if enemy is weak to this weapon type)
    if let (Some(wt), Some(weakness)) = (weapon_type, weapon_weakness) {
        if wt == weakness {
            damage *= 1.5;
        }
    }

    // Element multiplier
    let element_mult = element.multiplier_against(target_element);
    damage *= element_mult;

    // Elemental damage bonus
    damage += elemental_damage_bonus;

    let base_amount = damage;

    // Critical hit
    let is_crit = rand::random::<f32>() < crit_chance;
    if is_crit {
        damage *= crit_multiplier;
    }

    // Defense reduction
    damage -= target_defense * 0.5;

    // Elemental resistance reduction
    damage -= target_elemental_resistance;

    // Floor at 1.0
    damage = damage.max(1.0);

    DamageEvent {
        base_amount,
        final_amount: damage,
        element,
        attack_type,
        is_crit,
        element_multiplier: element_mult,
        weapon_multiplier: weapon_mult,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attack_type_properties() {
        assert_eq!(AttackType::Light.damage_multiplier(), 1.0);
        assert_eq!(AttackType::Heavy.damage_multiplier(), 1.8);
        assert!(AttackType::Heavy.base_cooldown() > AttackType::Light.base_cooldown());
        assert!(AttackType::Heavy.windup() > AttackType::Light.windup());
    }

    #[test]
    fn test_stat_modifiers_add() {
        let mut a = StatModifiers {
            attack: 5.0,
            defense: 3.0,
            ..Default::default()
        };
        let b = StatModifiers {
            attack: 2.0,
            max_hp: 10.0,
            ..Default::default()
        };
        a.add(&b);
        assert_eq!(a.attack, 7.0);
        assert_eq!(a.defense, 3.0);
        assert_eq!(a.max_hp, 10.0);
    }

    #[test]
    fn test_damage_pipeline_basic() {
        // Simple case: 10 base + 5 weapon, light attack, physical, no crit, 0 defense
        let event = calculate_combat_damage(
            10.0,               // base_attack
            5.0,                // weapon_damage
            None,               // weapon_type
            AttackType::Light,  // attack_type
            Element::Physical,  // element
            0.0,                // crit_chance (no crit)
            1.5,                // crit_multiplier
            0.0,                // elemental_damage_bonus
            0.0,                // target_defense
            Element::Physical,  // target_element
            0.0,                // target_elemental_resistance
            None,               // weapon_weakness
        );
        assert_eq!(event.final_amount, 15.0);
        assert!(!event.is_crit);
    }

    #[test]
    fn test_damage_pipeline_heavy_attack() {
        let event = calculate_combat_damage(
            10.0, 0.0, None, AttackType::Heavy, Element::Physical,
            0.0, 1.5, 0.0, 0.0, Element::Physical, 0.0, None,
        );
        assert_eq!(event.final_amount, 18.0); // 10 * 1.8
    }

    #[test]
    fn test_damage_pipeline_element_advantage() {
        let event = calculate_combat_damage(
            10.0, 0.0, None, AttackType::Light, Element::Fire,
            0.0, 1.5, 0.0, 0.0, Element::Earth, 0.0, None,
        );
        assert_eq!(event.final_amount, 13.0); // 10 * 1.3
        assert_eq!(event.element_multiplier, 1.3);
    }

    #[test]
    fn test_damage_pipeline_defense_reduction() {
        let event = calculate_combat_damage(
            10.0, 0.0, None, AttackType::Light, Element::Physical,
            0.0, 1.5, 0.0, 10.0, Element::Physical, 0.0, None,
        );
        assert_eq!(event.final_amount, 5.0); // 10 - 10*0.5
    }

    #[test]
    fn test_damage_minimum_floor() {
        let event = calculate_combat_damage(
            1.0, 0.0, None, AttackType::Light, Element::Physical,
            0.0, 1.5, 0.0, 100.0, Element::Physical, 0.0, None,
        );
        assert_eq!(event.final_amount, 1.0); // floor at 1
    }

    #[test]
    fn test_damage_weapon_weakness() {
        let event = calculate_combat_damage(
            10.0, 0.0, Some(WeaponType::Sword), AttackType::Light, Element::Physical,
            0.0, 1.5, 0.0, 0.0, Element::Physical, 0.0, Some(WeaponType::Sword),
        );
        assert_eq!(event.final_amount, 10.0 * 1.1 * 1.5); // weapon_mult * weakness
    }
}
