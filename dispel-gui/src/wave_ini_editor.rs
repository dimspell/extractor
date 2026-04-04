use crate::generic_editor::GenericEditorState;
use dispel_core::WaveIni;

pub type WaveIniEditorState = GenericEditorState<WaveIni>;

impl WaveIniEditorState {
    pub fn refresh_waves(&mut self) {
        self.refresh();
    }

    pub fn select_wave(&mut self, idx: usize) {
        self.select(idx);
    }

    pub fn save_waves(&self, game_path: &str) -> Result<(), String> {
        self.save(game_path, "Wave.ini")
    }
}
