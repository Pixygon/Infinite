use std::collections::HashMap;
use std::path::{Path, PathBuf};

use kira::manager::AudioManager;
use kira::manager::backend::DefaultBackend;
use kira::sound::static_sound::{StaticSoundData, StaticSoundHandle, StaticSoundSettings};
use kira::sound::PlaybackState;
use kira::tween::Tween;

use crate::error::AudioError;
use crate::spatial::{self, Listener, SpatialParams};

/// Manages fire-and-forget sound effects with basic caching.
pub struct SfxPlayer {
    cache: HashMap<PathBuf, StaticSoundData>,
    active: Vec<StaticSoundHandle>,
    sfx_volume: f64,
}

impl SfxPlayer {
    pub fn new(sfx_volume: f64) -> Self {
        Self {
            cache: HashMap::new(),
            active: Vec::new(),
            sfx_volume,
        }
    }

    /// Play a one-shot sound effect at default volume.
    pub fn play(
        &mut self,
        manager: &mut AudioManager<DefaultBackend>,
        path: &Path,
    ) -> Result<(), AudioError> {
        let data = self.load_or_cache(path)?;
        let settings = StaticSoundSettings::new().volume(self.sfx_volume);
        let data = data.with_settings(settings);
        let handle = manager.play(data).map_err(|e| AudioError::PlaybackFailed(e.to_string()))?;
        self.active.push(handle);
        Ok(())
    }

    /// Play a sound effect with spatial positioning.
    pub fn play_at(
        &mut self,
        manager: &mut AudioManager<DefaultBackend>,
        path: &Path,
        listener: &Listener,
        position: glam::Vec3,
    ) -> Result<(), AudioError> {
        let SpatialParams { volume, panning } = spatial::compute_spatial(listener, position);
        let data = self.load_or_cache(path)?;
        let settings = StaticSoundSettings::new()
            .volume(self.sfx_volume * volume)
            .panning(panning);
        let data = data.with_settings(settings);
        let handle = manager.play(data).map_err(|e| AudioError::PlaybackFailed(e.to_string()))?;
        self.active.push(handle);
        Ok(())
    }

    /// Play a looping sound effect. Returns the handle so the caller can stop it later.
    pub fn play_looping(
        &mut self,
        manager: &mut AudioManager<DefaultBackend>,
        path: &Path,
    ) -> Result<StaticSoundHandle, AudioError> {
        let data = self.load_or_cache(path)?;
        let settings = StaticSoundSettings::new()
            .volume(self.sfx_volume)
            .loop_region(..);
        let data = data.with_settings(settings);
        let handle = manager.play(data).map_err(|e| AudioError::PlaybackFailed(e.to_string()))?;
        Ok(handle)
    }

    /// Update volume on all active sounds and remove finished ones.
    pub fn set_volume(&mut self, volume: f64) {
        self.sfx_volume = volume;
    }

    /// Remove handles for sounds that have stopped playing.
    pub fn cleanup(&mut self) {
        self.active.retain(|h| h.state() != PlaybackState::Stopped);
    }

    fn load_or_cache(&mut self, path: &Path) -> Result<StaticSoundData, AudioError> {
        if let Some(data) = self.cache.get(path) {
            return Ok(data.clone());
        }
        let data = StaticSoundData::from_file(path)
            .map_err(|e| AudioError::LoadFailed(path.to_path_buf(), e.to_string()))?;
        self.cache.insert(path.to_path_buf(), data.clone());
        Ok(data)
    }
}

/// Stop a looping sound with a fade-out.
pub fn stop_looping(handle: &mut StaticSoundHandle, fade_out: std::time::Duration) {
    handle.stop(Tween {
        duration: fade_out,
        ..Default::default()
    });
}
