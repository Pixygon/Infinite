//! Loading screen UI

use egui::{Color32, FontId, RichText, Ui};

use crate::state::LoadingPhase;

/// Loading screen renderer
pub struct LoadingScreen {
    /// Current animated progress (smoothly interpolates to target)
    animated_progress: f32,
    /// Logo fade-in animation (0.0 to 1.0)
    logo_alpha: f32,
    /// Time since loading started
    elapsed: f32,
}

impl LoadingScreen {
    pub fn new() -> Self {
        Self {
            animated_progress: 0.0,
            logo_alpha: 0.0,
            elapsed: 0.0,
        }
    }

    /// Update animations
    pub fn update(&mut self, delta: f32, target_progress: f32) {
        self.elapsed += delta;

        // Fade in logo over 1 second
        if self.logo_alpha < 1.0 {
            self.logo_alpha = (self.logo_alpha + delta * 2.0).min(1.0);
        }

        // Smooth progress bar animation
        let diff = target_progress - self.animated_progress;
        self.animated_progress += diff * delta * 3.0;
    }

    /// Render the loading screen
    pub fn render(&self, ui: &mut Ui, phase: &LoadingPhase) {
        let available = ui.available_size();

        ui.vertical_centered(|ui| {
            // Center content vertically
            ui.add_space(available.y * 0.35);

            // Logo text with fade-in
            let alpha = (self.logo_alpha * 255.0) as u8;
            ui.label(
                RichText::new("INFINITE")
                    .font(FontId::proportional(64.0))
                    .color(Color32::from_rgba_unmultiplied(200, 200, 255, alpha)),
            );

            ui.add_space(60.0);

            // Progress bar
            let bar_width = (available.x * 0.5).min(400.0);
            let bar_height = 8.0;

            let (rect, _response) = ui.allocate_exact_size(
                egui::vec2(bar_width, bar_height),
                egui::Sense::hover(),
            );

            if ui.is_rect_visible(rect) {
                let painter = ui.painter();

                // Background
                painter.rect_filled(
                    rect,
                    4.0,
                    Color32::from_rgb(40, 40, 50),
                );

                // Progress fill
                let fill_width = rect.width() * self.animated_progress;
                let fill_rect = egui::Rect::from_min_size(
                    rect.min,
                    egui::vec2(fill_width, rect.height()),
                );
                painter.rect_filled(
                    fill_rect,
                    4.0,
                    Color32::from_rgb(100, 150, 255),
                );
            }

            ui.add_space(20.0);

            // Phase description
            ui.label(
                RichText::new(phase.description())
                    .font(FontId::proportional(16.0))
                    .color(Color32::from_rgb(150, 150, 170)),
            );

            // Progress percentage
            ui.add_space(8.0);
            ui.label(
                RichText::new(format!("{:.0}%", self.animated_progress * 100.0))
                    .font(FontId::proportional(14.0))
                    .color(Color32::from_rgb(120, 120, 140)),
            );
        });
    }
}

impl Default for LoadingScreen {
    fn default() -> Self {
        Self::new()
    }
}
