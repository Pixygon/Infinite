use std::path::Path;

use tracing::debug;

use crate::error::AssetError;
use crate::mesh::{MeshAsset, MeshPrimitive};
use crate::texture::{TextureAsset, TextureFormat};

/// Result of loading a glTF file.
pub struct GltfContents {
    pub meshes: Vec<MeshAsset>,
    pub textures: Vec<TextureAsset>,
}

/// Load a glTF 2.0 file (.gltf or .glb) and extract all meshes and textures.
pub fn load_gltf(path: &Path) -> Result<GltfContents, AssetError> {
    let (document, buffers, images) = gltf::import(path)
        .map_err(|e| AssetError::GltfLoadFailed(path.to_path_buf(), e.to_string()))?;

    let mut meshes = Vec::new();
    let mut textures = Vec::new();

    // Extract meshes.
    for mesh in document.meshes() {
        let name = mesh
            .name()
            .unwrap_or("unnamed")
            .to_string();

        let mut primitives = Vec::new();

        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            let positions: Vec<[f32; 3]> = reader
                .read_positions()
                .map(|iter| iter.collect())
                .unwrap_or_default();

            let normals: Vec<[f32; 3]> = reader
                .read_normals()
                .map(|iter| iter.collect())
                .unwrap_or_default();

            let tex_coords: Option<Vec<[f32; 2]>> = reader
                .read_tex_coords(0)
                .map(|tc| tc.into_f32().collect());

            let colors: Option<Vec<[f32; 4]>> = reader
                .read_colors(0)
                .map(|c| c.into_rgba_f32().collect());

            let indices: Option<Vec<u32>> = reader
                .read_indices()
                .map(|idx| idx.into_u32().collect());

            primitives.push(MeshPrimitive {
                positions,
                normals,
                tex_coords,
                colors,
                indices,
            });
        }

        debug!("Loaded mesh '{}' with {} primitives", name, primitives.len());
        meshes.push(MeshAsset { name, primitives });
    }

    // Extract images (embedded or referenced).
    for image_data in &images {
        let (width, height) = (image_data.width, image_data.height);
        let (data, format) = match image_data.format {
            gltf::image::Format::R8G8B8A8 => {
                (image_data.pixels.clone(), TextureFormat::Rgba8)
            }
            gltf::image::Format::R8G8B8 => {
                // Convert RGB to RGBA.
                let mut rgba = Vec::with_capacity(image_data.pixels.len() / 3 * 4);
                for chunk in image_data.pixels.chunks(3) {
                    rgba.extend_from_slice(chunk);
                    rgba.push(255);
                }
                (rgba, TextureFormat::Rgba8)
            }
            _ => {
                // For other formats, try to convert via the image crate.
                let img = image::RgbaImage::from_raw(width, height, image_data.pixels.clone());
                if let Some(img) = img {
                    (img.into_raw(), TextureFormat::Rgba8)
                } else {
                    // Skip unsupported format.
                    debug!("Skipping unsupported image format in glTF");
                    continue;
                }
            }
        };

        textures.push(TextureAsset {
            width,
            height,
            data,
            format,
        });
    }

    debug!(
        "glTF '{}': {} meshes, {} textures",
        path.display(),
        meshes.len(),
        textures.len()
    );

    Ok(GltfContents { meshes, textures })
}
