//! Interaction system for interactable objects in the world
//!
//! Players can focus on nearby interactables and interact with them (E key).
//! Stateful interactables (doors, levers, containers) persist their state
//! and can be saved/loaded.

use std::collections::HashMap;

use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::npc::NpcId;

/// Unique identifier for a stateful interactable
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InteractableId(pub u64);

/// Persistent state for stateful interactables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InteractableState {
    Door { is_open: bool, is_locked: bool },
    Lever { is_on: bool, linked_ids: Vec<InteractableId> },
    Button { is_pressed: bool },
    Container { is_open: bool, items: Vec<String> },
}

/// Serializable snapshot of all interaction world state
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InteractionSaveData {
    pub states: Vec<(u64, InteractableState)>,
    pub next_id: u64,
}

/// The kind of interactable object
#[derive(Debug, Clone)]
pub enum InteractableKind {
    /// A sign that displays text when read
    Sign { text: String },
    /// A portal that transports the player to a different year
    TimePortal { target_year: i64 },
    /// An item that can be picked up
    Pickup { item_name: String },
    /// An NPC that can be talked to
    Npc { npc_id: NpcId },
    /// A door that can be opened/closed (may be locked)
    Door { id: InteractableId },
    /// A lever that toggles linked objects
    Lever { id: InteractableId },
    /// A button that can be pressed
    Button { id: InteractableId },
    /// A container with items inside
    Container { id: InteractableId },
    /// A ladder that can be climbed
    Ladder { height: f32, direction: Vec3 },
}

/// Result of interacting with an object
#[derive(Debug, Clone)]
pub enum InteractionResult {
    /// Show text to the player (from a sign)
    ShowText(String),
    /// Travel to a different year
    ChangeTimePeriod(i64),
    /// Pick up an item
    PickupItem(String),
    /// Talk to an NPC
    TalkToNpc(NpcId),
    /// Toggle a door open/closed
    ToggleDoor { id: InteractableId, now_open: bool },
    /// Toggle a lever and get linked IDs to trigger
    ToggleLever { id: InteractableId, now_on: bool, linked: Vec<InteractableId> },
    /// Press a button
    PressButton { id: InteractableId },
    /// Open a container and get its items
    OpenContainer { id: InteractableId, items: Vec<String> },
    /// Start climbing a ladder
    StartClimbing { height: f32, direction: Vec3 },
    /// The object is locked
    Locked,
}

/// An interactable object in the world
#[derive(Debug, Clone)]
pub struct Interactable {
    /// What kind of interactable this is
    pub kind: InteractableKind,
    /// World position
    pub position: Vec3,
    /// Maximum distance for interaction
    pub interaction_radius: f32,
    /// Prompt text shown when focused (e.g., "Read", "Enter Portal")
    pub prompt: String,
}

impl Interactable {
    /// Create a sign interactable
    pub fn sign(position: Vec3, text: impl Into<String>) -> Self {
        Self {
            kind: InteractableKind::Sign {
                text: text.into(),
            },
            position,
            interaction_radius: 3.0,
            prompt: "Read".to_string(),
        }
    }

    /// Create a time portal interactable
    pub fn time_portal(position: Vec3, target_year: i64, label: impl Into<String>) -> Self {
        Self {
            kind: InteractableKind::TimePortal { target_year },
            position,
            interaction_radius: 4.0,
            prompt: format!("Enter {}", label.into()),
        }
    }

    /// Create a pickup interactable
    pub fn pickup(position: Vec3, item_name: impl Into<String>) -> Self {
        let name = item_name.into();
        Self {
            kind: InteractableKind::Pickup {
                item_name: name.clone(),
            },
            position,
            interaction_radius: 2.5,
            prompt: format!("Pick up {}", name),
        }
    }

    /// Create an NPC interactable
    pub fn npc(position: Vec3, npc_id: NpcId, name: impl Into<String>, interaction_radius: f32) -> Self {
        Self {
            kind: InteractableKind::Npc { npc_id },
            position,
            interaction_radius,
            prompt: format!("Talk to {}", name.into()),
        }
    }
}

/// Manages interactable objects and focus detection
pub struct InteractionSystem {
    /// All interactables in the world
    interactables: Vec<Interactable>,
    /// Index of the currently focused interactable (if any)
    focused: Option<usize>,
    /// Persistent state for stateful interactables (doors, levers, etc.)
    world_state: HashMap<InteractableId, InteractableState>,
    /// Next ID to assign
    next_id: u64,
}

