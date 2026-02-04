//! Year-based terrain configuration modifiers
//!
//! Different time periods produce visually distinct terrain by modifying noise parameters.
//! Terrain changes smoothly based on how far from the present the active year is.

use serde::{Deserialize, Serialize};

/// Terrain modifiers for a specific time period (year)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeTerrainConfig {
    /// Added to the base seed to produce different noise patterns per time period
    pub seed_offset: u32,
    /// Multiplier on max_height (1.0 = default)
    pub height_scale: f32,
    /// Multiplier on noise_scale (1.0 = default)
    pub noise_scale_mult: f32,
}

impl TimeTerrainConfig {
    /// Get terrain config for a given year, relative to the present year.
    ///
    /// Terrain characteristics change smoothly based on distance from present:
    /// - Far past (>5000 years ago): dramatic, mountainous landscape
    /// - Near past (~1000 years ago): rolling hills
    /// - Present: default terrain
    /// - Near future (~500 years ahead): slightly flatter, more detail
    /// - Far future (>2000 years ahead): very flat, high-frequency detail
    pub fn for_year(year: i64, present_year: i64) -> Self {
        let years_from_present = year - present_year;

        if years_from_present == 0 {
            // Present: default terrain
            return Self {
                seed_offset: 0,
                height_scale: 1.0,
                noise_scale_mult: 1.0,
            };
        }

        // Use absolute distance for magnitude, sign for direction
        let abs_years = years_from_present.unsigned_abs() as f32;

        if years_from_present < 0 {
            // Past: terrain gets more dramatic the further back you go
            let t = (abs_years / 5000.0).min(1.0); // normalize to 0..1 over 5000 years
            Self {
                seed_offset: (abs_years as u32 / 10).wrapping_mul(73),
                height_scale: 1.0 + t * 1.0,         // 1.0 to 2.0
                noise_scale_mult: 1.0 - t * 0.4,      // 1.0 to 0.6
            }
        } else {
            // Future: terrain gets flatter and more detailed the further forward
            let t = (abs_years / 3000.0).min(1.0); // normalize to 0..1 over 3000 years
            Self {
                seed_offset: (abs_years as u32 / 10).wrapping_mul(97),
                height_scale: 1.0 - t * 0.4,         // 1.0 to 0.6
                noise_scale_mult: 1.0 + t * 0.5,      // 1.0 to 1.5
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_present_is_default() {
        let config = TimeTerrainConfig::for_year(2025, 2025);
        assert_eq!(config.seed_offset, 0);
        assert_eq!(config.height_scale, 1.0);
        assert_eq!(config.noise_scale_mult, 1.0);
    }

    #[test]
    fn test_past_is_taller() {
        let present = TimeTerrainConfig::for_year(2025, 2025);
        let past = TimeTerrainConfig::for_year(25, 2025);
        assert!(past.height_scale > present.height_scale);
    }

    #[test]
    fn test_future_is_flatter() {
        let present = TimeTerrainConfig::for_year(2025, 2025);
        let future = TimeTerrainConfig::for_year(4025, 2025);
        assert!(future.height_scale < present.height_scale);
    }

    #[test]
    fn test_different_years_different_seeds() {
        let a = TimeTerrainConfig::for_year(1025, 2025);
        let b = TimeTerrainConfig::for_year(3025, 2025);
        assert_ne!(a.seed_offset, b.seed_offset);
    }
}
