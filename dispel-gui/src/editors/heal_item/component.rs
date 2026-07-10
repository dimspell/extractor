use crate::components::editable::{EditableRecord, FieldKind};
use dispel_core::{HealItem, HealItemFlag};

use crate::editable_record_fields;

const FLAGS: FieldKind = FieldKind::Enum {
    variants: &["None", "FullRestoration"],
};

editable_record_fields!(HealItem, {
    { name = String / "Name:" },
    { description = TextArea / "Description:" },
    { base_price = Integer / "Base Price:" },
    { padding1 = Integer / "Padding 2:" },
    { padding2 = Integer / "Padding 3:" },
    { health_points = Integer / "HP Restore:" },
    { mana_points = Integer / "MP Restore:" },
    { restore_full_health = Enum(HealItemFlag, Shared(FLAGS)) / "Full HP:" },
    { restore_full_mana = Enum(HealItemFlag, Shared(FLAGS)) / "Full MP:" },
    { poison_heal = Enum(HealItemFlag, Shared(FLAGS)) / "Cure Poison:" },
    { petrif_heal = Enum(HealItemFlag, Shared(FLAGS)) / "Cure Petrify:" },
    { polimorph_heal = Enum(HealItemFlag, Shared(FLAGS)) / "Cure Polymorph:" },
    { padding4 = Integer / "Padding 4:" },
    { padding5 = Integer / "Padding 5:" },
});

impl EditableRecord for HealItem {
    crate::editable_record_delegate!();

    fn validate_field(&self, field: &str, value: &str) -> Option<String> {
        match field {
            "name" | "description" => {
                if value.trim().is_empty() {
                    Some(format!("{field} cannot be empty"))
                } else {
                    None
                }
            }
            "base_price" | "health_points" | "mana_points" => match value.parse::<i32>() {
                Ok(v) if v < 0 => Some(format!("{field} must be non-negative")),
                Err(_) => Some(format!("{field} must be a valid integer")),
                _ => None,
            },
            "restore_full_health"
            | "restore_full_mana"
            | "poison_heal"
            | "petrif_heal"
            | "polimorph_heal" => {
                if HealItemFlag::from_name(value).is_none() {
                    Some(format!("Invalid {field}"))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn list_label(&self) -> String {
        format!(
            "[{}] {} - {}g (HP:{}/MP:{})",
            self.id, self.name, self.base_price, self.health_points, self.mana_points
        )
    }

    fn detail_title() -> &'static str {
        "Heal Item Details"
    }
    fn empty_selection_text() -> &'static str {
        "No heal item selected"
    }
    fn save_button_label() -> &'static str {
        "Save Heal Items"
    }
    fn detail_width() -> f32 {
        320.0
    }
}
