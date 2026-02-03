//! Character creator UI with tabbed interface

use egui::{Align, Color32, FontId, Layout, RichText, ScrollArea, Ui, Vec2};

use crate::character::{
    CharacterAppearance, CharacterData,
    HairCustomization, SkinCustomization,
};
use crate::state::{ApplicationState, StateTransition};

/// Currently selected tab in character creator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CreatorTab {
    #[default]
    Basics,
    Body,
    Face,
    Hair,
    Skin,
    Preview,
}

impl CreatorTab {
    fn name(&self) -> &'static str {
        match self {
            Self::Basics => "Basics",
            Self::Body => "Body",
            Self::Face => "Face",
            Self::Hair => "Hair",
            Self::Skin => "Skin",
            Self::Preview => "Preview",
        }
    }

    fn all() -> &'static [CreatorTab] {
        &[
            Self::Basics,
            Self::Body,
            Self::Face,
            Self::Hair,
            Self::Skin,
            Self::Preview,
        ]
    }
}

/// Face customization sub-tabs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FaceSubTab {
    #[default]
    Shape,
    Eyes,
    Nose,
    Mouth,
    Ears,
    Chin,
    Brows,
}

impl FaceSubTab {
    fn name(&self) -> &'static str {
        match self {
            Self::Shape => "Shape",
            Self::Eyes => "Eyes",
            Self::Nose => "Nose",
            Self::Mouth => "Mouth",
            Self::Ears => "Ears",
            Self::Chin => "Chin",
            Self::Brows => "Brows",
        }
    }

    fn all() -> &'static [FaceSubTab] {
        &[
            Self::Shape,
            Self::Eyes,
            Self::Nose,
            Self::Mouth,
            Self::Ears,
            Self::Chin,
            Self::Brows,
        ]
    }
}

/// Character creator state
pub struct CharacterCreator {
    /// Character name input
    pub name: String,
    /// Current appearance being customized
    pub appearance: CharacterAppearance,
    /// Current tab
    pub current_tab: CreatorTab,
    /// Current face sub-tab
    pub face_sub_tab: FaceSubTab,
    /// Preview rotation angle (degrees)
    pub preview_rotation: f32,
    /// Preview zoom level
    pub preview_zoom: f32,
    /// Show wireframe overlay in preview
    pub show_wireframe: bool,
    /// Name validation error message
    pub name_error: Option<String>,
}

impl Default for CharacterCreator {
    fn default() -> Self {
        Self::new()
    }
}

impl CharacterCreator {
    /// Create a new character creator
    pub fn new() -> Self {
        Self {
            name: String::new(),
            appearance: CharacterAppearance::default(),
            current_tab: CreatorTab::default(),
            face_sub_tab: FaceSubTab::default(),
            preview_rotation: 0.0,
            preview_zoom: 1.0,
            show_wireframe: false,
            name_error: None,
        }
    }

    /// Reset the creator to defaults
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Randomize appearance
    pub fn randomize(&mut self) {
        self.appearance.randomize();
    }

    /// Validate the character name
    fn validate_name(&mut self) -> bool {
        let name = self.name.trim();

        if name.is_empty() {
            self.name_error = Some("Name cannot be empty".to_string());
            return false;
        }

        if name.len() < 2 {
            self.name_error = Some("Name must be at least 2 characters".to_string());
            return false;
        }

        if name.len() > 24 {
            self.name_error = Some("Name cannot exceed 24 characters".to_string());
            return false;
        }

        // Check for valid characters
        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == ' ' || c == '-' || c == '_')
        {
            self.name_error = Some("Name contains invalid characters".to_string());
            return false;
        }

