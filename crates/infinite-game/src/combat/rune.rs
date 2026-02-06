//! Rune stacking magic composition system
//!
//! Players stack up to 4 runes to compose custom spells on the fly.
//! Runes have aspects: Element, Shape, Modifier, Amplifier.

use serde::{Deserialize, Serialize};

use super::element::Element;
use super::skill::{SkillShape, SkillTarget};
use super::status::StatusEffectType;

/// Maximum runes in a composition stack
pub const MAX_RUNE_STACK: usize = 4;

/// The aspect/role of a rune
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RuneAspect {
    Element,
    Shape,
    Modifier,
    Amplifier,
}

/// Modifier effects that change spell behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RuneModifier {
    Splitting,
    Piercing,
    Homing,
    Bouncing,
    Chaining,
    Lingering,
}

impl RuneModifier {
    pub fn name(self) -> &'static str {
        match self {
            Self::Splitting => "Splitting",
            Self::Piercing => "Piercing",
            Self::Homing => "Homing",
            Self::Bouncing => "Bouncing",
            Self::Chaining => "Chaining",
            Self::Lingering => "Lingering",
        }
    }
}

/// Amplifier effects that scale spell properties
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RuneAmplifier {
    Power,
    Range,
    Area,
    Speed,
    Duration,
}

impl RuneAmplifier {
    pub fn name(self) -> &'static str {
        match self {
            Self::Power => "Power",
            Self::Range => "Range",
            Self::Area => "Area",
            Self::Speed => "Speed",
            Self::Duration => "Duration",
        }
    }
}

/// A single rune that can be stacked
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rune {
    pub id: u64,
    pub name: String,
    pub aspect: RuneAspect,
    pub element: Option<Element>,
    pub shape: Option<SkillShape>,
    pub modifier: Option<RuneModifier>,
    pub amplifier: Option<RuneAmplifier>,
}

/// The result of composing runes into a spell
#[derive(Debug, Clone)]
pub struct ComposedSpell {
    pub name: String,
    pub element: Element,
    pub shape: SkillShape,
    pub modifiers: Vec<RuneModifier>,
    pub amplifiers: Vec<RuneAmplifier>,
    pub target: SkillTarget,
    pub damage_multiplier: f32,
    pub range_multiplier: f32,
    pub area_multiplier: f32,
    pub applies_status: Option<StatusEffectType>,
}

impl ComposedSpell {
    /// Compose a spell from a list of runes.
    /// Defaults: Physical element, Bolt shape.
    /// Last element rune wins. Last shape rune wins. Modifiers/amplifiers stack.
    pub fn from_runes(runes: &[Rune]) -> Option<ComposedSpell> {
        if runes.is_empty() {
            return None;
        }

        let mut element = Element::Physical;
        let mut shape = SkillShape::Bolt;
        let mut modifiers = Vec::new();
        let mut amplifiers = Vec::new();

        for rune in runes {
            if let Some(e) = rune.element {
                element = e;
            }
            if let Some(s) = rune.shape {
                shape = s;
            }
            if let Some(m) = rune.modifier {
                modifiers.push(m);
            }
            if let Some(a) = rune.amplifier {
                amplifiers.push(a);
            }
        }

        // Calculate multipliers from amplifiers
        let mut damage_multiplier = 1.0;
        let mut range_multiplier = 1.0;
        let mut area_multiplier = 1.0;

        for amp in &amplifiers {
            match amp {
                RuneAmplifier::Power => damage_multiplier += 0.3,
                RuneAmplifier::Range => range_multiplier += 0.5,
                RuneAmplifier::Area => area_multiplier += 0.4,
                RuneAmplifier::Speed => {} // affects projectile speed, handled elsewhere
                RuneAmplifier::Duration => {} // affects status duration, handled elsewhere
            }
        }

        // Determine target from shape
        let target = match shape {
            SkillShape::Bolt => SkillTarget::Projectile {
                speed: 20.0,
                range: 30.0 * range_multiplier,
            },
            SkillShape::Blast => SkillTarget::AreaAroundSelf {
                radius: 5.0 * area_multiplier,
            },
            SkillShape::Wave => SkillTarget::Cone {
                angle: 60.0,
                range: 8.0 * range_multiplier,
            },
            SkillShape::Shield => SkillTarget::SelfBuff,
            SkillShape::Aura => SkillTarget::AreaAroundSelf {
                radius: 8.0 * area_multiplier,
            },
            SkillShape::Nova => SkillTarget::AreaAroundSelf {
                radius: 10.0 * area_multiplier,
            },
        };

        // Elemental proc as status effect
        let applies_status = match element {
            Element::Fire => Some(StatusEffectType::Burning),
            Element::Water => Some(StatusEffectType::Frozen),
            Element::Air => Some(StatusEffectType::Shocked),
            Element::Earth => Some(StatusEffectType::Rooted),
            Element::Void => Some(StatusEffectType::Silenced),
            Element::Meta => Some(StatusEffectType::Blessed),
            Element::Physical => None,
        };

        // Auto-generate name
        let name = generate_spell_name(element, shape, &modifiers);

        Some(ComposedSpell {
            name,
            element,
            shape,
            modifiers,
            amplifiers,
            target,
            damage_multiplier,
            range_multiplier,
            area_multiplier,
            applies_status,
        })
    }
}

