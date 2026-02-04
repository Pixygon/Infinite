//! Chunk-based world streaming system
//!
//! Replaces monolithic terrain with a grid of chunks that load/unload around the player.

use std::collections::HashMap;

use glam::Vec3;
use infinite_physics::PhysicsWorld;
use rapier3d::prelude::ColliderHandle;

use crate::era_config::TimeTerrainConfig;
use crate::terrain::{Terrain, TerrainConfig};

/// Grid coordinate for a chunk
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkCoord {
    pub x: i32,
    pub z: i32,
}

impl ChunkCoord {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    /// Convert a world position to the chunk coordinate that contains it
    pub fn from_world_pos(pos: Vec3, chunk_size: f32) -> Self {
        Self {
            x: (pos.x / chunk_size).floor() as i32,
            z: (pos.z / chunk_size).floor() as i32,
        }
    }

    /// Get the world-space origin (min corner) of this chunk
    pub fn world_origin(&self, chunk_size: f32) -> Vec3 {
        Vec3::new(self.x as f32 * chunk_size, 0.0, self.z as f32 * chunk_size)
    }

    /// Get the world-space center of this chunk
    pub fn world_center(&self, chunk_size: f32) -> Vec3 {
        let origin = self.world_origin(chunk_size);
        Vec3::new(
            origin.x + chunk_size / 2.0,
            0.0,
            origin.z + chunk_size / 2.0,
        )
    }

    /// Manhattan distance to another chunk coord
    pub fn distance(&self, other: &ChunkCoord) -> u32 {
        ((self.x - other.x).unsigned_abs()).max((self.z - other.z).unsigned_abs())
    }
}

/// Configuration for the chunk system
#[derive(Clone, Debug)]
pub struct ChunkConfig {
    /// Size of each chunk in meters
    pub chunk_size: f32,
    /// Number of subdivisions per chunk (for terrain mesh)
    pub subdivisions: u32,
    /// Radius in chunks around player to keep loaded
    pub load_radius: u32,
    /// Radius beyond which chunks are unloaded (hysteresis)
    pub unload_radius: u32,
}

impl Default for ChunkConfig {
    fn default() -> Self {
        Self {
            chunk_size: 64.0,
            subdivisions: 32,
            load_radius: 3,
            unload_radius: 4,
        }
    }
}

/// A single terrain chunk
pub struct Chunk {
    /// Grid coordinate of this chunk
    pub coord: ChunkCoord,
    /// Generated terrain data for this chunk
    pub terrain: Terrain,
    /// Physics collider handle (if registered)
    pub collider_handle: Option<ColliderHandle>,
    /// Whether the terrain mesh needs rebuilding (for rendering)
    pub mesh_dirty: bool,
}

/// Manages loading/unloading of chunks around the player
pub struct ChunkManager {
    /// Chunk configuration
    pub config: ChunkConfig,
    /// Terrain generation config (base parameters)
    pub terrain_config: TerrainConfig,
    /// Currently loaded chunks
    loaded_chunks: HashMap<ChunkCoord, Chunk>,
    /// Current time-period terrain modifiers
    time_terrain_config: Option<TimeTerrainConfig>,
    /// Chunks that were just loaded this frame (for mesh creation)
    pub newly_loaded: Vec<ChunkCoord>,
    /// Chunks that were just unloaded this frame (for mesh cleanup)
    pub newly_unloaded: Vec<ChunkCoord>,
}

impl ChunkManager {
    /// Create a new chunk manager
    pub fn new(config: ChunkConfig, terrain_config: TerrainConfig) -> Self {
        Self {
            config,
            terrain_config,
            loaded_chunks: HashMap::new(),
            time_terrain_config: None,
            newly_loaded: Vec::new(),
            newly_unloaded: Vec::new(),
        }
    }

    /// Set the time-period terrain config (triggers full reload on next update)
    pub fn set_time_terrain_config(&mut self, config: Option<TimeTerrainConfig>) {
        self.time_terrain_config = config;
    }

    /// Get the current time-period terrain config
    pub fn time_terrain_config(&self) -> Option<&TimeTerrainConfig> {
        self.time_terrain_config.as_ref()
    }

