use crate::generic_editor::GenericEditorState;
use dispel_core::PartyIniNpc;

pub type PartyIniEditorState = GenericEditorState<PartyIniNpc>;

impl PartyIniEditorState {
    pub fn refresh_npcs(&mut self) {
        self.refresh();
    }

    pub fn select_npc(&mut self, idx: usize) {
        self.select(idx);
    }

    pub fn save_npcs(&self, game_path: &str) -> Result<(), String> {
        self.save(game_path, "NpcInGame/PrtIni.db")
    }
}
