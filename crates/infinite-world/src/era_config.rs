//! Per-era terrain configuration modifiers
//!
//! Different eras produce visually distinct terrain by modifying noise parameters.

use serde::{Deserialize, Serialize};

/// Terrain modifiers for a specific era
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EraTerrainConfig {
    /// Added to the base seed to produce different noise patterns per era
    pub seed_offset: u32,
    /// Multiplier on max_height (1.0 = default)
    pub height_scale: f32,
    /// Multiplier on noise_scale (1.0 = default)
    pub noise_scale_mult: f32,
}

impl EraTerrainConfig {
    /// Get terrain config for a given era index.
    ///
    /// Default timeline:
    /// 0 = Ancient, 1 = Medieval, 2 = Industrial, 3 = Present, 4 = Near Future, 5 = Far Future
    pub fn for_era(era_index: usize) -> Self {
        match era_index {
            0 => Self {
                // Ancient: dramatic, mountainous landscape
                seed_offset: 300,
                height_scale: 2.0,
                noise_scale_mult: 0.6,
            },
            1 => Self {
                // Medieval: rolling hills
                seed_offset: 200,
                height_scale: 1.5,
                noise_scale_mult: 0.8,
            },
            2 => Self {
                // Industrial: moderately hilly
                seed_offset: 100,
                height_scale: 1.2,
                noise_scale_mult: 0.9,
            },
            3 => Self {
                // Present: default terrain
                seed_offset: 0,
                height_scale: 1.0,
                noise_scale_mult: 1.0,
            },
            4 => Self {
                // Near Future: slightly flatter, more detail
                seed_offset: 400,
                height_scale: 0.8,
                noise_scale_mult: 1.3,
            },
            5 => Self {
                // Far Future: very flat, high-frequency detail
                seed_offset: 500,
                height_scale: 0.6,
                noise_scale_mult: 1.5,
            },
            _ => Self {
                // Unknown era: use defaults
                seed_offset: era_index as u32 * 100,
                height_scale: 1.0,
                noise_scale_mult: 1.0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_different_eras_produce_different_configs() {
        let past = EraTerrainConfig::for_era(1);
        let present = EraTerrainConfig::for_era(3);
        let future = EraTerrainConfig::for_era(5);

        // Each era should have a different seed offset
        assert_ne!(past.seed_offset, present.seed_offset);
        assert_ne!(present.seed_offset, future.seed_offset);

        // Past should be taller, future should be flatter
        assert!(past.height_scale > present.height_scale);
        assert!(future.height_scale < present.height_scale);
    }

    #[test]
    fn test_present_era_is_default() {
        let present = EraTerrainConfig::for_era(3);
        assert_eq!(present.seed_offset, 0);
        assert_eq!(present.height_scale, 1.0);
        assert_eq!(present.noise_scale_mult, 1.0);
    }
}