    /// Get the chunk coordinate for a world position
    pub fn player_chunk(&self, player_pos: Vec3) -> ChunkCoord {
        ChunkCoord::from_world_pos(player_pos, self.config.chunk_size)
    }

    /// Get a loaded chunk by coordinate
    pub fn get_chunk(&self, coord: &ChunkCoord) -> Option<&Chunk> {
        self.loaded_chunks.get(coord)
    }

    /// Iterate over all loaded chunks
    pub fn loaded_chunks(&self) -> impl Iterator<Item = &Chunk> {
        self.loaded_chunks.values()
    }

    /// Number of currently loaded chunks
    pub fn loaded_count(&self) -> usize {
        self.loaded_chunks.len()
    }

    /// Unload all chunks and reload around the given position.
    /// Used for time-period transitions.
    pub fn reload_all(&mut self, player_pos: Vec3, physics: &mut PhysicsWorld) {
        // Unload everything
        let coords: Vec<ChunkCoord> = self.loaded_chunks.keys().copied().collect();
        for coord in coords {
            self.unload_chunk(coord, physics);
        }
        self.newly_unloaded.clear();

        // Load around player
        let center = self.player_chunk(player_pos);
        let radius = self.config.load_radius as i32;
        for dz in -radius..=radius {
            for dx in -radius..=radius {
                let coord = ChunkCoord::new(center.x + dx, center.z + dz);
                self.load_chunk(coord, physics);
            }
        }
    }

    /// Update chunk loading/unloading based on player position.
    /// Call this each frame.
    pub fn update(&mut self, player_pos: Vec3, physics: &mut PhysicsWorld) {
        self.newly_loaded.clear();
        self.newly_unloaded.clear();

        let center = self.player_chunk(player_pos);

        // Unload distant chunks
        let to_unload: Vec<ChunkCoord> = self
            .loaded_chunks
            .keys()
            .filter(|coord| coord.distance(&center) > self.config.unload_radius)
            .copied()
            .collect();

        for coord in to_unload {
            self.unload_chunk(coord, physics);
        }

        // Load nearby chunks
        let radius = self.config.load_radius as i32;
        for dz in -radius..=radius {
            for dx in -radius..=radius {
                let coord = ChunkCoord::new(center.x + dx, center.z + dz);
                if !self.loaded_chunks.contains_key(&coord) {
                    self.load_chunk(coord, physics);
                }
            }
        }
    }

    /// Get terrain height at a world position, sampling from the correct chunk
    pub fn height_at(&self, x: f32, z: f32) -> f32 {
        let coord = ChunkCoord::from_world_pos(Vec3::new(x, 0.0, z), self.config.chunk_size);
        if let Some(chunk) = self.loaded_chunks.get(&coord) {
            // Convert world pos to chunk-local coordinates
            let origin = coord.world_origin(self.config.chunk_size);
            let local_x = x - origin.x;
            let local_z = z - origin.z;
            // Terrain::height_at expects coordinates relative to the terrain's center
            let half = self.config.chunk_size / 2.0;
            chunk.terrain.height_at(local_x - half, local_z - half)
        } else {
            0.0
        }
    }

    fn load_chunk(&mut self, coord: ChunkCoord, physics: &mut PhysicsWorld) {
        let origin = coord.world_origin(self.config.chunk_size);

        // Build a terrain config for this chunk
        let mut chunk_terrain_config = TerrainConfig {
            size: self.config.chunk_size,
            subdivisions: self.config.subdivisions,
            max_height: self.terrain_config.max_height,
            noise_scale: self.terrain_config.noise_scale,
            seed: self.terrain_config.seed,
            octaves: self.terrain_config.octaves,
            persistence: self.terrain_config.persistence,
            lacunarity: self.terrain_config.lacunarity,
        };

        // Apply time-period terrain modifiers if present
        if let Some(tc) = &self.time_terrain_config {
            chunk_terrain_config.seed = chunk_terrain_config.seed.wrapping_add(tc.seed_offset);
            chunk_terrain_config.max_height *= tc.height_scale;
            chunk_terrain_config.noise_scale *= tc.noise_scale_mult;
        }

        // Generate terrain for this chunk at its world offset
        let terrain = Terrain::generate_chunk(
            chunk_terrain_config,
            origin.x,
            origin.z,
        );

        // Create physics heightfield at the chunk's world position
        let (nrows, ncols) = terrain.physics_dimensions();
        let heights = terrain.physics_heights();
        let center = coord.world_center(self.config.chunk_size);
        let collider_handle = physics.create_heightfield_at(
            &heights,
            nrows,
            ncols,
            Vec3::new(self.config.chunk_size, 1.0, self.config.chunk_size),
            Vec3::new(center.x, 0.0, center.z),
        );

        self.loaded_chunks.insert(
            coord,
            Chunk {
                coord,
                terrain,
                collider_handle: Some(collider_handle),
                mesh_dirty: true,
            },
        );
        self.newly_loaded.push(coord);
    }

