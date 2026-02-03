//! Pause menu UI

use egui::{Align, Color32, FontId, Layout, RichText, Ui, Vec2};

use crate::state::{ApplicationState, StateTransition};

/// Pause menu renderer
pub struct PauseMenu;

impl PauseMenu {
    pub fn new() -> Self {
        Self
    }

    /// Render the pause menu and return any state transition
    pub fn render(&self, ui: &mut Ui) -> StateTransition {
        let mut transition = StateTransition::None;
        let available = ui.available_size();

        // Semi-transparent background overlay
        let painter = ui.painter();
        painter.rect_filled(
            ui.max_rect(),
            0.0,
            Color32::from_rgba_unmultiplied(0, 0, 0, 180),
        );

        ui.vertical_centered(|ui| {
            ui.add_space(available.y * 0.25);

            // Title
            ui.label(
                RichText::new("PAUSED")
                    .font(FontId::proportional(48.0))
                    .color(Color32::from_rgb(200, 200, 255)),
            );

            ui.add_space(50.0);

            let button_size = Vec2::new(180.0, 40.0);

            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                // Resume
                if pause_button(ui, "Resume", button_size) {
                    transition = StateTransition::Pop;
                }

                ui.add_space(10.0);

                // Settings
                if pause_button(ui, "Settings", button_size) {
                    transition = StateTransition::Push(ApplicationState::Settings {
                        return_to: Box::new(ApplicationState::Paused),
                    });
                }

                ui.add_space(10.0);

                // Main Menu
                if pause_button(ui, "Main Menu", button_size) {
                    transition = StateTransition::Replace(ApplicationState::MainMenu);
                }

                ui.add_space(10.0);

                // Quit
                if pause_button(ui, "Quit", button_size) {
                    transition = StateTransition::Replace(ApplicationState::Exiting);
                }
            });
        });

        transition
    }
}

impl Default for PauseMenu {
    fn default() -> Self {
        Self::new()
    }
}

fn pause_button(ui: &mut Ui, text: &str, size: Vec2) -> bool {
    ui.add(
        egui::Button::new(
            RichText::new(text)
                .font(FontId::proportional(18.0))
                .color(Color32::from_rgb(220, 220, 240)),
        )
        .min_size(size)
        .fill(Color32::from_rgba_unmultiplied(50, 50, 70, 220))
        .stroke(egui::Stroke::new(1.0, Color32::from_rgb(80, 80, 100)))
    ).clicked()
}
