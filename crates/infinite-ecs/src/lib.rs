//! Infinite ECS - Entity Component System
//!
//! A custom ECS implementation optimized for the Infinite engine.
//! Uses generational indices for entities and sparse-set storage for components.

mod component;
mod entity;
mod query;
mod resource;
mod system;
mod world;

pub use entity::Entity;
pub use query::WorldQuery;
pub use system::{System, SystemSchedule};
pub use world::World;
