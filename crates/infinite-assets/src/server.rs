use std::collections::HashMap;
use std::path::{Path, PathBuf};

use tracing::info;

use crate::error::AssetError;
use crate::gltf_loader;
use crate::handle::{next_asset_id, AssetHandle, AssetId};
use crate::mesh::MeshAsset;
use crate::texture::{self, TextureAsset};

/// Central asset registry. Loads, caches, and provides access to game assets.
pub struct AssetServer {
    base_path: PathBuf,
    meshes: HashMap<AssetId, MeshAsset>,
    textures: HashMap<AssetId, TextureAsset>,
    path_to_mesh: HashMap<PathBuf, AssetHandle<MeshAsset>>,
    path_to_texture: HashMap<PathBuf, AssetHandle<TextureAsset>>,
}

impl AssetServer {
    /// Create a new AssetServer rooted at the given base path.
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        let base_path = base_path.into();
        info!("AssetServer created with base path: {}", base_path.display());
        Self {
            base_path,
            meshes: HashMap::new(),
            textures: HashMap::new(),
            path_to_mesh: HashMap::new(),
            path_to_texture: HashMap::new(),
        }
    }

    /// Resolve a relative asset path against the base path.
    fn resolve(&self, path: &Path) -> PathBuf {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.base_path.join(path)
        }
    }

    /// Load a glTF file and return a handle to the first mesh.
    /// Subsequent loads of the same path return the cached handle.
    pub fn load_mesh(&mut self, path: &Path) -> Result<AssetHandle<MeshAsset>, AssetError> {
        let full_path = self.resolve(path);

        // Deduplication: return existing handle if already loaded.
        if let Some(&handle) = self.path_to_mesh.get(&full_path) {
            return Ok(handle);
        }

        if !full_path.exists() {
            return Err(AssetError::NotFound(full_path));
        }

        let contents = gltf_loader::load_gltf(&full_path)?;

        let first_mesh = contents
            .meshes
            .into_iter()
            .next()
            .ok_or_else(|| AssetError::GltfLoadFailed(full_path.clone(), "no meshes found".into()))?;

        let id = next_asset_id();
        let handle = AssetHandle::new(id);
        self.meshes.insert(id, first_mesh);
        self.path_to_mesh.insert(full_path, handle);

        Ok(handle)
    }

    /// Load all meshes from a glTF file.
    pub fn load_meshes(&mut self, path: &Path) -> Result<Vec<AssetHandle<MeshAsset>>, AssetError> {
        let full_path = self.resolve(path);

        if !full_path.exists() {
            return Err(AssetError::NotFound(full_path));
        }

        let contents = gltf_loader::load_gltf(&full_path)?;
        let mut handles = Vec::new();

        for mesh in contents.meshes {
            let id = next_asset_id();
            let handle = AssetHandle::new(id);
            self.meshes.insert(id, mesh);
            handles.push(handle);
        }

        Ok(handles)
    }

    /// Load an image file (PNG, JPEG, etc.) as a texture.
    /// Subsequent loads of the same path return the cached handle.
    pub fn load_texture(
        &mut self,
        path: &Path,
    ) -> Result<AssetHandle<TextureAsset>, AssetError> {
        let full_path = self.resolve(path);

        if let Some(&handle) = self.path_to_texture.get(&full_path) {
            return Ok(handle);
        }

        if !full_path.exists() {
            return Err(AssetError::NotFound(full_path));
        }

        let tex = texture::load_texture(&full_path)?;
        let id = next_asset_id();
        let handle = AssetHandle::new(id);
        self.textures.insert(id, tex);
        self.path_to_texture.insert(full_path, handle);

        Ok(handle)
    }

    /// Get a reference to a loaded mesh by its handle.
    pub fn get_mesh(&self, handle: AssetHandle<MeshAsset>) -> Option<&MeshAsset> {
        self.meshes.get(&handle.id())
    }

    /// Get a reference to a loaded texture by its handle.
    pub fn get_texture(&self, handle: AssetHandle<TextureAsset>) -> Option<&TextureAsset> {
        self.textures.get(&handle.id())
    }

    /// Check if a mesh handle refers to a loaded asset.
    pub fn is_mesh_loaded(&self, handle: AssetHandle<MeshAsset>) -> bool {
        self.meshes.contains_key(&handle.id())
    }

    /// Check if a texture handle refers to a loaded asset.
    pub fn is_texture_loaded(&self, handle: AssetHandle<TextureAsset>) -> bool {
        self.textures.contains_key(&handle.id())
    }

    /// The base path this server resolves relative paths against.
    pub fn base_path(&self) -> &Path {
        &self.base_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn missing_file_returns_error() {
        let mut server = AssetServer::new("/nonexistent");
        let result = server.load_mesh(Path::new("does_not_exist.glb"));
        assert!(result.is_err());
        match result.unwrap_err() {
            AssetError::NotFound(_) => {}
            other => panic!("expected NotFound, got: {:?}", other),
        }
    }

    #[test]
    fn missing_texture_returns_error() {
        let mut server = AssetServer::new("/nonexistent");
        let result = server.load_texture(Path::new("does_not_exist.png"));
        assert!(result.is_err());
    }

    #[test]
    fn resolve_absolute_path() {
        let server = AssetServer::new("/home/user/assets");
        assert_eq!(
            server.resolve(Path::new("/absolute/path.glb")),
            PathBuf::from("/absolute/path.glb")
        );
    }

    #[test]
    fn resolve_relative_path() {
        let server = AssetServer::new("/home/user/assets");
        assert_eq!(
            server.resolve(Path::new("models/box.glb")),
            PathBuf::from("/home/user/assets/models/box.glb")
        );
    }
}
