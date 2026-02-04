//! GOAP action definitions

use super::world_state::WorldState;

/// Something an NPC can do to change world state
#[derive(Debug, Clone)]
pub struct Action {
    pub name: String,
    pub preconditions: WorldState,
    pub effects: WorldState,
    pub cost: f32,
    pub duration: f32,
}
