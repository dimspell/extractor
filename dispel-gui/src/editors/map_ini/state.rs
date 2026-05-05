use crate::generic_editor::GenericEditorState;
use dispel_core::MapIni;

pub type MapIniEditorState = GenericEditorState<MapIni>;

impl MapIniEditorState {
    pub fn refresh_maps(&mut self) {
        self.refresh();
    }

    pub fn select_map(&mut self, idx: usize) {
        self.select(idx);
    }

    pub fn save_maps(&self, game_path: &str) -> Result<(), String> {
        self.save(game_path, "Map.ini")
    }
}
