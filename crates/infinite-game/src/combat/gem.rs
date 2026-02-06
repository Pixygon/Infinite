//! Gem system — shapes, qualities, and socket mechanics
//!
//! Gems can be socketed into equipment to grant stat bonuses and skills.

use serde::{Deserialize, Serialize};

use super::damage::StatModifiers;
use super::element::Element;
use super::skill::SkillId;

/// Gem shapes — must match socket shape to fit (Star fits any socket)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GemShape {
    Circle,
    Triangle,
    Square,
    Star,
}

impl GemShape {
    /// Display name
    pub fn name(self) -> &'static str {
        match self {
            Self::Circle => "Circle",
            Self::Triangle => "Triangle",
            Self::Square => "Square",
            Self::Star => "Star",
        }
    }
}

/// Gem quality — multiplier on base stats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GemQuality {
    Rough,
    Cut,
    Polished,
    Perfect,
    Prismatic,
}

impl GemQuality {
    /// Stat multiplier for this quality tier
    pub fn multiplier(self) -> f32 {
        match self {
            Self::Rough => 1.0,
            Self::Cut => 1.5,
            Self::Polished => 2.0,
            Self::Perfect => 2.5,
            Self::Prismatic => 3.0,
        }
    }

    /// Display name
    pub fn name(self) -> &'static str {
        match self {
            Self::Rough => "Rough",
            Self::Cut => "Cut",
            Self::Polished => "Polished",
            Self::Perfect => "Perfect",
            Self::Prismatic => "Prismatic",
        }
    }
}

/// A gem that can be socketed into equipment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gem {
    pub name: String,
    pub shape: GemShape,
    pub quality: GemQuality,
    pub element: Element,
    /// Base stat bonuses (before quality multiplier)
    pub base_modifiers: StatModifiers,
    /// Optional skill granted by this gem
    pub granted_skill: Option<SkillId>,
}

impl Gem {
    /// Effective modifiers after applying quality multiplier
    pub fn effective_modifiers(&self) -> StatModifiers {
        let mult = self.quality.multiplier();
        let base = &self.base_modifiers;
        StatModifiers {
            max_hp: base.max_hp * mult,
            attack: base.attack * mult,
            defense: base.defense * mult,
            speed: base.speed * mult,
            crit_chance: base.crit_chance * mult,
            crit_multiplier: base.crit_multiplier * mult,
            elemental_damage_bonus: {
                let mut arr = base.elemental_damage_bonus;
                for v in &mut arr {
                    *v *= mult;
                }
                arr
            },
            elemental_resistance: {
                let mut arr = base.elemental_resistance;
                for v in &mut arr {
                    *v *= mult;
                }
                arr
            },
        }
    }

    /// Whether this gem fits in a socket of the given shape
    pub fn fits_socket(&self, socket_shape: GemShape) -> bool {
        self.shape == GemShape::Star || self.shape == socket_shape
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_gem(shape: GemShape, quality: GemQuality) -> Gem {
        Gem {
            name: "Test Gem".to_string(),
            shape,
            quality,
            element: Element::Fire,
            base_modifiers: StatModifiers {
                attack: 10.0,
                ..Default::default()
            },
            granted_skill: None,
        }
    }

    #[test]
    fn test_gem_quality_multiplier() {
        assert_eq!(GemQuality::Rough.multiplier(), 1.0);
        assert_eq!(GemQuality::Prismatic.multiplier(), 3.0);
    }

    #[test]
    fn test_effective_modifiers_scaling() {
        let gem = test_gem(GemShape::Circle, GemQuality::Polished);
        let mods = gem.effective_modifiers();
        assert_eq!(mods.attack, 20.0); // 10 * 2.0
    }

    #[test]
    fn test_gem_fits_matching_socket() {
        let gem = test_gem(GemShape::Circle, GemQuality::Rough);
        assert!(gem.fits_socket(GemShape::Circle));
        assert!(!gem.fits_socket(GemShape::Triangle));
        assert!(!gem.fits_socket(GemShape::Square));
    }

    #[test]
    fn test_star_fits_any_socket() {
        let gem = test_gem(GemShape::Star, GemQuality::Rough);
        assert!(gem.fits_socket(GemShape::Circle));
        assert!(gem.fits_socket(GemShape::Triangle));
        assert!(gem.fits_socket(GemShape::Square));
        assert!(gem.fits_socket(GemShape::Star));
    }

    #[test]
    fn test_quality_progression() {
        let qualities = [
            GemQuality::Rough,
            GemQuality::Cut,
            GemQuality::Polished,
            GemQuality::Perfect,
            GemQuality::Prismatic,
        ];
        for i in 1..qualities.len() {
            assert!(
                qualities[i].multiplier() > qualities[i - 1].multiplier(),
                "{:?} should have higher multiplier than {:?}",
                qualities[i],
                qualities[i - 1],
            );
        }
    }
}
