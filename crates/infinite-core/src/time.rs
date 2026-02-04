//! Time system for the Infinite engine
//!
//! Handles game time, delta time, and the timeline system for time travel mechanics.
//! The world exists on a continuous year-based timeline. The "present" is a specific
//! year (for MMO mode), and single-player stories can start at any date.

use serde::{Deserialize, Serialize};

/// A timeline representing the game's continuous time system.
///
/// Rather than fixed eras, the world uses a year-based timeline where any year
/// can be visited. The same location changes organically over time — ancient ruins
/// in one period might be a thriving castle in another.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    /// The year the player is currently in
    pub active_year: i64,
    /// The "present" year (used for MMO sync; single-player can be anywhere)
    pub present_year: i64,
    /// Minimum allowed year (how far back the timeline goes)
    pub min_year: i64,
    /// Maximum allowed year (how far forward the timeline goes)
    pub max_year: i64,
}

impl Default for Timeline {
    fn default() -> Self {
        Self {
            active_year: 2025,
            present_year: 2025,
            min_year: -10000,
            max_year: 5000,
        }
    }
}

impl Timeline {
    /// Create a new timeline starting at a specific year
    pub fn new(start_year: i64, present_year: i64) -> Self {
        Self {
            active_year: start_year,
            present_year,
            min_year: -10000,
            max_year: 5000,
        }
    }

    /// Get a display label for the current year
    pub fn year_label(&self) -> String {
        format_year(self.active_year)
    }

    /// Check if the player is in the present
    pub fn is_present(&self) -> bool {
        self.active_year == self.present_year
    }

    /// Check if the player is in the past (before present)
    pub fn is_past(&self) -> bool {
        self.active_year < self.present_year
    }

    /// Check if the player is in the future (after present)
    pub fn is_future(&self) -> bool {
        self.active_year > self.present_year
    }

    /// How many years from the present the active year is (signed: negative = past)
    pub fn years_from_present(&self) -> i64 {
        self.active_year - self.present_year
    }

    /// Travel to a specific year
    pub fn travel_to_year(&mut self, year: i64) -> Result<(), TimelineError> {
        if year < self.min_year || year > self.max_year {
            return Err(TimelineError::YearOutOfRange {
                year,
                min: self.min_year,
                max: self.max_year,
            });
        }
        self.active_year = year;
        Ok(())
    }

    /// Travel forward by a number of years
    pub fn travel_forward(&mut self, years: i64) -> Result<(), TimelineError> {
        self.travel_to_year(self.active_year + years)
    }

    /// Travel backward by a number of years
    pub fn travel_backward(&mut self, years: i64) -> Result<(), TimelineError> {
        self.travel_to_year(self.active_year - years)
    }
}

/// Format a year for display (e.g., "500 BCE", "2025 CE", "3500 CE")
pub fn format_year(year: i64) -> String {
    if year <= 0 {
        format!("{} BCE", 1 - year)
    } else {
        format!("{} CE", year)
    }
}

/// Errors that can occur during timeline operations
#[derive(Debug, Clone, thiserror::Error)]
pub enum TimelineError {
    #[error("Year {year} is out of range ({min}..{max})")]
    YearOutOfRange { year: i64, min: i64, max: i64 },
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
        assert!(timeline.is_present());

        timeline.travel_backward(1000).unwrap();
        assert!(timeline.is_past());
        assert_eq!(timeline.active_year, 1025);

        timeline.travel_forward(1000).unwrap();
        assert!(timeline.is_present());

        timeline.travel_forward(500).unwrap();
        assert!(timeline.is_future());
        assert_eq!(timeline.active_year, 2525);
    }

    #[test]
    fn test_timeline_year_out_of_range() {
        let mut timeline = Timeline::default();
        assert!(timeline.travel_to_year(-20000).is_err());
        assert!(timeline.travel_to_year(10000).is_err());
        assert!(timeline.travel_to_year(3000).is_ok());
    }

    #[test]
    fn test_format_year() {
        assert_eq!(format_year(2025), "2025 CE");
        assert_eq!(format_year(0), "1 BCE");
        assert_eq!(format_year(-999), "1000 BCE");
        assert_eq!(format_year(1), "1 CE");
    }

    #[test]
    fn test_year_label() {
        let timeline = Timeline::new(-5000, 2025);
        assert_eq!(timeline.year_label(), "5001 BCE");
        assert!(timeline.is_past());
        assert_eq!(timeline.years_from_present(), -7025);
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
