use std::path::PathBuf;

/// Errors that can occur during asset loading.
#[derive(Debug, thiserror::Error)]
pub enum AssetError {
    #[error("asset not found: {0}")]
    NotFound(PathBuf),

    #[error("failed to load glTF file '{0}': {1}")]
    GltfLoadFailed(PathBuf, String),

    #[error("failed to load image '{0}': {1}")]
    ImageLoadFailed(PathBuf, String),

    #[error("I/O error loading '{0}': {1}")]
    Io(PathBuf, #[source] std::io::Error),

    #[error("unsupported image format in '{0}'")]
    UnsupportedFormat(PathBuf),
}
