//! Recording-mode glue between catalog editors and the Mod Manager.
//!
//! The `define_standard_editor!` macro calls [`observe_field_change`] right
//! after a successful `FieldChanged` edit. If a recording session is active,
//! we dispatch a [`ModPackagerMessage::RecordingObserved`] message carrying
//! just the raw `(file, record_id, field, old, new)` tuple. The Mod Manager
//! handler then debounces those observations per key and only persists the
//! coalesced result after an idle interval has elapsed.
//!
//! When no session is active, `observe_field_change` is a no-op returning
//! `Task::none()` — callers pay only an `Option::is_some()` check.

use dispel_core::modding::Value;
use iced::Task;

use crate::app::App;
use crate::components::editable::EditableRecord;
use crate::components::generic_editor::MultiFileEditorState;
use crate::editors::mod_packager::ModPackagerMessage;
use crate::message::{Message, MessageExt};
use crate::state::RecordingKey;

/// Idle time after the last keystroke before a pending edit is persisted.
pub const DEBOUNCE: std::time::Duration = std::time::Duration::from_millis(800);

/// Observe one successful field-edit. Called from the editor macro after the
/// underlying edit committed; old/new are the string values from the
/// edit-history entry.
pub fn observe_field_change(
    app: &App,
    file_path: impl Into<String>,
    record_id: u32,
    field: &str,
    old: String,
    new: String,
) -> Task<Message> {
    if app.state.recording.is_none() {
        return Task::none();
    }
    let key = RecordingKey {
        file_path: file_path.into(),
        record_id,
        field: field.to_owned(),
    };
    Task::done(Message::mod_packager(
        ModPackagerMessage::RecordingObserved(ObservedAction {
            key,
            old: Value::String(old),
            new: Value::String(new),
        }),
    ))
}

/// Payload for [`ModPackagerMessage::RecordingObserved`].
#[derive(Debug, Clone)]
pub struct ObservedAction {
    pub key: RecordingKey,
    pub old: Value,
    pub new: Value,
}

/// Snapshot needed to record a `FieldChanged` against a multi-file editor.
///
/// Reads the *original* record index (not the filtered position) and the
/// current file path, so the eventual `FieldDelta` addresses the right row
/// even when a filter is active. Returns `None` when the row or file isn't
/// present — callers should treat that as "nothing to record" and skip
/// `observe_field_change` entirely (rather than recording a phantom delta
/// against `record_id=0` with an empty path).
pub fn capture_field_recording_context<R: EditableRecord>(
    editor: Option<&MultiFileEditorState<R>>,
    index: usize,
    field: &str,
) -> Option<(String, u32, String)> {
    let editor = editor?;
    let (orig_idx, record) = editor.editor.filtered.get(index)?;
    let file_path = editor.current_file.as_ref()?.to_string_lossy().into_owned();
    Some((record.get_field(field), *orig_idx as u32, file_path))
}
