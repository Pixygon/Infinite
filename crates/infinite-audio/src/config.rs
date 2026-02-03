/// Audio volume configuration. Maps to the `AudioSettings` in the game's settings.
#[derive(Debug, Clone)]
pub struct AudioConfig {
    /// Master volume multiplier (0.0–1.0).
    pub master_volume: f64,
    /// Music volume multiplier (0.0–1.0).
    pub music_volume: f64,
    /// Sound effects volume multiplier (0.0–1.0).
    pub sfx_volume: f64,
    /// Voice/dialogue volume multiplier (0.0–1.0).
    pub voice_volume: f64,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            music_volume: 0.8,
            sfx_volume: 1.0,
            voice_volume: 1.0,
        }
    }
}

impl AudioConfig {
    /// Effective music volume (master * music).
    pub fn effective_music_volume(&self) -> f64 {
        self.master_volume * self.music_volume
    }

    /// Effective SFX volume (master * sfx).
    pub fn effective_sfx_volume(&self) -> f64 {
        self.master_volume * self.sfx_volume
    }

    /// Effective voice volume (master * voice).
    pub fn effective_voice_volume(&self) -> f64 {
        self.master_volume * self.voice_volume
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_volumes() {
        let config = AudioConfig::default();
        assert_eq!(config.master_volume, 1.0);
        assert_eq!(config.music_volume, 0.8);
        assert_eq!(config.sfx_volume, 1.0);
        assert_eq!(config.voice_volume, 1.0);
    }

    #[test]
    fn effective_volumes() {
        let config = AudioConfig {
            master_volume: 0.5,
            music_volume: 0.8,
            sfx_volume: 1.0,
            voice_volume: 0.6,
        };
        assert!((config.effective_music_volume() - 0.4).abs() < f64::EPSILON);
        assert!((config.effective_sfx_volume() - 0.5).abs() < f64::EPSILON);
        assert!((config.effective_voice_volume() - 0.3).abs() < f64::EPSILON);
    }
}
