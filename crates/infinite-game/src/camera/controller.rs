//! Camera controller with mouse look and zoom

use glam::{Mat4, Quat, Vec2, Vec3};
use infinite_physics::PhysicsWorld;
use rapier3d::prelude::QueryFilter;

use crate::input::InputState;

use super::CameraConfig;

/// Camera mode (first-person or third-person)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CameraMode {
    /// First-person view (camera at eye position)
    FirstPerson,
    /// Third-person view (camera orbiting at distance)
    ThirdPerson { distance: f32 },
}

impl CameraMode {
    /// Get the camera distance (0 for first-person)
    pub fn distance(&self) -> f32 {
        match self {
            CameraMode::FirstPerson => 0.0,
            CameraMode::ThirdPerson { distance } => *distance,
        }
    }

    /// Check if in first-person mode
    pub fn is_first_person(&self) -> bool {
        matches!(self, CameraMode::FirstPerson)
    }
}

/// Camera controller
pub struct CameraController {
    /// Configuration
    pub config: CameraConfig,
    /// Current camera mode
    pub mode: CameraMode,
    /// Yaw rotation in radians (horizontal)
    pub yaw: f32,
    /// Pitch rotation in radians (vertical)
    pub pitch: f32,
    /// Target zoom distance (for smooth interpolation)
    target_distance: f32,
    /// Current interpolated zoom distance
    current_distance: f32,
    /// Camera world position (computed each frame)
    position: Vec3,
    /// Target position we're looking at
    target: Vec3,
}

impl CameraController {
    /// Create a new camera controller
    pub fn new() -> Self {
        Self::with_config(CameraConfig::default())
    }

    /// Create a camera controller with custom config
    pub fn with_config(config: CameraConfig) -> Self {
        let default_distance = 5.0;
        Self {
            config,
            mode: CameraMode::ThirdPerson {
                distance: default_distance,
            },
            yaw: 0.0,
            pitch: 0.0,
            target_distance: default_distance,
            current_distance: default_distance,
            position: Vec3::ZERO,
            target: Vec3::ZERO,
        }
    }

    /// Get the camera's current world position
    pub fn position(&self) -> Vec3 {
        self.position
    }

    /// Get the point the camera is looking at
    pub fn target(&self) -> Vec3 {
        self.target
    }

    /// Get the camera's forward direction
    pub fn forward(&self) -> Vec3 {
        let cos_pitch = self.pitch.cos();
        Vec3::new(
            self.yaw.sin() * cos_pitch,
            self.pitch.sin(),
            -self.yaw.cos() * cos_pitch,
        )
    }

    /// Get the camera's right direction
    pub fn right(&self) -> Vec3 {
        Vec3::new(self.yaw.cos(), 0.0, self.yaw.sin())
    }

    /// Get the camera's up direction
    pub fn up(&self) -> Vec3 {
        self.right().cross(self.forward()).normalize()
    }

