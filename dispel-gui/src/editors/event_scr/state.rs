use crate::components::generic_editor::GenericEditorState;
use dispel_core::references::event_scr::EventScript;

pub type EventScriptEditorState = GenericEditorState<EventScript>;

impl EventScriptEditorState {
    pub fn current_section(&self) -> String {
        self.catalog
            .as_ref()
            .and_then(|c| c.first())
            .map(|_| "VAR".to_string())
            .unwrap_or_default()
    }

    pub fn set_current_section(&mut self, section: String) {
        // In a real implementation, this would switch which section's data is edited
        let _ = section;
    }
}
