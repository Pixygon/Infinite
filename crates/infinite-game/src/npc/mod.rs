//! NPC system — data structures, manager, spawning, dialogue, and combat

pub mod ai_dialogue;
pub mod archetype_mapping;
pub mod character_cache;
pub mod combat;
pub mod dialogue;
pub mod game_context;
pub mod goap;
pub mod manager;
pub mod npc_generator;
pub mod relationship;
pub mod spawn;

use glam::Vec3;
use serde::{Deserialize, Serialize};

/// Unique identifier for an NPC instance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NpcId(pub u64);

/// What the NPC does in the world
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NpcRole {
    Villager,
    Guard,
    Shopkeeper,
    QuestGiver,
    Enemy,
}

impl NpcRole {
    /// Default capsule color for this role
    pub fn color(&self) -> [f32; 4] {
        match self {
            NpcRole::Villager => [0.4, 0.7, 0.3, 1.0],    // green
            NpcRole::Guard => [0.3, 0.3, 0.8, 1.0],       // blue
            NpcRole::Shopkeeper => [0.8, 0.7, 0.2, 1.0],  // gold
            NpcRole::QuestGiver => [0.7, 0.3, 0.8, 1.0],  // purple
            NpcRole::Enemy => [0.8, 0.2, 0.2, 1.0],       // red
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            NpcRole::Villager => "Villager",
            NpcRole::Guard => "Guard",
            NpcRole::Shopkeeper => "Shopkeeper",
            NpcRole::QuestGiver => "Quest Giver",
            NpcRole::Enemy => "Enemy",
        }
    }
}

/// Relationship grouping
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NpcFaction {
    Friendly,
    Neutral,
    Hostile,
}

/// Static definition of an NPC type
#[derive(Debug, Clone)]
pub struct NpcData {
    pub name: String,
    pub role: NpcRole,
    pub faction: NpcFaction,
    pub home_position: Vec3,
    pub wander_radius: f32,
    pub interaction_radius: f32,
    pub color: [f32; 4],
    /// Linked PixygonServer character ID (if any)
    pub server_character_id: Option<String>,
}

/// Simple behavior state (used before GOAP takes over)
#[derive(Debug, Clone)]
pub enum NpcBehaviorState {
    Idle { timer: f32 },
    Walking { target: Vec3 },
    Talking,
}

/// A living NPC in the world
pub struct NpcInstance {
    pub id: NpcId,
    pub data: NpcData,
    pub position: Vec3,
    pub velocity: Vec3,
    pub yaw: f32,
    pub chunk: infinite_world::ChunkCoord,
    pub state: NpcBehaviorState,
    pub brain: Option<goap::NpcBrain>,
    /// Deterministic key for persistence (hash of chunk coords + spawn index)
    pub persistent_key: u64,
}

impl NpcInstance {
    /// Get the display name
    pub fn name(&self) -> &str {
        &self.data.name
    }

    /// Whether this NPC can be interacted with
    pub fn is_interactable(&self) -> bool {
        self.data.faction != NpcFaction::Hostile || self.data.role != NpcRole::Enemy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_npc_id_equality() {
        assert_eq!(NpcId(1), NpcId(1));
        assert_ne!(NpcId(1), NpcId(2));
    }

    #[test]
    fn test_role_colors_are_opaque() {
        for role in [NpcRole::Villager, NpcRole::Guard, NpcRole::Shopkeeper, NpcRole::QuestGiver, NpcRole::Enemy] {
            let c = role.color();
            assert_eq!(c[3], 1.0, "{:?} should have alpha 1.0", role);
        }
    }
}
