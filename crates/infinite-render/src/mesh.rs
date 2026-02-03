//! Mesh generation utilities

use crate::vertex::{SkyVertex, Vertex3D};
use glam::Vec3;
use std::f32::consts::PI;

/// Generated mesh data
#[derive(Clone, Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex3D>,
    pub indices: Vec<u32>,
}

impl Mesh {
    /// Create an empty mesh
    pub fn empty() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    /// Generate a capsule mesh (cylindrical body with hemispherical caps)
    pub fn capsule(height: f32, radius: f32, segments: u32, rings: u32, color: [f32; 4]) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let half_height = (height - 2.0 * radius).max(0.0) / 2.0;

        // Top hemisphere
        for ring in 0..=rings / 2 {
            let phi = PI * 0.5 * (1.0 - ring as f32 / (rings / 2) as f32);
            let y = phi.sin() * radius + half_height;
            let ring_radius = phi.cos() * radius;

            for seg in 0..=segments {
                let theta = 2.0 * PI * seg as f32 / segments as f32;
                let x = ring_radius * theta.cos();
                let z = ring_radius * theta.sin();

                let normal = Vec3::new(x, phi.sin() * radius, z).normalize();

                vertices.push(Vertex3D::new(
                    [x, y, z],
                    [normal.x, normal.y, normal.z],
                    color,
                ));
            }
        }

        // Cylinder body
        for ring in 0..=1 {
            let y = half_height - ring as f32 * 2.0 * half_height;

            for seg in 0..=segments {
                let theta = 2.0 * PI * seg as f32 / segments as f32;
                let x = radius * theta.cos();
                let z = radius * theta.sin();

                let normal = Vec3::new(x, 0.0, z).normalize();

                vertices.push(Vertex3D::new(
                    [x, y, z],
                    [normal.x, normal.y, normal.z],
                    color,
                ));
            }
        }

        // Bottom hemisphere
        for ring in 0..=rings / 2 {
            let phi = -PI * 0.5 * ring as f32 / (rings / 2) as f32;
            let y = phi.sin() * radius - half_height;
            let ring_radius = phi.cos() * radius;

            for seg in 0..=segments {
                let theta = 2.0 * PI * seg as f32 / segments as f32;
                let x = ring_radius * theta.cos();
                let z = ring_radius * theta.sin();

                let normal = Vec3::new(x, phi.sin() * radius, z).normalize();

                vertices.push(Vertex3D::new(
                    [x, y, z],
                    [normal.x, normal.y, normal.z],
                    color,
                ));
            }
        }

        // Generate indices
        let total_rings = rings / 2 + 2 + rings / 2;
        for ring in 0..total_rings {
            for seg in 0..segments {
                let current = ring * (segments + 1) + seg;
                let next = current + segments + 1;

                indices.push(current);
                indices.push(next);
                indices.push(current + 1);

                indices.push(current + 1);
                indices.push(next);
                indices.push(next + 1);
            }
        }

