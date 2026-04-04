use crate::generic_editor::GenericEditorState;
use dispel_core::Extra;

pub type ExtraIniEditorState = GenericEditorState<Extra>;

impl ExtraIniEditorState {
    pub fn refresh_extras(&mut self) {
        self.refresh();
    }

    pub fn select_extra(&mut self, idx: usize) {
        self.select(idx);
    }

    pub fn save_extras(&self, game_path: &str) -> Result<(), String> {
        self.save(game_path, "Extra.ini")
    }
}
