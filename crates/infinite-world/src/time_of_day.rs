//! Time of day system with sun/moon position and sky colors

use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

/// Time of day configuration and state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimeOfDay {
    /// Current time in hours (0.0 - 24.0)
    pub time_hours: f32,
    /// Duration of a full day cycle in real seconds (default: 1440 = 24 minutes)
    pub cycle_duration: f32,
    /// Whether the cycle is paused
    pub paused: bool,
}

impl Default for TimeOfDay {
    fn default() -> Self {
        Self {
            time_hours: 10.0, // Start at 10 AM
            cycle_duration: 1440.0,
            paused: false,
        }
    }
}

impl TimeOfDay {
    /// Create with a specific starting time
    pub fn new(start_time: f32) -> Self {
        Self {
            time_hours: start_time.rem_euclid(24.0),
            ..Default::default()
        }
    }

    /// Update the time of day based on delta time
    pub fn update(&mut self, delta_seconds: f32) {
        if self.paused {
            return;
        }

        // Convert real seconds to game hours
        let hours_per_second = 24.0 / self.cycle_duration;
        self.time_hours += delta_seconds * hours_per_second;
        self.time_hours = self.time_hours.rem_euclid(24.0);
    }

    /// Set the time directly
    pub fn set_time(&mut self, hours: f32) {
        self.time_hours = hours.rem_euclid(24.0);
    }

    /// Get the sun direction in world space
    /// Sun rises in +X (East), sets in -X (West), noon at +Y
    pub fn sun_direction(&self) -> Vec3 {
        // Convert time to angle (6:00 = sunrise = 0, 12:00 = noon = PI/2, 18:00 = sunset = PI)
        let sun_angle = (self.time_hours - 6.0) / 12.0 * PI;

        if self.is_day() {
            Vec3::new(-sun_angle.cos(), sun_angle.sin().max(0.01), 0.3).normalize()
        } else {
            // Sun is below horizon
            Vec3::new(0.0, -1.0, 0.0)
        }
    }

    /// Get the moon direction in world space (opposite to sun)
    pub fn moon_direction(&self) -> Vec3 {
        let moon_angle = (self.time_hours - 18.0) / 12.0 * PI;

        if self.is_night() {
            Vec3::new(-moon_angle.cos(), moon_angle.sin().max(0.01), -0.3).normalize()
        } else {
            Vec3::new(0.0, -1.0, 0.0)
        }
    }

    /// Check if it's currently daytime (6:00 - 18:00)
    pub fn is_day(&self) -> bool {
        self.time_hours >= 6.0 && self.time_hours < 18.0
    }

    /// Check if it's currently nighttime
    pub fn is_night(&self) -> bool {
        !self.is_day()
    }

    /// Get the sun intensity (0.0 at night, 1.0 at noon)
    pub fn sun_intensity(&self) -> f32 {
        if !self.is_day() {
            return 0.0;
        }

        // Smooth transition at dawn/dusk
        let dawn_end = 7.0;
        let dusk_start = 17.0;

        if self.time_hours < dawn_end {
            // Dawn transition (6:00 - 7:00)
            (self.time_hours - 6.0) / (dawn_end - 6.0)
        } else if self.time_hours > dusk_start {
            // Dusk transition (17:00 - 18:00)
            (18.0 - self.time_hours) / (18.0 - dusk_start)
        } else {
            // Full day
            1.0
        }
    }

    /// Get the moon intensity for nighttime lighting
    pub fn moon_intensity(&self) -> f32 {
        if !self.is_night() {
            return 0.0;
        }

        // Simple moon intensity (could be expanded for moon phases)
        0.15
    }

    /// Get the effective light direction (sun during day, moon at night)
    pub fn light_direction(&self) -> Vec3 {
        if self.is_day() {
            self.sun_direction()
        } else {
            self.moon_direction()
        }
    }

    /// Get the effective light intensity
    pub fn light_intensity(&self) -> f32 {
        if self.is_day() {
            self.sun_intensity()
        } else {
            self.moon_intensity()
        }
    }