/// Generate a spell name from its components
fn generate_spell_name(
    element: Element,
    shape: SkillShape,
    modifiers: &[RuneModifier],
) -> String {
    let shape_name = match shape {
        SkillShape::Bolt => "Bolt",
        SkillShape::Blast => "Blast",
        SkillShape::Wave => "Wave",
        SkillShape::Shield => "Shield",
        SkillShape::Aura => "Aura",
        SkillShape::Nova => "Nova",
    };

    let mut parts = Vec::new();
    for m in modifiers {
        parts.push(m.name().to_string());
    }
    if element != Element::Physical {
        parts.push(element.name().to_string());
    }
    parts.push(shape_name.to_string());

    parts.join(" ")
}

/// Interactive rune composer — tracks the active composition stack
#[derive(Debug, Clone, Default)]
pub struct RuneComposer {
    /// Current stack of runes being composed
    pub stack: Vec<Rune>,
    /// Whether composition mode is active
    pub active: bool,
}

impl RuneComposer {
    /// Start composition mode
    pub fn begin(&mut self) {
        self.active = true;
        self.stack.clear();
    }

    /// Push a rune onto the stack (up to MAX_RUNE_STACK)
    pub fn push_rune(&mut self, rune: Rune) -> bool {
        if !self.active || self.stack.len() >= MAX_RUNE_STACK {
            return false;
        }
        self.stack.push(rune);
        true
    }

    /// Pop the last rune from the stack
    pub fn pop_rune(&mut self) -> Option<Rune> {
        self.stack.pop()
    }

    /// Cancel composition, clearing the stack
    pub fn cancel(&mut self) {
        self.active = false;
        self.stack.clear();
    }

    /// Compose the spell and end composition mode
    pub fn compose(&mut self) -> Option<ComposedSpell> {
        if !self.active || self.stack.is_empty() {
            self.cancel();
            return None;
        }
        let spell = ComposedSpell::from_runes(&self.stack);
        self.active = false;
        self.stack.clear();
        spell
    }

    /// Preview the spell that would be composed without consuming runes
    pub fn preview(&self) -> Option<ComposedSpell> {
        if !self.active || self.stack.is_empty() {
            return None;
        }
        ComposedSpell::from_runes(&self.stack)
    }

