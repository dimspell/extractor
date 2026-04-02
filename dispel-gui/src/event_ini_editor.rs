use dispel_core::Event;
use dispel_core::Extractor;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct EventIniEditorState {
    pub catalog: Option<Vec<Event>>,
    pub filtered_events: Vec<(usize, Event)>,
    pub selected_idx: Option<usize>,

    pub edit_event_id: String,
    pub edit_previous_event_id: String,
    pub edit_event_type: String,
    pub edit_event_filename: String,
    pub edit_counter: String,

    pub status_msg: String,
    pub is_loading: bool,
}

impl EventIniEditorState {
    pub fn refresh_events(&mut self) {
        if let Some(catalog) = &self.catalog {
            self.filtered_events = catalog
                .iter()
                .enumerate()
                .map(|(i, r)| (i, r.clone()))
                .collect::<Vec<_>>();
        }
    }

    pub fn select_event(&mut self, idx: usize) {
        self.selected_idx = Some(idx);
        if let Some((_, record)) = self.filtered_events.get(idx) {
            self.edit_event_id = record.event_id.to_string();
            self.edit_previous_event_id = record.previous_event_id.to_string();
            self.edit_event_type = format!("{:?}", record.event_type);
            self.edit_event_filename = record.event_filename.clone().unwrap_or_default();
            self.edit_counter = record.counter.to_string();
        }
    }

    pub fn update_field(&mut self, idx: usize, field: &str, value: String) {
        if let Some(record) = self.filtered_events.get_mut(idx).map(|(_, r)| r) {
            match field {
                "event_id" => {
                    self.edit_event_id = value.clone();
                    if let Ok(v) = value.parse() {
                        record.event_id = v
                    }
                }
                "previous_event_id" => {
                    self.edit_previous_event_id = value.clone();
                    if let Ok(v) = value.parse() {
                        record.previous_event_id = v
                    }
                }
                "event_type" => {
                    self.edit_event_type = value.clone();
                    record.event_type = if value.contains("Conditional") {
                        dispel_core::EventType::Conditional
                    } else if value.contains("ContinueOnUnsatisfied") {
                        dispel_core::EventType::ContinueOnUnsatisfied
                    } else if value.contains("ExecuteOnSatisfied") {
                        dispel_core::EventType::ExecuteOnSatisfied
                    } else {
                        dispel_core::EventType::Unknown
                    };
                }
                "event_filename" => {
                    self.edit_event_filename = value.clone();
                    record.event_filename = if value.is_empty() { None } else { Some(value) };
                }
                "counter" => {
                    self.edit_counter = value.clone();
                    if let Ok(v) = value.parse() {
                        record.counter = v
                    }
                }
                _ => {}
            }
            self.refresh_events();
        }
    }

    pub fn save_events(&self, game_path: &str) -> Result<(), String> {
        let path = PathBuf::from(game_path).join("Event.ini");
        if let Some(catalog) = &self.catalog {
            Event::save_file(catalog, &path).map_err(|e| format!("Failed to save events: {}", e))
        } else {
            Err("No catalog loaded".to_string())
        }
    }
}
