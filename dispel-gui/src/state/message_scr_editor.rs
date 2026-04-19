use crate::generic_editor::GenericEditorState;
use dispel_core::Message as ScrMessage;

pub type MessageScrEditorState = GenericEditorState<ScrMessage>;

impl MessageScrEditorState {
    pub fn refresh_messages(&mut self) {
        self.refresh();
    }

    pub fn select_message(&mut self, idx: usize) {
        self.select(idx);
    }

    pub fn save_messages(&self, game_path: &str) -> Result<(), String> {
        self.save(game_path, "Message.scr")
    }
}