impl InteractionSystem {
    /// Create a new empty interaction system
    pub fn new() -> Self {
        Self {
            interactables: Vec::new(),
            focused: None,
            world_state: HashMap::new(),
            next_id: 1,
        }
    }

    /// Add an interactable to the system
    pub fn add(&mut self, interactable: Interactable) {
        self.interactables.push(interactable);
    }

    /// Clear all interactables
    pub fn clear(&mut self) {
        self.interactables.clear();
        self.focused = None;
    }

    /// Clear interactables but keep world state (for chunk reload)
    pub fn clear_keeping_state(&mut self) {
        self.interactables.clear();
        self.focused = None;
    }

    /// Remove all interactables matching a predicate
    pub fn retain(&mut self, f: impl Fn(&Interactable) -> bool) {
        self.interactables.retain(|i| f(i));
        self.focused = None;
    }

    /// Number of interactables
    pub fn count(&self) -> usize {
        self.interactables.len()
    }

    // --- Builder methods for stateful interactables ---

    /// Add a door and return its ID
    pub fn add_door(&mut self, position: Vec3, is_locked: bool) -> InteractableId {
        let id = InteractableId(self.next_id);
        self.next_id += 1;

        self.world_state.insert(id, InteractableState::Door {
            is_open: false,
            is_locked,
        });

        let prompt = if is_locked { "Locked" } else { "Open Door" };
        self.interactables.push(Interactable {
            kind: InteractableKind::Door { id },
            position,
            interaction_radius: 3.0,
            prompt: prompt.to_string(),
        });

        id
    }

    /// Add a lever linked to other interactable IDs, return the lever's ID
    pub fn add_lever(&mut self, position: Vec3, linked_ids: Vec<InteractableId>) -> InteractableId {
        let id = InteractableId(self.next_id);
        self.next_id += 1;

        self.world_state.insert(id, InteractableState::Lever {
            is_on: false,
            linked_ids,
        });

        self.interactables.push(Interactable {
            kind: InteractableKind::Lever { id },
            position,
            interaction_radius: 3.0,
            prompt: "Pull Lever".to_string(),
        });

        id
    }

    /// Add a button, return its ID
    pub fn add_button(&mut self, position: Vec3) -> InteractableId {
        let id = InteractableId(self.next_id);
        self.next_id += 1;

        self.world_state.insert(id, InteractableState::Button {
            is_pressed: false,
        });

        self.interactables.push(Interactable {
            kind: InteractableKind::Button { id },
            position,
            interaction_radius: 2.5,
            prompt: "Press Button".to_string(),
        });

        id
    }

    /// Add a container with items, return its ID
    pub fn add_container(&mut self, position: Vec3, items: Vec<String>) -> InteractableId {
        let id = InteractableId(self.next_id);
        self.next_id += 1;

        self.world_state.insert(id, InteractableState::Container {
            is_open: false,
            items,
        });

        self.interactables.push(Interactable {
            kind: InteractableKind::Container { id },
            position,
            interaction_radius: 2.5,
            prompt: "Open Container".to_string(),
        });

        id
    }

    /// Add a ladder (stateless)
    pub fn add_ladder(&mut self, position: Vec3, height: f32, direction: Vec3) {
        self.interactables.push(Interactable {
            kind: InteractableKind::Ladder { height, direction },
            position,
            interaction_radius: 2.5,
            prompt: "Climb".to_string(),
        });
    }

    /// Update focus detection based on player position and facing direction.
    ///
    /// Finds the nearest interactable that is:
    /// - Within interaction_radius distance
    /// - Roughly in front of the player (dot product > 0.5 with forward vector)
    pub fn update(&mut self, player_pos: Vec3, player_forward: Vec3) {
        let mut best_index: Option<usize> = None;
        let mut best_distance = f32::MAX;

        for (i, interactable) in self.interactables.iter().enumerate() {
            let to_target = interactable.position - player_pos;
            let distance = to_target.length();

            // Check distance
            if distance > interactable.interaction_radius {
                continue;
            }

            // Check facing direction (must be roughly looking at it)
            if distance > 0.1 {
                let dir_to_target = to_target / distance;
                // Use horizontal direction only (ignore Y) for facing check
                let horizontal_forward = Vec3::new(player_forward.x, 0.0, player_forward.z)
                    .normalize_or_zero();
                let horizontal_to_target = Vec3::new(dir_to_target.x, 0.0, dir_to_target.z)
                    .normalize_or_zero();
                let dot = horizontal_forward.dot(horizontal_to_target);
                if dot < 0.5 {
                    continue;
                }
            }

            // Track nearest valid target
            if distance < best_distance {
                best_distance = distance;
                best_index = Some(i);
            }
        }

        self.focused = best_index;

        // Update prompts for stateful interactables to reflect current state
        self.update_prompts();
    }

