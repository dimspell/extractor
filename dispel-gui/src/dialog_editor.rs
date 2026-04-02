use dispel_core::Dialog;
use dispel_core::Extractor;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct DialogEditorState {
    pub catalog: Option<Vec<Dialog>>,
    pub filtered_dialogs: Vec<(usize, Dialog)>,
    pub selected_idx: Option<usize>,
    pub current_file: String,
    pub dialog_files: Vec<PathBuf>,

    pub edit_id: String,
    pub edit_previous_event_id: String,
    pub edit_next_dialog_to_check: String,
    pub edit_dialog_type: String,
    pub edit_dialog_owner: String,
    pub edit_dialog_id: String,
    pub edit_event_id: String,

    pub status_msg: String,
    pub is_loading: bool,
}

impl DialogEditorState {
    pub fn refresh_dialogs(&mut self) {
        if let Some(catalog) = &self.catalog {
            self.filtered_dialogs = catalog
                .iter()
                .enumerate()
                .map(|(i, r)| (i, r.clone()))
                .collect::<Vec<_>>();
        }
    }

    pub fn select_dialog(&mut self, idx: usize) {
        self.selected_idx = Some(idx);
        if let Some((_, record)) = self.filtered_dialogs.get(idx) {
            self.edit_id = record.id.to_string();
            self.edit_previous_event_id = record
                .previous_event_id
                .map_or("null".into(), |v| v.to_string());
            self.edit_next_dialog_to_check = record
                .next_dialog_to_check
                .map_or("null".into(), |v| v.to_string());
            self.edit_dialog_type = record
                .dialog_type
                .map_or("null".into(), |v| format!("{:?}", v));
            self.edit_dialog_owner = record
                .dialog_owner
                .map_or("null".into(), |v| format!("{:?}", v));
            self.edit_dialog_id = record.dialog_id.map_or("null".into(), |v| v.to_string());
            self.edit_event_id = record.event_id.map_or("null".into(), |v| v.to_string());
        }
    }

    pub fn update_field(&mut self, idx: usize, field: &str, value: String) {
        if let Some(record) = self.filtered_dialogs.get_mut(idx).map(|(_, r)| r) {
            match field {
                "id" => {
                    self.edit_id = value.clone();
                    if let Ok(v) = value.parse() {
                        record.id = v
                    }
                }
                "previous_event_id" => {
                    self.edit_previous_event_id = value.clone();
                    record.previous_event_id = value.parse().ok();
                }
                "next_dialog_to_check" => {
                    self.edit_next_dialog_to_check = value.clone();
                    record.next_dialog_to_check = value.parse().ok();
                }
                "dialog_type" => {
                    self.edit_dialog_type = value.clone();
                    record.dialog_type = if value.contains("Choice") {
                        Some(dispel_core::DialogType::Choice)
                    } else if value.contains("Normal") {
                        Some(dispel_core::DialogType::Normal)
                    } else {
                        None
                    };
                }
                "dialog_owner" => {
                    self.edit_dialog_owner = value.clone();
                    record.dialog_owner = if value.contains("Npc") {
                        Some(dispel_core::DialogOwner::Npc)
                    } else if value.contains("Player") {
                        Some(dispel_core::DialogOwner::Player)
                    } else {
                        None
                    };
                }
                "dialog_id" => {
                    self.edit_dialog_id = value.clone();
                    record.dialog_id = value.parse().ok();
                }
                "event_id" => {
                    self.edit_event_id = value.clone();
                    record.event_id = value.parse().ok();
                }
                _ => {}
            }
            self.refresh_dialogs();
        }
    }

    pub fn save_dialogs(&self, game_path: &str, filename: &str) -> Result<(), String> {
        let path = PathBuf::from(game_path).join(filename);
        if let Some(catalog) = &self.catalog {
            Dialog::save_file(catalog, &path).map_err(|e| format!("Failed to save dialogs: {}", e))
        } else {
            Err("No catalog loaded".to_string())
        }
    }
}
