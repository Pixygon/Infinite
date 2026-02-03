//! Game settings with persistence
//!
//! Settings are saved to `~/.config/infinite/settings.toml`

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tracing::{info, warn};

/// All game settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSettings {
    pub video: VideoSettings,
    pub audio: AudioSettings,
    pub gameplay: GameplaySettings,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            video: VideoSettings::default(),
            audio: AudioSettings::default(),
            gameplay: GameplaySettings::default(),
        }
    }
}

impl GameSettings {
    /// Get the config directory path
    fn config_dir() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("infinite"))
    }

    /// Get the settings file path
    fn settings_path() -> Option<PathBuf> {
        Self::config_dir().map(|p| p.join("settings.toml"))
    }

    /// Load settings from disk, or return defaults if not found
    pub fn load() -> Self {
        let Some(path) = Self::settings_path() else {
            warn!("Could not determine config directory");
            return Self::default();
        };

        if !path.exists() {
            info!("No settings file found, using defaults");
            return Self::default();
        }

        match fs::read_to_string(&path) {
            Ok(content) => match toml::from_str(&content) {
                Ok(settings) => {
                    info!("Loaded settings from {:?}", path);
                    settings
                }
                Err(e) => {
                    warn!("Failed to parse settings: {}, using defaults", e);
                    Self::default()
                }
            },
            Err(e) => {
                warn!("Failed to read settings file: {}, using defaults", e);
                Self::default()
            }
        }
    }

    /// Save settings to disk
    pub fn save(&self) -> anyhow::Result<()> {
        let Some(dir) = Self::config_dir() else {
            anyhow::bail!("Could not determine config directory");
        };

        let path = dir.join("settings.toml");

        // Create config directory if it doesn't exist
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }

        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content)?;
        info!("Saved settings to {:?}", path);
        Ok(())
    }
}

/// Video/graphics settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoSettings {
    /// Window/screen width
    pub width: u32,
    /// Window/screen height
    pub height: u32,
    /// Fullscreen mode
    pub fullscreen: bool,
    /// VSync enabled
    pub vsync: bool,
    /// Ray tracing quality (0 = off, 1 = low, 2 = medium, 3 = high, 4 = ultra)
    pub ray_tracing_quality: u8,
    /// Field of view in degrees
    pub fov: f32,
}

impl Default for VideoSettings {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            fullscreen: false,
            vsync: true,
            ray_tracing_quality: 2,
            fov: 90.0,
        }
    }
}

impl VideoSettings {
    /// Get the resolution as a tuple
    pub fn resolution(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Get ray tracing quality as a string
    pub fn ray_tracing_quality_name(&self) -> &'static str {
        match self.ray_tracing_quality {
            0 => "Off",
            1 => "Low",
            2 => "Medium",
            3 => "High",
            _ => "Ultra",
        }
    }
}

/// Audio settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSettings {
    /// Master volume (0.0 to 1.0)
    pub master: f32,
    /// Music volume (0.0 to 1.0)
    pub music: f32,
    /// Sound effects volume (0.0 to 1.0)
    pub sfx: f32,
    /// Voice/dialogue volume (0.0 to 1.0)
    pub voice: f32,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            master: 1.0,
            music: 0.8,
            sfx: 1.0,
            voice: 1.0,
        }
    }
}

/// Gameplay settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameplaySettings {
    /// Time scale multiplier (affects gameplay speed)
    pub time_scale: f32,
    /// Auto-save enabled
    pub auto_save: bool,
    /// Auto-save interval in seconds
    pub auto_save_interval: u32,
}

impl Default for GameplaySettings {
    fn default() -> Self {
        Self {
            time_scale: 1.0,
            auto_save: true,
            auto_save_interval: 300, // 5 minutes
        }
    }
}
