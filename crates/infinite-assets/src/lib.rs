//! Infinite Assets - Asset loading and management
//!
//! Provides glTF 2.0 model loading, texture management, and asset caching
//! for the Infinite engine.

mod error;
mod gltf_loader;
mod handle;
mod mesh;
mod server;
mod texture;

pub use error::AssetError;
pub use gltf_loader::{load_gltf, GltfContents};
pub use handle::{AssetHandle, AssetId};
pub use mesh::{MeshAsset, MeshPrimitive};
pub use server::AssetServer;
pub use texture::{load_texture, TextureAsset, TextureFormat};
