//! Player stats and progression system
//!
//! Provides character stats, leveling, and XP calculations.

use serde::{Deserialize, Serialize};

use crate::combat::damage::StatModifiers;
use crate::combat::element::Element;

/// Core combat stats for player and NPCs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterStats {
    /// Maximum health points
    pub max_hp: f32,
    /// Current health points
    pub current_hp: f32,
    /// Attack power
    pub attack: f32,
    /// Defense (damage reduction)
    pub defense: f32,
    /// Movement speed multiplier
    pub speed: f32,
    /// Critical hit chance (0.0 - 1.0)
    pub crit_chance: f32,
    /// Critical hit damage multiplier (usually 1.5 - 2.0)
    pub crit_multiplier: f32,
    /// Elemental affinity
    #[serde(default)]
    pub elemental_affinity: Element,
    /// Maximum mana points
    #[serde(default = "default_max_mana")]
    pub max_mana: f32,
    /// Current mana points
    #[serde(default = "default_max_mana")]
    pub current_mana: f32,
    /// Mana regeneration per second
    #[serde(default = "default_mana_regen")]
    pub mana_regen: f32,
}

fn default_max_mana() -> f32 {
    100.0
}

fn default_mana_regen() -> f32 {
    2.0
}

impl Default for CharacterStats {
    fn default() -> Self {
        Self {
            max_hp: 100.0,
            current_hp: 100.0,
            attack: 10.0,
            defense: 5.0,
            speed: 1.0,
            crit_chance: 0.05,
            crit_multiplier: 1.5,
            elemental_affinity: Element::Physical,
            max_mana: 100.0,
            current_mana: 100.0,
            mana_regen: 2.0,
        }
    }
}

impl CharacterStats {
    /// Create new stats with full HP
    pub fn new(max_hp: f32, attack: f32, defense: f32, speed: f32) -> Self {
        Self {
            max_hp,
            current_hp: max_hp,
            attack,
            defense,
            speed,
            crit_chance: 0.05,
            crit_multiplier: 1.5,
            elemental_affinity: Element::Physical,
            max_mana: 100.0,
            current_mana: 100.0,
            mana_regen: 2.0,
        }
    }

    /// Compute effective stats after applying equipment/buff modifiers
    pub fn effective_stats(&self, modifiers: &StatModifiers) -> CharacterStats {
        CharacterStats {
            max_hp: self.max_hp + modifiers.max_hp,
            current_hp: self.current_hp,
            attack: self.attack + modifiers.attack,
            defense: self.defense + modifiers.defense,
            speed: self.speed + modifiers.speed,
            crit_chance: (self.crit_chance + modifiers.crit_chance).clamp(0.0, 1.0),
            crit_multiplier: self.crit_multiplier + modifiers.crit_multiplier,
            elemental_affinity: self.elemental_affinity,
            max_mana: self.max_mana,
            current_mana: self.current_mana,
            mana_regen: self.mana_regen,
        }
    }

    /// HP as a 0.0-1.0 fraction
    pub fn hp_fraction(&self) -> f32 {
        if self.max_hp <= 0.0 {
            return 0.0;
        }
        (self.current_hp / self.max_hp).clamp(0.0, 1.0)
    }

    /// Whether this character is alive
    pub fn is_alive(&self) -> bool {
        self.current_hp > 0.0
    }

    /// Calculate damage dealt to a target with given defense
    /// Returns (damage, is_crit)
    pub fn calculate_damage(&self, target_defense: f32) -> (f32, bool) {
        let is_crit = rand::random::<f32>() < self.crit_chance;
        let base_damage = self.attack - target_defense * 0.5;
        let damage = if is_crit {
            base_damage * self.crit_multiplier
        } else {
            base_damage
        };
        (damage.max(1.0), is_crit)
    }

    /// Calculate damage without crit (for NPCs attacking player)
    pub fn calculate_damage_no_crit(&self, target_defense: f32) -> f32 {
        (self.attack - target_defense * 0.5).max(1.0)
    }

    /// Apply level-up stat growth
    pub fn apply_growth(&mut self, growth: &StatGrowth) {
        self.max_hp += growth.hp_per_level;
        self.attack += growth.attack_per_level;
        self.defense += growth.defense_per_level;
        self.speed += growth.speed_per_level;
        // Full heal and mana restore on level up
        self.current_hp = self.max_hp;
        self.current_mana = self.max_mana;
    }

    /// Heal by a percentage of max HP
    pub fn heal_percent(&mut self, percent: f32) {
        let heal_amount = self.max_hp * percent;
        self.current_hp = (self.current_hp + heal_amount).min(self.max_hp);
    }

