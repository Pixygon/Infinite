//! Status effects and status manager
//!
//! Handles damage over time, movement prevention, stat modifications, and shields.

use serde::{Deserialize, Serialize};

use super::damage::StatModifiers;
use super::element::Element;

/// Types of status effects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StatusEffectType {
    // Elemental procs
    Burning,   // Fire
    Frozen,    // Water
    Shocked,   // Air
    Rooted,    // Earth
    Silenced,  // Void
    Blessed,   // Meta

    // General
    Poisoned,
    Stunned,
    Slowed,
    Weakened,
    Empowered,
    Hastened,
    Shielded,
}

impl StatusEffectType {
    /// Get the element associated with this status (if it's an elemental proc)
    pub fn element(self) -> Option<Element> {
        match self {
            Self::Burning => Some(Element::Fire),
            Self::Frozen => Some(Element::Water),
            Self::Shocked => Some(Element::Air),
            Self::Rooted => Some(Element::Earth),
            Self::Silenced => Some(Element::Void),
            Self::Blessed => Some(Element::Meta),
            _ => None,
        }
    }

    /// Display name
    pub fn name(self) -> &'static str {
        match self {
            Self::Burning => "Burning",
            Self::Frozen => "Frozen",
            Self::Shocked => "Shocked",
            Self::Rooted => "Rooted",
            Self::Silenced => "Silenced",
            Self::Blessed => "Blessed",
            Self::Poisoned => "Poisoned",
            Self::Stunned => "Stunned",
            Self::Slowed => "Slowed",
            Self::Weakened => "Weakened",
            Self::Empowered => "Empowered",
            Self::Hastened => "Hastened",
            Self::Shielded => "Shielded",
        }
    }
}

/// An active status effect instance
#[derive(Debug, Clone)]
pub struct StatusEffect {
    pub effect_type: StatusEffectType,
    /// Remaining duration in seconds
    pub duration: f32,
    /// Damage dealt per tick (for DOT effects)
    pub damage_per_tick: f32,
    /// Time between ticks
    pub tick_interval: f32,
    /// Timer tracking next tick
    pub tick_timer: f32,
    /// Stat modifiers applied while active
    pub modifiers: StatModifiers,
    /// Shield HP (for Shielded effect)
    pub shield_hp: f32,
    /// Max shield HP
    pub shield_hp_max: f32,
}

impl StatusEffect {
    /// Create an elemental proc status effect
    pub fn elemental_proc(effect_type: StatusEffectType, duration: f32) -> Self {
        let (damage_per_tick, tick_interval) = match effect_type {
            StatusEffectType::Burning => (5.0, 1.0),
            StatusEffectType::Poisoned => (3.0, 1.5),
            StatusEffectType::Shocked => (8.0, 2.0),
            _ => (0.0, 1.0),
        };

        let modifiers = match effect_type {
            StatusEffectType::Frozen => StatModifiers {
                speed: -0.8,
                ..Default::default()
            },
            StatusEffectType::Slowed => StatModifiers {
                speed: -0.4,
                ..Default::default()
            },
            StatusEffectType::Weakened => StatModifiers {
                attack: -5.0,
                defense: -3.0,
                ..Default::default()
            },
            StatusEffectType::Empowered => StatModifiers {
                attack: 10.0,
                crit_chance: 0.1,
                ..Default::default()
            },
            StatusEffectType::Hastened => StatModifiers {
                speed: 0.5,
                ..Default::default()
            },
            StatusEffectType::Blessed => StatModifiers {
                max_hp: 20.0,
                defense: 5.0,
                ..Default::default()
            },
            _ => StatModifiers::default(),
        };

        Self {
            effect_type,
            duration,
            damage_per_tick,
            tick_interval,
            tick_timer: tick_interval,
            modifiers,
            shield_hp: 0.0,
            shield_hp_max: 0.0,
        }
    }

    /// Create a stat modifier status effect
    pub fn stat_modifier(
        effect_type: StatusEffectType,
        duration: f32,
        modifiers: StatModifiers,
    ) -> Self {
        Self {
            effect_type,
            duration,
            damage_per_tick: 0.0,
            tick_interval: 1.0,
            tick_timer: 1.0,
            modifiers,
            shield_hp: 0.0,
            shield_hp_max: 0.0,
        }
    }

