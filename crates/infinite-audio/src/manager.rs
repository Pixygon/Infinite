use std::path::Path;
use std::time::Duration;

use kira::manager::{AudioManager, AudioManagerSettings};
use kira::manager::backend::DefaultBackend;
use kira::sound::static_sound::StaticSoundHandle;
use tracing::info;

use crate::config::AudioConfig;
use crate::error::AudioError;
use crate::music::MusicPlayer;
use crate::sfx::SfxPlayer;
use crate::spatial::Listener;

/// The main audio engine. Wraps kira's AudioManager and provides high-level
/// music, SFX, and spatial audio APIs.
pub struct AudioEngine {
    manager: AudioManager<DefaultBackend>,
    music: MusicPlayer,
    sfx: SfxPlayer,
    config: AudioConfig,
    listener: Listener,
}

impl AudioEngine {
    /// Create a new AudioEngine with the given config.
    pub fn new(config: AudioConfig) -> Result<Self, AudioError> {
        let manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())
            .map_err(|e| AudioError::InitFailed(e.to_string()))?;

        info!("Audio engine initialized");

        Ok(Self {
            manager,
            music: MusicPlayer::new(config.effective_music_volume()),
            sfx: SfxPlayer::new(config.effective_sfx_volume()),
            listener: Listener::default(),
            config,
        })
    }

    /// Create an AudioEngine with default configuration.
    pub fn with_default() -> Result<Self, AudioError> {
        Self::new(AudioConfig::default())
    }

    /// Apply new volume settings at runtime.
    pub fn update_volumes(&mut self, config: AudioConfig) {
        self.music.set_volume(config.effective_music_volume());
        self.sfx.set_volume(config.effective_sfx_volume());
        self.config = config;
    }

    // ---- Music ----

    /// Play a music track, looping, with a fade-in.
    pub fn play_music(&mut self, path: &Path, fade_in: Duration) -> Result<(), AudioError> {
        self.music.play(&mut self.manager, path, fade_in)
    }

    /// Stop the current music with a fade-out.
    pub fn stop_music(&mut self, fade_out: Duration) {
        self.music.stop(fade_out);
    }

    /// Crossfade from the current music track to a new one.
    pub fn crossfade_music(&mut self, path: &Path, duration: Duration) -> Result<(), AudioError> {
        self.music.crossfade(&mut self.manager, path, duration)
    }

    // ---- Sound Effects ----

    /// Play a one-shot sound effect.
    pub fn play_sfx(&mut self, path: &Path) -> Result<(), AudioError> {
        self.sfx.play(&mut self.manager, path)
    }

    /// Play a one-shot sound effect at a 3D position.
    pub fn play_sfx_at(&mut self, path: &Path, position: glam::Vec3) -> Result<(), AudioError> {
        self.sfx
            .play_at(&mut self.manager, path, &self.listener, position)
    }

    /// Play a looping sound effect. Returns a handle to stop it later.
    pub fn play_looping(&mut self, path: &Path) -> Result<StaticSoundHandle, AudioError> {
        self.sfx.play_looping(&mut self.manager, path)
    }

    /// Stop a looping sound with a fade-out.
    pub fn stop_looping(&mut self, handle: &mut StaticSoundHandle, fade_out: Duration) {
        crate::sfx::stop_looping(handle, fade_out);
    }

    // ---- Spatial ----

    /// Update the listener position and orientation for spatial audio.
    pub fn set_listener(&mut self, position: glam::Vec3, forward: glam::Vec3, up: glam::Vec3) {
        self.listener.position = position;
        self.listener.forward = forward;
        self.listener.up = up;
    }

    // ---- Per-frame ----

    /// Call each frame to clean up finished sounds.
    pub fn update(&mut self) {
        self.sfx.cleanup();
    }

    /// Get a reference to the current audio config.
    pub fn config(&self) -> &AudioConfig {
        &self.config
    }
}
