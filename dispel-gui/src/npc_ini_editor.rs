use crate::edit_history::EditHistory;
use crate::generic_editor::UndoRedo;
use dispel_core::{Extractor, NpcIni};
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct NpcIniEditorState {
    pub catalog: Option<Vec<NpcIni>>,
    pub filtered_npcs: Vec<(usize, NpcIni)>,
    pub selected_idx: Option<usize>,

    pub edit_sprite_filename: String,
    pub edit_description: String,

    pub status_msg: String,
    pub is_loading: bool,
    pub edit_history: EditHistory,
}

impl UndoRedo for NpcIniEditorState {
    fn undo(&mut self) -> Option<String> {
        if let Some(action) = self.edit_history.undo() {
            Some(format!("Undid: {:?}", action))
        } else {
            None
        }
    }

    fn redo(&mut self) -> Option<String> {
        if let Some(action) = self.edit_history.redo() {
            Some(format!("Redid: {:?}", action))
        } else {
            None
        }
    }

    fn can_undo(&self) -> bool {
        self.edit_history.can_undo()
    }

    fn can_redo(&self) -> bool {
        self.edit_history.can_redo()
    }

    fn edit_history(&self) -> &EditHistory {
        &self.edit_history
    }
}

impl NpcIniEditorState {
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
            self.edit_sprite_filename = record.sprite_filename.clone().unwrap_or_default();
            self.edit_description = record.description.clone();
        }
    }

    pub fn update_field(&mut self, idx: usize, field: &str, value: String) {
        if let Some(record) = self.filtered_npcs.get_mut(idx).map(|(_, r)| r) {
            match field {
                "sprite_filename" => {
                    self.edit_sprite_filename = value.clone();
                    record.sprite_filename = if value.is_empty() { None } else { Some(value) };
                }
                "description" => {
                    self.edit_description = value.clone();
                    record.description = value;
                }
                _ => {}
            }
            self.refresh_npcs();
        }
    }

    pub fn save_npcs(&self, game_path: &str) -> Result<(), String> {
        let path = PathBuf::from(game_path).join("Npc.ini");
        if let Some(catalog) = &self.catalog {
            NpcIni::save_file(catalog, &path).map_err(|e| format!("Failed to save NPC ini: {}", e))
        } else {
            Err("No catalog loaded".to_string())
        }
    }
}
