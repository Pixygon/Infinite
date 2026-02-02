//! Time system for the Infinite engine
//!
//! Handles game time, delta time, and the era/timeline system for time travel mechanics.

use serde::{Deserialize, Serialize};

/// Configuration for past eras
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PastConfig {
    /// Name of this historical period
    pub name: String,
    /// How many years in the past (relative to Present)
    pub years_ago: u64,
    /// Description of this era
    pub description: String,
}

/// Configuration for future eras
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FutureConfig {
    /// Name of this future period
    pub name: String,
    /// How many years in the future (relative to Present)
    pub years_ahead: u64,
    /// Description of this speculative future
    pub description: String,
}

/// Represents different time periods the player can travel to
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Era {
    /// Historical periods - freely explorable in single-player
    Past(PastConfig),
    /// The "now" - MMO players are locked here, synced to real time
    Present,
    /// Speculative futures - freely explorable in single-player
    Future(FutureConfig),
}

impl Era {
    /// Create a past era with the given parameters
    pub fn past(name: impl Into<String>, years_ago: u64, description: impl Into<String>) -> Self {
        Era::Past(PastConfig {
            name: name.into(),
            years_ago,
            description: description.into(),
        })
    }

    /// Create a future era with the given parameters
    pub fn future(
        name: impl Into<String>,
        years_ahead: u64,
        description: impl Into<String>,
    ) -> Self {
        Era::Future(FutureConfig {
            name: name.into(),
            years_ahead,
            description: description.into(),
        })
    }

    /// Get the display name of this era
    pub fn name(&self) -> &str {
        match self {
            Era::Past(config) => &config.name,
            Era::Present => "Present",
            Era::Future(config) => &config.name,
        }
    }

    /// Check if this is a past era
    pub fn is_past(&self) -> bool {
        matches!(self, Era::Past(_))
    }

    /// Check if this is the present era
    pub fn is_present(&self) -> bool {
        matches!(self, Era::Present)
    }

    /// Check if this is a future era
    pub fn is_future(&self) -> bool {
        matches!(self, Era::Future(_))
    }
}

impl Default for Era {
    fn default() -> Self {
        Era::Present
    }
}

/// A timeline containing multiple eras
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    /// All available eras in this timeline
    pub eras: Vec<Era>,
    /// Currently active era index
    pub active_era: usize,
}

impl Default for Timeline {
    fn default() -> Self {
        Self {
            eras: vec![
                Era::past("Ancient", 10000, "The dawn of civilization"),
                Era::past("Medieval", 1000, "Knights and castles"),
                Era::past("Industrial", 200, "Steam and steel"),
                Era::Present,
                Era::future("Near Future", 100, "Technology ascendant"),
                Era::future("Far Future", 1000, "Among the stars"),
            ],
            active_era: 3, // Present
        }
    }
}

impl Timeline {
    /// Get the currently active era
    pub fn current_era(&self) -> &Era {
        &self.eras[self.active_era]
    }

    /// Travel to a different era by index
    pub fn travel_to(&mut self, era_index: usize) -> Result<(), TimelineError> {
        if era_index >= self.eras.len() {
            return Err(TimelineError::InvalidEra(era_index));
        }
        self.active_era = era_index;
        Ok(())
    }

    /// Travel to the next era (forward in time)
    pub fn travel_forward(&mut self) -> Result<(), TimelineError> {
        if self.active_era >= self.eras.len() - 1 {
            return Err(TimelineError::AtEndOfTimeline);
        }
        self.active_era += 1;
        Ok(())
    }

    /// Travel to the previous era (backward in time)
    pub fn travel_backward(&mut self) -> Result<(), TimelineError> {
        if self.active_era == 0 {
            return Err(TimelineError::AtBeginningOfTimeline);
        }
        self.active_era -= 1;
        Ok(())
    }
}

/// Errors that can occur during timeline operations
#[derive(Debug, Clone, thiserror::Error)]
pub enum TimelineError {
    #[error("Invalid era index: {0}")]
    InvalidEra(usize),

    #[error("Already at the end of the timeline")]
    AtEndOfTimeline,

    #[error("Already at the beginning of the timeline")]
    AtBeginningOfTimeline,
}

/// Configuration for game time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeConfig {
    /// How many in-game seconds pass per real second
    pub time_scale: f32,
    /// Whether to pause when window loses focus
    pub pause_on_unfocus: bool,
    /// Fixed timestep for physics (in seconds)
    pub fixed_timestep: f32,
    /// Maximum delta time to prevent spiral of death
    pub max_delta_time: f32,
}

impl Default for TimeConfig {
    fn default() -> Self {
        Self {
            time_scale: 1.0,
            pause_on_unfocus: true,
            fixed_timestep: 1.0 / 60.0,
            max_delta_time: 0.25,
        }
    }
}

/// Game time tracking
#[derive(Debug, Clone)]
pub struct GameTime {
    /// Configuration
    pub config: TimeConfig,
    /// Time since game start in seconds
    pub total_time: f64,
    /// Delta time for this frame (clamped)
    pub delta_time: f32,
    /// Unscaled delta time
    pub unscaled_delta_time: f32,
    /// Frame counter
    pub frame_count: u64,
    /// Whether the game is paused
    pub paused: bool,
    /// Accumulated time for fixed timestep
    fixed_accumulator: f32,
}

impl Default for GameTime {
    fn default() -> Self {
        Self {
            config: TimeConfig::default(),
            total_time: 0.0,
            delta_time: 0.0,
            unscaled_delta_time: 0.0,
            frame_count: 0,
            paused: false,
            fixed_accumulator: 0.0,
        }
    }
}

impl GameTime {
    /// Create a new game time with custom config
    pub fn new(config: TimeConfig) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    /// Update the game time with the raw delta from the previous frame
    pub fn update(&mut self, raw_delta: f32) {
        self.unscaled_delta_time = raw_delta.min(self.config.max_delta_time);
        self.frame_count += 1;

        if self.paused {
            self.delta_time = 0.0;
            return;
        }

        self.delta_time = self.unscaled_delta_time * self.config.time_scale;
        self.total_time += self.delta_time as f64;
        self.fixed_accumulator += self.delta_time;
    }

    /// Get the number of fixed timesteps to process this frame
    pub fn fixed_steps(&mut self) -> u32 {
        let mut steps = 0;
        while self.fixed_accumulator >= self.config.fixed_timestep {
            self.fixed_accumulator -= self.config.fixed_timestep;
            steps += 1;
        }
        steps
    }

    /// Get the interpolation factor for rendering between physics steps
    pub fn fixed_interpolation(&self) -> f32 {
        self.fixed_accumulator / self.config.fixed_timestep
    }

    /// Pause the game
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resume the game
    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// Toggle pause state
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    /// Set the time scale (0.0 = frozen, 1.0 = normal, 2.0 = double speed)
    pub fn set_time_scale(&mut self, scale: f32) {
        self.config.time_scale = scale.max(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeline_travel() {
        let mut timeline = Timeline::default();
        assert!(timeline.current_era().is_present());

        timeline.travel_backward().unwrap();
        assert!(timeline.current_era().is_past());

        timeline.travel_forward().unwrap();
        assert!(timeline.current_era().is_present());

        timeline.travel_forward().unwrap();
        assert!(timeline.current_era().is_future());
    }

    #[test]
    fn test_game_time() {
        let mut time = GameTime::default();
        time.update(0.016);

        assert!(time.delta_time > 0.0);
        assert_eq!(time.frame_count, 1);

        time.pause();
        time.update(0.016);
        assert_eq!(time.delta_time, 0.0);
    }
}