    /// Get the currently focused interactable, if any
    pub fn focused(&self) -> Option<&Interactable> {
        self.focused.and_then(|i| self.interactables.get(i))
    }

    /// Interact with the currently focused object.
    /// Returns the interaction result if something was focused.
    pub fn interact(&mut self) -> Option<InteractionResult> {
        let index = self.focused?;
        let interactable = &self.interactables[index];

        let result = match &interactable.kind {
            InteractableKind::Sign { text } => InteractionResult::ShowText(text.clone()),
            InteractableKind::TimePortal { target_year } => {
                InteractionResult::ChangeTimePeriod(*target_year)
            }
            InteractableKind::Pickup { item_name } => {
                InteractionResult::PickupItem(item_name.clone())
            }
            InteractableKind::Npc { npc_id } => {
                InteractionResult::TalkToNpc(*npc_id)
            }
            InteractableKind::Door { id } => {
                self.interact_door(*id)
            }
            InteractableKind::Lever { id } => {
                self.interact_lever(*id)
            }
            InteractableKind::Button { id } => {
                self.interact_button(*id)
            }
            InteractableKind::Container { id } => {
                self.interact_container(*id)
            }
            InteractableKind::Ladder { height, direction } => {
                InteractionResult::StartClimbing {
                    height: *height,
                    direction: *direction,
                }
            }
        };

        // Pickups are consumed on interaction
        if matches!(self.interactables[index].kind, InteractableKind::Pickup { .. }) {
            self.interactables.remove(index);
            self.focused = None;
        }

        Some(result)
    }

    /// Trigger linked effects (e.g., lever unlocks doors)
    pub fn trigger_linked(&mut self, linked_ids: &[InteractableId]) {
        for linked_id in linked_ids {
            if let Some(state) = self.world_state.get_mut(linked_id) {
                match state {
                    InteractableState::Door { is_locked, .. } => {
                        *is_locked = !*is_locked;
                    }
                    InteractableState::Button { is_pressed } => {
                        *is_pressed = !*is_pressed;
                    }
                    _ => {}
                }
            }
        }
    }

    /// Save all interaction states
    pub fn save_states(&self) -> InteractionSaveData {
        InteractionSaveData {
            states: self.world_state.iter().map(|(k, v)| (k.0, v.clone())).collect(),
            next_id: self.next_id,
        }
    }

    /// Load interaction states from save data
    pub fn load_states(&mut self, data: InteractionSaveData) {
        self.world_state = data.states.into_iter()
            .map(|(k, v)| (InteractableId(k), v))
            .collect();
        self.next_id = data.next_id;
        self.update_prompts();
    }

    // --- Private helpers ---

    fn interact_door(&mut self, id: InteractableId) -> InteractionResult {
        if let Some(InteractableState::Door { is_open, is_locked }) = self.world_state.get_mut(&id) {
            if *is_locked {
                return InteractionResult::Locked;
            }
            *is_open = !*is_open;
            InteractionResult::ToggleDoor { id, now_open: *is_open }
        } else {
            InteractionResult::Locked
        }
    }

    fn interact_lever(&mut self, id: InteractableId) -> InteractionResult {
        if let Some(InteractableState::Lever { is_on, linked_ids }) = self.world_state.get_mut(&id) {
            *is_on = !*is_on;
            let now_on = *is_on;
            let linked = linked_ids.clone();
            InteractionResult::ToggleLever { id, now_on, linked }
        } else {
            InteractionResult::Locked
        }
    }

    fn interact_button(&mut self, id: InteractableId) -> InteractionResult {
        if let Some(InteractableState::Button { is_pressed }) = self.world_state.get_mut(&id) {
            *is_pressed = true;
            InteractionResult::PressButton { id }
        } else {
            InteractionResult::Locked
        }
    }

    fn interact_container(&mut self, id: InteractableId) -> InteractionResult {
        if let Some(InteractableState::Container { is_open, items }) = self.world_state.get_mut(&id) {
            *is_open = true;
            let contained = std::mem::take(items);
            InteractionResult::OpenContainer { id, items: contained }
        } else {
            InteractionResult::Locked
        }
    }

