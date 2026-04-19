use super::editable::{set_int, EditableRecord, FieldDescriptor, FieldKind};
use dispel_core::PartyLevelNpc;

impl EditableRecord for PartyLevelNpc {
    fn field_descriptors() -> &'static [FieldDescriptor] {
        &[
            FieldDescriptor {
                name: "npc_index",
                label: "NPC Index:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "records_count",
                label: "Records:",
                kind: FieldKind::Integer,
            },
        ]
    }

    fn get_field(&self, field: &str) -> String {
        match field {
            "npc_index" => self.npc_index.to_string(),
            "records_count" => self.records.len().to_string(),
            _ => String::new(),
        }
    }

    fn set_field(&mut self, field: &str, value: String) -> bool {
        match field {
            "npc_index" => set_int(&mut self.npc_index, value),
            _ => false,
        }
    }

    fn list_label(&self) -> String {
        format!("[{}] {} records", self.npc_index, self.records.len())
    }

    fn detail_title() -> &'static str {
        "Party Level NPC"
    }
    fn empty_selection_text() -> &'static str {
        "No party level NPC selected"
    }
    fn save_button_label() -> &'static str {
        "Save Party Levels"
    }
    fn detail_width() -> f32 {
        280.0
    }
}
