//! Settings menu UI

use egui::{Color32, FontId, RichText, Slider, Ui, Vec2};

use crate::settings::GameSettings;
use crate::state::StateTransition;

/// Settings tab selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    Video,
    Audio,
    Gameplay,
}

/// Settings menu renderer
pub struct SettingsMenu {
    /// Currently selected tab
    current_tab: SettingsTab,
    /// Working copy of settings (applied on Apply)
    working_settings: GameSettings,
    /// Original settings (for Reset)
    original_settings: GameSettings,
}

impl SettingsMenu {
    pub fn new(settings: GameSettings) -> Self {
        Self {
            current_tab: SettingsTab::Video,
            working_settings: settings.clone(),
            original_settings: settings,
        }
    }

    /// Reset working settings to original
    pub fn reset_to_original(&mut self) {
        self.working_settings = self.original_settings.clone();
    }

    /// Get the working settings (to apply)
    pub fn working_settings(&self) -> &GameSettings {
        &self.working_settings
    }

    /// Render the settings menu and return any state transition
    pub fn render(&mut self, ui: &mut Ui) -> (StateTransition, bool) {
        let mut transition = StateTransition::None;
        let mut should_apply = false;
        let available = ui.available_size();

        ui.vertical_centered(|ui| {
            // Title
            ui.add_space(30.0);
            ui.label(
                RichText::new("Settings")
                    .font(FontId::proportional(36.0))
                    .color(Color32::from_rgb(200, 200, 255)),
            );

            ui.add_space(30.0);

            // Tab bar
            ui.horizontal(|ui| {
                ui.add_space((available.x - 300.0) / 2.0);
                if tab_button(ui, "Video", self.current_tab == SettingsTab::Video) {
                    self.current_tab = SettingsTab::Video;
                }
                ui.add_space(10.0);
                if tab_button(ui, "Audio", self.current_tab == SettingsTab::Audio) {
                    self.current_tab = SettingsTab::Audio;
                }
                ui.add_space(10.0);
                if tab_button(ui, "Gameplay", self.current_tab == SettingsTab::Gameplay) {
                    self.current_tab = SettingsTab::Gameplay;
                }
            });

            ui.add_space(30.0);

            // Settings panel
            let panel_width = (available.x * 0.6).min(500.0);
            ui.allocate_ui(Vec2::new(panel_width, 300.0), |ui| {
                match self.current_tab {
                    SettingsTab::Video => self.render_video_settings(ui),
                    SettingsTab::Audio => self.render_audio_settings(ui),
                    SettingsTab::Gameplay => self.render_gameplay_settings(ui),
                }
            });

            ui.add_space(30.0);

            // Action buttons
            ui.horizontal(|ui| {
                ui.add_space((available.x - 330.0) / 2.0);

                if action_button(ui, "Back") {
                    transition = StateTransition::Pop;
                }

                ui.add_space(10.0);

                if action_button(ui, "Reset") {
                    self.reset_to_original();
                }

                ui.add_space(10.0);

                if action_button(ui, "Apply") {
                    should_apply = true;
                }
            });
        });

        (transition, should_apply)
    }

