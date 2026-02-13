//! NPC combat system — stats, damage calculation, and aggro

use crate::combat::damage::{AttackType, calculate_combat_damage};
use crate::combat::element::Element;
use crate::combat::equipment::EquipmentSet;
use crate::combat::inventory::Inventory;
use crate::combat::rune::{Rune, RuneComposer};
use crate::combat::skill::SkillSlot;
use crate::combat::status::StatusManager;
use crate::combat::weapon::WeaponType;
use crate::player::stats::{CharacterStats, PlayerProgression, StatGrowth};
use serde::{Deserialize, Serialize};

/// Basic combat statistics for an NPC
#[derive(Debug, Clone)]
pub struct CombatStats {
    pub max_hp: f32,
    pub current_hp: f32,
    pub attack: f32,
    pub defense: f32,
    pub speed: f32,
    pub aggro_radius: f32,
    pub de_aggro_radius: f32,
    pub attack_radius: f32,
    pub attack_cooldown: f32,
    pub attack_timer: f32,
    /// NPC's elemental affinity
    pub element: Element,
    /// Weapon type this NPC is weak against
    pub weapon_weakness: Option<WeaponType>,
}

impl CombatStats {
    /// Default stats for a standard enemy
    pub fn default_enemy() -> Self {
        Self {
            max_hp: 50.0,
            current_hp: 50.0,
            attack: 5.0,
            defense: 2.0,
            speed: 1.2,
            aggro_radius: 12.0,
            de_aggro_radius: 20.0,
            attack_radius: 2.5,
            attack_cooldown: 1.5,
            attack_timer: 0.0,
            element: Element::Physical,
            weapon_weakness: None,
        }
    }

    /// Create an elemental enemy
    pub fn elemental_enemy(element: Element) -> Self {
        let mut stats = Self::default_enemy();
        stats.element = element;
        stats
    }

    /// Create a boss enemy
    pub fn boss(element: Element) -> Self {
        Self {
            max_hp: 500.0,
            current_hp: 500.0,
            attack: 15.0,
            defense: 8.0,
            speed: 0.8,
            aggro_radius: 20.0,
            de_aggro_radius: 30.0,
            attack_radius: 3.5,
            attack_cooldown: 2.0,
            attack_timer: 0.0,
            element,
            weapon_weakness: None,
        }
    }

    /// HP as a 0.0-1.0 fraction
    pub fn hp_fraction(&self) -> f32 {
        if self.max_hp <= 0.0 {
            return 0.0;
        }
        self.current_hp / self.max_hp
    }

    /// Whether this NPC is alive
    pub fn is_alive(&self) -> bool {
        self.current_hp > 0.0
    }

    /// Calculate damage dealt to a target with given defense
    pub fn calculate_damage(&self, target_defense: f32) -> f32 {
        (self.attack - target_defense).max(1.0)
    }

    /// Update attack cooldown timer. Returns true if an attack can fire.
    pub fn update_attack(&mut self, delta: f32) -> bool {
        self.attack_timer -= delta;
        if self.attack_timer <= 0.0 {
            self.attack_timer = self.attack_cooldown;
            true
        } else {
            false
        }
    }
}

