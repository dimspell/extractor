use super::editable::{set_int, set_str, EditableRecord, FieldDescriptor, FieldKind};
use dispel_core::Store;

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
            "index" => set_int(&mut self.index, value),
            "store_name" => set_str(&mut self.store_name, value),
            "inn_night_cost" => set_int(&mut self.inn_night_cost, value),
            "some_unknown_number" => set_int(&mut self.some_unknown_number, value),
            "invitation" => set_str(&mut self.invitation, value),
            "haggle_success" => set_str(&mut self.haggle_success, value),
            "haggle_fail" => set_str(&mut self.haggle_fail, value),
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
