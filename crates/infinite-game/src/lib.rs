//! Infinite Game - Game logic and systems
//!
//! Provides player controllers, camera, input handling, and game logic.

pub mod camera;
pub mod combat;
pub mod input;
pub mod interaction;
pub mod npc;
pub mod player;

pub use camera::{CameraConfig, CameraController, CameraMode};
pub use input::{InputAction, InputBindings, InputHandler, InputState};
pub use interaction::{
    Interactable, InteractableId, InteractableKind, InteractableState, InteractionResult,
    InteractionSaveData, InteractionSystem,
};
pub use npc::{NpcFaction, NpcId, NpcRole};
pub use npc::manager::DamageNpcResult;
pub use npc::ai_dialogue::AiDialogueManager;
pub use npc::character_cache::NpcCharacterCache;
pub use npc::game_context::GameContext;
pub use npc::relationship::{RelationshipManager, RelationshipSaveData};
pub use player::{CharacterStats, EnemyType, MovementConfig, PlayerController, PlayerProgression, StatGrowth};

// Combat system re-exports
pub use combat::{
    AttackType, DamageEvent, Element, EquipmentSet, EquipmentSlot, Gem, GemQuality, GemShape,
    Inventory, Item, ItemCategory, ItemId, ItemRarity, MAX_INVENTORY_SIZE, Rune, RuneComposer,
    Skill, SkillId, SkillSlot, StatModifiers, StatusEffect, StatusEffectType, StatusManager,
    WeaponData, WeaponType,
};
