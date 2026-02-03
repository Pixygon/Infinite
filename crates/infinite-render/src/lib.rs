//! Infinite Render - Vulkan-based renderer with ray tracing
//!
//! Provides both hardware ray tracing (VK_KHR_ray_tracing_pipeline) and
//! compute shader fallback for universal compatibility.

pub mod mesh;
pub mod scene;
pub mod vertex;

pub use mesh::{Mesh, SkyMesh};
pub use scene::{BasicPushConstants, SceneUniforms, SkyColors, SkyPushConstants};
pub use vertex::{SkyVertex, Vertex3D};
