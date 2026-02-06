//! Combat system module
//!
//! Provides elements, damage calculation, weapons, items, equipment,
//! gems, skills, rune composition, and status effects.

pub mod damage;
pub mod element;
pub mod equipment;
pub mod gem;
pub mod item;
pub mod rune;
pub mod skill;
pub mod status;
pub mod weapon;

pub use damage::{AttackType, DamageEvent, StatModifiers, calculate_combat_damage};
pub use element::Element;
pub use equipment::{EquipError, EquipmentSet, EquipmentSlot};
pub use gem::{Gem, GemQuality, GemShape};
pub use item::{GemSocket, Item, ItemCategory, ItemId, ItemRarity};
pub use rune::{ComposedSpell, Rune, RuneAmplifier, RuneAspect, RuneComposer, RuneModifier};
pub use skill::{ActiveSkill, PassiveSkill, Skill, SkillId, SkillSlot, SkillShape, SkillTarget, MAX_SKILL_SLOTS};
pub use status::{StatusEffect, StatusEffectType, StatusManager};
pub use weapon::{WeaponData, WeaponGrip, WeaponRange, WeaponType};