    /// Heal by a flat amount
    pub fn heal(&mut self, amount: f32) {
        self.current_hp = (self.current_hp + amount).min(self.max_hp);
    }

    /// Try to spend mana. Returns false if insufficient.
    pub fn use_mana(&mut self, cost: f32) -> bool {
        if self.current_mana >= cost {
            self.current_mana -= cost;
            true
        } else {
            false
        }
    }

    /// Regenerate mana passively each frame
    pub fn regenerate_mana(&mut self, delta: f32) {
        self.current_mana = (self.current_mana + self.mana_regen * delta).min(self.max_mana);
    }

    /// Mana as a 0.0-1.0 fraction
    pub fn mana_fraction(&self) -> f32 {
        if self.max_mana <= 0.0 {
            return 0.0;
        }
        (self.current_mana / self.max_mana).clamp(0.0, 1.0)
    }
}

/// Per-level stat growth rates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatGrowth {
    /// HP gained per level
    pub hp_per_level: f32,
    /// Attack gained per level
    pub attack_per_level: f32,
    /// Defense gained per level
    pub defense_per_level: f32,
    /// Speed gained per level
    pub speed_per_level: f32,
}

impl Default for StatGrowth {
    fn default() -> Self {
        Self {
            hp_per_level: 10.0,
            attack_per_level: 2.0,
            defense_per_level: 1.0,
            speed_per_level: 0.0,
        }
    }
}

/// Player-specific progression data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerProgression {
    /// Current level (starts at 1)
    pub level: u32,
    /// XP earned toward next level
    pub current_xp: u64,
    /// Total XP earned across all levels
    pub total_xp: u64,
}

impl Default for PlayerProgression {
    fn default() -> Self {
        Self {
            level: 1,
            current_xp: 0,
            total_xp: 0,
        }
    }
}

impl PlayerProgression {
    /// Create new progression at level 1
    pub fn new() -> Self {
        Self::default()
    }

    /// XP required to reach a given level (cumulative from level 1)
    /// Formula: 100 * level^1.5 (rounded)
    pub fn xp_for_level(level: u32) -> u64 {
        if level <= 1 {
            return 0;
        }
        (100.0 * (level as f64).powf(1.5)).round() as u64
    }

    /// XP needed to go from current level to next level
    pub fn xp_to_next_level(&self) -> u64 {
        Self::xp_for_level(self.level + 1) - Self::xp_for_level(self.level)
    }

    /// Progress toward next level as a 0.0-1.0 fraction
    pub fn xp_fraction(&self) -> f32 {
        let needed = self.xp_to_next_level();
        if needed == 0 {
            return 1.0;
        }
        (self.current_xp as f32 / needed as f32).clamp(0.0, 1.0)
    }

    /// Add XP and return a list of levels gained
    pub fn add_xp(&mut self, amount: u64) -> Vec<u32> {
        self.current_xp += amount;
        self.total_xp += amount;

        let mut levels_gained = Vec::new();
        let mut needed = self.xp_to_next_level();

        while self.current_xp >= needed {
            self.current_xp -= needed;
            self.level += 1;
            levels_gained.push(self.level);
            needed = self.xp_to_next_level();
        }

        levels_gained
    }
}

/// Enemy type for XP reward calculation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyType {
    /// Standard enemy
    Normal,
    /// Stronger enemy with better rewards
    Elite,
    /// Boss enemy with significant rewards
    Boss,
}