/// Player combat state with full stats, progression, and attack mechanics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerCombatState {
    /// Character combat stats (HP, attack, defense, etc.)
    pub stats: CharacterStats,
    /// Level and XP progression
    pub progression: PlayerProgression,
    /// Timer for damage flash effect
    #[serde(skip)]
    pub damage_flash_timer: f32,
    /// Last damage amount taken (for display)
    #[serde(skip)]
    pub last_damage_amount: f32,
    /// Attack cooldown duration in seconds
    pub attack_cooldown: f32,
    /// Current attack cooldown timer
    #[serde(skip)]
    pub attack_timer: f32,
    /// Whether player is currently in an attack animation
    #[serde(skip)]
    pub is_attacking: bool,
    /// Invincibility frames timer (prevents damage spam)
    #[serde(skip)]
    pub invincibility_timer: f32,
    /// Equipment set
    #[serde(default)]
    pub equipment: EquipmentSet,
    /// Skill slots (4 max)
    #[serde(default = "default_skill_slots")]
    pub skill_slots: Vec<SkillSlot>,
    /// Known runes the player has collected
    #[serde(default)]
    pub known_runes: Vec<Rune>,
    /// Active rune composer (runtime only)
    #[serde(skip)]
    pub rune_composer: RuneComposer,
    /// Status effect manager (runtime only)
    #[serde(skip)]
    pub status_manager: StatusManager,
    /// Current attack type being performed (runtime only)
    #[serde(skip)]
    pub active_attack_type: Option<AttackType>,
    /// Heavy attack windup timer (runtime only)
    #[serde(skip)]
    pub heavy_attack_timer: f32,
    /// Player inventory
    #[serde(default)]
    pub inventory: Inventory,
    /// Dodge cooldown duration in seconds
    #[serde(skip)]
    pub dodge_cooldown: f32,
    /// Remaining dodge cooldown timer
    #[serde(skip)]
    pub dodge_cooldown_timer: f32,
    /// Whether the player is currently dodging
    #[serde(skip)]
    pub is_dodging: bool,
    /// Remaining dodge duration timer
    #[serde(skip)]
    pub dodge_timer: f32,
    /// Dodge duration in seconds
    #[serde(skip)]
    pub dodge_duration: f32,
}

fn default_skill_slots() -> Vec<SkillSlot> {
    vec![SkillSlot::empty(); 4]
}

impl PlayerCombatState {
    /// Create new player combat state with default stats
    pub fn new() -> Self {
        Self {
            stats: CharacterStats::default(),
            progression: PlayerProgression::new(),
            damage_flash_timer: 0.0,
            last_damage_amount: 0.0,
            attack_cooldown: 0.5,
            attack_timer: 0.0,
            is_attacking: false,
            invincibility_timer: 0.0,
            equipment: EquipmentSet::default(),
            skill_slots: default_skill_slots(),
            known_runes: Vec::new(),
            rune_composer: RuneComposer::default(),
            status_manager: StatusManager::new(),
            active_attack_type: None,
            heavy_attack_timer: 0.0,
            inventory: Inventory::new(),
            dodge_cooldown: 1.5,
            dodge_cooldown_timer: 0.0,
            is_dodging: false,
            dodge_timer: 0.0,
            dodge_duration: 0.3,
        }
    }

    /// Create player combat state from archetype base stats
    pub fn from_stats(stats: CharacterStats) -> Self {
        Self {
            stats,
            progression: PlayerProgression::new(),
            damage_flash_timer: 0.0,
            last_damage_amount: 0.0,
            attack_cooldown: 0.5,
            attack_timer: 0.0,
            is_attacking: false,
            invincibility_timer: 0.0,
            equipment: EquipmentSet::default(),
            skill_slots: default_skill_slots(),
            known_runes: Vec::new(),
            rune_composer: RuneComposer::default(),
            status_manager: StatusManager::new(),
            active_attack_type: None,
            heavy_attack_timer: 0.0,
            inventory: Inventory::new(),
            dodge_cooldown: 1.5,
            dodge_cooldown_timer: 0.0,
            is_dodging: false,
            dodge_timer: 0.0,
            dodge_duration: 0.3,
        }
    }

    /// HP as a 0.0-1.0 fraction
    pub fn hp_fraction(&self) -> f32 {
        self.stats.hp_fraction()
    }

    /// Current HP (for backward compatibility)
    pub fn current_hp(&self) -> f32 {
        self.stats.current_hp
    }

    /// Max HP (for backward compatibility)
    pub fn max_hp(&self) -> f32 {
        self.stats.max_hp
    }

    /// Take damage (respects invincibility frames)
    /// Returns the actual damage taken (0 if invincible)
    pub fn take_damage(&mut self, damage: f32) -> f32 {
        if self.invincibility_timer > 0.0 {
            return 0.0;
        }

        let actual = (damage - self.stats.defense * 0.5).max(1.0);
        self.stats.current_hp = (self.stats.current_hp - actual).max(0.0);
        self.damage_flash_timer = 0.3;
        self.last_damage_amount = actual;
        self.invincibility_timer = 0.5; // Half second of i-frames

        actual
    }

