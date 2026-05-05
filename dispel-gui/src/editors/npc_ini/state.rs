use crate::generic_editor::GenericEditorState;
use dispel_core::NpcIni;

pub type NpcIniEditorState = GenericEditorState<NpcIni>;

impl NpcIniEditorState {
    pub fn refresh_npcs(&mut self) {
        self.refresh();
    }

    pub fn select_npc(&mut self, idx: usize) {
        self.select(idx);
    }

    pub fn save_npcs(&self, game_path: &str) -> Result<(), String> {
        self.save(game_path, "Npc.ini")
    }
}
