//! Element system with advantage wheel
//!
//! 7 elements: Physical, Fire, Earth, Water, Air, Void, Meta
//! Advantage cycle: Fire > Earth > Air > Water > Fire (1.3x)
//! Void is strong vs all except Meta (1.3x), Meta is strong vs Void (1.5x)
//! Physical is always neutral (1.0x)

use serde::{Deserialize, Serialize};

/// The 7 combat elements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Element {
    #[default]
    Physical,
    Fire,
    Earth,
    Water,
    Air,
    Void,
    Meta,
}

/// Total number of elements (for array indexing)
pub const ELEMENT_COUNT: usize = 7;

impl Element {
    /// Array index for this element (for elemental_damage_bonus / elemental_resistance arrays)
    pub fn index(self) -> usize {
        match self {
            Self::Physical => 0,
            Self::Fire => 1,
            Self::Earth => 2,
            Self::Water => 3,
            Self::Air => 4,
            Self::Void => 5,
            Self::Meta => 6,
        }
    }

    /// Display name
    pub fn name(self) -> &'static str {
        match self {
            Self::Physical => "Physical",
            Self::Fire => "Fire",
            Self::Earth => "Earth",
            Self::Water => "Water",
            Self::Air => "Air",
            Self::Void => "Void",
            Self::Meta => "Meta",
        }
    }

    /// Color as [r, g, b] floats (0.0-1.0)
    pub fn color(self) -> [f32; 3] {
        match self {
            Self::Physical => [0.7, 0.7, 0.7],
            Self::Fire => [1.0, 0.3, 0.1],
            Self::Earth => [0.6, 0.4, 0.2],
            Self::Water => [0.2, 0.5, 1.0],
            Self::Air => [0.8, 0.9, 1.0],
            Self::Void => [0.4, 0.1, 0.6],
            Self::Meta => [1.0, 0.85, 0.0],
        }
    }

    /// Whether this element is strong against the target
    pub fn is_strong_against(self, target: Self) -> bool {
        matches!(
            (self, target),
            (Self::Fire, Self::Earth)
                | (Self::Earth, Self::Air)
                | (Self::Air, Self::Water)
                | (Self::Water, Self::Fire)
                | (Self::Void, Self::Physical)
                | (Self::Void, Self::Fire)
                | (Self::Void, Self::Earth)
                | (Self::Void, Self::Water)
                | (Self::Void, Self::Air)
                | (Self::Meta, Self::Void)
        )
    }

    /// Whether this element is weak against the target
    pub fn is_weak_against(self, target: Self) -> bool {
        target.is_strong_against(self)
    }

    /// Damage multiplier when attacking a target of the given element
    pub fn multiplier_against(self, target: Self) -> f32 {
        if self == Self::Physical || target == Self::Physical && self != Self::Void {
            // Physical always neutral, and attacks against Physical are neutral
            // (except Void which is strong vs Physical)
            if self == Self::Void && target == Self::Physical {
                return 1.3;
            }
            return 1.0;
        }

        if self == Self::Meta && target == Self::Void {
            return 1.5;
        }

        if self.is_strong_against(target) {
            1.3
        } else if self.is_weak_against(target) {
            0.7
        } else {
            1.0
        }
    }

    /// All element variants
    pub fn all() -> &'static [Element] {
        &[
            Self::Physical,
            Self::Fire,
            Self::Earth,
            Self::Water,
            Self::Air,
            Self::Void,
            Self::Meta,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_count() {
        assert_eq!(Element::all().len(), ELEMENT_COUNT);
    }

    #[test]
    fn test_advantage_cycle() {
        // Fire > Earth > Air > Water > Fire
        assert!(Element::Fire.is_strong_against(Element::Earth));
        assert!(Element::Earth.is_strong_against(Element::Air));
        assert!(Element::Air.is_strong_against(Element::Water));
        assert!(Element::Water.is_strong_against(Element::Fire));
    }

    #[test]
    fn test_advantage_multipliers() {
        assert_eq!(Element::Fire.multiplier_against(Element::Earth), 1.3);
        assert_eq!(Element::Earth.multiplier_against(Element::Fire), 0.7);
        assert_eq!(Element::Fire.multiplier_against(Element::Fire), 1.0);
    }

    #[test]
    fn test_physical_always_neutral() {
        for &elem in Element::all() {
            if elem != Element::Void {
                assert_eq!(
                    Element::Physical.multiplier_against(elem),
                    1.0,
                    "Physical should be 1.0x vs {:?}",
                    elem
                );
            }
        }
    }

    #[test]
    fn test_void_strong_vs_core_elements() {
        assert_eq!(Element::Void.multiplier_against(Element::Physical), 1.3);
        assert_eq!(Element::Void.multiplier_against(Element::Fire), 1.3);
        assert_eq!(Element::Void.multiplier_against(Element::Earth), 1.3);
        assert_eq!(Element::Void.multiplier_against(Element::Water), 1.3);
        assert_eq!(Element::Void.multiplier_against(Element::Air), 1.3);
    }

    #[test]
    fn test_void_weak_vs_meta() {
        // Void is weak against Meta because Meta is strong against Void
        assert_eq!(Element::Void.multiplier_against(Element::Meta), 0.7);
    }

    #[test]
    fn test_meta_strong_vs_void() {
        assert_eq!(Element::Meta.multiplier_against(Element::Void), 1.5);
    }

    #[test]
    fn test_weak_against_inverse() {
        assert!(Element::Earth.is_weak_against(Element::Fire));
        assert!(Element::Air.is_weak_against(Element::Earth));
        assert!(Element::Water.is_weak_against(Element::Air));
        assert!(Element::Fire.is_weak_against(Element::Water));
    }

    #[test]
    fn test_element_indices_unique() {
        let mut indices: Vec<usize> = Element::all().iter().map(|e| e.index()).collect();
        indices.sort();
        indices.dedup();
        assert_eq!(indices.len(), ELEMENT_COUNT);
    }

    #[test]
    fn test_element_names_nonempty() {
        for &elem in Element::all() {
            assert!(!elem.name().is_empty());
        }
    }

    #[test]
    fn test_default_is_physical() {
        assert_eq!(Element::default(), Element::Physical);
    }
}
