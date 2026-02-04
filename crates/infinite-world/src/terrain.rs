//! Terrain generation using Perlin noise

use glam::Vec3;
use noise::{NoiseFn, Perlin};
use serde::{Deserialize, Serialize};

/// Terrain generation configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainConfig {
    /// Total size of the terrain in meters
    pub size: f32,
    /// Number of subdivisions (vertices = subdivisions + 1)
    pub subdivisions: u32,
    /// Maximum height of terrain features
    pub max_height: f32,
    /// Noise scale (smaller = larger features)
    pub noise_scale: f32,
    /// Random seed for generation
    pub seed: u32,
    /// Number of octaves for fractal noise
    pub octaves: u32,
    /// Persistence for fractal noise (amplitude decrease per octave)
    pub persistence: f32,
    /// Lacunarity for fractal noise (frequency increase per octave)
    pub lacunarity: f32,
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self {
            size: 100.0,
            subdivisions: 64,
            max_height: 5.0,
            noise_scale: 0.02,
            seed: 42,
            octaves: 4,
            persistence: 0.5,
            lacunarity: 2.0,
        }
    }
}

/// Generated terrain data
#[derive(Clone, Debug)]
pub struct Terrain {
    /// Configuration used to generate this terrain
    pub config: TerrainConfig,
    /// Height values for each vertex (row-major, size = (subdivisions+1)^2)
    pub heights: Vec<f32>,
    /// Minimum height in the terrain
    pub min_height: f32,
    /// Maximum height in the terrain
    pub max_height: f32,
}

impl Terrain {
    /// Generate terrain from configuration
    pub fn generate(config: TerrainConfig) -> Self {
        let perlin = Perlin::new(config.seed);
        let vertex_count = config.subdivisions + 1;
        let total_vertices = (vertex_count * vertex_count) as usize;

        let mut heights = Vec::with_capacity(total_vertices);
        let mut min_height = f32::MAX;
        let mut max_height = f32::MIN;

        let half_size = config.size / 2.0;
        let step = config.size / config.subdivisions as f32;

        for z in 0..vertex_count {
            for x in 0..vertex_count {
                let world_x = -half_size + x as f32 * step;
                let world_z = -half_size + z as f32 * step;

                // Generate fractal noise
                let height = fractal_noise(
                    &perlin,
                    (world_x * config.noise_scale) as f64,
                    (world_z * config.noise_scale) as f64,
                    config.octaves,
                    config.persistence,
                    config.lacunarity,
                ) * config.max_height;

                min_height = min_height.min(height);
                max_height = max_height.max(height);
                heights.push(height);
            }
        }

        Self {
            config,
            heights,
            min_height,
            max_height,
        }
    }

    /// Get the height at world coordinates (bilinear interpolation)
    pub fn height_at(&self, x: f32, z: f32) -> f32 {
        let half_size = self.config.size / 2.0;
        let step = self.config.size / self.config.subdivisions as f32;
        let vertex_count = self.config.subdivisions + 1;

        // Convert world coordinates to grid coordinates
        let grid_x = (x + half_size) / step;
        let grid_z = (z + half_size) / step;

        // Get integer and fractional parts
        let x0 = (grid_x.floor() as u32).min(self.config.subdivisions - 1);
        let z0 = (grid_z.floor() as u32).min(self.config.subdivisions - 1);
        let x1 = (x0 + 1).min(self.config.subdivisions);
        let z1 = (z0 + 1).min(self.config.subdivisions);

        let fx = grid_x.fract();
        let fz = grid_z.fract();

        // Get heights at corners
        let h00 = self.heights[(z0 * vertex_count + x0) as usize];
        let h10 = self.heights[(z0 * vertex_count + x1) as usize];
        let h01 = self.heights[(z1 * vertex_count + x0) as usize];
        let h11 = self.heights[(z1 * vertex_count + x1) as usize];

        // Bilinear interpolation
        let h0 = h00 + (h10 - h00) * fx;
        let h1 = h01 + (h11 - h01) * fx;
        h0 + (h1 - h0) * fz
    }

    /// Get the normal at world coordinates
    pub fn normal_at(&self, x: f32, z: f32) -> Vec3 {
        let epsilon = 0.5;

        let h_left = self.height_at(x - epsilon, z);
        let h_right = self.height_at(x + epsilon, z);
        let h_down = self.height_at(x, z - epsilon);
        let h_up = self.height_at(x, z + epsilon);

        Vec3::new(h_left - h_right, epsilon * 2.0, h_down - h_up).normalize()
    }

    /// Check if a point is within the terrain bounds
    pub fn contains(&self, x: f32, z: f32) -> bool {
        let half_size = self.config.size / 2.0;
        x >= -half_size && x <= half_size && z >= -half_size && z <= half_size
    }