    /// Whether the player is alive
    pub fn is_alive(&self) -> bool {
        self.stats.is_alive()
    }

    /// Get effective stats (base stats + equipment + status modifiers)
    pub fn effective_stats(&self) -> CharacterStats {
        let equip_mods = self.equipment.total_modifiers();
        let status_mods = self.status_manager.combined_modifiers();
        let combined = equip_mods.combined(&status_mods);
        self.stats.effective_stats(&combined)
    }

    /// Try to start an attack. Returns true if attack started.
    pub fn try_attack(&mut self) -> bool {
        if self.attack_timer <= 0.0 && !self.is_attacking {
            self.is_attacking = true;
            self.attack_timer = self.attack_cooldown;
            return true;
        }
        false
    }

    /// Try to start a light attack. Returns true if started.
    pub fn try_light_attack(&mut self) -> bool {
        if self.status_manager.are_attacks_prevented() {
            return false;
        }
        if self.attack_timer <= 0.0 && !self.is_attacking {
            self.is_attacking = true;
            self.active_attack_type = Some(AttackType::Light);
            // Apply weapon speed to cooldown
            let weapon_cd = self.equipment.main_weapon_type()
                .map(|wt| wt.light_attack_cooldown())
                .unwrap_or(AttackType::Light.base_cooldown());
            self.attack_cooldown = weapon_cd;
            self.attack_timer = weapon_cd;
            return true;
        }
        false
    }

    /// Try to start a heavy attack. Returns true if started.
    pub fn try_heavy_attack(&mut self) -> bool {
        if self.status_manager.are_attacks_prevented() {
            return false;
        }
        if self.attack_timer <= 0.0 && !self.is_attacking {
            self.is_attacking = true;
            self.active_attack_type = Some(AttackType::Heavy);
            let weapon_cd = self.equipment.main_weapon_type()
                .map(|wt| wt.heavy_attack_cooldown())
                .unwrap_or(AttackType::Heavy.base_cooldown());
            self.attack_cooldown = weapon_cd;
            self.attack_timer = weapon_cd;
            self.heavy_attack_timer = AttackType::Heavy.windup();
            return true;
        }
        false
    }

    /// Calculate full damage against a target using the damage pipeline
    pub fn calculate_full_damage(
        &self,
        target_defense: f32,
        target_element: Element,
        weapon_weakness: Option<WeaponType>,
    ) -> crate::combat::damage::DamageEvent {
        let effective = self.effective_stats();
        let equip_mods = self.equipment.total_modifiers();
        let attack_type = self.active_attack_type.unwrap_or(AttackType::Light);
        let element = self.stats.elemental_affinity;
        let elemental_bonus = equip_mods.elemental_damage_bonus[element.index()];

        calculate_combat_damage(
            effective.attack,
            self.equipment.main_weapon_damage(),
            self.equipment.main_weapon_type(),
            attack_type,
            element,
            effective.crit_chance,
            effective.crit_multiplier,
            elemental_bonus,
            target_defense,
            target_element,
            0.0, // target elemental resistance (NPCs don't have this yet)
            weapon_weakness,
        )
    }

    /// Try to use a skill in the given slot (0-3). Returns true if activated.
    pub fn try_use_skill(&mut self, slot: usize) -> bool {
        if self.status_manager.are_skills_prevented() {
            return false;
        }
        if slot < self.skill_slots.len() {
            self.skill_slots[slot].try_activate()
        } else {
            false
        }
    }

    /// Try to start a dodge. Returns true if dodge started.
    pub fn try_dodge(&mut self) -> bool {
        if self.dodge_cooldown_timer > 0.0 || self.is_dodging {
            return false;
        }
        if self.status_manager.is_movement_prevented() {
            return false;
        }
        self.is_dodging = true;
        self.dodge_timer = self.dodge_duration;
        self.dodge_cooldown_timer = self.dodge_cooldown;
        // Grant invincibility during dodge
        self.invincibility_timer = self.invincibility_timer.max(self.dodge_duration);
        true
    }

