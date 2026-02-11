//! Input system with action-based mapping
//!
//! Provides an abstraction layer between raw input events and game actions.

use std::collections::{HashMap, HashSet};

use glam::Vec2;
use serde::{Deserialize, Serialize};
use winit::event::{ElementState, MouseButton, MouseScrollDelta};
use winit::keyboard::{KeyCode, PhysicalKey};

/// Game actions that can be triggered by input
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InputAction {
    /// Move forward (W by default)
    MoveForward,
    /// Move backward (S by default)
    MoveBackward,
    /// Move left (A by default)
    MoveLeft,
    /// Move right (D by default)
    MoveRight,
    /// Jump (Space by default)
    Jump,
    /// Sprint modifier (Shift by default)
    Sprint,
    /// Zoom in (scroll up)
    ZoomIn,
    /// Zoom out (scroll down)
    ZoomOut,
    /// Pause/unpause (Escape by default)
    Pause,
    /// Interact with objects (E by default)
    Interact,
    /// Quick save (F5 by default)
    QuickSave,
    /// Quick load (F9 by default)
    QuickLoad,
    /// Attack (Left mouse button by default)
    Attack,
    /// Heavy attack (Right mouse button by default)
    HeavyAttack,
    /// Skill slot 1 (1 key by default)
    Skill1,
    /// Skill slot 2 (2 key by default)
    Skill2,
    /// Skill slot 3 (3 key by default)
    Skill3,
    /// Skill slot 4 (4 key by default)
    Skill4,
    /// Rune compose mode (R key by default)
    RuneCompose,
    /// Dodge (Left Ctrl by default)
    Dodge,
    /// Toggle inventory (Tab by default)
    Inventory,
}

/// Current state of all inputs for a frame
#[derive(Debug, Clone, Default)]
pub struct InputState {
    /// Actions currently held down
    pub held: HashSet<InputAction>,
    /// Actions that were just pressed this frame
    pub just_pressed: HashSet<InputAction>,
    /// Actions that were just released this frame
    pub just_released: HashSet<InputAction>,
    /// Mouse movement delta for this frame
    pub mouse_delta: Vec2,
    /// Scroll wheel delta for this frame
    pub scroll_delta: f32,
    /// Whether the cursor is captured (invisible, locked)
    pub cursor_captured: bool,
}

impl InputState {
    /// Create a new empty input state
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if an action is currently held
    pub fn is_held(&self, action: InputAction) -> bool {
        self.held.contains(&action)
    }

    /// Check if an action was just pressed this frame
    pub fn is_just_pressed(&self, action: InputAction) -> bool {
        self.just_pressed.contains(&action)
    }

    /// Check if an action was just released this frame
    pub fn is_just_released(&self, action: InputAction) -> bool {
        self.just_released.contains(&action)
    }

    /// Clear frame-specific data (call at end of frame)
    pub fn clear_frame(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
        self.mouse_delta = Vec2::ZERO;
        self.scroll_delta = 0.0;
    }

    /// Clear all input state
    pub fn clear_all(&mut self) {
        self.held.clear();
        self.just_pressed.clear();
        self.just_released.clear();
        self.mouse_delta = Vec2::ZERO;
        self.scroll_delta = 0.0;
    }
}

/// Binding of a physical key to an action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputBinding {
    /// Keyboard key
    Key(KeyCode),
    /// Mouse button
    Mouse(u32), // 0 = left, 1 = right, 2 = middle
}

impl From<KeyCode> for InputBinding {
    fn from(key: KeyCode) -> Self {
        Self::Key(key)
    }
}

/// Maps physical inputs to game actions
#[derive(Debug, Clone)]
pub struct InputBindings {
    /// Key/button to action mappings
    bindings: HashMap<InputBinding, InputAction>,
    /// Reverse lookup: action to all bindings
    reverse: HashMap<InputAction, Vec<InputBinding>>,
}

