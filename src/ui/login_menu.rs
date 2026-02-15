//! Login menu UI — requires a Pixygon account to proceed

use egui::{Align, Color32, FontId, Layout, RichText, Ui, Vec2};

use infinite_integration::{IntegrationClient, PendingRequest};
use infinite_integration::types::AuthResponse;

use crate::state::{ApplicationState, StateTransition};

/// Login menu state
pub struct LoginMenu {
    username: String,
    password: String,
    error_message: Option<String>,
    pending_login: Option<PendingRequest<AuthResponse>>,
    status_text: Option<String>,
}

impl LoginMenu {
    pub fn new() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            error_message: None,
            pending_login: None,
            status_text: None,
        }
    }

    /// Render the login menu and return any state transition
    pub fn render(
        &mut self,
        ui: &mut Ui,
        integration_client: Option<&IntegrationClient>,
    ) -> StateTransition {
        let mut transition = StateTransition::None;

        // If no integration client, skip login entirely
        let Some(client) = integration_client else {
            return StateTransition::Replace(ApplicationState::MainMenu);
        };

        // Poll pending login
        if let Some(pending) = &self.pending_login {
            if let Some(result) = pending.try_recv() {
                match result {
                    Ok(_auth) => {
                        self.pending_login = None;
                        self.error_message = None;
                        self.status_text = None;
                        return StateTransition::Replace(ApplicationState::MainMenu);
                    }
                    Err(e) => {
                        self.error_message = Some(format!("{}", e));
                        self.status_text = None;
                        self.pending_login = None;
                    }
                }
            }
        }

        let available = ui.available_size();

        ui.vertical_centered(|ui| {
            ui.add_space(available.y * 0.2);

            // Title
            ui.label(
                RichText::new("INFINITE")
                    .font(FontId::proportional(72.0))
                    .color(Color32::from_rgb(200, 200, 255)),
            );

            ui.add_space(40.0);

            ui.label(
                RichText::new("Sign in with your Pixygon account")
                    .font(FontId::proportional(16.0))
                    .color(Color32::from_rgb(150, 150, 180)),
            );

            ui.add_space(30.0);

            // Login form in a centered frame
            let form_width = 300.0;
            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                ui.set_max_width(form_width);

                // Username
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Username:")
                            .font(FontId::proportional(14.0))
                            .color(Color32::from_rgb(180, 180, 200)),
                    );
                });
                let username_response = ui.add(
                    egui::TextEdit::singleline(&mut self.username)
                        .desired_width(form_width)
                        .hint_text("Enter username...")
                        .font(FontId::proportional(16.0)),
                );

                ui.add_space(15.0);

                // Password
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Password:")
                            .font(FontId::proportional(14.0))
                            .color(Color32::from_rgb(180, 180, 200)),
                    );
                });
                let password_response = ui.add(
                    egui::TextEdit::singleline(&mut self.password)
                        .desired_width(form_width)
                        .password(true)
                        .hint_text("Enter password...")
                        .font(FontId::proportional(16.0)),
                );

                ui.add_space(20.0);

                // Error message
                if let Some(error) = &self.error_message {
                    ui.label(
                        RichText::new(error)
                            .font(FontId::proportional(14.0))
                            .color(Color32::from_rgb(255, 100, 100)),
                    );
                    ui.add_space(10.0);
                }

                // Status text (logging in...)
                if let Some(status) = &self.status_text {
                    ui.label(
                        RichText::new(status)
                            .font(FontId::proportional(14.0))
                            .color(Color32::from_rgb(150, 200, 255)),
                    );
                    ui.add_space(10.0);
                }

                // Login button
                let is_logging_in = self.pending_login.is_some();
                let can_submit = !is_logging_in
                    && !self.username.trim().is_empty()
                    && !self.password.is_empty();

                let button_label = if is_logging_in { "Logging in..." } else { "Log In" };
                let button_color = if can_submit {
                    Color32::from_rgb(60, 80, 130)
                } else {
                    Color32::from_rgb(40, 40, 55)
                };

                let button = egui::Button::new(
                    RichText::new(button_label)
                        .font(FontId::proportional(18.0))
                        .color(Color32::from_rgb(220, 220, 240)),
                )
                .min_size(Vec2::new(form_width, 40.0))
                .fill(button_color)
                .stroke(egui::Stroke::new(1.0, Color32::from_rgb(80, 80, 100)));

                let enter_pressed = username_response.lost_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::Enter))
                    || password_response.lost_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter));

                if (ui.add(button).clicked() || enter_pressed) && can_submit {
                    self.error_message = None;
                    self.status_text = Some("Logging in...".to_string());
                    self.pending_login = Some(client.login(
                        self.username.trim().to_string(),
                        self.password.clone(),
                    ));
                }

                ui.add_space(20.0);

                // Quit button
                let quit_button = egui::Button::new(
                    RichText::new("Quit")
                        .font(FontId::proportional(14.0))
                        .color(Color32::from_rgb(180, 180, 200)),
                )
                .min_size(Vec2::new(100.0, 30.0))
                .fill(Color32::from_rgb(50, 50, 70))
                .stroke(egui::Stroke::new(1.0, Color32::from_rgb(80, 80, 100)));

                if ui.add(quit_button).clicked() {
                    transition = StateTransition::Replace(ApplicationState::Exiting);
                }
            });
        });

        transition
    }
}

impl Default for LoginMenu {
    fn default() -> Self {
        Self::new()
    }
}
