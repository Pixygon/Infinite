//! Story editor — create and edit game stories with events

use egui::{Color32, FontId, RichText, ScrollArea, Ui, Vec2};

use infinite_integration::IntegrationClient;
use infinite_integration::types::{ServerGameStory, StoryEvent, StoryTrigger, StoryAction};
use infinite_integration::PendingRequest;

/// Story editor state
pub struct StoryEditor {
    /// Stories fetched from server
    stories: Vec<ServerGameStory>,
    /// Currently selected story index
    selected_index: Option<usize>,
    /// Editable form
    form: StoryForm,
    /// Creating new story
    creating_new: bool,
    /// Pending list fetch
    pending_list: Option<PendingRequest<Vec<ServerGameStory>>>,
    /// Pending save
    pending_save: Option<PendingRequest<ServerGameStory>>,
    /// Pending delete
    pending_delete: Option<PendingRequest<serde_json::Value>>,
    /// Status message
    status: Option<(String, bool)>,
}

/// Editable story form
struct StoryForm {
    story_id: String,
    name: String,
    description: String,
    start_x: f32,
    start_y: f32,
    start_z: f32,
    start_year: i64,
    start_time_of_day: f32,
    difficulty: String,
    estimated_minutes: u32,
    tags: String,
    is_published: bool,
    events: Vec<EventForm>,
}

/// Editable event form
struct EventForm {
    event_id: String,
    name: String,
    description: String,
    trigger_type: String,
    trigger_params: String,
    actions: Vec<ActionForm>,
}

/// Editable action form
struct ActionForm {
    action_type: String,
    params: String,
}

impl Default for StoryForm {
    fn default() -> Self {
        Self {
            story_id: String::new(),
            name: String::new(),
            description: String::new(),
            start_x: 0.0,
            start_y: 0.0,
            start_z: 0.0,
            start_year: 2025,
            start_time_of_day: 0.5,
            difficulty: "normal".to_string(),
            estimated_minutes: 30,
            tags: String::new(),
            is_published: false,
            events: Vec::new(),
        }
    }
}

impl StoryForm {
    fn from_server(story: &ServerGameStory) -> Self {
        let events = story.events.iter().map(|e| {
            EventForm {
                event_id: e.event_id.clone(),
                name: e.name.clone(),
                description: e.description.clone(),
                trigger_type: e.trigger.trigger_type.clone(),
                trigger_params: serde_json::to_string_pretty(&e.trigger.params)
                    .unwrap_or_default(),
                actions: e.actions.iter().map(|a| {
                    ActionForm {
                        action_type: a.action_type.clone(),
                        params: serde_json::to_string_pretty(&a.params)
                            .unwrap_or_default(),
                    }
                }).collect(),
            }
        }).collect();

        Self {
            story_id: story.story_id.clone(),
            name: story.name.clone(),
            description: story.description.clone(),
            start_x: story.start_location.as_ref().map(|l| l.x).unwrap_or(0.0),
            start_y: story.start_location.as_ref().map(|l| l.y).unwrap_or(0.0),
            start_z: story.start_location.as_ref().map(|l| l.z).unwrap_or(0.0),
            start_year: story.start_year.unwrap_or(2025),
            start_time_of_day: story.start_time_of_day.unwrap_or(0.5),
            difficulty: story.difficulty.clone().unwrap_or_else(|| "normal".to_string()),
            estimated_minutes: story.estimated_minutes.unwrap_or(30),
            tags: story.tags.join(", "),
            is_published: story.is_published,
            events,
        }
    }

    fn to_server(&self, project_id: &str) -> ServerGameStory {
        let events = self.events.iter().map(|e| {
            let params: serde_json::Value = serde_json::from_str(&e.trigger_params)
                .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
            let actions = e.actions.iter().map(|a| {
                let action_params: serde_json::Value = serde_json::from_str(&a.params)
                    .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
                StoryAction {
                    action_type: a.action_type.clone(),
                    params: action_params,
                }
            }).collect();

            StoryEvent {
                event_id: if e.event_id.is_empty() {
                    format!("evt_{}", uuid_simple())
                } else {
                    e.event_id.clone()
                },
                name: e.name.clone(),
                description: e.description.clone(),
                trigger: StoryTrigger {
                    trigger_type: e.trigger_type.clone(),
                    params,
                },
                actions,
                next_events: vec![],
            }
        }).collect();

        let tags: Vec<String> = self.tags
            .split(',')
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect();

        ServerGameStory {
            id: None,
            story_id: if self.story_id.is_empty() {
                format!("story_{}", uuid_simple())
            } else {
                self.story_id.clone()
            },
            project_id: project_id.to_string(),
            name: self.name.clone(),
            description: self.description.clone(),
            start_location: Some(infinite_integration::types::StoryLocation {
                x: self.start_x,
                y: self.start_y,
                z: self.start_z,
            }),
            start_year: Some(self.start_year),
            start_time_of_day: Some(self.start_time_of_day),
            difficulty: Some(self.difficulty.clone()),
            estimated_minutes: Some(self.estimated_minutes),
            tags,
            is_published: self.is_published,
            events,
        }
    }
}

fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("{:x}", t)
}

impl StoryEditor {
    pub fn new() -> Self {
        Self {
            stories: Vec::new(),
            selected_index: None,
            form: StoryForm::default(),
            creating_new: false,
            pending_list: None,
            pending_save: None,
            pending_delete: None,
            status: None,
        }
    }

    pub fn refresh_stories(&mut self, client: &IntegrationClient) {
        self.pending_list = Some(client.list_stories());
    }

    pub fn render(&mut self, ui: &mut Ui, integration_client: Option<&IntegrationClient>) {
        self.poll_pending(integration_client);

        ui.horizontal(|ui| {
            // Left: story list
            let list_width = 250.0;
            ui.vertical(|ui| {
                ui.set_width(list_width);
                ui.set_min_height(ui.available_height());

                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Stories")
                            .font(FontId::proportional(18.0))
                            .color(Color32::from_rgb(180, 180, 220)),
                    );
                    if let Some(client) = integration_client {
                        if ui.small_button("Refresh").clicked() {
                            self.refresh_stories(client);
                        }
                    }
                });

                ui.add_space(5.0);

                // New story button
                let new_btn = egui::Button::new(
                    RichText::new("+ New Story")
                        .font(FontId::proportional(14.0))
                        .color(Color32::from_rgb(100, 255, 100)),
                )
                .min_size(Vec2::new(list_width - 10.0, 28.0))
                .fill(Color32::from_rgb(40, 60, 40))
                .stroke(egui::Stroke::new(1.0, Color32::from_rgb(60, 100, 60)));

                if ui.add(new_btn).clicked() {
                    self.form = StoryForm::default();
                    self.selected_index = None;
                    self.creating_new = true;
                }

                ui.add_space(5.0);
                ui.separator();

                ScrollArea::vertical()
                    .max_height(ui.available_height() - 10.0)
                    .show(ui, |ui| {
                        let mut clicked_index = None;
                        for (i, story) in self.stories.iter().enumerate() {
                            let selected = self.selected_index == Some(i);
                            let bg = if selected {
                                Color32::from_rgb(60, 60, 90)
                            } else {
                                Color32::TRANSPARENT
                            };

                            let response = ui.horizontal(|ui| {
                                egui::Frame::new()
                                    .fill(bg)
                                    .corner_radius(3.0)
                                    .inner_margin(4.0)
                                    .show(ui, |ui| {
                                        ui.set_min_width(list_width - 30.0);
                                        ui.vertical(|ui| {
                                            ui.label(
                                                RichText::new(&story.name)
                                                    .font(FontId::proportional(13.0))
                                                    .color(Color32::from_rgb(200, 200, 240)),
                                            );
                                            let published = if story.is_published { "Published" } else { "Draft" };
                                            ui.label(
                                                RichText::new(format!("{} - {} events", published, story.events.len()))
                                                    .font(FontId::proportional(10.0))
                                                    .color(Color32::from_rgb(120, 120, 140)),
                                            );
                                        });
                                    });
                            });
                            if response.response.interact(egui::Sense::click()).clicked() {
                                clicked_index = Some(i);
                            }
                        }

                        if let Some(i) = clicked_index {
                            self.selected_index = Some(i);
                            self.creating_new = false;
                            self.form = StoryForm::from_server(&self.stories[i]);
                        }
                    });
            });

            ui.separator();