impl Default for InputBindings {
    fn default() -> Self {
        let mut bindings = Self {
            bindings: HashMap::new(),
            reverse: HashMap::new(),
        };

        // Default WASD bindings
        bindings.bind(KeyCode::KeyW, InputAction::MoveForward);
        bindings.bind(KeyCode::KeyS, InputAction::MoveBackward);
        bindings.bind(KeyCode::KeyA, InputAction::MoveLeft);
        bindings.bind(KeyCode::KeyD, InputAction::MoveRight);

        // Arrow keys as alternative
        bindings.bind(KeyCode::ArrowUp, InputAction::MoveForward);
        bindings.bind(KeyCode::ArrowDown, InputAction::MoveBackward);
        bindings.bind(KeyCode::ArrowLeft, InputAction::MoveLeft);
        bindings.bind(KeyCode::ArrowRight, InputAction::MoveRight);

        // Actions
        bindings.bind(KeyCode::Space, InputAction::Jump);
        bindings.bind(KeyCode::ShiftLeft, InputAction::Sprint);
        bindings.bind(KeyCode::ShiftRight, InputAction::Sprint);
        bindings.bind(KeyCode::Escape, InputAction::Pause);
        bindings.bind(KeyCode::KeyE, InputAction::Interact);
        bindings.bind(KeyCode::F5, InputAction::QuickSave);
        bindings.bind(KeyCode::F9, InputAction::QuickLoad);

        // Combat
        bindings.bind_mouse(0, InputAction::Attack); // Left mouse button
        bindings.bind_mouse(1, InputAction::HeavyAttack); // Right mouse button
        bindings.bind(KeyCode::Digit1, InputAction::Skill1);
        bindings.bind(KeyCode::Digit2, InputAction::Skill2);
        bindings.bind(KeyCode::Digit3, InputAction::Skill3);
        bindings.bind(KeyCode::Digit4, InputAction::Skill4);
        bindings.bind(KeyCode::KeyR, InputAction::RuneCompose);
        bindings.bind(KeyCode::ControlLeft, InputAction::Dodge);
        bindings.bind(KeyCode::Tab, InputAction::Inventory);

        bindings
    }
}

impl InputBindings {
    /// Create new input bindings with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Bind a key to an action
    pub fn bind(&mut self, key: KeyCode, action: InputAction) {
        let binding = InputBinding::Key(key);
        self.bindings.insert(binding, action);
        self.reverse.entry(action).or_default().push(binding);
    }

    /// Bind a mouse button to an action
    pub fn bind_mouse(&mut self, button: u32, action: InputAction) {
        let binding = InputBinding::Mouse(button);
        self.bindings.insert(binding, action);
        self.reverse.entry(action).or_default().push(binding);
    }

    /// Unbind a key
    pub fn unbind(&mut self, key: KeyCode) {
        let binding = InputBinding::Key(key);
        if let Some(action) = self.bindings.remove(&binding) {
            if let Some(bindings) = self.reverse.get_mut(&action) {
                bindings.retain(|b| *b != binding);
            }
        }
    }

    /// Get the action for a binding, if any
    pub fn get_action(&self, binding: &InputBinding) -> Option<InputAction> {
        self.bindings.get(binding).copied()
    }

    /// Get the action for a key, if any
    pub fn get_key_action(&self, key: KeyCode) -> Option<InputAction> {
        self.get_action(&InputBinding::Key(key))
    }

    /// Rebuild the reverse lookup table (call after deserialization)
    pub fn rebuild_reverse(&mut self) {
        self.reverse.clear();
        for (binding, action) in &self.bindings {
            self.reverse.entry(*action).or_default().push(*binding);
        }
    }
}

