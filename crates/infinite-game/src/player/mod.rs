//! Player controller module
//!
//! Provides first/third-person player movement with physics integration.

mod controller;
mod movement;

pub use controller::PlayerController;
pub use movement::MovementConfig;
