//! Skill system — active and passive skills with cooldown management
//!
//! Skills can be granted by gems, equipment, or learned directly.

use serde::{Deserialize, Serialize};

use super::damage::StatModifiers;
use super::element::Element;
use super::status::StatusEffectType;

/// Maximum number of skill slots a player can have
pub const MAX_SKILL_SLOTS: usize = 4;

/// Unique skill identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SkillId(pub u64);

/// What the skill targets
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SkillTarget {
    SingleTarget,
    AreaAroundSelf { radius: f32 },
    Cone { angle: f32, range: f32 },
    SelfBuff,
    Projectile { speed: f32, range: f32 },
}

/// Visual shape of the skill effect
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillShape {
    Bolt,
    Blast,
    Wave,
    Shield,
    Aura,
    Nova,
}

/// An active skill that can be used in combat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveSkill {
    pub id: SkillId,
    pub name: String,
    pub description: String,
    pub element: Element,
    pub shape: SkillShape,
    pub target: SkillTarget,
    pub base_damage: f32,
    pub damage_multiplier: f32,
    pub cooldown: f32,
    pub cost: f32,
    pub applies_status: Option<StatusEffectType>,
    pub status_duration: f32,
}

/// A passive skill that provides ongoing benefits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassiveSkill {
    pub id: SkillId,
    pub name: String,
    pub description: String,
    pub stat_modifiers: StatModifiers,
    pub proc_chance: f32,
    pub proc_status: Option<StatusEffectType>,
    pub proc_duration: f32,
}

/// A skill — either active or passive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Skill {
    Active(ActiveSkill),
    Passive(PassiveSkill),
}

impl Skill {
    /// Get the skill's name
    pub fn name(&self) -> &str {
        match self {
            Self::Active(s) => &s.name,
            Self::Passive(s) => &s.name,
        }
    }

    /// Get the skill's ID
    pub fn id(&self) -> SkillId {
        match self {
            Self::Active(s) => s.id,
            Self::Passive(s) => s.id,
        }
    }
}

/// A skill slot that tracks cooldown state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSlot {
    /// The skill in this slot (if any)
    pub skill: Option<Skill>,
    /// Remaining cooldown in seconds (not serialized — resets on load)
    #[serde(skip)]
    pub cooldown_remaining: f32,
}

impl Default for SkillSlot {
    fn default() -> Self {
        Self {
            skill: None,
            cooldown_remaining: 0.0,
        }
    }
}

impl SkillSlot {
    /// Create an empty skill slot
    pub fn empty() -> Self {
        Self::default()
    }

    /// Create a slot with a skill
    pub fn with_skill(skill: Skill) -> Self {
        Self {
            skill: Some(skill),
            cooldown_remaining: 0.0,
        }
    }

    /// Try to activate the skill. Returns true if activation succeeded.
    pub fn try_activate(&mut self) -> bool {
        if self.cooldown_remaining > 0.0 {
            return false;
        }
        if let Some(Skill::Active(active)) = &self.skill {
            self.cooldown_remaining = active.cooldown;
            true
        } else {
            false
        }
    }

    /// Update cooldown timer
    pub fn update(&mut self, delta: f32) {
        if self.cooldown_remaining > 0.0 {
            self.cooldown_remaining = (self.cooldown_remaining - delta).max(0.0);
        }
    }

    /// Whether this slot is on cooldown
    pub fn is_on_cooldown(&self) -> bool {
        self.cooldown_remaining > 0.0
    }

    /// Cooldown progress as a 0.0-1.0 fraction (1.0 = ready)
    pub fn cooldown_fraction(&self) -> f32 {
        if let Some(Skill::Active(active)) = &self.skill {
            if active.cooldown <= 0.0 {
                return 1.0;
            }
            1.0 - (self.cooldown_remaining / active.cooldown).clamp(0.0, 1.0)
        } else {
            1.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_active_skill() -> ActiveSkill {
        ActiveSkill {
            id: SkillId(1),
            name: "Fireball".to_string(),
            description: "Hurls a ball of fire".to_string(),
            element: Element::Fire,
            shape: SkillShape::Bolt,
            target: SkillTarget::Projectile {
                speed: 20.0,
                range: 30.0,
            },
            base_damage: 25.0,
            damage_multiplier: 1.5,
            cooldown: 3.0,
            cost: 10.0,
            applies_status: Some(StatusEffectType::Burning),
            status_duration: 5.0,
        }
    }

    #[test]
    fn test_skill_slot_activation() {
        let mut slot = SkillSlot::with_skill(Skill::Active(test_active_skill()));
        assert!(!slot.is_on_cooldown());
        assert!(slot.try_activate());
        assert!(slot.is_on_cooldown());
        assert!(!slot.try_activate()); // can't activate while on cooldown
    }

    #[test]
    fn test_skill_slot_cooldown_recovery() {
        let mut slot = SkillSlot::with_skill(Skill::Active(test_active_skill()));
        slot.try_activate();
        assert!(slot.is_on_cooldown());

        // Not enough time
        slot.update(1.0);
        assert!(slot.is_on_cooldown());

        // Enough time to fully recover
        slot.update(3.0);
        assert!(!slot.is_on_cooldown());
        assert!(slot.try_activate()); // can activate again
    }

    #[test]
    fn test_empty_slot_cannot_activate() {
        let mut slot = SkillSlot::empty();
        assert!(!slot.try_activate());
    }

    #[test]
    fn test_passive_slot_cannot_activate() {
        let passive = PassiveSkill {
            id: SkillId(2),
            name: "Fire Mastery".to_string(),
            description: "Increases fire damage".to_string(),
            stat_modifiers: StatModifiers::default(),
            proc_chance: 0.0,
            proc_status: None,
            proc_duration: 0.0,
        };
        let mut slot = SkillSlot::with_skill(Skill::Passive(passive));
        assert!(!slot.try_activate());
    }

    #[test]
    fn test_cooldown_fraction() {
        let mut slot = SkillSlot::with_skill(Skill::Active(test_active_skill()));
        assert_eq!(slot.cooldown_fraction(), 1.0); // ready

        slot.try_activate();
        assert!(slot.cooldown_fraction() < 0.01); // just activated

        slot.update(1.5); // 3s cooldown, 1.5s passed
        let frac = slot.cooldown_fraction();
        assert!((frac - 0.5).abs() < 0.01);

        slot.update(1.5); // fully recovered
        assert_eq!(slot.cooldown_fraction(), 1.0);
    }
}
