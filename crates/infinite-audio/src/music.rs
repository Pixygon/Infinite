use std::path::Path;
use std::time::Duration;

use kira::manager::AudioManager;
use kira::manager::backend::DefaultBackend;
use kira::sound::static_sound::{StaticSoundData, StaticSoundHandle, StaticSoundSettings};
use kira::tween::Tween;

use crate::error::AudioError;

/// Manages background music playback with crossfade support.
pub struct MusicPlayer {
    current: Option<StaticSoundHandle>,
    music_volume: f64,
}

impl MusicPlayer {
    pub fn new(music_volume: f64) -> Self {
        Self {
            current: None,
            music_volume,
        }
    }

    /// Start playing a music track, fading in over the given duration.
    pub fn play(
        &mut self,
        manager: &mut AudioManager<DefaultBackend>,
        path: &Path,
        fade_in: Duration,
    ) -> Result<(), AudioError> {
        self.stop(fade_in);

        let data = StaticSoundData::from_file(path)
            .map_err(|e| AudioError::LoadFailed(path.to_path_buf(), e.to_string()))?;
        let settings = StaticSoundSettings::new()
            .volume(0.0)
            .loop_region(..);
        let data = data.with_settings(settings);

        let mut handle = manager
            .play(data)
            .map_err(|e| AudioError::PlaybackFailed(e.to_string()))?;

        handle.set_volume(
            self.music_volume,
            Tween {
                duration: fade_in,
                ..Default::default()
            },
        );

        self.current = Some(handle);
        Ok(())
    }

    /// Stop the current music track with a fade-out.
    pub fn stop(&mut self, fade_out: Duration) {
        if let Some(ref mut handle) = self.current {
            handle.stop(Tween {
                duration: fade_out,
                ..Default::default()
            });
        }
        self.current = None;
    }

    /// Crossfade from the current track to a new one.
    pub fn crossfade(
        &mut self,
        manager: &mut AudioManager<DefaultBackend>,
        path: &Path,
        duration: Duration,
    ) -> Result<(), AudioError> {
        // Fade out old track
        if let Some(ref mut handle) = self.current {
            handle.stop(Tween {
                duration,
                ..Default::default()
            });
        }

        // Start new track fading in
        let data = StaticSoundData::from_file(path)
            .map_err(|e| AudioError::LoadFailed(path.to_path_buf(), e.to_string()))?;
        let settings = StaticSoundSettings::new()
            .volume(0.0)
            .loop_region(..);
        let data = data.with_settings(settings);

        let mut handle = manager
            .play(data)
            .map_err(|e| AudioError::PlaybackFailed(e.to_string()))?;

        handle.set_volume(
            self.music_volume,
            Tween {
                duration,
                ..Default::default()
            },
        );

        self.current = Some(handle);
        Ok(())
    }

    /// Update the music volume (applied to the current track immediately).
    pub fn set_volume(&mut self, volume: f64) {
        self.music_volume = volume;
        if let Some(ref mut handle) = self.current {
            handle.set_volume(volume, Tween::default());
        }
    }
}
