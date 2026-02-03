//! Infinite Audio - Audio playback and management using kira
//!
//! Provides sound effects, music, and spatial audio for the Infinite engine.

mod config;
mod error;
mod manager;
mod music;
mod sfx;
mod spatial;

pub use config::AudioConfig;
pub use error::AudioError;
pub use manager::AudioEngine;
pub use spatial::{compute_spatial, Listener, SpatialParams};
