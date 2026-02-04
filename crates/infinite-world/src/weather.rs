//! Weather system with states and ambient modifiers

use serde::{Deserialize, Serialize};

/// Weather state
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeatherState {
    #[default]
    Clear,
    Cloudy,
    Rain,
    Storm,
}

impl WeatherState {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Clear => "Clear",
            Self::Cloudy => "Cloudy",
            Self::Rain => "Rain",
            Self::Storm => "Storm",
        }
    }

    /// Get the next weather state (for cycling)
    pub fn next(&self) -> Self {
        match self {
            Self::Clear => Self::Cloudy,
            Self::Cloudy => Self::Rain,
            Self::Rain => Self::Storm,
            Self::Storm => Self::Clear,
        }
    }

    /// Get the previous weather state
    pub fn prev(&self) -> Self {
        match self {
            Self::Clear => Self::Storm,
            Self::Cloudy => Self::Clear,
            Self::Rain => Self::Cloudy,
            Self::Storm => Self::Rain,
        }
    }
}

/// Weather configuration and state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Weather {
    /// Current weather state
    pub current: WeatherState,
    /// Cloud coverage (0.0 = clear, 1.0 = overcast)
    pub cloud_coverage: f32,
    /// Fog density (0.0 = none, 1.0 = heavy)
    pub fog_density: f32,
    /// Wind strength (affects particles if implemented)
    pub wind_strength: f32,
    /// Target weather (for transitions)
    target: WeatherState,
    /// Transition progress (0.0 = at current, 1.0 = at target)
    transition: f32,
    /// Transition speed
    transition_speed: f32,
}

impl Default for Weather {
    fn default() -> Self {
        Self {
            current: WeatherState::Clear,
            cloud_coverage: 0.0,
            fog_density: 0.0,
            wind_strength: 0.0,
            target: WeatherState::Clear,
            transition: 0.0,
            transition_speed: 0.1,
        }
    }
}

impl Weather {
    /// Create with specific weather
    pub fn new(state: WeatherState) -> Self {
        let mut weather = Self::default();
        weather.set_weather_immediate(state);
        weather
    }

    /// Update weather transitions
    pub fn update(&mut self, delta: f32) {
        if self.transition < 1.0 && self.current != self.target {
            self.transition += delta * self.transition_speed;

            if self.transition >= 1.0 {
                self.transition = 0.0;
                self.current = self.target;
            }

            self.update_parameters();
        }
    }

    /// Set target weather (will transition smoothly)
    pub fn set_weather(&mut self, state: WeatherState) {
        if state != self.current {
            self.target = state;
            self.transition = 0.0;
        } else {
            self.target = state;
            self.transition = 0.0;
            self.update_parameters();
        }
    }

    /// Immediately set weather without transition
    pub fn set_weather_immediate(&mut self, state: WeatherState) {
        self.current = state;
        self.target = state;
        self.transition = 0.0;
        self.update_parameters();
    }

    /// Cycle to next weather state
    pub fn cycle_next(&mut self) {
        self.set_weather(self.current.next());
    }

    fn update_parameters(&mut self) {
        let params = self.target.parameters();
        let current_params = self.current.parameters();

        let t = self.transition;
        self.cloud_coverage = lerp(current_params.0, params.0, t);
        self.fog_density = lerp(current_params.1, params.1, t);
        self.wind_strength = lerp(current_params.2, params.2, t);
    }

    /// Get the sun intensity modifier based on weather
    pub fn sun_modifier(&self) -> f32 {
        match self.current {
            WeatherState::Clear => 1.0,
            WeatherState::Cloudy => 0.6,
            WeatherState::Rain => 0.3,
            WeatherState::Storm => 0.1,
        }
    }

    /// Get the ambient intensity modifier based on weather
    pub fn ambient_modifier(&self) -> f32 {
        match self.current {
            WeatherState::Clear => 1.0,
            WeatherState::Cloudy => 1.2, // Softer shadows, more ambient
            WeatherState::Rain => 0.8,
            WeatherState::Storm => 0.5,
        }
    }

    /// Get the visibility distance modifier
    pub fn visibility_modifier(&self) -> f32 {
        1.0 - self.fog_density * 0.8
    }

    /// Check if precipitation should be active
    pub fn has_precipitation(&self) -> bool {
        matches!(self.current, WeatherState::Rain | WeatherState::Storm)
    }

    /// Get sky color modifiers
    pub fn sky_tint(&self) -> [f32; 3] {
        match self.current {
            WeatherState::Clear => [1.0, 1.0, 1.0],
            WeatherState::Cloudy => [0.8, 0.8, 0.85],
            WeatherState::Rain => [0.5, 0.55, 0.6],
            WeatherState::Storm => [0.3, 0.3, 0.35],
        }
    }
}

impl WeatherState {
    /// Get weather parameters: (cloud_coverage, fog_density, wind_strength)
    fn parameters(&self) -> (f32, f32, f32) {
        match self {
            Self::Clear => (0.0, 0.0, 0.1),
            Self::Cloudy => (0.6, 0.1, 0.2),
            Self::Rain => (0.9, 0.3, 0.4),
            Self::Storm => (1.0, 0.5, 0.8),
        }
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weather_modifiers() {
        let clear = Weather::new(WeatherState::Clear);
        let storm = Weather::new(WeatherState::Storm);

        assert!(clear.sun_modifier() > storm.sun_modifier());
        assert!(clear.visibility_modifier() > storm.visibility_modifier());
    }

    #[test]
    fn test_weather_cycle() {
        let mut weather = Weather::new(WeatherState::Clear);
        weather.cycle_next();
        assert_eq!(weather.target, WeatherState::Cloudy);
    }
}