    /// Create a shield status effect
    pub fn shield(shield_hp: f32, duration: f32) -> Self {
        Self {
            effect_type: StatusEffectType::Shielded,
            duration,
            damage_per_tick: 0.0,
            tick_interval: 1.0,
            tick_timer: 1.0,
            modifiers: StatModifiers::default(),
            shield_hp,
            shield_hp_max: shield_hp,
        }
    }

    /// Whether this effect prevents movement
    pub fn prevents_movement(&self) -> bool {
        matches!(
            self.effect_type,
            StatusEffectType::Frozen | StatusEffectType::Rooted | StatusEffectType::Stunned
        )
    }

    /// Whether this effect prevents skill usage
    pub fn prevents_skills(&self) -> bool {
        matches!(
            self.effect_type,
            StatusEffectType::Silenced | StatusEffectType::Stunned
        )
    }

    /// Whether this effect prevents attacking
    pub fn prevents_attacks(&self) -> bool {
        matches!(self.effect_type, StatusEffectType::Stunned)
    }

    /// Whether this effect has expired
    pub fn is_expired(&self) -> bool {
        self.duration <= 0.0
    }
}

/// Manages all active status effects on an entity
#[derive(Debug, Clone, Default)]
pub struct StatusManager {
    pub effects: Vec<StatusEffect>,
}

impl StatusManager {
    /// Create a new empty status manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply a status effect. If the same type already exists, refresh duration (take longer).
    pub fn apply(&mut self, effect: StatusEffect) {
        if let Some(existing) = self.effects.iter_mut().find(|e| e.effect_type == effect.effect_type) {
            // Refresh: take the longer duration
            if effect.duration > existing.duration {
                existing.duration = effect.duration;
            }
            // Update shield HP to max
            if effect.effect_type == StatusEffectType::Shielded {
                existing.shield_hp = effect.shield_hp.max(existing.shield_hp);
                existing.shield_hp_max = existing.shield_hp;
            }
        } else {
            self.effects.push(effect);
        }
    }

    /// Update all effects. Returns total DOT damage dealt this frame.
    pub fn update(&mut self, delta: f32) -> f32 {
        let mut dot_damage = 0.0;

        for effect in &mut self.effects {
            effect.duration -= delta;

            // DOT ticking
            if effect.damage_per_tick > 0.0 {
                effect.tick_timer -= delta;
                if effect.tick_timer <= 0.0 {
                    dot_damage += effect.damage_per_tick;
                    effect.tick_timer += effect.tick_interval;
                }
            }
        }

        // Remove expired effects
        self.effects.retain(|e| !e.is_expired());

        dot_damage
    }

    /// Combined stat modifiers from all active effects
    pub fn combined_modifiers(&self) -> StatModifiers {
        let mut total = StatModifiers::default();
        for effect in &self.effects {
            total.add(&effect.modifiers);
        }
        total
    }

    /// Absorb damage through shields. Returns remaining damage after shields.
    pub fn absorb_damage(&mut self, damage: f32) -> f32 {
        let mut remaining = damage;
        for effect in &mut self.effects {
            if effect.effect_type == StatusEffectType::Shielded && effect.shield_hp > 0.0 {
                if remaining <= effect.shield_hp {
                    effect.shield_hp -= remaining;
                    return 0.0;
                } else {
                    remaining -= effect.shield_hp;
                    effect.shield_hp = 0.0;
                    effect.duration = 0.0; // shield broken
                }
            }
        }
        remaining
    }

    /// Whether any active effect prevents movement
    pub fn is_movement_prevented(&self) -> bool {
        self.effects.iter().any(|e| e.prevents_movement())
    }

    /// Whether any active effect prevents skill usage
    pub fn are_skills_prevented(&self) -> bool {
        self.effects.iter().any(|e| e.prevents_skills())
    }

    /// Whether any active effect prevents attacking
    pub fn are_attacks_prevented(&self) -> bool {
        self.effects.iter().any(|e| e.prevents_attacks())
    }

