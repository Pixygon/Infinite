//! Player controller with WASD movement and physics

use glam::Vec3;
use infinite_physics::{CharacterController, PhysicsWorld};

use crate::input::{InputAction, InputState};

use super::MovementConfig;

/// Player controller handling input, movement, and physics
pub struct PlayerController {
    /// Movement configuration
    pub config: MovementConfig,
    /// Physics character controller
    pub character: CharacterController,
    /// Horizontal velocity (X, Z only - Y is handled by physics)
    horizontal_velocity: Vec3,
    /// Vertical velocity (jumping/falling)
    vertical_velocity: f32,
    /// Time since last grounded (for coyote time)
    time_since_grounded: f32,
    /// Time since jump was pressed (for jump buffering)
    time_since_jump_pressed: f32,
    /// Whether jump input is buffered
    jump_buffered: bool,
    /// Whether we were grounded last frame
    was_grounded: bool,
}

impl PlayerController {
    /// Create a new player controller
    pub fn new() -> Self {
        Self {
            config: MovementConfig::default(),
            character: CharacterController::new(),
            horizontal_velocity: Vec3::ZERO,
            vertical_velocity: 0.0,
            time_since_grounded: 0.0,
            time_since_jump_pressed: f32::MAX,
            jump_buffered: false,
            was_grounded: false,
        }
    }

    /// Create a player controller with custom config
    pub fn with_config(config: MovementConfig) -> Self {
        Self {
            config,
            ..Self::new()
        }
    }

    /// Spawn the player in the world at a position
    pub fn spawn(&mut self, physics: &mut PhysicsWorld, position: Vec3) {
        self.character.spawn(physics, position);
        self.horizontal_velocity = Vec3::ZERO;
        self.vertical_velocity = 0.0;
        self.time_since_grounded = 0.0;
    }

    /// Get the player's current position
    pub fn position(&self) -> Vec3 {
        self.character.position
    }

    /// Get the player's eye position (for camera)
    pub fn eye_position(&self) -> Vec3 {
        self.character.eye_position()
    }

    /// Check if the player is grounded
    pub fn is_grounded(&self) -> bool {
        self.character.is_grounded()
    }

    /// Check if the player can jump (grounded or within coyote time)
    pub fn can_jump(&self) -> bool {
        self.is_grounded() || self.time_since_grounded < self.config.coyote_time
    }

    /// Update the player (fixed timestep)
    pub fn fixed_update(
        &mut self,
        physics: &mut PhysicsWorld,
        input: &InputState,
        camera_yaw: f32,
        dt: f32,
    ) {
        let grounded = self.character.is_grounded();

        // Track coyote time
        if grounded {
            self.time_since_grounded = 0.0;
        } else {
            self.time_since_grounded += dt;
        }

        // Handle jump buffering
        if input.is_just_pressed(InputAction::Jump) {
            self.jump_buffered = true;
            self.time_since_jump_pressed = 0.0;
        }
        self.time_since_jump_pressed += dt;
        if self.time_since_jump_pressed > self.config.jump_buffer {
            self.jump_buffered = false;
        }

        // Calculate movement direction from input
        let mut move_dir = Vec3::ZERO;
        if input.is_held(InputAction::MoveForward) {
            move_dir.z -= 1.0;
        }
        if input.is_held(InputAction::MoveBackward) {
            move_dir.z += 1.0;
        }
        if input.is_held(InputAction::MoveLeft) {
            move_dir.x -= 1.0;
        }
        if input.is_held(InputAction::MoveRight) {
            move_dir.x += 1.0;
        }

        // Rotate movement by camera yaw
        if move_dir.length_squared() > 0.0 {
            move_dir = move_dir.normalize();

            let cos_yaw = camera_yaw.cos();
            let sin_yaw = camera_yaw.sin();

            let rotated = Vec3::new(
                move_dir.x * cos_yaw - move_dir.z * sin_yaw,
                0.0,
                move_dir.x * sin_yaw + move_dir.z * cos_yaw,
            );
            move_dir = rotated;
        }

        // Check sprint
        let sprinting = input.is_held(InputAction::Sprint);
        let max_speed = self.config.max_speed(sprinting);

        // Apply horizontal movement with acceleration
        if move_dir.length_squared() > 0.0 {
            let target_velocity = move_dir * max_speed;
            let accel = self.config.acceleration(grounded);

            // Accelerate towards target velocity
            self.horizontal_velocity = Self::move_towards_vec3(
                self.horizontal_velocity,
                target_velocity,
                accel * dt,
            );
        } else {
            // Decelerate when no input
            let decel = self.config.deceleration(grounded);
            self.horizontal_velocity = Self::move_towards_vec3(
                self.horizontal_velocity,
                Vec3::ZERO,
                decel * dt,
            );
        }

        // Handle jumping
        let can_jump = self.can_jump();
        if self.jump_buffered && can_jump {
            self.vertical_velocity = self.config.jump_velocity;
            self.jump_buffered = false;
            self.time_since_grounded = self.config.coyote_time; // Consume coyote time
        }

        // Apply gravity
        if !grounded {
            let gravity = 9.81 * self.config.gravity_scale;
            self.vertical_velocity -= gravity * dt;
        } else if self.vertical_velocity < 0.0 {
            // Reset vertical velocity when landing
            self.vertical_velocity = 0.0;
        }

        // Combine velocities and move
        let total_velocity = Vec3::new(
            self.horizontal_velocity.x,
            self.vertical_velocity,
            self.horizontal_velocity.z,
        );

        self.character.velocity = total_velocity;
        self.character.update(physics, dt);

        // Track grounded state change
        self.was_grounded = grounded;
    }

    /// Move a vector towards a target by a maximum delta
    fn move_towards_vec3(current: Vec3, target: Vec3, max_delta: f32) -> Vec3 {
        let diff = target - current;
        let distance = diff.length();

        if distance <= max_delta || distance == 0.0 {
            target
        } else {
            current + diff / distance * max_delta
        }
    }

    /// Teleport the player to a position
    pub fn teleport(&mut self, physics: &mut PhysicsWorld, position: Vec3) {
        self.character.set_position(physics, position);
        self.horizontal_velocity = Vec3::ZERO;
        self.vertical_velocity = 0.0;
    }
}

impl Default for PlayerController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_controller_creation() {
        let player = PlayerController::new();
        assert_eq!(player.position(), Vec3::ZERO);
    }

    #[test]
    fn test_move_towards() {
        let result = PlayerController::move_towards_vec3(
            Vec3::ZERO,
            Vec3::new(10.0, 0.0, 0.0),
            5.0,
        );
        assert!((result.x - 5.0).abs() < 0.001);
    }
}