        Self { vertices, indices }
    }

    /// Generate a capsule mesh shaped by character appearance parameters.
    ///
    /// Body params: height (0-1), build (0-1), shoulder_width (0-1), hip_width (0-1)
    /// Skin params: tone (0-1, light to dark), undertone (0-1, cool to warm)
    pub fn character_capsule(
        height: f32,
        build: f32,
        shoulder_width: f32,
        hip_width: f32,
        skin_tone: f32,
        skin_undertone: f32,
    ) -> Self {
        // Calculate capsule dimensions from appearance
        let base_height = 1.8;
        let height_scale = 0.8 + height * 0.4; // 0.8 to 1.2
        let actual_height = base_height * height_scale;

        let base_radius = 0.4;
        let build_scale = 0.7 + build * 0.6; // 0.7 to 1.3
        let actual_radius = base_radius * build_scale;

        // Calculate skin color from tone and undertone
        let base_r = 0.95 - skin_tone * 0.5;
        let base_g = 0.85 - skin_tone * 0.4;
        let base_b = 0.75 - skin_tone * 0.35;

        let r = (base_r + (skin_undertone - 0.5) * 0.1).clamp(0.1, 1.0);
        let g = base_g.clamp(0.1, 1.0);
        let b = (base_b - (skin_undertone - 0.5) * 0.1).clamp(0.1, 1.0);

        let color = [r, g, b, 1.0];

        // Generate base capsule
        let segments = 16u32;
        let rings = 16u32;
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let half_height = (actual_height - 2.0 * actual_radius).max(0.0) / 2.0;

        // Top hemisphere
        for ring in 0..=rings / 2 {
            let phi = PI * 0.5 * (1.0 - ring as f32 / (rings / 2) as f32);
            let y = phi.sin() * actual_radius + half_height;
            let ring_radius = phi.cos() * actual_radius;

            for seg in 0..=segments {
                let theta = 2.0 * PI * seg as f32 / segments as f32;
                let x = ring_radius * theta.cos();
                let z = ring_radius * theta.sin();

                // Apply shoulder/hip width variation based on Y position
                let normalized_y = (y + half_height) / (2.0 * half_height + 2.0 * actual_radius);
                let width_blend = smoothstep(0.3, 0.7, normalized_y);
                let width_factor = 0.8 + lerp(hip_width, shoulder_width, width_blend) * 0.4;

                let final_x = x * width_factor;
                let final_z = z * width_factor;

                let normal = Vec3::new(final_x, phi.sin() * actual_radius, final_z).normalize();

                vertices.push(Vertex3D::new(
                    [final_x, y, final_z],
                    [normal.x, normal.y, normal.z],
                    color,
                ));
            }
        }

        // Cylinder body
        for ring in 0..=1 {
            let y = half_height - ring as f32 * 2.0 * half_height;

            for seg in 0..=segments {
                let theta = 2.0 * PI * seg as f32 / segments as f32;
                let x = actual_radius * theta.cos();
                let z = actual_radius * theta.sin();

                let normalized_y = (y + half_height) / (2.0 * half_height + 2.0 * actual_radius);
                let width_blend = smoothstep(0.3, 0.7, normalized_y);
                let width_factor = 0.8 + lerp(hip_width, shoulder_width, width_blend) * 0.4;

                let final_x = x * width_factor;
                let final_z = z * width_factor;

                let normal = Vec3::new(final_x, 0.0, final_z).normalize();

                vertices.push(Vertex3D::new(
                    [final_x, y, final_z],
                    [normal.x, normal.y, normal.z],
                    color,
                ));
            }
        }

        // Bottom hemisphere
        for ring in 0..=rings / 2 {
            let phi = -PI * 0.5 * ring as f32 / (rings / 2) as f32;
            let y = phi.sin() * actual_radius - half_height;
            let ring_radius = phi.cos() * actual_radius;

            for seg in 0..=segments {
                let theta = 2.0 * PI * seg as f32 / segments as f32;
                let x = ring_radius * theta.cos();
                let z = ring_radius * theta.sin();

                let normalized_y = (y + half_height) / (2.0 * half_height + 2.0 * actual_radius);
                let width_blend = smoothstep(0.3, 0.7, normalized_y.max(0.0));
                let width_factor = 0.8 + lerp(hip_width, shoulder_width, width_blend) * 0.4;

                let final_x = x * width_factor;
                let final_z = z * width_factor;

                let normal = Vec3::new(final_x, phi.sin() * actual_radius, final_z).normalize();

                vertices.push(Vertex3D::new(
                    [final_x, y, final_z],
                    [normal.x, normal.y, normal.z],
                    color,
                ));
            }
        }

        // Generate indices (same as regular capsule)
        let total_rings = rings / 2 + 2 + rings / 2;
        for ring in 0..total_rings {
            for seg in 0..segments {
                let current = ring * (segments + 1) + seg;
                let next = current + segments + 1;

                indices.push(current);
                indices.push(next);
                indices.push(current + 1);

                indices.push(current + 1);
                indices.push(next);
                indices.push(next + 1);
            }
        }

        Self { vertices, indices }
    }

    /// Generate a flat plane mesh
    pub fn plane(size: f32, subdivisions: u32, color: [f32; 4]) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let half_size = size / 2.0;
        let step = size / subdivisions as f32;

        for z in 0..=subdivisions {
            for x in 0..=subdivisions {
                let px = -half_size + x as f32 * step;
                let pz = -half_size + z as f32 * step;

                vertices.push(Vertex3D::new([px, 0.0, pz], [0.0, 1.0, 0.0], color));
            }
        }

        for z in 0..subdivisions {
            for x in 0..subdivisions {
                let current = z * (subdivisions + 1) + x;
                let next = current + subdivisions + 1;

                indices.push(current);
                indices.push(next);
                indices.push(current + 1);

                indices.push(current + 1);
                indices.push(next);
                indices.push(next + 1);
            }
        }

        Self { vertices, indices }
    }

    /// Generate a plane with heightmap
    pub fn terrain(
        size: f32,
        subdivisions: u32,
        heights: &[f32],
        color_fn: impl Fn(f32, f32, f32) -> [f32; 4],
    ) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let half_size = size / 2.0;
        let step = size / subdivisions as f32;
        let vertex_count = subdivisions + 1;

        // Generate vertices with heights
        for z in 0..=subdivisions {
            for x in 0..=subdivisions {
                let px = -half_size + x as f32 * step;
                let pz = -half_size + z as f32 * step;
                let idx = (z * vertex_count + x) as usize;
                let height = heights.get(idx).copied().unwrap_or(0.0);
                let color = color_fn(px, height, pz);

                // Calculate normal from neighboring heights
                let h_left = if x > 0 {
                    heights.get((z * vertex_count + x - 1) as usize).copied().unwrap_or(height)
                } else {
                    height
                };
                let h_right = if x < subdivisions {
                    heights.get((z * vertex_count + x + 1) as usize).copied().unwrap_or(height)
                } else {
                    height
                };
                let h_down = if z > 0 {
                    heights.get(((z - 1) * vertex_count + x) as usize).copied().unwrap_or(height)
                } else {
                    height
                };
                let h_up = if z < subdivisions {
                    heights.get(((z + 1) * vertex_count + x) as usize).copied().unwrap_or(height)
                } else {
                    height
                };

                let normal = Vec3::new(h_left - h_right, 2.0 * step, h_down - h_up).normalize();

                vertices.push(Vertex3D::new(
                    [px, height, pz],
                    [normal.x, normal.y, normal.z],
                    color,
                ));
            }
        }

        // Generate indices
        for z in 0..subdivisions {
            for x in 0..subdivisions {
                let current = z * vertex_count + x;
                let next = current + vertex_count;

                indices.push(current);
                indices.push(next);
                indices.push(current + 1);

                indices.push(current + 1);
                indices.push(next);
                indices.push(next + 1);
            }
        }

        Self { vertices, indices }
    }

    /// Generate a UV sphere mesh
    pub fn sphere(radius: f32, segments: u32, rings: u32, color: [f32; 4]) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for ring in 0..=rings {
            let phi = PI * ring as f32 / rings as f32;
            let y = radius * phi.cos();
            let ring_radius = radius * phi.sin();

            for seg in 0..=segments {
                let theta = 2.0 * PI * seg as f32 / segments as f32;
                let x = ring_radius * theta.cos();
                let z = ring_radius * theta.sin();

                let normal = Vec3::new(x, y, z).normalize();

                vertices.push(Vertex3D::new(
                    [x, y, z],
                    [normal.x, normal.y, normal.z],
                    color,
                ));
            }
        }

        for ring in 0..rings {
            for seg in 0..segments {
                let current = ring * (segments + 1) + seg;
                let next = current + segments + 1;

                indices.push(current);
                indices.push(next);
                indices.push(current + 1);

                indices.push(current + 1);
                indices.push(next);
                indices.push(next + 1);
            }
        }

        Self { vertices, indices }
    }
}