    /// Check if a specific status type is active
    pub fn has_effect(&self, effect_type: StatusEffectType) -> bool {
        self.effects.iter().any(|e| e.effect_type == effect_type)
    }

    /// Remove all effects
    pub fn clear(&mut self) {
        self.effects.clear();
    }

    /// Number of active effects
    pub fn count(&self) -> usize {
        self.effects.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_burning_dot() {
        let mut mgr = StatusManager::new();
        mgr.apply(StatusEffect::elemental_proc(StatusEffectType::Burning, 5.0));

        // Tick for 1 second (burning ticks every 1.0s)
        let dot = mgr.update(1.0);
        assert_eq!(dot, 5.0);
    }

    #[test]
    fn test_effect_expires() {
        let mut mgr = StatusManager::new();
        mgr.apply(StatusEffect::elemental_proc(StatusEffectType::Burning, 2.0));
        assert_eq!(mgr.count(), 1);

        mgr.update(3.0); // exceed duration
        assert_eq!(mgr.count(), 0);
    }

    #[test]
    fn test_refresh_duration() {
        let mut mgr = StatusManager::new();
        mgr.apply(StatusEffect::elemental_proc(StatusEffectType::Burning, 3.0));
        mgr.update(1.0); // 2.0 remaining

        // Apply again with longer duration
        mgr.apply(StatusEffect::elemental_proc(StatusEffectType::Burning, 5.0));
        assert_eq!(mgr.count(), 1); // still only one
        assert_eq!(mgr.effects[0].duration, 5.0);
    }

    #[test]
    fn test_movement_prevention() {
        let mut mgr = StatusManager::new();
        assert!(!mgr.is_movement_prevented());

        mgr.apply(StatusEffect::elemental_proc(StatusEffectType::Frozen, 3.0));
        assert!(mgr.is_movement_prevented());
    }

    #[test]
    fn test_stunned_prevents_everything() {
        let mut mgr = StatusManager::new();
        mgr.apply(StatusEffect::elemental_proc(StatusEffectType::Stunned, 2.0));
        assert!(mgr.is_movement_prevented());
        assert!(mgr.are_skills_prevented());
        assert!(mgr.are_attacks_prevented());
    }

    #[test]
    fn test_silenced_prevents_skills_only() {
        let mut mgr = StatusManager::new();
        mgr.apply(StatusEffect::elemental_proc(StatusEffectType::Silenced, 3.0));
        assert!(!mgr.is_movement_prevented());
        assert!(mgr.are_skills_prevented());
        assert!(!mgr.are_attacks_prevented());
    }

    #[test]
    fn test_shield_absorb() {
        let mut mgr = StatusManager::new();
        mgr.apply(StatusEffect::shield(50.0, 10.0));

        let remaining = mgr.absorb_damage(30.0);
        assert_eq!(remaining, 0.0);
        assert!(mgr.has_effect(StatusEffectType::Shielded));

        let remaining = mgr.absorb_damage(30.0);
        assert_eq!(remaining, 10.0); // 20 absorbed, 10 remaining
    }

    #[test]
    fn test_combined_modifiers() {
        let mut mgr = StatusManager::new();
        mgr.apply(StatusEffect::elemental_proc(StatusEffectType::Empowered, 5.0));
        mgr.apply(StatusEffect::elemental_proc(StatusEffectType::Hastened, 5.0));

        let mods = mgr.combined_modifiers();
        assert_eq!(mods.attack, 10.0);
        assert_eq!(mods.speed, 0.5);
    }

    #[test]
    fn test_has_effect() {
        let mut mgr = StatusManager::new();
        assert!(!mgr.has_effect(StatusEffectType::Burning));
        mgr.apply(StatusEffect::elemental_proc(StatusEffectType::Burning, 5.0));
        assert!(mgr.has_effect(StatusEffectType::Burning));
    }

    #[test]
    fn test_clear() {
        let mut mgr = StatusManager::new();
        mgr.apply(StatusEffect::elemental_proc(StatusEffectType::Burning, 5.0));
        mgr.apply(StatusEffect::elemental_proc(StatusEffectType::Frozen, 5.0));
        assert_eq!(mgr.count(), 2);
        mgr.clear();
        assert_eq!(mgr.count(), 0);
    }
}
