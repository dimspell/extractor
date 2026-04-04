use crate::generic_editor::GenericEditorState;
use dispel_core::PartyLevelNpc;

pub type PartyLevelDbEditorState = GenericEditorState<PartyLevelNpc>;

impl PartyLevelDbEditorState {
    pub fn select_record(&mut self, idx: usize) {
        self.select(idx);
    }

    pub fn save_levels(&self, game_path: &str) -> Result<(), String> {
        self.save(game_path, "PartyLevel.db")
    }
}
