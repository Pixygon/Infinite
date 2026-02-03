//! Camera system module
//!
//! Provides first-person and third-person camera with mouse look and zoom.

mod config;
mod controller;

pub use config::CameraConfig;
pub use controller::{CameraController, CameraMode};
