//! Recording-mode glue between catalog editors and the Mod Manager.
//!
//! The `define_standard_editor!` macro calls [`observe_field_change`] right
//! after a successful `FieldChanged` edit. If a recording session is active,
//! we dispatch a [`ModPackagerMessage::RecordingObserved`] message carrying
//! a domain-level [`dispel_core::modding::FieldChange`] event — one canonical
//! [`dispel_core::modding::FieldKey`] plus the old/new values. The Mod Manager
//! handler then debounces those observations per key and only persists the
//! coalesced result after an idle interval has elapsed.
//!
//! When no session is active, `observe_field_change` is a no-op returning
//! `Task::none()` — callers pay only an `Option::is_some()` check.

use dispel_core::modding::{ChangeOp, FieldChange, FieldKey, Modding, Value, decide_file_op};
use iced::Task;

use crate::app::App;
use crate::components::editable::EditableRecord;
use crate::components::generic_editor::MultiFileEditorState;
use crate::editors::mod_packager::ModPackagerMessage;
use crate::message::{Message, MessageExt};

/// Idle time after the last keystroke before a pending edit is persisted.
pub const DEBOUNCE: std::time::Duration = std::time::Duration::from_millis(800);

/// Decide between [`ChangeOp::BinaryDelta`] and [`ChangeOp::FileReplace`]
/// based on the relative size of the qbsdiff patch. The decision itself is
/// owned by the core facade; this wrapper keeps the GUI call sites stable.
pub fn decide_op(vanilla: &[u8], current: &[u8]) -> Result<ChangeOp, String> {
    Ok(decide_file_op(vanilla, current))
}

/// Shared wording for a changelog that dropped its oldest edits at the cap.
/// Returns `None` when nothing was pruned, so callers can append it verbatim
/// to their success status without changing a clean save.
pub fn prune_warning(pruned: usize) -> Option<String> {
    (pruned > 0).then(|| {
        format!("Warning: the mod changelog is full — {pruned} oldest edit(s) were dropped and are not in the mod.")
    })
}

/// Record a file replacement into the active mod session.
///
/// Ensures a vanilla snapshot exists (reads original from disk), computes a
/// binary delta or full replace, and appends the [`ChangeAction`] to the
/// workspace changelog. Does **not** write `new_bytes` to disk — callers
/// should do that separately after this succeeds. Returns the number of
/// oldest changelog entries dropped at the history cap.
pub fn record_file_replace(
    workspace_root: &std::path::Path,
    game_dir: &std::path::Path,
    mod_slug: &str,
    relative_path: &str,
    new_bytes: &[u8],
) -> Result<usize, String> {
    let pipeline = Modding::open(workspace_root, game_dir).map_err(|e| e.to_string())?;
    pipeline
        .record_file_reporting(mod_slug, relative_path, new_bytes)
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
    let key = FieldKey::new(file_path, record_id, field);
    Task::done(Message::mod_packager(
        ModPackagerMessage::RecordingObserved(FieldChange::new(
            key,
            Value::String(old),
            Value::String(new),
        )),
    ))
}

/// Payload for [`ModPackagerMessage::RecordingObserved`] — a domain event
/// owned by `dispel-core`, not a GUI-specific struct.
pub use dispel_core::modding::FieldChange as ObservedAction;

/// Typed snapshot of the field an editor just changed. Replaces the old
/// untyped `(old_value, record_id, file_path)` triple.
#[derive(Debug, Clone, PartialEq)]
pub struct FieldEditContext {
    /// Value before the edit, stable across keystrokes.
    pub old: String,
    /// Canonical field identity shared with the whole modding pipeline.
    pub key: FieldKey,
}

impl FieldEditContext {
    /// The typed address this recorded edit targets — the same representation
    /// used by localization entries and the hand-written patchers.
    pub fn field_address(&self) -> dispel_core::FieldAddress {
        self.key.field_address()
    }
}

/// Convenience for editor macros: emit a domain field-change event from a
/// captured context and the newly committed value.
pub fn observe_field_edit(app: &App, context: FieldEditContext, new: String) -> Task<Message> {
    observe_field_change(
        app,
        context.key.file_path,
        context.key.record_id,
        &context.key.field,
        context.old,
        new,
    )
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
) -> Option<FieldEditContext> {
    let editor = editor?;
    // `index` is a catalog index, so find the filtered entry by its stored
    // original index rather than treating `index` as a filtered position.
    let (orig_idx, record) = editor.editor.filtered.iter().find(|(i, _)| *i == index)?;
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
    Some(FieldEditContext {
        old: record.get_field(field),
        key: FieldKey::new(relative_path, *orig_idx as u32, field),
    })
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
    fn prune_warning_only_reports_dropped_edits() {
        assert_eq!(prune_warning(0), None);
        let warning = prune_warning(2).expect("a pruned count yields a warning");
        assert!(warning.contains('2'), "warning names the count: {warning}");
        assert!(
            warning.contains("dropped"),
            "warning says edits were dropped: {warning}"
        );
    }

    #[test]
    fn converts_absolute_to_relative_path() {
        let mut editor = MultiFileEditorState::<TestRecord> {
            current_file: Some(std::path::PathBuf::from("/game/MonsterInGame/Mondun01.ref")),
            ..Default::default()
        };
        editor.editor.catalog = Some(vec![
            TestRecord {
                name: "test".to_string(),
            },
            TestRecord {
                name: "other".to_string(),
            },
        ]);
        // Non-identity mapping: filtered position 0 is catalog index 1.
        editor.editor.filtered = vec![
            (
                1,
                TestRecord {
                    name: "other".to_string(),
                },
            ),
            (
                0,
                TestRecord {
                    name: "test".to_string(),
                },
            ),
        ];

        let result = capture_field_recording_context(Some(&editor), 0, "name", "/game");

        let ctx = result.expect("context should be captured");
        assert_eq!(ctx.key.file_path, "MonsterInGame/Mondun01.ref");
        assert_eq!(ctx.key.record_id, 0);
        assert_eq!(ctx.old, "test");
        assert_eq!(ctx.key.field, "name");
    }

    #[test]
    fn capture_context_resolves_catalog_index_not_filtered_position() {
        let mut editor = MultiFileEditorState::<TestRecord> {
            current_file: Some(std::path::PathBuf::from("/game/MonsterInGame/Mondun01.ref")),
            ..Default::default()
        };
        editor.editor.catalog = Some(vec![
            TestRecord {
                name: "alpha".to_string(),
            },
            TestRecord {
                name: "beta".to_string(),
            },
            TestRecord {
                name: "gamma".to_string(),
            },
        ]);
        // Non-identity mapping: filtered position 0 is catalog index 2.
        editor.editor.filtered = vec![
            (
                2,
                TestRecord {
                    name: "gamma".to_string(),
                },
            ),
            (
                0,
                TestRecord {
                    name: "alpha".to_string(),
                },
            ),
            (
                1,
                TestRecord {
                    name: "beta".to_string(),
                },
            ),
        ];

        // The edit targets catalog index 2, not filtered position 2.
        let ctx = capture_field_recording_context(Some(&editor), 2, "name", "/game")
            .expect("context should be captured");
        assert_eq!(ctx.key.record_id, 2);
        assert_eq!(ctx.old, "gamma");
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

        let ctx = result.expect("context should be captured");
        assert_eq!(ctx.key.file_path, "/game/MonsterInGame/Mondun01.ref");
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

        let ctx = result.expect("context should be captured");
        assert_eq!(ctx.key.file_path, "/some/other/path/Mondun01.ref");
    }
}
