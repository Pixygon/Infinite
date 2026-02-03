//! Infinite Physics - Physics simulation using rapier3d
//!
//! Provides collision detection, rigid body dynamics, and character controllers.

mod character_controller;

pub use character_controller::CharacterController;

use glam::Vec3;
use nalgebra::Unit;
use rapier3d::prelude::*;

/// Physics world configuration
#[derive(Debug, Clone)]
pub struct PhysicsConfig {
    /// Gravity vector (default: -9.81 on Y axis)
    pub gravity: Vec3,
    /// Physics timestep (default: 1/60)
    pub timestep: f32,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            timestep: 1.0 / 60.0,
        }
    }
}

/// The main physics world containing all simulation state
pub struct PhysicsWorld {
    /// Configuration
    pub config: PhysicsConfig,

    /// Rigid body storage
    pub rigid_body_set: RigidBodySet,
    /// Collider storage
    pub collider_set: ColliderSet,
    /// Impulse joint storage
    pub impulse_joint_set: ImpulseJointSet,
    /// Multi-body joint storage
    pub multibody_joint_set: MultibodyJointSet,

    /// Integration parameters
    integration_parameters: IntegrationParameters,
    /// Physics pipeline
    physics_pipeline: PhysicsPipeline,
    /// Island manager
    island_manager: IslandManager,
    /// Broad phase collision detection
    broad_phase: DefaultBroadPhase,
    /// Narrow phase collision detection
    narrow_phase: NarrowPhase,
    /// Continuous collision detection solver
    ccd_solver: CCDSolver,
    /// Query pipeline for raycasts and shape casts
    query_pipeline: QueryPipeline,
}

impl PhysicsWorld {
    /// Create a new physics world with default configuration
    pub fn new() -> Self {
        Self::with_config(PhysicsConfig::default())
    }

    /// Create a new physics world with custom configuration
    pub fn with_config(config: PhysicsConfig) -> Self {
        let mut integration_parameters = IntegrationParameters::default();
        integration_parameters.dt = config.timestep;

        Self {
            config,
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            integration_parameters,
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            ccd_solver: CCDSolver::new(),
            query_pipeline: QueryPipeline::new(),
        }
    }

    /// Step the physics simulation
    pub fn step(&mut self) {
        let gravity = vector![self.config.gravity.x, self.config.gravity.y, self.config.gravity.z];

        self.physics_pipeline.step(
            &gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            None,
            &(),
            &(),
        );

        // Update query pipeline after physics step
        self.query_pipeline.update(&self.collider_set);
    }

    /// Add a static collider (ground, walls, etc.)
    pub fn add_static_collider(&mut self, collider: Collider) -> ColliderHandle {
        self.collider_set.insert(collider)
    }

    /// Add a dynamic rigid body with a collider
    pub fn add_dynamic_body(
        &mut self,
        rigid_body: RigidBody,
        collider: Collider,
    ) -> (RigidBodyHandle, ColliderHandle) {
        let rb_handle = self.rigid_body_set.insert(rigid_body);
        let col_handle =
            self.collider_set
                .insert_with_parent(collider, rb_handle, &mut self.rigid_body_set);
        (rb_handle, col_handle)
    }

    /// Add a kinematic rigid body with a collider
    pub fn add_kinematic_body(
        &mut self,
        rigid_body: RigidBody,
        collider: Collider,
    ) -> (RigidBodyHandle, ColliderHandle) {
        let rb_handle = self.rigid_body_set.insert(rigid_body);
        let col_handle =
            self.collider_set
                .insert_with_parent(collider, rb_handle, &mut self.rigid_body_set);
        (rb_handle, col_handle)
    }

    /// Remove a rigid body and its colliders
    pub fn remove_rigid_body(&mut self, handle: RigidBodyHandle) {
        self.rigid_body_set.remove(
            handle,
            &mut self.island_manager,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            true,
        );
    }

    /// Remove a collider
    pub fn remove_collider(&mut self, handle: ColliderHandle) {
        self.collider_set
            .remove(handle, &mut self.island_manager, &mut self.rigid_body_set, true);
    }

    /// Get a rigid body by handle
    pub fn get_rigid_body(&self, handle: RigidBodyHandle) -> Option<&RigidBody> {
        self.rigid_body_set.get(handle)
    }

    /// Get a mutable rigid body by handle
    pub fn get_rigid_body_mut(&mut self, handle: RigidBodyHandle) -> Option<&mut RigidBody> {
        self.rigid_body_set.get_mut(handle)
    }