    /// Take elemental damage (applies status effects from element)
    pub fn take_elemental_damage(&mut self, damage: f32, _element: Element) -> f32 {
        // Let shields absorb first
        let after_shield = self.status_manager.absorb_damage(damage);
        if after_shield <= 0.0 {
            return 0.0;
        }
        self.take_damage(after_shield)
    }

    /// Check if we can deal damage (attack animation timing)
    /// Returns true once per attack when the "hit" frame occurs
    pub fn can_deal_damage(&self) -> bool {
        // Deal damage at the midpoint of the attack cooldown
        self.is_attacking && self.attack_timer < self.attack_cooldown * 0.5
    }

    /// Calculate damage to deal to a target
    /// Returns (damage, is_crit)
    pub fn calculate_damage(&self, target_defense: f32) -> (f32, bool) {
        self.stats.calculate_damage(target_defense)
    }

    /// Update timers each frame. Returns DOT damage taken from status effects.
    pub fn update(&mut self, delta: f32) -> f32 {
        // Attack cooldown
        if self.attack_timer > 0.0 {
            self.attack_timer = (self.attack_timer - delta).max(0.0);
        }

        // End attack animation at midpoint
        if self.is_attacking && self.attack_timer < self.attack_cooldown * 0.5 {
            self.is_attacking = false;
            self.active_attack_type = None;
        }

        // Heavy attack windup
        if self.heavy_attack_timer > 0.0 {
            self.heavy_attack_timer = (self.heavy_attack_timer - delta).max(0.0);
        }

        // Invincibility frames
        if self.invincibility_timer > 0.0 {
            self.invincibility_timer = (self.invincibility_timer - delta).max(0.0);
        }

        // Damage flash
        if self.damage_flash_timer > 0.0 {
            self.damage_flash_timer = (self.damage_flash_timer - delta).max(0.0);
        }

        // Dodge timers
        if self.dodge_timer > 0.0 {
            self.dodge_timer = (self.dodge_timer - delta).max(0.0);
            if self.dodge_timer <= 0.0 {
                self.is_dodging = false;
            }
        }
        if self.dodge_cooldown_timer > 0.0 {
            self.dodge_cooldown_timer = (self.dodge_cooldown_timer - delta).max(0.0);
        }

        // Mana regeneration
        self.stats.regenerate_mana(delta);

        // Skill cooldowns
        for slot in &mut self.skill_slots {
            slot.update(delta);
        }

        // Status effects (returns DOT damage)
        let dot_damage = self.status_manager.update(delta);
        if dot_damage > 0.0 {
            self.stats.current_hp = (self.stats.current_hp - dot_damage).max(0.0);
        }
        dot_damage
    }

    /// Add XP and return levels gained
    pub fn add_xp(&mut self, amount: u64) -> Vec<u32> {
        self.progression.add_xp(amount)
    }

    /// Apply stat growth for leveling up
    pub fn apply_level_up(&mut self, growth: &StatGrowth) {
        self.stats.apply_growth(growth);
    }

    /// Respawn player (full heal, reset state)
    pub fn respawn(&mut self) {
        self.stats.current_hp = self.stats.max_hp;
        self.stats.current_mana = self.stats.max_mana;
        self.damage_flash_timer = 0.0;
        self.attack_timer = 0.0;
        self.is_attacking = false;
        self.invincibility_timer = 1.0; // Brief invincibility on respawn
        self.active_attack_type = None;
        self.heavy_attack_timer = 0.0;
        self.is_dodging = false;
        self.dodge_timer = 0.0;
        self.dodge_cooldown_timer = 0.0;
        self.status_manager.clear();
    }

    /// Current level
    pub fn level(&self) -> u32 {
        self.progression.level
    }

    /// XP progress toward next level (0.0 - 1.0)
    pub fn xp_fraction(&self) -> f32 {
        self.progression.xp_fraction()
    }

