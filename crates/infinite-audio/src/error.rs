use std::path::PathBuf;

/// Errors that can occur in the audio system.
#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("failed to initialize audio backend: {0}")]
    InitFailed(String),

    #[error("failed to load audio file '{0}': {1}")]
    LoadFailed(PathBuf, String),

    #[error("audio playback failed: {0}")]
    PlaybackFailed(String),
}