    /// Get the view matrix
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.target, Vec3::Y)
    }

    /// Get a projection matrix
    pub fn projection_matrix(&self, aspect_ratio: f32, fov_degrees: f32) -> Mat4 {
        let fov_radians = fov_degrees.to_radians();
        Mat4::perspective_rh(fov_radians, aspect_ratio, 0.1, 1000.0)
    }

    /// Get the rotation quaternion
    pub fn rotation(&self) -> Quat {
        Quat::from_euler(glam::EulerRot::YXZ, self.yaw, self.pitch, 0.0)
    }

    /// Handle mouse look input
    pub fn handle_mouse_look(&mut self, mouse_delta: Vec2) {
        // Apply yaw (horizontal)
        self.yaw += mouse_delta.x * self.config.sensitivity;

        // Apply pitch (vertical) with clamping
        self.pitch -= mouse_delta.y * self.config.sensitivity;
        let pitch_min = self.config.pitch_min.to_radians();
        let pitch_max = self.config.pitch_max.to_radians();
        self.pitch = self.pitch.clamp(pitch_min, pitch_max);
    }

    /// Handle scroll wheel zoom
    pub fn handle_zoom(&mut self, scroll_delta: f32) {
        // Adjust target distance
        self.target_distance -= scroll_delta * self.config.zoom_speed;
        self.target_distance = self
            .target_distance
            .clamp(self.config.min_distance, self.config.max_distance);
    }

    /// Update the camera (call each frame)
    pub fn update(
        &mut self,
        input: &InputState,
        player_eye_position: Vec3,
        physics: Option<&PhysicsWorld>,
        dt: f32,
    ) {
        // Handle mouse look
        if input.cursor_captured {
            self.handle_mouse_look(input.mouse_delta);
        }

        // Handle zoom
        if input.scroll_delta.abs() > 0.0 {
            self.handle_zoom(input.scroll_delta);
        }

        // Smooth zoom interpolation
        let zoom_lerp = 1.0 - (1.0 - self.config.zoom_smoothing).powf(dt * 60.0);
        self.current_distance =
            self.current_distance + (self.target_distance - self.current_distance) * zoom_lerp;

        // Snap to first-person if below threshold
        if self.current_distance < self.config.fps_threshold {
            self.mode = CameraMode::FirstPerson;
            self.current_distance = 0.0;
            self.target_distance = 0.0;
        } else {
            self.mode = CameraMode::ThirdPerson {
                distance: self.current_distance,
            };
        }

        // Calculate target position (where we look at)
        self.target = player_eye_position;

        // Calculate camera position based on mode
        match self.mode {
            CameraMode::FirstPerson => {
                self.position = player_eye_position;
            }
            CameraMode::ThirdPerson { distance } => {
                // Calculate offset direction (opposite of look direction)
                let offset_dir = Vec3::new(
                    -self.yaw.sin() * self.pitch.cos(),
                    -self.pitch.sin(),
                    self.yaw.cos() * self.pitch.cos(),
                );

                // Add shoulder offset for over-the-shoulder view
                let shoulder = self.right() * self.config.shoulder_offset;

                // Calculate ideal camera position
                let ideal_position = player_eye_position + shoulder + offset_dir * distance;

                // Check for collision with world geometry
                if let Some(physics) = physics {
                    let ray_start = player_eye_position + shoulder;
                    let ray_dir = (ideal_position - ray_start).normalize();
                    let ray_length = distance + self.config.collision_radius;

                    if let Some((_handle, toi)) = physics.raycast(
                        ray_start,
                        ray_dir,
                        ray_length,
                        QueryFilter::default(),
                    ) {
                        // Camera would clip - move it closer
                        let safe_distance = (toi - self.config.collision_radius).max(0.5);
                        self.position = ray_start + ray_dir * safe_distance;
                    } else {
                        self.position = ideal_position;
                    }
                } else {
                    self.position = ideal_position;
                }
            }
        }
    }

    /// Set the camera yaw directly
    pub fn set_yaw(&mut self, yaw: f32) {
        self.yaw = yaw;
    }

    /// Set the camera pitch directly
    pub fn set_pitch(&mut self, pitch: f32) {
        let pitch_min = self.config.pitch_min.to_radians();
        let pitch_max = self.config.pitch_max.to_radians();
        self.pitch = pitch.clamp(pitch_min, pitch_max);
    }

    /// Set the zoom distance directly (for settings/init)
    pub fn set_distance(&mut self, distance: f32) {
        let clamped = distance.clamp(self.config.min_distance, self.config.max_distance);
        self.target_distance = clamped;
        self.current_distance = clamped;

        if clamped < self.config.fps_threshold {
            self.mode = CameraMode::FirstPerson;
        } else {
            self.mode = CameraMode::ThirdPerson { distance: clamped };
        }
    }

    /// Toggle between first and third person
    pub fn toggle_perspective(&mut self) {
        match self.mode {
            CameraMode::FirstPerson => {
                self.set_distance(5.0); // Default third-person distance
            }
            CameraMode::ThirdPerson { .. } => {
                self.set_distance(0.0);
            }
        }
    }
}

impl Default for CameraController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_controller_creation() {
        let camera = CameraController::new();
        assert_eq!(camera.yaw, 0.0);
        assert_eq!(camera.pitch, 0.0);
    }

    #[test]
    fn test_camera_zoom() {
        let mut camera = CameraController::new();
        camera.set_distance(10.0);
        assert_eq!(camera.current_distance, 10.0);
        assert!(matches!(camera.mode, CameraMode::ThirdPerson { .. }));

        camera.set_distance(0.0);
        assert_eq!(camera.current_distance, 0.0);
        assert!(camera.mode.is_first_person());
    }

    #[test]
    fn test_camera_pitch_clamping() {
        let mut camera = CameraController::new();
        camera.set_pitch(100.0_f32.to_radians()); // Over max
        assert!(camera.pitch <= camera.config.pitch_max.to_radians() + 0.01);

        camera.set_pitch(-100.0_f32.to_radians()); // Under min
        assert!(camera.pitch >= camera.config.pitch_min.to_radians() - 0.01);
    }
}
