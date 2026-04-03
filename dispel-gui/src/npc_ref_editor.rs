use dispel_core::Extractor;
use dispel_core::NPC;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct NpcRefEditorState {
    pub catalog: Option<Vec<NPC>>,
    pub filtered_npcs: Vec<(usize, NPC)>,
    pub selected_idx: Option<usize>,
    pub current_map_file: String,
    pub map_files: Vec<PathBuf>,

    pub edit_id: String,
    pub edit_npc_id: String,
    pub edit_name: String,
    pub edit_party_script_id: String,
    pub edit_show_on_event: String,
    pub edit_goto1_filled: String,
    pub edit_goto2_filled: String,
    pub edit_goto3_filled: String,
    pub edit_goto4_filled: String,
    pub edit_goto1_x: String,
    pub edit_goto2_x: String,
    pub edit_goto3_x: String,
    pub edit_goto4_x: String,
    pub edit_goto1_y: String,
    pub edit_goto2_y: String,
    pub edit_goto3_y: String,
    pub edit_goto4_y: String,
    pub edit_looking_direction: String,
    pub edit_dialog_id: String,

    pub status_msg: String,
    pub is_loading: bool,
}

impl NpcRefEditorState {
    pub fn refresh_npcs(&mut self) {
        if let Some(catalog) = &self.catalog {
            self.filtered_npcs = catalog
                .iter()
                .enumerate()
                .map(|(i, r)| (i, r.clone()))
                .collect::<Vec<_>>();
        }
    }

    pub fn select_npc(&mut self, idx: usize) {
        self.selected_idx = Some(idx);
        if let Some((_, record)) = self.filtered_npcs.get(idx) {
            self.edit_id = record.id.to_string();
            self.edit_npc_id = record.npc_id.to_string();
            self.edit_name = record.name.clone();
            self.edit_party_script_id = record.party_script_id.to_string();
            self.edit_show_on_event = record.show_on_event.to_string();
            self.edit_goto1_filled = record.goto1_filled.to_string();
            self.edit_goto2_filled = record.goto2_filled.to_string();
            self.edit_goto3_filled = record.goto3_filled.to_string();
            self.edit_goto4_filled = record.goto4_filled.to_string();
            self.edit_goto1_x = record.goto1_x.to_string();
            self.edit_goto2_x = record.goto2_x.to_string();
            self.edit_goto3_x = record.goto3_x.to_string();
            self.edit_goto4_x = record.goto4_x.to_string();
            self.edit_goto1_y = record.goto1_y.to_string();
            self.edit_goto2_y = record.goto2_y.to_string();
            self.edit_goto3_y = record.goto3_y.to_string();
            self.edit_goto4_y = record.goto4_y.to_string();
            self.edit_looking_direction = format!("{:?}", record.looking_direction);
            self.edit_dialog_id = record.dialog_id.to_string();
        }
    }

    pub fn update_field(&mut self, idx: usize, field: &str, value: String) {
        if let Some(record) = self.filtered_npcs.get_mut(idx).map(|(_, r)| r) {
            match field {
                "id" => {
                    self.edit_id = value.clone();
                    if let Ok(v) = value.parse() {
                        record.id = v
                    }
                }
                "npc_id" => {
                    self.edit_npc_id = value.clone();
                    if let Ok(v) = value.parse() {
                        record.npc_id = v
                    }
                }
                "name" => {
                    self.edit_name = value.clone();
                    record.name = value;
                }
                "party_script_id" => {
                    self.edit_party_script_id = value.clone();
                    if let Ok(v) = value.parse() {
                        record.party_script_id = v
                    }
                }
                "show_on_event" => {
                    self.edit_show_on_event = value.clone();
                    if let Ok(v) = value.parse() {
                        record.show_on_event = v
                    }
                }
                "goto1_filled" => {
                    self.edit_goto1_filled = value.clone();
                    if let Ok(v) = value.parse() {
                        record.goto1_filled = v
                    }
                }
                "goto2_filled" => {
                    self.edit_goto2_filled = value.clone();
                    if let Ok(v) = value.parse() {
                        record.goto2_filled = v
                    }
                }
                "goto3_filled" => {
                    self.edit_goto3_filled = value.clone();
                    if let Ok(v) = value.parse() {
                        record.goto3_filled = v
                    }
                }
                "goto4_filled" => {
                    self.edit_goto4_filled = value.clone();
                    if let Ok(v) = value.parse() {
                        record.goto4_filled = v
                    }
                }
                "goto1_x" => {
                    self.edit_goto1_x = value.clone();
                    if let Ok(v) = value.parse() {
                        record.goto1_x = v
                    }
                }
                "goto2_x" => {
                    self.edit_goto2_x = value.clone();
                    if let Ok(v) = value.parse() {
                        record.goto2_x = v
                    }
                }
                "goto3_x" => {
                    self.edit_goto3_x = value.clone();
                    if let Ok(v) = value.parse() {
                        record.goto3_x = v
                    }
                }
                "goto4_x" => {
                    self.edit_goto4_x = value.clone();
                    if let Ok(v) = value.parse() {
                        record.goto4_x = v
                    }
                }
                "goto1_y" => {
                    self.edit_goto1_y = value.clone();
                    if let Ok(v) = value.parse() {
                        record.goto1_y = v
                    }
                }
                "goto2_y" => {
                    self.edit_goto2_y = value.clone();
                    if let Ok(v) = value.parse() {
                        record.goto2_y = v
                    }
                }
                "goto3_y" => {
                    self.edit_goto3_y = value.clone();
                    if let Ok(v) = value.parse() {
                        record.goto3_y = v
                    }
                }
                "goto4_y" => {
                    self.edit_goto4_y = value.clone();
                    if let Ok(v) = value.parse() {
                        record.goto4_y = v
                    }
                }
                "looking_direction" => {
                    self.edit_looking_direction = value.clone();
                    record.looking_direction = match value.as_str() {
                        "Up" => dispel_core::NpcLookingDirection::Up,
                        "UpRight" => dispel_core::NpcLookingDirection::UpRight,
                        "Right" => dispel_core::NpcLookingDirection::Right,
                        "DownRight" => dispel_core::NpcLookingDirection::DownRight,
                        "Down" => dispel_core::NpcLookingDirection::Down,
                        "DownLeft" => dispel_core::NpcLookingDirection::DownLeft,
                        "Left" => dispel_core::NpcLookingDirection::Left,
                        "UpLeft" => dispel_core::NpcLookingDirection::UpLeft,
                        _ => dispel_core::NpcLookingDirection::Up,
                    };
                }
                "dialog_id" => {
                    self.edit_dialog_id = value.clone();
                    if let Ok(v) = value.parse() {
                        record.dialog_id = v
                    }
                }
                _ => {}
            }
            self.refresh_npcs();
        }
    }

    pub fn save_npcs(&self) -> Result<(), String> {
        if self.current_map_file.is_empty() {
            return Err("No map file selected".to_string());
        }
        let path = PathBuf::from(&self.current_map_file);
        if let Some(catalog) = &self.catalog {
            NPC::save_file(catalog, &path).map_err(|e| format!("Failed to save NPC refs: {}", e))
        } else {
            Err("No catalog loaded".to_string())
        }
    }
}
