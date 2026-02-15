//! UI module for Infinite
//!
//! Contains all egui-based UI screens and menus.

pub mod admin;
mod character_creator;
mod inventory_menu;
mod loading_screen;
mod login_menu;
mod main_menu;
mod pause_menu;
mod save_load_menu;
mod settings_menu;

pub use admin::AdminPanel;
pub use character_creator::CharacterCreator;
pub use inventory_menu::{InventoryAction, InventoryMenu};
pub use loading_screen::LoadingScreen;
pub use login_menu::LoginMenu;
pub use main_menu::MainMenu;
pub use pause_menu::PauseMenu;
pub use save_load_menu::{SaveLoadAction, SaveLoadMenu};
pub use settings_menu::SettingsMenu;
