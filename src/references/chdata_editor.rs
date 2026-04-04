use super::chdata_db::ChData;
use super::editable::{EditableRecord, FieldDescriptor, FieldKind};

impl EditableRecord for ChData {
    fn field_descriptors() -> &'static [FieldDescriptor] {
        // ChData has a unique structure: a magic string, arrays of values/counts, and a total
        &[
            FieldDescriptor {
                name: "magic",
                label: "Magic:",
                kind: FieldKind::String,
            },
            FieldDescriptor {
                name: "total",
                label: "Total:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "values",
                label: "Values (comma-sep):",
                kind: FieldKind::String,
            },
            FieldDescriptor {
                name: "counts",
                label: "Counts (comma-sep):",
                kind: FieldKind::String,
            },
        ]
    }

    fn get_field(&self, field: &str) -> String {
        match field {
            "magic" => self.magic.clone(),
            "total" => self.total.to_string(),
            "values" => self
                .values
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(", "),
            "counts" => self
                .counts
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join(", "),
            _ => String::new(),
        }
    }

    fn set_field(&mut self, field: &str, value: String) -> bool {
        match field {
            "magic" => {
                self.magic = value;
                true
            }
            "total" => {
                if let Ok(v) = value.parse() {
                    self.total = v;
                    true
                } else {
                    false
                }
            }
            "values" => {
                self.values = value
                    .split(',')
                    .filter_map(|s| s.trim().parse().ok())
                    .collect();
                true
            }
            "counts" => {
                self.counts = value
                    .split(',')
                    .filter_map(|s| s.trim().parse().ok())
                    .collect();
                true
            }
            _ => false,
        }
    }

    fn list_label(&self) -> String {
        format!("[{}] {} ({} entries)", 0, self.magic, self.values.len())
    }

    fn detail_title() -> &'static str {
        "Character Data"
    }
    fn empty_selection_text() -> &'static str {
        "No character data loaded"
    }
    fn save_button_label() -> &'static str {
        "Save ChData"
    }
    fn detail_width() -> f32 {
        380.0
    }
}