    /// Number of runes currently in the stack
    pub fn stack_size(&self) -> usize {
        self.stack.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn element_rune(element: Element) -> Rune {
        Rune {
            id: 1,
            name: format!("{} Rune", element.name()),
            aspect: RuneAspect::Element,
            element: Some(element),
            shape: None,
            modifier: None,
            amplifier: None,
        }
    }

    fn shape_rune(shape: SkillShape) -> Rune {
        Rune {
            id: 2,
            name: "Shape Rune".to_string(),
            aspect: RuneAspect::Shape,
            element: None,
            shape: Some(shape),
            modifier: None,
            amplifier: None,
        }
    }

    fn modifier_rune(modifier: RuneModifier) -> Rune {
        Rune {
            id: 3,
            name: format!("{} Rune", modifier.name()),
            aspect: RuneAspect::Modifier,
            element: None,
            shape: None,
            modifier: Some(modifier),
            amplifier: None,
        }
    }

    #[test]
    fn test_compose_default_spell() {
        // No element or shape runes → Physical Bolt
        let rune = modifier_rune(RuneModifier::Piercing);
        let spell = ComposedSpell::from_runes(&[rune]).unwrap();
        assert_eq!(spell.element, Element::Physical);
        assert_eq!(spell.shape, SkillShape::Bolt);
        assert_eq!(spell.name, "Piercing Bolt");
    }

    #[test]
    fn test_compose_fire_bolt() {
        let spell = ComposedSpell::from_runes(&[element_rune(Element::Fire)]).unwrap();
        assert_eq!(spell.element, Element::Fire);
        assert_eq!(spell.shape, SkillShape::Bolt);
        assert_eq!(spell.name, "Fire Bolt");
        assert_eq!(spell.applies_status, Some(StatusEffectType::Burning));
    }

    #[test]
    fn test_compose_splitting_fire_bolt() {
        let runes = vec![
            modifier_rune(RuneModifier::Splitting),
            element_rune(Element::Fire),
        ];
        let spell = ComposedSpell::from_runes(&runes).unwrap();
        assert_eq!(spell.name, "Splitting Fire Bolt");
    }

    #[test]
    fn test_last_element_wins() {
        let runes = vec![
            element_rune(Element::Fire),
            element_rune(Element::Water),
        ];
        let spell = ComposedSpell::from_runes(&runes).unwrap();
        assert_eq!(spell.element, Element::Water);
    }

    #[test]
    fn test_last_shape_wins() {
        let runes = vec![
            shape_rune(SkillShape::Bolt),
            shape_rune(SkillShape::Nova),
        ];
        let spell = ComposedSpell::from_runes(&runes).unwrap();
        assert_eq!(spell.shape, SkillShape::Nova);
    }

    #[test]
    fn test_empty_runes_returns_none() {
        assert!(ComposedSpell::from_runes(&[]).is_none());
    }

    #[test]
    fn test_composer_workflow() {
        let mut composer = RuneComposer::default();
        composer.begin();
        assert!(composer.active);

        composer.push_rune(element_rune(Element::Fire));
        composer.push_rune(modifier_rune(RuneModifier::Splitting));
        assert_eq!(composer.stack_size(), 2);

        let preview = composer.preview().unwrap();
        assert_eq!(preview.element, Element::Fire);

        let spell = composer.compose().unwrap();
        assert_eq!(spell.name, "Splitting Fire Bolt");
        assert!(!composer.active);
        assert_eq!(composer.stack_size(), 0);
    }

    #[test]
    fn test_composer_max_stack() {
        let mut composer = RuneComposer::default();
        composer.begin();
        for _ in 0..MAX_RUNE_STACK {
            assert!(composer.push_rune(element_rune(Element::Fire)));
        }
        assert!(!composer.push_rune(element_rune(Element::Water))); // over limit
        assert_eq!(composer.stack_size(), MAX_RUNE_STACK);
    }

    #[test]
    fn test_composer_cancel() {
        let mut composer = RuneComposer::default();
        composer.begin();
        composer.push_rune(element_rune(Element::Fire));
        composer.cancel();
        assert!(!composer.active);
        assert_eq!(composer.stack_size(), 0);
    }

    #[test]
    fn test_composer_pop() {
        let mut composer = RuneComposer::default();
        composer.begin();
        composer.push_rune(element_rune(Element::Fire));
        composer.push_rune(element_rune(Element::Water));
        let popped = composer.pop_rune();
        assert!(popped.is_some());
        assert_eq!(composer.stack_size(), 1);
    }

    #[test]
    fn test_amplifier_stacking() {
        let runes = vec![
            Rune {
                id: 10,
                name: "Power Rune".to_string(),
                aspect: RuneAspect::Amplifier,
                element: None,
                shape: None,
                modifier: None,
                amplifier: Some(RuneAmplifier::Power),
            },
            Rune {
                id: 11,
                name: "Power Rune 2".to_string(),
                aspect: RuneAspect::Amplifier,
                element: None,
                shape: None,
                modifier: None,
                amplifier: Some(RuneAmplifier::Power),
            },
        ];
        let spell = ComposedSpell::from_runes(&runes).unwrap();
        assert!((spell.damage_multiplier - 1.6).abs() < 0.01); // 1.0 + 0.3 + 0.3
    }
}
