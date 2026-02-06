//! Weapon types and properties
//!
//! 15 weapon types with per-type speed, damage, and range multipliers.

use serde::{Deserialize, Serialize};

/// The 15 weapon types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WeaponType {
    Sword,
    Axe,
    Mace,
    Dagger,
    Spear,
    Bow,
    Staff,
    Wand,
    Halberd,
    Crossbow,
    Greatsword,
    DualBlades,
    Scythe,
    Hammer,
    Whip,
}

/// Whether the weapon is melee or ranged
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WeaponRange {
    Melee,
    Ranged,
}

/// Whether the weapon is one-handed or two-handed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WeaponGrip {
    OneHanded,
    TwoHanded,
}

impl WeaponType {
    /// Speed multiplier (applied to attack cooldown — higher = faster)
    pub fn speed_multiplier(self) -> f32 {
        match self {
            Self::Sword => 1.0,
            Self::Axe => 0.85,
            Self::Mace => 0.8,
            Self::Dagger => 1.4,
            Self::Spear => 0.9,
            Self::Bow => 0.95,
            Self::Staff => 0.75,
            Self::Wand => 1.2,
            Self::Halberd => 0.7,
            Self::Crossbow => 0.6,
            Self::Greatsword => 0.65,
            Self::DualBlades => 1.3,
            Self::Scythe => 0.8,
            Self::Hammer => 0.55,
            Self::Whip => 1.1,
        }
    }

    /// Damage multiplier (base weapon damage is multiplied by this)
    pub fn damage_multiplier(self) -> f32 {
        match self {
            Self::Sword => 1.1,
            Self::Axe => 1.2,
            Self::Mace => 1.15,
            Self::Dagger => 0.8,
            Self::Spear => 1.05,
            Self::Bow => 1.0,
            Self::Staff => 1.3,
            Self::Wand => 0.9,
            Self::Halberd => 1.35,
            Self::Crossbow => 1.4,
            Self::Greatsword => 1.5,
            Self::DualBlades => 0.85,
            Self::Scythe => 1.25,
            Self::Hammer => 1.6,
            Self::Whip => 0.95,
        }
    }

    /// Attack range in world units
    pub fn attack_range(self) -> f32 {
        match self {
            Self::Sword => 2.5,
            Self::Axe => 2.5,
            Self::Mace => 2.0,
            Self::Dagger => 1.5,
            Self::Spear => 3.5,
            Self::Bow => 25.0,
            Self::Staff => 3.0,
            Self::Wand => 15.0,
            Self::Halberd => 4.0,
            Self::Crossbow => 30.0,
            Self::Greatsword => 3.0,
            Self::DualBlades => 2.0,
            Self::Scythe => 3.0,
            Self::Hammer => 2.5,
            Self::Whip => 5.0,
        }
    }

    /// Light attack cooldown in seconds (base * 1/speed_multiplier)
    pub fn light_attack_cooldown(self) -> f32 {
        0.4 / self.speed_multiplier()
    }

    /// Heavy attack cooldown in seconds
    pub fn heavy_attack_cooldown(self) -> f32 {
        1.2 / self.speed_multiplier()
    }

    /// Whether this weapon is melee or ranged
    pub fn range_type(self) -> WeaponRange {
        match self {
            Self::Bow | Self::Crossbow | Self::Wand => WeaponRange::Ranged,
            _ => WeaponRange::Melee,
        }
    }

    /// Whether this weapon is one-handed or two-handed
    pub fn grip(self) -> WeaponGrip {
        match self {
            Self::Greatsword | Self::Halberd | Self::Hammer | Self::Scythe | Self::Staff
            | Self::Bow | Self::Crossbow => WeaponGrip::TwoHanded,
            _ => WeaponGrip::OneHanded,
        }
    }

