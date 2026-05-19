use std::sync::Arc;

use iced::Task;

use super::message::HexEditorMessage;
use super::state::HexEditorState;

/// Callback invoked when the user presses the save button.
/// Receives the editor state and returns a task that eventually produces
/// a `SavedIntoRecording(result)` message.
pub type OnSaveFn = Arc<dyn Fn(&HexEditorState) -> Task<HexEditorMessage> + Send + Sync>;

/// External configuration injected into the hex editor by the host application.
#[derive(Default)]
pub struct HexEditorConfig {
    /// Optional save-to-mod callback. `None` hides the save button.
    pub on_save: Option<OnSaveFn>,
    /// Label for the save button (e.g. "Save into `my-mod`").
    pub save_label: String,
    /// True when all prerequisites for saving are met (mod session active,
    /// file is inside game directory, etc.).
    pub can_save: bool,
    /// Contextual hint shown next to the save button explaining why it's disabled
    /// (e.g. "no recording active", "set a game directory").
    pub save_hint: String,
    /// Additional inspector entries from scripts or host-specific decoders.
    pub extra_entries: Vec<super::inspector::InspectorEntry>,
}

impl HexEditorConfig {
    pub fn save_label(&self) -> &str {
        if self.save_label.is_empty() {
            "Save"
        } else {
            &self.save_label
        }
    }

    pub fn has_save(&self) -> bool {
        self.on_save.is_some()
    }

    pub fn can_save_now(&self, state: &HexEditorState) -> bool {
        self.can_save && state.provider.dirty_count() > 0
    }
}