    /// Get sky colors based on current time
    pub fn sky_colors(&self) -> SkyColors {
        let hour = self.time_hours;

        // Define key times and their colors
        if hour < 5.0 {
            // Deep night
            SkyColors::night()
        } else if hour < 6.0 {
            // Pre-dawn
            let t = hour - 5.0;
            SkyColors::lerp(&SkyColors::night(), &SkyColors::dawn(), t)
        } else if hour < 7.0 {
            // Dawn
            let t = hour - 6.0;
            SkyColors::lerp(&SkyColors::dawn(), &SkyColors::morning(), t)
        } else if hour < 10.0 {
            // Morning
            let t = (hour - 7.0) / 3.0;
            SkyColors::lerp(&SkyColors::morning(), &SkyColors::noon(), t)
        } else if hour < 16.0 {
            // Day
            SkyColors::noon()
        } else if hour < 17.0 {
            // Late afternoon
            let t = hour - 16.0;
            SkyColors::lerp(&SkyColors::noon(), &SkyColors::dusk(), t)
        } else if hour < 18.0 {
            // Dusk
            let t = hour - 17.0;
            SkyColors::lerp(&SkyColors::dusk(), &SkyColors::twilight(), t)
        } else if hour < 19.0 {
            // Twilight
            let t = hour - 18.0;
            SkyColors::lerp(&SkyColors::twilight(), &SkyColors::night(), t)
        } else {
            // Night
            SkyColors::night()
        }
    }

    /// Get formatted time string (HH:MM)
    pub fn formatted_time(&self) -> String {
        let hours = self.time_hours as u32;
        let minutes = ((self.time_hours - hours as f32) * 60.0) as u32;
        format!("{:02}:{:02}", hours, minutes)
    }

    /// Get time period name
    pub fn period_name(&self) -> &'static str {
        let hour = self.time_hours;
        if hour < 5.0 {
            "Night"
        } else if hour < 7.0 {
            "Dawn"
        } else if hour < 12.0 {
            "Morning"
        } else if hour < 14.0 {
            "Noon"
        } else if hour < 17.0 {
            "Afternoon"
        } else if hour < 19.0 {
            "Dusk"
        } else if hour < 21.0 {
            "Evening"
        } else {
            "Night"
        }
    }
}

/// Sky color palette for different times of day
#[derive(Clone, Copy, Debug)]
pub struct SkyColors {
    /// Color at the top of the sky (zenith)
    pub zenith: Vec3,
    /// Color at the horizon
    pub horizon: Vec3,
    /// Sun glow intensity
    pub sun_glow: f32,
    /// Sun disk size
    pub sun_size: f32,
}

impl Default for SkyColors {
    fn default() -> Self {
        Self::noon()
    }
}

impl SkyColors {
    pub fn night() -> Self {
        Self {
            zenith: Vec3::new(0.01, 0.01, 0.03),
            horizon: Vec3::new(0.02, 0.02, 0.05),
            sun_glow: 0.0,
            sun_size: 0.005, // Moon size
        }
    }

    pub fn dawn() -> Self {
        Self {
            zenith: Vec3::new(0.15, 0.1, 0.2),
            horizon: Vec3::new(0.9, 0.5, 0.3),
            sun_glow: 0.8,
            sun_size: 0.03,
        }
    }

    pub fn morning() -> Self {
        Self {
            zenith: Vec3::new(0.2, 0.4, 0.7),
            horizon: Vec3::new(0.6, 0.7, 0.8),
            sun_glow: 0.5,
            sun_size: 0.02,
        }
    }

    pub fn noon() -> Self {
        Self {
            zenith: Vec3::new(0.1, 0.3, 0.6),
            horizon: Vec3::new(0.5, 0.65, 0.8),
            sun_glow: 0.3,
            sun_size: 0.02,
        }
    }

    pub fn dusk() -> Self {
        Self {
            zenith: Vec3::new(0.2, 0.2, 0.4),
            horizon: Vec3::new(0.95, 0.5, 0.2),
            sun_glow: 0.9,
            sun_size: 0.03,
        }
    }

    pub fn twilight() -> Self {
        Self {
            zenith: Vec3::new(0.05, 0.05, 0.15),
            horizon: Vec3::new(0.3, 0.15, 0.2),
            sun_glow: 0.4,
            sun_size: 0.02,
        }
    }

    /// Linearly interpolate between two sky color palettes
    pub fn lerp(a: &Self, b: &Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        Self {
            zenith: a.zenith.lerp(b.zenith, t),
            horizon: a.horizon.lerp(b.horizon, t),
            sun_glow: a.sun_glow + (b.sun_glow - a.sun_glow) * t,
            sun_size: a.sun_size + (b.sun_size - a.sun_size) * t,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_of_day_cycle() {
        let mut tod = TimeOfDay::new(0.0);
        assert!(tod.is_night());

        tod.set_time(12.0);
        assert!(tod.is_day());
        assert_eq!(tod.sun_intensity(), 1.0);

        tod.set_time(6.5);
        assert!(tod.is_day());
        assert!(tod.sun_intensity() < 1.0); // Dawn transition
    }

    #[test]
    fn test_time_update() {
        let mut tod = TimeOfDay::new(0.0);
        tod.cycle_duration = 24.0; // 1 hour = 1 second

        tod.update(1.0);
        assert!((tod.time_hours - 1.0).abs() < 0.01);
    }
}
