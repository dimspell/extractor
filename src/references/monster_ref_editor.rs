use super::editable::{EditableRecord, FieldDescriptor, FieldKind};
use super::enums::ItemTypeId;
use super::monster_ref::MonsterRef;

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
                kind: FieldKind::Enum {
                    variants: &["Weapon", "Armor", "Heal", "Misc", "Edit", "Event", "Other"],
                },
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
                kind: FieldKind::Enum {
                    variants: &["Weapon", "Armor", "Heal", "Misc", "Edit", "Event", "Other"],
                },
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
                kind: FieldKind::Enum {
                    variants: &["Weapon", "Armor", "Heal", "Misc", "Edit", "Event", "Other"],
                },
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
            "loot1_item_type" => format!("{:?}", self.loot1_item_type),
            "padding6" => self.padding6.to_string(),
            "padding7" => self.padding7.to_string(),
            "loot2_item_id" => self.loot2_item_id.to_string(),
            "loot2_item_type" => format!("{:?}", self.loot2_item_type),
            "padding8" => self.padding8.to_string(),
            "padding9" => self.padding9.to_string(),
            "loot3_item_id" => self.loot3_item_id.to_string(),
            "loot3_item_type" => format!("{:?}", self.loot3_item_type),
            "padding10" => self.padding10.to_string(),
            "padding11" => self.padding11.to_string(),
            "padding12" => self.padding12.to_string(),
            "padding13" => self.padding13.to_string(),
            _ => String::new(),
        }
    }

    fn set_field(&mut self, field: &str, value: String) -> bool {
        match field {
            "file_id" => {
                if let Ok(v) = value.parse() {
                    self.file_id = v;
                    true
                } else {
                    false
                }
            }
            "mon_id" => {
                if let Ok(v) = value.parse() {
                    self.mon_id = v;
                    true
                } else {
                    false
                }
            }
            "pos_x" => {
                if let Ok(v) = value.parse() {
                    self.pos_x = v;
                    true
                } else {
                    false
                }
            }
            "pos_y" => {
                if let Ok(v) = value.parse() {
                    self.pos_y = v;
                    true
                } else {
                    false
                }
            }
            "padding1" => {
                if let Ok(v) = value.parse() {
                    self.padding1 = v;
                    true
                } else {
                    false
                }
            }
            "padding2" => {
                if let Ok(v) = value.parse() {
                    self.padding2 = v;
                    true
                } else {
                    false
                }
            }
            "padding3" => {
                if let Ok(v) = value.parse() {
                    self.padding3 = v;
                    true
                } else {
                    false
                }
            }
            "padding4" => {
                if let Ok(v) = value.parse() {
                    self.padding4 = v;
                    true
                } else {
                    false
                }
            }
            "event_id" => {
                if let Ok(v) = value.parse() {
                    self.event_id = v;
                    true
                } else {
                    false
                }
            }
            "loot1_item_id" => {
                if let Ok(v) = value.parse() {
                    self.loot1_item_id = v;
                    true
                } else {
                    false
                }
            }
            "loot1_item_type" => {
                if let Some(t) = ItemTypeId::from_name(&value) {
                    self.loot1_item_type = t;
                    true
                } else {
                    false
                }
            }
            "padding6" => {
                if let Ok(v) = value.parse() {
                    self.padding6 = v;
                    true
                } else {
                    false
                }
            }
            "padding7" => {
                if let Ok(v) = value.parse() {
                    self.padding7 = v;
                    true
                } else {
                    false
                }
            }
            "loot2_item_id" => {
                if let Ok(v) = value.parse() {
                    self.loot2_item_id = v;
                    true
                } else {
                    false
                }
            }
            "loot2_item_type" => {
                if let Some(t) = ItemTypeId::from_name(&value) {
                    self.loot2_item_type = t;
                    true
                } else {
                    false
                }
            }
            "padding8" => {
                if let Ok(v) = value.parse() {
                    self.padding8 = v;
                    true
                } else {
                    false
                }
            }
            "padding9" => {
                if let Ok(v) = value.parse() {
                    self.padding9 = v;
                    true
                } else {
                    false
                }
            }
            "loot3_item_id" => {
                if let Ok(v) = value.parse() {
                    self.loot3_item_id = v;
                    true
                } else {
                    false
                }
            }
            "loot3_item_type" => {
                if let Some(t) = ItemTypeId::from_name(&value) {
                    self.loot3_item_type = t;
                    true
                } else {
                    false
                }
            }
            "padding10" => {
                if let Ok(v) = value.parse() {
                    self.padding10 = v;
                    true
                } else {
                    false
                }
            }
            "padding11" => {
                if let Ok(v) = value.parse() {
                    self.padding11 = v;
                    true
                } else {
                    false
                }
            }
            "padding12" => {
                if let Ok(v) = value.parse() {
                    self.padding12 = v;
                    true
                } else {
                    false
                }
            }
            "padding13" => {
                if let Ok(v) = value.parse() {
                    self.padding13 = v;
                    true
                } else {
                    false
                }
            }
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
