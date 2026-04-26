use super::editable::{set_int, EditableRecord, FieldDescriptor, FieldKind};
use dispel_core::{PartyLevelNpc, PartyLevelRecord};

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

impl EditableRecord for PartyLevelRecord {
    fn field_descriptors() -> &'static [FieldDescriptor] {
        &[
            FieldDescriptor { name: "strength",     label: "Strength:",     kind: FieldKind::Integer },
            FieldDescriptor { name: "constitution", label: "Constitution:", kind: FieldKind::Integer },
            FieldDescriptor { name: "wisdom",       label: "Wisdom:",       kind: FieldKind::Integer },
            FieldDescriptor { name: "health_points",label: "HP:",           kind: FieldKind::Integer },
            FieldDescriptor { name: "mana_points",  label: "MP:",           kind: FieldKind::Integer },
            FieldDescriptor { name: "agility",      label: "Agility:",      kind: FieldKind::Integer },
            FieldDescriptor { name: "attack",       label: "Attack:",       kind: FieldKind::Integer },
            FieldDescriptor { name: "mana_recharge",label: "MP Recharge:",  kind: FieldKind::Integer },
            FieldDescriptor { name: "defense",      label: "Defense:",      kind: FieldKind::Integer },
        ]
    }

    fn get_field(&self, field: &str) -> String {
        match field {
            "strength"      => self.strength.to_string(),
            "constitution"  => self.constitution.to_string(),
            "wisdom"        => self.wisdom.to_string(),
            "health_points" => self.health_points.to_string(),
            "mana_points"   => self.mana_points.to_string(),
            "agility"       => self.agility.to_string(),
            "attack"        => self.attack.to_string(),
            "mana_recharge" => self.mana_recharge.to_string(),
            "defense"       => self.defense.to_string(),
            _ => String::new(),
        }
    }

    fn set_field(&mut self, field: &str, value: String) -> bool {
        match field {
            "strength"      => set_int(&mut self.strength, value),
            "constitution"  => set_int(&mut self.constitution, value),
            "wisdom"        => set_int(&mut self.wisdom, value),
            "health_points" => set_int(&mut self.health_points, value),
            "mana_points"   => set_int(&mut self.mana_points, value),
            "agility"       => set_int(&mut self.agility, value),
            "attack"        => set_int(&mut self.attack, value),
            "mana_recharge" => set_int(&mut self.mana_recharge, value),
            "defense"       => set_int(&mut self.defense, value),
            _ => false,
        }
    }

    fn list_label(&self) -> String {
        format!("Level {}", self.level)
    }

    fn detail_title() -> &'static str { "Level Stats" }
    fn empty_selection_text() -> &'static str { "No level selected" }
    fn save_button_label() -> &'static str { "Save Party Levels" }
    fn detail_width() -> f32 { 280.0 }
}
