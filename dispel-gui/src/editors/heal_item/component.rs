use crate::components::editable::{EditableRecord, FieldKind};
use dispel_core::{HealItem, HealItemFlag};

use crate::editable_record_fields;

const FLAGS: FieldKind = FieldKind::Enum {
    variants: &["None", "Active"],
};

editable_record_fields!(HealItem, {
    { name = String / "Name:" },
    { description = TextArea / "Description:" },
    { base_price = Integer / "Base Price:" },
    { runtime_item_index_slot = Integer / "Runtime Item Index:" },
    { health_points = Integer / "HP Restore:" },
    { mana_points = Integer / "MP Restore:" },
    { restores_full_health = Enum(HealItemFlag, Shared(FLAGS)) / "Restore Full HP:" },
    { restores_full_mana = Enum(HealItemFlag, Shared(FLAGS)) / "Restore Full MP:" },
    { cures_poison = Enum(HealItemFlag, Shared(FLAGS)) / "Cures Poison:" },
    { cures_petrification = Enum(HealItemFlag, Shared(FLAGS)) / "Cures Petrification:" },
    { cures_polymorph = Enum(HealItemFlag, Shared(FLAGS)) / "Cures Polymorph:" },
    { reserved_trailer = HexString / "Reserved Trailer:" },
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
            "restores_full_health"
            | "restores_full_mana"
            | "cures_poison"
            | "cures_petrification"
            | "cures_polymorph" => {
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