    /// Update prompt text for all stateful interactables to reflect current state
    fn update_prompts(&mut self) {
        for interactable in &mut self.interactables {
            match &interactable.kind {
                InteractableKind::Door { id } => {
                    if let Some(InteractableState::Door { is_open, is_locked }) = self.world_state.get(id) {
                        interactable.prompt = if *is_locked {
                            "Locked".to_string()
                        } else if *is_open {
                            "Close Door".to_string()
                        } else {
                            "Open Door".to_string()
                        };
                    }
                }
                InteractableKind::Lever { id } => {
                    if let Some(InteractableState::Lever { is_on, .. }) = self.world_state.get(id) {
                        interactable.prompt = if *is_on {
                            "Pull Lever (On)".to_string()
                        } else {
                            "Pull Lever".to_string()
                        };
                    }
                }
                InteractableKind::Container { id } => {
                    if let Some(InteractableState::Container { is_open, items, .. }) = self.world_state.get(id) {
                        interactable.prompt = if *is_open && items.is_empty() {
                            "Empty Container".to_string()
                        } else if *is_open {
                            "Search Container".to_string()
                        } else {
                            "Open Container".to_string()
                        };
                    }
                }
                _ => {}
            }
        }
    }
}

impl Default for InteractionSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focus_in_range_and_facing() {
        let mut system = InteractionSystem::new();
        system.add(Interactable::sign(
            Vec3::new(0.0, 0.0, -2.0),
            "Hello!",
        ));