        self.name_error = None;
        true
    }

    /// Create the character from current settings
    pub fn create_character(&self) -> CharacterData {
        CharacterData::with_appearance(
            self.name.trim().to_string(),
            self.appearance.clone(),
        )
    }

    /// Render the character creator and return any state transition
    pub fn render(&mut self, ui: &mut Ui) -> StateTransition {
        let mut transition = StateTransition::None;
        let available = ui.available_size();

        // Split layout: 60% controls, 40% preview
        let controls_width = available.x * 0.6;
        let preview_width = available.x * 0.4;

        ui.horizontal(|ui| {
            // Left panel: Controls
            ui.vertical(|ui| {
                ui.set_width(controls_width - 20.0);
                ui.set_height(available.y);

                // Title
                ui.add_space(20.0);
                ui.label(
                    RichText::new("Create Your Character")
                        .font(FontId::proportional(32.0))
                        .color(Color32::from_rgb(200, 200, 255)),
                );
                ui.add_space(20.0);

                // Tabs
                ui.horizontal(|ui| {
                    for tab in CreatorTab::all() {
                        let selected = self.current_tab == *tab;
                        let color = if selected {
                            Color32::from_rgb(100, 100, 180)
                        } else {
                            Color32::from_rgb(50, 50, 70)
                        };

                        let button = egui::Button::new(
                            RichText::new(tab.name())
                                .font(FontId::proportional(14.0))
                                .color(Color32::from_rgb(220, 220, 240)),
                        )
                        .fill(color)
                        .min_size(Vec2::new(70.0, 30.0));

                        if ui.add(button).clicked() {
                            self.current_tab = *tab;
                        }
                    }
                });

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                // Tab content
                ScrollArea::vertical()
                    .max_height(available.y - 200.0)
                    .show(ui, |ui| match self.current_tab {
                        CreatorTab::Basics => self.render_basics_tab(ui),
                        CreatorTab::Body => self.render_body_tab(ui),
                        CreatorTab::Face => self.render_face_tab(ui),
                        CreatorTab::Hair => self.render_hair_tab(ui),
                        CreatorTab::Skin => self.render_skin_tab(ui),
                        CreatorTab::Preview => self.render_preview_tab(ui),
                    });

                ui.add_space(20.0);

                // Bottom buttons
                ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
                    ui.add_space(20.0);

                    ui.horizontal(|ui| {
                        let button_size = Vec2::new(120.0, 40.0);

                        // Back button
                        if action_button(ui, "Back", button_size, Color32::from_rgb(80, 60, 60)) {
                            transition = StateTransition::Replace(ApplicationState::MainMenu);
                        }

                        ui.add_space(20.0);

                        // Randomize button
                        if action_button(
                            ui,
                            "Randomize",
                            button_size,
                            Color32::from_rgb(60, 80, 60),
                        ) {
                            self.randomize();
                        }

                        ui.add_space(20.0);

                        // Create button
                        if action_button(
                            ui,
                            "Create Character",
                            Vec2::new(150.0, 40.0),
                            Color32::from_rgb(60, 80, 120),
                        ) {
                            if self.validate_name() {
                                let character = self.create_character();
                                if let Err(e) = crate::character::save_character(&character) {
                                    tracing::error!("Failed to save character: {}", e);
                                    self.name_error = Some("Failed to save character".to_string());
                                } else {
                                    transition =
                                        StateTransition::Replace(ApplicationState::Playing);
                                }
                            }
                        }
                    });

                    // Show name error if any
                    if let Some(error) = &self.name_error {
                        ui.add_space(10.0);
                        ui.label(
                            RichText::new(error)
                                .font(FontId::proportional(14.0))
                                .color(Color32::from_rgb(255, 100, 100)),
                        );
                    }
                });
            });

            ui.separator();

            // Right panel: Preview
            ui.vertical(|ui| {
                ui.set_width(preview_width - 20.0);
                ui.set_height(available.y);

                self.render_character_preview(ui);
            });
        });

        transition
    }

    /// Render the basics tab (name only - archetype chosen later in gameplay)
    fn render_basics_tab(&mut self, ui: &mut Ui) {
        section_header(ui, "Character Name");

        ui.horizontal(|ui| {
            ui.label("Name:");
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.name)
                    .desired_width(250.0)
                    .hint_text("Enter character name..."),
            );
            if response.changed() {
                self.name_error = None;
            }
        });

        ui.add_space(20.0);

        // Info about archetype selection
        egui::Frame::new()
            .fill(Color32::from_rgb(40, 40, 55))
            .corner_radius(5.0)
            .inner_margin(15.0)
            .show(ui, |ui| {
                ui.label(
                    RichText::new("About Classes")
                        .font(FontId::proportional(16.0))
                        .color(Color32::from_rgb(180, 180, 220)),
                );
                ui.add_space(10.0);
                ui.label(
                    RichText::new("Your character's class will be chosen during gameplay. \
                        Focus on creating your character's appearance first - \
                        you'll discover your path through the timeline as you play.")
                        .font(FontId::proportional(13.0))
                        .color(Color32::from_rgb(150, 150, 170)),
                );
            });

        ui.add_space(20.0);

        // Quick tips
        section_header(ui, "Tips");
        ui.label(
            RichText::new("- Use the tabs above to customize your character's appearance")
                .font(FontId::proportional(12.0))
                .color(Color32::from_rgb(130, 130, 150)),
        );
        ui.label(
            RichText::new("- Click 'Randomize' at the bottom for a random look")
                .font(FontId::proportional(12.0))
                .color(Color32::from_rgb(130, 130, 150)),
        );
        ui.label(
            RichText::new("- The preview on the right shows your character in 3D")
                .font(FontId::proportional(12.0))
                .color(Color32::from_rgb(130, 130, 150)),
        );
    }

    /// Render the body tab
    fn render_body_tab(&mut self, ui: &mut Ui) {
        section_header(ui, "Body Proportions");

        slider_row(ui, "Height", &mut self.appearance.body.height, "Short", "Tall");
        slider_row(ui, "Build", &mut self.appearance.body.build, "Slim", "Heavy");
        slider_row(
            ui,
            "Shoulders",
            &mut self.appearance.body.shoulder_width,
            "Narrow",
            "Wide",
        );
        slider_row(ui, "Hips", &mut self.appearance.body.hip_width, "Narrow", "Wide");

        ui.add_space(10.0);
        section_header(ui, "Limb Proportions");

        slider_row(ui, "Arms", &mut self.appearance.body.arm_length, "Short", "Long");
        slider_row(ui, "Legs", &mut self.appearance.body.leg_length, "Short", "Long");
        slider_row(
            ui,
            "Torso",
            &mut self.appearance.body.torso_length,
            "Short",
            "Long",
        );
    }

    /// Render the face tab with sub-tabs
    fn render_face_tab(&mut self, ui: &mut Ui) {
        // Face sub-tabs
        ui.horizontal(|ui| {
            for tab in FaceSubTab::all() {
                let selected = self.face_sub_tab == *tab;
                if ui.selectable_label(selected, tab.name()).clicked() {
                    self.face_sub_tab = *tab;
                }
            }
        });

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        match self.face_sub_tab {
            FaceSubTab::Shape => {
                section_header(ui, "Face Shape");
                slider_row(
                    ui,
                    "Shape",
                    &mut self.appearance.face.face_shape,
                    "Round",
                    "Angular",
                );
                slider_row(
                    ui,
                    "Width",
                    &mut self.appearance.face.face_width,
                    "Narrow",
                    "Wide",
                );
                slider_row(
                    ui,
                    "Length",
                    &mut self.appearance.face.face_length,
                    "Short",
                    "Long",
                );
                slider_row(ui, "Jaw", &mut self.appearance.face.jaw, "Soft", "Sharp");
                slider_row(
                    ui,
                    "Cheekbones",
                    &mut self.appearance.face.cheekbones,
                    "Flat",
                    "Prominent",
                );
            }
            FaceSubTab::Eyes => {
                section_header(ui, "Eyes");
                slider_row(ui, "Size", &mut self.appearance.face.eye_size, "Small", "Large");
                slider_row(
                    ui,
                    "Spacing",
                    &mut self.appearance.face.eye_spacing,
                    "Close",
                    "Far",
                );
                slider_row(ui, "Slant", &mut self.appearance.face.eye_slant, "Down", "Up");
                color_slider(ui, "Color", &mut self.appearance.face.eye_color);
            }
            FaceSubTab::Nose => {
                section_header(ui, "Nose");
                slider_row(
                    ui,
                    "Length",
                    &mut self.appearance.face.nose_length,
                    "Short",
                    "Long",
                );
                slider_row(
                    ui,
                    "Width",
                    &mut self.appearance.face.nose_width,
                    "Narrow",
                    "Wide",
                );
                slider_row(
                    ui,
                    "Bridge",
                    &mut self.appearance.face.nose_bridge,
                    "Flat",
                    "High",
                );
            }
            FaceSubTab::Mouth => {
                section_header(ui, "Mouth");
                slider_row(
                    ui,
                    "Lip Fullness",
                    &mut self.appearance.face.lip_fullness,
                    "Thin",
                    "Full",
                );
                slider_row(
                    ui,
                    "Width",
                    &mut self.appearance.face.mouth_width,
                    "Narrow",
                    "Wide",
                );
            }
            FaceSubTab::Ears => {
                section_header(ui, "Ears");
                slider_row(ui, "Size", &mut self.appearance.face.ear_size, "Small", "Large");
                slider_row(
                    ui,
                    "Shape",
                    &mut self.appearance.face.ear_pointiness,
                    "Round",
                    "Pointed",
                );
            }
            FaceSubTab::Chin => {
                section_header(ui, "Chin");
                slider_row(
                    ui,
                    "Length",
                    &mut self.appearance.face.chin_length,
                    "Short",
                    "Long",
                );
                slider_row(
                    ui,
                    "Width",
                    &mut self.appearance.face.chin_width,
                    "Narrow",
                    "Wide",
                );
            }
            FaceSubTab::Brows => {
                section_header(ui, "Eyebrows");
                slider_row(
                    ui,
                    "Thickness",
                    &mut self.appearance.face.brow_thickness,
                    "Thin",
                    "Thick",
                );
                slider_row(ui, "Arch", &mut self.appearance.face.brow_arch, "Flat", "Arched");
            }
        }
    }

    /// Render the hair tab
    fn render_hair_tab(&mut self, ui: &mut Ui) {
        section_header(ui, "Hair Style");

        ui.horizontal(|ui| {
            ui.label("Style:");
            let style_label = format!("Style {}", self.appearance.hair.style + 1);
            egui::ComboBox::from_id_salt("hair_style")
                .selected_text(style_label)
                .show_ui(ui, |ui| {
                    for i in 0..HairCustomization::HAIR_STYLE_COUNT {
                        ui.selectable_value(
                            &mut self.appearance.hair.style,
                            i,
                            format!("Style {}", i + 1),
                        );
                    }
                });
        });

        slider_row(ui, "Length", &mut self.appearance.hair.length, "Short", "Long");
        slider_row(ui, "Volume", &mut self.appearance.hair.volume, "Flat", "Full");

        ui.add_space(10.0);
        section_header(ui, "Hair Color");

        color_slider(ui, "Color", &mut self.appearance.hair.color_hue);
        slider_row(
            ui,
            "Saturation",
            &mut self.appearance.hair.color_saturation,
            "Gray",
            "Vivid",
        );
        slider_row(
            ui,
            "Brightness",
            &mut self.appearance.hair.color_brightness,
            "Dark",
            "Light",
        );

        ui.add_space(5.0);
        color_slider(ui, "Highlights", &mut self.appearance.hair.highlight_hue);
        slider_row(
            ui,
            "Highlight Intensity",
            &mut self.appearance.hair.highlight_intensity,
            "None",
            "Strong",
        );

        ui.add_space(10.0);
        section_header(ui, "Facial Hair");

        ui.horizontal(|ui| {
            ui.label("Style:");
            let style_label = if self.appearance.hair.facial_hair_style == 0 {
                "None".to_string()
            } else {
                format!("Style {}", self.appearance.hair.facial_hair_style)
            };
            egui::ComboBox::from_id_salt("facial_hair_style")
                .selected_text(style_label)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.appearance.hair.facial_hair_style, 0, "None");
                    for i in 1..HairCustomization::FACIAL_HAIR_STYLE_COUNT {
                        ui.selectable_value(
                            &mut self.appearance.hair.facial_hair_style,
                            i,
                            format!("Style {}", i),
                        );
                    }
                });
        });

        if self.appearance.hair.facial_hair_style > 0 {
            slider_row(
                ui,
                "Density",
                &mut self.appearance.hair.facial_hair_density,
                "Sparse",
                "Full",
            );
        }
    }

    /// Render the skin tab
    fn render_skin_tab(&mut self, ui: &mut Ui) {
        section_header(ui, "Skin Tone");

        slider_row(ui, "Tone", &mut self.appearance.skin.tone, "Light", "Dark");
        slider_row(
            ui,
            "Undertone",
            &mut self.appearance.skin.undertone,
            "Cool",
            "Warm",
        );

        ui.add_space(10.0);
        section_header(ui, "Details");

        slider_row(ui, "Age", &mut self.appearance.skin.age, "Young", "Aged");
        slider_row(
            ui,
            "Freckles",
            &mut self.appearance.skin.freckles,
            "None",
            "Many",
        );
        slider_row(
            ui,
            "Blemishes",
            &mut self.appearance.skin.blemishes,
            "Clear",
            "Rough",
        );

        ui.add_space(10.0);
        section_header(ui, "Tattoos");

        ui.horizontal(|ui| {
            ui.label("Style:");
            let style_label = if self.appearance.skin.tattoo_style == 0 {
                "None".to_string()
            } else {
                format!("Style {}", self.appearance.skin.tattoo_style)
            };
            egui::ComboBox::from_id_salt("tattoo_style")
                .selected_text(style_label)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.appearance.skin.tattoo_style, 0, "None");
                    for i in 1..SkinCustomization::TATTOO_STYLE_COUNT {
                        ui.selectable_value(
                            &mut self.appearance.skin.tattoo_style,
                            i,
                            format!("Style {}", i),
                        );
                    }
                });
        });

        if self.appearance.skin.tattoo_style > 0 {
            slider_row(
                ui,
                "Intensity",
                &mut self.appearance.skin.tattoo_intensity,
                "Faded",
                "Bold",
            );
            color_slider(ui, "Color", &mut self.appearance.skin.tattoo_color);
        }

        ui.add_space(10.0);
        section_header(ui, "Face Paint");

        ui.horizontal(|ui| {
            ui.label("Style:");
            let style_label = if self.appearance.skin.face_paint_style == 0 {
                "None".to_string()
            } else {
                format!("Style {}", self.appearance.skin.face_paint_style)
            };
            egui::ComboBox::from_id_salt("face_paint_style")
                .selected_text(style_label)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.appearance.skin.face_paint_style, 0, "None");
                    for i in 1..SkinCustomization::FACE_PAINT_STYLE_COUNT {
                        ui.selectable_value(
                            &mut self.appearance.skin.face_paint_style,
                            i,
                            format!("Style {}", i),
                        );
                    }
                });
        });

        if self.appearance.skin.face_paint_style > 0 {
            slider_row(
                ui,
                "Intensity",
                &mut self.appearance.skin.face_paint_intensity,
                "Subtle",
                "Bold",
            );
            color_slider(ui, "Color", &mut self.appearance.skin.face_paint_color);
        }
    }

    /// Render the preview tab
    fn render_preview_tab(&mut self, ui: &mut Ui) {
        section_header(ui, "Preview Controls");

        ui.horizontal(|ui| {
            ui.label("Rotation:");
            ui.add(egui::Slider::new(&mut self.preview_rotation, 0.0..=360.0).suffix("°"));
        });

        ui.horizontal(|ui| {
            ui.label("Zoom:");
            ui.add(egui::Slider::new(&mut self.preview_zoom, 0.5..=2.0));
        });

        ui.checkbox(&mut self.show_wireframe, "Show Wireframe");

        ui.add_space(20.0);
        section_header(ui, "Quick Rotate");

        ui.horizontal(|ui| {
            if ui.button("Front").clicked() {
                self.preview_rotation = 0.0;
            }
            if ui.button("Left").clicked() {
                self.preview_rotation = 90.0;
            }
            if ui.button("Back").clicked() {
                self.preview_rotation = 180.0;
            }
            if ui.button("Right").clicked() {
                self.preview_rotation = 270.0;
            }
        });
    }

    /// Render the 3D character preview
    /// Note: The actual 3D rendering happens in main.rs render loop
    /// This just provides the UI frame and controls
    fn render_character_preview(&mut self, ui: &mut Ui) {
        let available = ui.available_size();

        // Reserve the preview area - actual 3D rendering happens in main.rs
        // We'll use allocate_space to get the rect for the 3D viewport
        ui.vertical(|ui| {
            ui.set_min_size(available);

            // Preview header
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Preview")
                        .font(FontId::proportional(16.0))
                        .color(Color32::from_rgb(180, 180, 220)),
                );

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if !self.name.is_empty() {
                        ui.label(
                            RichText::new(&self.name)
                                .font(FontId::proportional(14.0))
                                .color(Color32::from_rgb(150, 150, 180)),
                        );
                    }
                });
            });

            ui.add_space(5.0);

            // 3D preview area (rendered in main.rs)
            let preview_height = available.y - 100.0;
            let (rect, _response) = ui.allocate_exact_size(
                Vec2::new(available.x - 10.0, preview_height),
                egui::Sense::click_and_drag(),
            );

            // Draw background for the 3D area
            ui.painter().rect_filled(
                rect,
                5.0,
                Color32::from_rgb(20, 20, 30),
            );

            // Store the preview rect for 3D rendering (accessed via egui context)
            ui.ctx().data_mut(|data| {
                data.insert_temp(
                    egui::Id::new("character_preview_rect"),
                    [rect.min.x, rect.min.y, rect.width(), rect.height()],
                );
            });

            ui.add_space(10.0);

            // Preview controls at bottom
            ui.horizontal(|ui| {
                ui.label("Rotate:");
                if ui.add(egui::Slider::new(&mut self.preview_rotation, 0.0..=360.0).show_value(false)).changed() {
                    // Rotation changed
                }
            });

            ui.horizontal(|ui| {
                ui.label("Zoom:");
                ui.add(egui::Slider::new(&mut self.preview_zoom, 0.5..=2.0).show_value(false));

                ui.separator();

                // Quick rotation buttons
                if ui.small_button("Front").clicked() {
                    self.preview_rotation = 0.0;
                }
                if ui.small_button("Side").clicked() {
                    self.preview_rotation = 90.0;
                }
                if ui.small_button("Back").clicked() {
                    self.preview_rotation = 180.0;
                }
            });
        });
    }
}

