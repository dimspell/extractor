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
                name: "variables",
                label: "Variables (name=value):",
                kind: FieldKind::TextArea,
            },
            FieldDescriptor {
                name: "map_content",
                label: "Map Content:",
                kind: FieldKind::TextArea,
            },
            FieldDescriptor {
                name: "chr_content",
                label: "Character Content:",
                kind: FieldKind::TextArea,
            },
            FieldDescriptor {
                name: "npc_content",
                label: "NPC Content:",
                kind: FieldKind::TextArea,
            },
            FieldDescriptor {
                name: "spr_content",
                label: "Sprites (alias(file)):",
                kind: FieldKind::TextArea,
            },
            FieldDescriptor {
                name: "wav_content",
                label: "Sounds:",
                kind: FieldKind::TextArea,
            },
            FieldDescriptor {
                name: "actions",
                label: "Actions:",
                kind: FieldKind::TextArea,
            },
        ]
    }

    fn get_field(&self, field: &str) -> String {
        match field {
            "id" => self.id.to_string(),
            "header_comments" => self.header_comments.join("\n"),
            "variables" => self.variables
                .iter()
                .map(|v| format!("{}={}", v.name, v.value))
                .collect::<Vec<_>>()
                .join("\n"),
            "map_content" => self.map_content.join("\n"),
            "chr_content" => self.chr_content.join("\n"),
            "npc_content" => self.npc_content.join("\n"),
            "spr_content" => self.spr_content
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join("\n"),
            "wav_content" => self.wav_content.join("\n"),
            "actions" => self.actions
                .iter()
                .map(|a| a.to_string())
                .collect::<Vec<_>>()
                .join("\n"),
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
            "variables" => {
                self.variables = value
                    .lines()
                    .filter_map(|line| {
                        let line = line.trim();
                        if line.is_empty() {
                            return None;
                        }
                        let parts: Vec<&str> = line.splitn(2, '=').collect();
                        if parts.len() == 2 {
                            Some(dispel_core::references::event_scr::Variable {
                                name: parts[0].trim().to_string(),
                                value: parts[1].trim().to_string(),
                            })
                        } else {
                            None
                        }
                    })
                    .collect();
                true
            }
            "map_content" => {
                self.map_content = value.lines().map(|s| s.to_string()).collect();
                true
            }
            "chr_content" => {
                self.chr_content = value.lines().map(|s| s.to_string()).collect();
                true
            }
            "npc_content" => {
                self.npc_content = value.lines().map(|s| s.to_string()).collect();
                true
            }
            "spr_content" => {
                self.spr_content = value
                    .lines()
                    .filter_map(|line| {
                        let line = line.trim();
                        if line.is_empty() {
                            return None;
                        }
                        Some(dispel_core::references::event_scr::SpriteDefinition::parse(line))
                    })
                    .collect();
                true
            }
            "wav_content" => {
                self.wav_content = value.lines().map(|s| s.to_string()).collect();
                true
            }
            "actions" => {
                self.actions = value
                    .lines()
                    .filter_map(|line| {
                        let line = line.trim();
                        if line.is_empty() {
                            return None;
                        }
                        Some(dispel_core::references::event_scr::ActionFunction::parse(line))
                    })
                    .collect();
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
        500.0
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

    #[test]
    fn test_set_variables() {
        let mut script = EventScript::default();
        assert!(script.set_field("variables", "spawn=5\nhealth=100".to_string()));
        assert_eq!(script.variables.len(), 2);
        assert_eq!(script.variables[0].name, "spawn");
        assert_eq!(script.variables[0].value, "5");
    }

    #[test]
    fn test_set_actions() {
        let mut script = EventScript::default();
        assert!(script.set_field("actions", "do_action(1)\nif(cond)".to_string()));
        assert_eq!(script.actions.len(), 2);
    }
}
