//! Scene rendering coordination

use glam::{Mat4, Vec3};

/// Scene-wide uniforms for rendering
#[derive(Clone, Copy, Debug)]
pub struct SceneUniforms {
    /// View matrix (camera)
    pub view: Mat4,
    /// Projection matrix
    pub projection: Mat4,
    /// Sun/light direction (normalized)
    pub sun_direction: Vec3,
    /// Sun light intensity (0.0 - 1.0)
    pub sun_intensity: f32,
    /// Sun light color
    pub sun_color: Vec3,
    /// Ambient light intensity
    pub ambient_intensity: f32,
}

impl Default for SceneUniforms {
    fn default() -> Self {
        Self {
            view: Mat4::IDENTITY,
            projection: Mat4::IDENTITY,
            sun_direction: Vec3::new(0.5, 0.8, 0.3).normalize(),
            sun_intensity: 1.0,
            sun_color: Vec3::new(1.0, 0.95, 0.85),
            ambient_intensity: 0.3,
        }
    }
}

/// Push constants for basic 3D rendering
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BasicPushConstants {
    pub model: [[f32; 4]; 4],
    pub view: [[f32; 4]; 4],
    pub projection: [[f32; 4]; 4],
    pub sun_direction: [f32; 4], // xyz = direction, w = intensity
    pub sun_color: [f32; 4],     // xyz = color, w = ambient intensity
}

impl BasicPushConstants {
    pub fn new(
        model: Mat4,
        view: Mat4,
        projection: Mat4,
        sun_direction: Vec3,
        sun_intensity: f32,
        sun_color: Vec3,
        ambient_intensity: f32,
    ) -> Self {
        Self {
            model: model.to_cols_array_2d(),
            view: view.to_cols_array_2d(),
            projection: projection.to_cols_array_2d(),
            sun_direction: [sun_direction.x, sun_direction.y, sun_direction.z, sun_intensity],
            sun_color: [sun_color.x, sun_color.y, sun_color.z, ambient_intensity],
        }
    }

    pub fn from_uniforms(model: Mat4, uniforms: &SceneUniforms) -> Self {
        Self::new(
            model,
            uniforms.view,
            uniforms.projection,
            uniforms.sun_direction,
            uniforms.sun_intensity,
            uniforms.sun_color,
            uniforms.ambient_intensity,
        )
    }
}

/// Sky colors for procedural sky rendering
#[derive(Clone, Copy, Debug)]
pub struct SkyColors {
    /// Color at the top of the sky
    pub zenith: Vec3,
    /// Color at the horizon
    pub horizon: Vec3,
    /// Sun glow intensity
    pub sun_glow: f32,
    /// Sun disk size (0.0 - 0.1)
    pub sun_size: f32,
}

impl Default for SkyColors {
    fn default() -> Self {
        Self {
            zenith: Vec3::new(0.1, 0.2, 0.5),
            horizon: Vec3::new(0.5, 0.6, 0.7),
            sun_glow: 0.5,
            sun_size: 0.02,
        }
    }
}

/// Push constants for sky dome rendering
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SkyPushConstants {
    pub view: [[f32; 4]; 4],
    pub projection: [[f32; 4]; 4],
    pub sun_direction: [f32; 4], // xyz = direction, w = intensity (for stars)
    pub sky_zenith: [f32; 4],    // rgb = color
    pub sky_horizon: [f32; 4],   // rgb = color
    pub sun_params: [f32; 4],    // x = size, y = glow, z = time_of_day
}

impl SkyPushConstants {
    pub fn new(
        view: Mat4,
        projection: Mat4,
        sun_direction: Vec3,
        sun_intensity: f32,
        colors: &SkyColors,
        time_of_day: f32,
    ) -> Self {
        Self {
            view: view.to_cols_array_2d(),
            projection: projection.to_cols_array_2d(),
            sun_direction: [sun_direction.x, sun_direction.y, sun_direction.z, sun_intensity],
            sky_zenith: [colors.zenith.x, colors.zenith.y, colors.zenith.z, 0.0],
            sky_horizon: [colors.horizon.x, colors.horizon.y, colors.horizon.z, 0.0],
            sun_params: [colors.sun_size, colors.sun_glow, time_of_day, 0.0],
        }
    }
}