    fn render_video_settings(&mut self, ui: &mut Ui) {
        let video = &mut self.working_settings.video;

        ui.horizontal(|ui| {
            ui.label("Resolution:");
            ui.add_space(20.0);
            egui::ComboBox::from_id_salt("resolution")
                .selected_text(format!("{}x{}", video.width, video.height))
                .show_ui(ui, |ui| {
                    for (w, h) in [(1280, 720), (1920, 1080), (2560, 1440), (3840, 2160)] {
                        if ui.selectable_label(
                            video.width == w && video.height == h,
                            format!("{}x{}", w, h),
                        ).clicked() {
                            video.width = w;
                            video.height = h;
                        }
                    }
                });
        });

        ui.add_space(15.0);
        ui.checkbox(&mut video.fullscreen, "Fullscreen");

        ui.add_space(15.0);
        ui.checkbox(&mut video.vsync, "VSync");

        ui.add_space(15.0);
        ui.horizontal(|ui| {
            ui.label("Ray Tracing:");
            ui.add_space(20.0);
            egui::ComboBox::from_id_salt("ray_tracing")
                .selected_text(video.ray_tracing_quality_name())
                .show_ui(ui, |ui| {
                    for (i, name) in ["Off", "Low", "Medium", "High", "Ultra"].iter().enumerate() {
                        if ui.selectable_label(
                            video.ray_tracing_quality == i as u8,
                            *name,
                        ).clicked() {
                            video.ray_tracing_quality = i as u8;
                        }
                    }
                });
        });

        ui.add_space(15.0);
        ui.horizontal(|ui| {
            ui.label("Field of View:");
            ui.add(Slider::new(&mut video.fov, 60.0..=120.0).suffix(""));
        });
    }

    fn render_audio_settings(&mut self, ui: &mut Ui) {
        let audio = &mut self.working_settings.audio;

        ui.horizontal(|ui| {
            ui.label("Master Volume:");
            ui.add(Slider::new(&mut audio.master, 0.0..=1.0).show_value(false));
            ui.label(format!("{:.0}%", audio.master * 100.0));
        });

        ui.add_space(15.0);
        ui.horizontal(|ui| {
            ui.label("Music Volume:");
            ui.add(Slider::new(&mut audio.music, 0.0..=1.0).show_value(false));
            ui.label(format!("{:.0}%", audio.music * 100.0));
        });

        ui.add_space(15.0);
        ui.horizontal(|ui| {
            ui.label("SFX Volume:");
            ui.add(Slider::new(&mut audio.sfx, 0.0..=1.0).show_value(false));
            ui.label(format!("{:.0}%", audio.sfx * 100.0));
        });

        ui.add_space(15.0);
        ui.horizontal(|ui| {
            ui.label("Voice Volume:");
            ui.add(Slider::new(&mut audio.voice, 0.0..=1.0).show_value(false));
            ui.label(format!("{:.0}%", audio.voice * 100.0));
        });
    }

    fn render_gameplay_settings(&mut self, ui: &mut Ui) {
        let gameplay = &mut self.working_settings.gameplay;

        ui.horizontal(|ui| {
            ui.label("Time Scale:");
            ui.add(Slider::new(&mut gameplay.time_scale, 0.25..=4.0).show_value(false));
            ui.label(format!("{:.2}x", gameplay.time_scale));
        });

        ui.add_space(15.0);
        ui.checkbox(&mut gameplay.auto_save, "Auto-save");

        if gameplay.auto_save {
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.label("Auto-save interval:");
                let mut minutes = gameplay.auto_save_interval / 60;
                if ui.add(Slider::new(&mut minutes, 1..=30).suffix(" min")).changed() {
                    gameplay.auto_save_interval = minutes * 60;
                }
            });
        }
    }
}

fn tab_button(ui: &mut Ui, text: &str, selected: bool) -> bool {
    let (bg, fg) = if selected {
        (Color32::from_rgb(70, 70, 100), Color32::from_rgb(220, 220, 255))
    } else {
        (Color32::from_rgb(50, 50, 70), Color32::from_rgb(150, 150, 170))
    };

    ui.add(
        egui::Button::new(RichText::new(text).color(fg))
            .min_size(Vec2::new(80.0, 30.0))
            .fill(bg)
    ).clicked()
}

fn action_button(ui: &mut Ui, text: &str) -> bool {
    ui.add(
        egui::Button::new(
            RichText::new(text)
                .font(FontId::proportional(16.0))
                .color(Color32::from_rgb(220, 220, 240)),
        )
        .min_size(Vec2::new(100.0, 35.0))
        .fill(Color32::from_rgb(50, 50, 70))
    ).clicked()
}
