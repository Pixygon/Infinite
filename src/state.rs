//! Application state machine for Infinite
//!
//! Manages the game's state transitions: loading, menus, gameplay, and exiting.

/// The current application state
#[derive(Debug, Clone)]
pub enum ApplicationState {
    /// Loading screen with progress phases
    Loading(LoadingPhase),
    /// Login screen (requires Pixygon account)
    Login,
    /// Main menu (title screen)
    MainMenu,
    /// Admin tools panel (items, stories)
    AdminTools,
    /// Character creation screen
    CharacterCreation,
    /// Settings menu with return target
    Settings { _return_to: Box<ApplicationState> },
    /// Game is paused
    Paused,
    /// Save/Load menu (accessed from pause)
    SaveLoad { is_saving: bool },
    /// Active gameplay
    Playing,
    /// Application is exiting
    Exiting,
}

impl Default for ApplicationState {
    fn default() -> Self {
        Self::Loading(LoadingPhase::VulkanInit)
    }
}

/// Loading phases with associated progress percentages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadingPhase {
    /// Initializing Vulkan instance and device (0%)
    VulkanInit,
    /// Compiling shaders (15%)
    Shaders,
    /// Loading core assets (35%)
    CoreAssets,
    /// Loading settings (60%)
    Settings,
    /// Initializing audio system (80%)
    Audio,
    /// Final setup (95%)
    Finalizing,
}

impl LoadingPhase {
    /// Get the progress percentage for this phase (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        match self {
            Self::VulkanInit => 0.0,
            Self::Shaders => 0.15,
            Self::CoreAssets => 0.35,
            Self::Settings => 0.60,
            Self::Audio => 0.80,
            Self::Finalizing => 0.95,
        }
    }

    /// Get a human-readable description of this phase
    pub fn description(&self) -> &'static str {
        match self {
            Self::VulkanInit => "Initializing graphics...",
            Self::Shaders => "Compiling shaders...",
            Self::CoreAssets => "Loading core assets...",
            Self::Settings => "Loading settings...",
            Self::Audio => "Initializing audio...",
            Self::Finalizing => "Finalizing...",
        }
    }

    /// Get the next phase, if any
    pub fn next(&self) -> Option<Self> {
        match self {
            Self::VulkanInit => Some(Self::Shaders),
            Self::Shaders => Some(Self::CoreAssets),
            Self::CoreAssets => Some(Self::Settings),
            Self::Settings => Some(Self::Audio),
            Self::Audio => Some(Self::Finalizing),
            Self::Finalizing => None,
        }
    }
}

/// State transition commands
#[derive(Debug, Clone)]
pub enum StateTransition {
    /// No transition
    None,
    /// Push a new state onto the stack (for menus that return)
    Push(ApplicationState),
    /// Pop the current state
    Pop,
    /// Replace the current state entirely
    Replace(ApplicationState),
}

impl Default for StateTransition {
    fn default() -> Self {
        Self::None
    }
}