    fn unload_chunk(&mut self, coord: ChunkCoord, physics: &mut PhysicsWorld) {
        if let Some(chunk) = self.loaded_chunks.remove(&coord) {
            if let Some(handle) = chunk.collider_handle {
                physics.remove_collider(handle);
            }
            self.newly_unloaded.push(coord);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_coord_from_world_pos() {
        let chunk_size = 64.0;

        // Origin maps to (0, 0)
        assert_eq!(
            ChunkCoord::from_world_pos(Vec3::new(0.0, 0.0, 0.0), chunk_size),
            ChunkCoord::new(0, 0)
        );

        // Positive coords
        assert_eq!(
            ChunkCoord::from_world_pos(Vec3::new(65.0, 0.0, 130.0), chunk_size),
            ChunkCoord::new(1, 2)
        );

        // Negative coords
        assert_eq!(
            ChunkCoord::from_world_pos(Vec3::new(-1.0, 0.0, -1.0), chunk_size),
            ChunkCoord::new(-1, -1)
        );
    }

    #[test]
    fn test_chunk_coord_distance() {
        let a = ChunkCoord::new(0, 0);
        let b = ChunkCoord::new(3, 2);
        assert_eq!(a.distance(&b), 3); // Chebyshev distance

        let c = ChunkCoord::new(-2, 1);
        assert_eq!(a.distance(&c), 2);
    }

    #[test]
    fn test_chunk_manager_load_unload() {
        let config = ChunkConfig {
            chunk_size: 64.0,
            subdivisions: 4, // Small for test
            load_radius: 1,
            unload_radius: 2,
        };
        let terrain_config = TerrainConfig {
            size: 64.0,
            subdivisions: 4,
            max_height: 5.0,
            ..Default::default()
        };

        let mut manager = ChunkManager::new(config, terrain_config);
        let mut physics = PhysicsWorld::new();

        // Load around origin
        manager.update(Vec3::ZERO, &mut physics);

        // Should have (2*1+1)^2 = 9 chunks loaded
        assert_eq!(manager.loaded_count(), 9);

        // Move far away - old chunks should unload, new ones load
        manager.update(Vec3::new(500.0, 0.0, 500.0), &mut physics);

        // Old chunks at origin should be unloaded (distance > 2)
        let origin_chunk = ChunkCoord::new(0, 0);
        assert!(manager.get_chunk(&origin_chunk).is_none());

        // New chunks around (7,7) should be loaded
        let new_center = ChunkCoord::from_world_pos(Vec3::new(500.0, 0.0, 500.0), 64.0);
        assert!(manager.get_chunk(&new_center).is_some());
    }

    #[test]
    fn test_chunk_terrain_generation_offsets() {
        let config = TerrainConfig {
            size: 64.0,
            subdivisions: 4,
            max_height: 5.0,
            ..Default::default()
        };

        // Two chunks at different positions should produce different terrain
        let t1 = Terrain::generate_chunk(config.clone(), 0.0, 0.0);
        let t2 = Terrain::generate_chunk(config, 64.0, 0.0);

        // Heights should generally differ (same noise, different offset)
        let different = t1.heights.iter().zip(t2.heights.iter()).any(|(a, b)| (a - b).abs() > 0.001);
        assert!(different, "Chunks at different positions should have different heights");
    }
}
