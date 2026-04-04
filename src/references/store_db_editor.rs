use super::editable::{EditableRecord, FieldDescriptor, FieldKind};
use super::store_db::Store;

impl EditableRecord for Store {
    fn field_descriptors() -> &'static [FieldDescriptor] {
        &[
            FieldDescriptor {
                name: "index",
                label: "Index:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "store_name",
                label: "Store Name:",
                kind: FieldKind::String,
            },
            FieldDescriptor {
                name: "inn_night_cost",
                label: "Inn Cost:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "some_unknown_number",
                label: "Unknown Number:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "invitation",
                label: "Invitation:",
                kind: FieldKind::String,
            },
            FieldDescriptor {
                name: "haggle_success",
                label: "Haggle Success:",
                kind: FieldKind::String,
            },
            FieldDescriptor {
                name: "haggle_fail",
                label: "Haggle Fail:",
                kind: FieldKind::String,
            },
        ]
    }

    fn get_field(&self, field: &str) -> String {
        match field {
            "index" => self.index.to_string(),
            "store_name" => self.store_name.clone(),
            "inn_night_cost" => self.inn_night_cost.to_string(),
            "some_unknown_number" => self.some_unknown_number.to_string(),
            "invitation" => self.invitation.clone(),
            "haggle_success" => self.haggle_success.clone(),
            "haggle_fail" => self.haggle_fail.clone(),
            _ => String::new(),
        }
    }

    fn set_field(&mut self, field: &str, value: String) -> bool {
        match field {
            "index" => {
                if let Ok(v) = value.parse() {
                    self.index = v;
                    true
                } else {
                    false
                }
            }
            "store_name" => {
                self.store_name = value;
                true
            }
            "inn_night_cost" => {
                if let Ok(v) = value.parse() {
                    self.inn_night_cost = v;
                    true
                } else {
                    false
                }
            }
            "some_unknown_number" => {
                if let Ok(v) = value.parse() {
                    self.some_unknown_number = v;
                    true
                } else {
                    false
                }
            }
            "invitation" => {
                self.invitation = value;
                true
            }
            "haggle_success" => {
                self.haggle_success = value;
                true
            }
            "haggle_fail" => {
                self.haggle_fail = value;
                true
            }
            _ => false,
        }
    }

    fn list_label(&self) -> String {
        format!("[{}] {}", self.index, self.store_name)
    }

    fn detail_title() -> &'static str {
        "Store Details"
    }

    fn empty_selection_text() -> &'static str {
        "No store selected"
    }

    fn save_button_label() -> &'static str {
        "Save Store"
    }

    fn detail_width() -> f32 {
        340.0
    }
}
