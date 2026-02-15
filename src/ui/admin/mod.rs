//! Admin panel — tabs for Items and Stories (admin-only)

mod item_editor;
mod story_editor;

use egui::{Align, Color32, FontId, Layout, RichText, Ui, Vec2};

use infinite_integration::IntegrationClient;

use crate::state::{ApplicationState, StateTransition};

use item_editor::ItemEditor;
use story_editor::StoryEditor;

/// Which admin tab is selected
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AdminTab {
    Items,
    Stories,
}

/// Admin panel with tabbed sub-editors
pub struct AdminPanel {
    tab: AdminTab,
    item_editor: ItemEditor,
    story_editor: StoryEditor,
    initialized: bool,
}

impl AdminPanel {
    pub fn new() -> Self {
        Self {
            tab: AdminTab::Items,
            item_editor: ItemEditor::new(),
            story_editor: StoryEditor::new(),
            initialized: false,
        }
    }

    /// Render the admin panel and return any state transition
    pub fn render(
        &mut self,
        ui: &mut Ui,
        integration_client: Option<&IntegrationClient>,
    ) -> StateTransition {
        let mut transition = StateTransition::None;

        // Fetch items on first render
        if !self.initialized {
            if let Some(client) = integration_client {
                self.item_editor.refresh_items(client);
                self.story_editor.refresh_stories(client);
            }
            self.initialized = true;
        }

        let _available = ui.available_size();

        ui.vertical(|ui| {
            // Header
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Admin Tools")
                        .font(FontId::proportional(28.0))
                        .color(Color32::from_rgb(200, 200, 255)),
                );

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let back_btn = egui::Button::new(
                        RichText::new("Back to Menu")
                            .font(FontId::proportional(14.0))
                            .color(Color32::from_rgb(220, 220, 240)),
                    )
                    .min_size(Vec2::new(120.0, 30.0))
                    .fill(Color32::from_rgb(80, 60, 60))
                    .stroke(egui::Stroke::new(1.0, Color32::from_rgb(100, 80, 80)));

                    if ui.add(back_btn).clicked() {
                        self.initialized = false;
                        transition = StateTransition::Replace(ApplicationState::MainMenu);
                    }
                });
            });

            ui.add_space(10.0);

            // Tab bar
            ui.horizontal(|ui| {
                let tabs = [
                    (AdminTab::Items, "Items"),
                    (AdminTab::Stories, "Stories"),
                ];
                for (tab, label) in &tabs {
                    let selected = self.tab == *tab;
                    let color = if selected {
                        Color32::from_rgb(100, 100, 180)
                    } else {
                        Color32::from_rgb(50, 50, 70)
                    };
                    let button = egui::Button::new(
                        RichText::new(*label)
                            .font(FontId::proportional(16.0))
                            .color(Color32::from_rgb(220, 220, 240)),
                    )
                    .min_size(Vec2::new(100.0, 32.0))
                    .fill(color)
                    .stroke(egui::Stroke::new(1.0, Color32::from_rgb(80, 80, 100)));

                    if ui.add(button).clicked() {
                        self.tab = *tab;
                    }
                }
            });

            ui.add_space(5.0);
            ui.separator();
            ui.add_space(5.0);

            // Tab content
            let remaining = ui.available_size();
            ui.allocate_ui(remaining, |ui| {
                match self.tab {
                    AdminTab::Items => self.item_editor.render(ui, integration_client),
                    AdminTab::Stories => self.story_editor.render(ui, integration_client),
                }
            });
        });

        transition
    }
}

impl Default for AdminPanel {
    fn default() -> Self {
        Self::new()
    }
}
