use crate::generic_editor::GenericEditorState;
use dispel_core::DrawItem;

pub type DrawItemEditorState = GenericEditorState<DrawItem>;

impl DrawItemEditorState {
    pub fn refresh_items(&mut self) {
        self.refresh();
    }

    pub fn select_item(&mut self, idx: usize) {
        self.select(idx);
    }

    pub fn save_items(&self, game_path: &str) -> Result<(), String> {
        self.save(game_path, "DRAWITEM.ref")
    }
}
