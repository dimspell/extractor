use crate::generic_editor::GenericEditorState;
use dispel_core::Event;

pub type EventIniEditorState = GenericEditorState<Event>;

impl EventIniEditorState {
    pub fn refresh_events(&mut self) {
        self.refresh();
    }

    pub fn select_event(&mut self, idx: usize) {
        self.select(idx);
    }

    pub fn save_events(&self, game_path: &str) -> Result<(), String> {
        self.save(game_path, "Event.ini")
    }
}