/// Calculate XP reward for defeating an enemy
pub fn xp_for_enemy(enemy_level: u32, enemy_type: EnemyType) -> u64 {
    let base = match enemy_type {
        EnemyType::Normal => 10,
        EnemyType::Elite => 50,
        EnemyType::Boss => 200,
    };
    base * (enemy_level as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_character_stats_default() {
        let stats = CharacterStats::default();
        assert!(stats.is_alive());
        assert_eq!(stats.hp_fraction(), 1.0);
    }

    #[test]
    fn test_damage_calculation() {
        let stats = CharacterStats::new(100.0, 20.0, 5.0, 1.0);
        // With 10 defense, damage should be 20 - 10*0.5 = 15 (minimum)
        let (damage, _) = stats.calculate_damage(10.0);
        assert!(damage >= 1.0);
    }

    #[test]
    fn test_damage_minimum() {
        let stats = CharacterStats::new(100.0, 5.0, 5.0, 1.0);
        let (damage, _) = stats.calculate_damage(100.0);
        assert_eq!(damage, 1.0); // Minimum 1 damage
    }

    #[test]
    fn test_stat_growth() {
        let mut stats = CharacterStats::new(100.0, 10.0, 5.0, 1.0);
        stats.current_hp = 50.0; // Damage the player

        let growth = StatGrowth {
            hp_per_level: 15.0,
            attack_per_level: 3.0,
            defense_per_level: 2.0,
            speed_per_level: 0.05,
        };

        stats.apply_growth(&growth);

        assert_eq!(stats.max_hp, 115.0);
        assert_eq!(stats.current_hp, 115.0); // Full heal
        assert_eq!(stats.attack, 13.0);
        assert_eq!(stats.defense, 7.0);
    }

    #[test]
    fn test_xp_for_level() {
        // Level 1 requires 0 XP (you start there)
        assert_eq!(PlayerProgression::xp_for_level(1), 0);
        // Level 2 requires 100 * 2^1.5 ≈ 283
        let level2_xp = PlayerProgression::xp_for_level(2);
        assert!(level2_xp > 200 && level2_xp < 400);
    }

    #[test]
    fn test_xp_progression() {
        let mut prog = PlayerProgression::new();
        assert_eq!(prog.level, 1);

        // Add enough XP to level up
        let needed = prog.xp_to_next_level();
        let levels = prog.add_xp(needed);

        assert_eq!(levels, vec![2]);
        assert_eq!(prog.level, 2);
        assert_eq!(prog.current_xp, 0);
    }

    #[test]
    fn test_multi_level_up() {
        let mut prog = PlayerProgression::new();

        // Add massive XP to gain multiple levels
        let levels = prog.add_xp(10000);

        assert!(levels.len() > 1);
        assert!(prog.level > 2);
    }

    #[test]
    fn test_xp_fraction() {
        let mut prog = PlayerProgression::new();
        assert_eq!(prog.xp_fraction(), 0.0);

        let needed = prog.xp_to_next_level();
        prog.add_xp(needed / 2);

        let frac = prog.xp_fraction();
        assert!(frac > 0.4 && frac < 0.6);
    }

    #[test]
    fn test_xp_for_enemy() {
        assert_eq!(xp_for_enemy(1, EnemyType::Normal), 10);
        assert_eq!(xp_for_enemy(5, EnemyType::Normal), 50);
        assert_eq!(xp_for_enemy(1, EnemyType::Elite), 50);
        assert_eq!(xp_for_enemy(1, EnemyType::Boss), 200);
    }

    #[test]
    fn test_heal() {
        let mut stats = CharacterStats::new(100.0, 10.0, 5.0, 1.0);
        stats.current_hp = 50.0;

        stats.heal(30.0);
        assert_eq!(stats.current_hp, 80.0);

        stats.heal(100.0); // Over-heal should cap
        assert_eq!(stats.current_hp, 100.0);
    }

    #[test]
    fn test_heal_percent() {
        let mut stats = CharacterStats::new(100.0, 10.0, 5.0, 1.0);
        stats.current_hp = 50.0;

        stats.heal_percent(0.25); // 25% of 100 = 25 HP
        assert_eq!(stats.current_hp, 75.0);
    }

    #[test]
    fn test_mana_default() {
        let stats = CharacterStats::default();
        assert_eq!(stats.max_mana, 100.0);
        assert_eq!(stats.current_mana, 100.0);
        assert_eq!(stats.mana_regen, 2.0);
        assert_eq!(stats.mana_fraction(), 1.0);
    }

    #[test]
    fn test_use_mana() {
        let mut stats = CharacterStats::default();
        assert!(stats.use_mana(30.0));
        assert_eq!(stats.current_mana, 70.0);

        // Can't spend more than available
        assert!(!stats.use_mana(80.0));
        assert_eq!(stats.current_mana, 70.0);
    }

    #[test]
    fn test_regenerate_mana() {
        let mut stats = CharacterStats::default();
        stats.current_mana = 50.0;

        stats.regenerate_mana(1.0); // 2.0 per second
        assert_eq!(stats.current_mana, 52.0);

        // Should cap at max
        stats.current_mana = 99.5;
        stats.regenerate_mana(1.0);
        assert_eq!(stats.current_mana, 100.0);
    }

    #[test]
    fn test_mana_fraction() {
        let mut stats = CharacterStats::default();
        assert_eq!(stats.mana_fraction(), 1.0);

        stats.current_mana = 50.0;
        assert_eq!(stats.mana_fraction(), 0.5);

        stats.current_mana = 0.0;
        assert_eq!(stats.mana_fraction(), 0.0);
    }
}