            // Right: edit form
            ui.vertical(|ui| {
                if self.creating_new || self.selected_index.is_some() {
                    self.render_form(ui, integration_client);
                } else {
                    ui.add_space(100.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("Select a story or create a new one")
                                .font(FontId::proportional(16.0))
                                .color(Color32::from_rgb(120, 120, 140)),
                        );
                    });
                }
            });
        });
    }

    fn render_form(&mut self, ui: &mut Ui, integration_client: Option<&IntegrationClient>) {
        let title = if self.creating_new { "New Story" } else { "Edit Story" };
        ui.label(
            RichText::new(title)
                .font(FontId::proportional(20.0))
                .color(Color32::from_rgb(200, 200, 255)),
        );

        if let Some((msg, is_err)) = &self.status {
            let color = if *is_err {
                Color32::from_rgb(255, 100, 100)
            } else {
                Color32::from_rgb(100, 255, 100)
            };
            ui.label(RichText::new(msg).font(FontId::proportional(13.0)).color(color));
        }

        ui.add_space(10.0);

        ScrollArea::vertical()
            .max_height(ui.available_height() - 50.0)
            .show(ui, |ui| {
                // Metadata
                section_header(ui, "Metadata");
                form_field(ui, "Name", &mut self.form.name);
                form_field(ui, "Description", &mut self.form.description);
                combo_field(ui, "Difficulty", &mut self.form.difficulty,
                    &["easy", "normal", "hard", "nightmare"]);
                ui.horizontal(|ui| {
                    ui.label("Est. Minutes:");
                    let mut em = self.form.estimated_minutes as i32;
                    ui.add(egui::DragValue::new(&mut em).range(1..=600));
                    self.form.estimated_minutes = em.max(1) as u32;
                });
                form_field(ui, "Tags", &mut self.form.tags);
                ui.checkbox(&mut self.form.is_published, "Published");

                ui.add_space(10.0);

                // Start conditions
                section_header(ui, "Start Conditions");
                ui.horizontal(|ui| {
                    ui.label("Location X:");
                    ui.add(egui::DragValue::new(&mut self.form.start_x).speed(1.0));
                    ui.label("Y:");
                    ui.add(egui::DragValue::new(&mut self.form.start_y).speed(1.0));
                    ui.label("Z:");
                    ui.add(egui::DragValue::new(&mut self.form.start_z).speed(1.0));
                });
                ui.horizontal(|ui| {
                    ui.label("Start Year:");
                    let mut sy = self.form.start_year as i32;
                    ui.add(egui::DragValue::new(&mut sy));
                    self.form.start_year = sy as i64;
                });
                ui.horizontal(|ui| {
                    ui.label("Time of Day:");
                    ui.add(egui::Slider::new(&mut self.form.start_time_of_day, 0.0..=1.0));
                });

                ui.add_space(10.0);

                // Events
                section_header(ui, "Events");

                let add_btn = egui::Button::new(
                    RichText::new("+ Add Event")
                        .font(FontId::proportional(12.0))
                        .color(Color32::from_rgb(100, 255, 100)),
                )
                .fill(Color32::from_rgb(40, 60, 40));

                if ui.add(add_btn).clicked() {
                    self.form.events.push(EventForm {
                        event_id: String::new(),
                        name: format!("Event {}", self.form.events.len() + 1),
                        description: String::new(),
                        trigger_type: "on_start".to_string(),
                        trigger_params: "{}".to_string(),
                        actions: Vec::new(),
                    });
                }

                ui.add_space(5.0);

                let mut remove_event = None;
                for (i, event) in self.form.events.iter_mut().enumerate() {
                    egui::Frame::new()
                        .fill(Color32::from_rgb(35, 35, 50))
                        .corner_radius(4.0)
                        .inner_margin(8.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(format!("Event #{}", i + 1))
                                        .font(FontId::proportional(14.0))
                                        .color(Color32::from_rgb(180, 180, 220)),
                                );
                                if ui.small_button("Remove").clicked() {
                                    remove_event = Some(i);
                                }
                            });

                            form_field(ui, "Name", &mut event.name);
                            form_field(ui, "Description", &mut event.description);
                            combo_field(ui, "Trigger", &mut event.trigger_type,
                                &["on_start", "reach_location", "reach_year",
                                  "interact_object", "defeat_npc", "collect_item",
                                  "timer", "event_completed"]);
                            ui.horizontal(|ui| {
                                ui.label("Trigger Params (JSON):");
                            });
                            ui.text_edit_multiline(&mut event.trigger_params);

                            // Actions
                            ui.add_space(5.0);
                            ui.label(
                                RichText::new("Actions")
                                    .font(FontId::proportional(12.0))
                                    .color(Color32::from_rgb(150, 150, 180)),
                            );

                            let add_action_btn = egui::Button::new("+ Action")
                                .fill(Color32::from_rgb(40, 50, 60));
                            if ui.add(add_action_btn).clicked() {
                                event.actions.push(ActionForm {
                                    action_type: "show_dialogue".to_string(),
                                    params: "{}".to_string(),
                                });
                            }

                            let mut remove_action = None;
                            for (j, action) in event.actions.iter_mut().enumerate() {
                                ui.horizontal(|ui| {
                                    combo_field(ui, &format!("Action {}", j + 1), &mut action.action_type,
                                        &["spawn_npc", "despawn_npc", "show_dialogue",
                                          "teleport_player", "change_year", "give_item",
                                          "set_objective", "complete_story"]);
                                    if ui.small_button("X").clicked() {
                                        remove_action = Some(j);
                                    }
                                });
                                ui.text_edit_multiline(&mut action.params);
                            }

                            if let Some(j) = remove_action {
                                event.actions.remove(j);
                            }
                        });

                    ui.add_space(5.0);
                }

                if let Some(i) = remove_event {
                    self.form.events.remove(i);
                }
            });

        // Action buttons
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            let is_busy = self.pending_save.is_some() || self.pending_delete.is_some();

            let save_label = if is_busy { "Saving..." } else if self.creating_new { "Create" } else { "Save" };
            let save_btn = egui::Button::new(
                RichText::new(save_label)
                    .font(FontId::proportional(14.0))
                    .color(Color32::from_rgb(220, 220, 240)),
            )
            .min_size(Vec2::new(100.0, 32.0))
            .fill(Color32::from_rgb(40, 70, 40))
            .stroke(egui::Stroke::new(1.0, Color32::from_rgb(60, 100, 60)));

            if ui.add(save_btn).clicked() && !is_busy {
                if let Some(client) = integration_client {
                    let story = self.form.to_server("6981e8eda259e89734bd007a");
                    if self.creating_new {
                        self.pending_save = Some(client.create_story(story));
                    } else {
                        self.pending_save = Some(client.update_story(&self.form.story_id, story));
                    }
                }
            }

            if ui.button("Cancel").clicked() {
                self.selected_index = None;
                self.creating_new = false;
                self.status = None;
            }

            if !self.creating_new && self.selected_index.is_some() {
                ui.add_space(20.0);
                let del_btn = egui::Button::new(
                    RichText::new("Delete")
                        .font(FontId::proportional(14.0))
                        .color(Color32::from_rgb(255, 150, 150)),
                )
                .min_size(Vec2::new(80.0, 32.0))
                .fill(Color32::from_rgb(80, 30, 30));

                if ui.add(del_btn).clicked() && !is_busy {
                    if let Some(client) = integration_client {
                        self.pending_delete = Some(client.delete_story(&self.form.story_id));
                    }
                }
            }
        });
    }

    fn poll_pending(&mut self, integration_client: Option<&IntegrationClient>) {
        if let Some(pending) = &self.pending_list {
            if let Some(result) = pending.try_recv() {
                match result {
                    Ok(stories) => {
                        self.status = Some((format!("Loaded {} stories", stories.len()), false));
                        self.stories = stories;
                    }
                    Err(e) => {
                        self.status = Some((format!("Failed to load: {}", e), true));
                    }
                }
                self.pending_list = None;
            }
        }

        if let Some(pending) = &self.pending_save {
            if let Some(result) = pending.try_recv() {
                match result {
                    Ok(_) => {
                        self.status = Some(("Story saved!".to_string(), false));
                        self.creating_new = false;
                        self.selected_index = None;
                        if let Some(client) = integration_client {
                            self.refresh_stories(client);
                        }
                    }
                    Err(e) => {
                        self.status = Some((format!("Save failed: {}", e), true));
                    }
                }
                self.pending_save = None;
            }
        }

        if let Some(pending) = &self.pending_delete {
            if let Some(result) = pending.try_recv() {
                match result {
                    Ok(_) => {
                        self.status = Some(("Story deleted".to_string(), false));
                        self.selected_index = None;
                        self.creating_new = false;
                        if let Some(client) = integration_client {
                            self.refresh_stories(client);
                        }
                    }
                    Err(e) => {
                        self.status = Some((format!("Delete failed: {}", e), true));
                    }
                }
                self.pending_delete = None;
            }
        }
    }
}

// -- UI helpers (same pattern as item_editor) --

fn section_header(ui: &mut Ui, text: &str) {
    ui.label(
        RichText::new(text)
            .font(FontId::proportional(16.0))
            .color(Color32::from_rgb(180, 180, 220)),
    );
    ui.add_space(3.0);
}

fn form_field(ui: &mut Ui, label: &str, value: &mut String) {
    ui.horizontal(|ui| {
        ui.label(format!("{}:", label));
        ui.text_edit_singleline(value);
    });
}

fn combo_field(ui: &mut Ui, label: &str, value: &mut String, options: &[&str]) {
    ui.horizontal(|ui| {
        ui.label(format!("{}:", label));
        egui::ComboBox::from_id_salt(format!("story_{}", label))
            .selected_text(value.as_str())
            .show_ui(ui, |ui| {
                for &opt in options {
                    ui.selectable_value(value, opt.to_string(), opt);
                }
            });
    });
}
