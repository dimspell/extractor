use crate::generic_editor::GenericEditorState;
use dispel_core::Quest;

pub type QuestScrEditorState = GenericEditorState<Quest>;

impl QuestScrEditorState {
    pub fn refresh_quests(&mut self) {
        self.refresh();
    }

    pub fn select_quest(&mut self, idx: usize) {
        self.select(idx);
    }

    pub fn update_description(&mut self, idx: usize, value: String) {
        self.update_field(idx, "description", value);
    }

    pub fn save_quests(&self, game_path: &str) -> Result<(), String> {
        self.save(game_path, "Quest.scr")
    }
}
