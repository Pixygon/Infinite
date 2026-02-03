use std::path::Path;

use crate::error::AssetError;

/// Pixel format of a loaded texture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureFormat {
    Rgba8,
    Rgb8,
}

/// A loaded texture asset with raw pixel data.
#[derive(Debug, Clone)]
pub struct TextureAsset {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
    pub format: TextureFormat,
}

/// Load an image file and return it as an RGBA8 TextureAsset.
pub fn load_texture(path: &Path) -> Result<TextureAsset, AssetError> {
    let img = image::open(path)
        .map_err(|e| AssetError::ImageLoadFailed(path.to_path_buf(), e.to_string()))?;

    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    Ok(TextureAsset {
        width,
        height,
        data: rgba.into_raw(),
        format: TextureFormat::Rgba8,
    })
}