    /// Current XP toward next level
    pub fn current_xp(&self) -> u64 {
        self.progression.current_xp
    }

    /// XP needed to reach next level
    pub fn xp_to_next_level(&self) -> u64 {
        self.progression.xp_to_next_level()
    }
}

impl Default for PlayerCombatState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_enemy_stats() {
        let stats = CombatStats::default_enemy();
        assert!(stats.is_alive());
        assert_eq!(stats.hp_fraction(), 1.0);
    }

    #[test]
    fn test_damage_calculation() {
        let stats = CombatStats::default_enemy();
        let dmg = stats.calculate_damage(2.0);
        assert_eq!(dmg, 3.0); // 5 - 2
    }

    #[test]
    fn test_damage_minimum() {
        let stats = CombatStats::default_enemy();
        let dmg = stats.calculate_damage(100.0);
        assert_eq!(dmg, 1.0); // minimum 1 damage
    }

    #[test]
    fn test_attack_cooldown() {
        let mut stats = CombatStats::default_enemy();
        stats.attack_timer = 0.0;
        assert!(stats.update_attack(0.016)); // should fire
        assert!(!stats.update_attack(0.016)); // on cooldown
    }

    #[test]
    fn test_player_combat() {
        let mut player = PlayerCombatState::new();
        assert!(player.is_alive());
        player.take_damage(10.0);
        assert!(player.current_hp() < 100.0);
        assert!(player.damage_flash_timer > 0.0);
    }

    #[test]
    fn test_player_minimum_damage() {
        let mut player = PlayerCombatState::new();
        player.stats.defense = 100.0;
        player.take_damage(1.0); // should still take at least 1
        assert_eq!(player.current_hp(), 99.0);
    }

    #[test]
    fn test_player_attack() {
        let mut player = PlayerCombatState::new();
        assert!(!player.is_attacking);
        assert!(player.try_attack());
        assert!(player.is_attacking);
        assert!(!player.try_attack()); // Can't attack again while attacking
    }

    #[test]
    fn test_player_invincibility() {
        let mut player = PlayerCombatState::new();
        let dmg1 = player.take_damage(10.0);
        assert!(dmg1 > 0.0);

        // Immediately try again - should be invincible
        let dmg2 = player.take_damage(10.0);
        assert_eq!(dmg2, 0.0);

        // Wait out invincibility
        let _ = player.update(1.0);
        let dmg3 = player.take_damage(10.0);
        assert!(dmg3 > 0.0);
    }

    #[test]
    fn test_player_respawn() {
        let mut player = PlayerCombatState::new();
        player.take_damage(50.0);
        assert!(player.current_hp() < player.max_hp());

        player.respawn();
        assert_eq!(player.current_hp(), player.max_hp());
        assert!(player.invincibility_timer > 0.0);
    }

    #[test]
    fn test_player_xp() {
        let mut player = PlayerCombatState::new();
        assert_eq!(player.level(), 1);

        // Add enough XP to level up
        let needed = player.xp_to_next_level();
        let levels = player.add_xp(needed);

        assert_eq!(levels, vec![2]);
        assert_eq!(player.level(), 2);
    }

    #[test]
    fn test_dodge() {
        let mut player = PlayerCombatState::new();
        assert!(!player.is_dodging);

        // Should be able to dodge
        assert!(player.try_dodge());
        assert!(player.is_dodging);
        assert!(player.invincibility_timer > 0.0);

        // Can't dodge again while dodging
        assert!(!player.try_dodge());

        // Wait out dodge duration
        let _ = player.update(0.5);
        assert!(!player.is_dodging);

        // Still on cooldown
        assert!(!player.try_dodge());

        // Wait out cooldown
        let _ = player.update(1.5);
        assert!(player.try_dodge());
    }

    #[test]
    fn test_mana_regen_in_update() {
        let mut player = PlayerCombatState::new();
        player.stats.current_mana = 50.0;
        let _ = player.update(1.0);
        // Should have regenerated
        assert!(player.stats.current_mana > 50.0);
    }
}
