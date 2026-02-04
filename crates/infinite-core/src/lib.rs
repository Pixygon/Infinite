//! Infinite Core - Core types and utilities for the Infinite engine
//!
//! This crate provides the foundational types used throughout the engine:
//! - Mathematical primitives (re-exported from glam)
//! - Transform component for entity positioning
//! - Time system for game time and time travel mechanics
//! - Common error types

pub mod time;
pub mod types;

pub use glam::{Mat4, Quat, Vec2, Vec3, Vec4};
pub use time::{GameTime, TimeConfig, Timeline};
pub use types::{Color, EntityId, Transform};
