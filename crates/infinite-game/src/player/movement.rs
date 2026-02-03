//! Movement configuration and constants

use serde::{Deserialize, Serialize};

/// Movement configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovementConfig {
    /// Walking speed in meters per second
    pub walk_speed: f32,
    /// Sprint speed multiplier (applied to walk_speed)
    pub sprint_multiplier: f32,
    /// Ground acceleration (how fast you reach max speed)
    pub ground_acceleration: f32,
    /// Ground deceleration (how fast you stop)
    pub ground_deceleration: f32,
    /// Air acceleration (reduced control in air)
    pub air_acceleration: f32,
    /// Air deceleration
    pub air_deceleration: f32,
    /// Jump initial velocity
    pub jump_velocity: f32,
    /// Gravity multiplier (1.0 = normal gravity)
    pub gravity_scale: f32,
    /// Coyote time - grace period after leaving ground where you can still jump
    pub coyote_time: f32,
    /// Jump buffer - how long a jump input is remembered before landing
    pub jump_buffer: f32,
}

impl Default for MovementConfig {
    fn default() -> Self {
        Self {
            walk_speed: 5.0,
            sprint_multiplier: 1.8,
            ground_acceleration: 50.0,
            ground_deceleration: 30.0,
            air_acceleration: 10.0,
            air_deceleration: 5.0,
            jump_velocity: 8.0,
            gravity_scale: 1.0,
            coyote_time: 0.15,
            jump_buffer: 0.1,
        }
    }
}

impl MovementConfig {
    /// Get the current max speed based on sprint state
    pub fn max_speed(&self, sprinting: bool) -> f32 {
        if sprinting {
            self.walk_speed * self.sprint_multiplier
        } else {
            self.walk_speed
        }
    }

    /// Get the current acceleration based on grounded state
    pub fn acceleration(&self, grounded: bool) -> f32 {
        if grounded {
            self.ground_acceleration
        } else {
            self.air_acceleration
        }
    }

    /// Get the current deceleration based on grounded state
    pub fn deceleration(&self, grounded: bool) -> f32 {
        if grounded {
            self.ground_deceleration
        } else {
            self.air_deceleration
        }
    }
}
