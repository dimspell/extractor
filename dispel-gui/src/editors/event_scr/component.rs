use crate::components::editable::{
    set_int, EditableRecord, FieldDescriptor, FieldKind,
};
use dispel_core::references::event_scr::EventScript;

impl EditableRecord for EventScript {
    fn field_descriptors() -> &'static [FieldDescriptor] {
        &[
            FieldDescriptor {
                name: "id",
                label: "ID:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "header_comments",
                label: "Header Comments:",
                kind: FieldKind::TextArea,
            },
            FieldDescriptor {
                name: "variable_count",
                label: "Variable Count:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "map_content_count",
                label: "Map Content Count:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "chr_content_count",
                label: "Character Content Count:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "npc_content_count",
                label: "NPC Content Count:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "spr_content_count",
                label: "Sprite Count:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "wav_content_count",
                label: "Sound Count:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "action_count",
                label: "Action Count:",
                kind: FieldKind::Integer,
            },
        ]
    }

    fn get_field(&self, field: &str) -> String {
        match field {
            "id" => self.id.to_string(),
            "header_comments" => self.header_comments.join("\n"),
            "variable_count" => self.variables.len().to_string(),
            "map_content_count" => self.map_content.len().to_string(),
            "chr_content_count" => self.chr_content.len().to_string(),
            "npc_content_count" => self.npc_content.len().to_string(),
            "spr_content_count" => self.spr_content.len().to_string(),
            "wav_content_count" => self.wav_content.len().to_string(),
            "action_count" => self.actions.len().to_string(),
            _ => String::new(),
        }
    }

    fn set_field(&mut self, field: &str, value: String) -> bool {
        match field {
            "id" => set_int(&mut self.id, value),
            "header_comments" => {
                self.header_comments = value.lines().map(|s| s.to_string()).collect();
                true
            }
            _ => false,
        }
    }

    fn list_label(&self) -> String {
        format!("EventScript [{}]", self.id)
    }

    fn detail_title() -> &'static str {
        "EventScript Details"
    }

    fn empty_selection_text() -> &'static str {
        "No EventScript selected"
    }

    fn save_button_label() -> &'static str {
        "Save EventScript"
    }

    fn detail_width() -> f32 {
        360.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dispel_core::references::event_scr::EventScript;

    #[test]
    fn test_field_descriptors() {
        let descriptors = EventScript::field_descriptors();
        assert!(!descriptors.is_empty());
        assert_eq!(descriptors[0].name, "id");
    }

    #[test]
    fn test_get_field() {
        let mut script = EventScript::default();
        script.id = 5;
        assert_eq!(script.get_field("id"), "5");
    }

    #[test]
    fn test_set_field() {
        let mut script = EventScript::default();
        assert!(script.set_field("id", "10".to_string()));
        assert_eq!(script.id, 10);
    }

    #[test]
    fn test_list_label() {
        let script = EventScript { id: 3, ..Default::default() };
        assert_eq!(script.list_label(), "EventScript [3]");
    }
}