/// Input handler that processes raw events and updates state
#[derive(Debug)]
pub struct InputHandler {
    /// Current input state
    pub state: InputState,
    /// Input bindings
    pub bindings: InputBindings,
    /// Mouse sensitivity multiplier
    pub mouse_sensitivity: f32,
    /// Invert Y axis
    pub invert_y: bool,
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl InputHandler {
    /// Create a new input handler with default bindings
    pub fn new() -> Self {
        Self {
            state: InputState::new(),
            bindings: InputBindings::default(),
            mouse_sensitivity: 1.0,
            invert_y: false,
        }
    }

    /// Handle a keyboard event
    pub fn handle_keyboard(&mut self, physical_key: PhysicalKey, element_state: ElementState) {
        if let PhysicalKey::Code(key_code) = physical_key {
            if let Some(action) = self.bindings.get_key_action(key_code) {
                match element_state {
                    ElementState::Pressed => {
                        if !self.state.held.contains(&action) {
                            self.state.just_pressed.insert(action);
                        }
                        self.state.held.insert(action);
                    }
                    ElementState::Released => {
                        self.state.held.remove(&action);
                        self.state.just_released.insert(action);
                    }
                }
            }
        }
    }

    /// Handle a mouse button event
    pub fn handle_mouse_button(&mut self, button: MouseButton, state: ElementState) {
        let button_id = match button {
            MouseButton::Left => 0,
            MouseButton::Right => 1,
            MouseButton::Middle => 2,
            MouseButton::Back => 3,
            MouseButton::Forward => 4,
            MouseButton::Other(id) => id as u32,
        };

        let binding = InputBinding::Mouse(button_id);
        if let Some(action) = self.bindings.get_action(&binding) {
            match state {
                ElementState::Pressed => {
                    if !self.state.held.contains(&action) {
                        self.state.just_pressed.insert(action);
                    }
                    self.state.held.insert(action);
                }
                ElementState::Released => {
                    self.state.held.remove(&action);
                    self.state.just_released.insert(action);
                }
            }
        }
    }

    /// Handle mouse movement
    pub fn handle_mouse_motion(&mut self, delta: (f64, f64)) {
        if self.state.cursor_captured {
            let y_mult = if self.invert_y { -1.0 } else { 1.0 };
            self.state.mouse_delta += Vec2::new(
                delta.0 as f32 * self.mouse_sensitivity,
                delta.1 as f32 * self.mouse_sensitivity * y_mult,
            );
        }
    }

    /// Handle scroll wheel
    pub fn handle_scroll(&mut self, delta: MouseScrollDelta) {
        let scroll = match delta {
            MouseScrollDelta::LineDelta(_, y) => y,
            MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 120.0,
        };

        self.state.scroll_delta += scroll;

        // Also trigger zoom actions
        if scroll > 0.0 {
            self.state.just_pressed.insert(InputAction::ZoomIn);
        } else if scroll < 0.0 {
            self.state.just_pressed.insert(InputAction::ZoomOut);
        }
    }

    /// Clear frame-specific input data
    pub fn end_frame(&mut self) {
        self.state.clear_frame();
    }

    /// Set cursor capture state
    pub fn set_cursor_captured(&mut self, captured: bool) {
        self.state.cursor_captured = captured;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_bindings() {
        let bindings = InputBindings::default();
        assert_eq!(
            bindings.get_key_action(KeyCode::KeyW),
            Some(InputAction::MoveForward)
        );
        assert_eq!(
            bindings.get_key_action(KeyCode::Space),
            Some(InputAction::Jump)
        );
    }

    #[test]
    fn test_input_state() {
        let mut state = InputState::new();
        state.held.insert(InputAction::MoveForward);
        state.just_pressed.insert(InputAction::Jump);

        assert!(state.is_held(InputAction::MoveForward));
        assert!(state.is_just_pressed(InputAction::Jump));
        assert!(!state.is_held(InputAction::Sprint));

        state.clear_frame();
        assert!(state.is_held(InputAction::MoveForward));
        assert!(!state.is_just_pressed(InputAction::Jump));
    }
}