        // Player at origin, facing -Z (toward the sign)
        system.update(Vec3::ZERO, Vec3::new(0.0, 0.0, -1.0));
        assert!(system.focused().is_some());
        assert_eq!(system.focused().unwrap().prompt, "Read");
    }

    #[test]
    fn test_no_focus_when_out_of_range() {
        let mut system = InteractionSystem::new();
        system.add(Interactable::sign(
            Vec3::new(0.0, 0.0, -10.0),
            "Far away",
        ));

        system.update(Vec3::ZERO, Vec3::new(0.0, 0.0, -1.0));
        assert!(system.focused().is_none());
    }

    #[test]
    fn test_no_focus_when_facing_away() {
        let mut system = InteractionSystem::new();
        system.add(Interactable::sign(
            Vec3::new(0.0, 0.0, -2.0),
            "Behind you",
        ));

        // Facing +Z (away from sign)
        system.update(Vec3::ZERO, Vec3::new(0.0, 0.0, 1.0));
        assert!(system.focused().is_none());
    }

    #[test]
    fn test_interact_returns_result() {
        let mut system = InteractionSystem::new();
        system.add(Interactable::sign(
            Vec3::new(0.0, 0.0, -2.0),
            "Test text",
        ));

        system.update(Vec3::ZERO, Vec3::new(0.0, 0.0, -1.0));

        let result = system.interact();
        assert!(result.is_some());
        match result.unwrap() {
            InteractionResult::ShowText(text) => assert_eq!(text, "Test text"),
            _ => panic!("Expected ShowText result"),
        }
    }

    #[test]
    fn test_time_portal_interaction() {
        let mut system = InteractionSystem::new();
        system.add(Interactable::time_portal(
            Vec3::new(0.0, 0.0, -2.0),
            3025,
            "Year 3025",
        ));

        system.update(Vec3::ZERO, Vec3::new(0.0, 0.0, -1.0));

        let result = system.interact();
        assert!(result.is_some());
        match result.unwrap() {
            InteractionResult::ChangeTimePeriod(year) => assert_eq!(year, 3025),
            _ => panic!("Expected ChangeTimePeriod result"),
        }
    }

    #[test]
    fn test_pickup_consumed_on_interact() {
        let mut system = InteractionSystem::new();
        system.add(Interactable::pickup(
            Vec3::new(0.0, 0.0, -1.0),
            "Gem",
        ));

        system.update(Vec3::ZERO, Vec3::new(0.0, 0.0, -1.0));
        assert!(system.focused().is_some());

        let result = system.interact();
        assert!(matches!(result, Some(InteractionResult::PickupItem(_))));

        // Should be removed now
        assert_eq!(system.count(), 0);
    }

    #[test]
    fn test_door_open_close() {
        let mut system = InteractionSystem::new();
        let door_id = system.add_door(Vec3::new(0.0, 0.0, -2.0), false);

        system.update(Vec3::ZERO, Vec3::new(0.0, 0.0, -1.0));
        assert!(system.focused().is_some());

        // Open the door
        let result = system.interact().unwrap();
        match result {
            InteractionResult::ToggleDoor { id, now_open } => {
                assert_eq!(id, door_id);
                assert!(now_open);
            }
            _ => panic!("Expected ToggleDoor"),
        }

        // Close the door
        system.update(Vec3::ZERO, Vec3::new(0.0, 0.0, -1.0));
        let result = system.interact().unwrap();
        match result {
            InteractionResult::ToggleDoor { now_open, .. } => {
                assert!(!now_open);
            }
            _ => panic!("Expected ToggleDoor"),
        }
    }

    #[test]
    fn test_locked_door() {
        let mut system = InteractionSystem::new();
        system.add_door(Vec3::new(0.0, 0.0, -2.0), true);

        system.update(Vec3::ZERO, Vec3::new(0.0, 0.0, -1.0));
        let result = system.interact().unwrap();
        assert!(matches!(result, InteractionResult::Locked));
    }

    #[test]
    fn test_lever_unlocks_door() {
        let mut system = InteractionSystem::new();
        let door_id = system.add_door(Vec3::new(5.0, 0.0, 0.0), true);
        system.add_lever(Vec3::new(0.0, 0.0, -2.0), vec![door_id]);

        // Focus on lever
        system.update(Vec3::ZERO, Vec3::new(0.0, 0.0, -1.0));
        let result = system.interact().unwrap();

        match result {
            InteractionResult::ToggleLever { now_on, linked, .. } => {
                assert!(now_on);
                system.trigger_linked(&linked);
            }
            _ => panic!("Expected ToggleLever"),
        }

        // Door should now be unlocked
        if let Some(InteractableState::Door { is_locked, .. }) = system.world_state.get(&door_id) {
            assert!(!is_locked);
        } else {
            panic!("Door state not found");
        }
    }

    #[test]
    fn test_container_items() {
        let mut system = InteractionSystem::new();
        system.add_container(
            Vec3::new(0.0, 0.0, -2.0),
            vec!["Gold Coin".to_string(), "Health Potion".to_string()],
        );

        system.update(Vec3::ZERO, Vec3::new(0.0, 0.0, -1.0));
        let result = system.interact().unwrap();

        match result {
            InteractionResult::OpenContainer { items, .. } => {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0], "Gold Coin");
                assert_eq!(items[1], "Health Potion");
            }
            _ => panic!("Expected OpenContainer"),
        }

        // Container should now be empty
        system.update(Vec3::ZERO, Vec3::new(0.0, 0.0, -1.0));
        let result = system.interact().unwrap();
        match result {
            InteractionResult::OpenContainer { items, .. } => {
                assert!(items.is_empty());
            }
            _ => panic!("Expected OpenContainer"),
        }
    }

    #[test]
    fn test_ladder() {
        let mut system = InteractionSystem::new();
        system.add_ladder(Vec3::new(0.0, 0.0, -2.0), 5.0, Vec3::Y);

        system.update(Vec3::ZERO, Vec3::new(0.0, 0.0, -1.0));
        let result = system.interact().unwrap();

        match result {
            InteractionResult::StartClimbing { height, direction } => {
                assert_eq!(height, 5.0);
                assert_eq!(direction, Vec3::Y);
            }
            _ => panic!("Expected StartClimbing"),
        }
    }

    #[test]
    fn test_button() {
        let mut system = InteractionSystem::new();
        system.add_button(Vec3::new(0.0, 0.0, -2.0));

        system.update(Vec3::ZERO, Vec3::new(0.0, 0.0, -1.0));
        let result = system.interact().unwrap();

        assert!(matches!(result, InteractionResult::PressButton { .. }));
    }

    #[test]
    fn test_save_load_states() {
        let mut system = InteractionSystem::new();
        let door_id = system.add_door(Vec3::new(0.0, 0.0, -2.0), false);

        // Open the door
        system.update(Vec3::ZERO, Vec3::new(0.0, 0.0, -1.0));
        system.interact();

        // Save
        let save_data = system.save_states();

        // Create a new system and load
        let mut system2 = InteractionSystem::new();
        system2.add_door(Vec3::new(0.0, 0.0, -2.0), false);
        system2.load_states(save_data);

        // Door should be open
        if let Some(InteractableState::Door { is_open, .. }) = system2.world_state.get(&door_id) {
            assert!(*is_open);
        } else {
            panic!("Door state not found after load");
        }
    }
}