    /// Get a collider by handle
    pub fn get_collider(&self, handle: ColliderHandle) -> Option<&Collider> {
        self.collider_set.get(handle)
    }

    /// Cast a ray and return the first hit
    pub fn raycast(
        &self,
        origin: Vec3,
        direction: Vec3,
        max_distance: f32,
        filter: QueryFilter,
    ) -> Option<(ColliderHandle, f32)> {
        let ray = Ray::new(
            point![origin.x, origin.y, origin.z],
            vector![direction.x, direction.y, direction.z],
        );

        self.query_pipeline
            .cast_ray(&self.rigid_body_set, &self.collider_set, &ray, max_distance, true, filter)
            .map(|(handle, toi)| (handle, toi))
    }

    /// Cast a ray and get detailed hit information
    pub fn raycast_detailed(
        &self,
        origin: Vec3,
        direction: Vec3,
        max_distance: f32,
        filter: QueryFilter,
    ) -> Option<RaycastHit> {
        let ray = Ray::new(
            point![origin.x, origin.y, origin.z],
            vector![direction.x, direction.y, direction.z],
        );

        self.query_pipeline
            .cast_ray_and_get_normal(&self.rigid_body_set, &self.collider_set, &ray, max_distance, true, filter)
            .map(|(handle, intersection)| RaycastHit {
                collider: handle,
                distance: intersection.time_of_impact,
                point: Vec3::new(
                    origin.x + direction.x * intersection.time_of_impact,
                    origin.y + direction.y * intersection.time_of_impact,
                    origin.z + direction.z * intersection.time_of_impact,
                ),
                normal: Vec3::new(
                    intersection.normal.x,
                    intersection.normal.y,
                    intersection.normal.z,
                ),
            })
    }

    /// Create a ground plane collider
    pub fn create_ground(&mut self, y: f32) -> ColliderHandle {
        let normal = Unit::new_normalize(vector![0.0, 1.0, 0.0]);
        let ground = ColliderBuilder::halfspace(normal)
            .translation(vector![0.0, y, 0.0])
            .friction(0.7)
            .restitution(0.0)
            .build();
        self.add_static_collider(ground)
    }

    /// Create a terrain heightfield collider from height data
    ///
    /// - `heights`: row-major height values (nrows * ncols), Z-outer X-inner
    /// - `nrows`: number of rows (Z axis vertex count)
    /// - `ncols`: number of columns (X axis vertex count)
    /// - `scale`: world-space size (x = total X size, y = height scale, z = total Z size)
    pub fn create_heightfield(
        &mut self,
        heights: &[f32],
        nrows: usize,
        ncols: usize,
        scale: Vec3,
    ) -> ColliderHandle {
        use nalgebra::DMatrix;

        // rapier3d heightfield expects DMatrix<Real> with nrows x ncols
        let matrix = DMatrix::from_fn(nrows, ncols, |r, c| {
            heights[r * ncols + c]
        });

        let collider = ColliderBuilder::heightfield(matrix, vector![scale.x, scale.y, scale.z])
            .friction(0.7)
            .restitution(0.0)
            .build();

        self.add_static_collider(collider)
    }

    /// Create a static box collider
    pub fn create_static_box(&mut self, half_extents: Vec3, position: Vec3) -> ColliderHandle {
        let collider = ColliderBuilder::cuboid(half_extents.x, half_extents.y, half_extents.z)
            .translation(vector![position.x, position.y, position.z])
            .friction(0.7)
            .build();
        self.add_static_collider(collider)
    }
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self::new()
    }
}

/// Detailed raycast hit information
#[derive(Debug, Clone)]
pub struct RaycastHit {
    /// The collider that was hit
    pub collider: ColliderHandle,
    /// Distance along the ray to the hit point
    pub distance: f32,
    /// World-space hit point
    pub point: Vec3,
    /// Surface normal at hit point
    pub normal: Vec3,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physics_world_creation() {
        let world = PhysicsWorld::new();
        assert_eq!(world.config.gravity, Vec3::new(0.0, -9.81, 0.0));
    }

    #[test]
    fn test_ground_creation() {
        let mut world = PhysicsWorld::new();
        let ground = world.create_ground(0.0);
        assert!(world.get_collider(ground).is_some());
    }

    #[test]
    fn test_raycast() {
        let mut world = PhysicsWorld::new();
        world.create_ground(0.0);
        world.query_pipeline.update(&world.collider_set);

        let hit = world.raycast(
            Vec3::new(0.0, 10.0, 0.0),
            Vec3::new(0.0, -1.0, 0.0),
            100.0,
            QueryFilter::default(),
        );
        assert!(hit.is_some());
    }
}
