//! GOAP goal definitions

use super::world_state::WorldState;

/// A desired world state change with priority
#[derive(Debug, Clone)]
pub struct Goal {
    pub name: String,
    pub desired_state: WorldState,
    pub priority: f32,
}