/// Hermite smoothstep interpolation
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Linear interpolation
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Sky dome mesh (inverted sphere for rendering from inside)
#[derive(Clone, Debug)]
pub struct SkyMesh {
    pub vertices: Vec<SkyVertex>,
    pub indices: Vec<u32>,
}

impl SkyMesh {
    /// Generate an inverted sphere for sky rendering
    pub fn dome(segments: u32, rings: u32) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let radius = 1.0; // Unit sphere, scaled in shader

        for ring in 0..=rings {
            let phi = PI * ring as f32 / rings as f32;
            let y = radius * phi.cos();
            let ring_radius = radius * phi.sin();

            for seg in 0..=segments {
                let theta = 2.0 * PI * seg as f32 / segments as f32;
                let x = ring_radius * theta.cos();
                let z = ring_radius * theta.sin();

                vertices.push(SkyVertex::new([x, y, z]));
            }
        }

        // Indices are reversed for inside-out rendering
        for ring in 0..rings {
            for seg in 0..segments {
                let current = ring * (segments + 1) + seg;
                let next = current + segments + 1;

                // Reversed winding order
                indices.push(current);
                indices.push(current + 1);
                indices.push(next);

                indices.push(current + 1);
                indices.push(next + 1);
                indices.push(next);
            }
        }

        Self { vertices, indices }
    }
}
