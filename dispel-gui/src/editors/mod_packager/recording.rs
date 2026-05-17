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

use dispel_core::modding::{make_delta, ChangeAction, ChangeOp, Value, Workspace};
use iced::Task;

use crate::app::App;
use crate::components::editable::EditableRecord;
use crate::components::generic_editor::MultiFileEditorState;
use crate::editors::mod_packager::ModPackagerMessage;
use crate::message::{Message, MessageExt};
use crate::state::RecordingKey;

/// Idle time after the last keystroke before a pending edit is persisted.
pub const DEBOUNCE: std::time::Duration = std::time::Duration::from_millis(800);

/// If the bsdiff delta is at least this fraction of the full file size, we
/// emit `FileReplace` instead of `BinaryDelta` — the patch is no longer a
/// space win and a full replace is simpler to inspect.
const DELTA_KEEP_THRESHOLD: f64 = 0.7;

/// Decide between [`ChangeOp::BinaryDelta`] and [`ChangeOp::FileReplace`]
/// based on the relative size of the qbsdiff patch.
pub fn decide_op(vanilla: &[u8], current: &[u8]) -> Result<ChangeOp, String> {
    let delta = make_delta(vanilla, current).map_err(|e| e.to_string())?;
    let keep_delta =
        !current.is_empty() && (delta.len() as f64) < (current.len() as f64) * DELTA_KEEP_THRESHOLD;
    if keep_delta {
        Ok(ChangeOp::BinaryDelta { patch_bytes: delta })
    } else {
        Ok(ChangeOp::FileReplace {
            content: current.to_vec(),
        })
    }
}

/// Record a file replacement into the active mod session.
///
/// Ensures a vanilla snapshot exists (reads original from disk), computes a
/// binary delta or full replace, and appends the [`ChangeAction`] to the
/// workspace changelog. Does **not** write `new_bytes` to disk — callers
/// should do that separately after this succeeds.
pub fn record_file_replace(
    workspace_root: &std::path::Path,
    game_dir: &std::path::Path,
    mod_slug: &str,
    relative_path: &str,
    new_bytes: &[u8],
) -> Result<(), String> {
    let ws = Workspace::open(workspace_root.to_path_buf()).map_err(|e| e.to_string())?;
    let vanilla_bytes = ws
        .vanilla()
        .ensure_snapshot(game_dir, relative_path)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Vanilla file not present on disk; cannot diff.".to_string())?;

    let op = decide_op(&vanilla_bytes, new_bytes)?;
    let action = ChangeAction::new(relative_path, op);
    ws.append_action(mod_slug, action)
        .map_err(|e| e.to_string())
}

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
///
/// The `shared_game_path` is used to convert absolute file paths to relative paths.
pub fn capture_field_recording_context<R: EditableRecord>(
    editor: Option<&MultiFileEditorState<R>>,
    index: usize,
    field: &str,
    shared_game_path: &str,
) -> Option<(String, u32, String)> {
    let editor = editor?;
    let (orig_idx, record) = editor.editor.filtered.get(index)?;
    let absolute_path = editor.current_file.as_ref()?;
    let relative_path = if shared_game_path.is_empty() {
        absolute_path.to_string_lossy().into_owned()
    } else {
        let game_path = std::path::Path::new(shared_game_path);
        absolute_path
            .strip_prefix(game_path)
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|_| absolute_path.to_string_lossy().into_owned())
    };
    Some((record.get_field(field), *orig_idx as u32, relative_path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::editable::{EditableRecord, FieldDescriptor};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
    struct TestRecord {
        name: String,
    }

    impl EditableRecord for TestRecord {
        fn field_descriptors() -> &'static [FieldDescriptor] {
            &[]
        }

        fn get_field(&self, _field: &str) -> String {
            self.name.clone()
        }

        fn set_field(&mut self, _field: &str, _value: String) -> bool {
            false
        }

        fn list_label(&self) -> String {
            self.name.clone()
        }

        fn detail_title() -> &'static str {
            "Test"
        }

        fn empty_selection_text() -> &'static str {
            "No selection"
        }

        fn save_button_label() -> &'static str {
            "Save"
        }
    }

    #[test]
    fn converts_absolute_to_relative_path() {
        let mut editor = MultiFileEditorState::<TestRecord> {
            current_file: Some(std::path::PathBuf::from("/game/MonsterInGame/Mondun01.ref")),
            ..Default::default()
        };
        editor.editor.catalog = Some(vec![TestRecord {
            name: "test".to_string(),
        }]);
        editor.editor.filtered = vec![(
            0,
            TestRecord {
                name: "test".to_string(),
            },
        )];

        let result = capture_field_recording_context(Some(&editor), 0, "name", "/game");

        assert!(result.is_some());
        let (_, _, file_path) = result.unwrap();
        assert_eq!(file_path, "MonsterInGame/Mondun01.ref");
    }

    #[test]
    fn preserves_absolute_when_shared_game_path_empty() {
        let mut editor = MultiFileEditorState::<TestRecord> {
            current_file: Some(std::path::PathBuf::from("/game/MonsterInGame/Mondun01.ref")),
            ..Default::default()
        };
        editor.editor.catalog = Some(vec![TestRecord {
            name: "test".to_string(),
        }]);
        editor.editor.filtered = vec![(
            0,
            TestRecord {
                name: "test".to_string(),
            },
        )];

        let result = capture_field_recording_context(Some(&editor), 0, "name", "");

        assert!(result.is_some());
        let (_, _, file_path) = result.unwrap();
        assert_eq!(file_path, "/game/MonsterInGame/Mondun01.ref");
    }

    #[test]
    fn falls_back_to_absolute_when_not_prefix() {
        let mut editor = MultiFileEditorState::<TestRecord> {
            current_file: Some(std::path::PathBuf::from("/some/other/path/Mondun01.ref")),
            ..Default::default()
        };
        editor.editor.catalog = Some(vec![TestRecord {
            name: "test".to_string(),
        }]);
        editor.editor.filtered = vec![(
            0,
            TestRecord {
                name: "test".to_string(),
            },
        )];

        let result = capture_field_recording_context(Some(&editor), 0, "name", "/game");

        assert!(result.is_some());
        let (_, _, file_path) = result.unwrap();
        assert_eq!(file_path, "/some/other/path/Mondun01.ref");
    }
}