    /// Display name
    pub fn name(self) -> &'static str {
        match self {
            Self::Sword => "Sword",
            Self::Axe => "Axe",
            Self::Mace => "Mace",
            Self::Dagger => "Dagger",
            Self::Spear => "Spear",
            Self::Bow => "Bow",
            Self::Staff => "Staff",
            Self::Wand => "Wand",
            Self::Halberd => "Halberd",
            Self::Crossbow => "Crossbow",
            Self::Greatsword => "Greatsword",
            Self::DualBlades => "Dual Blades",
            Self::Scythe => "Scythe",
            Self::Hammer => "Hammer",
            Self::Whip => "Whip",
        }
    }

    /// All weapon type variants
    pub fn all() -> &'static [WeaponType] {
        &[
            Self::Sword,
            Self::Axe,
            Self::Mace,
            Self::Dagger,
            Self::Spear,
            Self::Bow,
            Self::Staff,
            Self::Wand,
            Self::Halberd,
            Self::Crossbow,
            Self::Greatsword,
            Self::DualBlades,
            Self::Scythe,
            Self::Hammer,
            Self::Whip,
        ]
    }
}

/// Weapon-specific data stored on an item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponData {
    /// Type of weapon
    pub weapon_type: WeaponType,
    /// Base damage of this specific weapon instance
    pub base_damage: f32,
    /// Attack speed of this specific weapon instance
    pub attack_speed: f32,
}

impl WeaponData {
    /// Create new weapon data
    pub fn new(weapon_type: WeaponType, base_damage: f32) -> Self {
        Self {
            weapon_type,
            base_damage,
            attack_speed: weapon_type.speed_multiplier(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weapon_count() {
        assert_eq!(WeaponType::all().len(), 15);
    }

    #[test]
    fn test_dagger_fastest() {
        for &wt in WeaponType::all() {
            assert!(
                WeaponType::Dagger.speed_multiplier() >= wt.speed_multiplier(),
                "Dagger should be fastest or tied, but {:?} is faster",
                wt
            );
        }
    }

    #[test]
    fn test_hammer_highest_damage() {
        for &wt in WeaponType::all() {
            assert!(
                WeaponType::Hammer.damage_multiplier() >= wt.damage_multiplier(),
                "Hammer should have highest damage mult, but {:?} is higher",
                wt
            );
        }
    }

    #[test]
    fn test_ranged_types() {
        assert_eq!(WeaponType::Bow.range_type(), WeaponRange::Ranged);
        assert_eq!(WeaponType::Crossbow.range_type(), WeaponRange::Ranged);
        assert_eq!(WeaponType::Wand.range_type(), WeaponRange::Ranged);
        assert_eq!(WeaponType::Sword.range_type(), WeaponRange::Melee);
    }

    #[test]
    fn test_two_handed_types() {
        assert_eq!(WeaponType::Greatsword.grip(), WeaponGrip::TwoHanded);
        assert_eq!(WeaponType::Halberd.grip(), WeaponGrip::TwoHanded);
        assert_eq!(WeaponType::Hammer.grip(), WeaponGrip::TwoHanded);
        assert_eq!(WeaponType::Sword.grip(), WeaponGrip::OneHanded);
        assert_eq!(WeaponType::Dagger.grip(), WeaponGrip::OneHanded);
    }

    #[test]
    fn test_cooldown_scales_with_speed() {
        let fast = WeaponType::Dagger.light_attack_cooldown();
        let slow = WeaponType::Hammer.light_attack_cooldown();
        assert!(fast < slow, "Dagger should attack faster than Hammer");
    }

    #[test]
    fn test_weapon_data_new() {
        let data = WeaponData::new(WeaponType::Sword, 15.0);
        assert_eq!(data.weapon_type, WeaponType::Sword);
        assert_eq!(data.base_damage, 15.0);
        assert_eq!(data.attack_speed, WeaponType::Sword.speed_multiplier());
    }

    #[test]
    fn test_weapon_names_nonempty() {
        for &wt in WeaponType::all() {
            assert!(!wt.name().is_empty());
        }
    }
}