/// Helper: Section header
fn section_header(ui: &mut Ui, text: &str) {
    ui.label(
        RichText::new(text)
            .font(FontId::proportional(16.0))
            .color(Color32::from_rgb(180, 180, 220)),
    );
    ui.add_space(5.0);
}

/// Helper: Slider row with labels
fn slider_row(ui: &mut Ui, label: &str, value: &mut f32, min_label: &str, max_label: &str) {
    ui.horizontal(|ui| {
        ui.label(format!("{}:", label));
        ui.label(
            RichText::new(min_label)
                .font(FontId::proportional(10.0))
                .color(Color32::from_rgb(120, 120, 140)),
        );
        ui.add(egui::Slider::new(value, 0.0..=1.0).show_value(false));
        ui.label(
            RichText::new(max_label)
                .font(FontId::proportional(10.0))
                .color(Color32::from_rgb(120, 120, 140)),
        );
    });
}

/// Helper: Color hue slider
fn color_slider(ui: &mut Ui, label: &str, value: &mut f32) {
    ui.horizontal(|ui| {
        ui.label(format!("{}:", label));

        // Show color preview
        let hue = *value;
        let rgb = hsv_to_rgb(hue, 0.7, 0.8);
        let color = Color32::from_rgb(
            (rgb.0 * 255.0) as u8,
            (rgb.1 * 255.0) as u8,
            (rgb.2 * 255.0) as u8,
        );

        let (rect, _response) = ui.allocate_exact_size(Vec2::new(20.0, 20.0), egui::Sense::hover());
        ui.painter().rect_filled(rect, 3.0, color);

        ui.add(egui::Slider::new(value, 0.0..=1.0).show_value(false));
    });
}

/// Helper: Action button
fn action_button(ui: &mut Ui, text: &str, size: Vec2, color: Color32) -> bool {
    let button = egui::Button::new(
        RichText::new(text)
            .font(FontId::proportional(16.0))
            .color(Color32::from_rgb(220, 220, 240)),
    )
    .min_size(size)
    .fill(color)
    .stroke(egui::Stroke::new(1.0, Color32::from_rgb(100, 100, 120)));

    ui.add(button).clicked()
}

/// Convert HSV to RGB (all values 0.0-1.0)
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let c = v * s;
    let h = h * 6.0;
    let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
    let m = v - c;

    let (r, g, b) = if h < 1.0 {
        (c, x, 0.0)
    } else if h < 2.0 {
        (x, c, 0.0)
    } else if h < 3.0 {
        (0.0, c, x)
    } else if h < 4.0 {
        (0.0, x, c)
    } else if h < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (r + m, g + m, b + m)
}
