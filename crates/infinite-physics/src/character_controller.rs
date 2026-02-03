//! Character controller using rapier3d's kinematic character controller

use glam::Vec3;
use rapier3d::control::{CharacterAutostep, CharacterLength, KinematicCharacterController};
use rapier3d::prelude::*;

/// Character controller configuration
#[derive(Debug, Clone)]
pub struct CharacterControllerConfig {
    /// Capsule height (default: 1.8m)
    pub height: f32,
    /// Capsule radius (default: 0.4m)
    pub radius: f32,
    /// Maximum slope angle in degrees (default: 45)
    pub max_slope_angle: f32,
    /// Step height for climbing stairs (default: 0.25m)
    pub step_height: f32,
    /// Skin width for collision detection (default: 0.02m)
    pub skin_width: f32,
    /// Whether to snap to ground when walking down slopes
    pub snap_to_ground: bool,
    /// Maximum ground snap distance
    pub ground_snap_distance: f32,
}

impl Default for CharacterControllerConfig {
    fn default() -> Self {
        Self {
            height: 1.8,
            radius: 0.4,
            max_slope_angle: 45.0,
            step_height: 0.25,
            skin_width: 0.02,
            snap_to_ground: true,
            ground_snap_distance: 0.2,
        }
    }
}

/// Character controller for player movement with collision
pub struct CharacterController {
    /// Configuration
    pub config: CharacterControllerConfig,
    /// Current position
    pub position: Vec3,
    /// Current velocity
    pub velocity: Vec3,
    /// Whether the character is on the ground
    pub grounded: bool,
    /// The collider handle for this character
    pub collider_handle: Option<ColliderHandle>,
    /// Rapier's kinematic character controller
    controller: KinematicCharacterController,
}

impl CharacterController {
    /// Create a new character controller with default config
    pub fn new() -> Self {
        Self::with_config(CharacterControllerConfig::default())
    }

    /// Create a new character controller with custom config
    pub fn with_config(config: CharacterControllerConfig) -> Self {
        let mut controller = KinematicCharacterController::default();
        controller.max_slope_climb_angle = config.max_slope_angle.to_radians();
        controller.min_slope_slide_angle = config.max_slope_angle.to_radians();
        controller.autostep = Some(CharacterAutostep {
            max_height: CharacterLength::Absolute(config.step_height),
            min_width: CharacterLength::Relative(0.5),
            include_dynamic_bodies: true,
        });
        controller.snap_to_ground = if config.snap_to_ground {
            Some(CharacterLength::Absolute(config.ground_snap_distance))
        } else {
            None
        };
        controller.offset = CharacterLength::Absolute(config.skin_width);

        Self {
            config,
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            grounded: false,
            collider_handle: None,
            controller,
        }
    }

    /// Spawn the character in the physics world
    pub fn spawn(
        &mut self,
        physics: &mut crate::PhysicsWorld,
        position: Vec3,
    ) -> ColliderHandle {
        self.position = position;

        // Create capsule collider
        let half_height = (self.config.height - 2.0 * self.config.radius) / 2.0;
        let collider = ColliderBuilder::capsule_y(half_height.max(0.01), self.config.radius)
            .translation(vector![position.x, position.y + self.config.height / 2.0, position.z])
            .friction(0.0) // Smooth sliding against walls
            .restitution(0.0)
            .build();

        let handle = physics.add_static_collider(collider);
        self.collider_handle = Some(handle);
        handle
    }

    /// Move the character with collision detection
    pub fn move_character(
        &mut self,
        physics: &mut crate::PhysicsWorld,
        desired_translation: Vec3,
        dt: f32,
    ) {
        let Some(collider_handle) = self.collider_handle else {
            return;
        };

        let Some(collider) = physics.collider_set.get(collider_handle) else {
            return;
        };

        // Get the shape from the collider
        let shape = collider.shape();
        let current_pos = Isometry::translation(
            self.position.x,
            self.position.y + self.config.height / 2.0,
            self.position.z,
        );

        // Compute the corrected movement
        let movement = self.controller.move_shape(
            dt,
            &physics.rigid_body_set,
            &physics.collider_set,
            &physics.query_pipeline,
            shape,
            &current_pos,
            vector![desired_translation.x, desired_translation.y, desired_translation.z],
            QueryFilter::default().exclude_collider(collider_handle),
            |_| {},
        );

        // Update grounded status
        self.grounded = movement.grounded;

        // Apply the corrected translation
        let effective_translation = movement.translation;
        self.position.x += effective_translation.x;
        self.position.y += effective_translation.y;
        self.position.z += effective_translation.z;

        // Update the collider position
        if let Some(collider) = physics.collider_set.get_mut(collider_handle) {
            collider.set_translation(vector![
                self.position.x,
                self.position.y + self.config.height / 2.0,
                self.position.z
            ]);
        }
    }

    /// Apply velocity and move the character
    pub fn update(&mut self, physics: &mut crate::PhysicsWorld, dt: f32) {
        let translation = self.velocity * dt;
        self.move_character(physics, translation, dt);
    }

    /// Set the character's position directly (teleport)
    pub fn set_position(&mut self, physics: &mut crate::PhysicsWorld, position: Vec3) {
        self.position = position;

        if let Some(handle) = self.collider_handle {
            if let Some(collider) = physics.collider_set.get_mut(handle) {
                collider.set_translation(vector![
                    position.x,
                    position.y + self.config.height / 2.0,
                    position.z
                ]);
            }
        }
    }

    /// Get the eye position (top of capsule)
    pub fn eye_position(&self) -> Vec3 {
        Vec3::new(
            self.position.x,
            self.position.y + self.config.height - 0.1, // Slightly below top
            self.position.z,
        )
    }

    /// Get the center position (middle of capsule)
    pub fn center_position(&self) -> Vec3 {
        Vec3::new(
            self.position.x,
            self.position.y + self.config.height / 2.0,
            self.position.z,
        )
    }

    /// Check if standing on ground
    pub fn is_grounded(&self) -> bool {
        self.grounded
    }

    /// Apply an impulse to the character's velocity
    pub fn apply_impulse(&mut self, impulse: Vec3) {
        self.velocity += impulse;
    }

    /// Set the character's velocity directly
    pub fn set_velocity(&mut self, velocity: Vec3) {
        self.velocity = velocity;
    }
}

impl Default for CharacterController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_character_controller_config() {
        let config = CharacterControllerConfig::default();
        assert_eq!(config.height, 1.8);
        assert_eq!(config.radius, 0.4);
        assert_eq!(config.max_slope_angle, 45.0);
    }

    #[test]
    fn test_character_eye_position() {
        let mut controller = CharacterController::new();
        controller.position = Vec3::new(0.0, 0.0, 0.0);
        let eye = controller.eye_position();
        assert!(eye.y > 0.0);
        assert!(eye.y < controller.config.height);
    }
}
