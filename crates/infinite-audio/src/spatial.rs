use glam::Vec3;

/// Listener state for spatial audio calculations.
#[derive(Debug, Clone)]
pub struct Listener {
    pub position: Vec3,
    pub forward: Vec3,
    pub up: Vec3,
}

impl Default for Listener {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            forward: -Vec3::Z,
            up: Vec3::Y,
        }
    }
}

/// Parameters computed for a sound emitter relative to the listener.
#[derive(Debug, Clone, Copy)]
pub struct SpatialParams {
    /// Volume attenuation factor (0.0–1.0).
    pub volume: f64,
    /// Stereo panning (-1.0 = full left, 0.0 = center, 1.0 = full right).
    pub panning: f64,
}

/// Maximum distance at which a sound is audible.
const MAX_DISTANCE: f32 = 100.0;

/// Minimum distance before attenuation begins.
const MIN_DISTANCE: f32 = 1.0;

/// Compute spatial audio parameters for an emitter position relative to a listener.
///
/// Uses inverse-distance attenuation clamped between `MIN_DISTANCE` and `MAX_DISTANCE`.
/// Panning is derived from the angle between the listener's right vector and the
/// direction to the emitter.
pub fn compute_spatial(listener: &Listener, emitter_pos: Vec3) -> SpatialParams {
    let to_emitter = emitter_pos - listener.position;
    let distance = to_emitter.length();

    if distance < f32::EPSILON {
        return SpatialParams {
            volume: 1.0,
            panning: 0.0,
        };
    }

    // Inverse-distance attenuation.
    let clamped = distance.clamp(MIN_DISTANCE, MAX_DISTANCE);
    let volume = (MIN_DISTANCE / clamped) as f64;

    // Panning based on angle to listener's right vector.
    let right = listener.forward.cross(listener.up).normalize();
    let direction = to_emitter.normalize();
    let panning = direction.dot(right) as f64;

    SpatialParams {
        volume: volume.clamp(0.0, 1.0),
        panning: panning.clamp(-1.0, 1.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emitter_at_listener() {
        let listener = Listener::default();
        let params = compute_spatial(&listener, Vec3::ZERO);
        assert!((params.volume - 1.0).abs() < 0.01);
        assert!(params.panning.abs() < 0.01);
    }

    #[test]
    fn emitter_to_the_right() {
        let listener = Listener::default();
        let params = compute_spatial(&listener, Vec3::new(5.0, 0.0, 0.0));
        assert!(params.panning > 0.5, "should pan right: {}", params.panning);
        assert!(params.volume < 1.0, "should attenuate");
    }

    #[test]
    fn emitter_to_the_left() {
        let listener = Listener::default();
        let params = compute_spatial(&listener, Vec3::new(-5.0, 0.0, 0.0));
        assert!(params.panning < -0.5, "should pan left: {}", params.panning);
    }

    #[test]
    fn far_emitter_quiet() {
        let listener = Listener::default();
        let params = compute_spatial(&listener, Vec3::new(0.0, 0.0, -100.0));
        assert!(params.volume < 0.02, "should be very quiet at max distance: {}", params.volume);
    }

    #[test]
    fn attenuation_increases_with_distance() {
        let listener = Listener::default();
        let near = compute_spatial(&listener, Vec3::new(0.0, 0.0, -2.0));
        let far = compute_spatial(&listener, Vec3::new(0.0, 0.0, -10.0));
        assert!(near.volume > far.volume);
    }
}
