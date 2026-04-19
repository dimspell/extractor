use super::editable::{fmt_enum, set_enum, set_int, EditableRecord, FieldDescriptor, FieldKind};
use dispel_core::{ItemTypeId, MonsterRef};

const LOOT_TYPES: FieldKind = FieldKind::Enum {
    variants: &["Weapon", "Armor", "Heal", "Misc", "Edit", "Event", "Other"],
};

impl EditableRecord for MonsterRef {
    fn field_descriptors() -> &'static [FieldDescriptor] {
        &[
            FieldDescriptor {
                name: "file_id",
                label: "File ID:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "mon_id",
                label: "Monster ID:",
                kind: FieldKind::Lookup("monster_names"),
            },
            FieldDescriptor {
                name: "pos_x",
                label: "Position X:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "pos_y",
                label: "Position Y:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "padding1",
                label: "Flag 1 (0/1):",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "padding2",
                label: "Flag 2 (0/1):",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "padding3",
                label: "Flag 3 (0):",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "padding4",
                label: "Flag 4 (-1/0/1):",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "event_id",
                label: "Event ID:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "loot1_item_id",
                label: "Loot 1 Item ID:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "loot1_item_type",
                label: "Loot 1 Type:",
                kind: LOOT_TYPES,
            },
            FieldDescriptor {
                name: "padding6",
                label: "Padding 6 (0/255):",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "padding7",
                label: "Padding 7 (0/255):",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "loot2_item_id",
                label: "Loot 2 Item ID:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "loot2_item_type",
                label: "Loot 2 Type:",
                kind: LOOT_TYPES,
            },
            FieldDescriptor {
                name: "padding8",
                label: "Padding 8 (0/255):",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "padding9",
                label: "Padding 9 (0/255):",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "loot3_item_id",
                label: "Loot 3 Item ID:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "loot3_item_type",
                label: "Loot 3 Type:",
                kind: LOOT_TYPES,
            },
            FieldDescriptor {
                name: "padding10",
                label: "Padding 10 (0/255):",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "padding11",
                label: "Padding 11 (0/255):",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "padding12",
                label: "Padding 12 (-1/0/1):",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "padding13",
                label: "Padding 13 (0/1):",
                kind: FieldKind::Integer,
            },
        ]
    }

    fn get_field(&self, field: &str) -> String {
        match field {
            "file_id" => self.file_id.to_string(),
            "mon_id" => self.mon_id.to_string(),
            "pos_x" => self.pos_x.to_string(),
            "pos_y" => self.pos_y.to_string(),
            "padding1" => self.padding1.to_string(),
            "padding2" => self.padding2.to_string(),
            "padding3" => self.padding3.to_string(),
            "padding4" => self.padding4.to_string(),
            "event_id" => self.event_id.to_string(),
            "loot1_item_id" => self.loot1_item_id.to_string(),
            "loot1_item_type" => fmt_enum(&self.loot1_item_type),
            "padding6" => self.padding6.to_string(),
            "padding7" => self.padding7.to_string(),
            "loot2_item_id" => self.loot2_item_id.to_string(),
            "loot2_item_type" => fmt_enum(&self.loot2_item_type),
            "padding8" => self.padding8.to_string(),
            "padding9" => self.padding9.to_string(),
            "loot3_item_id" => self.loot3_item_id.to_string(),
            "loot3_item_type" => fmt_enum(&self.loot3_item_type),
            "padding10" => self.padding10.to_string(),
            "padding11" => self.padding11.to_string(),
            "padding12" => self.padding12.to_string(),
            "padding13" => self.padding13.to_string(),
            _ => String::new(),
        }
    }

    fn set_field(&mut self, field: &str, value: String) -> bool {
        match field {
            "file_id" => set_int(&mut self.file_id, value),
            "mon_id" => set_int(&mut self.mon_id, value),
            "pos_x" => set_int(&mut self.pos_x, value),
            "pos_y" => set_int(&mut self.pos_y, value),
            "padding1" => set_int(&mut self.padding1, value),
            "padding2" => set_int(&mut self.padding2, value),
            "padding3" => set_int(&mut self.padding3, value),
            "padding4" => set_int(&mut self.padding4, value),
            "event_id" => set_int(&mut self.event_id, value),
            "loot1_item_id" => set_int(&mut self.loot1_item_id, value),
            "loot1_item_type" => set_enum(&mut self.loot1_item_type, value, ItemTypeId::from_name),
            "padding6" => set_int(&mut self.padding6, value),
            "padding7" => set_int(&mut self.padding7, value),
            "loot2_item_id" => set_int(&mut self.loot2_item_id, value),
            "loot2_item_type" => set_enum(&mut self.loot2_item_type, value, ItemTypeId::from_name),
            "padding8" => set_int(&mut self.padding8, value),
            "padding9" => set_int(&mut self.padding9, value),
            "loot3_item_id" => set_int(&mut self.loot3_item_id, value),
            "loot3_item_type" => set_enum(&mut self.loot3_item_type, value, ItemTypeId::from_name),
            "padding10" => set_int(&mut self.padding10, value),
            "padding11" => set_int(&mut self.padding11, value),
            "padding12" => set_int(&mut self.padding12, value),
            "padding13" => set_int(&mut self.padding13, value),
            _ => false,
        }
    }

    fn list_label(&self) -> String {
        format!(
            "[{}] File:{} Monster:{} Pos:({},{})",
            self.index, self.file_id, self.mon_id, self.pos_x, self.pos_y
        )
    }

    fn list_label_with_lookups(
        &self,
        lookups: &std::collections::HashMap<String, Vec<(String, String)>>,
    ) -> String {
        let monster_name = lookups
            .get("monster_names")
            .and_then(|entries| {
                entries
                    .iter()
                    .find(|(id, _)| id == &self.mon_id.to_string())
            })
            .map(|(_, name)| name.as_str())
            .unwrap_or("???");
        format!(
            "[{}] File:{} Monster:{} Pos:({},{})",
            self.index, self.file_id, monster_name, self.pos_x, self.pos_y
        )
    }

    fn detail_title() -> &'static str {
        "Monster Placement Details"
    }
    fn empty_selection_text() -> &'static str {
        "No monster placement selected"
    }
    fn save_button_label() -> &'static str {
        "Save Monster Ref"
    }
    fn detail_width() -> f32 {
        380.0
    }
}