    /// Get terrain color based on height and position
    pub fn color_at(&self, _x: f32, height: f32, _z: f32) -> [f32; 4] {
        let height_normalized =
            (height - self.min_height) / (self.max_height - self.min_height).max(0.01);

        // Color gradient based on height
        // Low = grass green, Mid = dirt brown, High = rock gray
        if height_normalized < 0.3 {
            // Grass
            let t = height_normalized / 0.3;
            let base = [0.2, 0.5, 0.15, 1.0];
            let mid = [0.3, 0.4, 0.15, 1.0];
            lerp_color(base, mid, t)
        } else if height_normalized < 0.6 {
            // Dirt/grass transition
            let t = (height_normalized - 0.3) / 0.3;
            let grass = [0.3, 0.4, 0.15, 1.0];
            let dirt = [0.45, 0.35, 0.2, 1.0];
            lerp_color(grass, dirt, t)
        } else if height_normalized < 0.85 {
            // Dirt/rock transition
            let t = (height_normalized - 0.6) / 0.25;
            let dirt = [0.45, 0.35, 0.2, 1.0];
            let rock = [0.5, 0.5, 0.5, 1.0];
            lerp_color(dirt, rock, t)
        } else {
            // Rock/snow
            let t = (height_normalized - 0.85) / 0.15;
            let rock = [0.5, 0.5, 0.5, 1.0];
            let snow = [0.9, 0.9, 0.95, 1.0];
            lerp_color(rock, snow, t)
        }
    }

    /// Generate terrain for a specific chunk at a world offset.
    ///
    /// The terrain is generated as if centered at (world_offset_x + size/2, world_offset_z + size/2),
    /// using the same noise function but sampling at the chunk's world coordinates.
    pub fn generate_chunk(config: TerrainConfig, world_offset_x: f32, world_offset_z: f32) -> Self {
        let perlin = Perlin::new(config.seed);
        let vertex_count = config.subdivisions + 1;
        let total_vertices = (vertex_count * vertex_count) as usize;

        let mut heights = Vec::with_capacity(total_vertices);
        let mut min_height = f32::MAX;
        let mut max_height = f32::MIN;

        let half_size = config.size / 2.0;
        let step = config.size / config.subdivisions as f32;

        for z in 0..vertex_count {
            for x in 0..vertex_count {
                // Local position within chunk (centered)
                let local_x = -half_size + x as f32 * step;
                let local_z = -half_size + z as f32 * step;

                // World position = chunk origin + half_size (center) + local offset
                let world_x = world_offset_x + half_size + local_x;
                let world_z = world_offset_z + half_size + local_z;

                let height = fractal_noise(
                    &perlin,
                    (world_x * config.noise_scale) as f64,
                    (world_z * config.noise_scale) as f64,
                    config.octaves,
                    config.persistence,
                    config.lacunarity,
                ) * config.max_height;

                min_height = min_height.min(height);
                max_height = max_height.max(height);
                heights.push(height);
            }
        }

        Self {
            config,
            heights,
            min_height,
            max_height,
        }
    }

    /// Get heights for physics heightfield collider
    /// Returns heights in the format expected by rapier3d
    pub fn physics_heights(&self) -> Vec<f32> {
        self.heights.clone()
    }

    /// Get the number of rows/columns for physics heightfield
    pub fn physics_dimensions(&self) -> (usize, usize) {
        let count = (self.config.subdivisions + 1) as usize;
        (count, count)
    }
}

/// Generate fractal (multi-octave) Perlin noise
fn fractal_noise(
    perlin: &Perlin,
    x: f64,
    z: f64,
    octaves: u32,
    persistence: f32,
    lacunarity: f32,
) -> f32 {
    let mut total = 0.0f32;
    let mut amplitude = 1.0f32;
    let mut frequency = 1.0f32;
    let mut max_value = 0.0f32;

    for _ in 0..octaves {
        let value = perlin.get([x as f64 * frequency as f64, z as f64 * frequency as f64]) as f32;
        total += value * amplitude;
        max_value += amplitude;
        amplitude *= persistence;
        frequency *= lacunarity;
    }

    // Normalize to -1 to 1 range, then shift to 0 to 1
    (total / max_value + 1.0) / 2.0
}

fn lerp_color(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    let t = t.clamp(0.0, 1.0);
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
        a[3] + (b[3] - a[3]) * t,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terrain_generation() {
        let config = TerrainConfig {
            size: 10.0,
            subdivisions: 4,
            ..Default::default()
        };

        let terrain = Terrain::generate(config);
        assert_eq!(terrain.heights.len(), 25); // 5x5 vertices
    }

    #[test]
    fn test_height_at() {
        let config = TerrainConfig {
            size: 10.0,
            subdivisions: 4,
            max_height: 1.0,
            ..Default::default()
        };

        let terrain = Terrain::generate(config);

        // Height at center should be valid
        let height = terrain.height_at(0.0, 0.0);
        assert!(height >= terrain.min_height);
        assert!(height <= terrain.max_height);
    }

    #[test]
    fn test_terrain_bounds() {
        let config = TerrainConfig {
            size: 100.0,
            ..Default::default()
        };

        let terrain = Terrain::generate(config);

        assert!(terrain.contains(0.0, 0.0));
        assert!(terrain.contains(49.0, 49.0));
        assert!(!terrain.contains(60.0, 0.0));
    }
}
