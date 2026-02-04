//! Infinite Game - Game logic and systems
//!
//! Provides player controllers, camera, input handling, and game logic.

pub mod camera;
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
pub use player::{MovementConfig, PlayerController};
