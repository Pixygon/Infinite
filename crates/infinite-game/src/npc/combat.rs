//! NPC combat system — stats, damage calculation, and aggro

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

/// Player combat state (placeholder for Milestone 4)
#[derive(Debug, Clone)]
pub struct PlayerCombatState {
    pub max_hp: f32,
    pub current_hp: f32,
    pub defense: f32,
    pub damage_flash_timer: f32,
    pub last_damage_amount: f32,
}

impl PlayerCombatState {
    pub fn new() -> Self {
        Self {
            max_hp: 100.0,
            current_hp: 100.0,
            defense: 3.0,
            damage_flash_timer: 0.0,
            last_damage_amount: 0.0,
        }
    }

    pub fn hp_fraction(&self) -> f32 {
        if self.max_hp <= 0.0 { return 0.0; }
        self.current_hp / self.max_hp
    }

    pub fn take_damage(&mut self, damage: f32) {
        let actual = (damage - self.defense).max(1.0);
        self.current_hp = (self.current_hp - actual).max(0.0);
        self.damage_flash_timer = 0.3;
        self.last_damage_amount = actual;
    }

    pub fn is_alive(&self) -> bool {
        self.current_hp > 0.0
    }

    pub fn update(&mut self, delta: f32) {
        if self.damage_flash_timer > 0.0 {
            self.damage_flash_timer = (self.damage_flash_timer - delta).max(0.0);
        }
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
        assert!(player.current_hp < 100.0);
        assert!(player.damage_flash_timer > 0.0);
    }

    #[test]
    fn test_player_minimum_damage() {
        let mut player = PlayerCombatState::new();
        player.defense = 100.0;
        player.take_damage(1.0); // should still take at least 1
        assert_eq!(player.current_hp, 99.0);
    }
}
