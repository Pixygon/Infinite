//! Core types used throughout the Infinite engine

use glam::{Mat4, Quat, Vec3};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for entities in the ECS
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(pub Uuid);

impl EntityId {
    /// Create a new random entity ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create an entity ID from a UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Default for EntityId {
    fn default() -> Self {
        Self::new()
    }
}

/// Transform component representing position, rotation, and scale
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl Transform {
    /// Create a new transform at the given position
    pub fn from_position(position: Vec3) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }

    /// Create a new transform with position and rotation
    pub fn from_position_rotation(position: Vec3, rotation: Quat) -> Self {
        Self {
            position,
            rotation,
            scale: Vec3::ONE,
        }
    }

    /// Compute the model matrix for this transform
    pub fn matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }

    /// Get the forward direction (negative Z in local space)
    pub fn forward(&self) -> Vec3 {
        self.rotation * -Vec3::Z
    }

    /// Get the right direction (positive X in local space)
    pub fn right(&self) -> Vec3 {
        self.rotation * Vec3::X
    }

    /// Get the up direction (positive Y in local space)
    pub fn up(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }

    /// Translate by the given offset
    pub fn translate(&mut self, offset: Vec3) {
        self.position += offset;
    }

    /// Rotate by the given quaternion
    pub fn rotate(&mut self, rotation: Quat) {
        self.rotation = rotation * self.rotation;
    }

    /// Look at a target position
    pub fn look_at(&mut self, target: Vec3, up: Vec3) {
        let forward = (target - self.position).normalize();
        let right = up.cross(forward).normalize();
        let up = forward.cross(right);

        self.rotation = Quat::from_mat4(&Mat4::from_cols(
            right.extend(0.0),
            up.extend(0.0),
            forward.extend(0.0),
            Vec3::ZERO.extend(1.0),
        ));
    }

    /// Interpolate between two transforms
    pub fn lerp(a: &Transform, b: &Transform, t: f32) -> Transform {
        Transform {
            position: a.position.lerp(b.position, t),
            rotation: a.rotation.slerp(b.rotation, t),
            scale: a.scale.lerp(b.scale, t),
        }
    }
}

/// RGBA color with floating point components (0.0 to 1.0)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const WHITE: Color = Color::rgb(1.0, 1.0, 1.0);
    pub const BLACK: Color = Color::rgb(0.0, 0.0, 0.0);
    pub const RED: Color = Color::rgb(1.0, 0.0, 0.0);
    pub const GREEN: Color = Color::rgb(0.0, 1.0, 0.0);
    pub const BLUE: Color = Color::rgb(0.0, 0.0, 1.0);
    pub const TRANSPARENT: Color = Color::rgba(0.0, 0.0, 0.0, 0.0);

    /// Create a color from RGB values (alpha = 1.0)
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    /// Create a color from RGBA values
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Create a color from a hex value (0xRRGGBB)
    pub fn from_hex(hex: u32) -> Self {
        let r = ((hex >> 16) & 0xFF) as f32 / 255.0;
        let g = ((hex >> 8) & 0xFF) as f32 / 255.0;
        let b = (hex & 0xFF) as f32 / 255.0;
        Self::rgb(r, g, b)
    }

    /// Create a color from a hex value with alpha (0xRRGGBBAA)
    pub fn from_hex_alpha(hex: u32) -> Self {
        let r = ((hex >> 24) & 0xFF) as f32 / 255.0;
        let g = ((hex >> 16) & 0xFF) as f32 / 255.0;
        let b = ((hex >> 8) & 0xFF) as f32 / 255.0;
        let a = (hex & 0xFF) as f32 / 255.0;
        Self::rgba(r, g, b, a)
    }

    /// Convert to an array [r, g, b, a]
    pub fn to_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// Linear interpolation between two colors
    pub fn lerp(a: &Color, b: &Color, t: f32) -> Color {
        Color {
            r: a.r + (b.r - a.r) * t,
            g: a.g + (b.g - a.g) * t,
            b: a.b + (b.b - a.b) * t,
            a: a.a + (b.a - a.a) * t,
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::WHITE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_matrix() {
        let transform = Transform::from_position(Vec3::new(1.0, 2.0, 3.0));
        let matrix = transform.matrix();
        let translation = matrix.col(3).truncate();
        assert_eq!(translation, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_color_from_hex() {
        let color = Color::from_hex(0xFF8000);
        assert!((color.r - 1.0).abs() < 0.01);
        assert!((color.g - 0.5).abs() < 0.01);
        assert!((color.b - 0.0).abs() < 0.01);
    }
}
