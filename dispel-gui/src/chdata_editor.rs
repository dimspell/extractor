use crate::generic_editor::GenericEditorState;
use dispel_core::ChData;

pub type ChDataEditorState = GenericEditorState<ChData>;

impl ChDataEditorState {
    pub fn select_data(&mut self, idx: usize) {
        self.select(idx);
    }

    pub fn save_data(&self, game_path: &str) -> Result<(), String> {
        self.save(game_path, "ChData.db")
    }
}
