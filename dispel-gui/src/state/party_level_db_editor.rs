use crate::edit_history::EditHistory;
use crate::loading_state::LoadingState;
use dispel_core::{Extractor, PartyLevelNpc};
use std::path::PathBuf;

#[derive(Clone, Debug, Default)]
pub struct PartyLevelDbEditorState {
    pub catalog: Option<Vec<PartyLevelNpc>>,
    pub selected_npc_idx: Option<usize>,
    pub status_msg: String,
    pub loading_state: LoadingState<()>,
    pub edit_history: EditHistory,
}

impl PartyLevelDbEditorState {
    pub fn selected_npc(&self) -> Option<&PartyLevelNpc> {
        self.selected_npc_idx
            .and_then(|idx| self.catalog.as_ref()?.get(idx))
    }

    pub fn edit_history(&self) -> &EditHistory {
        &self.edit_history
    }

    pub fn save_levels(&self, game_path: &str) -> Result<(), String> {
        let Some(catalog) = &self.catalog else {
            return Err("No catalog loaded".to_string());
        };
        let path = PathBuf::from(game_path)
            .join("NpcInGame")
            .join("PrtLevel.db");
        if path.exists() {
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let backup = path.with_extension(format!("db.{}.bak", ts));
            std::fs::copy(&path, &backup)
                .map_err(|e| format!("Failed to create backup: {}", e))?;
        }
        PartyLevelNpc::save_file(catalog, &path).map_err(|e| format!("Failed to save: {}", e))
    }
}
