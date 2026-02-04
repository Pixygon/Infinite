//! Save/Load menu UI

use egui::{Align, Color32, FontId, Layout, RichText, Ui, Vec2};

use crate::save::{self, format_play_time, SaveSlotInfo};
use crate::state::StateTransition;

/// Action requested by the save/load menu
pub enum SaveLoadAction {
    None,
    /// Save to a new slot with the given name
    SaveNew(String),
    /// Load from the slot with this filename
    Load(String),
    /// Delete the slot with this filename
    Delete(String),
}

/// Save/Load menu renderer
pub struct SaveLoadMenu {
    /// Whether we're in save mode (true) or load mode (false)
    is_saving: bool,
    /// Cached list of save slots
    slots: Vec<SaveSlotInfo>,
    /// Text input for new save name
    new_save_name: String,
    /// Whether the slot list needs a refresh
    needs_refresh: bool,
}

impl SaveLoadMenu {
    pub fn new(is_saving: bool) -> Self {
        Self {
            is_saving,
            slots: Vec::new(),
            new_save_name: String::new(),
            needs_refresh: true,
        }
    }

    /// Mark that the slot list needs to be refreshed next render
    pub fn mark_needs_refresh(&mut self) {
        self.needs_refresh = true;
    }

    /// Refresh the slot list from disk
    fn refresh_slots(&mut self) {
        if let Ok(slots) = save::list_save_slots() {
            self.slots = slots;
        }
        self.needs_refresh = false;
    }

    /// Render the save/load menu and return transition + action
    pub fn render(&mut self, ui: &mut Ui) -> (StateTransition, SaveLoadAction) {
        if self.needs_refresh {
            self.refresh_slots();
        }

        let mut transition = StateTransition::None;
        let mut action = SaveLoadAction::None;
        let available = ui.available_size();

        // Semi-transparent background overlay
        let painter = ui.painter();
        painter.rect_filled(
            ui.max_rect(),
            0.0,
            Color32::from_rgba_unmultiplied(0, 0, 0, 200),
        );

        ui.vertical_centered(|ui| {
            ui.add_space(available.y * 0.1);

            // Title
            let title = if self.is_saving { "SAVE GAME" } else { "LOAD GAME" };
            ui.label(
                RichText::new(title)
                    .font(FontId::proportional(42.0))
                    .color(Color32::from_rgb(200, 200, 255)),
            );

            ui.add_space(20.0);

            // New save input (only in save mode)
            if self.is_saving {
                ui.horizontal(|ui| {
                    ui.add_space(available.x * 0.2);
                    ui.label(
                        RichText::new("Save name:")
                            .font(FontId::proportional(16.0))
                            .color(Color32::from_rgb(180, 180, 200)),
                    );
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.new_save_name)
                            .desired_width(200.0)
                            .font(FontId::proportional(16.0)),
                    );
                    let can_save = !self.new_save_name.trim().is_empty();
                    if ui.add_enabled(
                        can_save,
                        egui::Button::new(
                            RichText::new("Save")
                                .font(FontId::proportional(16.0))
                                .color(Color32::from_rgb(220, 220, 240)),
                        )
                        .min_size(Vec2::new(80.0, 30.0))
                        .fill(Color32::from_rgba_unmultiplied(40, 80, 40, 220))
                        .stroke(egui::Stroke::new(1.0, Color32::from_rgb(60, 120, 60))),
                    ).clicked()
                        || (can_save && response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                    {
                        action = SaveLoadAction::SaveNew(self.new_save_name.trim().to_string());
                        self.new_save_name.clear();
                        self.needs_refresh = true;
                    }
                });

                ui.add_space(15.0);
                ui.separator();
                ui.add_space(10.0);
            }

            // Slot list
            if self.slots.is_empty() {
                ui.add_space(30.0);
                ui.label(
                    RichText::new("No saved games found")
                        .font(FontId::proportional(18.0))
                        .color(Color32::from_rgb(150, 150, 170)),
                );
            } else {
                egui::ScrollArea::vertical()
                    .max_height(available.y * 0.5)
                    .show(ui, |ui| {
                        let slots_copy: Vec<SaveSlotInfo> = self.slots.clone();
                        for slot in &slots_copy {
                            ui.horizontal(|ui| {
                                ui.add_space(available.x * 0.15);

                                // Slot info
                                ui.vertical(|ui| {
                                    let display_name = if slot.slot_name.is_empty() {
                                        &slot.filename
                                    } else {
                                        &slot.slot_name
                                    };
                                    ui.label(
                                        RichText::new(display_name)
                                            .font(FontId::proportional(18.0))
                                            .color(Color32::from_rgb(220, 220, 240)),
                                    );
                                    ui.label(
                                        RichText::new(format!(
                                            "{} | {} | {}",
                                            slot.character_name,
                                            slot.timestamp,
                                            format_play_time(slot.play_time_seconds),
                                        ))
                                        .font(FontId::proportional(13.0))
                                        .color(Color32::from_rgb(140, 140, 160)),
                                    );
                                });

                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.add_space(available.x * 0.15);

                                    // Delete button
                                    if slot_button(ui, "Delete", Vec2::new(70.0, 28.0), Color32::from_rgba_unmultiplied(80, 30, 30, 220)) {
                                        action = SaveLoadAction::Delete(slot.filename.clone());
                                        self.needs_refresh = true;
                                    }

                                    ui.add_space(5.0);

                                    // Load/Overwrite button
                                    let btn_text = if self.is_saving { "Overwrite" } else { "Load" };
                                    let btn_color = if self.is_saving {
                                        Color32::from_rgba_unmultiplied(80, 60, 20, 220)
                                    } else {
                                        Color32::from_rgba_unmultiplied(30, 50, 80, 220)
                                    };
                                    if slot_button(ui, btn_text, Vec2::new(90.0, 28.0), btn_color) {
                                        if self.is_saving {
                                            action = SaveLoadAction::SaveNew(slot.slot_name.clone());
                                            self.needs_refresh = true;
                                        } else {
                                            action = SaveLoadAction::Load(slot.filename.clone());
                                        }
                                    }
                                });
                            });

                            ui.add_space(5.0);
                            ui.separator();
                            ui.add_space(5.0);
                        }
                    });
            }

            ui.add_space(30.0);

            // Back button
            if slot_button(ui, "Back", Vec2::new(120.0, 36.0), Color32::from_rgba_unmultiplied(50, 50, 70, 220)) {
                transition = StateTransition::Pop;
            }
        });

        (transition, action)
    }
}

fn slot_button(ui: &mut Ui, text: &str, size: Vec2, fill: Color32) -> bool {
    ui.add(
        egui::Button::new(
            RichText::new(text)
                .font(FontId::proportional(15.0))
                .color(Color32::from_rgb(220, 220, 240)),
        )
        .min_size(size)
        .fill(fill)
        .stroke(egui::Stroke::new(1.0, Color32::from_rgb(80, 80, 100)))
    ).clicked()
}
