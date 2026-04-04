use crate::generic_editor::GenericEditorState;
use dispel_core::MagicSpell;

pub type MagicEditorState = GenericEditorState<MagicSpell>;

impl MagicEditorState {
    pub fn refresh_spells(&mut self) {
        self.refresh();
    }

    pub fn select_spell(&mut self, idx: usize) {
        self.select(idx);
    }

    pub fn save_spells(&self, game_path: &str) -> Result<(), String> {
        self.save(game_path, "MagicInGame/Magic.db")
    }
}
