//! Main menu UI

use egui::{Align, Color32, FontId, Layout, RichText, Ui, Vec2};

use crate::state::{ApplicationState, StateTransition};

/// Main menu renderer
pub struct MainMenu {
    /// Whether a save file exists (for Continue button)
    has_save: bool,
}

impl MainMenu {
    pub fn new() -> Self {
        Self { has_save: false }
    }

    /// Set whether a save file exists
    #[allow(dead_code)]
    pub fn set_has_save(&mut self, has_save: bool) {
        self.has_save = has_save;
    }

    /// Render the main menu and return any state transition.
    /// `is_admin` controls whether the Admin Tools button is shown.
    /// `user_name` is shown as a greeting if logged in.
    pub fn render(&self, ui: &mut Ui, is_admin: bool, user_name: Option<&str>) -> StateTransition {
        let mut transition = StateTransition::None;
        let available = ui.available_size();

        ui.vertical_centered(|ui| {
            // Title
            ui.add_space(available.y * 0.2);
            ui.label(
                RichText::new("INFINITE")
                    .font(FontId::proportional(72.0))
                    .color(Color32::from_rgb(200, 200, 255)),
            );

            ui.add_space(80.0);

            // Menu buttons
            let button_width = 200.0;
            let button_height = 40.0;
            let button_size = Vec2::new(button_width, button_height);

            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                // New Game
                if menu_button(ui, "New Game", button_size) {
                    transition = StateTransition::Replace(ApplicationState::CharacterCreation);
                }

                ui.add_space(10.0);

                // Continue (only if save exists)
                if self.has_save {
                    if menu_button(ui, "Continue", button_size) {
                        // TODO: Load save then transition to Playing
                        transition = StateTransition::Replace(ApplicationState::Playing);
                    }
                    ui.add_space(10.0);
                }

                // Settings
                if menu_button(ui, "Settings", button_size) {
                    transition = StateTransition::Push(ApplicationState::Settings {
                        _return_to: Box::new(ApplicationState::MainMenu),
                    });
                }

                ui.add_space(10.0);

                // Admin Tools (only for admins)
                if is_admin {
                    if menu_button(ui, "Admin Tools", button_size) {
                        transition = StateTransition::Replace(ApplicationState::AdminTools);
                    }
                    ui.add_space(10.0);
                }

                // Quit
                if menu_button(ui, "Quit", button_size) {
                    transition = StateTransition::Replace(ApplicationState::Exiting);
                }
            });

            // Version info and user at bottom
            ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
                ui.add_space(20.0);
                ui.label(
                    RichText::new("v0.1.0")
                        .font(FontId::proportional(12.0))
                        .color(Color32::from_rgb(100, 100, 120)),
                );
                if let Some(name) = user_name {
                    ui.label(
                        RichText::new(format!("Logged in as {}", name))
                            .font(FontId::proportional(12.0))
                            .color(Color32::from_rgb(120, 140, 120)),
                    );
                }
            });
        });

        transition
    }
}

impl Default for MainMenu {
    fn default() -> Self {
        Self::new()
    }
}

/// Render a styled menu button
fn menu_button(ui: &mut Ui, text: &str, size: Vec2) -> bool {
    let button = egui::Button::new(
        RichText::new(text)
            .font(FontId::proportional(18.0))
            .color(Color32::from_rgb(220, 220, 240)),
    )
    .min_size(size)
    .fill(Color32::from_rgb(50, 50, 70))
    .stroke(egui::Stroke::new(1.0, Color32::from_rgb(80, 80, 100)));

    ui.add(button).clicked()
}
