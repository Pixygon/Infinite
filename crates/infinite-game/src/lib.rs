//! Infinite Game - Game logic and systems
//!
//! Provides player controllers, camera, input handling, and game logic.

pub mod camera;
pub mod input;
pub mod player;

pub use camera::{CameraConfig, CameraController, CameraMode};
pub use input::{InputAction, InputBindings, InputHandler, InputState};
pub use player::{MovementConfig, PlayerController};
