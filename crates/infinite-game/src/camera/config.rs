//! Camera configuration

use serde::{Deserialize, Serialize};

/// Camera configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraConfig {
    /// Minimum camera distance (0 = first person)
    pub min_distance: f32,
    /// Maximum camera distance for third person
    pub max_distance: f32,
    /// Distance threshold below which camera snaps to FPS mode
    pub fps_threshold: f32,
    /// Zoom speed (scroll sensitivity)
    pub zoom_speed: f32,
    /// Zoom interpolation smoothing (0-1, lower = smoother)
    pub zoom_smoothing: f32,
    /// Mouse sensitivity (radians per pixel)
    pub sensitivity: f32,
    /// Minimum pitch angle in degrees
    pub pitch_min: f32,
    /// Maximum pitch angle in degrees
    pub pitch_max: f32,
    /// Collision radius for camera
    pub collision_radius: f32,
    /// Vertical offset from player position (eye height)
    pub eye_height_offset: f32,
    /// Horizontal offset in third person (shoulder view)
    pub shoulder_offset: f32,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            min_distance: 0.0,
            max_distance: 20.0,
            fps_threshold: 0.5,
            zoom_speed: 2.0,
            zoom_smoothing: 0.15,
            sensitivity: 0.003,
            pitch_min: -89.0,
            pitch_max: 89.0,
            collision_radius: 0.3,
            eye_height_offset: 0.0, // Use player's eye position directly
            shoulder_offset: 0.3,
        }
    }
}
