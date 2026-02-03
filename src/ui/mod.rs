//! UI module for Infinite
//!
//! Contains all egui-based UI screens and menus.

mod character_creator;
mod loading_screen;
mod main_menu;
mod pause_menu;
mod settings_menu;

pub use character_creator::CharacterCreator;
pub use loading_screen::LoadingScreen;
pub use main_menu::MainMenu;
pub use pause_menu::PauseMenu;
pub use settings_menu::SettingsMenu;
