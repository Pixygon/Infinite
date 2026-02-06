//! Player controller module
//!
//! Provides first/third-person player movement with physics integration.

mod controller;
mod movement;
pub mod stats;

pub use controller::PlayerController;
pub use movement::MovementConfig;
pub use stats::{CharacterStats, EnemyType, PlayerProgression, StatGrowth};

// Re-export combat types for convenience
pub use crate::combat;
