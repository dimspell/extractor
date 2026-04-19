use crate::generic_editor::GenericEditorState;
use dispel_core::MonsterIni;

pub type MonsterIniEditorState = GenericEditorState<MonsterIni>;

impl MonsterIniEditorState {
    pub fn refresh_monsters(&mut self) {
        self.refresh();
    }

    pub fn select_monster(&mut self, idx: usize) {
        self.select(idx);
    }

    pub fn save_monsters(&self, game_path: &str) -> Result<(), String> {
        self.save(game_path, "Monster.ini")
    }
}
